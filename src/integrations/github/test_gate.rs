use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Snapshot {
    pub head_sha: String,
    pub base_ref: String,
    pub base_sha: String,
    pub policy_version: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExpectedTest {
    pub name: String,
    pub source_id: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Evidence {
    pub snapshot: Snapshot,
    pub name: String,
    pub source_id: u64,
    pub attempt: u64,
    pub conclusion: String,
}

/// An adapter must prove the association to base and attempt; an unassociated green check is insufficient.
pub fn evaluate(
    snapshot: &Snapshot,
    expected: &[ExpectedTest],
    evidence: &[Evidence],
    complete: bool,
) -> &'static str {
    if expected.is_empty() {
        return "tests_not_configured";
    }
    if !complete {
        return "partial_coverage";
    }
    for test in expected {
        let mut matching: Vec<_> = evidence
            .iter()
            .filter(|item| {
                item.name == test.name
                    && item.source_id == test.source_id
                    && item.snapshot == *snapshot
            })
            .collect();
        matching.sort_by_key(|item| std::cmp::Reverse(item.attempt));
        let Some(latest) = matching.first() else {
            return "tests_not_triggered";
        };
        if matching
            .iter()
            .any(|item| item.attempt == latest.attempt && item.conclusion != latest.conclusion)
        {
            return "ambiguous_evidence";
        }
        match latest.conclusion.as_str() {
            "success" => {}
            "queued" | "pending" | "in_progress" => return "waiting_for_checks",
            _ => return "blocked_checks",
        }
    }
    "passed"
}
