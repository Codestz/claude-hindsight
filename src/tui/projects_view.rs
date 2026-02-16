//! Projects browser view
//!
//! Shows all discovered projects with statistics and allows drilling down into sessions.

use crate::error::Result;
use crate::storage::{ProjectStats, SessionIndex, GlobalAnalytics};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

/// Projects view state
pub struct ProjectsView {
    /// All project statistics
    pub projects: Vec<ProjectStats>,

    /// List selection state
    pub list_state: ListState,

    /// Filter query
    pub filter_query: String,

    /// Whether we're in filter input mode
    pub filter_mode: bool,

    /// Status message
    pub status_message: String,

    /// Global analytics across all sessions
    pub analytics: GlobalAnalytics,
}

impl ProjectsView {
    /// Create a new projects view
    pub fn new() -> Result<Self> {
        let index = SessionIndex::new()?;
        let projects = index.get_all_project_stats()?;
        let analytics = index.get_global_analytics()?;

        let mut list_state = ListState::default();
        if !projects.is_empty() {
            list_state.select(Some(0));
        }

        let status_message = format!("{} projects found", projects.len());

        Ok(ProjectsView {
            projects,
            list_state,
            filter_query: String::new(),
            filter_mode: false,
            status_message,
            analytics,
        })
    }

    /// Refresh project statistics
    pub fn refresh(&mut self) -> Result<()> {
        let index = SessionIndex::new()?;
        self.projects = index.get_all_project_stats()?;
        self.analytics = index.get_global_analytics()?;

        // Reset selection if empty
        if self.projects.is_empty() {
            self.list_state.select(None);
        } else if self.list_state.selected().is_none() {
            self.list_state.select(Some(0));
        }

        self.status_message = format!("{} projects found", self.projects.len());
        Ok(())
    }

    /// Handle keyboard input
    pub fn handle_key(&mut self, key: KeyEvent) -> Result<ProjectAction> {
        // Handle filter input mode
        if self.filter_mode {
            return self.handle_filter_input(key);
        }

        match (key.code, key.modifiers) {
            // Navigation
            (KeyCode::Char('j'), KeyModifiers::NONE) | (KeyCode::Down, _) => {
                self.next();
                Ok(ProjectAction::None)
            }
            (KeyCode::Char('k'), KeyModifiers::NONE) | (KeyCode::Up, _) => {
                self.previous();
                Ok(ProjectAction::None)
            }

            // Jump to top/bottom
            (KeyCode::Char('g'), KeyModifiers::NONE) | (KeyCode::Home, _) => {
                self.select_first();
                Ok(ProjectAction::None)
            }
            (KeyCode::Char('G'), KeyModifiers::SHIFT) | (KeyCode::End, _) => {
                self.select_last();
                Ok(ProjectAction::None)
            }

            // Select project (drill down to sessions)
            (KeyCode::Enter, KeyModifiers::NONE) => {
                if let Some(project) = self.selected_project() {
                    Ok(ProjectAction::SelectProject(project.project_name.clone()))
                } else {
                    Ok(ProjectAction::None)
                }
            }

            // Start filter
            (KeyCode::Char('/'), KeyModifiers::NONE) => {
                self.filter_mode = true;
                self.status_message = "Filter: ".to_string();
                Ok(ProjectAction::None)
            }

            // Refresh
            (KeyCode::Char('r'), KeyModifiers::NONE) => {
                self.refresh()?;
                self.status_message = "Refreshed project list".to_string();
                Ok(ProjectAction::None)
            }

            // Quit
            (KeyCode::Char('q'), KeyModifiers::NONE) => {
                Ok(ProjectAction::Quit)
            }

            _ => Ok(ProjectAction::None),
        }
    }

    /// Handle filter input
    fn handle_filter_input(&mut self, key: KeyEvent) -> Result<ProjectAction> {
        match key.code {
            KeyCode::Enter => {
                self.filter_mode = false;
                self.apply_filter()?;
                Ok(ProjectAction::None)
            }
            KeyCode::Esc => {
                self.filter_mode = false;
                self.filter_query.clear();
                self.refresh()?;
                self.status_message = "Filter cancelled".to_string();
                Ok(ProjectAction::None)
            }
            KeyCode::Backspace => {
                self.filter_query.pop();
                self.status_message = format!("Filter: {}", self.filter_query);
                Ok(ProjectAction::None)
            }
            KeyCode::Char(c) => {
                self.filter_query.push(c);
                self.status_message = format!("Filter: {}", self.filter_query);
                Ok(ProjectAction::None)
            }
            _ => Ok(ProjectAction::None),
        }
    }

    /// Apply filter
    fn apply_filter(&mut self) -> Result<()> {
        let index = SessionIndex::new()?;
        let all_projects = index.get_all_project_stats()?;

        if self.filter_query.is_empty() {
            self.projects = all_projects;
        } else {
            let query = self.filter_query.to_lowercase();
            self.projects = all_projects
                .into_iter()
                .filter(|p| p.project_name.to_lowercase().contains(&query))
                .collect();
        }

        // Reset selection
        if !self.projects.is_empty() {
            self.list_state.select(Some(0));
        } else {
            self.list_state.select(None);
        }

        self.status_message = format!("{} projects match filter", self.projects.len());
        Ok(())
    }

    /// Get selected project
    pub fn selected_project(&self) -> Option<&ProjectStats> {
        self.list_state.selected().and_then(|i| self.projects.get(i))
    }

    /// Select next project
    fn next(&mut self) {
        if self.projects.is_empty() {
            return;
        }

        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.projects.len() - 1 {
                    i
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    /// Select previous project
    fn previous(&mut self) {
        if self.projects.is_empty() {
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

    /// Select first project
    fn select_first(&mut self) {
        if !self.projects.is_empty() {
            self.list_state.select(Some(0));
        }
    }

    /// Select last project
    fn select_last(&mut self) {
        if !self.projects.is_empty() {
            self.list_state.select(Some(self.projects.len() - 1));
        }
    }

    /// Render the projects view
    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                Constraint::Length(12),  // Welcome header (taller for new ASCII art)
                Constraint::Min(0),      // Content area (projects + analytics)
                Constraint::Length(3),   // Status bar
            ])
            .split(area);

        // Render welcome header
        self.render_header(f, chunks[0]);

        // Split content area horizontally: projects list (left) and analytics panel (right)
        let content_chunks = Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints([
                Constraint::Percentage(65),  // Projects list
                Constraint::Percentage(35),  // Analytics panel
            ])
            .split(chunks[1]);

        // Render project list
        self.render_list(f, content_chunks[0]);

        // Render analytics panel
        self.render_analytics_panel(f, content_chunks[1]);

        // Render status bar
        self.render_status(f, chunks[2]);
    }

    /// Render the welcome header with ASCII art
    fn render_header(&self, f: &mut Frame, area: Rect) {
        // Calculate horizontal padding for centering (assuming ~80 char width for content)
        let content_width = 85;
        let padding = (area.width.saturating_sub(content_width)) / 2;
        let pad = " ".repeat(padding as usize);

        let header_text = vec![
            Line::from(""),
            Line::from(vec![
                Span::raw(&pad),
                Span::styled(
                    "██╗  ██╗██╗███╗   ██╗██████╗ ███████╗██╗ ██████╗ ██╗  ██╗████████╗",
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::raw(&pad),
                Span::styled(
                    "██║  ██║██║████╗  ██║██╔══██╗██╔════╝██║██╔════╝ ██║  ██║╚══██╔══╝",
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::raw(&pad),
                Span::styled(
                    "███████║██║██╔██╗ ██║██║  ██║███████╗██║██║  ███╗███████║   ██║   ",
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::raw(&pad),
                Span::styled(
                    "██╔══██║██║██║╚██╗██║██║  ██║╚════██║██║██║   ██║██╔══██║   ██║   ",
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::raw(&pad),
                Span::styled(
                    "██║  ██║██║██║ ╚████║██████╔╝███████║██║╚██████╔╝██║  ██║   ██║   ",
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::raw(&pad),
                Span::styled(
                    "╚═╝  ╚═╝╚═╝╚═╝  ╚═══╝╚═════╝ ╚══════╝╚═╝ ╚═════╝ ╚═╝  ╚═╝   ╚═╝   ",
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
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
            Line::from(""),
        ];

        let header = Paragraph::new(header_text)
            .block(Block::default())
            .alignment(ratatui::layout::Alignment::Left);

        f.render_widget(header, area);
    }

    /// Render the project list
    fn render_list(&mut self, f: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self.projects
            .iter()
            .map(|project| {
                let size_mb = project.total_size as f64 / 1_000_000.0;
                let time_ago = format_time_ago(project.last_activity);

                let line = Line::from(vec![
                    Span::raw("  "),
                    Span::styled(
                        format!("{:22}", project.project_name),
                        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!("{:4} sessions", project.session_count),
                        Style::default().fg(Color::Yellow),
                    ),
                    Span::raw("    "),
                    Span::styled(
                        format!("{:8.1} MB", size_mb),
                        Style::default().fg(Color::Green),
                    ),
                    Span::raw("    "),
                    Span::styled(
                        format!("{:>12}", time_ago),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]);

                ListItem::new(line)
            })
            .collect();

        let title = if self.projects.is_empty() {
            "Projects (none found - run 'hindsight init' first)"
        } else {
            "Projects"
        };

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title(title))
            .highlight_style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(" ▶ ");

        f.render_stateful_widget(list, area, &mut self.list_state);
    }

    /// Render analytics panel
    fn render_analytics_panel(&self, f: &mut Frame, area: Rect) {
        let size_mb = self.analytics.total_size as f64 / 1_000_000.0;
        let avg_size_kb = self.analytics.avg_session_size / 1024;

        let mut lines = vec![
            Line::from(""),
            // Overview section
            Line::from(vec![
                Span::styled(" Overview", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::raw("  Total Sessions: "),
                Span::styled(
                    format!("{}", self.analytics.total_sessions),
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::raw("  Total Projects: "),
                Span::styled(
                    format!("{}", self.analytics.total_projects),
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::raw("  Total Size:     "),
                Span::styled(
                    format!("{:.1} MB", size_mb),
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::raw("  Avg Size:       "),
                Span::styled(
                    format!("{} KB", avg_size_kb),
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            // Activity section
            Line::from(vec![
                Span::styled(" Activity", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::raw("  This Week:      "),
                Span::styled(
                    format!("{}", self.analytics.sessions_this_week),
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                ),
                Span::raw(" sessions"),
            ]),
            Line::from(vec![
                Span::raw("  Today:          "),
                Span::styled(
                    format!("{}", self.analytics.sessions_today),
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                ),
                Span::raw(" sessions"),
            ]),
            Line::from(""),
            // Session Types section
            Line::from(vec![
                Span::styled(" Session Types", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::raw("  With Subagents: "),
                Span::styled(
                    format!("{}", self.analytics.subagent_count),
                    Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
        ];

        // Most Active Project
        if let Some(ref project) = self.analytics.most_active_project {
            lines.push(Line::from(vec![
                Span::styled(" Most Active", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            ]));
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    project,
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                ),
            ]));
            lines.push(Line::from(""));
        }

        // Top Tools section
        lines.push(Line::from(vec![
            Span::styled(" Top Tools", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]));
        lines.push(Line::from(""));

        if !self.analytics.top_tools.is_empty() {
            for (tool, count) in &self.analytics.top_tools {
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
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                    ),
                ]));
            }
        } else {
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    "Analyzing...",
                    Style::default().fg(Color::DarkGray),
                ),
            ]));
        }

        let paragraph = Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).title("Analytics"))
            .alignment(ratatui::layout::Alignment::Left);

        f.render_widget(paragraph, area);
    }

    /// Render status bar
    fn render_status(&self, f: &mut Frame, area: Rect) {
        let shortcuts = if self.projects.is_empty() {
            vec![
                Line::from(vec![
                    Span::styled(" Tip: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    Span::raw("Run "),
                    Span::styled("hindsight init", Style::default().fg(Color::Cyan)),
                    Span::raw(" to discover Claude Code sessions"),
                ]),
            ]
        } else {
            vec![
                Line::from(vec![
                    Span::styled(" ↑↓", Style::default().fg(Color::Cyan)),
                    Span::raw(" navigate  "),
                    Span::styled("Enter", Style::default().fg(Color::Cyan)),
                    Span::raw(" select  "),
                    Span::styled("/", Style::default().fg(Color::Cyan)),
                    Span::raw(" filter  "),
                    Span::styled("r", Style::default().fg(Color::Cyan)),
                    Span::raw(" refresh  "),
                    Span::styled("q", Style::default().fg(Color::Cyan)),
                    Span::raw(" quit"),
                ]),
            ]
        };

        let mut text = shortcuts;
        if !self.status_message.is_empty() {
            text.push(Line::from(vec![
                Span::styled(" ", Style::default()),
                Span::styled(self.status_message.as_str(), Style::default().fg(Color::Yellow)),
            ]));
        }

        let status = Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL));

        f.render_widget(status, area);
    }
}

/// Actions that can be triggered from the projects view
#[derive(Debug)]
pub enum ProjectAction {
    None,
    SelectProject(String),
    Quit,
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
    } else {
        format!("{}d ago", diff / 86400)
    }
}
