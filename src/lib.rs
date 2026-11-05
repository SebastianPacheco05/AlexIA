pub mod alexa;
pub mod auth;
pub mod config;
pub mod control;
pub mod integrations;
pub mod jobs;
pub mod projects;
pub mod providers;
pub mod secrets;
pub mod settings;
pub mod static_ui;
pub mod storage;
pub mod tunnel;
pub mod ui_process;
pub mod voice;

use alexa::service::AlexaService;
use axum::{
    Json, Router,
    routing::{any, get},
};
use axum::{extract::DefaultBodyLimit, routing::post};
use serde::Serialize;
use std::sync::Arc;

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

pub fn app_with_alexa(service: Arc<AlexaService>) -> Router {
    Router::new()
        .route(
            "/health",
            get(|| async {
                Json(Health {
                    status: "ok",
                    alexa: "verification_enabled",
                    gemini: "local_cli",
                })
            }),
        )
        .route("/alexa", post(alexa::service::handle))
        .layer(DefaultBodyLimit::max(65536))
        .with_state(service)
}

async fn health() -> Json<Health> {
    Json(Health {
        status: "ok",
        alexa: "blocked",
        gemini: "not_configured",
    })
}
