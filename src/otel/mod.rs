//! OTLP JSON parser and extractor for Claude Code telemetry.
//!
//! Provides typed serde structs for the OTLP http/json encoding used by
//! Claude Code, and stateless extraction functions that convert parsed
//! payloads into flat `OtelMetricRecord` / `OtelLogRecord` structs ready
//! for insertion into SQLite.

pub mod extractor;
pub mod parser;

pub use extractor::{extract_log_records, extract_metric_records};

/// Flattened, per-data-point metric record — one row in `otel_metrics`.
#[derive(Debug, Default)]
pub struct OtelMetricRecord {
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

/// Flattened, per-record log record — one row in `otel_logs`.
#[derive(Debug, Default)]
pub struct OtelLogRecord {
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
    pub attributes: Option<String>,
    pub time_unix_nano: Option<String>,
}
