use crate::{providers::LanguageModel, voice::normalize_for_voice};
use axum::{
    Json,
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
};
use serde_json::{Value, json};
use std::{
    env,
    path::PathBuf,
    process::Stdio,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};
use tokio::{io::AsyncWriteExt, process::Command, sync::Semaphore, time::timeout};

static NEXT_TRACE: AtomicU64 = AtomicU64::new(1);

pub struct AlexaConfig {
    pub skill_id: String,
    pub verifier: PathBuf,
    pub node: PathBuf,
    pub openssl: PathBuf,
}

impl AlexaConfig {
    pub fn from_env() -> Result<Option<Self>, String> {
        let skill_id = match env::var("ALEXA_SKILL_ID") {
            Ok(value) => value,
            Err(env::VarError::NotPresent) => return Ok(None),
            Err(_) => return Err("ALEXA_SKILL_ID invalido".into()),
        };
        if !skill_id.starts_with("amzn1.ask.skill.")
            || skill_id.len() > 256
            || skill_id.chars().any(char::is_whitespace)
        {
            return Err("ALEXA_SKILL_ID invalido".into());
        }
        let verifier = env::var_os("ALEXA_VERIFIER_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("tools/alexa-verifier/verify.cjs"));
        let verifier = verifier
            .canonicalize()
            .map_err(|_| "No se encuentra verify.cjs")?;
        Ok(Some(Self {
            skill_id,
            verifier,
            node: env::var_os("ALEXA_NODE_BIN")
                .map(PathBuf::from)
                .unwrap_or_else(|| "node".into()),
            openssl: env::var_os("ALEXA_OPENSSL_BIN")
                .map(PathBuf::from)
                .unwrap_or_else(|| "openssl".into()),
        }))
    }

    pub async fn check(&self) -> Result<(), String> {
        let result = timeout(
            Duration::from_secs(5),
            Command::new(&self.node)
                .arg(&self.verifier)
                .arg("--check")
                .arg(&self.openssl)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .kill_on_drop(true)
                .status(),
        )
        .await;
        match result {
            Ok(Ok(status)) if status.success() => Ok(()),
            _ => Err("No se pudo iniciar el verificador: revisa Node y ALEXA_OPENSSL_BIN".into()),
        }
    }
}

pub struct AlexaService {
    config: AlexaConfig,
    model: Arc<dyn LanguageModel>,
    verification_slots: Semaphore,
    generation_slot: Semaphore,
}

impl AlexaService {
    pub fn new(config: AlexaConfig, model: Arc<dyn LanguageModel>) -> Self {
        Self {
            config,
            model,
            verification_slots: Semaphore::new(2),
            generation_slot: Semaphore::new(1),
        }
    }

    async fn verify(&self, headers: &HeaderMap, body: &str) -> Result<(), ()> {
        // Reject absent/duplicate/oversized signature headers before spawning anything.
        let mut selected = serde_json::Map::new();
        for name in ["signature-256", "signaturecertchainurl"] {
            if headers.get_all(name).iter().count() != 1 {
                return Err(());
            }
            let value = headers
                .get(name)
                .and_then(|v| v.to_str().ok())
                .filter(|v| !v.is_empty() && v.len() <= 2048)
                .ok_or(())?;
            selected.insert(name.into(), Value::String(value.into()));
        }
        let payload = serde_json::to_vec(&json!({
            "skillId": self.config.skill_id, "opensslBinary": self.config.openssl,
            "headers": selected, "body": body
        }))
        .map_err(|_| ())?;
        timeout(Duration::from_secs(5), async {
            let mut child = Command::new(&self.config.node)
                .arg("--max-old-space-size=64")
                .arg(&self.config.verifier)
                .stdin(Stdio::piped())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .kill_on_drop(true)
                .spawn()
                .map_err(|_| ())?;
            let mut stdin = child.stdin.take().ok_or(())?;
            stdin.write_all(&payload).await.map_err(|_| ())?;
            drop(stdin);
            // Only the installed verifier's successful exit authorizes processing.
            if child.wait().await.map_err(|_| ())?.success() {
                Ok(())
            } else {
                Err(())
            }
        })
        .await
        .map_err(|_| ())?
    }

    async fn dispatch(&self, envelope: &Value) -> Value {
        let request = &envelope["request"];
        let trace = NEXT_TRACE.fetch_add(1, Ordering::Relaxed);
        let request_type = match request["type"].as_str() {
            Some("LaunchRequest") => "launch",
            Some("IntentRequest") => "intent",
            Some("SessionEndedRequest") => "session_ended",
            _ => "other",
        };
        let intent = match request["intent"]["name"].as_str() {
            Some("PreguntarIntent") => "question",
            Some("AMAZON.FallbackIntent") => "fallback",
            Some("AMAZON.HelpIntent") => "help",
            Some("AMAZON.StopIntent" | "AMAZON.CancelIntent") => "stop",
            _ => "other",
        };
        let reason = match request["reason"].as_str() {
            Some("ERROR") => "error",
            Some("EXCEEDED_MAX_REPROMPTS") => "no_reply",
            Some("USER_INITIATED") => "user",
            _ => "other",
        };
        eprintln!(
            "{}",
            json!({"event":"alexa_request","trace":trace,"type":request_type,"intent":intent,"end_reason":reason})
        );
        match request["type"].as_str() {
            Some("LaunchRequest") => speech(
                "AlexIA está conectada.",
                false,
            ),
            Some("SessionEndedRequest") => json!({"version":"1.0","response":{}}),
            Some("IntentRequest") => match request["intent"]["name"].as_str() {
                Some("AMAZON.StopIntent" | "AMAZON.CancelIntent") => speech("Hasta luego.", true),
                Some("AMAZON.FallbackIntent") => speech(
                    "No entendí. Di, por ejemplo: pregunta qué es una estrella.",
                    false,
                ),
                Some("AMAZON.HelpIntent") => speech("Di pregunta, seguida de tu pregunta.", false),
                Some("PreguntarIntent") => {
                    let question = request["intent"]["slots"]["pregunta"]["value"]
                        .as_str()
                        .unwrap_or("")
                        .trim();
                    let user = envelope["context"]["System"]["user"]["userId"]
                        .as_str()
                        .unwrap_or("");
                    if user.is_empty()
                        || user.len() > 512
                        || question.is_empty()
                        || question.chars().count() > 4000
                    {
                        return speech(
                            "No pude entender la pregunta. Di pregunta y vuelve a intentarlo.",
                            false,
                        );
                    }
                    self.ask(question, trace).await
                }
                _ => speech("Di, por ejemplo: pregunta qué es una estrella.", false),
            },
            _ => speech("No puedo atender ese tipo de solicitud.", true),
        }
    }

    // This budget includes verification; leave one second for the tunnel and Alexa.
    async fn respond(&self, envelope: &Value, budget: Duration) -> Value {
        match timeout(budget, self.dispatch(envelope)).await {
            Ok(response) => response,
            Err(_) => {
                eprintln!("{}", json!({"event":"gemini_completed","status":"timeout"}));
                speech("La consulta tardó demasiado. Inténtalo de nuevo.", false)
            }
        }
    }

    async fn ask(&self, question: &str, trace: u64) -> Value {
        let Ok(_permit) = self.generation_slot.try_acquire() else {
            return speech(
                "Estoy atendiendo otra consulta. Inténtalo en unos segundos.",
                false,
            );
        };
        let started = Instant::now();
        // Await in this request. Dropping this future cancels the CLI on timeout.
        let generated = self.model.generate(question).await;
        let status = match &generated {
            Ok(_) => "ok",
            Err(crate::providers::ProviderError::CliUnavailable) => "cli_unavailable",
            Err(crate::providers::ProviderError::TimedOut) => "cli_timeout",
            Err(_) => "provider_error",
        };
        eprintln!(
            "{}",
            json!({"event":"gemini_completed","trace":trace,
            "status":status,"duration_ms":started.elapsed().as_millis()})
        );
        let answer = match generated {
            Ok(text) => {
                let text = normalize_for_voice(&text, 800);
                if text.is_empty() { "Gemini no devolvió texto para voz.".into() } else { text }
            }
            Err(_) => "No pude completar la consulta. Comprueba la sesión o la cuota de Antigravity en tu equipo.".into(),
        };
        speech(&answer, true)
    }
}

fn speech(text: &str, end: bool) -> Value {
    let mut response =
        json!({"outputSpeech":{"type":"PlainText","text":text},"shouldEndSession":end});
    if !end {
        response["reprompt"] =
            json!({"outputSpeech":{"type":"PlainText","text":"Di pregunta y tu pregunta."}});
    }
    json!({"version":"1.0","response":response})
}

pub async fn handle(
    State(service): State<Arc<AlexaService>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<Value>, StatusCode> {
    let started = Instant::now();
    let _permit = service
        .verification_slots
        .try_acquire()
        .map_err(|_| StatusCode::TOO_MANY_REQUESTS)?;
    let body = std::str::from_utf8(&body).map_err(|_| StatusCode::BAD_REQUEST)?;
    if body.len() > 65536 {
        return Err(StatusCode::PAYLOAD_TOO_LARGE);
    }
    service.verify(&headers, body).await.map_err(|_| {
        let trace = NEXT_TRACE.fetch_add(1, Ordering::Relaxed);
        eprintln!(
            "{}",
            json!({"event":"alexa_verification_rejected","trace":trace})
        );
        StatusCode::BAD_REQUEST
    })?;
    let envelope: Value = serde_json::from_str(body).map_err(|_| StatusCode::BAD_REQUEST)?;
    drop(_permit);
    let remaining = Duration::from_secs(7).saturating_sub(started.elapsed());
    Ok(Json(service.respond(&envelope, remaining).await))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::{Generation, ProviderError};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tokio::sync::Notify;
    use tower::ServiceExt;

    struct Model {
        calls: AtomicUsize,
        finish: Notify,
    }
    impl LanguageModel for Model {
        fn generate<'a>(&'a self, _: &'a str) -> Generation<'a> {
            Box::pin(async move {
                self.calls.fetch_add(1, Ordering::SeqCst);
                self.finish.notified().await;
                Ok::<_, ProviderError>("**Una estrella** brilla.".into())
            })
        }
    }
    fn setup() -> (Arc<AlexaService>, Arc<Model>) {
        let model = Arc::new(Model {
            calls: AtomicUsize::new(0),
            finish: Notify::new(),
        });
        let service = Arc::new(AlexaService::new(
            AlexaConfig {
                skill_id: "amzn1.ask.skill.test".into(),
                verifier: "missing".into(),
                node: "missing".into(),
                openssl: "missing".into(),
            },
            model.clone(),
        ));
        (service, model)
    }
    fn question(user: &str, text: &str) -> Value {
        json!({"context":{"System":{"user":{"userId":user}}},"request":{"type":"IntentRequest","intent":{"name":"PreguntarIntent","slots":{"pregunta":{"value":text}}}}})
    }

    #[tokio::test]
    async fn question_speaks_the_answer_without_a_second_command() {
        let (service, model) = setup();
        model.finish.notify_one();
        let answer = service
            .dispatch(&question("owner", "qué es una estrella"))
            .await;
        assert_eq!(
            answer["response"]["outputSpeech"]["text"],
            "Una estrella brilla."
        );
        assert_eq!(model.calls.load(Ordering::SeqCst), 1);
        assert!(!answer.to_string().contains("consulta resultado"));
        assert_eq!(answer["response"]["shouldEndSession"], true);
        assert!(answer["response"].get("reprompt").is_none());
    }

    #[tokio::test]
    async fn unsigned_requests_never_reach_the_model() {
        let (service, model) = setup();
        let router = crate::app_with_alexa(service);
        let request = axum::http::Request::post("/alexa")
            .body(axum::body::Body::from(
                question("owner", "hola").to_string(),
            ))
            .unwrap();
        assert_eq!(
            router.oneshot(request).await.unwrap().status(),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(model.calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn launch_and_help_return_alexa_plaintext_without_starting_a_query() {
        let (service, model) = setup();
        let response = service
            .dispatch(&json!({"request":{"type":"LaunchRequest"}}))
            .await;
        assert_eq!(response["version"], "1.0");
        assert_eq!(response["response"]["outputSpeech"]["type"], "PlainText");
        assert_eq!(response["response"]["shouldEndSession"], false);
        assert_eq!(model.calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn slow_queries_are_cancelled_and_release_the_generation_slot() {
        let (service, model) = setup();
        let question = question("owner", "qué es una estrella");
        let response = service.respond(&question, Duration::from_millis(20)).await;
        assert!(
            response["response"]["outputSpeech"]["text"]
                .as_str()
                .unwrap()
                .contains("tardó")
        );
        assert_eq!(service.generation_slot.available_permits(), 1);
        assert_eq!(model.calls.load(Ordering::SeqCst), 1);
        model.finish.notify_one();
        let response = service.respond(&question, Duration::from_secs(1)).await;
        assert_eq!(
            response["response"]["outputSpeech"]["text"],
            "Una estrella brilla."
        );
        assert_eq!(model.calls.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn concurrent_queries_do_not_start_another_generation() {
        let (service, model) = setup();
        let permit = service.generation_slot.acquire().await.unwrap();
        let response = service.dispatch(&question("other", "otra pregunta")).await;
        assert!(
            response["response"]["outputSpeech"]["text"]
                .as_str()
                .unwrap()
                .contains("otra consulta")
        );
        assert_eq!(model.calls.load(Ordering::SeqCst), 0);
        drop(permit);
    }

    #[tokio::test]
    async fn invalid_questions_do_not_reach_the_model() {
        let (service, model) = setup();
        for request in [question("", "hola"), question("owner", "")] {
            let answer = service.dispatch(&request).await;
            assert!(
                answer["response"]["outputSpeech"]["text"]
                    .as_str()
                    .unwrap()
                    .contains("No pude entender")
            );
        }
        assert_eq!(model.calls.load(Ordering::SeqCst), 0);
    }
}
