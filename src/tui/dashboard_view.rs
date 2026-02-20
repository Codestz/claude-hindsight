//! Dashboard view — global stats + sparkline + top tools

use crate::error::Result;
use crate::storage::{GlobalAnalytics, SessionIndex};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Sparkline},
    Frame,
};

/// Key actions returned from DashboardView
pub enum DashboardAction {
    None,
    Back,
    Quit,
}

/// Dashboard view showing global statistics
pub struct DashboardView {
    pub analytics: GlobalAnalytics,
    pub sparkline_data: Vec<u64>,
    pub total_tokens: u64,
    pub total_cost: f64,
}

impl DashboardView {
    /// Load from the index
    pub fn new() -> Result<Self> {
        let index = SessionIndex::new()?;
        let analytics = index.get_global_analytics()?;
        let sparkline_data = index.get_daily_session_counts(14)?;
        let (total_tokens, total_cost) = index.get_global_totals()?;

        Ok(DashboardView {
            analytics,
            sparkline_data,
            total_tokens,
            total_cost,
        })
    }

    /// Handle a keypress; returns the action
    pub fn handle_key(&mut self, key: crossterm::event::KeyEvent) -> Result<DashboardAction> {
        use crossterm::event::KeyCode;
        match key.code {
            KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => Ok(DashboardAction::Quit),
            KeyCode::Backspace => Ok(DashboardAction::Back),
            _ => Ok(DashboardAction::None),
        }
    }

    /// Render the dashboard
    pub fn render(&self, f: &mut Frame, area: Rect) {
        let a = &self.analytics;

        // Outer layout: title + content
        let outer = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // header
                Constraint::Min(0),    // body
            ])
            .split(area);

        // Header
        let header = Paragraph::new(Line::from(vec![
            Span::styled(
                "  Dashboard",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled(
                format!(
                    "{} sessions  ·  {} projects  ·  {} today",
                    a.total_sessions, a.total_projects, a.sessions_today
                ),
                Style::default().fg(Color::DarkGray),
            ),
            Span::raw("  "),
            Span::styled("Esc: back", Style::default().fg(Color::DarkGray)),
        ]))
        .block(Block::default().borders(Borders::BOTTOM));
        f.render_widget(header, outer[0]);

        // Body: two columns
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(outer[1]);

        self.render_left(f, cols[0]);
        self.render_right(f, cols[1]);
    }

    fn render_left(&self, f: &mut Frame, area: Rect) {
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(8), // stats
                Constraint::Length(8), // sparkline
                Constraint::Min(0),
            ])
            .split(area);

        // ── Stats card ──────────────────────────────────────────────────────────
        let a = &self.analytics;
        let total_cost = self.total_cost;
        let total_tok = self.total_tokens;

        let stats_lines = vec![
            Line::from(vec![
                Span::styled("  Sessions  ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{}", a.total_sessions),
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("  (this week: {})", a.sessions_this_week),
                    Style::default().fg(Color::DarkGray),
                ),
            ]),
            Line::from(vec![
                Span::styled("  Projects  ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{}", a.total_projects),
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("  Tokens    ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    fmt_tokens(total_tok),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("  Cost      ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("${:.2}", total_cost),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("  Subagents ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{}", a.subagent_count),
                    Style::default().fg(Color::Magenta),
                ),
            ]),
            if let Some(ref proj) = a.most_active_project {
                Line::from(vec![
                    Span::styled("  Latest    ", Style::default().fg(Color::DarkGray)),
                    Span::styled(proj.clone(), Style::default().fg(Color::Cyan)),
                ])
            } else {
                Line::from("")
            },
        ];

        let stats_widget = Paragraph::new(stats_lines).block(
            Block::default()
                .title(" Global Stats ")
                .borders(Borders::ALL),
        );
        f.render_widget(stats_widget, rows[0]);

        // ── Sparkline (sessions per day, last 14 days) ───────────────────────
        let sparkline_widget = Sparkline::default()
            .block(
                Block::default()
                    .title(" Sessions / day (14d) ")
                    .borders(Borders::ALL),
            )
            .data(&self.sparkline_data)
            .style(Style::default().fg(Color::Cyan));
        f.render_widget(sparkline_widget, rows[1]);
    }

    fn render_right(&self, f: &mut Frame, area: Rect) {
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(9), // top tools
                Constraint::Min(0),
            ])
            .split(area);

        // ── Top tools ────────────────────────────────────────────────────────
        let tools = &self.analytics.top_tools;
        let max_count = tools.first().map(|(_, c)| *c).unwrap_or(1);

        let mut tool_lines: Vec<Line> = vec![
            Line::from(Span::styled("", Style::default())), // padding
        ];

        for (name, count) in tools.iter().take(5) {
            let bar_len = ((*count as f64 / max_count as f64) * 18.0) as usize;
            let bar = "█".repeat(bar_len);
            let empty = "░".repeat(18usize.saturating_sub(bar_len));

            tool_lines.push(Line::from(vec![
                Span::styled(
                    format!("  {:<14}", truncate(name, 14)),
                    Style::default().fg(Color::White),
                ),
                Span::styled(bar, Style::default().fg(Color::Blue)),
                Span::styled(empty, Style::default().fg(Color::DarkGray)),
                Span::styled(format!("  {}", count), Style::default().fg(Color::DarkGray)),
            ]));
        }

        if tools.is_empty() {
            tool_lines.push(Line::from(Span::styled(
                "  No tool data yet — run `hindsight reindex`",
                Style::default().fg(Color::DarkGray),
            )));
        }

        let tools_widget = Paragraph::new(tool_lines)
            .block(Block::default().title(" Top Tools ").borders(Borders::ALL));
        f.render_widget(tools_widget, rows[0]);
    }
}

fn fmt_tokens(n: u64) -> String {
    if n < 1_000 {
        format!("{}", n)
    } else if n < 1_000_000 {
        format!("{:.1}k", n as f64 / 1_000.0)
    } else {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    }
}

fn truncate(s: &str, max: usize) -> &str {
    if s.len() <= max {
        s
    } else {
        &s[..max]
    }
}
