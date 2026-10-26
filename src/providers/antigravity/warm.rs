//! Keep one unused CLI ready; every question gets a fresh conversation.
use super::{MAX_OUTPUT_BYTES, ProviderError, parse_response};
use serde_json::{Value, json};
use std::{process::Stdio, sync::Arc, time::Duration};
use tokio::{
    io::{AsyncBufRead, AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    process::{Child, ChildStdout, Command},
    sync::{Mutex, Notify},
    time::timeout,
};

pub(super) struct PreparedCli {
    child: Child,
    output: BufReader<ChildStdout>,
}

impl PreparedCli {
    pub(super) async fn start(
        mut command: Command,
        metaprompt: Arc<str>,
    ) -> Result<Self, ProviderError> {
        command
            .args([
                "--input-format",
                "stream-json",
                "--output-format",
                "stream-json",
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true);
        let mut child = command.spawn().map_err(|_| ProviderError::CliUnavailable)?;
        let output = child.stdout.take().ok_or(ProviderError::CliFailed)?;
        let mut ready = Self {
            child,
            output: BufReader::new(output),
        };
        let mut remaining = MAX_OUTPUT_BYTES;
        let init = timeout(
            Duration::from_secs(15),
            read_event(&mut ready.output, &mut remaining),
        )
        .await
        .map_err(|_| ProviderError::TimedOut)??;
        if init["event"] != "init" {
            return Err(ProviderError::InvalidResponse);
        }
        // Process the voice instructions before any user's question arrives.
        let priming =
            format!("{metaprompt}\nConfirma estas instrucciones respondiendo solo: listo.");
        timeout(Duration::from_secs(15), ready.turn(&priming))
            .await
            .map_err(|_| ProviderError::TimedOut)??;
        Ok(ready)
    }

    pub(super) async fn ask(mut self, prompt: &str) -> Result<String, ProviderError> {
        let answer = self.turn(prompt).await;
        // This process is never reused: no question/history crosses users.
        let _ = self.child.kill().await;
        let _ = self.child.wait().await;
        answer
    }

    async fn turn(&mut self, prompt: &str) -> Result<String, ProviderError> {
        let mut input = serde_json::to_vec(&json!({"event":"user","message":{"content":prompt}}))
            .map_err(|_| ProviderError::InvalidPrompt)?;
        input.push(b'\n');
        let stdin = self.child.stdin.as_mut().ok_or(ProviderError::CliFailed)?;
        stdin
            .write_all(&input)
            .await
            .map_err(|_| ProviderError::CliFailed)?;
        stdin.flush().await.map_err(|_| ProviderError::CliFailed)?;
        let mut remaining = MAX_OUTPUT_BYTES;
        loop {
            let event = read_event(&mut self.output, &mut remaining).await?;
            if event["event"] == "result" {
                let bytes = serde_json::to_vec(&event["result"])
                    .map_err(|_| ProviderError::InvalidResponse)?;
                return parse_response(&bytes);
            }
        }
    }
}

async fn read_event<R: AsyncBufRead + Unpin>(
    reader: &mut R,
    remaining: &mut u64,
) -> Result<Value, ProviderError> {
    let mut line = Vec::new();
    let size = reader
        .take(*remaining + 1)
        .read_until(b'\n', &mut line)
        .await
        .map_err(|_| ProviderError::CliFailed)?;
    if size as u64 > *remaining {
        return Err(ProviderError::OutputTooLarge);
    }
    *remaining -= size as u64;
    if size == 0 {
        return Err(ProviderError::CliFailed);
    }
    serde_json::from_slice(&line).map_err(|_| ProviderError::InvalidResponse)
}

pub(super) struct Standby {
    slot: Mutex<Option<Result<PreparedCli, ProviderError>>>,
    changed: Notify,
}

impl Standby {
    pub(super) fn new(ready: PreparedCli) -> Arc<Self> {
        Arc::new(Self {
            slot: Mutex::new(Some(Ok(ready))),
            changed: Notify::new(),
        })
    }

    pub(super) async fn take(
        self: &Arc<Self>,
        replacement: Command,
        metaprompt: Arc<str>,
    ) -> Result<PreparedCli, ProviderError> {
        loop {
            let changed = self.changed.notified();
            let ready = self.slot.lock().await.take();
            if let Some(ready) = ready {
                let pool = self.clone();
                tokio::spawn(async move {
                    let next = PreparedCli::start(replacement, metaprompt).await;
                    *pool.slot.lock().await = Some(next);
                    pool.changed.notify_one();
                });
                return ready;
            }
            changed.await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn prepares_the_metaprompt_before_accepting_the_question() {
        // Real pipes and a deterministic CLI double, without network or credentials.
        let mut command = Command::new("node");
        command.args([
            "-e",
            r#"
            const readline = require('node:readline');
            console.log(JSON.stringify({event:'init'}));
            let prepared = false;
            readline.createInterface({input:process.stdin}).on('line', line => {
                const prompt = JSON.parse(line).message.content;
                let response;
                if (!prepared && prompt.includes('TEST_SYSTEM_RULE')) {
                    prepared = true;
                    response = 'listo';
                } else if (prepared && prompt === 'Pregunta:\nqué es una estrella') {
                    response = 'Una estrella brilla.';
                }
                console.log(JSON.stringify({event:'result',result:{
                    status:response ? 'SUCCESS' : 'ERROR', response:response || ''
                }}));
            });
        "#,
            "--",
        ]);
        let ready = PreparedCli::start(command, "TEST_SYSTEM_RULE".into())
            .await
            .unwrap();
        assert_eq!(
            ready.ask("Pregunta:\nqué es una estrella").await.unwrap(),
            "Una estrella brilla."
        );
    }

    #[tokio::test]
    async fn stream_events_are_bounded_across_all_lines() {
        let bytes = b"{\"event\":\"init\"}\n{\"event\":\"result\"}\n";
        let mut input = &bytes[..];
        let mut budget = 25;
        assert_eq!(
            read_event(&mut input, &mut budget).await.unwrap()["event"],
            "init"
        );
        assert_eq!(
            read_event(&mut input, &mut budget).await.unwrap_err(),
            ProviderError::OutputTooLarge
        );
    }

    #[tokio::test]
    async fn malformed_or_closed_stream_never_becomes_an_answer() {
        for bytes in [&b"not json\n"[..], &b""[..]] {
            let mut input = bytes;
            assert!(read_event(&mut input, &mut 4096).await.is_err());
        }
    }
}
