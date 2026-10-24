use axum::{Json, http::StatusCode};
use serde::Serialize;

#[derive(Serialize)]
pub struct BlockedResponse {
    error: &'static str,
    message: &'static str,
}

/// Fail closed. No body/header extraction, certificate fetch, or provider call.
/// Replace this gate only after signature, certificate chain, timestamp and
/// expected skill ID verification are implemented and tested together.
/// There is deliberately no environment switch or development bypass.
pub async fn blocked() -> (StatusCode, Json<BlockedResponse>) {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(BlockedResponse {
            error: "alexa_verification_not_implemented",
            message: "Alexa requests are blocked until signature and timestamp verification is implemented.",
        }),
    )
}
