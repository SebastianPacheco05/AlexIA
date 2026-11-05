//! Read-only CI observations. These are never promoted to proof of a new base snapshot.
use super::{client::GithubClient, selection::PullRequest};
use serde_json::{Value, json};

pub async fn inspect(
    client: &GithubClient,
    token: &str,
    owner: &str,
    repo: &str,
    pr: &PullRequest,
) -> Value {
    let checks = client
        .pages(
            token,
            &["repos", owner, repo, "commits", &pr.head_sha, "check-runs"],
            &[("filter", "latest".into())],
            Some("check_runs"),
        )
        .await;
    let statuses = client
        .pages(
            token,
            &["repos", owner, repo, "commits", &pr.head_sha, "statuses"],
            &[],
            None,
        )
        .await;
    let runs = client
        .pages(
            token,
            &["repos", owner, repo, "actions", "runs"],
            &[("head_sha", pr.head_sha.clone())],
            Some("workflow_runs"),
        )
        .await;
    let mut errors = Vec::new();
    fn collect(
        result: Result<Vec<Value>, &'static str>,
        source: &str,
        errors: &mut Vec<Value>,
    ) -> Vec<Value> {
        match result {
            Ok(values) => values,
            Err(error) => {
                errors.push(json!({"source":source,"code":error}));
                Vec::new()
            }
        }
    }
    let checks=collect(checks,"checks",&mut errors).into_iter().map(|check| json!({
        "id":check["id"],"name":check["name"],"head_sha":check["head_sha"],"app_id":check["app"]["id"],
        "status":check["status"],"conclusion":check["conclusion"]})).collect::<Vec<_>>();
    let statuses=collect(statuses,"statuses",&mut errors).into_iter().map(|status|json!({
        "id":status["id"],"context":status["context"],"state":status["state"],"creator_id":status["creator"]["id"],"created_at":status["created_at"]})).collect::<Vec<_>>();
    let runs=collect(runs,"actions",&mut errors).into_iter().map(|run|json!({
        "id":run["id"],"workflow_id":run["workflow_id"],"head_sha":run["head_sha"],"attempt":run["run_attempt"],
        "status":run["status"],"conclusion":run["conclusion"],"event":run["event"]})).collect::<Vec<_>>();
    json!({"head_sha":pr.head_sha,"checks":checks,"statuses":statuses,"workflow_runs":runs,"errors":errors,
        "coverage":"partial","test_gate":"base_and_requirements_not_verified","all_tests_passed":false})
}
