//! Clean node rendering
//!
//! Simple, maintainable rendering functions for each node type.

use crate::analyzer::TreeNode;
use crate::parser::models::ContentBlock;
use crate::tui::code_render;
use crate::tui::theme::{colors, icons};
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};
use std::collections::HashMap;

/// Context passed to all renderers carrying session-level data.
pub struct RenderContext<'a> {
    /// tool_use_id → tool_name (for matching ToolResult with its ToolUse)
    pub tool_correlation: &'a HashMap<String, String>,
    /// tool_use_id → brief result summary (✓ 42 lines / ✗ error msg)
    pub tool_result_map: &'a HashMap<String, String>,
}

/// Helper to render regular tool parameters
fn render_parameters(lines: &mut Vec<Line<'static>>, input: &serde_json::Value) {
    lines.push(Line::from(Span::styled(
        "Parameters:",
        Style::default().fg(Color::Cyan),
    )));

    if let Some(obj) = input.as_object() {
        for (key, value) in obj.iter() {
            lines.push(Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(format!("{}: ", key), Style::default().fg(Color::Yellow)),
            ]));
            // Render value line-by-line so multiline strings (old_string, new_string, etc.) display fully
            let value_str = if let Some(s) = value.as_str() {
                s.to_string()
            } else {
                format!("{}", value)
            };
            for line in value_str.lines() {
                lines.push(Line::from(vec![
                    Span::styled("    ", Style::default()),
                    Span::raw(line.to_string()),
                ]));
            }
        }
    }
}

/// Render a node's content for the details panel
pub fn render_node_content(node: &TreeNode, ctx: &RenderContext) -> Vec<Line<'static>> {
    match node.node.node_type.as_str() {
        "user" => render_user(node, ctx),
        // All assistant nodes (including those with thinking + tool_use merged blocks)
        // go through render_assistant which handles every block type in order.
        "assistant" => render_assistant(node, ctx),
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
fn render_user(node: &TreeNode, ctx: &RenderContext) -> Vec<Line<'static>> {
    let mut lines = vec![];

    // Check if this is a tool result using typed blocks
    let is_tool_result = node
        .node
        .message
        .as_ref()
        .map(|m| {
            m.content_blocks()
                .iter()
                .any(|b| matches!(b, ContentBlock::ToolResult { .. }))
        })
        .unwrap_or(false);

    if is_tool_result {
        if let Some(ref msg) = node.node.message {
            for block in msg.content_blocks() {
                if let ContentBlock::ToolResult {
                    tool_use_id,
                    content,
                    is_error,
                } = block
                {
                    // Look up matched tool name from correlation map
                    let tool_name = ctx
                        .tool_correlation
                        .get(tool_use_id.as_str())
                        .map(|s| s.as_str())
                        .unwrap_or("Unknown");

                    // Header: "Tool Result ← ToolName  [id]"
                    lines.push(Line::from(vec![
                        Span::styled(
                            format!("{}  Tool Result ← ", icons::TOOL_RESULT),
                            Style::default()
                                .fg(colors::TOOL_RESULT)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(
                            tool_name.to_string(),
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(
                            format!("  [{}]", tool_use_id),
                            Style::default().fg(Color::DarkGray),
                        ),
                    ]));
                    lines.push(Line::from(""));

                    // Show error status
                    let err = is_error.unwrap_or(false);
                    lines.push(Line::from(vec![
                        Span::styled("Status: ", Style::default().fg(Color::Cyan)),
                        Span::styled(
                            if err { "Error" } else { "Success" },
                            Style::default()
                                .fg(if err { Color::Red } else { Color::Green }),
                        ),
                    ]));

                    lines.push(Line::from(""));

                    // Show content — prefer clean toolUseResult over block content
                    let (result_str, file_path) =
                        if let Some(ref tool_use_result) = node.node.tool_use_result {
                            if let Ok(result) = serde_json::from_value::<
                                crate::parser::models::ToolResult,
                            >(tool_use_result.clone())
                            {
                                if let Some(ref file) = result.file {
                                    (file.content.clone(), file.file_path.clone())
                                } else {
                                    (result.content.clone(), None)
                                }
                            } else {
                                // Fallback: extract string from block content
                                let s = content
                                    .as_ref()
                                    .and_then(|c| c.as_str())
                                    .map(String::from);
                                (s, None)
                            }
                        } else {
                            let s = content
                                .as_ref()
                                .and_then(|c| c.as_str())
                                .map(String::from);
                            (s, None)
                        };

                    if let Some(result_content) = result_str {
                        if !result_content.is_empty() {
                            lines.push(Line::from(Span::styled(
                                "Output:",
                                Style::default().fg(Color::Cyan),
                            )));

                            let language = file_path
                                .as_ref()
                                .and_then(|path| code_render::detect_language(path));
                            let highlighted_lines =
                                code_render::highlight_code(&result_content, language);
                            lines.extend(highlighted_lines);
                        } else {
                            lines.push(Line::from(Span::styled(
                                "(no output)",
                                Style::default().fg(Color::DarkGray),
                            )));
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
    } else {
        // Regular user message with icon
        lines.push(Line::from(Span::styled(
            format!("{}  User Message", icons::USER),
            Style::default()
                .fg(colors::USER_MSG)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));

        // Extract text from message using typed helper
        if let Some(ref msg) = node.node.message {
            let text = msg.text_content();
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

    lines
}

/// Render assistant message (with separate thinking and text)
fn render_assistant(node: &TreeNode, ctx: &RenderContext) -> Vec<Line<'static>> {
    let mut lines = vec![];

    if let Some(ref msg) = node.node.message {
        let blocks = msg.content_blocks();

        if !blocks.is_empty() {
            let mut has_thinking = false;
            let mut has_text = false;
            let mut has_tools = false;

            // First pass: render thinking blocks (always expanded)
            for block in blocks {
                if let ContentBlock::Thinking { thinking, .. } = block {
                    if !has_thinking {
                        lines.push(Line::from(Span::styled(
                            "  Extended Thinking",
                            Style::default()
                                .fg(Color::Blue)
                                .add_modifier(Modifier::BOLD),
                        )));
                        lines.push(Line::from(""));
                        has_thinking = true;
                    }
                    for line in thinking.lines() {
                        lines.push(Line::from(line.to_string()));
                    }
                }
            }

            if has_thinking {
                lines.push(Line::from(""));
            }

            // Second pass: render text and tool_use blocks
            for block in blocks {
                match block {
                    ContentBlock::Text { text } => {
                        if !has_text {
                            lines.push(Line::from(Span::styled(
                                "  Response",
                                Style::default()
                                    .fg(Color::Green)
                                    .add_modifier(Modifier::BOLD),
                            )));
                            lines.push(Line::from(""));
                            has_text = true;
                        }
                        for line in text.lines() {
                            lines.push(Line::from(line.to_string()));
                        }
                    }
                    ContentBlock::ToolUse { id, name, input } => {
                        if !has_tools {
                            if has_text {
                                lines.push(Line::from(""));
                            }
                            lines.push(Line::from(Span::styled(
                                "  Tool Calls",
                                Style::default()
                                    .fg(Color::Yellow)
                                    .add_modifier(Modifier::BOLD),
                            )));
                            lines.push(Line::from(""));
                            has_tools = true;
                        }
                        lines.push(Line::from(vec![
                            Span::styled(
                                format!("{}  Tool: {}", icons::TOOL_USE, name),
                                Style::default().fg(colors::TOOL_USE).add_modifier(Modifier::BOLD),
                            ),
                        ]));
                        lines.push(Line::from(vec![
                            Span::styled("ID: ", Style::default().fg(Color::Cyan)),
                            Span::styled(id.clone(), Style::default().fg(Color::DarkGray)),
                        ]));
                        lines.push(Line::from(""));
                        if name == "Edit" {
                            if let Some(rendered) = code_render::render_edit_result(&input.to_string()) {
                                lines.extend(rendered);
                            } else {
                                render_parameters(&mut lines, input);
                            }
                        } else {
                            render_parameters(&mut lines, input);
                        }
                        // Brief result summary
                        if let Some(summary) = ctx.tool_result_map.get(id.as_str()) {
                            lines.push(Line::from(""));
                            let is_err = summary.starts_with('✗');
                            lines.push(Line::from(vec![
                                Span::styled(
                                    "Result: ",
                                    Style::default().fg(Color::DarkGray),
                                ),
                                Span::styled(
                                    summary.clone(),
                                    Style::default().fg(if is_err { Color::Red } else { Color::Green }),
                                ),
                            ]));
                        }
                        lines.push(Line::from(""));
                    }
                    _ => {}
                }
            }
        } else {
            // Fallback for legacy string content
            let text = msg.text_content();
            if !text.is_empty() {
                lines.push(Line::from(Span::styled(
                    "  Response",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                )));
                lines.push(Line::from(""));
                for line in text.lines() {
                    lines.push(Line::from(line.to_string()));
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

    // Check if this is an assistant message with tool_use content blocks
    if node.node.node_type == "assistant" {
        if let Some(ref msg) = node.node.message {
            for block in msg.content_blocks() {
                if let ContentBlock::ToolUse { id, name, input } = block {
                    lines.push(Line::from(Span::styled(
                        format!("{}  Tool: {}", icons::TOOL_USE, name),
                        Style::default()
                            .fg(colors::TOOL_USE)
                            .add_modifier(Modifier::BOLD),
                    )));
                    lines.push(Line::from(""));

                    // Show tool ID
                    lines.push(Line::from(vec![
                        Span::styled("ID: ", Style::default().fg(Color::Cyan)),
                        Span::styled(id.clone(), Style::default().fg(Color::DarkGray)),
                    ]));

                    // Show input parameters with smart rendering for Edit tool
                    lines.push(Line::from(""));

                    if name == "Edit" {
                        if let Some(rendered) =
                            code_render::render_edit_result(&input.to_string())
                        {
                            lines.extend(rendered);
                        } else {
                            render_parameters(&mut lines, input);
                        }
                    } else if name == "Read" {
                        if let Some(file_path) =
                            input.get("file_path").and_then(|v| v.as_str())
                        {
                            lines.push(Line::from(vec![
                                Span::styled(
                                    " Reading: ",
                                    Style::default()
                                        .fg(Color::Cyan)
                                        .add_modifier(Modifier::BOLD),
                                ),
                                Span::styled(
                                    file_path.to_string(),
                                    Style::default().fg(Color::Yellow),
                                ),
                            ]));
                        }
                    } else {
                        render_parameters(&mut lines, input);
                    }
                }
            }
        }
    } else if let Some(ref tool_use) = node.node.tool_use {
        // Legacy tool_use node
        lines.push(Line::from(Span::styled(
            format!("  Tool: {}", tool_use.name),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
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

    // Try to parse toolUseResult from JSON value (if it's an object, not error string)
    let tool_use_result: Option<crate::parser::models::ToolResult> = node
        .node
        .tool_use_result
        .as_ref()
        .and_then(|v| serde_json::from_value(v.clone()).ok());

    // Prefer toolUseResult (clean content) over tool_result (has line numbers)
    let result = tool_use_result.as_ref().or(node.node.tool_result.as_ref());

    if let Some(result) = result {
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
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(""));

            // Prefer file.content (clean) over content (may have line numbers)
            let (content_str, file_path) = if let Some(ref file) = result.file {
                (file.content.as_ref(), file.file_path.as_ref())
            } else {
                (result.content.as_ref(), None)
            };

            if let Some(content) = content_str {
                // Detect language from file path if available
                let language = file_path.and_then(|path| code_render::detect_language(path));

                // Try smart rendering for Edit tool results
                if let Some(rendered) = code_render::render_edit_result(content) {
                    lines.extend(rendered);
                } else {
                    let highlighted_lines = code_render::highlight_code(content, language);
                    lines.extend(highlighted_lines);
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
        Style::default()
            .fg(colors::THINKING)
            .add_modifier(Modifier::BOLD),
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
        Style::default()
            .fg(colors::PROGRESS)
            .add_modifier(Modifier::BOLD),
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
                    if let Some(elapsed) =
                        data.get("elapsedTimeSeconds").and_then(|e| e.as_f64())
                    {
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
                    if let Some(agent_id) = data.get("agentId").and_then(|a| a.as_str()) {
                        lines.push(Line::from(vec![
                            Span::styled("Agent ID: ", Style::default().fg(Color::Cyan)),
                            Span::raw(agent_id.to_string()),
                        ]));
                    }

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
        Style::default()
            .fg(Color::Magenta)
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    if let Some(snapshot) = node.node.extra.as_ref().and_then(|e| e.get("snapshot")) {
        if let Some(tracked_files) = snapshot.get("trackedFileBackups") {
            if let Some(files_obj) = tracked_files.as_object() {
                lines.push(Line::from(format!("{} tracked files:", files_obj.len())));
                lines.push(Line::from(""));

                for (file_path, _) in files_obj.iter() {
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
        Style::default()
            .fg(Color::Gray)
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    if let Some(subtype) = node
        .node
        .extra
        .as_ref()
        .and_then(|e| e.get("subtype"))
        .and_then(|s| s.as_str())
    {
        lines.push(Line::from(vec![
            Span::styled("Type: ", Style::default().fg(Color::Cyan)),
            Span::raw(subtype.to_string()),
        ]));

        if subtype == "turn_duration" {
            if let Some(duration_ms) = node
                .node
                .extra
                .as_ref()
                .and_then(|e| e.get("durationMs"))
                .and_then(|d| d.as_i64())
            {
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
    vec![Line::from(Span::styled(
        format!("Unknown: {}", node.node.node_type),
        Style::default().fg(Color::DarkGray),
    ))]
}
