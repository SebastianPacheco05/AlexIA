use rand::{RngCore, rngs::OsRng};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

pub const OWNER: &str = "local_owner";

pub fn random_id() -> String {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub fn digest(value: &[u8]) -> String {
    format!("{:x}", Sha256::digest(value))
}

pub fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

pub struct Session {
    pub token: String,
    pub csrf: String,
    pub principal_id: String,
}

struct Verifier {
    csrf: String,
    principal: String,
    expires_at: u64,
}
struct Pairing {
    client_id: String,
    expires_at: u64,
}

pub struct Sessions {
    bootstrap_hash: Option<String>,
    bootstrap_expires_at: u64,
    attempts: u8,
    sessions: HashMap<String, Verifier>,
    pairing: HashMap<String, Pairing>,
}

impl Sessions {
    pub fn new(now: u64) -> (Self, String) {
        let code = random_id();
        (
            Self {
                bootstrap_hash: Some(digest(code.as_bytes())),
                bootstrap_expires_at: now + 600,
                attempts: 0,
                sessions: HashMap::new(),
                pairing: HashMap::new(),
            },
            code,
        )
    }

    pub fn bootstrap(&mut self, code: &str, now: u64) -> Result<Session, &'static str> {
        if self.attempts >= 10 || now >= self.bootstrap_expires_at {
            return Err("bootstrap_expired");
        }
        self.attempts += 1;
        if self.bootstrap_hash.as_deref() != Some(digest(code.as_bytes()).as_str()) {
            return Err("unauthenticated");
        }
        self.bootstrap_hash = None;
        Ok(self.issue(OWNER.into(), now))
    }

    fn issue(&mut self, principal_id: String, now: u64) -> Session {
        self.sessions.retain(|_, session| session.expires_at > now);
        let token = random_id();
        let csrf = random_id();
        self.sessions.insert(
            digest(token.as_bytes()),
            Verifier {
                csrf: csrf.clone(),
                principal: principal_id.clone(),
                expires_at: now + 28_800,
            },
        );
        Session {
            token,
            csrf,
            principal_id,
        }
    }

    pub fn authenticate(&self, token: &str, csrf: &str, now: u64) -> Result<String, &'static str> {
        let session = self
            .sessions
            .get(&digest(token.as_bytes()))
            .ok_or("unauthenticated")?;
        if now >= session.expires_at || digest(session.csrf.as_bytes()) != digest(csrf.as_bytes()) {
            return Err("unauthenticated");
        }
        Ok(session.principal.clone())
    }

    pub fn pairing_code(
        &mut self,
        token: &str,
        csrf: &str,
        client: &str,
        now: u64,
    ) -> Result<String, &'static str> {
        if self.authenticate(token, csrf, now)? != OWNER || client.is_empty() || client.len() > 100
        {
            return Err("permission_denied");
        }
        self.pairing.retain(|_, pairing| pairing.expires_at > now);
        if self.pairing.len() >= 10 {
            return Err("capacity_exceeded");
        }
        let code = random_id();
        self.pairing.insert(
            digest(code.as_bytes()),
            Pairing {
                client_id: client.into(),
                expires_at: now + 120,
            },
        );
        Ok(code)
    }

    pub fn pair(&mut self, code: &str, client: &str, now: u64) -> Result<Session, &'static str> {
        let hash = digest(code.as_bytes());
        let pairing = self.pairing.get(&hash).ok_or("unauthenticated")?;
        if now >= pairing.expires_at || pairing.client_id != client {
            return Err("unauthenticated");
        }
        self.pairing.remove(&hash);
        Ok(self.issue(format!("paired:{}", random_id()), now))
    }

    pub fn revoke(&mut self, principal: &str) {
        self.sessions.retain(|_, s| s.principal != principal);
    }

    pub fn resume(&mut self, token: &str, now: u64) -> Result<(String, String), &'static str> {
        let session = self
            .sessions
            .get(&digest(token.as_bytes()))
            .ok_or("unauthenticated")?;
        if now >= session.expires_at {
            return Err("unauthenticated");
        }
        Ok((session.principal.clone(), session.csrf.clone()))
    }
}
