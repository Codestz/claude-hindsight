//! Minimal OTLP-only HTTP server for the `hindsight daemon` command
//!
//! Listens on port 7228 (by default) and accepts OTLP http/json payloads at:
//!   POST /v1/metrics
//!   POST /v1/logs
//!
//! When auto-spawned by the SessionStart hook, the daemon uses an idle timeout:
//! if no OTLP data arrives for `IDLE_TIMEOUT_SECS`, the process exits cleanly.
//! Each incoming request resets the timer.

use axum::{routing::post, Router};
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Start the daemon with an optional idle timeout.
/// `idle_timeout_secs = 0` means no timeout (manual Ctrl+C only).
pub async fn serve_with_idle_timeout(addr: SocketAddr, idle_timeout_secs: u64) -> anyhow::Result<()> {
    use super::AppState;
    use super::routes::otel;

    let last_activity = Arc::new(AtomicU64::new(now_secs()));

    let state = AppState {
        last_activity: if idle_timeout_secs > 0 {
            Some(Arc::clone(&last_activity))
        } else {
            None
        },
    };

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
    if idle_timeout_secs > 0 {
        println!("  idle timeout: {}s — will auto-exit if no data received", idle_timeout_secs);
    }

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal(last_activity, idle_timeout_secs))
        .await?;

    println!("Daemon stopped.");
    Ok(())
}

/// Shutdown future: fires on Ctrl+C or idle timeout (whichever comes first).
async fn shutdown_signal(last_activity: Arc<AtomicU64>, idle_timeout_secs: u64) {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };

    if idle_timeout_secs == 0 {
        ctrl_c.await;
        return;
    }

    let idle_check = async {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
            let last = last_activity.load(Ordering::Relaxed);
            let elapsed = now_secs().saturating_sub(last);
            if elapsed >= idle_timeout_secs {
                println!("No OTLP data for {elapsed}s — idle timeout, shutting down.");
                return;
            }
        }
    };

    tokio::select! {
        _ = ctrl_c => {},
        _ = idle_check => {},
    }
}
