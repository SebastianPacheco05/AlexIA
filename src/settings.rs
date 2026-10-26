//! Persistent local settings. Never mutate the process environment at runtime.
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    hash::{DefaultHasher, Hash, Hasher},
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

pub type Environment = BTreeMap<String, String>;
static NEXT_WRITE: AtomicU64 = AtomicU64::new(1);
pub const DEFAULT_PROMPT: &str = include_str!("../prompts/meta-prompt.txt");
pub const ENV_KEYS: &[&str] = &[
    "BIND_ADDR",
    "UI_HOST",
    "UI_MODE",
    "UI_PORT",
    "AI_PROVIDER",
    "ALEXA_SKILL_ID",
    "GEMINI_MODEL",
    "GEMINI_LEVEL",
    "AGY_BIN",
    "ALEXA_NODE_BIN",
    "ALEXA_OPENSSL_BIN",
    "ALEXA_VERIFIER_PATH",
];

#[derive(Clone)]
pub struct Document {
    pub environment: Environment,
    pub metaprompt: String,
    pub revision: String,
    original_env: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Update {
    pub revision: String,
    pub environment: BTreeMap<String, Option<String>>,
    pub metaprompt: String,
}

#[derive(Serialize)]
pub struct EnvField {
    pub key: String,
    pub value: Option<String>,
    pub secret: bool,
    pub configured: bool,
}

pub struct Repository {
    root: PathBuf,
    inherited: Environment,
}

impl Repository {
    pub fn new(root: PathBuf) -> Self {
        let inherited = ENV_KEYS
            .iter()
            .filter_map(|key| std::env::var(key).ok().map(|v| (key.to_string(), v)))
            .collect();
        Self { root, inherited }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn read(&self) -> Result<Document, String> {
        let original_env = read_or_default(&self.root.join(".env"), "")?;
        let original_prompt =
            read_or_default(&self.root.join("prompts/meta-prompt.txt"), DEFAULT_PROMPT)?;
        let mut environment = self.inherited.clone();
        for item in dotenvy::from_read_iter(original_env.as_bytes()) {
            let (key, value) = item.map_err(|_| "El archivo .env tiene un formato inválido")?;
            environment.insert(key, value);
        }
        for (key, value) in [
            ("BIND_ADDR", "127.0.0.1:3000"),
            ("UI_HOST", "localhost"),
            ("UI_MODE", "dev"),
            ("UI_PORT", "3001"),
            ("AI_PROVIDER", "antigravity"),
            ("GEMINI_MODEL", "gemini-3.8-flash"),
            ("GEMINI_LEVEL", "low"),
        ] {
            environment
                .entry(key.into())
                .or_insert_with(|| value.into());
        }
        for key in ENV_KEYS {
            environment.entry(key.to_string()).or_default();
        }
        let mut hasher = DefaultHasher::new();
        original_env.hash(&mut hasher);
        original_prompt.hash(&mut hasher);
        Ok(Document {
            environment,
            metaprompt: original_prompt,
            revision: format!("{:016x}", hasher.finish()),
            original_env,
        })
    }

    pub fn propose(&self, current: &Document, update: Update) -> Result<Document, String> {
        if current.revision != update.revision {
            return Err("La configuración cambió. Recarga antes de guardar.".into());
        }
        let mut next = current.clone();
        for (key, value) in update.environment {
            if let Some(value) = value {
                next.environment.insert(key, value);
            }
        }
        next.metaprompt = update.metaprompt;
        next.validate()?;
        Ok(next)
    }

    pub fn persist(&self, before: &Document, next: &Document) -> Result<Document, String> {
        if self.read()?.revision != before.revision {
            return Err(
                "La configuración cambió durante el guardado. Recarga antes de guardar.".into(),
            );
        }
        // Preserve comments, unknown keys and existing secrets. Values are quoted literally.
        let mut content = before
            .original_env
            .lines()
            .filter(|line| line.trim_start().starts_with('#'))
            .collect::<Vec<_>>()
            .join("\n");
        content.push('\n');
        for (key, value) in &next.environment {
            content.push_str(&format!("{key}={}\n", quote(value)));
        }
        atomic_write(&self.root.join(".env"), &content)?;
        if let Err(error) =
            atomic_write(&self.root.join("prompts/meta-prompt.txt"), &next.metaprompt)
        {
            atomic_write(&self.root.join(".env"), &before.original_env)
                .map_err(|_| "No se pudo restaurar .env después de un error de escritura")?;
            return Err(error);
        }
        self.read()
    }
}

impl Document {
    pub fn get(&self, key: &str) -> &str {
        self.environment.get(key).map(String::as_str).unwrap_or("")
    }

    pub fn bind_addr(&self) -> Result<SocketAddr, String> {
        let addr = self
            .get("BIND_ADDR")
            .parse::<SocketAddr>()
            .map_err(|_| "BIND_ADDR debe ser IP:puerto")?;
        if addr.port() == 0 {
            return Err("BIND_ADDR no puede usar el puerto 0".into());
        }
        Ok(addr)
    }

    pub fn ui_addr(&self) -> Result<SocketAddr, String> {
        let host = match self.get("UI_HOST") {
            "localhost" | "127.0.0.1" => "127.0.0.1",
            "::1" | "[::1]" => "[::1]",
            _ => return Err("UI_HOST debe ser localhost o una IP de loopback".into()),
        };
        let port = self
            .get("UI_PORT")
            .parse::<u16>()
            .ok()
            .filter(|p| *p > 0)
            .ok_or("UI_PORT debe estar entre 1 y 65535")?;
        format!("{host}:{port}")
            .parse()
            .map_err(|_| "Dirección UI inválida".into())
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.environment.len() > 128 {
            return Err("Máximo 128 variables de entorno".into());
        }
        for (key, value) in &self.environment {
            let mut chars = key.bytes();
            if key.len() > 80
                || !chars
                    .next()
                    .is_some_and(|c| c.is_ascii_alphabetic() || c == b'_')
                || !chars.all(|c| c.is_ascii_alphanumeric() || c == b'_')
            {
                return Err("Nombre de variable inválido".into());
            }
            if value.len() > 8192 || value.chars().any(|c| c.is_control()) {
                return Err(
                    "Las variables deben ser de una línea y tener como máximo 8192 bytes".into(),
                );
            }
        }
        if self
            .environment
            .iter()
            .map(|(k, v)| k.len() + v.len())
            .sum::<usize>()
            > 65536
        {
            return Err("El entorno supera 64 KiB".into());
        }
        if self.metaprompt.trim().is_empty()
            || self.metaprompt.len() > 16384
            || self.metaprompt.contains('\0')
        {
            return Err("El metaprompt debe contener entre 1 y 16384 bytes de texto".into());
        }
        if self.get("AI_PROVIDER") != "antigravity" {
            return Err("El único proveedor disponible es Antigravity".into());
        }
        if !matches!(self.get("UI_MODE"), "dev" | "static") {
            return Err("UI_MODE debe ser dev o static".into());
        }
        let backend = self.bind_addr()?;
        let ui = self.ui_addr()?;
        if backend.port() == ui.port() {
            return Err("UI_PORT y el puerto de BIND_ADDR deben ser diferentes".into());
        }
        let skill = self.get("ALEXA_SKILL_ID");
        if !skill.is_empty()
            && (!skill.starts_with("amzn1.ask.skill.")
                || skill.len() > 256
                || skill.chars().any(char::is_whitespace))
        {
            return Err("ALEXA_SKILL_ID inválido".into());
        }
        // Reuse the provider's exact model validation, without starting a process.
        crate::providers::antigravity::AntigravityProvider::new("agy".into(), ".".into())
            .with_gemini_settings(self.get("GEMINI_MODEL"), self.get("GEMINI_LEVEL"))?;
        Ok(())
    }

    pub fn fields(&self) -> Vec<EnvField> {
        self.environment
            .iter()
            .map(|(key, value)| {
                let secret = !ENV_KEYS.contains(&key.as_str())
                    || ["TOKEN", "SECRET", "PASSWORD", "API_KEY", "CREDENTIAL"]
                        .iter()
                        .any(|s| key.to_uppercase().contains(s));
                EnvField {
                    key: key.clone(),
                    value: if secret { None } else { Some(value.clone()) },
                    secret,
                    configured: !value.is_empty(),
                }
            })
            .collect()
    }
}

fn read_or_default(path: &Path, default: &str) -> Result<String, String> {
    match fs::read_to_string(path) {
        Ok(s) => Ok(s),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(default.into()),
        Err(_) => Err("No se pudo leer la configuración local".into()),
    }
}

fn quote(value: &str) -> String {
    format!(
        "\"{}\"",
        value
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('$', "\\$")
    )
}

fn atomic_write(path: &Path, content: &str) -> Result<(), String> {
    use std::io::Write;
    let temp = path.with_file_name(format!(
        ".alexia-ui-{}-{}.tmp",
        std::process::id(),
        NEXT_WRITE.fetch_add(1, Ordering::Relaxed)
    ));
    let result = (|| {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)?;
        file.write_all(content.as_bytes())?;
        file.sync_all()?;
        drop(file);
        fs::rename(&temp, path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temp);
    }
    result.map_err(|_| "No se pudo guardar la configuración local".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn repo() -> Repository {
        let root = std::env::temp_dir().join(format!(
            "alexia-settings-{}-{}",
            std::process::id(),
            NEXT_WRITE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(root.join("prompts")).unwrap();
        Repository {
            root,
            inherited: Environment::new(),
        }
    }
    fn update(doc: &Document) -> Update {
        Update {
            revision: doc.revision.clone(),
            environment: BTreeMap::new(),
            metaprompt: doc.metaprompt.clone(),
        }
    }
    #[test]
    fn preserves_secrets_unknown_keys_comments_and_literal_values() {
        let repo = repo();
        fs::write(repo.root.join(".env"), "# keep\nCUSTOM_SECRET='private'\n").unwrap();
        let before = repo.read().unwrap();
        assert!(
            before
                .fields()
                .iter()
                .find(|f| f.key == "CUSTOM_SECRET")
                .unwrap()
                .value
                .is_none()
        );
        let mut change = update(&before);
        change.environment.insert("CUSTOM_SECRET".into(), None);
        change
            .environment
            .insert("AGY_BIN".into(), Some("C:\\it's here\\$agy.exe".into()));
        change.metaprompt = "Responde con claridad.".into();
        let next = repo.propose(&before, change).unwrap();
        let saved = repo.persist(&before, &next).unwrap();
        assert_eq!(saved.get("CUSTOM_SECRET"), "private");
        assert_eq!(saved.get("AGY_BIN"), "C:\\it's here\\$agy.exe");
        assert_eq!(saved.metaprompt, "Responde con claridad.");
        assert!(
            fs::read_to_string(repo.root.join(".env"))
                .unwrap()
                .contains("# keep")
        );
    }
    #[test]
    fn rejects_stale_revision_and_changes_made_during_prepare() {
        let repo = repo();
        let before = repo.read().unwrap();
        let mut change = update(&before);
        change.revision = "stale".into();
        assert!(repo.propose(&before, change).is_err());
        fs::write(repo.root.join(".env"), "NEW_KEY=changed\n").unwrap();
        assert!(repo.persist(&before, &before).is_err());
    }
    #[test]
    fn rejects_invalid_ports_provider_and_nonlocal_ui() {
        let repo = repo();
        let before = repo.read().unwrap();
        assert!(before.validate().is_ok());
        for (key, value) in [
            ("UI_PORT", "0"),
            ("UI_PORT", "3000"),
            ("UI_HOST", "0.0.0.0"),
            ("AI_PROVIDER", "other"),
            ("GEMINI_LEVEL", "fast"),
            ("BAD\nKEY", "value"),
        ] {
            let mut change = update(&before);
            change.environment.insert(key.into(), Some(value.into()));
            assert!(repo.propose(&before, change).is_err(), "{key}");
        }
    }
}
