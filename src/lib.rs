pub mod alexa;
pub mod config;
pub mod providers;
pub mod voice;

use axum::{Json, Router, routing::{any, get}};
use serde::Serialize;

#[derive(Serialize)]
struct Health {
    status: &'static str,
    alexa: &'static str,
    gemini: &'static str,
}

/// Liveness only: no upstream requests and no credentials required.
pub fn app() -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/alexa", any(alexa::blocked))
}

async fn health() -> Json<Health> {
    Json(Health {
        status: "ok",
        alexa: "blocked",
        gemini: "not_configured",
    })
}
