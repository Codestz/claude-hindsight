//! Minimal OTLP-only HTTP server for the `hindsight daemon` command
//!
//! Listens on port 7228 (by default) and accepts OTLP http/json payloads at:
//!   POST /v1/metrics
//!   POST /v1/logs

use axum::{routing::post, Router};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};

pub async fn serve(addr: SocketAddr) -> anyhow::Result<()> {
    use super::AppState;
    use super::routes::otel;

    let state = AppState {};

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/v1/metrics", post(otel::receive_metrics))
        .route("/v1/logs", post(otel::receive_logs))
        .layer(cors)
        .with_state(state);

    println!("Hindsight OTLP daemon listening on http://{addr}  (Ctrl+C to stop)");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;

    println!("Daemon stopped.");
    Ok(())
}
