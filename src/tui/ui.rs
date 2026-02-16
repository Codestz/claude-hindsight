//! TUI rendering
//!
//! Draws the terminal UI layout and components.

use crate::tui::app::{App, ViewMode};
use crate::tui::render::render_node_content;
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
        .highlight_symbol(">> ")
        // Enable tree visualization with nice symbols
        .node_open_symbol("▼ ")
        .node_closed_symbol("▶ ")
        .node_no_children_symbol("  ");

    f.render_stateful_widget(tree_widget, area, &mut app.tree_state);
}

/// Draw the details panel (split into content 70% + metadata 30%)
fn draw_details(f: &mut Frame, area: Rect, app: &mut App) {
    use crate::tui::app::FocusMode;

    let title = match app.view_mode {
        ViewMode::Summary => {
            if app.focus_mode == FocusMode::Details {
                "Details [s] *FOCUSED*"
            } else {
                "Details [s]"
            }
        }
        ViewMode::Input => {
            if app.focus_mode == FocusMode::Details {
                "Tool Input [i] *FOCUSED*"
            } else {
                "Tool Input [i]"
            }
        }
        ViewMode::Output => {
            if app.focus_mode == FocusMode::Details {
                "Tool Output [o] *FOCUSED*"
            } else {
                "Tool Output [o]"
            }
        }
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

    // Apply scroll offset
    let content_widget = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).title(title))
        .wrap(Wrap { trim: true })
        .scroll((app.details_scroll as u16, 0));

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

/// Render node content (main content area) - now using clean rendering module
fn render_summary_content(node: &crate::analyzer::TreeNode) -> Text<'static> {
    // Use the clean rendering function from render.rs
    let lines = render_node_content(node);
    Text::from(lines)
}

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
            Span::raw(": Navigate/Scroll | "),
            Span::styled("Tab", Style::default().fg(Color::Yellow)),
            Span::raw(": Switch Focus | "),
            Span::styled("Enter", Style::default().fg(Color::Yellow)),
            Span::raw(": Expand | "),
            Span::styled("i/o/s", Style::default().fg(Color::Yellow)),
            Span::raw(": Views | "),
            Span::styled("q", Style::default().fg(Color::Yellow)),
            Span::raw(": Quit"),
        ]),
        Line::from(app.status_message.clone()),
    ];

    let footer = Paragraph::new(shortcuts).block(Block::default().borders(Borders::ALL));

    f.render_widget(footer, area);
}
