//! Cross-session prompts browser view
//!
//! Shows all sessions' first messages as prompts across all projects.

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

/// A row in the prompts list
pub struct PromptRow {
    pub session_id: String,
    pub project_name: String,
    pub prompt_text: String,
    pub created_at: i64,
    pub model: Option<String>,
}

/// Prompts view state
pub struct PromptsView {
    pub prompts: Vec<PromptRow>,
    pub list_state: ListState,
    pub filter_query: String,
    pub filter_mode: bool,
    pub status_message: String,
}

impl PromptsView {
    pub fn new() -> Result<Self> {
        let index = SessionIndex::new()?;
        let sessions = index.list_sessions()?;
        let prompts = Self::sessions_to_rows(sessions);

        let mut list_state = ListState::default();
        if !prompts.is_empty() {
            list_state.select(Some(0));
        }

        let status_message = format!("{} prompts across all sessions", prompts.len());

        Ok(PromptsView {
            prompts,
            list_state,
            filter_query: String::new(),
            filter_mode: false,
            status_message,
        })
    }

    fn sessions_to_rows(mut sessions: Vec<SessionFile>) -> Vec<PromptRow> {
        sessions.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        sessions
            .into_iter()
            .filter_map(|s| {
                // Parse actual session file to get full prompt text
                let prompt_text = Self::extract_first_prompt(&s.path)?;
                Some(PromptRow {
                    session_id: s.session_id,
                    project_name: s.project_name,
                    prompt_text,
                    created_at: s.created_at,
                    model: s.model,
                })
            })
            .collect()
    }

    /// Minimum character length for a message to qualify as a prompt.
    const MIN_PROMPT_LENGTH: usize = 300;

    /// Extract the full first real user message from a session file.
    fn extract_first_prompt(path: &std::path::Path) -> Option<String> {
        let parsed = crate::parser::parse_session(path).ok()?;
        parsed
            .nodes
            .iter()
            .filter(|n| n.node_type == "user")
            .find_map(|n| {
                let text = n.message.as_ref()?.text_content();
                let trimmed = text.trim().to_string();
                if trimmed.len() < Self::MIN_PROMPT_LENGTH {
                    return None;
                }
                if crate::analyzer::prompt_detect::is_local_command_text(&trimmed) {
                    return None;
                }
                Some(trimmed)
            })
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Result<PromptAction> {
        if self.filter_mode {
            return self.handle_filter_input(key);
        }

        match (key.code, key.modifiers) {
            (KeyCode::Char('j'), KeyModifiers::NONE) | (KeyCode::Down, _) => {
                self.next();
                Ok(PromptAction::None)
            }
            (KeyCode::Char('k'), KeyModifiers::NONE) | (KeyCode::Up, _) => {
                self.previous();
                Ok(PromptAction::None)
            }
            (KeyCode::Char('g'), KeyModifiers::NONE) | (KeyCode::Home, _) => {
                self.select_first();
                Ok(PromptAction::None)
            }
            (KeyCode::Char('G'), KeyModifiers::SHIFT) | (KeyCode::End, _) => {
                self.select_last();
                Ok(PromptAction::None)
            }
            (KeyCode::Enter, KeyModifiers::NONE) => {
                if let Some(row) = self.selected_prompt() {
                    Ok(PromptAction::SelectSession(row.session_id.clone()))
                } else {
                    Ok(PromptAction::None)
                }
            }
            (KeyCode::Char('h'), KeyModifiers::NONE) | (KeyCode::Esc, _) => Ok(PromptAction::Back),
            (KeyCode::Char('/'), KeyModifiers::NONE) => {
                self.filter_mode = true;
                self.status_message = "Filter: ".to_string();
                Ok(PromptAction::None)
            }
            (KeyCode::Char('r'), KeyModifiers::NONE) => {
                self.refresh()?;
                self.status_message = "Refreshed prompts list".to_string();
                Ok(PromptAction::None)
            }
            (KeyCode::Char('q'), KeyModifiers::NONE) => Ok(PromptAction::Quit),
            _ => Ok(PromptAction::None),
        }
    }

    fn handle_filter_input(&mut self, key: KeyEvent) -> Result<PromptAction> {
        match key.code {
            KeyCode::Enter => {
                self.filter_mode = false;
                self.apply_filter()?;
                Ok(PromptAction::None)
            }
            KeyCode::Esc => {
                self.filter_mode = false;
                self.filter_query.clear();
                self.refresh()?;
                self.status_message = "Filter cancelled".to_string();
                Ok(PromptAction::None)
            }
            KeyCode::Backspace => {
                self.filter_query.pop();
                self.status_message = format!("Filter: {}", self.filter_query);
                Ok(PromptAction::None)
            }
            KeyCode::Char(c) => {
                self.filter_query.push(c);
                self.status_message = format!("Filter: {}", self.filter_query);
                Ok(PromptAction::None)
            }
            _ => Ok(PromptAction::None),
        }
    }

    fn refresh(&mut self) -> Result<()> {
        let index = SessionIndex::new()?;
        let sessions = index.list_sessions()?;
        self.prompts = Self::sessions_to_rows(sessions);

        if self.prompts.is_empty() {
            self.list_state.select(None);
        } else if self.list_state.selected().is_none() {
            self.list_state.select(Some(0));
        }

        self.status_message = format!("{} prompts across all sessions", self.prompts.len());
        Ok(())
    }

    fn apply_filter(&mut self) -> Result<()> {
        let index = SessionIndex::new()?;
        let sessions = index.list_sessions()?;
        let all = Self::sessions_to_rows(sessions);

        if self.filter_query.is_empty() {
            self.prompts = all;
        } else {
            let query = self.filter_query.to_lowercase();
            self.prompts = all
                .into_iter()
                .filter(|r| {
                    r.prompt_text.to_lowercase().contains(&query)
                        || r.project_name.to_lowercase().contains(&query)
                })
                .collect();
        }

        if !self.prompts.is_empty() {
            self.list_state.select(Some(0));
        } else {
            self.list_state.select(None);
        }

        self.status_message = format!("{} prompts match filter", self.prompts.len());
        Ok(())
    }

    fn selected_prompt(&self) -> Option<&PromptRow> {
        self.list_state
            .selected()
            .and_then(|i| self.prompts.get(i))
    }

    fn next(&mut self) {
        if self.prompts.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.prompts.len() - 1 {
                    i
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    fn previous(&mut self) {
        if self.prompts.is_empty() {
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

    fn select_first(&mut self) {
        if !self.prompts.is_empty() {
            self.list_state.select(Some(0));
        }
    }

    fn select_last(&mut self) {
        if !self.prompts.is_empty() {
            self.list_state.select(Some(self.prompts.len() - 1));
        }
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                Constraint::Length(12), // Header
                Constraint::Min(0),     // Content
                Constraint::Length(3),  // Status bar
            ])
            .split(area);

        self.render_header(f, chunks[0]);
        self.render_list(f, chunks[1]);
        self.render_status(f, chunks[2]);
    }

    fn render_header(&self, f: &mut Frame, area: Rect) {
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
                    "Prompts across all sessions — first messages from every conversation",
                    Style::default().fg(Color::Gray),
                ),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    format!("{} prompts", self.prompts.len()),
                    Style::default().fg(Color::Cyan),
                ),
            ]),
        ];

        let header = Paragraph::new(header_text)
            .block(Block::default())
            .alignment(ratatui::layout::Alignment::Left);

        f.render_widget(header, area);
    }

    fn render_list(&mut self, f: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Prompts — sorted by newest first");
        let inner = block.inner(area);
        f.render_widget(block, area);

        let chunks = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(0)])
            .split(inner);

        // Column header
        let header = Paragraph::new(Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(
                format!("{:14}", "Project"),
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
                format!("{:12}", "Model"),
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::UNDERLINED),
            ),
            Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                "Prompt",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::UNDERLINED),
            ),
        ]));
        f.render_widget(header, chunks[0]);

        // Data rows
        let items: Vec<ListItem> = self
            .prompts
            .iter()
            .map(|row| {
                let age_str = format_time_ago(row.created_at);
                let project_display = if row.project_name.len() > 14 {
                    format!("{}…", &row.project_name[..13])
                } else {
                    row.project_name.clone()
                };
                let model_raw = row.model.as_deref().unwrap_or("-");
                let model_display = if model_raw.len() > 12 {
                    &model_raw[..12]
                } else {
                    model_raw
                };
                let preview_end = row
                    .prompt_text
                    .char_indices()
                    .nth(50)
                    .map(|(i, _)| i)
                    .unwrap_or(row.prompt_text.len());
                let preview = &row.prompt_text[..preview_end];

                ListItem::new(Line::from(vec![
                    Span::styled("  ", Style::default()),
                    Span::styled(
                        format!("{:14}", project_display),
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" │ "),
                    Span::styled(
                        format!("{:7}", age_str),
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::raw(" │ "),
                    Span::styled(
                        format!("{:12}", model_display),
                        Style::default().fg(Color::Blue),
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

    fn render_status(&self, f: &mut Frame, area: Rect) {
        let shortcuts = vec![
            Line::from(vec![
                Span::styled("j/k", Style::default().fg(Color::Yellow)),
                Span::raw(": Nav | "),
                Span::styled("Enter", Style::default().fg(Color::Yellow)),
                Span::raw(": View Session | "),
                Span::styled("g/G", Style::default().fg(Color::Yellow)),
                Span::raw(": Top/Bottom | "),
                Span::styled("/", Style::default().fg(Color::Yellow)),
                Span::raw(": Filter | "),
                Span::styled("h/Esc", Style::default().fg(Color::Yellow)),
                Span::raw(": Back | "),
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

/// Actions from the prompts view
#[derive(Debug)]
pub enum PromptAction {
    None,
    SelectSession(String),
    Back,
    Quit,
}

fn format_time_ago(timestamp: i64) -> String {
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
        use chrono::{DateTime, Local};
        let dt = DateTime::from_timestamp(timestamp, 0)
            .unwrap_or_else(|| DateTime::from_timestamp(0, 0).unwrap());
        let local: DateTime<Local> = dt.into();
        local.format("%Y-%m-%d").to_string()
    }
}
