use serde::Deserialize;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Deserialize)]
struct Tunnel {
    url: String,
    port: u16,
    updated_at: u64,
}

/// Read only recent heartbeats written by the local tunnel launcher.
pub fn endpoint(root: &Path, port: u16) -> Option<String> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_secs();
    let mut entries: Vec<_> = std::fs::read_dir(root.join("target/tunnels"))
        .ok()?
        .filter_map(Result::ok)
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "json"))
        .collect();
    entries.sort_by_key(|entry| {
        std::cmp::Reverse(entry.metadata().ok().and_then(|m| m.modified().ok()))
    });
    entries.into_iter().take(32).find_map(|entry| {
        if entry.metadata().ok()?.len() > 2048 {
            return None;
        }
        let tunnel: Tunnel = serde_json::from_slice(&std::fs::read(entry.path()).ok()?).ok()?;
        if tunnel.port != port
            || tunnel.updated_at > now + 5
            || now.saturating_sub(tunnel.updated_at) > 10
        {
            return None;
        }
        let label = tunnel
            .url
            .strip_prefix("https://")?
            .strip_suffix(".trycloudflare.com")?;
        if label.is_empty()
            || label.len() > 63
            || label.starts_with('-')
            || label.ends_with('-')
            || !label
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || c == b'-')
        {
            return None;
        }
        Some(format!("{}/alexa", tunnel.url))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn only_fresh_matching_quick_tunnels_are_shown() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let root = std::env::temp_dir().join(format!("alexia-tunnel-{}-{now}", std::process::id()));
        let directory = root.join("target/tunnels");
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("test.json");
        let write = |url: &str, port: u16, updated_at: u64| {
            std::fs::write(
                &path,
                serde_json::json!({"url":url,"port":port,"updated_at":updated_at}).to_string(),
            )
            .unwrap();
        };
        assert_eq!(endpoint(&root, 3000), None);
        write("https://test-example.trycloudflare.com", 3000, now);
        assert_eq!(
            endpoint(&root, 3000).as_deref(),
            Some("https://test-example.trycloudflare.com/alexa")
        );
        assert_eq!(endpoint(&root, 3001), None);
        write("https://test-example.trycloudflare.com", 3000, now - 30);
        assert_eq!(endpoint(&root, 3000), None);
        for url in [
            "http://test.trycloudflare.com",
            "https://evil.example",
            "https://evil.example@x.trycloudflare.com",
            "https://x.trycloudflare.com/other",
            "https://x.trycloudflare.com?next=x",
            "https://-x.trycloudflare.com",
        ] {
            write(url, 3000, now);
            assert_eq!(endpoint(&root, 3000), None, "{url}");
        }
        std::fs::write(&path, "broken json").unwrap();
        assert_eq!(endpoint(&root, 3000), None);
        std::fs::remove_file(path).unwrap();
    }
}
