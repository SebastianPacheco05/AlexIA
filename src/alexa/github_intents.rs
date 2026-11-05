use crate::{
    auth::OWNER, integrations::commands::Commands, jobs::JobRequest, voice::normalize_for_voice,
};
use serde_json::{Value, json};
use std::sync::Arc;

fn say(text: &str) -> Value {
    json!({"version":"1.0","response":{"outputSpeech":{"type":"PlainText","text":normalize_for_voice(text,800)},"shouldEndSession":true}})
}

/// Called only after the existing signature/timestamp/application verifier succeeds.
pub async fn dispatch(
    commands: Option<&Arc<Commands>>,
    skill_id: &str,
    envelope: &Value,
) -> Option<Value> {
    let intent = envelope["request"]["intent"]["name"].as_str()?;
    if !matches!(
        intent,
        "CrearUltimaPrIntent" | "MergeUltimaPrIntent" | "ResultadoGithubIntent"
    ) {
        return None;
    }
    let Some(commands) = commands else {
        return Some(say("Configura GitHub desde el panel local."));
    };
    let user = envelope["context"]["System"]["user"]["userId"]
        .as_str()
        .unwrap_or("");
    let connection = {
        let state = commands.connections.lock().await;
        let Ok(connection) = state.owned(OWNER) else {
            return Some(say(
                "Conecta GitHub y vincula tu usuario Alexa en el panel local.",
            ));
        };
        if user.is_empty()
            || user.len() > 512
            || connection.alexa_skill_id.as_deref() != Some(skill_id)
            || connection.alexa_user_id.as_deref() != Some(user)
        {
            return Some(say(
                "Este usuario Alexa no está vinculado a GitHub. Revisa el panel local.",
            ));
        }
        connection
    };
    if intent == "ResultadoGithubIntent" {
        let mut jobs: Vec<_> = commands
            .jobs
            .lock()
            .await
            .list(OWNER)
            .into_iter()
            .filter(|job| job.request.connection_id.as_deref() == Some(&connection.id))
            .collect();
        jobs.sort_by_key(|job| std::cmp::Reverse(job.created_at));
        let text = if let Some(job) = jobs.first() {
            match job.result_code.as_deref() {
                Some("already_exists") => {
                    "La rama ya tiene una PR abierta. Puedes verla en el panel."
                }
                Some("destination_guard_unverified") => {
                    "Encontré la PR, pero el merge sigue bloqueado porque no se ha verificado la protección del destino."
                }
                Some("technical_isolation_unverified") => {
                    "La creación está bloqueada hasta verificar el aislamiento del analizador técnico."
                }
                Some("freshness_unknown") => {
                    "No tengo una última subida confirmada. Selecciona la rama en el panel."
                }
                _ if job.status.terminal() => {
                    "El trabajo terminó. Consulta el resultado detallado en el panel."
                }
                _ => "El trabajo sigue pendiente. Puedes consultar su estado en el panel.",
            }
        } else {
            "Todavía no tienes trabajos de GitHub."
        };
        return Some(say(text));
    }
    let request_id = envelope["request"]["requestId"].as_str().unwrap_or("");
    if request_id.is_empty() || request_id.len() > 512 {
        return Some(say("No pude identificar esta orden. Vuelve a intentarlo."));
    }
    let operation = if intent == "CrearUltimaPrIntent" {
        "create_pr"
    } else {
        "squash_merge"
    };
    let branch = envelope["request"]["intent"]["slots"]["Rama"]["value"]
        .as_str()
        .unwrap_or("");
    if operation == "squash_merge" && (branch.is_empty() || branch.len() > 255) {
        return Some(say("Indica la rama destino del merge."));
    }
    let request = JobRequest {
        kind: "selection".into(),
        integration_id: "github".into(),
        connection_id: Some(connection.id),
        project_id: None,
        capability: "github.targets.resolve".into(),
        input: json!({"operation":operation,"target_branch_alias":branch}),
        idempotency_key: format!(
            "alexa:{}",
            crate::auth::digest(format!("{skill_id}:{user}:{request_id}").as_bytes())
        ),
    };
    Some(match commands.submit(OWNER, request).await {
        Ok(_) => say(if operation == "create_pr" {
            "Voy a buscar tu última rama para preparar la PR. Consulta el resultado en el panel."
        } else {
            "Voy a buscar tu última PR y revisar el destino del merge. Consulta el resultado en el panel."
        }),
        Err(_) => say(
            "No pude iniciar el trabajo. Revisa la conexión y los repositorios autorizados en el panel.",
        ),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn unlinked_intents_close_without_invoking_a_model_or_network() {
        let envelope = json!({"request":{"intent":{"name":"CrearUltimaPrIntent"}}});
        let response = dispatch(None, "skill", &envelope).await.unwrap();
        assert_eq!(response["response"]["shouldEndSession"], true);
        assert_eq!(response["response"]["outputSpeech"]["type"], "PlainText");
        assert!(
            dispatch(
                None,
                "skill",
                &json!({"request":{"intent":{"name":"PreguntarIntent"}}})
            )
            .await
            .is_none()
        );
    }
}
