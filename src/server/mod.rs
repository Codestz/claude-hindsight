//! Axum HTTP server for the Hindsight web dashboard
//!
//! Exposes a REST + SSE API and serves the pre-built Next.js static bundle,
//! embedded at compile time via rust-embed.

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
use std::path::PathBuf;
use tower_http::cors::{Any, CorsLayer};

/// The compiled-in Next.js static bundle.
#[derive(RustEmbed)]
#[folder = "web/out/"]
struct WebAssets;

/// Shared application state — no Connection here; each handler opens its own.
#[derive(Clone)]
pub struct AppState {
    #[allow(dead_code)]
    pub db_path: PathBuf,
}

/// Start the axum server on `addr`.
pub async fn serve(addr: SocketAddr) -> anyhow::Result<()> {
    let db_path = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("hindsight")
        .join("sessions.db");

    let state = AppState { db_path };

    let api_router = Router::new()
        .route("/health", get(health))
        .route("/projects", get(routes::projects::list_projects))
        .route("/analytics/global", get(routes::analytics::global_analytics))
        .route("/analytics/global/sparkline", get(routes::analytics::global_sparkline))
        .route("/analytics/:project", get(routes::analytics::project_analytics))
        .route("/sessions", get(routes::sessions::list_sessions))
        .route("/sessions/:id", get(routes::sessions::get_session))
        .route("/sessions/:id/nodes", get(routes::sessions::get_session_nodes))
        .route("/search", get(routes::search::search_sessions))
        .route("/events", get(routes::events::live_events));

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .nest("/api", api_router)
        .layer(cors)
        .with_state(state)
        .fallback(serve_embedded);

    println!("Hindsight web server listening on http://{addr}");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health(State(_): State<AppState>) -> Json<serde_json::Value> {
    Json(json!({ "ok": true, "version": env!("CARGO_PKG_VERSION") }))
}

/// Serve embedded static assets from the compiled-in Next.js bundle.
async fn serve_embedded(uri: Uri) -> Response {
    let raw = uri.path().trim_start_matches('/');
    let path = if raw.is_empty() { "index.html" } else { raw };

    // Exact match.
    if let Some(asset) = WebAssets::get(path) {
        return make_response(StatusCode::OK, path, asset);
    }

    // Next.js trailingSlash pages: try {path}/index.html.
    let index_path = format!("{}/index.html", path.trim_end_matches('/'));
    if let Some(asset) = WebAssets::get(&index_path) {
        return make_response(StatusCode::OK, &index_path, asset);
    }

    // 404 page.
    if let Some(asset) = WebAssets::get("404.html") {
        return make_response(StatusCode::NOT_FOUND, "404.html", asset);
    }

    StatusCode::NOT_FOUND.into_response()
}

fn make_response(status: StatusCode, path: &str, asset: rust_embed::EmbeddedFile) -> Response {
    let mime = mime_guess::from_path(path)
        .first_or_octet_stream()
        .to_string();

    // Hashed _next/static/ assets are immutable; HTML must always revalidate.
    let cache: &'static str = if path.starts_with("_next/static/") {
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
    resp.headers_mut()
        .insert(header::CACHE_CONTROL, header::HeaderValue::from_static(cache));
    resp
}
