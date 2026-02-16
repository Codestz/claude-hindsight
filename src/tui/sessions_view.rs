//! Sessions browser view
//!
//! Shows all sessions for a selected project with metadata.

use crate::error::Result;
use crate::storage::{SessionFile, SessionIndex};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

/// Sessions view state
pub struct SessionsView {
    /// Project name
    pub project_name: String,

    /// Sessions for this project
    pub sessions: Vec<SessionFile>,

    /// List selection state
    pub list_state: ListState,

    /// Filter query
    pub filter_query: String,

    /// Whether we're in filter input mode
    pub filter_mode: bool,

    /// Status message
    pub status_message: String,
}

impl SessionsView {
    /// Create a new sessions view for a project
    pub fn new(project_name: String) -> Result<Self> {
        let index = SessionIndex::new()?;
        let sessions = index.find_by_project(&project_name)?;

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
        })
    }

    /// Refresh sessions
    pub fn refresh(&mut self) -> Result<()> {
        let index = SessionIndex::new()?;
        self.sessions = index.find_by_project(&self.project_name)?;

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
            (KeyCode::Char('h'), KeyModifiers::NONE) | (KeyCode::Esc, _) => {
                Ok(SessionAction::Back)
            }

            // Start filter
            (KeyCode::Char('/'), KeyModifiers::NONE) => {
                self.filter_mode = true;
                self.status_message = "Filter: ".to_string();
                Ok(SessionAction::None)
            }

            // Refresh
            (KeyCode::Char('r'), KeyModifiers::NONE) => {
                self.refresh()?;
                self.status_message = "Refreshed session list".to_string();
                Ok(SessionAction::None)
            }

            // Quit
            (KeyCode::Char('q'), KeyModifiers::NONE) => {
                Ok(SessionAction::Quit)
            }

            _ => Ok(SessionAction::None),
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
                .filter(|s| s.session_id.to_lowercase().contains(&query))
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
        self.list_state.selected().and_then(|i| self.sessions.get(i))
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
                Constraint::Min(0),      // Session list
                Constraint::Length(3),   // Status bar
            ])
            .split(area);

        // Render session list
        self.render_list(f, chunks[0]);

        // Render status bar
        self.render_status(f, chunks[1]);
    }

    /// Render the session list
    fn render_list(&mut self, f: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self.sessions
            .iter()
            .map(|session| {
                let size_kb = session.file_size / 1024;
                let time_ago = format_time_ago(Some(session.modified_at));

                // Short session ID (first 8 chars)
                let short_id = if session.session_id.len() > 8 {
                    &session.session_id[..8]
                } else {
                    &session.session_id
                };

                let line = Line::from(vec![
                    Span::styled(
                        format!("{:8}", short_id),
                        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" | "),
                    Span::styled(
                        time_ago,
                        Style::default().fg(Color::Yellow),
                    ),
                    Span::raw(" | "),
                    Span::styled(
                        format!("{:6} KB", size_kb),
                        Style::default().fg(Color::Green),
                    ),
                    Span::raw(if session.has_subagents { " | " } else { "" }),
                    Span::styled(
                        if session.has_subagents { "subagents" } else { "" },
                        Style::default().fg(Color::Magenta),
                    ),
                ]);

                ListItem::new(line)
            })
            .collect();

        let title = format!("Sessions - {}", self.project_name);
        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title(title))
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        f.render_stateful_widget(list, area, &mut self.list_state);
    }

    /// Render status bar
    fn render_status(&self, f: &mut Frame, area: Rect) {
        let shortcuts = vec![
            Line::from(vec![
                Span::styled("j/k", Style::default().fg(Color::Yellow)),
                Span::raw(": Nav | "),
                Span::styled("Enter", Style::default().fg(Color::Yellow)),
                Span::raw(": View | "),
                Span::styled("h/Esc", Style::default().fg(Color::Yellow)),
                Span::raw(": Back | "),
                Span::styled("/", Style::default().fg(Color::Yellow)),
                Span::raw(": Filter | "),
                Span::styled("r", Style::default().fg(Color::Yellow)),
                Span::raw(": Refresh | "),
                Span::styled("q", Style::default().fg(Color::Yellow)),
                Span::raw(": Quit"),
            ]),
            Line::from(self.status_message.as_str()),
        ];

        let status = Paragraph::new(shortcuts)
            .block(Block::default().borders(Borders::ALL));

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
