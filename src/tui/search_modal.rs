//! Fuzzy search modal inspired by fzf and Telescope
//!
//! A beautiful, context-aware search overlay that can be triggered from any view.

use crate::error::Result;
use crate::parser::{parse_session, ExecutionNode};
use crate::storage::{SessionFile, SessionIndex};
use chrono::TimeZone;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

/// Search context - determines what we're searching
#[derive(Debug, Clone, PartialEq)]
pub enum SearchContext {
    /// Search all sessions globally
    Global,
    /// Search sessions within a specific project
    Project(String),
    /// Search within a specific session's content
    Session(String),
}

/// A single search result item
#[derive(Debug, Clone)]
pub struct SearchResultItem {
    /// Display title
    pub title: String,
    /// Subtitle/secondary info
    pub subtitle: String,
    /// Preview text (shown in preview pane)
    pub preview: String,
    /// Identifier for selection (session_id, node_uuid, etc.)
    pub id: String,
    /// Match score (higher = better match)
    pub score: f64,
}

/// Search modal state
pub struct SearchModal {
    /// Current search context
    pub context: SearchContext,

    /// Search query input
    pub query: String,

    /// Cursor position in input
    pub cursor_pos: usize,

    /// All matching results
    pub results: Vec<SearchResultItem>,

    /// List state for result selection
    pub list_state: ListState,

    /// Whether search is active
    pub is_active: bool,

    /// Status message
    pub status: String,

    /// Total results before filtering
    pub total_results: usize,
}

impl SearchModal {
    /// Create a new search modal
    pub fn new(context: SearchContext) -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));

        SearchModal {
            context,
            query: String::new(),
            cursor_pos: 0,
            results: Vec::new(),
            list_state,
            is_active: false,
            status: String::new(),
            total_results: 0,
        }
    }

    /// Activate the search modal
    pub fn activate(&mut self) {
        self.is_active = true;
        self.query.clear();
        self.cursor_pos = 0;
        self.results.clear();
        self.list_state.select(Some(0));
        self.update_search().ok(); // Initial load
    }

    /// Deactivate the search modal
    pub fn deactivate(&mut self) {
        self.is_active = false;
    }

    /// Handle keyboard input
    pub fn handle_key(&mut self, key: KeyEvent) -> Result<SearchAction> {
        if !self.is_active {
            return Ok(SearchAction::None);
        }

        match (key.code, key.modifiers) {
            // Exit search
            (KeyCode::Esc, _) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                self.deactivate();
                Ok(SearchAction::Cancel)
            }

            // Select result
            (KeyCode::Enter, _) => {
                if let Some(selected) = self.list_state.selected() {
                    if let Some(result) = self.results.get(selected) {
                        let action = match &self.context {
                            SearchContext::Global | SearchContext::Project(_) => {
                                SearchAction::SelectSession(result.id.clone())
                            }
                            SearchContext::Session(_) => {
                                SearchAction::SelectNode(result.id.clone())
                            }
                        };
                        self.deactivate();
                        return Ok(action);
                    }
                }
                Ok(SearchAction::None)
            }

            // Navigate results
            (KeyCode::Down, _) | (KeyCode::Char('j'), KeyModifiers::CONTROL) => {
                self.next_result();
                Ok(SearchAction::None)
            }
            (KeyCode::Up, _) | (KeyCode::Char('k'), KeyModifiers::CONTROL) => {
                self.previous_result();
                Ok(SearchAction::None)
            }

            // Edit query
            (KeyCode::Char(c), KeyModifiers::NONE) | (KeyCode::Char(c), KeyModifiers::SHIFT) => {
                self.query.insert(self.cursor_pos, c);
                self.cursor_pos += 1;
                self.update_search()?;
                Ok(SearchAction::None)
            }
            (KeyCode::Backspace, _) => {
                if self.cursor_pos > 0 {
                    self.query.remove(self.cursor_pos - 1);
                    self.cursor_pos -= 1;
                    self.update_search()?;
                }
                Ok(SearchAction::None)
            }
            (KeyCode::Delete, _) => {
                if self.cursor_pos < self.query.len() {
                    self.query.remove(self.cursor_pos);
                    self.update_search()?;
                }
                Ok(SearchAction::None)
            }
            (KeyCode::Left, _) => {
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                }
                Ok(SearchAction::None)
            }
            (KeyCode::Right, _) => {
                if self.cursor_pos < self.query.len() {
                    self.cursor_pos += 1;
                }
                Ok(SearchAction::None)
            }
            (KeyCode::Home, _) | (KeyCode::Char('a'), KeyModifiers::CONTROL) => {
                self.cursor_pos = 0;
                Ok(SearchAction::None)
            }
            (KeyCode::End, _) | (KeyCode::Char('e'), KeyModifiers::CONTROL) => {
                self.cursor_pos = self.query.len();
                Ok(SearchAction::None)
            }

            _ => Ok(SearchAction::None),
        }
    }

    /// Move to next result
    fn next_result(&mut self) {
        if self.results.is_empty() {
            return;
        }
        let selected = self.list_state.selected().unwrap_or(0);
        let next = if selected >= self.results.len() - 1 {
            0
        } else {
            selected + 1
        };
        self.list_state.select(Some(next));
    }

    /// Move to previous result
    fn previous_result(&mut self) {
        if self.results.is_empty() {
            return;
        }
        let selected = self.list_state.selected().unwrap_or(0);
        let prev = if selected == 0 {
            self.results.len() - 1
        } else {
            selected - 1
        };
        self.list_state.select(Some(prev));
    }

    /// Update search results based on current query
    fn update_search(&mut self) -> Result<()> {
        match &self.context.clone() {
            SearchContext::Global => self.search_global_sessions(),
            SearchContext::Project(project) => self.search_project_sessions(project),
            SearchContext::Session(session_id) => self.search_session_content(session_id),
        }
    }

    /// Search all sessions globally
    fn search_global_sessions(&mut self) -> Result<()> {
        let index = SessionIndex::new()?;
        let all_sessions = index.list_sessions()?;

        self.total_results = all_sessions.len();

        // Filter and score sessions based on query
        let filtered = if self.query.is_empty() {
            all_sessions
        } else {
            let query_lower = self.query.to_lowercase();
            all_sessions.into_iter()
                .filter(|s| {
                    s.session_id.to_lowercase().contains(&query_lower) ||
                    s.project_name.to_lowercase().contains(&query_lower)
                })
                .collect()
        };

        // Convert to search results
        self.results = filtered.iter()
            .map(|session| session_to_result_item(session))
            .collect();

        // Update status
        self.status = if self.results.len() == self.total_results {
            format!("{} sessions", self.total_results)
        } else {
            format!("{}/{} sessions", self.results.len(), self.total_results)
        };

        // Reset selection
        if !self.results.is_empty() {
            self.list_state.select(Some(0));
        }

        Ok(())
    }

    /// Search sessions within a project
    fn search_project_sessions(&mut self, project: &str) -> Result<()> {
        let index = SessionIndex::new()?;
        let all_sessions = index.find_by_project(project)?;

        self.total_results = all_sessions.len();

        // Filter and score sessions based on query
        let filtered = if self.query.is_empty() {
            all_sessions
        } else {
            let query_lower = self.query.to_lowercase();
            all_sessions.into_iter()
                .filter(|s| {
                    s.session_id.to_lowercase().contains(&query_lower)
                })
                .collect()
        };

        // Convert to search results
        self.results = filtered.iter()
            .map(|session| session_to_result_item(session))
            .collect();

        // Update status
        self.status = if self.results.len() == self.total_results {
            format!("{} sessions in {}", self.total_results, project)
        } else {
            format!("{}/{} sessions in {}", self.results.len(), self.total_results, project)
        };

        // Reset selection
        if !self.results.is_empty() {
            self.list_state.select(Some(0));
        }

        Ok(())
    }

    /// Search within a specific session's content
    fn search_session_content(&mut self, session_id: &str) -> Result<()> {
        // Load the session
        let index = SessionIndex::new()?;
        let session_file = index.find_by_id(session_id)?
            .ok_or_else(|| crate::error::HindsightError::SessionNotFound(session_id.to_string()))?;

        let session = parse_session(&session_file.path)?;

        self.total_results = session.nodes.len();

        // Filter nodes based on query
        let query_lower = self.query.to_lowercase();
        let matching_nodes: Vec<(usize, &ExecutionNode)> = if self.query.is_empty() {
            // Show all nodes if no query
            session.nodes.iter().enumerate().collect()
        } else {
            // Search in node content
            session.nodes.iter().enumerate()
                .filter(|(_, node)| {
                    // Get the actual categorized type (same logic as display)
                    let category = get_node_category(node);

                    // Search in categorized type
                    if category.to_lowercase().contains(&query_lower) {
                        return true;
                    }

                    // Search in tool name
                    if let Some(ref tool_use) = node.tool_use {
                        if tool_use.name.to_lowercase().contains(&query_lower) {
                            return true;
                        }
                    }

                    // Search in message content
                    if let Some(ref message) = node.message {
                        if let Some(ref content) = message.content {
                            if let Some(text) = content.as_str() {
                                if text.to_lowercase().contains(&query_lower) {
                                    return true;
                                }
                            }
                        }
                    }

                    // Search in thinking content
                    if let Some(ref thinking) = node.thinking {
                        if thinking.to_lowercase().contains(&query_lower) {
                            return true;
                        }
                    }

                    // Search in tool result content
                    if let Some(ref tool_result) = node.tool_result {
                        if let Some(ref content) = tool_result.content {
                            if content.to_lowercase().contains(&query_lower) {
                                return true;
                            }
                        }
                    }

                    false
                })
                .collect()
        };

        // Convert to search results
        self.results = matching_nodes.iter()
            .take(100) // Limit to 100 results for performance
            .map(|(idx, node)| node_to_result_item(*idx, node))
            .collect();

        // Update status
        self.status = if self.results.len() == self.total_results {
            format!("{} nodes", self.total_results)
        } else {
            format!("{}/{} nodes", self.results.len(), self.total_results)
        };

        // Reset selection
        if !self.results.is_empty() {
            self.list_state.select(Some(0));
        }

        Ok(())
    }

    /// Render the search modal as an overlay
    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        if !self.is_active {
            return;
        }

        // Create centered modal area (80% width, 70% height)
        let modal_area = centered_rect(80, 70, area);

        // Render background only for modal area (not full screen)
        use ratatui::widgets::Clear;
        let background = Block::default()
            .style(Style::default().bg(Color::Rgb(40, 42, 54))); // Dracula-inspired background
        f.render_widget(Clear, modal_area);
        f.render_widget(background, modal_area);

        // Add padding (1 cell on all sides)
        let padded_area = Rect {
            x: modal_area.x + 1,
            y: modal_area.y + 1,
            width: modal_area.width.saturating_sub(2),
            height: modal_area.height.saturating_sub(2),
        };

        // Split modal into sections
        let chunks = Layout::vertical([
            Constraint::Length(3),  // Input box
            Constraint::Length(1),  // Separator
            Constraint::Min(10),    // Results list
            Constraint::Length(1),  // Separator
            Constraint::Length(8),  // Preview pane
            Constraint::Length(1),  // Status bar
        ])
        .split(padded_area);

        // Render input box
        self.render_input(f, chunks[0]);

        // Render results list
        self.render_results(f, chunks[2]);

        // Render preview
        self.render_preview(f, chunks[4]);

        // Render status bar
        self.render_status(f, chunks[5]);
    }

    /// Render the search input box
    fn render_input(&self, f: &mut Frame, area: Rect) {
        let context_name = match &self.context {
            SearchContext::Global => "Search: All Sessions",
            SearchContext::Project(p) => &format!("Search: Project {}", p),
            SearchContext::Session(_) => "Search: Session Content",
        };

        let title = format!(" {} ", context_name);

        let input_text = if self.query.is_empty() {
            Span::styled("Type to search...", Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC))
        } else {
            Span::styled(&self.query, Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
        };

        let input = Paragraph::new(Line::from(vec![
            Span::styled(" > ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            input_text,
        ]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        );

        f.render_widget(input, area);

        // Position cursor (after the prompt)
        f.set_cursor_position((area.x + 4 + self.cursor_pos as u16, area.y + 1));
    }

    /// Render the results list
    fn render_results(&mut self, f: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self.results.iter()
            .map(|result| {
                // Color-code based on node type
                let title_color = if result.title.contains("[TOOL]") {
                    Color::Yellow
                } else if result.title.contains("[USER]") {
                    Color::Green
                } else if result.title.contains("[ASSISTANT]") {
                    Color::Cyan
                } else if result.title.contains("[THINKING]") {
                    Color::Magenta
                } else if result.title.contains("[RESULT]") {
                    if result.title.contains("ERROR") {
                        Color::Red
                    } else {
                        Color::Blue
                    }
                } else {
                    Color::White
                };

                let title_span = Span::styled(&result.title, Style::default().fg(title_color).add_modifier(Modifier::BOLD));
                let subtitle_span = Span::styled(
                    format!(" │ {}", result.subtitle),
                    Style::default().fg(Color::Gray)
                );

                ListItem::new(Line::from(vec![title_span, subtitle_span]))
            })
            .collect();

        let title = if self.results.is_empty() {
            " No Results ".to_string()
        } else {
            format!(" Results ({}/{}) ", self.results.len(), self.total_results)
        };

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .border_style(Style::default().fg(Color::White))
            )
            .highlight_style(
                Style::default()
                    .bg(Color::Rgb(40, 40, 60))
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            )
            .highlight_symbol("  ");

        f.render_stateful_widget(list, area, &mut self.list_state);
    }

    /// Render the preview pane
    fn render_preview(&self, f: &mut Frame, area: Rect) {
        let preview_text = if let Some(selected) = self.list_state.selected() {
            if let Some(result) = self.results.get(selected) {
                &result.preview
            } else {
                "No preview available"
            }
        } else {
            "No preview available"
        };

        let preview = Paragraph::new(preview_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Preview ")
                    .border_style(Style::default().fg(Color::White))
            )
            .wrap(Wrap { trim: false })
            .style(Style::default().fg(Color::White));

        f.render_widget(preview, area);
    }

    /// Render the status bar
    fn render_status(&self, f: &mut Frame, area: Rect) {
        let help_text = " Enter: Select | Up/Down: Navigate | Esc: Cancel ";

        let status = Paragraph::new(Line::from(vec![
            Span::styled(&self.status, Style::default().fg(Color::Cyan)),
            Span::raw(" ".repeat(area.width.saturating_sub(self.status.len() as u16 + help_text.len() as u16 + 3) as usize)),
            Span::styled(help_text, Style::default().fg(Color::Gray)),
        ]))
        .alignment(Alignment::Left)
        .style(Style::default().bg(Color::Rgb(20, 20, 30)));

        f.render_widget(status, area);
    }
}

/// Action returned by search modal
#[derive(Debug, Clone)]
pub enum SearchAction {
    None,
    Cancel,
    SelectSession(String),
    SelectNode(String),
}

/// Convert a SessionFile to a SearchResultItem
fn session_to_result_item(session: &SessionFile) -> SearchResultItem {
    let time_ago = format_time_ago(session.modified_at);
    let size = format_file_size(session.file_size);

    SearchResultItem {
        title: format!("{} | {}", &session.session_id[..8.min(session.session_id.len())], session.project_name),
        subtitle: format!("{} • {}", time_ago, size),
        preview: format!(
            "Session: {}\nProject: {}\nModified: {}\nSize: {}\nHas subagents: {}",
            session.session_id,
            session.project_name,
            time_ago,
            size,
            if session.has_subagents { "Yes" } else { "No" }
        ),
        id: session.session_id.clone(),
        score: 1.0, // Simple scoring for now
    }
}

/// Get the categorized type of a node (matches display logic)
fn get_node_category(node: &ExecutionNode) -> String {
    if node.tool_use.is_some() {
        "tool".to_string()
    } else if node.tool_result.is_some() {
        "result".to_string()
    } else if node.thinking.is_some() {
        "thinking".to_string()
    } else if node.message.is_some() {
        let node_type_lower = node.node_type.to_lowercase();
        if node_type_lower.contains("user") {
            "user".to_string()
        } else if node_type_lower.contains("assistant") {
            "assistant".to_string()
        } else {
            node.node_type.clone()
        }
    } else {
        node.node_type.clone()
    }
}

/// Convert an ExecutionNode to a SearchResultItem
fn node_to_result_item(index: usize, node: &ExecutionNode) -> SearchResultItem {
    let node_type = &node.node_type;
    let node_type_lower = node_type.to_lowercase();

    // Build title and subtitle based on node type
    // Match by actual content, not just type string
    let (title, subtitle) = if node.tool_use.is_some() {
        // This is a tool use node
        let tool_name = node.tool_use.as_ref()
            .map(|t| t.name.clone())
            .unwrap_or_else(|| "Unknown".to_string());

        let time = node.timestamp
            .map(|ts| format_timestamp(ts))
            .unwrap_or_else(|| "??:??:??".to_string());

        (
            format!("#{} [TOOL] {}", index + 1, tool_name),
            format!("{} • tool use", time)
        )
    } else if node.tool_result.is_some() {
        // This is a tool result node
        let status = node.tool_result.as_ref()
            .and_then(|r| r.is_error)
            .map(|is_err| if is_err { "ERROR" } else { "OK" })
            .unwrap_or("RESULT");

        let time = node.timestamp
            .map(|ts| format_timestamp(ts))
            .unwrap_or_else(|| "??:??:??".to_string());

        (
            format!("#{} [RESULT] {}", index + 1, status),
            format!("{} • tool result", time)
        )
    } else if node.thinking.is_some() {
        // This is a thinking node
        let preview = node.thinking.as_ref()
            .map(|t| {
                let preview: String = t.chars().take(60).collect();
                if t.len() > 60 {
                    format!("{}...", preview)
                } else {
                    preview
                }
            })
            .unwrap_or_else(|| "Thinking...".to_string());

        let time = node.timestamp
            .map(|ts| format_timestamp(ts))
            .unwrap_or_else(|| "??:??:??".to_string());

        (
            format!("#{} [THINKING] {}", index + 1, preview),
            format!("{} • thinking", time)
        )
    } else if node.message.is_some() && node_type_lower.contains("user") {
        // This is a user message node
        // Extract preview of user message
        let preview = node.message.as_ref()
            .and_then(|m| m.content.as_ref())
            .and_then(|c| c.as_str())
            .map(|text| {
                let preview: String = text.chars().take(60).collect();
                if text.len() > 60 {
                    format!("{}...", preview)
                } else {
                    preview
                }
            })
            .unwrap_or_else(|| "User message".to_string());

        let time = node.timestamp
            .map(|ts| format_timestamp(ts))
            .unwrap_or_else(|| "??:??:??".to_string());

        (
            format!("#{} [USER] {}", index + 1, preview),
            format!("{} • user message", time)
        )
    } else if node.message.is_some() && node_type_lower.contains("assistant") {
        // This is an assistant message node
        let time = node.timestamp
            .map(|ts| format_timestamp(ts))
            .unwrap_or_else(|| "??:??:??".to_string());

        (
            format!("#{} [ASSISTANT]", index + 1),
            format!("{} • assistant response", time)
        )
    } else {
        // Fallback for other node types
        let time = node.timestamp
            .map(|ts| format_timestamp(ts))
            .unwrap_or_else(|| "??:??:??".to_string());

        (
            format!("#{} [{}]", index + 1, node_type.to_uppercase()),
            format!("{} • {}", time, node_type)
        )
    };

    // Build preview
    let preview = build_node_preview(node);

    // Use UUID as ID, fallback to index
    let id = node.uuid.clone().unwrap_or_else(|| index.to_string());

    SearchResultItem {
        title,
        subtitle,
        preview,
        id,
        score: 1.0,
    }
}

/// Build a preview text for a node
fn build_node_preview(node: &ExecutionNode) -> String {
    let mut preview = String::new();

    // Show the categorized type (same as display logic)
    let category = get_node_category(node);
    preview.push_str(&format!("Type: {}\n", category));

    if let Some(ref uuid) = node.uuid {
        preview.push_str(&format!("UUID: {}\n", uuid));
    }

    if let Some(ref tool_use) = node.tool_use {
        preview.push_str(&format!("Tool: {}\n", tool_use.name));
        preview.push_str(&format!("Input: {}\n",
            serde_json::to_string_pretty(&tool_use.input).unwrap_or_default()));
    }

    if let Some(ref message) = node.message {
        if let Some(ref content) = message.content {
            if let Some(text) = content.as_str() {
                let preview_text = truncate_string(text, 500);
                preview.push_str(&format!("Content:\n{}\n", preview_text));
            }
        }
    }

    if let Some(ref thinking) = node.thinking {
        let preview_text = truncate_string(thinking, 500);
        preview.push_str(&format!("Thinking:\n{}\n", preview_text));
    }

    if let Some(ref tool_result) = node.tool_result {
        if let Some(is_error) = tool_result.is_error {
            preview.push_str(&format!("Status: {}\n", if is_error { "ERROR" } else { "OK" }));
        }
        if let Some(ref content) = tool_result.content {
            let preview_text = truncate_string(content, 300);
            preview.push_str(&format!("Result:\n{}\n", preview_text));
        }
    }

    preview
}

/// Safely truncate a string at character boundaries, not byte boundaries
fn truncate_string(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max_chars).collect();
        format!("{}...", truncated)
    }
}

/// Format timestamp (milliseconds) as HH:MM:SS
fn format_timestamp(ts_ms: i64) -> String {
    let ts_s = ts_ms / 1000;
    let dt = chrono::Local.timestamp_opt(ts_s, 0);
    match dt.single() {
        Some(datetime) => datetime.format("%H:%M:%S").to_string(),
        None => "??:??:??".to_string(),
    }
}

/// Format timestamp as "time ago" string
fn format_time_ago(timestamp: i64) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    let diff = now - timestamp;

    if diff < 60 {
        "just now".to_string()
    } else if diff < 3600 {
        format!("{}m ago", diff / 60)
    } else if diff < 86400 {
        format!("{}h ago", diff / 3600)
    } else if diff < 604800 {
        format!("{}d ago", diff / 86400)
    } else {
        format!("{}w ago", diff / 604800)
    }
}

/// Format file size as human-readable string
fn format_file_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

/// Create a centered rectangle
fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(area);

    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(popup_layout[1])[1]
}
