//! Sessions browser view
//!
//! Shows all sessions for a selected project with metadata.

use crate::config::Config;
use crate::error::Result;
use crate::storage::{ProjectAnalytics, SessionFile, SessionIndex};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

/// Sort order for the sessions list
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortMode {
    Newest,
    MostExpensive,
    MostTokens,
    MostErrors,
}

impl SortMode {
    fn label(self) -> &'static str {
        match self {
            SortMode::Newest => "Newest",
            SortMode::MostExpensive => "Cost ↓",
            SortMode::MostTokens => "Tokens ↓",
            SortMode::MostErrors => "Errors ↓",
        }
    }

    fn next(self) -> Self {
        match self {
            SortMode::Newest => SortMode::MostExpensive,
            SortMode::MostExpensive => SortMode::MostTokens,
            SortMode::MostTokens => SortMode::MostErrors,
            SortMode::MostErrors => SortMode::Newest,
        }
    }
}

/// Sessions view state
pub struct SessionsView {
    /// Project name
    pub project_name: String,

    /// Sessions for this project (sorted per sort_mode)
    pub sessions: Vec<SessionFile>,

    /// List selection state
    pub list_state: ListState,

    /// Filter query
    pub filter_query: String,

    /// Whether we're in filter input mode
    pub filter_mode: bool,

    /// Status message
    pub status_message: String,

    /// Project-specific analytics
    pub analytics: ProjectAnalytics,

    /// Application configuration
    pub config: Config,

    /// Current sort mode
    pub sort_mode: SortMode,
}

impl SessionsView {
    /// Create a new sessions view for a project
    pub fn new(project_name: String, config: &Config) -> Result<Self> {
        let index = SessionIndex::new()?;
        let sessions = index.find_by_project(&project_name)?;
        let analytics = index.get_project_analytics(&project_name)?;

        let mut list_state = ListState::default();
        if !sessions.is_empty() {
            list_state.select(Some(0));
        }

        let status_message = format!("{} sessions in {}", sessions.len(), project_name);

        Ok(SessionsView {
            project_name,
            sessions,
            list_state,
            filter_query: String::new(),
            filter_mode: false,
            status_message,
            analytics,
            config: config.clone(),
            sort_mode: SortMode::Newest,
        })
    }

    /// Refresh sessions
    pub fn refresh(&mut self) -> Result<()> {
        let index = SessionIndex::new()?;
        self.sessions = index.find_by_project(&self.project_name)?;
        self.analytics = index.get_project_analytics(&self.project_name)?;

        // Reset selection if empty
        if self.sessions.is_empty() {
            self.list_state.select(None);
        } else if self.list_state.selected().is_none() {
            self.list_state.select(Some(0));
        }

        self.status_message = format!("{} sessions in {}", self.sessions.len(), self.project_name);
        Ok(())
    }

    /// Handle keyboard input
    pub fn handle_key(&mut self, key: KeyEvent) -> Result<SessionAction> {
        // Handle filter input mode
        if self.filter_mode {
            return self.handle_filter_input(key);
        }

        match (key.code, key.modifiers) {
            // Navigation
            (KeyCode::Char('j'), KeyModifiers::NONE) | (KeyCode::Down, _) => {
                self.next();
                Ok(SessionAction::None)
            }
            (KeyCode::Char('k'), KeyModifiers::NONE) | (KeyCode::Up, _) => {
                self.previous();
                Ok(SessionAction::None)
            }

            // Jump to top/bottom
            (KeyCode::Char('g'), KeyModifiers::NONE) | (KeyCode::Home, _) => {
                self.select_first();
                Ok(SessionAction::None)
            }
            (KeyCode::Char('G'), KeyModifiers::SHIFT) | (KeyCode::End, _) => {
                self.select_last();
                Ok(SessionAction::None)
            }

            // Select session (view details)
            (KeyCode::Enter, KeyModifiers::NONE) => {
                if let Some(session) = self.selected_session() {
                    Ok(SessionAction::SelectSession(session.session_id.clone()))
                } else {
                    Ok(SessionAction::None)
                }
            }

            // Go back to projects
            (KeyCode::Char('h'), KeyModifiers::NONE) | (KeyCode::Esc, _) => Ok(SessionAction::Back),

            // Start filter
            (KeyCode::Char('/'), KeyModifiers::NONE) => {
                self.filter_mode = true;
                self.status_message = "Filter by ID or message: ".to_string();
                Ok(SessionAction::None)
            }

            // Refresh
            (KeyCode::Char('r'), KeyModifiers::NONE) => {
                self.refresh()?;
                self.status_message = "Refreshed session list".to_string();
                Ok(SessionAction::None)
            }

            // Sort toggle (S cycles through sort modes)
            (KeyCode::Char('s'), KeyModifiers::NONE)
            | (KeyCode::Char('S'), KeyModifiers::SHIFT) => {
                self.sort_mode = self.sort_mode.next();
                self.apply_sort();
                self.status_message = format!("Sort: {}", self.sort_mode.label());
                Ok(SessionAction::None)
            }

            // Quit
            (KeyCode::Char('q'), KeyModifiers::NONE) => Ok(SessionAction::Quit),

            _ => Ok(SessionAction::None),
        }
    }

    /// Re-sort sessions according to the current sort mode
    fn apply_sort(&mut self) {
        match self.sort_mode {
            SortMode::Newest => {
                self.sessions
                    .sort_by(|a, b| b.modified_at.cmp(&a.modified_at));
            }
            SortMode::MostExpensive => {
                self.sessions.sort_by(|a, b| {
                    b.estimated_cost
                        .partial_cmp(&a.estimated_cost)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
            }
            SortMode::MostTokens => {
                self.sessions
                    .sort_by(|a, b| b.total_tokens.cmp(&a.total_tokens));
            }
            SortMode::MostErrors => {
                self.sessions
                    .sort_by(|a, b| b.error_count.cmp(&a.error_count));
            }
        }
        // Keep selection at top after sort
        if !self.sessions.is_empty() {
            self.list_state.select(Some(0));
        }
    }

    /// Handle filter input
    fn handle_filter_input(&mut self, key: KeyEvent) -> Result<SessionAction> {
        match key.code {
            KeyCode::Enter => {
                self.filter_mode = false;
                self.apply_filter()?;
                Ok(SessionAction::None)
            }
            KeyCode::Esc => {
                self.filter_mode = false;
                self.filter_query.clear();
                self.refresh()?;
                self.status_message = "Filter cancelled".to_string();
                Ok(SessionAction::None)
            }
            KeyCode::Backspace => {
                self.filter_query.pop();
                self.status_message = format!("Filter: {}", self.filter_query);
                Ok(SessionAction::None)
            }
            KeyCode::Char(c) => {
                self.filter_query.push(c);
                self.status_message = format!("Filter: {}", self.filter_query);
                Ok(SessionAction::None)
            }
            _ => Ok(SessionAction::None),
        }
    }

    /// Apply filter
    fn apply_filter(&mut self) -> Result<()> {
        let index = SessionIndex::new()?;
        let all_sessions = index.find_by_project(&self.project_name)?;

        if self.filter_query.is_empty() {
            self.sessions = all_sessions;
        } else {
            let query = self.filter_query.to_lowercase();
            self.sessions = all_sessions
                .into_iter()
                .filter(|s| {
                    s.session_id.to_lowercase().contains(&query)
                        || s.first_message
                            .as_deref()
                            .map(|m| m.to_lowercase().contains(&query))
                            .unwrap_or(false)
                })
                .collect();
        }

        // Reset selection
        if !self.sessions.is_empty() {
            self.list_state.select(Some(0));
        } else {
            self.list_state.select(None);
        }

        self.status_message = format!("{} sessions match filter", self.sessions.len());
        Ok(())
    }

    /// Get selected session
    pub fn selected_session(&self) -> Option<&SessionFile> {
        self.list_state
            .selected()
            .and_then(|i| self.sessions.get(i))
    }

    /// Select next session
    fn next(&mut self) {
        if self.sessions.is_empty() {
            return;
        }

        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.sessions.len() - 1 {
                    i
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    /// Select previous session
    fn previous(&mut self) {
        if self.sessions.is_empty() {
            return;
        }

        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    0
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    /// Select first session
    fn select_first(&mut self) {
        if !self.sessions.is_empty() {
            self.list_state.select(Some(0));
        }
    }

    /// Select last session
    fn select_last(&mut self) {
        if !self.sessions.is_empty() {
            self.list_state.select(Some(self.sessions.len() - 1));
        }
    }

    /// Render the sessions view
    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                Constraint::Length(12), // Header (ASCII art + description + project info)
                Constraint::Min(0),     // Content area (sessions + analytics)
                Constraint::Length(3),  // Status bar
            ])
            .split(area);

        // Render header
        self.render_header(f, chunks[0]);

        // Split content area horizontally: sessions list (left) and analytics panel (right)
        let content_chunks = Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints([
                Constraint::Percentage(65), // Sessions list
                Constraint::Percentage(35), // Analytics panel
            ])
            .split(chunks[1]);

        // Render session list
        self.render_list(f, content_chunks[0]);

        // Render analytics panel
        self.render_analytics_panel(f, content_chunks[1]);

        // Render status bar
        self.render_status(f, chunks[2]);
    }

    /// Render the header with ASCII art
    fn render_header(&self, f: &mut Frame, area: Rect) {
        // Calculate horizontal padding for centering
        let content_width = 85;
        let padding = (area.width.saturating_sub(content_width)) / 2;
        let pad = " ".repeat(padding as usize);

        let header_text = vec![
            Line::from(""),
            Line::from(vec![
                Span::raw(&pad),
                Span::styled(
                    "██╗  ██╗██╗███╗   ██╗██████╗ ███████╗██╗ ██████╗ ██╗  ██╗████████╗",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::raw(&pad),
                Span::styled(
                    "██║  ██║██║████╗  ██║██╔══██╗██╔════╝██║██╔════╝ ██║  ██║╚══██╔══╝",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::raw(&pad),
                Span::styled(
                    "███████║██║██╔██╗ ██║██║  ██║███████╗██║██║  ███╗███████║   ██║   ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::raw(&pad),
                Span::styled(
                    "██╔══██║██║██║╚██╗██║██║  ██║╚════██║██║██║   ██║██╔══██║   ██║   ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::raw(&pad),
                Span::styled(
                    "██║  ██║██║██║ ╚████║██████╔╝███████║██║╚██████╔╝██║  ██║   ██║   ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::raw(&pad),
                Span::styled(
                    "╚═╝  ╚═╝╚═╝╚═╝  ╚═══╝╚═════╝ ╚══════╝╚═╝ ╚═════╝ ╚═╝  ╚═╝   ╚═╝   ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::raw(" ".repeat((area.width as usize).saturating_sub(76) / 2)),
                Span::styled(
                    "A powerful observability tool for Claude Code. Debug sessions,",
                    Style::default().fg(Color::Gray),
                ),
            ]),
            Line::from(vec![
                Span::raw(" ".repeat((area.width as usize).saturating_sub(76) / 2)),
                Span::styled(
                    "analyze costs, and understand Claude's decision-making process.",
                    Style::default().fg(Color::Gray),
                ),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("Project: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    &self.project_name,
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("  •  "),
                Span::styled(
                    format!("{} sessions", self.sessions.len()),
                    Style::default().fg(Color::Cyan),
                ),
            ]),
        ];

        let header = Paragraph::new(header_text)
            .block(Block::default())
            .alignment(ratatui::layout::Alignment::Left);

        f.render_widget(header, area);
    }

    /// Render the session list with column header
    fn render_list(&mut self, f: &mut Frame, area: Rect) {
        // Render the outer block and get inner area for children
        let title = format!(
            "Sessions — Sort: {} (S:cycle)  ●=clean ●=errors",
            self.sort_mode.label()
        );
        let block = Block::default().borders(Borders::ALL).title(title);
        let inner = block.inner(area);
        f.render_widget(block, area);

        // Split inner: 1-line header row + list body
        let chunks = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(0)])
            .split(inner);

        // ── #11: Column header row ────────────────────────────────────────
        let header = Paragraph::new(Line::from(vec![
            Span::styled("  ● ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{:8}", "ID"),
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::UNDERLINED),
            ),
            Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{:7}", "Age"),
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::UNDERLINED),
            ),
            Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{:8}", "Updated"),
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::UNDERLINED),
            ),
            Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{:12}", "Model"),
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::UNDERLINED),
            ),
            Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{:7}", "Tokens"),
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::UNDERLINED),
            ),
            Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{:7}", "Cost"),
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::UNDERLINED),
            ),
            Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                "Message",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::UNDERLINED),
            ),
        ]));
        f.render_widget(header, chunks[0]);

        // ── Data rows ─────────────────────────────────────────────────────
        let items: Vec<ListItem> = self
            .sessions
            .iter()
            .map(|session| {
                let age_str = format_time_ago(Some(session.created_at));
                let updated_str = format_time_ago(Some(session.modified_at));
                let short_id = &session.session_id[..8.min(session.session_id.len())];
                let dot_color = if session.error_count == 0 {
                    Color::Green
                } else {
                    Color::Red
                };
                let model_raw = session.model.as_deref().unwrap_or("-");
                let model_display = if model_raw.len() > 12 {
                    &model_raw[..12]
                } else {
                    model_raw
                };
                let tok_str = fmt_tokens(session.total_tokens);
                let cost_str = format!("${:.3}", session.estimated_cost);
                let preview_raw = session.first_message.as_deref().unwrap_or("(no message)");
                let preview_end = preview_raw
                    .char_indices()
                    .nth(35)
                    .map(|(i, _)| i)
                    .unwrap_or(preview_raw.len());
                let preview = &preview_raw[..preview_end];

                ListItem::new(Line::from(vec![
                    Span::styled("  ", Style::default()),
                    Span::styled("●", Style::default().fg(dot_color)),
                    Span::raw(" "),
                    Span::styled(
                        format!("{:8}", short_id),
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" │ "),
                    Span::styled(
                        format!("{:7}", age_str),
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::raw(" │ "),
                    Span::styled(
                        format!("{:8}", updated_str),
                        Style::default().fg(Color::Yellow),
                    ),
                    Span::raw(" │ "),
                    Span::styled(
                        format!("{:12}", model_display),
                        Style::default().fg(Color::Blue),
                    ),
                    Span::raw(" │ "),
                    Span::styled(format!("{:7}", tok_str), Style::default().fg(Color::Green)),
                    Span::raw(" │ "),
                    Span::styled(
                        format!("{:7}", cost_str),
                        Style::default().fg(Color::Yellow),
                    ),
                    Span::raw(" │ "),
                    Span::styled(preview.to_string(), Style::default().fg(Color::Gray)),
                ]))
            })
            .collect();

        let list = List::new(items)
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        f.render_stateful_widget(list, chunks[1], &mut self.list_state);
    }

    /// Render analytics panel
    fn render_analytics_panel(&self, f: &mut Frame, area: Rect) {
        let size_mb = self.analytics.total_size as f64 / 1_000_000.0;
        let total_tokens = fmt_tokens(self.analytics.total_tokens);
        let avg_cost = if self.analytics.total_sessions > 0 {
            self.analytics.total_cost / self.analytics.total_sessions as f64
        } else {
            0.0
        };

        let mut lines = vec![
            Line::from(""),
            // Overview section
            Line::from(vec![Span::styled(
                " Overview",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![
                Span::raw("  Sessions:       "),
                Span::styled(
                    format!("{}", self.analytics.total_sessions),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::raw("  Total Size:     "),
                Span::styled(
                    format!("{:.1} MB", size_mb),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            // Cost & tokens
            Line::from(vec![Span::styled(
                " Cost & Usage",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![
                Span::raw("  Total Tokens:   "),
                Span::styled(
                    total_tokens,
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::raw("  Total Cost:     "),
                Span::styled(
                    format!("${:.2}", self.analytics.total_cost),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::raw("  Avg Cost/Sess:  "),
                Span::styled(
                    format!("${:.3}", avg_cost),
                    Style::default().fg(Color::Yellow),
                ),
            ]),
            Line::from(vec![
                Span::raw("  Total Errors:   "),
                Span::styled(
                    format!("{}", self.analytics.total_errors),
                    Style::default()
                        .fg(if self.analytics.total_errors > 0 {
                            Color::Red
                        } else {
                            Color::Green
                        })
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
        ];

        // Activity section (conditional based on config)
        if self.config.analytics.show_activity {
            lines.push(Line::from(vec![Span::styled(
                " Activity",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )]));
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::raw("  This Week:      "),
                Span::styled(
                    format!("{}", self.analytics.sessions_this_week),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" sessions"),
            ]));
            lines.push(Line::from(vec![
                Span::raw("  Today:          "),
                Span::styled(
                    format!("{}", self.analytics.sessions_today),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" sessions"),
            ]));
            lines.push(Line::from(""));
        }

        // Session Types section (conditional based on config)
        if self.config.analytics.show_subagent_count {
            lines.push(Line::from(vec![Span::styled(
                " Session Types",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )]));
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::raw("  With Subagents: "),
                Span::styled(
                    format!("{}", self.analytics.subagent_count),
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD),
                ),
            ]));
            lines.push(Line::from(""));
        }

        // Top Tools section (conditional based on config)
        if self.config.analytics.show_top_tools {
            lines.push(Line::from(vec![Span::styled(
                " Top Tools",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )]));
            lines.push(Line::from(""));

            if !self.analytics.top_tools.is_empty() {
                // Limit to configured number of tools
                let tools_to_show = self
                    .analytics
                    .top_tools
                    .iter()
                    .take(self.config.analytics.tools_limit);

                for (tool, count) in tools_to_show {
                    let tool_name = if tool.len() > 12 {
                        format!("{}...", &tool[..9])
                    } else {
                        tool.clone()
                    };

                    lines.push(Line::from(vec![
                        Span::raw("  "),
                        Span::styled(
                            format!("{:12}", tool_name),
                            Style::default().fg(Color::Blue),
                        ),
                        Span::styled(
                            format!("{:>4}", count),
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                        ),
                    ]));
                }
            } else {
                lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled("Analyzing...", Style::default().fg(Color::DarkGray)),
                ]));
            }
        }

        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Project Analytics"),
            )
            .alignment(ratatui::layout::Alignment::Left);

        f.render_widget(paragraph, area);
    }

    /// Render status bar
    fn render_status(&self, f: &mut Frame, area: Rect) {
        let shortcuts = vec![
            Line::from(vec![
                Span::styled("j/k", Style::default().fg(Color::Yellow)),
                Span::raw(": Nav | "),
                Span::styled("Enter", Style::default().fg(Color::Yellow)),
                Span::raw(": View | "),
                Span::styled("S", Style::default().fg(Color::Yellow)),
                Span::raw(": Sort | "),
                Span::styled("h/Esc", Style::default().fg(Color::Yellow)),
                Span::raw(": Back | "),
                Span::styled("/", Style::default().fg(Color::Yellow)),
                Span::raw(": Filter (ID or message) | "),
                Span::styled("r", Style::default().fg(Color::Yellow)),
                Span::raw(": Refresh | "),
                Span::styled("q", Style::default().fg(Color::Yellow)),
                Span::raw(": Quit"),
            ]),
            Line::from(self.status_message.as_str()),
        ];

        let status = Paragraph::new(shortcuts).block(Block::default().borders(Borders::ALL));

        f.render_widget(status, area);
    }
}

/// Actions that can be triggered from the sessions view
#[derive(Debug)]
pub enum SessionAction {
    None,
    SelectSession(String),
    Back,
    Quit,
}

/// Format token count as compact string (e.g. 8200 → "8.2k", 1500000 → "1.5M")
fn fmt_tokens(n: u64) -> String {
    if n < 1_000 {
        format!("{}", n)
    } else if n < 1_000_000 {
        format!("{:.1}k", n as f64 / 1_000.0)
    } else {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    }
}

/// Format timestamp as relative time
fn format_time_ago(timestamp: Option<i64>) -> String {
    let timestamp = match timestamp {
        Some(t) => t,
        None => return "never".to_string(),
    };

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let diff = now - timestamp;

    if diff < 60 {
        format!("{}s ago", diff)
    } else if diff < 3600 {
        format!("{}m ago", diff / 60)
    } else if diff < 86400 {
        format!("{}h ago", diff / 3600)
    } else if diff < 604800 {
        format!("{}d ago", diff / 86400)
    } else {
        // Show actual date
        use chrono::{DateTime, Local};
        let dt = DateTime::from_timestamp(timestamp, 0)
            .unwrap_or_else(|| DateTime::from_timestamp(0, 0).unwrap());
        let local: DateTime<Local> = dt.into();
        local.format("%Y-%m-%d").to_string()
    }
}
