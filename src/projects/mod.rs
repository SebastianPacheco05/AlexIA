use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::Read,
    path::{Component, Path, PathBuf},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub owner_id: String,
    pub source: Source,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Source {
    Local {
        canonical_root: PathBuf,
    },
    RemoteRepository {
        connection_id: String,
        repository_id: u64,
        provider_origin: String,
    },
}

impl Project {
    pub fn remote(owner: &str, connection: &str, repository_id: u64) -> Self {
        Self {
            id: format!("{owner}:{connection}:{repository_id}"),
            owner_id: owner.into(),
            source: Source::RemoteRepository {
                connection_id: connection.into(),
                repository_id,
                provider_origin: "https://api.github.com".into(),
            },
        }
    }

    pub fn local(owner: &str, root: &Path) -> Result<Self, &'static str> {
        let canonical_root = root.canonicalize().map_err(|_| "invalid_root")?;
        if !canonical_root.is_dir() {
            return Err("invalid_root");
        }
        Ok(Self {
            id: format!("{owner}:{}", canonical_root.display()),
            owner_id: owner.into(),
            source: Source::Local { canonical_root },
        })
    }

    /// Reads a bounded, explicitly selected text file. Never recursively scans a root.
    pub fn read_context(&self, relative: &str, limit: usize) -> Result<String, &'static str> {
        let Source::Local { canonical_root } = &self.source else {
            return Err("not_local");
        };
        let relative = Path::new(relative);
        if relative.as_os_str().is_empty()
            || relative.components().any(|part| {
                !matches!(part, Component::Normal(_))
                    || part.as_os_str().to_str().is_none_or(secret_path)
            })
        {
            return Err("path_not_allowed");
        }
        let path = canonical_root
            .join(relative)
            .canonicalize()
            .map_err(|_| "invalid_path")?;
        if !path.starts_with(canonical_root) {
            return Err("path_not_allowed");
        }
        let file = fs::File::open(path).map_err(|_| "unavailable")?;
        if !file.metadata().map_err(|_| "unavailable")?.is_file() {
            return Err("invalid_path");
        }
        let mut bytes = Vec::new();
        file.take(limit.min(256 * 1024) as u64 + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| "unavailable")?;
        if bytes.len() > limit.min(256 * 1024) {
            return Err("output_too_large");
        }
        let text = String::from_utf8(bytes).map_err(|_| "not_text")?;
        if text.contains('\0') {
            return Err("not_text");
        }
        Ok(text)
    }
}

fn secret_path(name: &str) -> bool {
    let name = name.to_lowercase();
    name.starts_with(".env")
        || matches!(
            name.as_str(),
            ".git" | ".ssh" | ".aws" | "credentials" | "node_modules" | "target"
        )
        || [".pem", ".key", ".p12", ".pfx"]
            .iter()
            .any(|suffix| name.ends_with(suffix))
        || name.contains(':')
        || name.contains('\\')
}
