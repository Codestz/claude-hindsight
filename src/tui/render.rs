//! Clean node rendering
//!
//! Simple, maintainable rendering functions for each node type.

use crate::analyzer::TreeNode;
use crate::tui::code_render;
use crate::tui::theme::{colors, icons};
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

/// Render a node's content for the details panel
pub fn render_node_content(node: &TreeNode) -> Vec<Line<'static>> {
    // For assistant messages, check if it's actually a tool call
    if node.node.node_type == "assistant" {
        if let Some(ref msg) = node.node.message {
            if let Some(ref content) = msg.content {
                if let Some(arr) = content.as_array() {
                    if let Some(first_item) = arr.first() {
                        if let Some(item_type) = first_item.get("type").and_then(|t| t.as_str()) {
                            if item_type == "tool_use" {
                                return render_tool_use(node);
                            }
                        }
                    }
                }
            }
        }
    }

    match node.node.node_type.as_str() {
        "user" => render_user(node),
        "assistant" => render_assistant(node),
        "tool_use" => render_tool_use(node),
        "tool_result" => render_tool_result(node),
        "thinking" => render_thinking(node),
        "progress" => render_progress(node),
        "file-history-snapshot" => render_file_snapshot(node),
        "system" => render_system(node),
        _ => render_unknown(node),
    }
}

/// Render user message
fn render_user(node: &TreeNode) -> Vec<Line<'static>> {
    let mut lines = vec![];

    // Check if this is a tool result or user message
    let is_tool_result = if let Some(ref msg) = node.node.message {
        if let Some(ref content) = msg.content {
            if let Some(arr) = content.as_array() {
                arr.iter().any(|item| {
                    item.get("type")
                        .and_then(|t| t.as_str())
                        .map(|t| t == "tool_result")
                        .unwrap_or(false)
                })
            } else {
                false
            }
        } else {
            false
        }
    } else {
        false
    };

    if is_tool_result {
        // This is a tool result - show it differently
        lines.push(Line::from(Span::styled(
            format!("{}  Tool Result", icons::TOOL_RESULT),
            Style::default().fg(colors::TOOL_RESULT).add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));

        if let Some(ref msg) = node.node.message {
            if let Some(ref content) = msg.content {
                if let Some(arr) = content.as_array() {
                    for item in arr {
                        if let Some(item_type) = item.get("type").and_then(|t| t.as_str()) {
                            if item_type == "tool_result" {
                                // Show tool_use_id
                                if let Some(tool_use_id) = item.get("tool_use_id").and_then(|id| id.as_str()) {
                                    lines.push(Line::from(vec![
                                        Span::styled("Tool ID: ", Style::default().fg(Color::Cyan)),
                                        Span::raw(tool_use_id.to_string()),
                                    ]));
                                }

                                // Show error status
                                let is_error = item.get("is_error").and_then(|e| e.as_bool()).unwrap_or(false);
                                lines.push(Line::from(vec![
                                    Span::styled("Status: ", Style::default().fg(Color::Cyan)),
                                    Span::styled(
                                        if is_error { "Error" } else { "Success" },
                                        Style::default().fg(if is_error { Color::Red } else { Color::Green }),
                                    ),
                                ]));

                                lines.push(Line::from(""));

                                // Show content
                                if let Some(result_content) = item.get("content").and_then(|c| c.as_str()) {
                                    if !result_content.is_empty() {
                                        lines.push(Line::from(Span::styled(
                                            "Output:",
                                            Style::default().fg(Color::Cyan),
                                        )));
                                        for line in result_content.lines() {
                                            lines.push(Line::from(line.to_string()));
                                        }
                                    } else {
                                        lines.push(Line::from(Span::styled(
                                            "(no output)",
                                            Style::default().fg(Color::DarkGray),
                                        )));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    } else {
        // Regular user message with icon
        lines.push(Line::from(Span::styled(
            format!("{}  User Message", icons::USER),
            Style::default().fg(colors::USER_MSG).add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));

        // Extract text from message.content array
        if let Some(ref msg) = node.node.message {
            if let Some(ref content) = msg.content {
                let text = extract_text_content(content);
                if !text.is_empty() {
                    for line in text.lines() {
                        lines.push(Line::from(line.to_string()));
                    }
                } else {
                    lines.push(Line::from(Span::styled(
                        "(empty message)",
                        Style::default().fg(Color::DarkGray),
                    )));
                }
            }
        }
    }

    lines
}

/// Render assistant message (with separate thinking and text)
fn render_assistant(node: &TreeNode) -> Vec<Line<'static>> {
    let mut lines = vec![];

    if let Some(ref msg) = node.node.message {
        if let Some(ref content) = msg.content {
            // Check if content is an array
            if let Some(arr) = content.as_array() {
                // Separate thinking and text
                let mut has_thinking = false;
                let mut has_text = false;
                let mut has_tools = false;

                // First pass: render thinking blocks
                for item in arr {
                    if let Some(item_type) = item.get("type").and_then(|t| t.as_str()) {
                        if item_type == "thinking" {
                            if !has_thinking {
                                lines.push(Line::from(Span::styled(
                                    "  Thinking",
                                    Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD),
                                )));
                                lines.push(Line::from(""));
                                has_thinking = true;
                            }

                            if let Some(thinking) = item.get("thinking").and_then(|t| t.as_str()) {
                                for line in thinking.lines() {
                                    lines.push(Line::from(line.to_string()));
                                }
                            }
                        }
                    }
                }

                // Add spacing if we had thinking
                if has_thinking {
                    lines.push(Line::from(""));
                }

                // Second pass: render text content
                for item in arr {
                    if let Some(item_type) = item.get("type").and_then(|t| t.as_str()) {
                        if item_type == "text" {
                            if !has_text {
                                lines.push(Line::from(Span::styled(
                                    "  Response",
                                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                                )));
                                lines.push(Line::from(""));
                                has_text = true;
                            }

                            if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
                                for line in text.lines() {
                                    lines.push(Line::from(line.to_string()));
                                }
                            }
                        } else if item_type == "tool_use" {
                            if !has_tools {
                                if has_text {
                                    lines.push(Line::from(""));
                                }
                                lines.push(Line::from(Span::styled(
                                    "  Tool Calls",
                                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                                )));
                                lines.push(Line::from(""));
                                has_tools = true;
                            }

                            if let Some(name) = item.get("name").and_then(|n| n.as_str()) {
                                lines.push(Line::from(vec![
                                    Span::styled("  • ", Style::default().fg(Color::Yellow)),
                                    Span::raw(name.to_string()),
                                ]));
                            }
                        }
                    }
                }
            } else {
                // Fallback for non-array content
                let text = extract_text_content(content);
                if !text.is_empty() {
                    lines.push(Line::from(Span::styled(
                        "  Response",
                        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                    )));
                    lines.push(Line::from(""));
                    for line in text.lines() {
                        lines.push(Line::from(line.to_string()));
                    }
                }
            }
        }
    }

    if lines.is_empty() {
        lines.push(Line::from(Span::styled(
            "Assistant (no content)",
            Style::default().fg(Color::DarkGray),
        )));
    }

    lines
}

/// Render tool use (also handles assistant messages with tool_use content)
fn render_tool_use(node: &TreeNode) -> Vec<Line<'static>> {
    let mut lines = vec![];

    // Check if this is an assistant message with tool_use content
    if node.node.node_type == "assistant" {
        if let Some(ref msg) = node.node.message {
            if let Some(ref content) = msg.content {
                if let Some(arr) = content.as_array() {
                    for item in arr {
                        if let Some(item_type) = item.get("type").and_then(|t| t.as_str()) {
                            if item_type == "tool_use" {
                                let tool_name = item.get("name").and_then(|n| n.as_str()).unwrap_or("Unknown");

                                lines.push(Line::from(Span::styled(
                                    format!("{}  Tool: {}", icons::TOOL_USE, tool_name),
                                    Style::default().fg(colors::TOOL_USE).add_modifier(Modifier::BOLD),
                                )));
                                lines.push(Line::from(""));

                                // Show tool ID
                                if let Some(tool_id) = item.get("id").and_then(|id| id.as_str()) {
                                    lines.push(Line::from(vec![
                                        Span::styled("ID: ", Style::default().fg(Color::Cyan)),
                                        Span::styled(tool_id.to_string(), Style::default().fg(Color::DarkGray)),
                                    ]));
                                }

                                // Show input parameters in detail
                                if let Some(input) = item.get("input") {
                                    lines.push(Line::from(""));
                                    lines.push(Line::from(Span::styled(
                                        "Parameters:",
                                        Style::default().fg(Color::Cyan),
                                    )));

                                    if let Some(obj) = input.as_object() {
                                        for (key, value) in obj.iter() {
                                            // Format value nicely
                                            let value_str = if let Some(s) = value.as_str() {
                                                s.to_string()
                                            } else {
                                                // For other types, use debug format
                                                format!("{}", value)
                                            };

                                            lines.push(Line::from(vec![
                                                Span::styled("  ", Style::default()),
                                                Span::styled(format!("{}: ", key), Style::default().fg(Color::Yellow)),
                                                Span::raw(value_str),
                                            ]));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    } else if let Some(ref tool_use) = node.node.tool_use {
        // Legacy tool_use node
        lines.push(Line::from(Span::styled(
            format!("  Tool: {}", tool_use.name),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));

        // Show input parameters
        if let Some(obj) = tool_use.input.as_object() {
            lines.push(Line::from(Span::styled(
                "Parameters:",
                Style::default().fg(Color::Cyan),
            )));
            for (key, value) in obj.iter() {
                let value_str = if let Some(s) = value.as_str() {
                    s.to_string()
                } else {
                    format!("{}", value)
                };

                lines.push(Line::from(vec![
                    Span::styled("  ", Style::default()),
                    Span::styled(format!("{}: ", key), Style::default().fg(Color::Yellow)),
                    Span::raw(value_str),
                ]));
            }
        }
    }

    lines
}

/// Render tool result
fn render_tool_result(node: &TreeNode) -> Vec<Line<'static>> {
    let mut lines = vec![];

    if let Some(ref result) = node.node.tool_result {
        let is_error = result.is_error.unwrap_or(false);

        if is_error {
            lines.push(Line::from(Span::styled(
                "  Error",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(""));

            if let Some(ref error) = result.error {
                for line in error.lines() {
                    lines.push(Line::from(Span::styled(
                        line.to_string(),
                        Style::default().fg(Color::Red),
                    )));
                }
            }
        } else {
            lines.push(Line::from(Span::styled(
                "  Success",
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(""));

            if let Some(ref content) = result.content {
                // Try smart rendering for Edit tool results (JSON with file_path, old_string, new_string)
                if let Some(rendered) = code_render::render_edit_result(content) {
                    lines.extend(rendered);
                } else {
                    // Fallback to plain text
                    for line in content.lines() {
                        lines.push(Line::from(line.to_string()));
                    }
                }
            }
        }
    }

    lines
}

/// Render thinking block
fn render_thinking(node: &TreeNode) -> Vec<Line<'static>> {
    let mut lines = vec![];

    lines.push(Line::from(Span::styled(
        format!("{}  Thinking", icons::THINKING),
        Style::default().fg(colors::THINKING).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    if let Some(ref thinking) = node.node.thinking {
        for line in thinking.lines() {
            lines.push(Line::from(line.to_string()));
        }
    }

    lines
}

/// Render progress update
fn render_progress(node: &TreeNode) -> Vec<Line<'static>> {
    let mut lines = vec![];

    lines.push(Line::from(Span::styled(
        format!("{}  Progress", icons::PROGRESS),
        Style::default().fg(colors::PROGRESS).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    // Check for progress subtype in data.type
    if let Some(data) = node.node.extra.as_ref().and_then(|e| e.get("data")) {
        if let Some(progress_type) = data.get("type").and_then(|t| t.as_str()) {
            lines.push(Line::from(vec![
                Span::styled("Type: ", Style::default().fg(Color::Cyan)),
                Span::raw(progress_type.to_string()),
            ]));

            // Show type-specific fields
            match progress_type {
                "bash_progress" => {
                    if let Some(elapsed) = data.get("elapsedTimeSeconds").and_then(|e| e.as_f64()) {
                        lines.push(Line::from(vec![
                            Span::styled("Elapsed: ", Style::default().fg(Color::Cyan)),
                            Span::raw(format!("{:.1}s", elapsed)),
                        ]));
                    }
                    if let Some(output) = data.get("fullOutput").and_then(|o| o.as_str()) {
                        lines.push(Line::from(""));
                        for line in output.lines() {
                            lines.push(Line::from(line.to_string()));
                        }
                    }
                }
                "agent_progress" => {
                    // Show agent ID
                    if let Some(agent_id) = data.get("agentId").and_then(|a| a.as_str()) {
                        lines.push(Line::from(vec![
                            Span::styled("Agent ID: ", Style::default().fg(Color::Cyan)),
                            Span::raw(agent_id.to_string()),
                        ]));
                    }

                    // Show the task/prompt
                    if let Some(prompt) = data.get("prompt").and_then(|p| p.as_str()) {
                        lines.push(Line::from(""));
                        lines.push(Line::from(Span::styled(
                            "Task:",
                            Style::default().fg(Color::Cyan),
                        )));
                        for line in prompt.lines() {
                            lines.push(Line::from(line.to_string()));
                        }
                    }
                }
                _ => {}
            }
        }
    }

    lines
}

/// Render file snapshot
fn render_file_snapshot(node: &TreeNode) -> Vec<Line<'static>> {
    let mut lines = vec![];

    lines.push(Line::from(Span::styled(
        "  File Snapshot",
        Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    if let Some(snapshot) = node.node.extra.as_ref().and_then(|e| e.get("snapshot")) {
        if let Some(tracked_files) = snapshot.get("trackedFileBackups") {
            if let Some(files_obj) = tracked_files.as_object() {
                lines.push(Line::from(format!("{} tracked files:", files_obj.len())));
                lines.push(Line::from(""));

                for (file_path, _) in files_obj.iter().take(15) {
                    lines.push(Line::from(vec![
                        Span::styled("  • ", Style::default().fg(Color::Cyan)),
                        Span::raw(file_path.clone()),
                    ]));
                }

                if files_obj.len() > 15 {
                    lines.push(Line::from(Span::styled(
                        format!("  ... and {} more", files_obj.len() - 15),
                        Style::default().fg(Color::DarkGray),
                    )));
                }
            }
        }
    }

    lines
}

/// Render system node
fn render_system(node: &TreeNode) -> Vec<Line<'static>> {
    let mut lines = vec![];

    lines.push(Line::from(Span::styled(
        "   System",
        Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    if let Some(subtype) = node.node.extra.as_ref().and_then(|e| e.get("subtype")).and_then(|s| s.as_str()) {
        lines.push(Line::from(vec![
            Span::styled("Type: ", Style::default().fg(Color::Cyan)),
            Span::raw(subtype.to_string()),
        ]));

        if subtype == "turn_duration" {
            if let Some(duration_ms) = node.node.extra.as_ref().and_then(|e| e.get("durationMs")).and_then(|d| d.as_i64()) {
                lines.push(Line::from(vec![
                    Span::styled("Duration: ", Style::default().fg(Color::Cyan)),
                    Span::raw(format!("{:.2}s", duration_ms as f64 / 1000.0)),
                ]));
            }
        }
    }

    lines
}

/// Render unknown node type
fn render_unknown(node: &TreeNode) -> Vec<Line<'static>> {
    vec![
        Line::from(Span::styled(
            format!("Unknown: {}", node.node.node_type),
            Style::default().fg(Color::DarkGray),
        )),
    ]
}

/// Extract text content from message.content (handles array and string)
fn extract_text_content(content: &serde_json::Value) -> String {
    match content {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Array(arr) => {
            arr.iter()
                .filter_map(|item| {
                    if let Some(item_type) = item.get("type").and_then(|t| t.as_str()) {
                        match item_type {
                            "text" => item.get("text").and_then(|t| t.as_str()).map(String::from),
                            _ => None,
                        }
                    } else {
                        // Fallback: try direct text field
                        item.get("text").and_then(|t| t.as_str()).map(String::from)
                    }
                })
                .collect::<Vec<_>>()
                .join("\n\n")
        }
        _ => String::new(),
    }
}
