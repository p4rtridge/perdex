use axum::{Router, serve::ListenerExt};
use error_stack::{Report, ResultExt};
use std::net::SocketAddr;
use tokio::net::TcpListener;

use crate::error::AppError;

pub mod error;
mod modules;
mod shared;

pub async fn run() -> Result<(), Report<AppError>> {
    pd_telemetry::initialize().change_context(AppError)?;

    let router = Router::new().without_v07_checks();

    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
    let listener = TcpListener::bind(addr)
        .await
        .change_context(AppError)?
        .tap_io(|stream| {
            // Disable Nagle's algorithm for lower latency at the cost of potentially higher bandwidth usage.
            if let Err(error) = stream.set_nodelay(true) {
                tracing::trace!(?error, "Failed to set TCP_NODELAY for connection");
            }
        });

    tracing::info!("Server listening on {}", addr);
    axum::serve(listener, router)
        .await
        .change_context(AppError)?;

    Ok(())
}
