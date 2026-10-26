//! Antigravity CLI adapter for local questions and the verified Alexa endpoint.
use super::{Generation, LanguageModel, ProviderError};
use serde::Deserialize;
use std::{path::PathBuf, process::Stdio, time::Duration};
use tokio::{io::AsyncReadExt, process::Command, time::timeout};

mod warm;

const MAX_OUTPUT_BYTES: u64 = 64 * 1024;
const DEADLINE: Duration = Duration::from_secs(35);
const VOICE_METAPROMPT: &str = include_str!("../../prompts/meta-prompt.txt");

pub struct AntigravityProvider {
    executable: PathBuf,
    workspace: PathBuf,
    model: Option<String>,
    standby: Option<std::sync::Arc<warm::Standby>>,
    metaprompt: std::sync::Arc<str>,
}

impl AntigravityProvider {
    pub fn new(executable: PathBuf, workspace: PathBuf) -> Self {
        Self {
            executable,
            workspace,
            model: None,
            standby: None,
            metaprompt: VOICE_METAPROMPT.into(),
        }
    }

    /// Antigravity lists Flash variants as model slugs ending in the thinking level.
    pub fn with_gemini_settings(mut self, model: &str, level: &str) -> Result<Self, String> {
        if !model.starts_with("gemini-")
            || model.len() <= "gemini-".len()
            || model.len() > 128
            || !model
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'-' | b'.'))
        {
            return Err(
                "GEMINI_MODEL debe ser el nombre base del modelo, por ejemplo gemini-3.8-flash"
                    .into(),
            );
        }
        if !matches!(level, "low" | "medium" | "high") {
            return Err("GEMINI_LEVEL debe ser low, medium o high".into());
        }
        self.model = Some(format!("{model}-{level}"));
        Ok(self)
    }

    pub async fn prewarm(mut self) -> Result<Self, ProviderError> {
        let ready = warm::PreparedCli::start(self.command(), self.metaprompt.clone()).await?;
        self.standby = Some(warm::Standby::new(ready));
        Ok(self)
    }

    pub fn with_metaprompt(mut self, prompt: &str) -> Result<Self, String> {
        if prompt.trim().is_empty() || prompt.len() > 16384 || prompt.contains('\0') {
            return Err("Metaprompt inválido".into());
        }
        self.metaprompt = prompt.into();
        Ok(self)
    }

    fn command(&self) -> Command {
        let mut command = Command::new(&self.executable);
        if let Some(model) = &self.model {
            command.args(["--model", model]);
        }
        command
            .current_dir(&self.workspace)
            .args(["--mode", "plan", "--disable-slash-commands"]);
        command
    }

    async fn ask(&self, prompt: &str) -> Result<String, ProviderError> {
        if prompt.trim().is_empty() || prompt.chars().count() > 4000 {
            return Err(ProviderError::InvalidPrompt);
        }
        // No shell, concatenated command, session continuation, or permission bypass.
        // Plan mode is a precaution; it does NOT disable every tool in Antigravity.
        // https://antigravity.google/docs/cli/headless
        if let Some(standby) = &self.standby {
            let ready = standby
                .take(self.command(), self.metaprompt.clone())
                .await?;
            return timeout(DEADLINE, ready.ask(&format!("Pregunta:\n{prompt}")))
                .await
                .map_err(|_| ProviderError::TimedOut)?;
        }
        let mut command = self.command();
        let prompt = format!("{}\nPregunta:\n{prompt}", self.metaprompt);
        command
            .args(["--output-format", "json", "--print-timeout", "30s"])
            .arg("--print")
            .arg(prompt)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true);
        let bytes = run(&mut command, DEADLINE).await?;
        parse_response(&bytes)
    }
}

impl LanguageModel for AntigravityProvider {
    fn generate<'a>(&'a self, prompt: &'a str) -> Generation<'a> {
        Box::pin(self.ask(prompt))
    }
}

async fn run(command: &mut Command, deadline: Duration) -> Result<Vec<u8>, ProviderError> {
    let mut child = command.spawn().map_err(|_| ProviderError::CliUnavailable)?;
    let stdout = child.stdout.take().ok_or(ProviderError::CliFailed)?;
    let result = timeout(deadline, async {
        let mut bytes = Vec::new();
        stdout
            .take(MAX_OUTPUT_BYTES + 1)
            .read_to_end(&mut bytes)
            .await
            .map_err(|_| ProviderError::CliFailed)?;
        if bytes.len() as u64 > MAX_OUTPUT_BYTES {
            return Err(ProviderError::OutputTooLarge);
        }
        let status = child.wait().await.map_err(|_| ProviderError::CliFailed)?;
        if !status.success() {
            return Err(ProviderError::CliFailed);
        }
        Ok(bytes)
    })
    .await;
    // Explicitly kill and reap the direct child on timeout or excessive output.
    // kill_on_drop also covers cancellation by the caller.
    match result {
        Ok(Ok(bytes)) => Ok(bytes),
        other => {
            let _ = child.kill().await;
            match other {
                Ok(Err(error)) => Err(error),
                _ => Err(ProviderError::TimedOut),
            }
        }
    }
}

#[derive(Deserialize)]
struct Reply {
    status: String,
    #[serde(default)]
    response: String,
    error: Option<serde_json::Value>,
}

fn parse_response(bytes: &[u8]) -> Result<String, ProviderError> {
    let reply: Reply = serde_json::from_slice(bytes).map_err(|_| ProviderError::InvalidResponse)?;
    if reply.status != "SUCCESS" || reply.error.is_some() {
        return Err(ProviderError::CliFailed);
    }
    let answer = reply.response.trim();
    if answer.is_empty() {
        return Err(ProviderError::InvalidResponse);
    }
    Ok(answer.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gemini_settings_select_the_cli_model_and_reject_invalid_values() {
        let provider = AntigravityProvider::new("agy".into(), ".".into())
            .with_gemini_settings("gemini-3.8-flash", "low")
            .unwrap();
        assert_eq!(provider.model.as_deref(), Some("gemini-3.8-flash-low"));
        for (model, level) in [
            ("", "low"),
            ("gemini-3.8-flash --help", "low"),
            ("gemini-3.8-flash", "fast"),
        ] {
            assert!(
                AntigravityProvider::new("agy".into(), ".".into())
                    .with_gemini_settings(model, level)
                    .is_err()
            );
        }
    }

    #[test]
    fn accepts_only_successful_nonempty_json_responses() {
        assert_eq!(
            parse_response(br#"{"status":"SUCCESS","response":" Hola "}"#),
            Ok("Hola".into())
        );
        assert_eq!(
            parse_response(br#"{"status":"SUCCESS","response":" "}"#),
            Err(ProviderError::InvalidResponse)
        );
        assert_eq!(
            parse_response(b"not json"),
            Err(ProviderError::InvalidResponse)
        );
        assert_eq!(
            parse_response(br#"{"response":"Hola"}"#),
            Err(ProviderError::InvalidResponse)
        );
        assert_eq!(
            parse_response(br#"{"status":"ERROR","error":"private details"}"#),
            Err(ProviderError::CliFailed)
        );
        assert_eq!(
            parse_response(br#"{"status":"SUCCESS","response":"Hola","error":"failure"}"#),
            Err(ProviderError::CliFailed)
        );
    }

    #[cfg(windows)]
    #[tokio::test]
    async fn kills_cli_on_timeout() {
        let mut command = Command::new("powershell.exe");
        command
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                "Start-Sleep -Seconds 10",
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true);
        assert_eq!(
            run(&mut command, Duration::from_millis(200)).await,
            Err(ProviderError::TimedOut)
        );
    }

    #[cfg(windows)]
    #[tokio::test]
    async fn bounds_output_without_waiting_for_process_exit() {
        let mut command = Command::new("powershell.exe");
        command
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                "[Console]::Write(('a' * 70000)); Start-Sleep -Seconds 10",
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true);
        assert_eq!(
            run(&mut command, Duration::from_secs(5)).await,
            Err(ProviderError::OutputTooLarge)
        );
    }
}
