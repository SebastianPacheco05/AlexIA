use alexia::{
    control::{self, Control},
    settings::Repository,
    ui_process,
};
use std::{future::IntoFuture, sync::Arc};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read .env explicitly: UI edits don't require unsafe process-env mutation.
    let root = std::env::current_dir()?;
    let repository = Repository::new(root.clone());
    let initial = repository.read()?;
    initial.validate()?;
    let backend = tokio::net::TcpListener::bind(initial.bind_addr()?).await?;
    let static_mode = initial.get("UI_MODE") == "static";
    let admin = tokio::net::TcpListener::bind(if static_mode {
        initial.ui_addr()?
    } else {
        "127.0.0.1:0".parse()?
    })
    .await?;
    let admin_addr = admin.local_addr()?;
    let control = Control::open(repository, admin_addr).await?;
    let mut vite = if static_mode {
        None
    } else {
        Some(
            ui_process::start(
                &root,
                initial.get("ALEXA_NODE_BIN"),
                control.ui_addr,
                admin_addr,
            )
            .await?,
        )
    };
    eprintln!("Alexa listening on {}", backend.local_addr()?);
    eprintln!("AlexIA UI: http://localhost:{}", control.ui_addr.port());
    let public = axum::serve(backend, control::public_router(Arc::clone(&control))).into_future();
    let management_router = if static_mode {
        alexia::static_ui::router(control, root)
    } else {
        control::admin_router(control)
    };
    let management = axum::serve(admin, management_router).into_future();
    tokio::select! {
        result = public => result?,
        result = management => result?,
        status = wait_vite(&mut vite) => {
            let status = status?;
            return Err(format!("Vite terminó ({status}). Comprueba UI_PORT y Node.js.").into());
        }
        _ = shutdown_signal() => {}
    }
    if let Some(vite) = vite.as_mut() {
        let _ = vite.kill().await;
        let _ = vite.wait().await;
    }
    Ok(())
}

async fn shutdown_signal() {
    #[cfg(unix)]
    {
        if let Ok(mut terminate) =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        {
            tokio::select! { _=tokio::signal::ctrl_c()=>{}, _=terminate.recv()=>{} }
            return;
        }
    }
    if let Err(error) = tokio::signal::ctrl_c().await {
        eprintln!("Unable to register shutdown signal: {error}");
        std::future::pending::<()>().await;
    }
}

async fn wait_vite(
    vite: &mut Option<tokio::process::Child>,
) -> std::io::Result<std::process::ExitStatus> {
    match vite {
        Some(child) => child.wait().await,
        None => std::future::pending().await,
    }
}
