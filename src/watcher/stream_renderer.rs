//! Terminal rendering for streaming session nodes

use crate::parser::ExecutionNode;
use chrono::{Local, TimeZone};

/// Statistics for the current watch session
#[derive(Debug, Default)]
pub struct WatchStats {
    pub total_nodes: usize,
    pub tools_used: Vec<String>,
    pub errors: usize,
    pub start_time: Option<i64>,
    pub last_update: Option<i64>,
}

impl WatchStats {
    /// Update stats with a new node
    pub fn update(&mut self, node: &ExecutionNode) {
        self.total_nodes += 1;

        // Track tool usage
        if let Some(ref tool_use) = node.tool_use {
            if !self.tools_used.contains(&tool_use.name) {
                self.tools_used.push(tool_use.name.clone());
            }
        }

        // Track errors
        if let Some(ref tool_result) = node.tool_result {
            if tool_result.is_error == Some(true) {
                self.errors += 1;
            }
        }

        // Update timestamps
        if let Some(ts) = node.timestamp {
            if self.start_time.is_none() {
                self.start_time = Some(ts);
            }
            self.last_update = Some(ts);
        }
    }

    /// Format duration from start to last update
    pub fn format_duration(&self) -> String {
        match (self.start_time, self.last_update) {
            (Some(start), Some(end)) => {
                let duration_ms = end - start;
                let duration_s = duration_ms / 1000;

                if duration_s < 60 {
                    format!("{}s", duration_s)
                } else if duration_s < 3600 {
                    format!("{}m {}s", duration_s / 60, duration_s % 60)
                } else {
                    format!("{}h {}m", duration_s / 3600, (duration_s % 3600) / 60)
                }
            }
            _ => "0s".to_string(),
        }
    }
}

/// Render a node to the terminal in a compact format
pub fn render_node(node: &ExecutionNode) {
    let timestamp = format_timestamp(node.timestamp);

    match node.node_type.as_str() {
        "user" => {
            println!("  [{}] USER", timestamp);
            if let Some(ref message) = node.message {
                if let Some(ref content) = message.content {
                    if let Some(text) = content.as_str() {
                        let preview = truncate(text, 80);
                        println!("    {}", preview);
                    }
                }
            }
        }

        "assistant" => {
            println!("  [{}] ASSISTANT", timestamp);
            if let Some(ref message) = node.message {
                if let Some(ref content) = message.content {
                    if let Some(text) = content.as_str() {
                        let preview = truncate(text, 80);
                        println!("    {}", preview);
                    }
                }
            }
        }

        "tool_use" => {
            if let Some(ref tool_use) = node.tool_use {
                println!("  [{}] TOOL: {}", timestamp, tool_use.name);
            } else {
                println!("  [{}] TOOL_USE", timestamp);
            }
        }

        "tool_result" => {
            if let Some(ref tool_result) = node.tool_result {
                if tool_result.is_error == Some(true) {
                    println!("  [{}] TOOL RESULT: \x1b[31mERROR\x1b[0m", timestamp);
                    if let Some(ref error) = tool_result.error {
                        println!("    {}", truncate(error, 80));
                    }
                } else {
                    println!("  [{}] TOOL RESULT: \x1b[32mOK\x1b[0m", timestamp);
                }

                if let Some(duration) = tool_result.duration_ms {
                    println!("    Duration: {}ms", duration);
                }
            }
        }

        "thinking" => {
            println!("  [{}] 💭 THINKING...", timestamp);
            if let Some(ref thinking) = node.thinking {
                let preview = truncate(thinking, 80);
                println!("    {}", preview);
            }
        }

        "progress" => {
            if let Some(ref progress) = node.progress {
                if let Some(ref message) = progress.message {
                    println!("  [{}] ⏳ {}", timestamp, message);
                }
            }
        }

        _ => {
            println!("  [{}] {}", timestamp, node.node_type.to_uppercase());
        }
    }
}

/// Render statistics footer
pub fn render_stats(stats: &WatchStats) {
    println!("\n{}", "=".repeat(80));
    println!(
        "  Nodes: {} | Tools: {} | Errors: {} | Duration: {}",
        stats.total_nodes,
        stats.tools_used.len(),
        stats.errors,
        stats.format_duration()
    );
    println!("{}", "=".repeat(80));
}

/// Format a timestamp for display
fn format_timestamp(timestamp: Option<i64>) -> String {
    match timestamp {
        Some(ts_ms) => {
            let ts_s = ts_ms / 1000;
            let dt = Local.timestamp_opt(ts_s, 0);
            match dt.single() {
                Some(datetime) => datetime.format("%H:%M:%S").to_string(),
                None => "??:??:??".to_string(),
            }
        }
        None => "??:??:??".to_string(),
    }
}

/// Truncate a string to a maximum length with ellipsis
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}
