//! Telemetry summary routes
//!
//! GET /api/telemetry/summary  — total cost + tokens across all sessions
//! GET /api/telemetry/sessions — per-session cost breakdown

use crate::server::{error::ApiError, AppState};
use crate::storage::SessionIndex;
use axum::{extract::State, Json};
use serde::Serialize;

#[derive(Serialize)]
pub struct TelemetrySummary {
    pub total_sessions: usize,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_read_tokens: i64,
    pub cache_creation_tokens: i64,
    pub cost_usd: f64,
}

#[derive(Serialize)]
pub struct SessionTelemetry {
    pub session_id: String,
    pub project_name: String,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_read_tokens: i64,
    pub cache_creation_tokens: i64,
    pub cost_usd: f64,
}

pub async fn telemetry_summary(
    State(_state): State<AppState>,
) -> Result<Json<TelemetrySummary>, ApiError> {
    let summary = tokio::task::spawn_blocking(|| -> crate::error::Result<TelemetrySummary> {
        let index = SessionIndex::new()?;
        index.get_telemetry_summary()
    })
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))??;

    Ok(Json(summary))
}

pub async fn telemetry_sessions(
    State(_state): State<AppState>,
) -> Result<Json<Vec<SessionTelemetry>>, ApiError> {
    let sessions = tokio::task::spawn_blocking(|| -> crate::error::Result<Vec<SessionTelemetry>> {
        let index = SessionIndex::new()?;
        index.get_telemetry_per_session()
    })
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))??;

    Ok(Json(sessions))
}
