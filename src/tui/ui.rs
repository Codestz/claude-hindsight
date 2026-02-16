//! TUI rendering
//!
//! Draws the terminal UI layout and components.

use crate::tui::app::App;
use crate::tui::render::render_node_content;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
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

    // Draw search overlay if in input mode
    if app.input_mode {
        draw_search_overlay(f, app);
    }
}

/// Draw the header
fn draw_header(f: &mut Frame, area: Rect, app: &mut App) {
    let breadcrumb = app.get_breadcrumb_path().join(" → ");
    let breadcrumb_display = if !breadcrumb.is_empty() {
        breadcrumb
    } else {
        "(no selection)".to_string()
    };

    let analytics = &app.analytics;

    let mut session_info = vec![
        Line::from(vec![
            Span::styled("Session: ", Style::default().fg(Color::Cyan)),
            Span::raw(&app.session.session_id),
            Span::raw(" | "),
            Span::styled("Nodes: ", Style::default().fg(Color::Cyan)),
            Span::raw(format!("{}", app.total_nodes)),
            Span::raw(" | "),
            Span::styled("Duration: ", Style::default().fg(Color::Cyan)),
            Span::styled(analytics.duration_string(), Style::default().fg(Color::Yellow)),
            Span::raw(" | "),
            Span::styled("Thinking: ", Style::default().fg(Color::Cyan)),
            Span::styled(format!("{}", analytics.thinking_count), Style::default().fg(Color::Magenta)),
            Span::raw(" | "),
            Span::styled("Errors: ", Style::default().fg(Color::Cyan)),
            Span::styled(
                format!("{}", analytics.error_count),
                Style::default().fg(if analytics.error_count > 0 { Color::Red } else { Color::Green })
            ),
        ]),
    ];

    // Add file path if available
    if let Some(ref file_path) = app.session.file_path {
        session_info.push(Line::from(vec![
            Span::styled("File: ", Style::default().fg(Color::Cyan)),
            Span::styled(file_path.clone(), Style::default().fg(Color::Gray)),
        ]));
    }

    session_info.push(Line::from(vec![
        Span::styled("Node: ", Style::default().fg(Color::Cyan)),
        Span::styled(breadcrumb_display, Style::default().fg(Color::Yellow)),
    ]));

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
                .title("Session Nodes"),
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

/// Draw the details panel (split into content 100% or content 70% + metadata 30%)
fn draw_details(f: &mut Frame, area: Rect, app: &mut App) {
    use crate::tui::app::FocusMode;

    let scroll_indicator = app.details_scroll_info.position_text();
    let scroll_text = if !scroll_indicator.is_empty() {
        format!(" ({})", scroll_indicator)
    } else {
        String::new()
    };

    let title = if app.focus_mode == FocusMode::Details {
        format!("Details *FOCUSED*{}", scroll_text)
    } else {
        format!("Details{}", scroll_text)
    };

    // Check if we have useful metadata to show
    let has_useful_metadata = if let Some(node) = app.selected_node() {
        has_valuable_metadata(node)
    } else {
        false
    };

    // Split layout based on whether we have useful metadata
    let (content_area, metadata_area) = if has_useful_metadata {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(70),  // Content
                Constraint::Percentage(30),  // Metadata
            ])
            .split(area);
        (chunks[0], Some(chunks[1]))
    } else {
        // Use full area for content if no useful metadata
        (area, None)
    };

    // Render content
    let content = if let Some(node) = app.selected_node() {
        render_summary_content(node)
    } else {
        Text::from("No node selected")
    };

    // Update scroll info based on content
    let total_lines = content.lines.len();
    let viewport_height = content_area.height.saturating_sub(2) as usize; // -2 for borders
    app.update_scroll_info(total_lines, viewport_height);

    // Apply scroll offset
    let content_widget = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).title(title))
        .wrap(Wrap { trim: false })  // Preserve indentation for code
        .scroll((app.details_scroll as u16, 0));

    f.render_widget(content_widget, content_area);

    // Render metadata only if we have useful info
    if let Some(metadata_rect) = metadata_area {
        if let Some(node) = app.selected_node() {
            let metadata = render_metadata(node);
            let metadata_widget = Paragraph::new(metadata)
                .block(Block::default().borders(Borders::ALL).title("Metadata"))
                .wrap(Wrap { trim: true });

            f.render_widget(metadata_widget, metadata_rect);
        }
    }
}

/// Render node content (main content area)
fn render_summary_content(node: &crate::analyzer::TreeNode) -> Text<'static> {
    let lines = render_node_content(node);
    Text::from(lines)
}

/// Check if node has valuable metadata worth displaying
fn has_valuable_metadata(node: &crate::analyzer::TreeNode) -> bool {
    // Show metadata if any of these are present:
    // - Token usage
    // - Duration (tool results)
    // - Errors

    // Has token usage?
    if let Some(ref usage) = node.node.token_usage {
        let input = usage.input_tokens.unwrap_or(0);
        let output = usage.output_tokens.unwrap_or(0);
        if input + output > 0 {
            return true;
        }
    }

    // Has duration (tool result)?
    if let Some(ref result) = node.node.tool_result {
        if result.duration_ms.is_some() {
            return true;
        }
        // Has error in tool result?
        if let Some(is_error) = result.is_error {
            if is_error {
                return true;
            }
        }
    }

    // No valuable metadata
    false
}

fn render_metadata(node: &crate::analyzer::TreeNode) -> Text<'static> {
    let mut lines = vec![];

    // Only show valuable metadata (no Type, Depth, or ID clutter)

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

    Text::from(lines)
}

/// Draw the footer with keyboard shortcuts
fn draw_footer(f: &mut Frame, area: Rect, app: &mut App) {
    use crate::tui::app::FocusMode;

    // Get current position for status display
    let current_position = if app.selected_node().is_some() {
        // Calculate rough position (could be improved with proper indexing)
        1
    } else {
        0
    };

    let shortcuts = vec![
        // Line 1: Keyboard shortcuts
        Line::from(vec![
            Span::styled("j/k", Style::default().fg(Color::Yellow)),
            Span::raw(": Nav | "),
            Span::styled("Ctrl+d/u", Style::default().fg(Color::Yellow)),
            Span::raw(": HalfPage | "),
            Span::styled("Tab", Style::default().fg(Color::Yellow)),
            Span::raw(": Focus | "),
            Span::styled("/", Style::default().fg(Color::Yellow)),
            Span::raw(": Filter | "),
            Span::styled("n/N", Style::default().fg(Color::Yellow)),
            Span::raw(": Next/Prev | "),
            Span::styled("g/G", Style::default().fg(Color::Yellow)),
            Span::raw(": Top/Bottom | "),
            Span::styled("q", Style::default().fg(Color::Yellow)),
            Span::raw(": Quit"),
        ]),
        // Line 2: Status info
        Line::from(vec![
            Span::styled(
                format!("[{}] ", if app.focus_mode == FocusMode::Tree { "LIST" } else { "DETAILS" }),
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            ),
            Span::raw(format!("Node {}/{} | ", current_position, app.total_nodes)),
            Span::styled(&app.status_message, Style::default().fg(Color::Gray)),
        ]),
    ];

    let footer = Paragraph::new(shortcuts).block(Block::default().borders(Borders::ALL));

    f.render_widget(footer, area);
}

/// Draw search overlay when in input mode
fn draw_search_overlay(f: &mut Frame, app: &App) {
    let area = f.area();

    // Position search overlay at bottom (above footer)
    let search_area = Rect {
        x: area.x + 2,
        y: area.height.saturating_sub(6),
        width: area.width.saturating_sub(4).min(60),
        height: 3,
    };

    let query = app.search_state.as_ref()
        .map(|s| s.query.as_str())
        .unwrap_or("");

    let widget = Paragraph::new(format!("{}", query))
        .style(Style::default().fg(Color::Yellow).bg(Color::DarkGray))
        .block(Block::default().borders(Borders::ALL).title("Filter by node type (user,assistant,tool_use) - Enter: apply | Esc: cancel"));

    // Clear the area first, then render the widget
    f.render_widget(Clear, search_area);
    f.render_widget(widget, search_area);
}
