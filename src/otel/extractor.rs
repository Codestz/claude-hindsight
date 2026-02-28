//! Stateless extraction: parsed OTLP structs → flat DB records.

use super::parser::{AnyValue, ExportLogsRequest, ExportMetricsRequest, KeyValue};
use super::{OtelLogRecord, OtelMetricRecord};

/// Convert an `ExportMetricsRequest` into flat, per-data-point records.
pub fn extract_metric_records(req: &ExportMetricsRequest) -> Vec<OtelMetricRecord> {
    let now = now_secs();
    let mut out = Vec::new();

    for rm in &req.resource_metrics {
        let session_id = get_attr_string(&rm.resource.attributes, "session.id");
        let service_name = get_attr_string(&rm.resource.attributes, "service.name");
        let service_version = get_attr_string(&rm.resource.attributes, "service.version");

        for sm in &rm.scope_metrics {
            for metric in &sm.metrics {
                // Collect data points from both `sum` and `gauge` variants.
                let data_points: Vec<_> = metric
                    .sum
                    .as_ref()
                    .map(|s| s.data_points.as_slice())
                    .unwrap_or(&[])
                    .iter()
                    .chain(
                        metric
                            .gauge
                            .as_ref()
                            .map(|g| g.data_points.as_slice())
                            .unwrap_or(&[])
                            .iter(),
                    )
                    .collect();

                for dp in data_points {
                    // Point-level session_id overrides resource-level (if present).
                    let dp_session_id = get_attr_string(&dp.attributes, "session.id")
                        .or_else(|| session_id.clone());

                    let value_int = dp
                        .as_int
                        .as_deref()
                        .and_then(|s| s.parse::<i64>().ok());

                    out.push(OtelMetricRecord {
                        received_at: now,
                        session_id: dp_session_id,
                        metric_name: metric.name.clone(),
                        token_type: get_attr_string(&dp.attributes, "type"),
                        model: get_attr_string(&dp.attributes, "model"),
                        value_int,
                        value_double: dp.as_double,
                        time_unix_nano: dp.time_unix_nano.clone(),
                        service_name: service_name.clone(),
                        service_version: service_version.clone(),
                    });
                }
            }
        }
    }

    out
}

/// Convert an `ExportLogsRequest` into flat, per-record structs.
pub fn extract_log_records(req: &ExportLogsRequest) -> Vec<OtelLogRecord> {
    let now = now_secs();
    let mut out = Vec::new();

    for rl in &req.resource_logs {
        let res_session_id = get_attr_string(&rl.resource.attributes, "session.id");

        for sl in &rl.scope_logs {
            for record in &sl.log_records {
                let attrs = &record.attributes;

                let session_id = get_attr_string(attrs, "session.id")
                    .or_else(|| res_session_id.clone());

                let body = record
                    .body
                    .as_ref()
                    .and_then(AnyValue::as_str)
                    .map(str::to_owned);

                // Full attributes as JSON for extensibility.
                let attributes_json = serde_json::to_string(
                    &attrs
                        .iter()
                        .map(|kv| (kv.key.clone(), anyvalue_to_json(&kv.value)))
                        .collect::<serde_json::Map<_, _>>(),
                )
                .ok();

                out.push(OtelLogRecord {
                    received_at: now,
                    session_id,
                    event_name: get_attr_string(attrs, "event.name"),
                    model: get_attr_string(attrs, "model"),
                    cost_usd: get_attr_double(attrs, "cost_usd"),
                    input_tokens: get_attr_int(attrs, "input_tokens"),
                    output_tokens: get_attr_int(attrs, "output_tokens"),
                    cache_read_tokens: get_attr_int(attrs, "cache_read_tokens"),
                    cache_creation_tokens: get_attr_int(attrs, "cache_creation_tokens"),
                    duration_ms: get_attr_int(attrs, "duration_ms"),
                    tool_name: get_attr_string(attrs, "tool_name"),
                    success: get_attr_bool(attrs, "success"),
                    error_message: get_attr_string(attrs, "error"),
                    status_code: get_attr_int(attrs, "status_code"),
                    severity: record.severity_text.clone(),
                    body,
                    attributes: attributes_json,
                    time_unix_nano: record.time_unix_nano.clone(),
                });
            }
        }
    }

    out
}

// ── Attribute helpers ─────────────────────────────────────────────────────────

fn get_attr_string(attrs: &[KeyValue], key: &str) -> Option<String> {
    attrs
        .iter()
        .find(|kv| kv.key == key)
        .and_then(|kv| kv.value.as_str())
        .map(str::to_owned)
}

fn get_attr_int(attrs: &[KeyValue], key: &str) -> Option<i64> {
    attrs.iter().find(|kv| kv.key == key).and_then(|kv| {
        kv.value
            .as_i64()
            .or_else(|| kv.value.as_f64().map(|f| f as i64))
    })
}

fn get_attr_double(attrs: &[KeyValue], key: &str) -> Option<f64> {
    attrs.iter().find(|kv| kv.key == key).and_then(|kv| {
        kv.value
            .as_f64()
            .or_else(|| kv.value.as_i64().map(|i| i as f64))
    })
}

fn get_attr_bool(attrs: &[KeyValue], key: &str) -> Option<bool> {
    attrs
        .iter()
        .find(|kv| kv.key == key)
        .and_then(|kv| kv.value.bool_value)
}

fn anyvalue_to_json(v: &super::parser::AnyValue) -> serde_json::Value {
    if let Some(s) = &v.string_value {
        return serde_json::Value::String(s.clone());
    }
    if let Some(i) = v.as_i64() {
        return serde_json::Value::Number(i.into());
    }
    if let Some(f) = v.as_f64() {
        return serde_json::json!(f);
    }
    if let Some(b) = v.bool_value {
        return serde_json::Value::Bool(b);
    }
    serde_json::Value::Null
}

fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
