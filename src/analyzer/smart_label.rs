//! Smart node labeling system
//!
//! Intelligently labels nodes based on content, not just type.

use crate::analyzer::TreeNode;

/// Get smart label for a node (with Nerd Font icon and color)
pub fn get_node_label(node: &TreeNode) -> (String, &'static str) {
    // Detect actual node type by inspecting content
    let (node_type, details) = detect_node_type(node);

    match node_type.as_str() {
        "user_message" => (format!(" User: {}", details), "cyan"),
        "user_tool_result" => (format!(" Tool Result: {}", details), "blue"),
        "assistant_thinking" => (format!(" Thinking"), "magenta"),
        "assistant_text" => (format!(" Assistant: {}", details), "green"),
        "assistant_tool_call" => (format!(" {}", details), "yellow"),
        "progress_bash" => (format!(" Bash: {}", details), "yellow"),
        "progress_hook" => (format!(" Hook: {}", details), "yellow"),
        "progress_agent" => (format!(" Agent: {}", details), "magenta"),
        "progress_other" => (format!(" Progress: {}", details), "yellow"),
        "file_snapshot" => (format!(" Snapshot: {}", details), "magenta"),
        "system" => (format!(" System: {}", details), "gray"),
        "queue_operation" => (format!(" Queued: {}", details), "gray"),
        "unknown" => (format!(" {}", details), "white"),
        _ => (format!(" {}", node.node.node_type), "white"),
    }
}

/// Detect the actual node type by inspecting content
fn detect_node_type(node: &TreeNode) -> (String, String) {
    match node.node.node_type.as_str() {
        "user" => detect_user_type(node),
        "assistant" => detect_assistant_type(node),
        "progress" => detect_progress_type(node),
        "file-history-snapshot" => ("file_snapshot".to_string(), detect_snapshot_details(node)),
        "system" => ("system".to_string(), detect_system_details(node)),
        "queue-operation" => ("queue_operation".to_string(), "queued".to_string()),
        _ => ("unknown".to_string(), node.node.node_type.clone()),
    }
}

/// Detect user message type (user text vs tool result)
fn detect_user_type(node: &TreeNode) -> (String, String) {
    if let Some(ref msg) = node.node.message {
        if let Some(ref content) = msg.content {
            if let Some(arr) = content.as_array() {
                // Check if this is a tool result
                for item in arr {
                    if let Some(item_type) = item.get("type").and_then(|t| t.as_str()) {
                        if item_type == "tool_result" {
                            // Get tool name from tool_use_id if possible
                            let tool_name = item
                                .get("tool_use_id")
                                .and_then(|id| id.as_str())
                                .unwrap_or("unknown");
                            return ("user_tool_result".to_string(), tool_name.to_string());
                        }
                    }
                }

                // Extract text preview for user message
                let text_preview = extract_text_from_content(content, 40);
                return ("user_message".to_string(), text_preview);
            }
        }
    }

    ("user_message".to_string(), String::new())
}

/// Detect assistant message type (thinking, text, or tool call)
fn detect_assistant_type(node: &TreeNode) -> (String, String) {
    if let Some(ref msg) = node.node.message {
        if let Some(ref content) = msg.content {
            if let Some(arr) = content.as_array() {
                // Check what type of content this assistant message has
                if let Some(first_item) = arr.first() {
                    if let Some(item_type) = first_item.get("type").and_then(|t| t.as_str()) {
                        match item_type {
                            "thinking" => {
                                return ("assistant_thinking".to_string(), String::new());
                            }
                            "tool_use" => {
                                let tool_name = first_item
                                    .get("name")
                                    .and_then(|n| n.as_str())
                                    .unwrap_or("Tool");

                                let details = get_tool_details(tool_name, first_item);
                                return ("assistant_tool_call".to_string(), details);
                            }
                            "text" => {
                                let text_preview = first_item
                                    .get("text")
                                    .and_then(|t| t.as_str())
                                    .unwrap_or("")
                                    .chars()
                                    .take(40)
                                    .collect::<String>();
                                return ("assistant_text".to_string(), text_preview);
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    ("assistant_text".to_string(), String::new())
}

/// Get tool-specific details for display
fn get_tool_details(tool_name: &str, tool_use: &serde_json::Value) -> String {
    let input = tool_use.get("input");

    match tool_name {
        "Bash" => {
            if let Some(cmd) = input.and_then(|i| i.get("command")).and_then(|c| c.as_str()) {
                let short_cmd = cmd.chars().take(50).collect::<String>();
                format!("Bash: {}", short_cmd)
            } else {
                "Bash".to_string()
            }
        }
        "Read" => {
            if let Some(path) = input
                .and_then(|i| i.get("file_path"))
                .and_then(|p| p.as_str())
            {
                let file_name = path.rsplit('/').next().unwrap_or(path);
                format!("Read: {}", file_name)
            } else {
                "Read".to_string()
            }
        }
        "Write" | "Edit" => {
            if let Some(path) = input
                .and_then(|i| i.get("file_path"))
                .and_then(|p| p.as_str())
            {
                let file_name = path.rsplit('/').next().unwrap_or(path);
                format!("{}: {}", tool_name, file_name)
            } else {
                tool_name.to_string()
            }
        }
        "Grep" => {
            if let Some(pattern) = input
                .and_then(|i| i.get("pattern"))
                .and_then(|p| p.as_str())
            {
                let short_pattern = pattern.chars().take(30).collect::<String>();
                format!("Grep: {}", short_pattern)
            } else {
                "Grep".to_string()
            }
        }
        "Glob" => {
            if let Some(pattern) = input
                .and_then(|i| i.get("pattern"))
                .and_then(|p| p.as_str())
            {
                format!("Glob: {}", pattern)
            } else {
                "Glob".to_string()
            }
        }
        "Task" => {
            if let Some(desc) = input
                .and_then(|i| i.get("description"))
                .and_then(|d| d.as_str())
            {
                let short_desc = desc.chars().take(40).collect::<String>();
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

/// Detect system event details
fn detect_system_details(node: &TreeNode) -> String {
    if let Some(subtype) = node.node.extra.as_ref().and_then(|e| e.get("subtype")).and_then(|s| s.as_str()) {
        match subtype {
            "turn_duration" => {
                if let Some(duration_ms) = node.node.extra.as_ref().and_then(|e| e.get("durationMs")).and_then(|d| d.as_i64()) {
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

/// Extract text from content array
fn extract_text_from_content(content: &serde_json::Value, max_len: usize) -> String {
    if let Some(arr) = content.as_array() {
        for item in arr {
            if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
                let cleaned = text.replace('\n', " ").trim().to_string();
                let truncated: String = cleaned.chars().take(max_len).collect();
                if cleaned.len() > max_len {
                    return format!("{}...", truncated);
                }
                return truncated;
            }
        }
    }
    String::new()
}
