//! Vite is a managed child of `cargo run`, never a separate manual dev command.
use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
    process::Stdio,
    time::Duration,
};
use tokio::{
    process::{Child, Command},
    time::timeout,
};

fn node_path(configured: &str) -> PathBuf {
    if !configured.is_empty() {
        return configured.into();
    }
    let name = if cfg!(windows) { "node.exe" } else { "node" };
    std::env::var_os("PATH")
        .and_then(|paths| {
            std::env::split_paths(&paths)
                .map(|p| p.join(name))
                .find(|p| p.is_file())
        })
        .unwrap_or_else(|| name.into())
}

pub async fn start(
    root: &Path,
    configured_node: &str,
    ui: SocketAddr,
    api: SocketAddr,
) -> Result<Child, String> {
    let node = node_path(configured_node);
    let vite = root.join("ui/node_modules/vite/bin/vite.js");
    if !vite.is_file() {
        let npm = node
            .parent()
            .ok_or("No se encuentra Node.js")?
            .join("node_modules/npm/bin/npm-cli.js");
        if !npm.is_file() {
            return Err("Instala las dependencias con: npm --prefix ui ci".into());
        }
        eprintln!("Instalando las dependencias locales de ui…");
        let status = timeout(
            Duration::from_secs(180),
            Command::new(&node)
                .arg(npm)
                .args(["ci", "--no-fund"])
                .current_dir(root.join("ui"))
                .stdin(Stdio::null())
                .kill_on_drop(true)
                .status(),
        )
        .await
        .map_err(|_| "La instalación de ui tardó demasiado")?
        .map_err(|_| "No se pudo iniciar npm")?;
        if !status.success() {
            return Err("No se pudieron instalar las dependencias de ui".into());
        }
    }
    Command::new(node)
        .arg(vite)
        .args([
            "--host",
            &ui.ip().to_string(),
            "--port",
            &ui.port().to_string(),
            "--strictPort",
        ])
        .current_dir(root.join("ui"))
        .env("ALEXIA_API_URL", format!("http://{api}"))
        .env_remove("__VITE_ADDITIONAL_SERVER_ALLOWED_HOSTS")
        .stdin(Stdio::null())
        .kill_on_drop(true)
        .spawn()
        .map_err(|_| "No se pudo iniciar Vite; comprueba Node.js".into())
}
