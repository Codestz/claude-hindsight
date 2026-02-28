//! Typed serde structs for OTLP JSON payloads (http/json encoding).
//!
//! All fields are optional / default so we stay forward-compatible with
//! new OTLP fields without breaking deserialization.

use serde::Deserialize;

// ── Shared ────────────────────────────────────────────────────────────────────

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Resource {
    pub attributes: Vec<KeyValue>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct KeyValue {
    pub key: String,
    pub value: AnyValue,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct AnyValue {
    pub string_value: Option<String>,
    /// OTLP encodes 64-bit ints as strings to avoid JSON precision loss.
    pub int_value: Option<String>,
    pub double_value: Option<f64>,
    pub bool_value: Option<bool>,
}

impl AnyValue {
    pub fn as_str(&self) -> Option<&str> {
        self.string_value.as_deref()
    }

    pub fn as_i64(&self) -> Option<i64> {
        self.int_value
            .as_deref()
            .and_then(|s| s.parse::<i64>().ok())
    }

    pub fn as_f64(&self) -> Option<f64> {
        self.double_value
    }
}

// ── Metrics ───────────────────────────────────────────────────────────────────

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ExportMetricsRequest {
    pub resource_metrics: Vec<ResourceMetric>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ResourceMetric {
    pub resource: Resource,
    pub scope_metrics: Vec<ScopeMetric>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ScopeMetric {
    pub metrics: Vec<Metric>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Metric {
    pub name: String,
    pub sum: Option<Sum>,
    pub gauge: Option<Gauge>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Sum {
    pub data_points: Vec<NumberDataPoint>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Gauge {
    pub data_points: Vec<NumberDataPoint>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct NumberDataPoint {
    /// Token counts arrive as quoted 64-bit integers.
    pub as_int: Option<String>,
    /// Cost/rate values arrive as f64.
    pub as_double: Option<f64>,
    pub attributes: Vec<KeyValue>,
    pub start_time_unix_nano: Option<String>,
    pub time_unix_nano: Option<String>,
}

// ── Logs ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ExportLogsRequest {
    pub resource_logs: Vec<ResourceLog>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ResourceLog {
    pub resource: Resource,
    pub scope_logs: Vec<ScopeLog>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ScopeLog {
    pub log_records: Vec<LogRecord>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct LogRecord {
    pub time_unix_nano: Option<String>,
    pub body: Option<AnyValue>,
    pub attributes: Vec<KeyValue>,
    pub severity_text: Option<String>,
}
