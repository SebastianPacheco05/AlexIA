use gemilexa::{app, config::Config};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_env()?;
    let listener = tokio::net::TcpListener::bind(config.bind_addr).await?;
    eprintln!("GemiLexa listening on {}; Alexa is blocked", listener.local_addr()?);
    axum::serve(listener, app())
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

async fn shutdown_signal() {
    if let Err(error) = tokio::signal::ctrl_c().await {
        eprintln!("Unable to register shutdown signal: {error}");
        std::future::pending::<()>().await;
    }
}
