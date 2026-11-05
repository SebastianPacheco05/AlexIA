use crate::{
    auth::{digest, random_id},
    storage::Journal,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::Path};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Queued,
    Running,
    WaitingForInput,
    WaitingForExternal,
    Reconciling,
    Completed,
    Failed,
    Cancelled,
    TimedOut,
    Interrupted,
    UnknownOutcome,
}
impl Status {
    pub fn terminal(self) -> bool {
        matches!(
            self,
            Self::Completed
                | Self::Failed
                | Self::Cancelled
                | Self::TimedOut
                | Self::Interrupted
                | Self::UnknownOutcome
        )
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JobRequest {
    pub kind: String,
    pub integration_id: String,
    pub connection_id: Option<String>,
    pub project_id: Option<String>,
    pub capability: String,
    pub input: Value,
    pub idempotency_key: String,
}
impl JobRequest {
    pub fn fixture(key: &str, input: Value) -> Self {
        Self {
            kind: "execution".into(),
            integration_id: "fixture".into(),
            connection_id: None,
            project_id: Some("fixture".into()),
            capability: "fixture.inspect".into(),
            input,
            idempotency_key: key.into(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    pub owner_id: String,
    pub status: Status,
    pub stage: String,
    pub result_status: Option<String>,
    pub result_code: Option<String>,
    pub result: Value,
    pub cancel_requested: bool,
    pub created_at: u64,
    pub finished_at: Option<u64>,
    pub deadline: u64,
    pub request: JobRequest,
    pub input_digest: String,
    pub continuation_id: Option<String>,
    pub revision: u64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct EffectIntent {
    pub job_id: String,
    pub fingerprint: String,
    pub reconciled: bool,
}

pub struct JobManager {
    journal: Journal,
    jobs: BTreeMap<String, Job>,
    effects: BTreeMap<String, EffectIntent>,
}
impl JobManager {
    pub fn open(root: &Path) -> Result<Self, &'static str> {
        let (journal, records) = Journal::open(root, "jobs.jsonl")?;
        let mut manager = Self {
            journal,
            jobs: BTreeMap::new(),
            effects: BTreeMap::new(),
        };
        for record in records {
            if let Some(value) = record.get("job") {
                let job: Job =
                    serde_json::from_value(value.clone()).map_err(|_| "journal_corrupt")?;
                manager.jobs.insert(job.id.clone(), job);
            }
            if let Some(value) = record.get("effect") {
                let effect: EffectIntent =
                    serde_json::from_value(value.clone()).map_err(|_| "journal_corrupt")?;
                manager.effects.insert(effect.job_id.clone(), effect);
            }
        }
        for job in manager.jobs.values_mut() {
            if !job.status.terminal() {
                job.status = if manager
                    .effects
                    .get(&job.id)
                    .is_some_and(|effect| !effect.reconciled)
                {
                    Status::Reconciling
                } else {
                    Status::Interrupted
                };
                job.stage = "recovered_after_restart".into();
                if job.status == Status::Interrupted {
                    job.finished_at = Some(crate::auth::now());
                    job.result_status = Some("error".into());
                    job.result_code = Some("interrupted".into());
                }
                // Recovered jobs never dispatch automatically, even on a second restart.
                manager.journal.append(json!({"job":job}))?;
            }
        }
        Ok(manager)
    }

    fn store(&mut self, mut job: Job) -> Result<Job, &'static str> {
        job.revision += 1;
        self.journal.append(json!({"job":job}))?;
        self.jobs.insert(job.id.clone(), job.clone());
        Ok(job)
    }

    pub fn enqueue(
        &mut self,
        owner: &str,
        request: JobRequest,
        now: u64,
    ) -> Result<Job, &'static str> {
        self.maintain(now)?;
        if request.idempotency_key.is_empty()
            || request.idempotency_key.len() > 512
            || serde_json::to_vec(&request.input)
                .map_err(|_| "invalid_input")?
                .len()
                > 16 * 1024
        {
            return Err("invalid_input");
        }
        if let Some(job) = self.duplicate(owner, &request)? {
            return Ok(job);
        }
        let input_digest = digest(&serde_json::to_vec(&request).map_err(|_| "invalid_input")?);
        if self
            .jobs
            .values()
            .filter(|job| !job.status.terminal())
            .count()
            >= 32
            || self.jobs.len() >= 1000
        {
            return Err("queue_full");
        }
        self.store(Job {
            id: random_id(),
            owner_id: owner.into(),
            status: Status::Queued,
            stage: "queued".into(),
            result_status: None,
            result_code: None,
            result: Value::Null,
            cancel_requested: false,
            created_at: now,
            finished_at: None,
            deadline: now + 1200,
            request,
            input_digest,
            continuation_id: None,
            revision: 0,
        })
    }

    pub fn duplicate(
        &self,
        owner: &str,
        request: &JobRequest,
    ) -> Result<Option<Job>, &'static str> {
        let input_digest = digest(&serde_json::to_vec(request).map_err(|_| "invalid_input")?);
        if let Some(job) = self.jobs.values().find(|job| {
            job.owner_id == owner
                && job.request.integration_id == request.integration_id
                && job.request.connection_id == request.connection_id
                && job.request.project_id == request.project_id
                && job.request.capability == request.capability
                && job.request.idempotency_key == request.idempotency_key
        }) {
            return if job.input_digest == input_digest {
                Ok(Some(job.clone()))
            } else {
                Err("idempotency_conflict")
            };
        }
        Ok(None)
    }

    pub fn get(&self, owner: &str, id: &str) -> Result<Job, &'static str> {
        self.jobs
            .get(id)
            .filter(|job| job.owner_id == owner)
            .cloned()
            .ok_or("not_found")
    }
    pub fn list(&self, owner: &str) -> Vec<Job> {
        self.jobs
            .values()
            .filter(|job| job.owner_id == owner)
            .cloned()
            .collect()
    }
    pub fn pending_effects(&self) -> Vec<EffectIntent> {
        self.effects
            .values()
            .filter(|effect| !effect.reconciled)
            .cloned()
            .collect()
    }

    pub fn maintain(&mut self, now: u64) -> Result<(), &'static str> {
        let mut jobs = self.jobs.clone();
        let mut changed = false;
        for job in jobs.values_mut() {
            if !job.status.terminal() && job.status != Status::Reconciling && now >= job.deadline {
                if self
                    .effects
                    .get(&job.id)
                    .is_some_and(|effect| !effect.reconciled)
                {
                    job.status = Status::Reconciling;
                } else {
                    job.status = Status::TimedOut;
                    job.finished_at = Some(now);
                    job.result_status = Some("error".into());
                    job.result_code = Some("timed_out".into());
                }
                job.continuation_id = None;
                job.revision += 1;
                changed = true;
            }
        }
        let before = jobs.len();
        jobs.retain(|id, job| {
            !job.finished_at
                .is_some_and(|end| now.saturating_sub(end) >= 86_400)
                || self
                    .effects
                    .get(id)
                    .is_some_and(|effect| !effect.reconciled)
        });
        if changed || before != jobs.len() {
            let mut records: Vec<_> = jobs.values().map(|job| json!({"job":job})).collect();
            records.extend(self.effects.values().map(|effect| json!({"effect":effect})));
            self.journal.compact(records)?;
            self.jobs = jobs;
        }
        Ok(())
    }

    pub fn start(&mut self, id: &str, now: u64) -> Result<Job, &'static str> {
        let mut job = self.jobs.get(id).cloned().ok_or("not_found")?;
        if job.cancel_requested || job.status != Status::Queued || now >= job.deadline {
            return Err("job_not_runnable");
        }
        job.status = Status::Running;
        job.stage = "resolving_target".into();
        self.store(job)
    }

    pub fn finish(
        &mut self,
        id: &str,
        result_status: &str,
        code: &str,
        result: Value,
        now: u64,
    ) -> Result<Job, &'static str> {
        if serde_json::to_vec(&result)
            .map_err(|_| "invalid_result")?
            .len()
            > 64 * 1024
        {
            return Err("output_too_large");
        }
        let mut job = self.jobs.get(id).cloned().ok_or("not_found")?;
        if job.cancel_requested
            || job.status.terminal()
            || self
                .effects
                .get(id)
                .is_some_and(|effect| !effect.reconciled)
        {
            return Err("job_not_runnable");
        }
        job.status = if now >= job.deadline {
            Status::TimedOut
        } else if result_status == "error" {
            Status::Failed
        } else {
            Status::Completed
        };
        job.stage = "finished".into();
        job.result_status = Some(result_status.into());
        job.result_code = Some(code.into());
        job.result = result;
        job.finished_at = Some(now);
        self.store(job)
    }

    pub fn wait_for_input(
        &mut self,
        id: &str,
        code: &str,
        result: Value,
    ) -> Result<Job, &'static str> {
        let mut job = self.jobs.get(id).cloned().ok_or("not_found")?;
        if job.cancel_requested || job.status.terminal() {
            return Err("job_not_runnable");
        }
        job.status = Status::WaitingForInput;
        job.stage = "awaiting_selection".into();
        job.result_code = Some(code.into());
        job.result = result;
        job.continuation_id = Some(random_id());
        self.store(job)
    }

    pub fn cancel(&mut self, owner: &str, id: &str, now: u64) -> Result<Job, &'static str> {
        let mut job = self.get(owner, id)?;
        if job.status.terminal() {
            return Ok(job);
        }
        job.cancel_requested = true;
        job.status = if self
            .effects
            .get(id)
            .is_some_and(|effect| !effect.reconciled)
        {
            Status::Reconciling
        } else {
            Status::Cancelled
        };
        if job.status == Status::Cancelled {
            job.finished_at = Some(now);
        }
        self.store(job)
    }

    pub fn continue_selection(
        &mut self,
        owner: &str,
        id: &str,
        continuation: &str,
        revision: u64,
        choice: &str,
        now: u64,
    ) -> Result<Job, &'static str> {
        let mut job = self.get(owner, id)?;
        if job.status != Status::WaitingForInput
            || job.cancel_requested
            || now >= job.deadline
            || job.continuation_id.as_deref() != Some(continuation)
            || job.revision != revision
        {
            return Err("continuation_invalid");
        }
        let selected = job.result["choices"]
            .as_array()
            .and_then(|choices| choices.iter().find(|item| item["choice_id"] == choice))
            .cloned()
            .ok_or("continuation_invalid")?;
        job.result = json!({"selected":selected});
        job.continuation_id = None;
        job.status = Status::Queued;
        job.stage = "selected".into();
        self.store(job)
    }

    /// Caller holds the same manager mutex as cancel/revoke and must revalidate its grant.
    pub fn record_effect(
        &mut self,
        id: &str,
        fingerprint: &str,
        now: u64,
    ) -> Result<(), &'static str> {
        let job = self.jobs.get(id).ok_or("not_found")?;
        if job.cancel_requested
            || job.status.terminal()
            || now >= job.deadline
            || self.effects.contains_key(id)
        {
            return Err("effect_denied");
        }
        let effect = EffectIntent {
            job_id: id.into(),
            fingerprint: fingerprint.into(),
            reconciled: false,
        };
        self.journal.append(json!({"effect":effect}))?;
        self.effects.insert(id.into(), effect);
        Ok(())
    }
}
