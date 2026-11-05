use alexia::jobs::{JobManager, JobRequest, Status};
use serde_json::json;

#[test]
fn idempotency_and_ownership_survive_reopen() {
    let root = std::env::temp_dir().join(format!("alexia-jobs-{}", alexia::auth::random_id()));
    let mut manager = JobManager::open(&root).unwrap();
    let request = JobRequest::fixture("key-1", json!({"message":"fixture"}));
    let first = manager.enqueue("a", request.clone(), 100).unwrap();
    assert_eq!(
        manager.enqueue("a", request.clone(), 101).unwrap().id,
        first.id
    );
    let mut changed = request.clone();
    changed.input = json!({"message":"different"});
    assert_eq!(
        manager.enqueue("a", changed, 101).unwrap_err(),
        "idempotency_conflict"
    );
    assert!(manager.get("b", &first.id).is_err());
    drop(manager);
    let mut manager = JobManager::open(&root).unwrap();
    assert_eq!(
        manager.get("a", &first.id).unwrap().status,
        Status::Interrupted
    );
    assert_eq!(manager.enqueue("a", request, 102).unwrap().id, first.id);
    assert_ne!(
        manager
            .enqueue("b", JobRequest::fixture("key-1", json!({})), 102)
            .unwrap()
            .id,
        first.id
    );
}

#[test]
fn cancellation_and_effect_recovery_never_report_false_success() {
    let root = std::env::temp_dir().join(format!("alexia-jobs-{}", alexia::auth::random_id()));
    let mut manager = JobManager::open(&root).unwrap();
    let job = manager
        .enqueue("a", JobRequest::fixture("key-1", json!({})), 100)
        .unwrap();
    manager.cancel("a", &job.id, 101).unwrap();
    assert!(
        manager
            .finish(&job.id, "success", "ok", json!({}), 102)
            .is_err()
    );
    assert_eq!(manager.get("a", &job.id).unwrap().status, Status::Cancelled);
    let effect = manager
        .enqueue("a", JobRequest::fixture("key-2", json!({})), 100)
        .unwrap();
    manager
        .record_effect(&effect.id, "fingerprint", 101)
        .unwrap();
    manager.cancel("a", &effect.id, 102).unwrap();
    drop(manager);
    let manager = JobManager::open(&root).unwrap();
    assert_eq!(
        manager.get("a", &effect.id).unwrap().status,
        Status::Reconciling
    );
    assert_eq!(manager.pending_effects().len(), 1);
}

#[test]
fn abandoned_selections_expire_and_reports_are_physically_compacted() {
    let root = std::env::temp_dir().join(format!("alexia-jobs-{}", alexia::auth::random_id()));
    let mut manager = JobManager::open(&root).unwrap();
    let job = manager
        .enqueue("a", JobRequest::fixture("k", json!({})), 100)
        .unwrap();
    manager
        .wait_for_input(&job.id, "freshness_unknown", json!({"choices":[]}))
        .unwrap();
    manager.maintain(1300).unwrap();
    assert_eq!(manager.get("a", &job.id).unwrap().status, Status::TimedOut);
    manager.maintain(1300 + 86_400).unwrap();
    assert!(manager.get("a", &job.id).is_err());
    drop(manager);
    assert!(JobManager::open(&root).unwrap().list("a").is_empty());
}

#[test]
fn continuation_is_bound_to_owner_revision_choice_and_deadline() {
    let root = std::env::temp_dir().join(format!("alexia-jobs-{}", alexia::auth::random_id()));
    let mut manager = JobManager::open(&root).unwrap();
    let job = manager
        .enqueue("a", JobRequest::fixture("k", json!({})), 100)
        .unwrap();
    let waiting = manager
        .wait_for_input(
            &job.id,
            "freshness_unknown",
            json!({"choices":[{"choice_id":"c","reference":"feature"}]}),
        )
        .unwrap();
    let token = waiting.continuation_id.unwrap();
    assert!(
        manager
            .continue_selection("b", &job.id, &token, waiting.revision, "c", 101)
            .is_err()
    );
    assert!(
        manager
            .continue_selection("a", &job.id, &token, waiting.revision + 1, "c", 101)
            .is_err()
    );
    assert!(
        manager
            .continue_selection("a", &job.id, &token, waiting.revision, "unknown", 101)
            .is_err()
    );
    manager
        .continue_selection("a", &job.id, &token, waiting.revision, "c", 101)
        .unwrap();
    assert!(
        manager
            .continue_selection("a", &job.id, &token, waiting.revision, "c", 102)
            .is_err()
    );
}
