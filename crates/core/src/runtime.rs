use crate::errors::ProxyError;
use crate::server::{AppState, router};
use axum::serve;
use tokio::net::TcpListener;
use tracing::{error, info};
use veto_config::Config;

/// Entry point for running the proxy server until shutdown.
pub async fn run(config: Config) -> Result<(), ProxyError> {
    let state = AppState::try_from_config(config)?;
    let bind_address = state.bind_address();

    let listener = TcpListener::bind(bind_address)
        .await
        .map_err(ProxyError::Bind)?;
    info!("veto proxy listening on http://{bind_address}");

    let router = router(state);
    serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(ProxyError::Server)
}

async fn shutdown_signal() {
    if let Err(error) = tokio::signal::ctrl_c().await {
        error!("failed to listen for shutdown signal: {error}");
    }
}
