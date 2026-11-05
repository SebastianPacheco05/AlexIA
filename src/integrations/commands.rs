use super::github::{
    connection::Connections,
    selection::{PullRequest, latest_owned},
};
use crate::{
    auth::{now, random_id},
    jobs::{Job, JobManager, JobRequest},
    projects::Project,
};
use serde_json::{Value, json};
use std::{path::Path, sync::Arc};
use tokio::sync::{Mutex, Semaphore};

pub struct Commands {
    pub jobs: Mutex<JobManager>,
    pub connections: Arc<Mutex<Connections>>,
    workers: Semaphore,
}

impl Commands {
    pub fn open(root: &Path) -> Result<Arc<Self>, &'static str> {
        let mut jobs = JobManager::open(root)?;
        jobs.maintain(now())?;
        let commands = Arc::new(Self {
            jobs: Mutex::new(jobs),
            connections: Arc::new(Mutex::new(Connections::open(root)?)),
            workers: Semaphore::new(2),
        });
        let weak = Arc::downgrade(&commands);
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(60)).await;
                let Some(commands) = weak.upgrade() else {
                    return;
                };
                if commands.jobs.lock().await.maintain(now()).is_err() {
                    eprintln!("{{\"event\":\"integration_retention_failed\"}}");
                }
            }
        });
        Ok(commands)
    }

    pub async fn submit(
        self: &Arc<Self>,
        owner: &str,
        request: JobRequest,
    ) -> Result<Job, &'static str> {
        {
            let mut jobs = self.jobs.lock().await;
            jobs.maintain(now())?;
            if let Some(job) = jobs.duplicate(owner, &request)? {
                return Ok(job);
            }
        }
        if request.integration_id == "github" {
            if request.input.as_object().is_none_or(|input| {
                input.keys().any(|key| {
                    !matches!(
                        key.as_str(),
                        "operation" | "selector" | "target_branch_alias"
                    )
                })
            }) {
                return Err("invalid_input");
            }
            let connections = self.connections.lock().await;
            let connection = connections.owned(owner)?;
            if connection.status != "connected"
                || request.connection_id.as_deref() != Some(&connection.id)
            {
                return Err("connection_required");
            }
            if request.kind != "selection"
                || request.project_id.is_some()
                || request.capability != "github.targets.resolve"
                || !matches!(
                    request.input["operation"].as_str(),
                    Some("create_pr" | "squash_merge")
                )
            {
                return Err("permission_denied");
            }
            if connection.repositories.is_empty() {
                return Err("repositories_required");
            }
            if request.input["operation"] == "squash_merge"
                && request.input["target_branch_alias"]
                    .as_str()
                    .is_none_or(|alias| alias.is_empty() || alias.len() > 255)
            {
                return Err("branch_required");
            }
        } else if request.integration_id != "fixture"
            || request.kind != "execution"
            || request.capability != "fixture.inspect"
            || request.project_id.as_deref() != Some("fixture")
            || request.input != json!({})
        {
            return Err("permission_denied");
        }
        let job = self.jobs.lock().await.enqueue(owner, request, now())?;
        self.spawn(job.clone());
        Ok(job)
    }

    pub fn spawn(self: &Arc<Self>, job: Job) {
        let commands = self.clone();
        tokio::spawn(async move {
            let Ok(_permit) = commands.workers.acquire().await else {
                return;
            };
            let Ok(job) = commands.jobs.lock().await.start(&job.id, now()) else {
                return;
            };
            let cancelled = async {
                loop {
                    tokio::time::sleep(std::time::Duration::from_millis(250)).await;
                    if commands
                        .jobs
                        .lock()
                        .await
                        .get(&job.owner_id, &job.id)
                        .map_or(true, |job| job.cancel_requested || job.status.terminal())
                    {
                        break;
                    }
                }
            };
            let outcome = tokio::select! {
                outcome=tokio::time::timeout(std::time::Duration::from_secs(job.deadline.saturating_sub(now())), commands.run(&job))=>outcome,
                _=cancelled=>return,
            };
            if let Err(code) = outcome.unwrap_or(Err("timed_out")) {
                let _ = commands
                    .jobs
                    .lock()
                    .await
                    .finish(&job.id, "error", code, json!({}), now());
            }
        });
    }

    async fn run(&self, job: &Job) -> Result<(), &'static str> {
        if job.request.integration_id == "fixture" {
            self.jobs.lock().await.finish(&job.id, "success", "fixture_completed", json!({"source":"fixture","coverage":"complete","evidence":[],"explanation_status":"not_requested"}), now())?;
            return Ok(());
        }
        let (connection, client, token) = {
            let state = self.connections.lock().await;
            let connection = state.owned(&job.owner_id)?;
            if Some(connection.id.as_str()) != job.request.connection_id.as_deref()
                || connection.status != "connected"
            {
                return Err("connection_revoked");
            }
            let token = state.token(&connection)?;
            (connection, state.client.clone(), token)
        };
        if job.request.input["operation"] == "create_pr" {
            if let Some(selected) = job.result.get("selected") {
                let id = selected["repository_id"]
                    .as_u64()
                    .ok_or("invalid_selection")?;
                let policy = connection
                    .repositories
                    .iter()
                    .find(|repo| repo.repository_id == id)
                    .ok_or("permission_denied")?;
                let reference = selected["reference"].as_str().ok_or("invalid_selection")?;
                let (owner, repo) = policy
                    .full_name
                    .split_once('/')
                    .ok_or("invalid_repository")?;
                let current = client
                    .get(&token, &["repos", owner, repo, "branches", reference], &[])
                    .await?;
                if current["commit"]["sha"] != selected["head_sha"] {
                    return Err("stale_head");
                }
                let existing = client
                    .pages(
                        &token,
                        &["repos", owner, repo, "pulls"],
                        &[
                            ("state", "open".into()),
                            ("head", format!("{owner}:{reference}")),
                        ],
                        None,
                    )
                    .await?;
                if !existing.is_empty() {
                    self.jobs.lock().await.finish(
                        &job.id,
                        "no_change",
                        "already_exists",
                        json!({"number":existing[0]["number"],"coverage":"complete"}),
                        now(),
                    )?;
                } else {
                    self.jobs.lock().await.finish(&job.id, "blocked", "technical_isolation_unverified", json!({"project":Project::remote(&job.owner_id,&connection.id,id),"snapshot":selected,"coverage":"partial"}), now())?;
                }
                return Ok(());
            }
            let mut choices = Vec::new();
            for policy in &connection.repositories {
                let (owner, repo) = policy
                    .full_name
                    .split_once('/')
                    .ok_or("invalid_repository")?;
                let actual = client.get(&token, &["repos", owner, repo], &[]).await?;
                if actual["id"].as_u64() != Some(policy.repository_id) {
                    return Err("repository_identity_changed");
                }
                for branch in client
                    .pages(&token, &["repos", owner, repo, "branches"], &[], None)
                    .await?
                {
                    if branch["name"] == policy.creation_base_branch {
                        continue;
                    }
                    choices.push(json!({"choice_id":random_id(),"repository_id":policy.repository_id,"repository":policy.full_name,
                        "reference":branch["name"],"head_sha":branch["commit"]["sha"]}));
                    if choices.len() > 100 {
                        return Err("partial_coverage");
                    }
                }
            }
            self.jobs.lock().await.wait_for_input(
                &job.id,
                "freshness_unknown",
                json!({"choices":choices,"coverage":"manual_selection_required"}),
            )?;
            return Ok(());
        }
        let mut prs = Vec::new();
        for policy in &connection.repositories {
            let (owner, repo) = policy
                .full_name
                .split_once('/')
                .ok_or("invalid_repository")?;
            let actual = client.get(&token, &["repos", owner, repo], &[]).await?;
            if actual["id"].as_u64() != Some(policy.repository_id) {
                return Err("repository_identity_changed");
            }
            for value in client
                .pages(
                    &token,
                    &["repos", owner, repo, "pulls"],
                    &[
                        ("state", "open".into()),
                        ("sort", "created".into()),
                        ("direction", "desc".into()),
                    ],
                    None,
                )
                .await?
            {
                prs.push(parse_pr(&value, policy.repository_id)?);
            }
        }
        let Some(pr) = latest_owned(
            &prs,
            &[],
            &job.owner_id,
            &connection.id,
            connection.github_user_id.ok_or("identity_required")?,
            true,
        )?
        else {
            self.jobs.lock().await.finish(
                &job.id,
                "no_change",
                "no_owned_pull_request",
                json!({"coverage":"complete"}),
                now(),
            )?;
            return Ok(());
        };
        let policy = connection
            .repositories
            .iter()
            .find(|policy| policy.repository_id == pr.repository_id)
            .ok_or("permission_denied")?;
        let alias = job.request.input["target_branch_alias"]
            .as_str()
            .ok_or("branch_required")?;
        let targets = if policy.merge_targets.iter().any(|target| target == alias) {
            vec![alias.to_owned()]
        } else {
            policy
                .branch_aliases
                .get(alias)
                .cloned()
                .unwrap_or_default()
        };
        if targets.len() != 1 || !policy.merge_targets.contains(&targets[0]) {
            return Err("branch_ambiguous_or_unauthorized");
        }
        let target = &targets[0];
        let (owner, repo) = policy
            .full_name
            .split_once('/')
            .ok_or("invalid_repository")?;
        let branch = client
            .get(&token, &["repos", owner, repo, "branches", target], &[])
            .await?;
        if pr.head_ref == *target {
            return Err("head_equals_base");
        }
        let code = if pr.draft {
            "draft_pull_request"
        } else if pr.fork {
            "merge_mechanism_not_supported"
        } else {
            "destination_guard_unverified"
        };
        let ci = super::github::ci::inspect(&client, &token, owner, repo, pr).await;
        self.jobs.lock().await.finish(&job.id, "blocked", code, json!({"pull_request":pr,"target_branch":target,
            "target_sha":branch["commit"]["sha"],"coverage":"complete_selection_only","destination_guard_status":"unverified",
            "ci":ci,"project":Project::remote(&job.owner_id,&connection.id,policy.repository_id)}), now())?;
        Ok(())
    }
}

fn parse_pr(value: &Value, repository_id: u64) -> Result<PullRequest, &'static str> {
    fn text(value: &Value) -> Result<String, &'static str> {
        value
            .as_str()
            .filter(|s| s.len() <= 2048)
            .map(str::to_owned)
            .ok_or("github_invalid_response")
    }
    Ok(PullRequest {
        number: value["number"].as_u64().ok_or("github_invalid_response")?,
        node_id: text(&value["node_id"])?,
        repository_id,
        author_id: value["user"]["id"]
            .as_u64()
            .ok_or("github_invalid_response")?,
        created_at: text(&value["created_at"])?,
        head_ref: text(&value["head"]["ref"])?,
        head_sha: text(&value["head"]["sha"])?,
        base_ref: text(&value["base"]["ref"])?,
        base_sha: text(&value["base"]["sha"])?,
        draft: value["draft"].as_bool().ok_or("github_invalid_response")?,
        fork: value["head"]["repo"]["id"].as_u64() != Some(repository_id),
        url: text(&value["html_url"])?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        integrations::github::{
            client::GithubClient,
            connection::{Connection, RepositoryPolicy},
        },
        secrets::SecretStore,
    };
    use axum::{Router, extract::Request, routing::any};
    use std::sync::atomic::{AtomicUsize, Ordering};
    struct SyntheticStore;
    impl SecretStore for SyntheticStore {
        fn put(&self, _: &str, _: &str) -> Result<(), &'static str> {
            Ok(())
        }
        fn get(&self, _: &str) -> Result<String, &'static str> {
            Ok("synthetic-fixture".into())
        }
        fn delete(&self, _: &str) -> Result<(), &'static str> {
            Ok(())
        }
    }
    async fn wait(commands: &Commands, id: &str) -> Job {
        tokio::time::timeout(std::time::Duration::from_secs(5), async {
            loop {
                let job = commands
                    .jobs
                    .lock()
                    .await
                    .get(crate::auth::OWNER, id)
                    .unwrap();
                if job.status.terminal() || job.continuation_id.is_some() {
                    return job;
                }
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
        })
        .await
        .unwrap()
    }
    #[tokio::test]
    async fn real_http_selection_and_continuation_never_publish_or_merge_with_unverified_guards() {
        let writes = Arc::new(AtomicUsize::new(0));
        let observed = writes.clone();
        let app=Router::new().fallback(any(move |request:Request|{
            let observed=observed.clone(); async move {
                if request.method()!=axum::http::Method::GET {observed.fetch_add(1,Ordering::SeqCst);}
                let value=match request.uri().path() {
                    "/repos/o/r"=>json!({"id":1}),
                    "/repos/o/r/branches"=>json!([{"name":"feature/demo","commit":{"sha":"aaa"}},{"name":"main","commit":{"sha":"bbb"}}]),
                    "/repos/o/r/branches/feature%2Fdemo"=>json!({"commit":{"sha":"aaa"}}),
                    "/repos/o/r/branches/main"=>json!({"commit":{"sha":"bbb"}}),
                    "/repos/o/r/pulls"=>{
                        if request.uri().query().unwrap_or("").contains("head="){json!([])} else {
                            json!([
                                {"number":2,"node_id":"other","user":{"id":9},"created_at":"2026-11-03T12:00:00Z","draft":false,"html_url":"https://github.com/o/r/pull/2","head":{"ref":"feature/other","sha":"ccc","repo":{"id":1}},"base":{"ref":"main","sha":"bbb"}},
                                {"number":1,"node_id":"own","user":{"id":7},"created_at":"2026-11-03T11:00:00Z","draft":false,"html_url":"https://github.com/o/r/pull/1","head":{"ref":"feature/demo","sha":"aaa","repo":{"id":1}},"base":{"ref":"main","sha":"bbb"}}
                            ])
                        }
                    },
                    "/repos/o/r/commits/aaa/check-runs"=>json!({"check_runs":[]}),
                    "/repos/o/r/commits/aaa/statuses"=>json!([]),
                    "/repos/o/r/actions/runs"=>json!({"workflow_runs":[]}),
                    _=>json!({"unexpected_path":request.uri().path()}),
                };
                axum::Json(value)
            }
        }));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let origin = format!("http://{}/", listener.local_addr().unwrap());
        let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        let root = std::env::temp_dir().join(format!("alexia-flow-{}", random_id()));
        let commands = Commands::open(&root).unwrap();
        {
            let mut state = commands.connections.lock().await;
            state.client = GithubClient::fixture(&origin);
            state.store = Arc::new(SyntheticStore);
            state
                .save(Connection {
                    id: "c".into(),
                    owner_id: crate::auth::OWNER.into(),
                    client_id: "Iv.fixture".into(),
                    app_id: 42,
                    status: "connected".into(),
                    last_error: None,
                    github_user_id: Some(7),
                    login: Some("human".into()),
                    verified_at: Some(now()),
                    expires_at: None,
                    credential_ref: "synthetic".into(),
                    repositories: vec![RepositoryPolicy {
                        repository_id: 1,
                        installation_id: 10,
                        full_name: "o/r".into(),
                        creation_base_branch: "main".into(),
                        merge_targets: vec!["main".into()],
                        branch_aliases: Default::default(),
                        expected_tests: Vec::new(),
                        follow_head_updates: false,
                    }],
                    alexa_skill_id: Some("skill".into()),
                    alexa_user_id: Some("user-a".into()),
                })
                .unwrap();
        }
        let request = JobRequest {
            kind: "selection".into(),
            integration_id: "github".into(),
            connection_id: Some("c".into()),
            project_id: None,
            capability: "github.targets.resolve".into(),
            input: json!({"operation":"create_pr"}),
            idempotency_key: "create".into(),
        };
        assert!(commands.submit("b", request.clone()).await.is_err());
        let selected = commands
            .submit(crate::auth::OWNER, request.clone())
            .await
            .unwrap();
        let selected = wait(&commands, &selected.id).await;
        assert_eq!(selected.result_code.as_deref(), Some("freshness_unknown"));
        assert_eq!(
            commands
                .submit(crate::auth::OWNER, request)
                .await
                .unwrap()
                .id,
            selected.id
        );
        let choice = selected.result["choices"][0]["choice_id"].as_str().unwrap();
        let resumed = commands
            .jobs
            .lock()
            .await
            .continue_selection(
                crate::auth::OWNER,
                &selected.id,
                selected.continuation_id.as_ref().unwrap(),
                selected.revision,
                choice,
                now(),
            )
            .unwrap();
        commands.spawn(resumed);
        let finished = wait(&commands, &selected.id).await;
        assert_eq!(
            finished.result_code.as_deref(),
            Some("technical_isolation_unverified")
        );
        let merge = commands
            .submit(
                crate::auth::OWNER,
                JobRequest {
                    kind: "selection".into(),
                    integration_id: "github".into(),
                    connection_id: Some("c".into()),
                    project_id: None,
                    capability: "github.targets.resolve".into(),
                    input: json!({"operation":"squash_merge","target_branch_alias":"main"}),
                    idempotency_key: "merge".into(),
                },
            )
            .await
            .unwrap();
        let merge = wait(&commands, &merge.id).await;
        assert_eq!(merge.result["pull_request"]["number"], 1);
        assert_eq!(
            merge.result_code.as_deref(),
            Some("destination_guard_unverified")
        );
        assert_eq!(writes.load(Ordering::SeqCst), 0);
        let envelope = json!({"context":{"System":{"user":{"userId":"user-b"}}},"request":{"intent":{"name":"CrearUltimaPrIntent"},"requestId":"new"}});
        let response = crate::alexa::github_intents::dispatch(Some(&commands), "skill", &envelope)
            .await
            .unwrap();
        assert!(
            response["response"]["outputSpeech"]["text"]
                .as_str()
                .unwrap()
                .contains("no está vinculado")
        );
        server.abort();
    }
}
