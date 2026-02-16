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
            Span::styled("Session: ", Style::default().fg(Color::Cyan)),
            Span::raw(&app.session.session_id),
        ]),
        Line::from(vec![
            Span::styled("Nodes: ", Style::default().fg(Color::Cyan)),
            Span::raw(format!("{}", app.tree.stats.total_nodes)),
            Span::raw(" | "),
            Span::styled("Tools: ", Style::default().fg(Color::Cyan)),
            Span::raw(format!("{}", app.tree.stats.tool_calls)),
            Span::raw(" | "),
            Span::styled("Errors: ", Style::default().fg(Color::Red)),
            Span::raw(format!("{}", app.tree.stats.errors)),
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

/// Draw the details panel
fn draw_details(f: &mut Frame, area: Rect, app: &mut App) {
    let title = match app.view_mode {
        ViewMode::Summary => "Details [s]",
        ViewMode::Input => "Tool Input [i]",
        ViewMode::Output => "Tool Output [o]",
    };

    let content = if let Some(node) = app.selected_node() {
        match app.view_mode {
            ViewMode::Summary => render_summary(node),
            ViewMode::Input => render_input(node),
            ViewMode::Output => render_output(node),
        }
    } else {
        Text::from("No node selected")
    };

    let details = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).title(title))
        .wrap(Wrap { trim: true });

    f.render_widget(details, area);
}

/// Render node summary
fn render_summary(node: &crate::analyzer::TreeNode) -> Text<'static> {
    let mut lines = vec![
        Line::from(vec![
            Span::styled("Type: ".to_string(), Style::default().fg(Color::Cyan)),
            Span::raw(node.node.node_type.clone()),
        ]),
        Line::from(vec![
            Span::styled("Depth: ".to_string(), Style::default().fg(Color::Cyan)),
            Span::raw(format!("{}", node.depth)),
        ]),
    ];

    // Timestamp
    if let Some(timestamp) = node.timestamp() {
        let dt = chrono::DateTime::from_timestamp_millis(timestamp);
        if let Some(dt) = dt {
            lines.push(Line::from(vec![
                Span::styled("Time: ".to_string(), Style::default().fg(Color::Cyan)),
                Span::raw(dt.format("%Y-%m-%d %H:%M:%S").to_string()),
            ]));
        }
    }

    // UUID
    if let Some(ref uuid) = node.node.uuid {
        lines.push(Line::from(vec![
            Span::styled("UUID: ".to_string(), Style::default().fg(Color::Cyan)),
            Span::raw(uuid.clone()),
        ]));
    }

    // Tool use
    if let Some(ref tool_use) = node.node.tool_use {
        lines.push(Line::from("".to_string()));
        lines.push(Line::from(vec![
            Span::styled("Tool: ".to_string(), Style::default().fg(Color::Yellow)),
            Span::raw(tool_use.name.clone()),
        ]));
    }

    // Tool result
    if let Some(ref result) = node.node.tool_result {
        lines.push(Line::from("".to_string()));
        if let Some(is_error) = result.is_error {
            let status = if is_error { "❌ Error".to_string() } else { "✅ Success".to_string() };
            let color = if is_error { Color::Red } else { Color::Green };
            lines.push(Line::from(Span::styled(status, Style::default().fg(color))));
        }

        if let Some(duration_ms) = result.duration_ms {
            lines.push(Line::from(vec![
                Span::styled("Duration: ".to_string(), Style::default().fg(Color::Cyan)),
                Span::raw(format!("{}.{}s", duration_ms / 1000, duration_ms % 1000 / 100)),
            ]));
        }
    }

    // Token usage
    if let Some(ref usage) = node.node.token_usage {
        lines.push(Line::from("".to_string()));
        if let Some(input) = usage.input_tokens {
            lines.push(Line::from(vec![
                Span::styled("Input tokens: ".to_string(), Style::default().fg(Color::Cyan)),
                Span::raw(format!("{}", input)),
            ]));
        }
        if let Some(output) = usage.output_tokens {
            lines.push(Line::from(vec![
                Span::styled("Output tokens: ".to_string(), Style::default().fg(Color::Cyan)),
                Span::raw(format!("{}", output)),
            ]));
        }
    }

    // Thinking
    if let Some(ref thinking) = node.node.thinking {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Thinking:".to_string(),
            Style::default().fg(Color::Blue),
        )));
        let preview = if thinking.len() > 200 {
            format!("{}...", &thinking[..200])
        } else {
            thinking.clone()
        };
        lines.push(Line::from(Span::raw(preview)));
    }

    Text::from(lines)
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
