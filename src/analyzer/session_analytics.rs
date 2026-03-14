//! Session-level analytics
//!
//! Calculates metrics and statistics for a single session

use crate::parser::models::{ContentBlock, NodeType};
use crate::parser::Session;
use std::collections::HashMap;

/// Analytics for a single session
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SessionAnalytics {
    /// Total number of nodes
    pub total_nodes: usize,

    /// Node counts by type
    pub node_counts: HashMap<NodeType, usize>,

    /// Tool usage breakdown (tool name -> count)
    pub tool_usage: Vec<(String, usize)>,

    /// Session duration in seconds (None if timestamps missing)
    pub duration_seconds: Option<i64>,

    /// Number of errors
    pub error_count: usize,

    /// Whether session uses subagents
    pub has_subagents: bool,

    /// Number of thinking blocks
    pub thinking_count: usize,

    /// Detected model (date suffix stripped)
    pub model: Option<String>,

    /// Total number of tool calls (ToolUse blocks in assistant messages)
    pub tool_call_count: usize,

    /// Number of tool calls that returned an error (tool_result with is_error=true)
    pub tool_result_error_count: usize,
}

impl SessionAnalytics {
    /// Calculate analytics from a session
    pub fn from_session(session: &Session) -> Self {
        let total_nodes = session.nodes.len();

        // Pre-allocate with capacity hints to reduce reallocations
        let mut node_counts: HashMap<NodeType, usize> = HashMap::with_capacity(10);
        let mut tool_counts: HashMap<String, usize> = HashMap::with_capacity(20);
        let mut timestamps = Vec::with_capacity(total_nodes);

        let mut error_count = 0;
        let mut has_subagents = false;
        let mut thinking_count = 0;
        let mut tool_call_count = 0;
        let mut tool_result_error_count = 0;

        for node in &session.nodes {
            // Count by node type
            *node_counts.entry(node.node_type).or_insert(0) += 1;

            // Check for subagents
            if let Some(extra) = node.extra.as_ref().and_then(|e| e.get("isSidechain")) {
                if let Some(is_sidechain) = extra.as_bool() {
                    if is_sidechain {
                        has_subagents = true;
                    }
                }
            }

            // Count thinking blocks using typed ContentBlock matching
            let has_thinking = node.thinking.is_some()
                || node.node_type == NodeType::Unknown // legacy "thinking" type maps to Unknown
                || node
                    .message
                    .as_ref()
                    .map(|m| {
                        m.content_blocks()
                            .iter()
                            .any(|b| matches!(b, ContentBlock::Thinking { .. }))
                    })
                    .unwrap_or(false);

            if has_thinking {
                thinking_count += 1;
            }

            // Count tool usage — top-level tool_use field
            if let Some(ref tool_use) = node.tool_use {
                *tool_counts.entry(tool_use.name.clone()).or_insert(0) += 1;
            }

            // Count tool usage — ToolUse blocks inside assistant message content
            for block in node
                .message
                .as_ref()
                .map(|m| m.content_blocks())
                .unwrap_or(&[])
            {
                if let ContentBlock::ToolUse { name, .. } = block {
                    *tool_counts.entry(name.clone()).or_insert(0) += 1;
                    tool_call_count += 1;
                }
            }

            // Count errors using the consolidated has_error() method
            if node.has_error() {
                error_count += 1;
                // Track tool_result-specific errors separately
                if node.tool_result.as_ref().and_then(|r| r.is_error).unwrap_or(false) {
                    tool_result_error_count += 1;
                }
            }

            // Collect timestamps
            if let Some(ts) = node.timestamp {
                timestamps.push(ts);
            }
        }

        // Model detection: first assistant message with a model field
        let model = session
            .nodes
            .iter()
            .filter_map(|n| n.message.as_ref())
            .filter_map(|m| m.model_short())
            .next()
            .map(str::to_string);

        // Calculate duration
        let duration_seconds = if timestamps.len() >= 2 {
            timestamps.sort_unstable();
            let first = timestamps.first().unwrap();
            let last = timestamps.last().unwrap();
            Some((last - first) / 1000) // Convert ms to seconds
        } else {
            None
        };

        // Sort tools by usage (unstable sort is faster, order of equal elements doesn't matter)
        let mut tool_usage: Vec<_> = tool_counts.into_iter().collect();
        tool_usage.sort_unstable_by(|a, b| b.1.cmp(&a.1));

        SessionAnalytics {
            total_nodes,
            node_counts,
            tool_usage,
            duration_seconds,
            error_count,
            has_subagents,
            thinking_count,
            model,
            tool_call_count,
            tool_result_error_count,
        }
    }

    /// Format duration as human-readable string
    pub fn duration_string(&self) -> String {
        match self.duration_seconds {
            Some(secs) => {
                if secs < 60 {
                    format!("{}s", secs)
                } else if secs < 3600 {
                    format!("{}m {}s", secs / 60, secs % 60)
                } else {
                    format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
                }
            }
            None => "unknown".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_session() {
        let session = Session::new("test-session".to_string(), None, vec![]);

        let analytics = SessionAnalytics::from_session(&session);
        assert_eq!(analytics.total_nodes, 0);
        assert_eq!(analytics.thinking_count, 0);
        assert_eq!(analytics.error_count, 0);
    }

    #[test]
    fn test_duration_formatting() {
        let session = Session::new("test-session".to_string(), None, vec![]);
        let mut analytics = SessionAnalytics::from_session(&session);

        // Test various durations
        analytics.duration_seconds = Some(45);
        assert_eq!(analytics.duration_string(), "45s");

        analytics.duration_seconds = Some(90);
        assert_eq!(analytics.duration_string(), "1m 30s");

        analytics.duration_seconds = Some(3661);
        assert_eq!(analytics.duration_string(), "1h 1m");

        analytics.duration_seconds = None;
        assert_eq!(analytics.duration_string(), "unknown");
    }
}
