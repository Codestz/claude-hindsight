//! Axum HTTP server for the Hindsight web dashboard
//!
//! Exposes a REST + SSE API and serves the pre-built Vite static bundle,
//! embedded at compile time via rust-embed.

pub mod daemon;
pub mod dto;
pub mod error;
pub mod routes;

use axum::{
    extract::State,
    http::{header, StatusCode, Uri},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use rust_embed::RustEmbed;
use serde_json::json;
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};

/// The compiled-in Vite static bundle.
#[derive(RustEmbed)]
#[folder = "web/dist/"]
struct WebAssets;

/// Shared application state — no Connection here; each handler opens its own.
#[derive(Clone)]
pub struct AppState {
    /// Unix epoch seconds of the last OTLP request. Used by the daemon idle-timeout.
    /// `None` for the full `serve` server (no timeout).
    pub last_activity: Option<std::sync::Arc<std::sync::atomic::AtomicU64>>,
}

/// Start the OTLP-only listener (inner server, used by both `serve` and `daemon`).
async fn serve_otlp(addr: SocketAddr) -> anyhow::Result<()> {
    use axum::routing::post;

    let state = AppState { last_activity: None };
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/v1/metrics", post(routes::otel::receive_metrics))
        .route("/v1/logs", post(routes::otel::receive_logs))
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}

/// Start the axum server on `addr`, optionally also starting an OTLP listener.
///
/// `otel_port = 0` disables the embedded OTLP listener.
pub async fn serve(addr: SocketAddr, otel_port: u16) -> anyhow::Result<()> {
    let state = AppState { last_activity: None };

    let api_router = Router::new()
        .route("/health", get(health))
        .route("/projects", get(routes::projects::list_projects))
        .route(
            "/analytics/global",
            get(routes::analytics::global_analytics),
        )
        .route(
            "/analytics/global/sparkline",
            get(routes::analytics::global_sparkline),
        )
        .route(
            "/analytics/global/files",
            get(routes::analytics::global_top_files),
        )
        .route(
            "/analytics/{project}",
            get(routes::analytics::project_analytics),
        )
        .route(
            "/analytics/{project}/files",
            get(routes::analytics::project_top_files),
        )
        .route("/sessions", get(routes::sessions::list_sessions))
        .route("/sessions/{id}", get(routes::sessions::get_session))
        .route(
            "/sessions/{id}/nodes",
            get(routes::sessions::get_session_nodes),
        )
        .route(
            "/sessions/{id}/prompts",
            get(routes::sessions::get_session_prompts),
        )
        .route("/prompts", get(routes::prompts::list_prompts))
        .route("/search", get(routes::search::search_sessions))
        .route("/events", get(routes::events::live_events))
        .route("/sessions/{id}/stream", get(routes::events::session_stream))
        .route("/telemetry/summary", get(routes::telemetry::telemetry_summary))
        .route("/telemetry/sessions", get(routes::telemetry::telemetry_sessions))
        .route("/v1/metrics", axum::routing::post(routes::otel::receive_metrics))
        .route("/v1/logs", axum::routing::post(routes::otel::receive_logs))
        .route("/hooks/tool-events", get(routes::hooks::get_tool_events))
        .route("/hooks/tool-failures", get(routes::hooks::get_tool_failures))
        .route("/hooks/subagent-events", get(routes::hooks::get_subagent_events))
        .route("/hooks/compaction-events", get(routes::hooks::get_compaction_events))
        .route("/hooks/permission-events", get(routes::hooks::get_permission_events))
        .route("/hooks/lifecycle-events", get(routes::hooks::get_lifecycle_events))
        .route("/hooks/activity-summary", get(routes::hooks::get_activity_summary))
        .route("/otel/metrics", get(routes::otel_query::get_metrics))
        .route("/otel/logs", get(routes::otel_query::get_logs))
        .route("/otel/session-summary", get(routes::otel_query::get_session_summary))
        .route("/otel/global-summary", get(routes::otel_query::get_global_summary))
        .route("/agents", get(routes::agents::list_agents))
        .route("/agents/{name}", get(routes::agents::get_agent))
        .route("/skills", get(routes::agents::list_skills))
        .route("/skills/{name}", get(routes::agents::get_skill));

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .nest("/api", api_router)
        .layer(cors)
        .with_state(state)
        .fallback(serve_embedded);

    println!("Hindsight dashboard listening on  http://{addr}");
    if otel_port > 0 {
        println!("Hindsight OTLP receiver listening on http://0.0.0.0:{otel_port}");
    }
    println!("(Ctrl+C to stop)");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    let dashboard_future = axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal());

    if otel_port > 0 {
        let otel_addr: SocketAddr = ([0, 0, 0, 0], otel_port).into();
        tokio::select! {
            r = dashboard_future => r?,
            r = serve_otlp(otel_addr) => r?,
        }
    } else {
        dashboard_future.await?;
    }

    println!("Server stopped.");
    Ok(())
}

async fn health(State(_): State<AppState>) -> Json<serde_json::Value> {
    Json(json!({ "ok": true, "version": env!("CARGO_PKG_VERSION") }))
}

/// Serve embedded static assets from the compiled-in Vite bundle.
/// For non-asset paths, serves index.html so react-router handles client-side routing.
async fn serve_embedded(uri: Uri) -> Response {
    let raw = uri.path().trim_start_matches('/');
    let path = if raw.is_empty() { "index.html" } else { raw };

    // Exact match for static assets (JS, CSS, images, etc.)
    if let Some(asset) = WebAssets::get(path) {
        return make_response(StatusCode::OK, path, asset);
    }

    // SPA fallback: serve index.html for all non-file paths.
    // react-router handles routing on the client side.
    if let Some(asset) = WebAssets::get("index.html") {
        return make_response(StatusCode::OK, "index.html", asset);
    }

    StatusCode::NOT_FOUND.into_response()
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}

fn make_response(status: StatusCode, path: &str, asset: rust_embed::EmbeddedFile) -> Response {
    let mime = mime_guess::from_path(path)
        .first_or_octet_stream()
        .to_string();

    // Hashed assets/ chunks are immutable; HTML must always revalidate.
    let cache: &'static str = if path.starts_with("assets/") {
        "public, max-age=31536000, immutable"
    } else if path.ends_with(".html") {
        "no-cache"
    } else {
        "public, max-age=3600"
    };

    let mut resp = (status, asset.data.to_vec()).into_response();
    resp.headers_mut().insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_str(&mime)
            .unwrap_or_else(|_| header::HeaderValue::from_static("application/octet-stream")),
    );
    resp.headers_mut().insert(
        header::CACHE_CONTROL,
        header::HeaderValue::from_static(cache),
    );
    resp
}
