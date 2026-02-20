//! Smart node labeling system
//!
//! Intelligently labels nodes based on content, not just type.

use crate::analyzer::TreeNode;
use crate::parser::models::ContentBlock;

/// Get smart label for a node (with Nerd Font icon and color).
///
/// `correlation` maps tool_use_id → tool_name so ToolResult labels show the
/// actual tool name ("Tool Result: Read") instead of the raw ID.
pub fn get_node_label(
    node: &TreeNode,
    correlation: Option<&std::collections::HashMap<String, String>>,
) -> (String, &'static str) {
    // Detect actual node type by inspecting content
    let (node_type, details) = detect_node_type(node, correlation);

    match node_type.as_str() {
        "user_message" => (format!(" User {}", details), "cyan"),
        "user_tool_result" => (format!(" Tool Result: {}", details), "blue"),
        "assistant_thinking" => (" Thinking".to_string(), "magenta"),
        "assistant_text" => (format!(" Assistant: {}", details), "green"),
        "assistant_tool_call" => (format!(" {}", details), "yellow"),
        "progress_bash" => (format!(" Bash: {}", details), "yellow"),
        "progress_hook" => (format!(" Hook: {}", details), "yellow"),
        "progress_agent" => (format!(" Agent: {}", details), "magenta"),
        "progress_other" => (format!(" Progress: {}", details), "yellow"),
        "file_snapshot" => (format!(" Snapshot: {}", details), "magenta"),
        "compact_boundary" => (format!("⟐ {}", details), "amber"),
        "system" => (format!(" System: {}", details), "gray"),
        "queue_operation" => (format!(" Queued: {}", details), "gray"),
        "unknown" => (format!(" {}", details), "white"),
        _ => (format!(" {}", node.node.node_type), "white"),
    }
}

/// Detect the actual node type by inspecting content
fn detect_node_type(
    node: &TreeNode,
    correlation: Option<&std::collections::HashMap<String, String>>,
) -> (String, String) {
    match node.node.node_type.as_str() {
        "user" => detect_user_type(node, correlation),
        "assistant" => detect_assistant_type(node),
        "progress" => detect_progress_type(node),
        "file-history-snapshot" => ("file_snapshot".to_string(), detect_snapshot_details(node)),
        "system" => detect_system_type(node),
        "queue-operation" => ("queue_operation".to_string(), "queued".to_string()),
        _ => ("unknown".to_string(), node.node.node_type.clone()),
    }
}

/// Detect user message type (user text vs tool result)
fn detect_user_type(
    node: &TreeNode,
    correlation: Option<&std::collections::HashMap<String, String>>,
) -> (String, String) {
    if let Some(ref msg) = node.node.message {
        // Check for tool result blocks using typed ContentBlock
        let is_tool_result = msg
            .content_blocks()
            .iter()
            .any(|b| matches!(b, ContentBlock::ToolResult { .. }));

        if is_tool_result {
            // Get tool_use_id from the first ToolResult block
            let tool_use_id = msg
                .content_blocks()
                .iter()
                .find_map(|b| {
                    if let ContentBlock::ToolResult { tool_use_id, .. } = b {
                        Some(tool_use_id.as_str())
                    } else {
                        None
                    }
                })
                .unwrap_or("unknown");

            // Prefer tool name from correlation map over the raw ID
            let label = correlation
                .and_then(|c| c.get(tool_use_id))
                .map(|name| name.as_str())
                .unwrap_or(tool_use_id);
            return ("user_tool_result".to_string(), label.to_string());
        }

        // Extract text preview for user message
        let text = msg.text_content();
        let cleaned = text.replace('\n', " ");
        let cleaned = cleaned.trim();
        let preview: String = cleaned.chars().take(40).collect();
        let preview = if cleaned.len() > 40 {
            format!("{}...", preview)
        } else {
            preview
        };
        return ("user_message".to_string(), preview);
    }

    ("user_message".to_string(), String::new())
}

/// Detect assistant message type (thinking, text, or tool call)
///
/// A merged node can contain multiple block types (e.g. Thinking + ToolUse).
/// Priority: if any Thinking block is present, label as "assistant_thinking"
/// since that is the most distinctive feature. The detail panel renders all
/// block types regardless of the label.
fn detect_assistant_type(node: &TreeNode) -> (String, String) {
    if let Some(ref msg) = node.node.message {
        let blocks = msg.content_blocks();

        let has_thinking = blocks
            .iter()
            .any(|b| matches!(b, ContentBlock::Thinking { .. }));
        if has_thinking {
            return ("assistant_thinking".to_string(), String::new());
        }

        // No thinking — find the first significant block
        for block in blocks {
            match block {
                ContentBlock::ToolUse { name, input, .. } => {
                    let details = get_tool_details(name, input);
                    return ("assistant_tool_call".to_string(), details);
                }
                ContentBlock::Text { text } => {
                    let preview: String = text.chars().take(40).collect();
                    return ("assistant_text".to_string(), preview);
                }
                _ => {}
            }
        }
    }

    ("assistant_text".to_string(), String::new())
}

/// Get tool-specific details for display
fn get_tool_details(tool_name: &str, input: &serde_json::Value) -> String {
    match tool_name {
        "Bash" => {
            if let Some(cmd) = input.get("command").and_then(|c| c.as_str()) {
                let short_cmd: String = cmd.chars().take(50).collect();
                format!("Bash: {}", short_cmd)
            } else {
                "Bash".to_string()
            }
        }
        "Read" => {
            if let Some(path) = input.get("file_path").and_then(|p| p.as_str()) {
                let file_name = path.rsplit('/').next().unwrap_or(path);
                format!("Read: {}", file_name)
            } else {
                "Read".to_string()
            }
        }
        "Write" | "Edit" => {
            if let Some(path) = input.get("file_path").and_then(|p| p.as_str()) {
                let file_name = path.rsplit('/').next().unwrap_or(path);
                format!("{}: {}", tool_name, file_name)
            } else {
                tool_name.to_string()
            }
        }
        "Grep" => {
            if let Some(pattern) = input.get("pattern").and_then(|p| p.as_str()) {
                let short_pattern: String = pattern.chars().take(30).collect();
                format!("Grep: {}", short_pattern)
            } else {
                "Grep".to_string()
            }
        }
        "Glob" => {
            if let Some(pattern) = input.get("pattern").and_then(|p| p.as_str()) {
                format!("Glob: {}", pattern)
            } else {
                "Glob".to_string()
            }
        }
        "Task" => {
            if let Some(desc) = input.get("description").and_then(|d| d.as_str()) {
                let short_desc: String = desc.chars().take(40).collect();
                format!("Task: {}", short_desc)
            } else {
                "Task".to_string()
            }
        }
        _ => tool_name.to_string(),
    }
}

/// Detect progress type
fn detect_progress_type(node: &TreeNode) -> (String, String) {
    if let Some(data) = node.node.extra.as_ref().and_then(|e| e.get("data")) {
        if let Some(progress_type) = data.get("type").and_then(|t| t.as_str()) {
            match progress_type {
                "bash_progress" => {
                    let elapsed = data
                        .get("elapsedTimeSeconds")
                        .and_then(|e| e.as_f64())
                        .map(|s| format!("{:.1}s", s))
                        .unwrap_or_else(|| "running".to_string());
                    return ("progress_bash".to_string(), elapsed);
                }
                "hook_progress" => {
                    let hook_name = data
                        .get("hookName")
                        .and_then(|h| h.as_str())
                        .unwrap_or("unknown");
                    return ("progress_hook".to_string(), hook_name.to_string());
                }
                "agent_progress" => {
                    let agent_id = data
                        .get("agentId")
                        .and_then(|a| a.as_str())
                        .unwrap_or("unknown");
                    // Show first 8 chars of agent ID (character-safe)
                    let short_id: String = agent_id.chars().take(8).collect();
                    return ("progress_agent".to_string(), short_id);
                }
                _ => return ("progress_other".to_string(), progress_type.to_string()),
            }
        }
    }

    ("progress_other".to_string(), String::new())
}

/// Detect file snapshot details
fn detect_snapshot_details(node: &TreeNode) -> String {
    if let Some(snapshot) = node.node.extra.as_ref().and_then(|e| e.get("snapshot")) {
        if let Some(tracked_files) = snapshot.get("trackedFileBackups") {
            if let Some(files_obj) = tracked_files.as_object() {
                return format!("{} files", files_obj.len());
            }
        }
    }
    String::new()
}

/// Detect system node type — promotes compact_boundary to a first-class type
fn detect_system_type(node: &TreeNode) -> (String, String) {
    let subtype = node
        .node
        .extra
        .as_ref()
        .and_then(|e| e.get("subtype"))
        .and_then(|s| s.as_str());

    if subtype == Some("compact_boundary") {
        let pre_tokens = node
            .node
            .extra
            .as_ref()
            .and_then(|e| e.get("compactMetadata"))
            .and_then(|m| m.get("preTokens"))
            .and_then(|v| v.as_i64());

        let detail = match pre_tokens {
            Some(t) if t >= 1_000_000 => format!("Compacted · {:.1}M tokens", t as f64 / 1_000_000.0),
            Some(t) if t >= 1_000 => format!("Compacted · {:.1}K tokens", t as f64 / 1_000.0),
            Some(t) => format!("Compacted · {} tokens", t),
            None => "Compacted".to_string(),
        };
        return ("compact_boundary".to_string(), detail);
    }

    ("system".to_string(), detect_system_details(node))
}

/// Detect system event details
fn detect_system_details(node: &TreeNode) -> String {
    if let Some(subtype) = node
        .node
        .extra
        .as_ref()
        .and_then(|e| e.get("subtype"))
        .and_then(|s| s.as_str())
    {
        match subtype {
            "turn_duration" => {
                if let Some(duration_ms) = node
                    .node
                    .extra
                    .as_ref()
                    .and_then(|e| e.get("durationMs"))
                    .and_then(|d| d.as_i64())
                {
                    return format!("Turn {:.1}s", duration_ms as f64 / 1000.0);
                }
                "Turn duration".to_string()
            }
            _ => subtype.to_string(),
        }
    } else {
        "event".to_string()
    }
}
