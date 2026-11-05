use alexia::integrations::github::{
    selection::{CreationReceipt, PullRequest, latest_owned},
    test_gate::{Evidence, ExpectedTest, Snapshot, evaluate},
};

fn pr(number: u64, author: u64, created: &str) -> PullRequest {
    PullRequest {
        number,
        node_id: format!("pr-{number}"),
        repository_id: 1,
        author_id: author,
        created_at: created.into(),
        head_ref: "feature/demo".into(),
        head_sha: "a".repeat(40),
        base_ref: "main".into(),
        base_sha: "b".repeat(40),
        draft: false,
        fork: false,
        url: format!("https://github.com/a/b/pull/{number}"),
    }
}

#[test]
fn own_pr_uses_stable_author_id_or_exact_receipt_and_never_skips_latest_draft() {
    let mut newest = pr(3, 7, "2026-11-03T12:00:00Z");
    newest.draft = true;
    let prs = vec![
        pr(1, 7, "2026-11-01T00:00:00Z"),
        pr(2, 8, "2026-11-02T00:00:00Z"),
        newest,
    ];
    assert_eq!(
        latest_owned(&prs, &[], "a", "c", 7, true)
            .unwrap()
            .unwrap()
            .number,
        3
    );
    assert!(latest_owned(&prs, &[], "a", "c", 7, false).is_err());
    let bot = pr(4, 99, "2026-11-03T13:00:00Z");
    let receipt = CreationReceipt {
        principal_id: "a".into(),
        connection_id: "c".into(),
        repository_id: 1,
        pr_node_id: bot.node_id.clone(),
        pr_number: 4,
        actor_id: 99,
    };
    assert!(
        latest_owned(&[bot.clone()], &[receipt.clone()], "b", "c", 7, true)
            .unwrap()
            .is_none()
    );
    assert_eq!(
        latest_owned(&[bot], &[receipt], "a", "c", 7, true)
            .unwrap()
            .unwrap()
            .number,
        4
    );
}

#[test]
fn gate_requires_every_expected_test_on_exact_snapshot_source_and_latest_attempt() {
    let snapshot = Snapshot {
        head_sha: "a".into(),
        base_ref: "develop".into(),
        base_sha: "b".into(),
        policy_version: 1,
    };
    let expected = vec![ExpectedTest {
        name: "tests".into(),
        source_id: 42,
    }];
    assert_eq!(evaluate(&snapshot, &[], &[], true), "tests_not_configured");
    assert_eq!(
        evaluate(&snapshot, &expected, &[], true),
        "tests_not_triggered"
    );
    let good = Evidence {
        snapshot: snapshot.clone(),
        name: "tests".into(),
        source_id: 42,
        attempt: 1,
        conclusion: "success".into(),
    };
    assert_eq!(
        evaluate(&snapshot, &expected, &[good.clone()], true),
        "passed"
    );
    for result in ["skipped", "neutral", "failure", "cancelled", "timed_out"] {
        let mut failed = good.clone();
        failed.conclusion = result.into();
        failed.attempt = 2;
        assert_ne!(
            evaluate(&snapshot, &expected, &[good.clone(), failed], true),
            "passed"
        );
    }
    let mut spoofed = good.clone();
    spoofed.source_id = 100;
    assert_ne!(evaluate(&snapshot, &expected, &[spoofed], true), "passed");
    let mut stale = good.clone();
    stale.snapshot.base_sha = "old".into();
    assert_ne!(evaluate(&snapshot, &expected, &[stale], true), "passed");
    assert_ne!(evaluate(&snapshot, &expected, &[good], false), "passed");
}
