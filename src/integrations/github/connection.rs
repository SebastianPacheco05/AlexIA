use super::{client::GithubClient, test_gate::ExpectedTest};
use crate::{
    auth::{OWNER, now, random_id},
    secrets::{OsSecretStore, SecretStore},
    storage::Journal,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::Path, sync::Arc};
use tokio::sync::Mutex;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RepositoryPolicy {
    pub repository_id: u64,
    pub installation_id: u64,
    pub full_name: String,
    pub creation_base_branch: String,
    pub merge_targets: Vec<String>,
    #[serde(default)]
    pub branch_aliases: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    pub expected_tests: Vec<ExpectedTest>,
    #[serde(default)]
    pub follow_head_updates: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Connection {
    pub id: String,
    pub owner_id: String,
    pub client_id: String,
    pub app_id: u64,
    pub status: String,
    #[serde(default)]
    pub last_error: Option<String>,
    pub github_user_id: Option<u64>,
    pub login: Option<String>,
    pub verified_at: Option<u64>,
    pub expires_at: Option<u64>,
    pub credential_ref: String,
    pub repositories: Vec<RepositoryPolicy>,
    pub alexa_skill_id: Option<String>,
    pub alexa_user_id: Option<String>,
}

pub struct Connections {
    journal: Journal,
    pub current: Option<Connection>,
    pub client: GithubClient,
    pub store: Arc<dyn SecretStore>,
}

impl Connections {
    pub fn open(root: &Path) -> Result<Self, &'static str> {
        let (journal, records) = Journal::open(root, "github.jsonl")?;
        let mut current: Option<Connection> = None;
        for record in records {
            current = serde_json::from_value(record["connection"].clone())
                .map_err(|_| "journal_corrupt")?;
        }
        if let Some(connection) = current.as_mut() {
            if connection.status == "connecting" {
                connection.status = "configured_unverified".into();
            }
            if connection.status == "connected" {
                connection.status = "configured_unverified".into();
            }
        }
        Ok(Self {
            journal,
            current,
            client: GithubClient::new()?,
            store: Arc::new(OsSecretStore),
        })
    }
    pub fn save(&mut self, connection: Connection) -> Result<(), &'static str> {
        self.journal
            .compact(vec![json!({"connection":connection})])?;
        self.current = Some(connection);
        Ok(())
    }
    pub fn owned(&self, owner: &str) -> Result<Connection, &'static str> {
        self.current
            .as_ref()
            .filter(|connection| connection.owner_id == owner && connection.status != "revoked")
            .cloned()
            .ok_or("connection_required")
    }
    pub fn token(&self, connection: &Connection) -> Result<String, &'static str> {
        if connection.github_user_id.is_none() {
            return Err(
                if connection.last_error.as_deref() == Some("app_installation_required") {
                    "app_installation_required"
                } else {
                    "github_authorization_incomplete"
                },
            );
        }
        if connection.expires_at.is_some_and(|expiry| now() >= expiry) {
            return Err("connection_expired");
        }
        self.store.get(&connection.credential_ref)
    }
    pub fn view(&self) -> Value {
        let connection = self.current.as_ref().map(|connection| json!({
            "id":connection.id,"status":if connection.status != "revoked" && connection.expires_at.is_some_and(|expiry| now() >= expiry) {"expired"} else {&connection.status},
            "github_user_id":connection.github_user_id,"login":connection.login,"app_id":connection.app_id,
            "repositories":connection.repositories,"verified_at":connection.verified_at,"last_error":connection.last_error,
            "alexa_linked":connection.alexa_skill_id.is_some() && connection.alexa_user_id.is_some()
        }));
        json!({"id":"github","authorization_flow_version":2,"connection":connection,"capabilities":[
            {"id":"github.targets.resolve","available":self.current.as_ref().is_some_and(|c| c.status=="connected")},
            {"id":"github.pull_requests.create","available":false,"reason":"technical_isolation_unverified"},
            {"id":"github.pull_requests.squash_merge","available":false,"reason":"destination_guard_unverified"}
        ],"push_observation_mode":"manual","coverage":"freshness_unknown"})
    }
}

pub async fn begin_device(
    state: Arc<Mutex<Connections>>,
    owner: &str,
    client_id: &str,
    app_id: u64,
) -> Result<Value, &'static str> {
    if owner != OWNER
        || app_id == 0
        || client_id.len() > 100
        || !client_id.starts_with("Iv")
        || !client_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'.')
    {
        return Err("invalid_app_configuration");
    }
    let mut connections = state.lock().await;
    if connections.current.as_ref().is_some_and(|connection| {
        connection.status != "revoked"
            || connection.last_error.as_deref() == Some("disconnect_cleanup_pending")
    }) {
        return Err("disconnect_first");
    }
    let client = connections.client.clone();
    let connection = Connection {
        id: random_id(),
        owner_id: owner.into(),
        client_id: client_id.into(),
        app_id,
        status: "connecting".into(),
        last_error: None,
        github_user_id: None,
        login: None,
        verified_at: None,
        expires_at: None,
        credential_ref: random_id(),
        repositories: Vec::new(),
        alexa_skill_id: None,
        alexa_user_id: None,
    };
    connections.save(connection.clone())?;
    drop(connections);
    let device = async {
        let code = client.device_code(client_id).await?;
        let secret = code["device_code"]
            .as_str()
            .filter(|code| !code.is_empty() && code.len() < 512)
            .ok_or("device_flow_unavailable")?
            .to_owned();
        let user_code = code["user_code"]
            .as_str()
            .filter(|code| !code.is_empty() && code.len() <= 32)
            .ok_or("device_flow_unavailable")?
            .to_owned();
        if code["verification_uri"] != "https://github.com/login/device" {
            return Err("github_invalid_response");
        }
        let lifetime = code["expires_in"]
            .as_u64()
            .filter(|seconds| *seconds > 0 && *seconds <= 1800)
            .ok_or("github_invalid_response")?;
        let interval = code["interval"].as_u64().unwrap_or(5).clamp(5, 60);
        Ok((secret, user_code, lifetime, interval))
    }
    .await;
    let (secret, user_code, lifetime, interval) = match device {
        Ok(device) => device,
        Err(error) => {
            let mut connections = state.lock().await;
            if let Some(mut current) = connections
                .current
                .clone()
                .filter(|current| current.id == connection.id && current.status == "connecting")
            {
                current.status = "degraded".into();
                current.last_error = Some(error.into());
                connections.save(current)?;
            }
            return Err(error);
        }
    };
    tokio::spawn(async move {
        let result = poll_device(
            &state,
            &client,
            &connection,
            &secret,
            interval,
            now() + lifetime,
        )
        .await;
        if let Err(code) = result {
            let mut connections = state.lock().await;
            if let Some(mut current) = connections
                .current
                .clone()
                .filter(|current| current.id == connection.id && current.status == "connecting")
            {
                current.status = if code == "connection_expired" {
                    "expired"
                } else {
                    "degraded"
                }
                .into();
                current.last_error = Some(code.into());
                let _ = connections.save(current);
            }
        }
    });
    Ok(
        json!({"verification_uri":"https://github.com/login/device","user_code":user_code,"expires_in":lifetime}),
    )
}

async fn poll_device(
    state: &Arc<Mutex<Connections>>,
    client: &GithubClient,
    connection: &Connection,
    code: &str,
    mut interval: u64,
    deadline: u64,
) -> Result<(), &'static str> {
    while now() < deadline {
        tokio::time::sleep(std::time::Duration::from_secs(interval)).await;
        {
            let state = state.lock().await;
            if !state
                .current
                .as_ref()
                .is_some_and(|c| c.id == connection.id && c.status == "connecting")
            {
                return Err("cancelled");
            }
        }
        let value = client.device_token(&connection.client_id, code).await?;
        if let Some(error) = value["error"].as_str() {
            match error {
                "authorization_pending" => continue,
                "slow_down" => {
                    interval = (interval + 5).min(60);
                    continue;
                }
                _ => return Err("authorization_failed"),
            }
        }
        let token = value["access_token"]
            .as_str()
            .filter(|value| value.len() < 2000)
            .ok_or("github_invalid_response")?;
        let user = client.get(token, &["user"], &[]).await?;
        if user["type"] != "User" {
            return Err("human_identity_required");
        }
        let user_id = user["id"].as_u64().ok_or("github_invalid_response")?;
        let installations = client
            .pages(
                token,
                &["user", "installations"],
                &[],
                Some("installations"),
            )
            .await?;
        let installed = installations
            .iter()
            .any(|installation| installation["app_id"].as_u64() == Some(connection.app_id));
        let mut state = state.lock().await;
        if !state
            .current
            .as_ref()
            .is_some_and(|c| c.id == connection.id && c.status == "connecting")
        {
            return Err("cancelled");
        }
        let expires_at = match value.get("expires_in") {
            Some(value) => Some(
                now()
                    + value
                        .as_u64()
                        .filter(|seconds| *seconds > 0 && *seconds <= 31_536_000)
                        .ok_or("github_invalid_response")?,
            ),
            None => None,
        };
        state.store.put(&connection.credential_ref, token)?;
        let mut updated = connection.clone();
        updated.github_user_id = Some(user_id);
        updated.login = user["login"].as_str().map(str::to_owned);
        updated.verified_at = Some(now());
        updated.status = if installed {
            "connected"
        } else {
            "configured_unverified"
        }
        .into();
        updated.last_error = if installed {
            None
        } else {
            Some("app_installation_required".into())
        };
        updated.expires_at = expires_at;
        if let Err(error) = state.save(updated) {
            let _ = state.store.delete(&connection.credential_ref);
            return Err(error);
        }
        return Ok(());
    }
    Err("connection_expired")
}

pub async fn verify(state: &Arc<Mutex<Connections>>, owner: &str) -> Result<Value, &'static str> {
    let (connection, client, token) = {
        let state = state.lock().await;
        let connection = state.owned(owner)?;
        let token = state.token(&connection)?;
        (connection, state.client.clone(), token)
    };
    let user = client.get(&token, &["user"], &[]).await?;
    if user["id"].as_u64() != connection.github_user_id || user["type"] != "User" {
        return Err("identity_changed");
    }
    let installations = client
        .pages(
            &token,
            &["user", "installations"],
            &[],
            Some("installations"),
        )
        .await?;
    let installed = installations
        .iter()
        .any(|installation| installation["app_id"].as_u64() == Some(connection.app_id));
    let mut state = state.lock().await;
    let mut current = state.owned(owner)?;
    if current.id != connection.id {
        return Err("connection_changed");
    }
    current.status = if installed {
        "connected"
    } else {
        "configured_unverified"
    }
    .into();
    current.last_error = if installed {
        None
    } else {
        Some("app_installation_required".into())
    };
    current.verified_at = Some(now());
    current.login = user["login"].as_str().map(str::to_owned);
    state.save(current)?;
    if !installed {
        return Err("app_installation_required");
    }
    Ok(state.view())
}

pub async fn available_repositories(
    state: &Arc<Mutex<Connections>>,
    owner: &str,
) -> Result<Value, &'static str> {
    let (connection, client, token) = {
        let state = state.lock().await;
        let connection = state.owned(owner)?;
        let token = state.token(&connection)?;
        (connection, state.client.clone(), token)
    };
    let installations = client
        .pages(
            &token,
            &["user", "installations"],
            &[],
            Some("installations"),
        )
        .await?;
    let mut repositories = Vec::new();
    for installation in installations
        .iter()
        .filter(|installation| installation["app_id"].as_u64() == Some(connection.app_id))
    {
        let id = installation["id"]
            .as_u64()
            .ok_or("github_invalid_response")?;
        let repos = client
            .pages(
                &token,
                &["user", "installations", &id.to_string(), "repositories"],
                &[],
                Some("repositories"),
            )
            .await?;
        for repo in repos {
            repositories.push(json!({"repository_id":repo["id"],"installation_id":id,"full_name":repo["full_name"],"default_branch":repo["default_branch"]}));
        }
    }
    Ok(json!(repositories))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        Router,
        routing::{get, post},
    };
    use std::{
        collections::HashMap,
        sync::{
            Mutex as StdMutex,
            atomic::{AtomicBool, Ordering},
        },
    };
    #[derive(Default)]
    struct MemoryStore(StdMutex<HashMap<String, String>>);
    impl SecretStore for MemoryStore {
        fn put(&self, key: &str, value: &str) -> Result<(), &'static str> {
            self.0.lock().unwrap().insert(key.into(), value.into());
            Ok(())
        }
        fn get(&self, key: &str) -> Result<String, &'static str> {
            self.0
                .lock()
                .unwrap()
                .get(key)
                .cloned()
                .ok_or("vault_unavailable")
        }
        fn delete(&self, key: &str) -> Result<(), &'static str> {
            self.0.lock().unwrap().remove(key);
            Ok(())
        }
    }
    #[tokio::test]
    async fn device_authorization_survives_missing_installation_and_verify_finishes_after_install()
    {
        let installed = Arc::new(AtomicBool::new(false));
        let observed = installed.clone();
        let app=Router::new()
            .route("/login/oauth/access_token",post(||async{axum::Json(json!({"access_token":"synthetic-test-token","expires_in":3600}))}))
            .route("/user",get(||async{axum::Json(json!({"id":7,"type":"User","login":"fixture-user"}))}))
            .route("/user/installations",get(move||{let observed=observed.clone();async move{axum::Json(json!({"installations":if observed.load(Ordering::SeqCst){vec![json!({"app_id":42,"id":10})]}else{vec![]}}))}}));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let origin = format!("http://{}/", listener.local_addr().unwrap());
        let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        let root = std::env::temp_dir().join(format!("alexia-oauth-{}", random_id()));
        let mut connections = Connections::open(&root).unwrap();
        connections.client = GithubClient::fixture(&origin);
        connections.store = Arc::new(MemoryStore::default());
        let connection:Connection=serde_json::from_value(json!({"id":"c","owner_id":OWNER,"client_id":"Iv.fixture","app_id":42,"status":"connecting","github_user_id":null,"login":null,"verified_at":null,"expires_at":null,"credential_ref":"synthetic","repositories":[],"alexa_skill_id":null,"alexa_user_id":null})).unwrap();
        connections.save(connection.clone()).unwrap();
        let client = connections.client.clone();
        let state = Arc::new(Mutex::new(connections));
        poll_device(
            &state,
            &client,
            &connection,
            "synthetic-device-code",
            0,
            now() + 5,
        )
        .await
        .unwrap();
        {
            let current = state.lock().await;
            assert_eq!(
                current.view()["connection"]["status"],
                "configured_unverified"
            );
            assert_eq!(
                current.view()["connection"]["last_error"],
                "app_installation_required"
            );
            assert_eq!(current.current.as_ref().unwrap().github_user_id, Some(7));
        }
        assert_eq!(
            verify(&state, OWNER).await.unwrap_err(),
            "app_installation_required"
        );
        installed.store(true, Ordering::SeqCst);
        let view = verify(&state, OWNER).await.unwrap();
        assert_eq!(view["connection"]["status"], "connected");
        assert!(view["connection"]["last_error"].is_null());
        server.abort();
    }

    #[tokio::test]
    async fn old_failed_authorization_reports_setup_problem_before_looking_for_a_token() {
        let root = std::env::temp_dir().join(format!("alexia-oauth-{}", random_id()));
        let mut connections = Connections::open(&root).unwrap();
        connections.store = Arc::new(MemoryStore::default());
        let mut connection:Connection=serde_json::from_value(json!({"id":"c","owner_id":OWNER,"client_id":"Iv.fixture","app_id":42,"status":"degraded","github_user_id":null,"login":null,"verified_at":null,"expires_at":null,"credential_ref":"synthetic","repositories":[],"alexa_skill_id":null,"alexa_user_id":null})).unwrap();
        connections.save(connection.clone()).unwrap();
        let state = Arc::new(Mutex::new(connections));
        assert_eq!(
            verify(&state, OWNER).await.unwrap_err(),
            "github_authorization_incomplete"
        );
        connection.last_error = Some("app_installation_required".into());
        state.lock().await.save(connection).unwrap();
        assert_eq!(
            verify(&state, OWNER).await.unwrap_err(),
            "app_installation_required"
        );
    }
}
