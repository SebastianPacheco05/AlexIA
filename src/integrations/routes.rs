use super::{
    commands::Commands,
    github::connection::{self, RepositoryPolicy},
};
use crate::{
    auth::{OWNER, now},
    control::Control,
    jobs::JobRequest,
};
use axum::{
    Json, Router,
    extract::{Path, Request, State},
    http::{HeaderMap, StatusCode, header::SET_COOKIE},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;

type ApiResult = Result<Json<Value>, Error>;
pub struct Error(pub &'static str);
impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let status = match self.0 {
            "unauthenticated" | "bootstrap_expired" => StatusCode::UNAUTHORIZED,
            "permission_denied" => StatusCode::FORBIDDEN,
            "not_found" => StatusCode::NOT_FOUND,
            "idempotency_conflict" | "continuation_invalid" | "disconnect_first" => {
                StatusCode::CONFLICT
            }
            "queue_full" => StatusCode::TOO_MANY_REQUESTS,
            "storage_unavailable" | "storage_not_private" | "storage_unavailable_or_locked" => {
                StatusCode::SERVICE_UNAVAILABLE
            }
            _ => StatusCode::UNPROCESSABLE_ENTITY,
        };
        (status, Json(json!({"error":self.0}))).into_response()
    }
}
impl From<&'static str> for Error {
    fn from(code: &'static str) -> Self {
        Self(code)
    }
}

pub fn router(control: Arc<Control>) -> Router<Arc<Control>> {
    let protected = Router::new()
        .route("/api/integrations", get(catalog))
        .route("/api/integrations/github/connect", post(connect))
        .route("/api/integrations/github/verify", post(verify))
        .route("/api/integrations/github/disconnect", post(disconnect))
        .route(
            "/api/integrations/github/repositories",
            get(repositories).put(save_repositories),
        )
        .route("/api/integrations/github/alexa", post(link_alexa))
        .route("/api/jobs", get(list_jobs).post(create_job))
        .route("/api/jobs/{id}", get(read_job))
        .route("/api/jobs/{id}/cancel", post(cancel_job))
        .route("/api/jobs/{id}/input", post(continue_job))
        .layer(middleware::from_fn_with_state(control, authenticated));
    protected
        .route("/api/session/bootstrap", post(bootstrap))
        .route("/api/session", get(resume))
}

fn commands(control: &Control) -> Result<Arc<Commands>, Error> {
    control.commands.clone().map_err(Error)
}
fn cookie(headers: &HeaderMap) -> &str {
    headers
        .get("cookie")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| {
            value
                .split(';')
                .filter_map(|cookie| cookie.trim().split_once('='))
                .find(|(name, _)| *name == "alexia_session")
                .map(|(_, value)| value)
        })
        .unwrap_or("")
}
async fn authenticated(
    State(control): State<Arc<Control>>,
    mut request: Request,
    next: Next,
) -> Response {
    let principal = control.sessions.lock().await.authenticate(
        cookie(request.headers()),
        request
            .headers()
            .get("x-alexia-csrf")
            .and_then(|value| value.to_str().ok())
            .unwrap_or(""),
        now(),
    );
    let Ok(principal) = principal else {
        return Error("unauthenticated").into_response();
    };
    // Paired clients have distinct principals and no implicit access to owner resources.
    if principal != OWNER {
        return Error("permission_denied").into_response();
    }
    request.extensions_mut().insert(principal);
    next.run(request).await
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Bootstrap {
    code: String,
}
async fn bootstrap(
    State(control): State<Arc<Control>>,
    Json(input): Json<Bootstrap>,
) -> Result<Response, Error> {
    if input.code.len() > 128 {
        return Err(Error("invalid_input"));
    }
    let session = control
        .sessions
        .lock()
        .await
        .bootstrap(&input.code, now())?;
    let mut response =
        Json(json!({"principal_id":session.principal_id,"csrf":session.csrf})).into_response();
    response.headers_mut().insert(
        SET_COOKIE,
        format!(
            "alexia_session={}; HttpOnly; SameSite=Strict; Path=/api; Max-Age=28800",
            session.token
        )
        .parse()
        .unwrap(),
    );
    Ok(response)
}
async fn resume(State(control): State<Arc<Control>>, headers: HeaderMap) -> ApiResult {
    match control
        .sessions
        .lock()
        .await
        .resume(cookie(&headers), now())
    {
        Ok((principal, csrf)) => Ok(Json(
            json!({"authenticated":true,"principal_id":principal,"csrf":csrf}),
        )),
        Err(_) => Ok(Json(json!({"authenticated":false}))),
    }
}
async fn catalog(State(control): State<Arc<Control>>) -> ApiResult {
    let commands = commands(&control)?;
    let view = commands.connections.lock().await.view();
    Ok(Json(json!({"integrations":[view]})))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Connect {
    client_id: String,
    app_id: u64,
}
async fn connect(State(control): State<Arc<Control>>, Json(input): Json<Connect>) -> ApiResult {
    Ok(Json(
        connection::begin_device(
            commands(&control)?.connections.clone(),
            OWNER,
            &input.client_id,
            input.app_id,
        )
        .await?,
    ))
}
async fn verify(State(control): State<Arc<Control>>) -> ApiResult {
    Ok(Json(
        connection::verify(&commands(&control)?.connections, OWNER).await?,
    ))
}
async fn repositories(State(control): State<Arc<Control>>) -> ApiResult {
    Ok(Json(
        connection::available_repositories(&commands(&control)?.connections, OWNER).await?,
    ))
}
async fn save_repositories(
    State(control): State<Arc<Control>>,
    Json(mut input): Json<Vec<RepositoryPolicy>>,
) -> ApiResult {
    if input.is_empty() || input.len() > 10 {
        return Err(Error("invalid_input"));
    }
    let commands = commands(&control)?;
    let available = connection::available_repositories(&commands.connections, OWNER).await?;
    let (connection, client, token) = {
        let state = commands.connections.lock().await;
        let connection = state.owned(OWNER)?;
        let token = state.token(&connection)?;
        (connection, state.client.clone(), token)
    };
    let mut ids = std::collections::HashSet::new();
    for policy in &mut input {
        if !ids.insert(policy.repository_id)
            || policy.merge_targets.len() > 20
            || policy.branch_aliases.len() > 20
            || policy.expected_tests.len() > 50
        {
            return Err(Error("invalid_input"));
        }
        let repo = available
            .as_array()
            .and_then(|repos| {
                repos.iter().find(|repo| {
                    repo["repository_id"].as_u64() == Some(policy.repository_id)
                        && repo["installation_id"].as_u64() == Some(policy.installation_id)
                })
            })
            .ok_or("permission_denied")?;
        policy.full_name = repo["full_name"]
            .as_str()
            .ok_or("github_invalid_response")?
            .into();
        let (owner, name) = policy
            .full_name
            .split_once('/')
            .ok_or("github_invalid_response")?;
        for branch in
            std::iter::once(&policy.creation_base_branch).chain(policy.merge_targets.iter())
        {
            if branch.is_empty() || branch.len() > 255 {
                return Err(Error("invalid_input"));
            }
            client
                .get(&token, &["repos", owner, name, "branches", branch], &[])
                .await?;
        }
        if policy.branch_aliases.iter().any(|(alias, targets)| {
            alias.is_empty()
                || alias.len() > 255
                || targets.is_empty()
                || targets
                    .iter()
                    .any(|target| !policy.merge_targets.contains(target))
        }) {
            return Err(Error("invalid_input"));
        }
    }
    let mut state = commands.connections.lock().await;
    let mut current = state.owned(OWNER)?;
    if current.id != connection.id {
        return Err(Error("connection_changed"));
    }
    current.repositories = input;
    state.save(current)?;
    Ok(Json(state.view()))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AlexaLink {
    skill_id: String,
    user_id: String,
}
async fn link_alexa(
    State(control): State<Arc<Control>>,
    Json(input): Json<AlexaLink>,
) -> ApiResult {
    if !input.skill_id.starts_with("amzn1.ask.skill.")
        || input.skill_id.len() > 256
        || !input.user_id.starts_with("amzn1.ask.account.")
        || input.user_id.len() > 512
    {
        return Err(Error("invalid_input"));
    }
    let commands = commands(&control)?;
    let mut state = commands.connections.lock().await;
    let mut connection = state.owned(OWNER)?;
    connection.alexa_skill_id = Some(input.skill_id);
    connection.alexa_user_id = Some(input.user_id);
    state.save(connection)?;
    Ok(Json(json!({"linked":true})))
}
async fn disconnect(State(control): State<Arc<Control>>) -> ApiResult {
    let commands = commands(&control)?;
    disconnect_current(&commands).await
}
async fn disconnect_current(commands: &Commands) -> ApiResult {
    let mut state = commands.connections.lock().await;
    let mut connection = state
        .current
        .clone()
        .filter(|connection| connection.owner_id == OWNER)
        .ok_or("connection_required")?;
    connection.status = "revoked".into();
    connection.last_error = Some("disconnect_cleanup_pending".into());
    connection.alexa_skill_id = None;
    connection.alexa_user_id = None;
    state.save(connection.clone())?;
    let mut jobs = commands.jobs.lock().await;
    for job in jobs
        .list(OWNER)
        .iter()
        .filter(|job| job.request.connection_id.as_deref() == Some(&connection.id))
    {
        jobs.cancel(OWNER, &job.id, now())?;
    }
    state.store.delete(&connection.credential_ref)?;
    connection.last_error = None;
    state.save(connection)?;
    Ok(Json(
        json!({"disconnected":true,"remote_revocation":"revoke_app_authorization_in_github_settings"}),
    ))
}

#[cfg(test)]
mod disconnect_tests {
    use super::*;
    use crate::{auth::random_id, secrets::SecretStore};
    use std::sync::atomic::{AtomicBool, Ordering};

    struct CleanupStore(AtomicBool);
    impl SecretStore for CleanupStore {
        fn put(&self, _: &str, _: &str) -> Result<(), &'static str> {
            Ok(())
        }
        fn get(&self, _: &str) -> Result<String, &'static str> {
            Err("vault_unavailable")
        }
        fn delete(&self, _: &str) -> Result<(), &'static str> {
            if self.0.load(Ordering::SeqCst) {
                Err("vault_unavailable")
            } else {
                Ok(())
            }
        }
    }

    #[tokio::test]
    async fn failed_disconnect_cleanup_stays_retryable_until_success_even_after_expiry() {
        let root = std::env::temp_dir().join(format!("alexia-disconnect-{}", random_id()));
        let commands = Commands::open(&root).unwrap();
        let store = Arc::new(CleanupStore(AtomicBool::new(true)));
        {
            let mut state = commands.connections.lock().await;
            state.store = store.clone();
            state.save(serde_json::from_value(json!({
                "id":"synthetic", "owner_id":OWNER, "client_id":"Iv.fixture", "app_id":42,
                "status":"connected", "github_user_id":7, "login":"fixture", "verified_at":null,
                "expires_at":1, "credential_ref":"synthetic", "repositories":[],
                "alexa_skill_id":"synthetic-skill", "alexa_user_id":"synthetic-user"
            })).unwrap()).unwrap();
        }
        assert!(matches!(
            disconnect_current(&commands).await,
            Err(Error("vault_unavailable"))
        ));
        {
            let state = commands.connections.lock().await;
            let view = state.view();
            assert_eq!(view["connection"]["status"], "revoked");
            assert_eq!(
                view["connection"]["last_error"],
                "disconnect_cleanup_pending"
            );
            assert_eq!(view["connection"]["alexa_linked"], false);
            assert!(state.owned(OWNER).is_err());
        }
        assert_eq!(
            connection::begin_device(commands.connections.clone(), OWNER, "Iv.fixture", 42)
                .await
                .unwrap_err(),
            "disconnect_first"
        );
        store.0.store(false, Ordering::SeqCst);
        assert!(disconnect_current(&commands).await.is_ok());
        drop(commands);
        let recovered = connection::Connections::open(&root).unwrap().view();
        assert_eq!(recovered["connection"]["status"], "revoked");
        assert!(recovered["connection"]["last_error"].is_null());
    }
}
async fn create_job(
    State(control): State<Arc<Control>>,
    Json(input): Json<JobRequest>,
) -> Result<Response, Error> {
    let job = commands(&control)?.submit(OWNER, input).await?;
    Ok((
        StatusCode::ACCEPTED,
        Json(json!({"job_id":job.id,"status_url":format!("/api/jobs/{}",job.id)})),
    )
        .into_response())
}
async fn list_jobs(State(control): State<Arc<Control>>) -> ApiResult {
    let commands = commands(&control)?;
    let mut jobs = commands.jobs.lock().await;
    jobs.maintain(now())?;
    Ok(Json(json!({"jobs":jobs.list(OWNER)})))
}
async fn read_job(State(control): State<Arc<Control>>, Path(id): Path<String>) -> ApiResult {
    let commands = commands(&control)?;
    let mut jobs = commands.jobs.lock().await;
    jobs.maintain(now())?;
    Ok(Json(json!(jobs.get(OWNER, &id)?)))
}
async fn cancel_job(State(control): State<Arc<Control>>, Path(id): Path<String>) -> ApiResult {
    Ok(Json(json!(commands(&control)?.jobs.lock().await.cancel(
        OWNER,
        &id,
        now()
    )?)))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Continuation {
    continuation_id: String,
    revision: u64,
    choice_id: String,
}
async fn continue_job(
    State(control): State<Arc<Control>>,
    Path(id): Path<String>,
    Json(input): Json<Continuation>,
) -> ApiResult {
    let commands = commands(&control)?;
    let job = commands.jobs.lock().await.continue_selection(
        OWNER,
        &id,
        &input.continuation_id,
        input.revision,
        &input.choice_id,
        now(),
    )?;
    commands.spawn(job.clone());
    Ok(Json(json!(job)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{Body, to_bytes},
        http::Request,
    };
    use tower::ServiceExt;
    #[tokio::test]
    async fn private_jobs_require_session_and_csrf_and_never_exist_on_public_listener() {
        let root = std::env::temp_dir().join(format!("alexia-api-{}", crate::auth::random_id()));
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(
            root.join(".env"),
            "ALEXA_SKILL_ID=''\nBIND_ADDR=127.0.0.1:4300\nUI_PORT=4301\n",
        )
        .unwrap();
        let control = Control::open(
            crate::settings::Repository::new(root),
            "127.0.0.1:4302".parse().unwrap(),
        )
        .await
        .unwrap();
        let (sessions, code) = crate::auth::Sessions::new(now());
        *control.sessions.lock().await = sessions;
        let app = crate::control::admin_router(control.clone());
        fn request(
            path: &str,
            method: &str,
            body: Value,
            cookie: &str,
            csrf: &str,
        ) -> Request<Body> {
            Request::builder()
                .method(method)
                .uri(path)
                .header("host", "127.0.0.1:4302")
                .header("origin", "http://127.0.0.1:4301")
                .header("x-alexia-ui", "1")
                .header("content-type", "application/json")
                .header("cookie", cookie)
                .header("x-alexia-csrf", csrf)
                .body(Body::from(body.to_string()))
                .unwrap()
        }
        let denied = app
            .clone()
            .oneshot(request("/api/jobs", "GET", json!(null), "", ""))
            .await
            .unwrap();
        assert_eq!(denied.status(), StatusCode::UNAUTHORIZED);
        let response = app
            .clone()
            .oneshot(request(
                "/api/session/bootstrap",
                "POST",
                json!({"code":code}),
                "",
                "",
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let cookie = response.headers()["set-cookie"]
            .to_str()
            .unwrap()
            .to_owned();
        assert!(cookie.contains("HttpOnly") && cookie.contains("SameSite=Strict"));
        let session: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), 4096).await.unwrap()).unwrap();
        let csrf = session["csrf"].as_str().unwrap();
        let catalog = app
            .clone()
            .oneshot(request(
                "/api/integrations",
                "GET",
                json!(null),
                &cookie,
                csrf,
            ))
            .await
            .unwrap();
        assert_eq!(catalog.status(), StatusCode::OK);
        let catalog: Value =
            serde_json::from_slice(&to_bytes(catalog.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(catalog["integrations"][0]["authorization_flow_version"], 2);
        assert_eq!(
            app.clone()
                .oneshot(request("/api/jobs", "GET", json!(null), &cookie, "wrong"))
                .await
                .unwrap()
                .status(),
            StatusCode::UNAUTHORIZED
        );
        let input = json!(JobRequest::fixture("fixture-key", json!({})));
        let response = app
            .clone()
            .oneshot(request("/api/jobs", "POST", input.clone(), &cookie, csrf))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::ACCEPTED);
        let job: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), 4096).await.unwrap()).unwrap();
        let mut changed = input;
        changed["input"] = json!({"different":true});
        assert_eq!(
            app.clone()
                .oneshot(request("/api/jobs", "POST", changed, &cookie, csrf))
                .await
                .unwrap()
                .status(),
            StatusCode::CONFLICT
        );
        let path = format!("/api/jobs/{}", job["job_id"].as_str().unwrap());
        let result = app
            .oneshot(request(&path, "GET", json!(null), &cookie, csrf))
            .await
            .unwrap();
        assert_eq!(result.status(), StatusCode::OK);
        let public = crate::control::public_router(control);
        for path in ["/api/jobs", "/api/integrations", "/api/session/bootstrap"] {
            assert_eq!(
                public
                    .clone()
                    .oneshot(Request::get(path).body(Body::empty()).unwrap())
                    .await
                    .unwrap()
                    .status(),
                StatusCode::NOT_FOUND
            );
        }
    }
}
