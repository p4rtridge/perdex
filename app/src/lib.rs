use std::net::SocketAddr;
use axum::{Router, serve::ListenerExt};
use eyre::Result;
use tokio::net::TcpListener;

mod modules;
mod shared;

pub async fn run() -> Result<()> {
    pd_telemetry::init()?;

    let router = Router::new().without_v07_checks();

    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
    let listener = TcpListener::bind(addr)
        .await?
        .tap_io(|stream| {
            // Disable Nagle's algorithm for lower latency at the cost of potentially higher bandwidth usage.
            if let Err(error) = stream.set_nodelay(true) {
                tracing::trace!(?error, "Failed to set TCP_NODELAY for connection");
            }
        });

    tracing::info!("Server listening on {}", addr);

    axum::serve(listener, router).await?;

    Ok(())
}