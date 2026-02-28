//! OTLP http/json receiver routes
//!
//! POST /v1/metrics  — accepts OTLP metrics payload, parses and stores per-data-point
//! POST /v1/logs     — accepts OTLP logs payload, parses and stores per-record

use crate::server::AppState;
use axum::{extract::State, http::StatusCode, Json};
use serde_json::Value;

pub async fn receive_metrics(
    State(_state): State<AppState>,
    Json(payload): Json<Value>,
) -> StatusCode {
    tokio::task::spawn_blocking(move || {
        let _ = store_metrics(payload);
    });
    StatusCode::OK
}

pub async fn receive_logs(
    State(_state): State<AppState>,
    Json(payload): Json<Value>,
) -> StatusCode {
    tokio::task::spawn_blocking(move || {
        let _ = store_logs(payload);
    });
    StatusCode::OK
}

fn store_metrics(payload: Value) -> crate::error::Result<()> {
    let req: crate::otel::parser::ExportMetricsRequest =
        serde_json::from_value(payload).unwrap_or_default();
    let records = crate::otel::extract_metric_records(&req);
    if records.is_empty() {
        return Ok(());
    }
    let index = crate::storage::SessionIndex::new()?;
    index.insert_otel_metrics(&records)
}

fn store_logs(payload: Value) -> crate::error::Result<()> {
    let req: crate::otel::parser::ExportLogsRequest =
        serde_json::from_value(payload).unwrap_or_default();
    let records = crate::otel::extract_log_records(&req);
    if records.is_empty() {
        return Ok(());
    }
    let index = crate::storage::SessionIndex::new()?;
    index.insert_otel_logs(&records)
}
