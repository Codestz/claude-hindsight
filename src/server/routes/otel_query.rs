//! OTLP query routes
//!
//! GET /api/otel/metrics?session_id=&metric=&since=
//! GET /api/otel/logs?session_id=&event=&since=
//! GET /api/otel/session-summary?session_id=
//! GET /api/otel/global-summary

use crate::server::{error::ApiError, AppState};
use crate::storage::SessionIndex;
use axum::{
    extract::{Query, State},
    Json,
};
use serde::{Deserialize, Serialize};

// ── Query param structs ───────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct MetricsQuery {
    pub session_id: Option<String>,
    pub metric: Option<String>,
}

#[derive(Deserialize)]
pub struct LogsQuery {
    pub session_id: Option<String>,
    pub event: Option<String>,
}

#[derive(Deserialize)]
pub struct SessionQuery {
    pub session_id: String,
}

// ── Response DTOs (thin wrappers so serde Serialize applies) ─────────────────

#[derive(Serialize)]
pub struct OtelMetricDto {
    pub received_at: i64,
    pub session_id: Option<String>,
    pub metric_name: String,
    pub token_type: Option<String>,
    pub model: Option<String>,
    pub value_int: Option<i64>,
    pub value_double: Option<f64>,
    pub time_unix_nano: Option<String>,
    pub service_name: Option<String>,
    pub service_version: Option<String>,
}

#[derive(Serialize)]
pub struct OtelLogDto {
    pub received_at: i64,
    pub session_id: Option<String>,
    pub event_name: Option<String>,
    pub model: Option<String>,
    pub cost_usd: Option<f64>,
    pub input_tokens: Option<i64>,
    pub output_tokens: Option<i64>,
    pub cache_read_tokens: Option<i64>,
    pub cache_creation_tokens: Option<i64>,
    pub duration_ms: Option<i64>,
    pub tool_name: Option<String>,
    pub success: Option<bool>,
    pub error_message: Option<String>,
    pub status_code: Option<i64>,
    pub severity: Option<String>,
    pub body: Option<String>,
    pub time_unix_nano: Option<String>,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

pub async fn get_metrics(
    State(_state): State<AppState>,
    Query(q): Query<MetricsQuery>,
) -> Result<Json<Vec<OtelMetricDto>>, ApiError> {
    let session_id = q.session_id.unwrap_or_default();
    let metric = q.metric;

    let rows = tokio::task::spawn_blocking(move || -> crate::error::Result<Vec<OtelMetricDto>> {
        let index = SessionIndex::new()?;
        let records = index.get_otel_metrics(&session_id, metric.as_deref())?;
        Ok(records
            .into_iter()
            .map(|r| OtelMetricDto {
                received_at: r.received_at,
                session_id: r.session_id,
                metric_name: r.metric_name,
                token_type: r.token_type,
                model: r.model,
                value_int: r.value_int,
                value_double: r.value_double,
                time_unix_nano: r.time_unix_nano,
                service_name: r.service_name,
                service_version: r.service_version,
            })
            .collect())
    })
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))??;

    Ok(Json(rows))
}

pub async fn get_logs(
    State(_state): State<AppState>,
    Query(q): Query<LogsQuery>,
) -> Result<Json<Vec<OtelLogDto>>, ApiError> {
    let session_id = q.session_id.unwrap_or_default();
    let event = q.event;

    let rows = tokio::task::spawn_blocking(move || -> crate::error::Result<Vec<OtelLogDto>> {
        let index = SessionIndex::new()?;
        let records = index.get_otel_logs(&session_id, event.as_deref())?;
        Ok(records
            .into_iter()
            .map(|r| OtelLogDto {
                received_at: r.received_at,
                session_id: r.session_id,
                event_name: r.event_name,
                model: r.model,
                cost_usd: r.cost_usd,
                input_tokens: r.input_tokens,
                output_tokens: r.output_tokens,
                cache_read_tokens: r.cache_read_tokens,
                cache_creation_tokens: r.cache_creation_tokens,
                duration_ms: r.duration_ms,
                tool_name: r.tool_name,
                success: r.success,
                error_message: r.error_message,
                status_code: r.status_code,
                severity: r.severity,
                body: r.body,
                time_unix_nano: r.time_unix_nano,
            })
            .collect())
    })
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))??;

    Ok(Json(rows))
}

pub async fn get_session_summary(
    State(_state): State<AppState>,
    Query(q): Query<SessionQuery>,
) -> Result<Json<crate::storage::OtelSessionSummary>, ApiError> {
    let summary = tokio::task::spawn_blocking(
        move || -> crate::error::Result<crate::storage::OtelSessionSummary> {
            let index = SessionIndex::new()?;
            index.get_otel_session_summary(&q.session_id)
        },
    )
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))??;

    Ok(Json(summary))
}

pub async fn get_global_summary(
    State(_state): State<AppState>,
) -> Result<Json<crate::storage::OtelGlobalSummary>, ApiError> {
    let summary = tokio::task::spawn_blocking(
        move || -> crate::error::Result<crate::storage::OtelGlobalSummary> {
            let index = SessionIndex::new()?;
            index.get_otel_global_summary()
        },
    )
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))??;

    Ok(Json(summary))
}
