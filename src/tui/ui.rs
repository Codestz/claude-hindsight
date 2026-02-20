//! TUI rendering
//!
//! Draws the terminal UI layout and components.

use crate::tui::app::App;
use crate::tui::render::render_node_content;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};
use tui_tree_widget::Tree;

/// Draw the main UI
pub fn draw(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7),  // Header (session info + model/tokens/cost + file + breadcrumb)
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

    // ── #14: Error summary overlay ────────────────────────────────────────
    if app.show_error_summary {
        draw_error_summary_overlay(f, app);
    }
}

/// Format token count for header display (e.g. 8200 → "8.2k", 1500000 → "1.5M")
fn fmt_tok(n: u64) -> String {
    if n < 1_000 {
        format!("{}", n)
    } else if n < 1_000_000 {
        format!("{:.1}k", n as f64 / 1_000.0)
    } else {
        format!("{:.1}M", n as f64 / 1_000_000.0)
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

    let model_name = app.session.model
        .as_deref()
        .unwrap_or("unknown")
        .to_string();

    let mut session_info = vec![
        // Line 1: session stats
        Line::from(vec![
            Span::styled("Session: ", Style::default().fg(Color::Cyan)),
            Span::raw(app.session.session_id.clone()),
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
            Span::styled("Tools: ", Style::default().fg(Color::Cyan)),
            Span::styled(
                {
                    let ok = analytics.tool_call_count.saturating_sub(analytics.tool_result_error_count);
                    format!("{}/{}", ok, analytics.tool_call_count)
                },
                Style::default().fg(if analytics.tool_result_error_count > 0 { Color::Yellow } else { Color::Green })
            ),
            Span::raw(" | "),
            Span::styled("Errors: ", Style::default().fg(Color::Cyan)),
            Span::styled(
                format!("{}", analytics.error_count),
                Style::default().fg(if analytics.error_count > 0 { Color::Red } else { Color::Green })
            ),
        ]),
        // Line 2: model + token breakdown + cost
        Line::from(vec![
            Span::styled("Model: ", Style::default().fg(Color::Cyan)),
            Span::styled(model_name, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::raw("  |  "),
            Span::styled("In: ", Style::default().fg(Color::Cyan)),
            Span::styled(fmt_tok(analytics.input_tokens), Style::default().fg(Color::Green)),
            Span::styled(
                format!(" Wr: {}", fmt_tok(analytics.cache_creation_tokens)),
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                format!(" Rd: {}", fmt_tok(analytics.cache_read_tokens)),
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                format!("  Out: {}", fmt_tok(analytics.output_tokens)),
                Style::default().fg(Color::Green),
            ),
            Span::raw("  |  "),
            Span::styled("Cost: ", Style::default().fg(Color::Cyan)),
            Span::styled(
                format!("${:.4}", app.session.estimated_cost),
                Style::default().fg(Color::Yellow),
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

/// Draw the details panel (content + optional metadata + optional timeline)
fn draw_details(f: &mut Frame, area: Rect, app: &mut App) {
    use crate::tui::app::FocusMode;

    let scroll_indicator = app.details_scroll_info.position_text();
    let scroll_text = if !scroll_indicator.is_empty() {
        format!(" ({})", scroll_indicator)
    } else {
        String::new()
    };

    // Build title reflecting active view mode
    let mode_tag = if app.show_raw_json {
        " [JSON]"
    } else if app.show_diff {
        " [DIFF]"
    } else {
        ""
    };

    let title = if app.focus_mode == FocusMode::Details {
        format!("Details *FOCUSED*{}{}", mode_tag, scroll_text)
    } else {
        format!("Details{}{}", mode_tag, scroll_text)
    };

    // Check if we have useful metadata / timeline
    let has_useful_metadata = app.selected_node().map(has_valuable_metadata).unwrap_or(false);
    let has_timeline = app.selected_node()
        .map(|n| collect_tool_timings(n, &app.tool_correlation))
        .map(|t| !t.is_empty())
        .unwrap_or(false);

    // Build vertical constraints dynamically
    let constraints: Vec<Constraint> = {
        let mut c = vec![Constraint::Min(0)]; // content
        if has_useful_metadata { c.push(Constraint::Length(7)); }
        if has_timeline        { c.push(Constraint::Length(6)); }
        c
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);

    let content_area = chunks[0];
    let metadata_area = if has_useful_metadata { Some(chunks[1]) } else { None };
    let timeline_area = if has_useful_metadata && has_timeline {
        chunks.get(2).copied()
    } else if !has_useful_metadata && has_timeline {
        chunks.get(1).copied()
    } else {
        None
    };

    // Build render context
    let ctx = crate::tui::render::RenderContext {
        tool_correlation: &app.tool_correlation,
        tool_result_map: &app.tool_result_map,
    };

    // Choose content based on active view mode
    let content = if let Some(node) = app.selected_node() {
        if app.show_raw_json {
            render_raw_json(node)
        } else if app.show_diff {
            render_diff_content(node)
        } else {
            render_summary_content(node, &ctx)
        }
    } else {
        Text::from("No node selected")
    };

    let total_lines = content.lines.len();
    let viewport_height = content_area.height.saturating_sub(2) as usize;
    app.update_scroll_info(total_lines, viewport_height);

    let content_widget = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).title(title))
        .wrap(Wrap { trim: false })
        .scroll((app.details_scroll as u16, 0));
    f.render_widget(content_widget, content_area);

    // Metadata panel (#13: includes cost share)
    if let Some(rect) = metadata_area {
        if let Some(node) = app.selected_node() {
            let metadata = render_metadata(node);
            let widget = Paragraph::new(metadata)
                .block(Block::default().borders(Borders::ALL).title("Metadata"))
                .wrap(Wrap { trim: true });
            f.render_widget(widget, rect);
        }
    }

    // Timeline panel (#12)
    if let Some(rect) = timeline_area {
        if let Some(node) = app.selected_node() {
            let timings = collect_tool_timings(node, &app.tool_correlation);
            draw_timeline(f, rect, &timings);
        }
    }
}

/// Render node content (main content area)
fn render_summary_content(
    node: &crate::analyzer::TreeNode,
    ctx: &crate::tui::render::RenderContext,
) -> Text<'static> {
    let lines = render_node_content(node, ctx);
    Text::from(lines)
}

/// Check if node has valuable metadata worth displaying
fn has_valuable_metadata(node: &crate::analyzer::TreeNode) -> bool {
    // Has any token usage (including cache tokens)?
    if let Some(usage) = node.node.effective_token_usage() {
        if usage.total() > 0 {
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

    false
}

fn render_metadata(node: &crate::analyzer::TreeNode) -> Text<'static> {
    let mut lines = vec![];

    // Token usage
    if let Some(usage) = node.node.effective_token_usage() {
        if usage.total() > 0 {
            let uncached     = usage.input_tokens.unwrap_or(0);
            let cache_write  = usage.cache_creation_input_tokens.unwrap_or(0);
            let cache_read   = usage.cache_read_input_tokens.unwrap_or(0);
            let output       = usage.output_tokens.unwrap_or(0);
            let total_in     = usage.total_input(); // uncached + cache_write + cache_read

            // "Input" shows the same total_input() used by the tree badge
            lines.push(Line::from(vec![
                Span::styled("Input:       ", Style::default().fg(Color::DarkGray)),
                Span::styled(fmt_tok(total_in as u64), Style::default().fg(Color::Green)),
            ]));

            // If there are cache tokens, show the breakdown so the numbers add up
            if cache_write > 0 || cache_read > 0 {
                lines.push(Line::from(vec![
                    Span::styled("  uncached:  ", Style::default().fg(Color::DarkGray)),
                    Span::styled(fmt_tok(uncached as u64), Style::default().fg(Color::DarkGray)),
                ]));
                if cache_write > 0 {
                    lines.push(Line::from(vec![
                        Span::styled("  cache wr:  ", Style::default().fg(Color::DarkGray)),
                        Span::styled(fmt_tok(cache_write as u64), Style::default().fg(Color::DarkGray)),
                    ]));
                }
                if cache_read > 0 {
                    lines.push(Line::from(vec![
                        Span::styled("  cache rd:  ", Style::default().fg(Color::DarkGray)),
                        Span::styled(fmt_tok(cache_read as u64), Style::default().fg(Color::DarkGray)),
                    ]));
                }
            }

            lines.push(Line::from(vec![
                Span::styled("Output:      ", Style::default().fg(Color::DarkGray)),
                Span::styled(fmt_tok(output as u64), Style::default().fg(Color::Green)),
            ]));
        }
    }

    // Duration (for tool results)
    if let Some(ref result) = node.node.tool_result {
        if let Some(duration_ms) = result.duration_ms {
            lines.push(Line::from(vec![
                Span::styled("Duration:    ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{:.2}s", duration_ms as f64 / 1000.0),
                    Style::default().fg(Color::Gray),
                ),
            ]));
        }
    }

    Text::from(lines)
}

// ── #9: Raw JSON view ────────────────────────────────────────────────────────

fn render_raw_json(node: &crate::analyzer::TreeNode) -> Text<'static> {
    let json = serde_json::to_string_pretty(&*node.node).unwrap_or_else(|_| "{}".to_string());
    Text::from(
        json.lines()
            .map(|l| Line::from(l.to_string()))
            .collect::<Vec<_>>(),
    )
}

// ── #18: Diff view ────────────────────────────────────────────────────────────

fn render_diff_content(node: &crate::analyzer::TreeNode) -> Text<'static> {
    use crate::parser::models::ContentBlock;
    use similar::{ChangeTag, TextDiff};

    // Extract old_string / new_string from an Edit tool call
    let pair = node.node.message.as_ref().and_then(|msg| {
        msg.content_blocks().iter().find_map(|b| {
            if let ContentBlock::ToolUse { name, input, .. } = b {
                if name == "Edit" || name == "MultiEdit" {
                    let old = input.get("old_string").and_then(|v| v.as_str()).map(String::from)?;
                    let new = input.get("new_string").and_then(|v| v.as_str()).map(String::from)?;
                    return Some((old, new));
                }
            }
            None
        })
    });

    let (old_str, new_str) = match pair {
        Some(p) => p,
        None => {
            return Text::from(Span::styled(
                "Diff view: select an Edit tool call node",
                Style::default().fg(Color::DarkGray),
            ));
        }
    };

    let diff = TextDiff::from_lines(old_str.as_str(), new_str.as_str());
    let mut lines = vec![
        Line::from(Span::styled(
            "  Diff (Edit tool)",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];

    for change in diff.iter_all_changes() {
        let (color, prefix) = match change.tag() {
            ChangeTag::Delete => (Color::Red,     "- "),
            ChangeTag::Insert => (Color::Green,   "+ "),
            ChangeTag::Equal  => (Color::DarkGray,"  "),
        };
        lines.push(Line::from(Span::styled(
            format!("{}{}", prefix, change.value().trim_end_matches('\n')),
            Style::default().fg(color),
        )));
    }

    Text::from(lines)
}

// ── #12: Tool timeline ───────────────────────────────────────────────────────

/// Collect (tool_name, duration_ms) pairs from all descendant tool_result nodes
fn collect_tool_timings(
    node: &crate::analyzer::TreeNode,
    correlation: &std::collections::HashMap<String, String>,
) -> Vec<(String, u64)> {
    let mut timings = vec![];
    collect_timings_recursive(node, correlation, &mut timings);
    timings
}

fn collect_timings_recursive(
    node: &crate::analyzer::TreeNode,
    correlation: &std::collections::HashMap<String, String>,
    out: &mut Vec<(String, u64)>,
) {
    for child in &node.children {
        if let Some(ref tr) = child.node.tool_result {
            if let Some(ms) = tr.duration_ms {
                // Try to resolve tool name via toolUseId in extra, then correlation map
                let name = child.node.extra.as_ref()
                    .and_then(|e| e.get("toolUseId"))
                    .and_then(|v| v.as_str())
                    .and_then(|id| correlation.get(id))
                    .cloned()
                    .or_else(|| child.node.tool_use.as_ref().map(|tu| tu.name.clone()))
                    .unwrap_or_else(|| "Tool".to_string());
                out.push((name, ms.max(0) as u64));
            }
        }
        collect_timings_recursive(child, correlation, out);
    }
}

fn draw_timeline(f: &mut Frame, area: Rect, timings: &[(String, u64)]) {
    if timings.is_empty() {
        return;
    }
    let max_ms = timings.iter().map(|(_, ms)| *ms).max().unwrap_or(1).max(1);
    let bar_width = area.width.saturating_sub(26) as usize;

    let lines: Vec<Line> = timings.iter().map(|(name, ms)| {
        let bar_len = ((*ms as f64 / max_ms as f64) * bar_width as f64) as usize;
        let filled: String = "█".repeat(bar_len);
        let empty:  String = "░".repeat(bar_width.saturating_sub(bar_len));
        Line::from(vec![
            Span::styled(format!("{:12} ", &name[..name.len().min(12)]), Style::default().fg(Color::Cyan)),
            Span::styled(filled, Style::default().fg(Color::Green)),
            Span::styled(empty,  Style::default().fg(Color::DarkGray)),
            Span::styled(format!("  {:.1}s", *ms as f64 / 1000.0), Style::default().fg(Color::Yellow)),
        ])
    }).collect();

    let widget = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title("Tool Timeline"));
    f.render_widget(widget, area);
}

// ── #14: Error summary overlay ────────────────────────────────────────────────

fn draw_error_summary_overlay(f: &mut Frame, app: &App) {
    let area = f.area();
    let w = 72u16;
    let h = (app.error_nodes_info.len() as u16 + 4).clamp(5, 20);
    let x = area.width.saturating_sub(w) / 2;
    let y = area.height.saturating_sub(h) / 2;
    let rect = Rect { x, y, width: w, height: h };

    f.render_widget(Clear, rect);

    if app.error_nodes_info.is_empty() {
        let msg = Paragraph::new("No errors in this session  (Esc to close)")
            .style(Style::default().fg(Color::Green))
            .block(Block::default().borders(Borders::ALL).title(" Errors "));
        f.render_widget(msg, rect);
        return;
    }

    let items: Vec<ListItem> = app.error_nodes_info.iter().enumerate().map(|(i, (_, ntype, desc))| {
        let selected = i == app.error_summary_selection;
        let bg = if selected { Color::DarkGray } else { Color::Reset };
        ListItem::new(Line::from(vec![
            Span::styled(format!(" {:2}. ", i + 1), Style::default().fg(Color::Yellow).bg(bg)),
            Span::styled(format!("{:12} ", ntype), Style::default().fg(Color::Cyan).bg(bg)),
            Span::styled(desc.clone(), Style::default().fg(Color::Red).bg(bg)),
        ]))
    }).collect();

    let title = format!(" {} Error(s) — j/k: nav  Enter: jump  Esc: close ", app.error_nodes_info.len());
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(Style::default().bg(Color::DarkGray));
    f.render_widget(list, rect);
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
            Span::styled("Tab", Style::default().fg(Color::Yellow)),
            Span::raw(": Focus | "),
            Span::styled("e/E", Style::default().fg(Color::Yellow)),
            Span::raw(": Err± | "),
            Span::styled("x", Style::default().fg(Color::Yellow)),
            Span::raw(": ErrList | "),
            Span::styled("y", Style::default().fg(Color::Yellow)),
            Span::raw(": Copy | "),
            Span::styled("J", Style::default().fg(Color::Yellow)),
            Span::raw(": JSON | "),
            Span::styled("d", Style::default().fg(Color::Yellow)),
            Span::raw(": Diff | "),
            Span::styled("p", Style::default().fg(Color::Yellow)),
            Span::raw(": Replay | "),
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
            if app.replay_mode {
                Span::styled("▶ REPLAY  ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
            } else {
                Span::raw("")
            },
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

    let widget = Paragraph::new(query.to_string())
        .style(Style::default().fg(Color::Yellow).bg(Color::DarkGray))
        .block(Block::default().borders(Borders::ALL).title("Filter by node type (user,assistant,tool_use) - Enter: apply | Esc: cancel"));

    // Clear the area first, then render the widget
    f.render_widget(Clear, search_area);
    f.render_widget(widget, search_area);
}
