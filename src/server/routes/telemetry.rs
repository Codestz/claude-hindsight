//! Telemetry summary routes

use crate::server::{error::ApiError, AppState};
use axum::{extract::State, Json};
use serde::Serialize;

use super::with_index;

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
    with_index(|index| index.get_telemetry_summary()).await
}

pub async fn telemetry_sessions(
    State(_state): State<AppState>,
) -> Result<Json<Vec<SessionTelemetry>>, ApiError> {
    with_index(|index| index.get_telemetry_per_session()).await
}
