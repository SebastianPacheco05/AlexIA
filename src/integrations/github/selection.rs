use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PullRequest {
    pub number: u64,
    pub node_id: String,
    pub repository_id: u64,
    pub author_id: u64,
    pub created_at: String,
    pub head_ref: String,
    pub head_sha: String,
    pub base_ref: String,
    pub base_sha: String,
    pub draft: bool,
    pub fork: bool,
    pub url: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreationReceipt {
    pub principal_id: String,
    pub connection_id: String,
    pub repository_id: u64,
    pub pr_node_id: String,
    pub pr_number: u64,
    pub actor_id: u64,
}

pub fn latest_owned<'a>(
    prs: &'a [PullRequest],
    receipts: &[CreationReceipt],
    principal: &str,
    connection: &str,
    user_id: u64,
    complete: bool,
) -> Result<Option<&'a PullRequest>, &'static str> {
    if !complete {
        return Err("partial_coverage");
    }
    let mut owned: Vec<_> = prs
        .iter()
        .filter(|pr| {
            pr.author_id == user_id
                || receipts.iter().any(|receipt| {
                    receipt.principal_id == principal
                        && receipt.connection_id == connection
                        && receipt.repository_id == pr.repository_id
                        && receipt.pr_node_id == pr.node_id
                        && receipt.pr_number == pr.number
                        && receipt.actor_id == pr.author_id
                })
        })
        .collect();
    owned.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    if owned.len() > 1 && owned[0].created_at == owned[1].created_at {
        return Err("ambiguous_target");
    }
    Ok(owned.first().copied())
}
