//! Serve Vite's built assets on the same loopback management listener in Docker.
use crate::control::{self, Control};
use axum::{
    Router,
    extract::{Path, Request, State},
    http::StatusCode,
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
};
use std::{path::PathBuf, sync::Arc};

pub fn router(control: Arc<Control>, root: PathBuf) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/favicon.svg", get(favicon))
        .route("/github-mark.png", get(github_mark))
        .route("/assets/{file}", get(asset))
        .with_state(Arc::new(root.join("ui/dist")))
        .layer(middleware::from_fn_with_state(control.clone(), local_files))
        .merge(control::admin_router(control))
}

async fn local_files(
    State(control): State<Arc<Control>>,
    request: Request,
    next: Next,
) -> Response {
    if !control::local_request(request.headers(), control.admin_addr, control.ui_addr) {
        return StatusCode::FORBIDDEN.into_response();
    }
    let mut response = next.run(request).await;
    response
        .headers_mut()
        .insert("cache-control", "no-store".parse().unwrap());
    response
        .headers_mut()
        .insert("x-frame-options", "DENY".parse().unwrap());
    response
        .headers_mut()
        .insert("x-content-type-options", "nosniff".parse().unwrap());
    response
}
async fn index(State(root): State<Arc<PathBuf>>) -> Response {
    file(&root.join("index.html"), "text/html; charset=utf-8")
}
async fn favicon(State(root): State<Arc<PathBuf>>) -> Response {
    file(&root.join("favicon.svg"), "image/svg+xml")
}
async fn github_mark(State(root): State<Arc<PathBuf>>) -> Response {
    file(&root.join("github-mark.png"), "image/png")
}
async fn asset(State(root): State<Arc<PathBuf>>, Path(name): Path<String>) -> Response {
    if name.contains("..")
        || !name
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'.' | b'-' | b'_'))
    {
        return StatusCode::NOT_FOUND.into_response();
    }
    let content_type = if name.ends_with(".js") {
        "text/javascript; charset=utf-8"
    } else if name.ends_with(".css") {
        "text/css; charset=utf-8"
    } else {
        return StatusCode::NOT_FOUND.into_response();
    };
    file(&root.join("assets").join(name), content_type)
}
fn file(path: &std::path::Path, content_type: &'static str) -> Response {
    match std::fs::read(path) {
        Ok(body) => ([("content-type", content_type)], body).into_response(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tower::ServiceExt;
    #[tokio::test]
    async fn static_mode_serves_built_ui_only_on_local_hosts() {
        let root = std::env::temp_dir().join(format!("alexia-static-{}", std::process::id()));
        std::fs::create_dir_all(root.join("ui/dist/assets")).unwrap();
        std::fs::write(root.join("ui/dist/index.html"), "<h1>AlexIA</h1>").unwrap();
        std::fs::write(root.join("ui/dist/assets/app.js"), "export {};").unwrap();
        std::fs::write(
            root.join(".env"),
            "ALEXA_SKILL_ID=''\nUI_PORT=3001\nBIND_ADDR=127.0.0.1:3000\n",
        )
        .unwrap();
        let control = Control::open(
            crate::settings::Repository::new(root.clone()),
            "127.0.0.1:3001".parse().unwrap(),
        )
        .await
        .unwrap();
        let app = router(control, root);
        for (path, host, status) in [
            ("/", "localhost:3001", StatusCode::OK),
            ("/assets/app.js", "localhost:3001", StatusCode::OK),
            ("/", "evil.example", StatusCode::FORBIDDEN),
            ("/assets/nope.js", "localhost:3001", StatusCode::NOT_FOUND),
        ] {
            let request = axum::http::Request::get(path)
                .header("host", host)
                .body(axum::body::Body::empty())
                .unwrap();
            assert_eq!(app.clone().oneshot(request).await.unwrap().status(), status);
        }
    }
}
