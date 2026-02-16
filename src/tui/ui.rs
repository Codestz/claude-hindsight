//! TUI rendering
//!
//! Draws the terminal UI layout and components.

use crate::tui::app::{App, ViewMode};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};
use tui_tree_widget::Tree;

/// Draw the main UI
pub fn draw(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(0),     // Main content
            Constraint::Length(3),  // Footer/status
        ])
        .split(f.area());

    draw_header(f, chunks[0], app);
    draw_main(f, chunks[1], app);
    draw_footer(f, chunks[2], app);
}

/// Draw the header
fn draw_header(f: &mut Frame, area: Rect, app: &mut App) {
    let session_info = vec![
        Line::from(vec![
            Span::styled("Session: ".to_string(), Style::default().fg(Color::Cyan)),
            Span::raw(&app.session.session_id),
        ]),
        Line::from(vec![
            Span::styled("Nodes: ".to_string(), Style::default().fg(Color::Cyan)),
            Span::raw(format!("{}", app.total_nodes)),
            Span::raw(" | ".to_string()),
            Span::styled("Groups: ".to_string(), Style::default().fg(Color::Cyan)),
            Span::raw(format!("{}", app.tree_roots.len())),
        ]),
    ];

    let header = Paragraph::new(session_info)
        .block(Block::default().borders(Borders::ALL).title("Claude Hindsight"));

    f.render_widget(header, area);
}

/// Draw the main content area (split between tree and details)
fn draw_main(f: &mut Frame, area: Rect, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50), // Tree view
            Constraint::Percentage(50), // Details panel
        ])
        .split(area);

    draw_tree(f, chunks[0], app);
    draw_details(f, chunks[1], app);
}

/// Draw the tree view
fn draw_tree(f: &mut Frame, area: Rect, app: &mut App) {
    let items = &app.tree_items;

    let tree_widget = Tree::new(items)
        .expect("Failed to create tree widget")
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Execution Tree"),
        )
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(tree_widget, area, &mut app.tree_state);
}

/// Draw the details panel (split into content 70% + metadata 30%)
fn draw_details(f: &mut Frame, area: Rect, app: &mut App) {
    let title = match app.view_mode {
        ViewMode::Summary => "Details [s]",
        ViewMode::Input => "Tool Input [i]",
        ViewMode::Output => "Tool Output [o]",
    };

    // Split into content (70%) and metadata (30%)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(70),  // Content
            Constraint::Percentage(30),  // Metadata
        ])
        .split(area);

    // Render content
    let content = if let Some(node) = app.selected_node() {
        match app.view_mode {
            ViewMode::Summary => render_summary_content(node),
            ViewMode::Input => render_input(node),
            ViewMode::Output => render_output(node),
        }
    } else {
        Text::from("No node selected")
    };

    let content_widget = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).title(title))
        .wrap(Wrap { trim: true });

    f.render_widget(content_widget, chunks[0]);

    // Render metadata
    if let Some(node) = app.selected_node() {
        let metadata = render_metadata(node);
        let metadata_widget = Paragraph::new(metadata)
            .block(Block::default().borders(Borders::ALL).title("Metadata"))
            .wrap(Wrap { trim: true });

        f.render_widget(metadata_widget, chunks[1]);
    }
}

/// Render node content (main content area)
fn render_summary_content(node: &crate::analyzer::TreeNode) -> Text<'static> {
    let mut lines = vec![];

    // USER MESSAGE - show full content
    if node.node.node_type == "user" {
        if let Some(ref msg) = node.node.message {
            if let Some(ref content) = msg.content {
                let text = extract_full_text(content);
                if !text.is_empty() {
                    lines.push(Line::from(Span::styled(" User".to_string(), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))));
                    lines.push(Line::from("".to_string()));
                    for line in text.lines() {
                        lines.push(Line::from(Span::raw(line.to_string())));
                    }
                }
            }
        }
    }
    // ASSISTANT MESSAGE
    else if node.node.node_type == "assistant" {
        if let Some(ref msg) = node.node.message {
            if let Some(ref content) = msg.content {
                let text = extract_full_text(content);
                if !text.is_empty() {
                    lines.push(Line::from(Span::styled(" Assistant".to_string(), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))));
                    lines.push(Line::from("".to_string()));
                    for line in text.lines() {
                        lines.push(Line::from(Span::raw(line.to_string())));
                    }
                }
            }
        }
    }
    // THINKING
    else if let Some(ref thinking) = node.node.thinking {
        lines.push(Line::from(Span::styled(" Thinking".to_string(), Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD))));
        lines.push(Line::from("".to_string()));
        for line in thinking.lines() {
            lines.push(Line::from(Span::raw(line.to_string())));
        }
    }
    // TOOL USE
    else if let Some(ref tool_use) = node.node.tool_use {
        lines.push(Line::from(Span::styled(format!(" Tool: {}", tool_use.name), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))));
        lines.push(Line::from("".to_string()));
        if let Some(obj) = tool_use.input.as_object() {
            if let Some(file_path) = obj.get("file_path").and_then(|v| v.as_str()) {
                lines.push(Line::from(vec![Span::styled(" File: ".to_string(), Style::default().fg(Color::Cyan)), Span::raw(file_path.to_string())]));
            }
            if let Some(command) = obj.get("command").and_then(|v| v.as_str()) {
                lines.push(Line::from(vec![Span::styled("$ ".to_string(), Style::default().fg(Color::Yellow)), Span::raw(command.to_string())]));
            }
            if let Some(pattern) = obj.get("pattern").and_then(|v| v.as_str()) {
                lines.push(Line::from(vec![Span::styled(" Search: ".to_string(), Style::default().fg(Color::Cyan)), Span::raw(pattern.to_string())]));
            }
        }
    }
    // TOOL RESULT
    else if let Some(ref result) = node.node.tool_result {
        if let Some(is_error) = result.is_error {
            if is_error {
                lines.push(Line::from(Span::styled(" Error".to_string(), Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))));
                lines.push(Line::from("".to_string()));
                if let Some(ref error) = result.error {
                    for line in error.lines().take(20) {
                        lines.push(Line::from(Span::raw(line.to_string())));
                    }
                }
            } else {
                lines.push(Line::from(Span::styled(" Success".to_string(), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))));
                if let Some(ref content) = result.content {
                    lines.push(Line::from("".to_string()));
                    for line in content.lines().take(15) {
                        lines.push(Line::from(Span::raw(line.to_string())));
                    }
                    if content.lines().count() > 15 {
                        lines.push(Line::from(Span::styled("... (output truncated)".to_string(), Style::default().fg(Color::DarkGray))));
                    }
                }
            }
        }
    }
    // FILE SNAPSHOT
    else if node.node.node_type == "file-history-snapshot" {
        lines.push(Line::from(Span::styled(" File Snapshot".to_string(), Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))));
        lines.push(Line::from("".to_string()));
        lines.push(Line::from("Backup point of tracked files in the session."));
    }
    // SYSTEM MESSAGE
    else if node.node.node_type == "system" {
        lines.push(Line::from(Span::styled(" System".to_string(), Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD))));
        if let Some(ref msg) = node.node.message {
            if let Some(ref content) = msg.content {
                let text = extract_full_text(content);
                if !text.is_empty() {
                    lines.push(Line::from("".to_string()));
                    for line in text.lines() {
                        lines.push(Line::from(Span::raw(line.to_string())));
                    }
                }
            }
        }
    }
    // PROGRESS UPDATE
    else if node.node.node_type == "progress" {
        lines.push(Line::from(Span::styled(" Progress".to_string(), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))));
        if let Some(ref progress) = node.node.progress {
            if let Some(ref message) = progress.message {
                lines.push(Line::from("".to_string()));
                lines.push(Line::from(Span::raw(message.clone())));
            }
            if let Some(percentage) = progress.percentage {
                lines.push(Line::from(Span::styled(format!("{}%", percentage), Style::default().fg(Color::Cyan))));
            }
        }
    }
    // GROUP NODE
    else if node.node.node_type == "group" {
        if let Some(ref msg) = node.node.message {
            if let Some(ref content) = msg.content {
                if let Some(label) = content.as_str() {
                    lines.push(Line::from(Span::styled(label.to_string(), Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))));
                    lines.push(Line::from("".to_string()));
                    lines.push(Line::from("Grouped items below."));
                }
            }
        }
    }
    // OTHER NODES
    else {
        lines.push(Line::from(vec![Span::styled(" ".to_string(), Style::default().fg(Color::Cyan)), Span::raw(node.node.node_type.clone())]));
    }

    if lines.is_empty() {
        lines.push(Line::from("No content available"));
    }

    Text::from(lines)
}

/// Render metadata panel (30% bottom section)
fn render_metadata(node: &crate::analyzer::TreeNode) -> Text<'static> {
    let mut lines = vec![];

    // Type
    lines.push(Line::from(vec![
        Span::styled("Type: ".to_string(), Style::default().fg(Color::DarkGray)),
        Span::styled(node.node.node_type.clone(), Style::default().fg(Color::Gray)),
    ]));

    // Depth
    lines.push(Line::from(vec![
        Span::styled("Depth: ".to_string(), Style::default().fg(Color::DarkGray)),
        Span::styled(format!("{}", node.depth), Style::default().fg(Color::Gray)),
    ]));

    // Timestamp
    if let Some(timestamp) = node.timestamp() {
        let dt = chrono::DateTime::from_timestamp_millis(timestamp);
        if let Some(dt) = dt {
            lines.push(Line::from(vec![
                Span::styled("Time: ".to_string(), Style::default().fg(Color::DarkGray)),
                Span::styled(dt.format("%Y-%m-%d %H:%M:%S").to_string(), Style::default().fg(Color::Gray)),
            ]));
        }
    }

    // Token usage
    if let Some(ref usage) = node.node.token_usage {
        let input = usage.input_tokens.unwrap_or(0);
        let output = usage.output_tokens.unwrap_or(0);
        if input + output > 0 {
            lines.push(Line::from(vec![
                Span::styled("Tokens: ".to_string(), Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{} in / {} out", input, output), Style::default().fg(Color::Gray)),
            ]));
        }
    }

    // Duration (for tool results)
    if let Some(ref result) = node.node.tool_result {
        if let Some(duration_ms) = result.duration_ms {
            lines.push(Line::from(vec![
                Span::styled("Duration: ".to_string(), Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{}.{}s", duration_ms / 1000, duration_ms % 1000 / 100), Style::default().fg(Color::Gray)),
            ]));
        }
    }

    // UUID (truncated)
    if let Some(ref uuid) = node.node.uuid {
        let short_uuid = if uuid.len() > 16 {
            format!("{}...", &uuid[..16])
        } else {
            uuid.clone()
        };
        lines.push(Line::from(vec![
            Span::styled("ID: ".to_string(), Style::default().fg(Color::DarkGray)),
            Span::styled(short_uuid, Style::default().fg(Color::DarkGray)),
        ]));
    }

    Text::from(lines)
}

/// Extract full text from message content
fn extract_full_text(content: &serde_json::Value) -> String {
    match content {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Array(arr) => {
            arr.iter().filter_map(|block| {
                block.get("text").and_then(|t| t.as_str().map(|s| s.to_string()))
            }).collect::<Vec<_>>().join("

")
        }
        _ => String::new(),
    }
}

/// Render tool input
fn render_input(node: &crate::analyzer::TreeNode) -> Text<'static> {
    if let Some(ref tool_use) = node.node.tool_use {
        let input_json = serde_json::to_string_pretty(&tool_use.input)
            .unwrap_or_else(|_| "Failed to serialize".to_string());
        Text::from(input_json)
    } else {
        Text::from("No tool input available")
    }
}

/// Render tool output
fn render_output(node: &crate::analyzer::TreeNode) -> Text<'static> {
    if let Some(ref result) = node.node.tool_result {
        let mut lines = vec![];

        if let Some(ref content) = result.content {
            let preview = if content.len() > 1000 {
                format!("{}...\n\n(showing first 1000 chars)", &content[..1000])
            } else {
                content.clone()
            };
            lines.push(Line::from(preview));
        }

        if let Some(ref error) = result.error {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Error:",
                Style::default().fg(Color::Red),
            )));
            lines.push(Line::from(error.clone()));
        }

        if lines.is_empty() {
            Text::from("No output available")
        } else {
            Text::from(lines)
        }
    } else {
        Text::from("No tool output available")
    }
}

/// Draw the footer with keyboard shortcuts
fn draw_footer(f: &mut Frame, area: Rect, app: &mut App) {
    let shortcuts = vec![
        Line::from(vec![
            Span::styled("j/k", Style::default().fg(Color::Yellow)),
            Span::raw(": Navigate | "),
            Span::styled("Enter", Style::default().fg(Color::Yellow)),
            Span::raw(": Expand/Collapse | "),
            Span::styled("i/o/s", Style::default().fg(Color::Yellow)),
            Span::raw(": Input/Output/Summary | "),
            Span::styled("q", Style::default().fg(Color::Yellow)),
            Span::raw(": Quit"),
        ]),
        Line::from(app.status_message.clone()),
    ];

    let footer = Paragraph::new(shortcuts).block(Block::default().borders(Borders::ALL));

    f.render_widget(footer, area);
}
