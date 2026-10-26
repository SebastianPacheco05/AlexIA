//! Local management API, served only behind Vite's loopback proxy.
use crate::{
    alexa::service::{AlexaConfig, AlexaService},
    providers::antigravity::AntigravityProvider,
    settings::{Document, EnvField, Repository, Update},
};
use axum::{
    Json, Router,
    body::Bytes,
    extract::{DefaultBodyLimit, Request, State},
    http::{HeaderMap, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::Serialize;
use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::sync::{Mutex, RwLock};

pub struct Control {
    repository: Repository,
    active: RwLock<Option<Arc<AlexaService>>>,
    active_document: RwLock<Document>,
    error: RwLock<Option<String>>,
    saving: Mutex<()>,
    pub backend_addr: SocketAddr,
    pub ui_addr: SocketAddr,
    pub admin_addr: SocketAddr,
}

#[derive(Serialize)]
struct View {
    revision: String,
    environment: Vec<EnvField>,
    metaprompt: String,
    provider: &'static str,
    status: &'static str,
    error: Option<String>,
    restart_required: bool,
    backend_address: String,
    ui_address: String,
}

#[derive(Debug)]
struct ApiError(StatusCode, String);
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.0, Json(serde_json::json!({"error":self.1}))).into_response()
    }
}

impl Control {
    pub async fn open(repository: Repository, admin_addr: SocketAddr) -> Result<Arc<Self>, String> {
        let document = repository.read()?;
        document.validate()?;
        let backend_addr = document.bind_addr()?;
        let ui_addr = document.ui_addr()?;
        let control = Arc::new(Self {
            repository,
            active: RwLock::new(None),
            active_document: RwLock::new(document.clone()),
            error: RwLock::new(None),
            saving: Mutex::new(()),
            backend_addr,
            ui_addr,
            admin_addr,
        });
        match control.prepare(&document).await {
            Ok(service) => *control.active.write().await = service,
            Err(error) => *control.error.write().await = Some(error),
        }
        Ok(control)
    }

    async fn prepare(&self, document: &Document) -> Result<Option<Arc<AlexaService>>, String> {
        if document.get("ALEXA_SKILL_ID").is_empty() {
            return Ok(None);
        }
        let root = self.repository.root();
        let verifier = resolve(
            root,
            document.get("ALEXA_VERIFIER_PATH"),
            "tools/alexa-verifier/verify.cjs",
        )
        .canonicalize()
        .map_err(|_| "No se encuentra el verificador de Alexa")?;
        let config = AlexaConfig {
            skill_id: document.get("ALEXA_SKILL_ID").into(),
            verifier,
            node: binary(document.get("ALEXA_NODE_BIN"), "node"),
            openssl: binary(document.get("ALEXA_OPENSSL_BIN"), "openssl"),
        };
        config.check().await?;
        let workspace = root.join("target/antigravity-local");
        std::fs::create_dir_all(&workspace)
            .map_err(|_| "No se pudo preparar el directorio del proveedor")?;
        let default_agy = std::env::var_os("USERPROFILE")
            .map(PathBuf::from)
            .map(|p| p.join("AppData/Local/agy/bin/agy.exe"))
            .filter(|p| p.is_file())
            .unwrap_or_else(|| "agy".into());
        let executable = if document.get("AGY_BIN").is_empty() {
            default_agy
        } else {
            document.get("AGY_BIN").into()
        };
        let model = AntigravityProvider::new(executable, workspace)
            .with_gemini_settings(document.get("GEMINI_MODEL"), document.get("GEMINI_LEVEL"))?
            .with_metaprompt(&document.metaprompt)?
            .prewarm().await.map_err(|_| "No se pudo preparar Antigravity. Comprueba el ejecutable, la sesión y la cuota.")?;
        Ok(Some(Arc::new(AlexaService::new(config, Arc::new(model)))))
    }

    async fn view(&self) -> Result<View, ApiError> {
        let document = self.repository.read().map_err(internal)?;
        let active = self.active_document.read().await;
        let restart_required = document.get("BIND_ADDR") != active.get("BIND_ADDR")
            || document.get("UI_MODE") != active.get("UI_MODE")
            || document.get("UI_HOST") != active.get("UI_HOST")
            || document.get("UI_PORT") != active.get("UI_PORT");
        let enabled = self.active.read().await.is_some();
        let error = self.error.read().await.clone();
        Ok(View {
            revision: document.revision.clone(),
            environment: document.fields(),
            metaprompt: document.metaprompt,
            provider: "antigravity",
            status: if error.is_some() {
                "error"
            } else if enabled {
                "ready"
            } else {
                "alexa_blocked"
            },
            error,
            restart_required,
            backend_address: self.backend_addr.to_string(),
            ui_address: self.ui_addr.to_string(),
        })
    }

    async fn save(&self, update: Update) -> Result<View, ApiError> {
        let _guard = self
            .saving
            .try_lock()
            .map_err(|_| ApiError(StatusCode::CONFLICT, "Ya hay un guardado en curso".into()))?;
        let before = self.repository.read().map_err(internal)?;
        if update.revision != before.revision {
            return Err(ApiError(
                StatusCode::CONFLICT,
                "La configuración cambió. Recarga antes de guardar.".into(),
            ));
        }
        let next = self
            .repository
            .propose(&before, update)
            .map_err(|e| ApiError(StatusCode::UNPROCESSABLE_ENTITY, e))?;
        let prepared = self
            .prepare(&next)
            .await
            .map_err(|e| ApiError(StatusCode::UNPROCESSABLE_ENTITY, e))?;
        self.repository
            .persist(&before, &next)
            .map_err(|e| ApiError(StatusCode::CONFLICT, e))?;
        *self.active.write().await = prepared;
        // Listener changes are persisted but cannot change sockets currently in use.
        let mut applied = next;
        {
            let old = self.active_document.read().await;
            for key in ["BIND_ADDR", "UI_HOST", "UI_PORT", "UI_MODE"] {
                applied.environment.insert(key.into(), old.get(key).into());
            }
        }
        *self.active_document.write().await = applied;
        *self.error.write().await = None;
        self.view().await
    }
}

fn internal(message: String) -> ApiError {
    ApiError(StatusCode::INTERNAL_SERVER_ERROR, message)
}
fn binary(value: &str, default: &str) -> PathBuf {
    if value.is_empty() {
        default.into()
    } else {
        value.into()
    }
}
fn resolve(root: &Path, value: &str, default: &str) -> PathBuf {
    root.join(if value.is_empty() { default } else { value })
}

pub fn admin_router(control: Arc<Control>) -> Router {
    Router::new()
        .route("/api/settings", get(read_settings).put(save_settings))
        .route("/api/tunnel", get(read_tunnel))
        .layer(DefaultBodyLimit::max(128 * 1024))
        .layer(middleware::from_fn_with_state(control.clone(), local_only))
        .with_state(control)
}

async fn read_settings(State(control): State<Arc<Control>>) -> Result<Json<View>, ApiError> {
    control.view().await.map(Json)
}
async fn read_tunnel(State(control): State<Arc<Control>>) -> Json<serde_json::Value> {
    Json(serde_json::json!({"endpoint": crate::tunnel::endpoint(control.repository.root(), control.backend_addr.port())}))
}
async fn save_settings(
    State(control): State<Arc<Control>>,
    Json(update): Json<Update>,
) -> Result<Json<View>, ApiError> {
    control.save(update).await.map(Json)
}

async fn local_only(State(control): State<Arc<Control>>, request: Request, next: Next) -> Response {
    if !allowed(request.headers(), control.admin_addr, control.ui_addr) {
        return StatusCode::FORBIDDEN.into_response();
    }
    let mut response = next.run(request).await;
    response
        .headers_mut()
        .insert("cache-control", "no-store".parse().unwrap());
    response
        .headers_mut()
        .insert("x-content-type-options", "nosniff".parse().unwrap());
    response
}

fn allowed(headers: &HeaderMap, admin: SocketAddr, ui: SocketAddr) -> bool {
    headers.get("x-alexia-ui").and_then(|v| v.to_str().ok()) == Some("1")
        && local_request(headers, admin, ui)
}

pub(crate) fn local_request(headers: &HeaderMap, admin: SocketAddr, ui: SocketAddr) -> bool {
    let host = headers
        .get("host")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if ![
        admin.to_string(),
        format!("localhost:{}", admin.port()),
        format!("127.0.0.1:{}", admin.port()),
        format!("[::1]:{}", admin.port()),
    ]
    .contains(&host.to_string())
    {
        return false;
    }
    if [
        "forwarded",
        "x-forwarded-for",
        "x-forwarded-host",
        "cf-connecting-ip",
    ]
    .iter()
    .any(|h| headers.contains_key(*h))
    {
        return false;
    }
    if headers
        .get("sec-fetch-site")
        .and_then(|v| v.to_str().ok())
        .is_some_and(|s| !matches!(s, "same-origin" | "none"))
    {
        return false;
    }
    if let Some(origin) = headers.get("origin") {
        let origin = origin.to_str().unwrap_or("");
        if origin != format!("http://{ui}") && origin != format!("http://localhost:{}", ui.port()) {
            return false;
        }
    }
    true
}

pub fn public_router(control: Arc<Control>) -> Router {
    Router::new()
        .route("/health", get(public_health))
        .route("/alexa", post(public_alexa))
        .layer(DefaultBodyLimit::max(65536))
        .with_state(control)
}
async fn public_health(State(control): State<Arc<Control>>) -> Json<serde_json::Value> {
    let enabled = control.active.read().await.is_some();
    Json(
        serde_json::json!({"status":"ok","alexa":if enabled {"verification_enabled"} else {"blocked"},"gemini":if enabled {"local_cli"} else {"not_configured"}}),
    )
}
async fn public_alexa(
    State(control): State<Arc<Control>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let service = control
        .active
        .read()
        .await
        .clone()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    crate::alexa::service::handle(State(service), headers, body).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use tower::ServiceExt;

    #[tokio::test]
    async fn public_server_does_not_expose_management_and_admin_masks_secrets() {
        let root = std::env::temp_dir().join(format!("alexia-control-{}", std::process::id()));
        std::fs::create_dir_all(root.join("prompts")).unwrap();
        std::fs::write(root.join(".env"), "ALEXA_SKILL_ID=''\nPRIVATE_TOKEN=hidden-value\nBIND_ADDR=127.0.0.1:3000\nUI_PORT=3001\n").unwrap();
        let control = Control::open(Repository::new(root), "127.0.0.1:45679".parse().unwrap())
            .await
            .unwrap();
        let request = axum::http::Request::get("/api/settings")
            .body(axum::body::Body::empty())
            .unwrap();
        assert_eq!(
            public_router(control.clone())
                .oneshot(request)
                .await
                .unwrap()
                .status(),
            StatusCode::NOT_FOUND
        );
        let request = axum::http::Request::get("/api/tunnel")
            .body(axum::body::Body::empty()).unwrap();
        assert_eq!(public_router(control.clone()).oneshot(request).await.unwrap().status(), StatusCode::NOT_FOUND);
        let request = axum::http::Request::get("/api/tunnel")
            .header("host", "127.0.0.1:45679").header("x-alexia-ui", "1")
            .body(axum::body::Body::empty()).unwrap();
        let response = admin_router(control.clone()).oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let tunnel_body = axum::body::to_bytes(response.into_body(), 2048).await.unwrap();
        assert_eq!(serde_json::from_slice::<serde_json::Value>(&tunnel_body).unwrap(), serde_json::json!({"endpoint":null}));
        let request = axum::http::Request::get("/api/settings")
            .header("host", "127.0.0.1:45679")
            .header("x-alexia-ui", "1")
            .body(axum::body::Body::empty())
            .unwrap();
        let response = admin_router(control.clone())
            .oneshot(request)
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), 65536)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(!String::from_utf8_lossy(&body).contains("hidden-value"));
        let payload = serde_json::json!({"revision":json["revision"],"environment":{"AI_PROVIDER":"unsupported"},"metaprompt":"Sé breve."});
        let request = axum::http::Request::put("/api/settings")
            .header("host", "127.0.0.1:45679")
            .header("x-alexia-ui", "1")
            .header("origin", "http://localhost:3001")
            .header("content-type", "application/json")
            .body(axum::body::Body::from(payload.to_string()))
            .unwrap();
        assert_eq!(
            admin_router(control)
                .oneshot(request)
                .await
                .unwrap()
                .status(),
            StatusCode::UNPROCESSABLE_ENTITY
        );
    }
    #[test]
    fn admin_rejects_public_hosts_cross_origin_and_forwarded_requests() {
        let admin = "127.0.0.1:45678".parse().unwrap();
        let ui = "127.0.0.1:3001".parse().unwrap();
        let mut headers = HeaderMap::new();
        headers.insert("host", "127.0.0.1:45678".parse().unwrap());
        headers.insert("x-alexia-ui", "1".parse().unwrap());
        assert!(allowed(&headers, admin, ui));
        headers.insert("origin", "https://evil.example".parse().unwrap());
        assert!(!allowed(&headers, admin, ui));
        headers.insert("origin", "http://localhost:3001".parse().unwrap());
        assert!(allowed(&headers, admin, ui));
        headers.insert("cf-connecting-ip", "1.2.3.4".parse().unwrap());
        assert!(!allowed(&headers, admin, ui));
        headers.remove("cf-connecting-ip");
        headers.insert("host", "evil.example".parse().unwrap());
        assert!(!allowed(&headers, admin, ui));
    }
}
