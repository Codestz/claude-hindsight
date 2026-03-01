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
    /// OTLP spec encodes 64-bit ints as strings, but some producers (e.g.
    /// Claude Code) send them as native JSON integers. Accept both.
    #[serde(deserialize_with = "deserialize_int_value", default)]
    pub int_value: Option<String>,
    pub double_value: Option<f64>,
    pub bool_value: Option<bool>,
}

/// Accept both `"61"` (string) and `61` (integer) for int_value fields.
fn deserialize_int_value<'de, D>(deserializer: D) -> std::result::Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de;

    struct IntOrString;

    impl<'de> de::Visitor<'de> for IntOrString {
        type Value = Option<String>;

        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("an integer or a string")
        }

        fn visit_i64<E: de::Error>(self, v: i64) -> std::result::Result<Self::Value, E> {
            Ok(Some(v.to_string()))
        }

        fn visit_u64<E: de::Error>(self, v: u64) -> std::result::Result<Self::Value, E> {
            Ok(Some(v.to_string()))
        }

        fn visit_str<E: de::Error>(self, v: &str) -> std::result::Result<Self::Value, E> {
            Ok(Some(v.to_owned()))
        }

        fn visit_none<E: de::Error>(self) -> std::result::Result<Self::Value, E> {
            Ok(None)
        }

        fn visit_unit<E: de::Error>(self) -> std::result::Result<Self::Value, E> {
            Ok(None)
        }
    }

    deserializer.deserialize_any(IntOrString)
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
    /// Token counts — accept both string and integer representations.
    #[serde(deserialize_with = "deserialize_int_value", default)]
    pub as_int: Option<String>,
    /// Cost/rate values arrive as f64.
    pub as_double: Option<f64>,
    pub attributes: Vec<KeyValue>,
    #[serde(deserialize_with = "deserialize_int_value", default)]
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
    #[serde(deserialize_with = "deserialize_int_value", default)]
    pub time_unix_nano: Option<String>,
    pub body: Option<AnyValue>,
    pub attributes: Vec<KeyValue>,
    pub severity_text: Option<String>,
}
