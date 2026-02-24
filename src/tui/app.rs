//! TUI application state
//!
//! Manages the state of the interactive terminal UI.

use crate::analyzer::prompt_detect;
use crate::analyzer::{build_simple_tree, SessionAnalytics, TreeNode};
use crate::error::Result;
use crate::parser::models::ContentBlock;
use crate::parser::Session;
use crate::tui::search::SearchState;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::collections::HashMap;
use tui_tree_widget::TreeItem;

/// Scroll position information for displaying scroll indicators
#[derive(Debug, Clone, Default)]
pub struct ScrollInfo {
    pub offset: usize,
    pub total_lines: usize,
    pub viewport_height: usize,
}

impl ScrollInfo {
    pub fn new() -> Self {
        ScrollInfo {
            offset: 0,
            total_lines: 0,
            viewport_height: 0,
        }
    }

    pub fn position_text(&self) -> String {
        if self.total_lines == 0 {
            return String::new();
        }
        let current = self.offset + 1;
        let end = (self.offset + self.viewport_height).min(self.total_lines);
        format!("Lines {}-{}/{}", current, end, self.total_lines)
    }
}

/// Focus mode - which pane is active
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FocusMode {
    /// Tree is focused (default)
    Tree,
    /// Details pane is focused
    Details,
}

/// Application state
pub struct App {
    /// The session being viewed
    pub session: Session,

    /// Conversation-based tree (smart grouping)
    pub tree_roots: Vec<TreeNode>,

    /// Tree widget state
    pub tree_state: tui_tree_widget::TreeState<String>,

    /// Tree items for rendering
    pub tree_items: Vec<TreeItem<'static, String>>,

    /// Mapping from UUID to node (for fast lookup)
    pub uuid_to_node: HashMap<String, TreeNode>,

    /// Total node count
    pub total_nodes: usize,

    /// Current focus mode
    pub focus_mode: FocusMode,

    /// Scroll offset for details pane
    pub details_scroll: usize,

    /// Scroll information for details pane
    pub details_scroll_info: ScrollInfo,

    /// Search state (None if not searching)
    pub search_state: Option<SearchState>,

    /// Whether we're in input mode (typing search query)
    pub input_mode: bool,

    /// Whether to quit the application
    pub should_quit: bool,

    /// Status message
    pub status_message: String,

    /// Session-level analytics
    pub analytics: SessionAnalytics,

    /// tool_use_id → tool_name (for ToolResult correlation in detail panel and tree labels)
    pub tool_correlation: HashMap<String, String>,

    /// tool_use_id → brief result summary ("✓ 42 lines" / "✗ error msg")
    pub tool_result_map: HashMap<String, String>,

    /// Last time search input was modified (for debouncing)
    pub last_search_input_time: Option<std::time::Instant>,

    /// Last executed search query (to avoid re-executing same query)
    pub last_search_query: Option<String>,

    // ── #7: Error navigation ──────────────────────────────────────────────
    /// UUIDs of error nodes in session order (for E-key cycling)
    pub error_node_uuids: Vec<String>,
    /// Current position in error_node_uuids
    pub current_error_idx: usize,

    // ── #9: Raw JSON view ────────────────────────────────────────────────
    /// When true, details panel shows raw JSON of selected node (J to toggle)
    pub show_raw_json: bool,

    // ── #14: Error summary overlay ────────────────────────────────────────
    /// When true, show the error summary popup
    pub show_error_summary: bool,
    /// Selected row inside the error summary popup
    pub error_summary_selection: usize,
    /// (uuid, node_type, short description) for each error node
    pub error_nodes_info: Vec<(String, String, String)>,

    // ── #17: Replay mode ─────────────────────────────────────────────────
    /// Auto-advance through nodes when true (P to toggle)
    pub replay_mode: bool,
    /// Timestamp of last replay advance
    pub last_replay_tick: Option<std::time::Instant>,

    // ── #18: Diff view ───────────────────────────────────────────────────
    /// Show old/new diff for Edit tool calls (d to toggle)
    pub show_diff: bool,

    // ── Prompt scores ──────────────────────────────────────────────────
    /// UUID → prompt confidence score (0–100) for user nodes
    pub prompt_scores: HashMap<String, u8>,
}

impl App {
    /// Create a new app from a session
    pub fn new(session: Session) -> Self {
        let total_nodes = session.nodes.len();

        // Calculate session analytics
        let analytics = SessionAnalytics::from_session(&session);

        // Build tool correlation map: tool_use_id → tool_name
        let mut tool_correlation: HashMap<String, String> = HashMap::new();
        for node in &session.nodes {
            if let Some(ref msg) = node.message {
                for block in msg.content_blocks() {
                    if let ContentBlock::ToolUse { id, name, .. } = block {
                        tool_correlation.insert(id.clone(), name.clone());
                    }
                }
            }
        }

        // Build tool result map: tool_use_id → brief summary
        let mut tool_result_map: HashMap<String, String> = HashMap::new();
        for node in &session.nodes {
            // Path 1: ToolResult content blocks inside user messages
            if let Some(ref msg) = node.message {
                for block in msg.content_blocks() {
                    if let ContentBlock::ToolResult {
                        tool_use_id,
                        content,
                        is_error,
                    } = block
                    {
                        let is_err = is_error.unwrap_or(false);
                        let prefix = if is_err { "✗" } else { "✓" };
                        let text = content.as_ref().and_then(|v| {
                            if let Some(s) = v.as_str() {
                                Some(s.to_string())
                            } else if let Some(arr) = v.as_array() {
                                arr.iter()
                                    .find_map(|b| b.get("text").and_then(|t| t.as_str()))
                                    .map(str::to_string)
                            } else {
                                None
                            }
                        });
                        let summary = match text.as_deref() {
                            None | Some("") => format!("{} ok", prefix),
                            Some(t) => {
                                let lines = t.lines().count();
                                let first = t.lines().next().unwrap_or("").trim();
                                let short: String = first.chars().take(60).collect();
                                if lines > 1 {
                                    format!("{} {} ({} lines)", prefix, short, lines)
                                } else {
                                    format!("{} {}", prefix, short)
                                }
                            }
                        };
                        tool_result_map.insert(tool_use_id.clone(), summary);
                    }
                }
            }
            // Path 2: top-level tool_result field
            if let Some(ref result) = node.tool_result {
                if let Some(ref id) = result.tool_use_id {
                    let summary = if result.is_error == Some(true) {
                        let err = result
                            .error
                            .as_deref()
                            .or(result.content.as_deref())
                            .unwrap_or("error");
                        let first = err.lines().next().unwrap_or("").trim();
                        format!("✗ {}", &first.chars().take(60).collect::<String>())
                    } else if let Some(ref file) = result.file {
                        let name = file
                            .file_path
                            .as_deref()
                            .and_then(|p| p.rsplit('/').next())
                            .unwrap_or("file");
                        let lines = file
                            .content
                            .as_deref()
                            .map(|c| c.lines().count())
                            .unwrap_or(0);
                        if lines > 0 {
                            format!("✓ {} ({} lines)", name, lines)
                        } else {
                            format!("✓ {}", name)
                        }
                    } else if let Some(ref content) = result.content {
                        let lines = content.lines().count();
                        let first = content.lines().next().unwrap_or("").trim();
                        let short: String = first.chars().take(60).collect();
                        if lines > 1 {
                            format!("✓ {} ({} lines)", short, lines)
                        } else {
                            format!("✓ {}", short)
                        }
                    } else {
                        "✓ ok".to_string()
                    };
                    tool_result_map.insert(id.clone(), summary);
                }
            }
        }

        // Compute prompt scores for user nodes (tracking position context)
        let prompt_scores = {
            let mut scores = HashMap::new();
            let mut seen_first_user = false;
            let mut prev_node_type: Option<&str> = None;
            for node in &session.nodes {
                if node.node_type == "user" {
                    if let Some(ref uuid) = node.uuid {
                        let is_first = !seen_first_user;
                        let is_after_assistant =
                            prev_node_type.map(|t| t == "assistant").unwrap_or(false);
                        let score =
                            prompt_detect::prompt_score(node, is_first, is_after_assistant);
                        if score > 0 {
                            scores.insert(uuid.clone(), score);
                        }
                        seen_first_user = true;
                    }
                }
                prev_node_type = Some(&node.node_type);
            }
            scores
        };

        // Build simple hierarchical tree from parent_uuid relationships
        let tree_roots = build_simple_tree(session.nodes.clone());

        // Build UUID-to-node mapping for fast lookup
        let mut uuid_to_node = HashMap::new();
        for root in &tree_roots {
            collect_uuid_mapping(root, &mut uuid_to_node);
        }

        // Build tree items for tui-tree-widget (using UUIDs as identifiers)
        let tree_items = build_tree_items(&tree_roots, &None, &tool_correlation, &prompt_scores);

        let mut tree_state = tui_tree_widget::TreeState::default();
        tree_state.select_first();

        // ── #7 / #14: collect error nodes ───────────────────────────────
        // Broadened to match Session::new() error detection: top-level tool_result.is_error,
        // <tool_use_error> tags in content, and ContentBlock::ToolResult with is_error.
        let (error_node_uuids, error_nodes_info) = {
            let mut uuids = vec![];
            let mut info = vec![];
            for node in &session.nodes {
                let tr = node.tool_result.as_ref();
                let flag_error = tr.and_then(|r| r.is_error).unwrap_or(false);
                let tag_error = tr
                    .and_then(|r| r.content.as_deref())
                    .map(|c| c.contains("<tool_use_error>"))
                    .unwrap_or(false);
                let block_error = node
                    .message
                    .as_ref()
                    .map(|m| {
                        m.content_blocks().iter().any(|b| match b {
                            ContentBlock::ToolResult {
                                content, is_error, ..
                            } => {
                                is_error.unwrap_or(false)
                                    || content
                                        .as_ref()
                                        .and_then(|v| v.as_str())
                                        .map(|s| s.contains("<tool_use_error>"))
                                        .unwrap_or(false)
                            }
                            _ => false,
                        })
                    })
                    .unwrap_or(false);

                let is_err =
                    node.node_type == "error" || flag_error || tag_error || block_error;

                if is_err {
                    if let Some(ref uuid) = node.uuid {
                        let desc: String = if node.node_type == "error" {
                            node.extra
                                .as_ref()
                                .and_then(|e| e.get("error"))
                                .and_then(|v| v.as_str())
                                .unwrap_or("Unknown error")
                                .chars()
                                .take(50)
                                .collect()
                        } else {
                            node.tool_result
                                .as_ref()
                                .and_then(|r| {
                                    r.error
                                        .as_ref()
                                        .or(r.content.as_ref())
                                })
                                .map(|e| e.chars().take(50).collect::<String>())
                                .unwrap_or_else(|| "Tool error".to_string())
                        };
                        uuids.push(uuid.clone());
                        info.push((uuid.clone(), node.node_type.clone(), desc));
                    }
                }
            }
            (uuids, info)
        };

        App {
            session,
            tree_roots,
            tree_state,
            tree_items,
            uuid_to_node,
            total_nodes,
            focus_mode: FocusMode::Tree,
            details_scroll: 0,
            details_scroll_info: ScrollInfo::new(),
            search_state: None,
            input_mode: false,
            should_quit: false,
            status_message: String::new(),
            analytics,
            tool_correlation,
            tool_result_map,
            last_search_input_time: None,
            last_search_query: None,
            error_node_uuids,
            current_error_idx: 0,
            show_raw_json: false,
            show_error_summary: false,
            error_summary_selection: 0,
            error_nodes_info,
            replay_mode: false,
            last_replay_tick: None,
            show_diff: false,
            prompt_scores,
        }
    }

    // ── #7: Error navigation ─────────────────────────────────────────────

    /// Build the full selection path from root to the given UUID.
    ///
    /// tui-tree-widget requires `vec![root_uuid, ..., parent_uuid, target_uuid]`
    /// to select a nested node. This walks `parent_uuid` links upward, then reverses.
    fn build_selection_path(&self, uuid: &str) -> Vec<String> {
        let mut path = vec![uuid.to_string()];
        let mut current = uuid.to_string();
        while let Some(node) = self.uuid_to_node.get(&current) {
            if let Some(ref parent) = node.node.parent_uuid {
                path.push(parent.clone());
                current = parent.clone();
            } else {
                break;
            }
        }
        path.reverse();
        path
    }

    /// Jump to the next error node in the session
    pub fn jump_to_next_error(&mut self) {
        if self.error_node_uuids.is_empty() {
            self.status_message = "No errors in this session".to_string();
            return;
        }
        self.current_error_idx = (self.current_error_idx + 1) % self.error_node_uuids.len();
        let uuid = self.error_node_uuids[self.current_error_idx].clone();
        let path = self.build_selection_path(&uuid);
        self.tree_state.select(path);
        self.details_scroll = 0;
        self.status_message = format!(
            "Error {}/{}",
            self.current_error_idx + 1,
            self.error_node_uuids.len()
        );
    }

    /// Jump to the previous error node in the session
    pub fn jump_to_prev_error(&mut self) {
        if self.error_node_uuids.is_empty() {
            self.status_message = "No errors in this session".to_string();
            return;
        }
        if self.current_error_idx == 0 {
            self.current_error_idx = self.error_node_uuids.len() - 1;
        } else {
            self.current_error_idx -= 1;
        }
        let uuid = self.error_node_uuids[self.current_error_idx].clone();
        let path = self.build_selection_path(&uuid);
        self.tree_state.select(path);
        self.details_scroll = 0;
        self.status_message = format!(
            "Error {}/{}",
            self.current_error_idx + 1,
            self.error_node_uuids.len()
        );
    }

    // ── #8: Clipboard ────────────────────────────────────────────────────

    /// Copy the rendered content of the selected node to the OS clipboard
    pub fn copy_node_to_clipboard(&mut self) {
        let text = if let Some(node) = self.selected_node() {
            let ctx = crate::tui::render::RenderContext {
                tool_correlation: &self.tool_correlation,
                tool_result_map: &self.tool_result_map,
            };
            let lines = crate::tui::render::render_node_content(node, &ctx);
            lines
                .iter()
                .map(|line| {
                    line.spans
                        .iter()
                        .map(|s| s.content.as_ref())
                        .collect::<String>()
                })
                .collect::<Vec<_>>()
                .join("\n")
        } else {
            self.status_message = "Nothing selected".to_string();
            return;
        };

        match arboard::Clipboard::new() {
            Ok(mut cb) => match cb.set_text(text) {
                Ok(_) => self.status_message = "Copied to clipboard".to_string(),
                Err(_) => self.status_message = "Copy failed".to_string(),
            },
            Err(_) => self.status_message = "Clipboard unavailable".to_string(),
        }
    }

    // ── #9: Raw JSON view ────────────────────────────────────────────────

    /// Toggle raw JSON display for the selected node
    pub fn toggle_raw_json(&mut self) {
        self.show_raw_json = !self.show_raw_json;
        self.show_diff = false;
        self.status_message = if self.show_raw_json {
            "Raw JSON (J: off)".to_string()
        } else {
            "Rendered view".to_string()
        };
    }

    // ── #14: Error summary overlay ────────────────────────────────────────

    /// Toggle the error summary popup
    pub fn toggle_error_summary(&mut self) {
        self.show_error_summary = !self.show_error_summary;
        if self.show_error_summary {
            self.error_summary_selection = 0;
        }
    }

    /// Handle keys when error summary overlay is open
    fn handle_error_summary_key(&mut self, key: crossterm::event::KeyEvent) -> Result<()> {
        use crossterm::event::KeyCode;
        match key.code {
            KeyCode::Esc | KeyCode::Char('x') | KeyCode::Char('X') => {
                self.show_error_summary = false;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                let max = self.error_nodes_info.len().saturating_sub(1);
                if self.error_summary_selection < max {
                    self.error_summary_selection += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if self.error_summary_selection > 0 {
                    self.error_summary_selection -= 1;
                }
            }
            KeyCode::Enter => {
                if let Some((uuid, _, _)) = self.error_nodes_info.get(self.error_summary_selection)
                {
                    let uuid = uuid.clone();
                    let path = self.build_selection_path(&uuid);
                    self.tree_state.select(path);
                    self.details_scroll = 0;
                    self.show_error_summary = false;
                }
            }
            _ => {}
        }
        Ok(())
    }

    // ── #17: Replay mode ─────────────────────────────────────────────────

    /// Toggle auto-replay through tree nodes
    pub fn toggle_replay(&mut self) {
        self.replay_mode = !self.replay_mode;
        if self.replay_mode {
            self.last_replay_tick = Some(std::time::Instant::now());
            self.status_message = "▶ Replay — press P or any key to stop".to_string();
        } else {
            self.last_replay_tick = None;
            self.status_message = "■ Replay stopped".to_string();
        }
    }

    // ── #18: Diff view ───────────────────────────────────────────────────

    /// Toggle diff view for Edit tool calls
    pub fn toggle_diff(&mut self) {
        self.show_diff = !self.show_diff;
        self.show_raw_json = false;
        self.status_message = if self.show_diff {
            "Diff view (d: off)".to_string()
        } else {
            "Rendered view".to_string()
        };
    }

    /// Update scroll info (called by UI during rendering)
    pub fn update_scroll_info(&mut self, total_lines: usize, viewport_height: usize) {
        self.details_scroll_info.offset = self.details_scroll;
        self.details_scroll_info.total_lines = total_lines;
        self.details_scroll_info.viewport_height = viewport_height;
    }

    /// Get breadcrumb path for currently selected node
    pub fn get_breadcrumb_path(&self) -> Vec<String> {
        let selected = match self.selected_node() {
            Some(node) => node,
            None => return vec![],
        };

        let mut path = vec![];
        let mut current_uuid = selected.node.uuid.clone();

        while let Some(uuid) = current_uuid {
            if let Some(node) = self.uuid_to_node.get(&uuid) {
                let label = match node.node.node_type.as_str() {
                    "user" => "User".to_string(),
                    "assistant" => "Assistant".to_string(),
                    "tool_use" => node
                        .node
                        .tool_use
                        .as_ref()
                        .map(|t| format!("Tool:{}", t.name))
                        .unwrap_or_else(|| "Tool".to_string()),
                    "tool_result" => "Result".to_string(),
                    "thinking" => "Think".to_string(),
                    "progress" => "Progress".to_string(),
                    _ => node.node.node_type.clone(),
                };
                path.push(label);
                current_uuid = node.node.parent_uuid.clone();
            } else {
                break;
            }
        }

        path.reverse();
        path
    }

    /// Start type-filter search mode (`/`)
    pub fn start_search(&mut self) {
        self.input_mode = true;
        self.search_state = Some(SearchState::new(String::new()));
        self.status_message =
            "Filter by type (error, prompt, tool, user, assistant): ".to_string();
    }

    /// Start keyword search mode (`?`)
    pub fn start_keyword_search(&mut self) {
        self.input_mode = true;
        self.search_state = Some(SearchState::new_keyword(String::new()));
        self.status_message = "Search content: ".to_string();
    }

    /// Execute search with current query
    pub fn execute_search(&mut self) {
        use crate::tui::search::SearchMode;

        let first_match_uuid = if let Some(ref mut search) = self.search_state {
            match search.mode {
                SearchMode::TypeFilter => {
                    // Parse the query into node types
                    search.parse_query();
                }
                SearchMode::KeywordSearch => {
                    // Keyword mode doesn't need parse_query
                }
            }

            search.matches.clear();

            // Find all matching nodes
            for (uuid, node) in &self.uuid_to_node {
                if search.matches_node(node) {
                    search.matches.push(uuid.clone());
                }
            }

            search.current_match = 0;

            self.status_message = match search.mode {
                SearchMode::TypeFilter => {
                    let filter_types = search
                        .node_types
                        .iter()
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join(", ");

                    if search.node_types.is_empty() {
                        "Filter cleared - showing all nodes".to_string()
                    } else {
                        format!(
                            "Filtered: {} ({} matches)",
                            filter_types,
                            search.matches.len()
                        )
                    }
                }
                SearchMode::KeywordSearch => {
                    if search.query.is_empty() {
                        "Search cleared - showing all nodes".to_string()
                    } else {
                        format!(
                            "Search: \"{}\" ({} matches)",
                            search.query,
                            search.matches.len()
                        )
                    }
                }
            };

            search.current_match_uuid().map(|s| s.to_string())
        } else {
            None
        };

        // Rebuild tree with filtering
        self.rebuild_tree_items();

        // Jump to first match if any
        if let Some(ref uuid) = first_match_uuid {
            let path = self.build_selection_path(uuid);
            self.tree_state.select(path);
            self.details_scroll = 0;
        }

        self.input_mode = false;
    }

    /// Select a node by UUID (used when jumping from search results)
    pub fn select_node_by_uuid(&mut self, uuid: &str) {
        // Check if the UUID exists in our mapping
        if self.uuid_to_node.contains_key(uuid) {
            let path = self.build_selection_path(uuid);
            self.tree_state.select(path);
            self.details_scroll = 0;
            self.status_message = format!("Jumped to node: {}", &uuid[..8.min(uuid.len())]);
        } else {
            self.status_message = format!("Node not found: {}", &uuid[..8.min(uuid.len())]);
        }
    }

    /// Process periodic updates (called on each event loop tick)
    ///
    /// Handles debounced search execution - waits 150ms after last keystroke
    /// before executing the search to avoid rebuilding tree on every character.
    /// Also advances replay mode every 800ms.
    pub fn tick(&mut self) {
        // ── #17: Replay advance ──────────────────────────────────────────
        if self.replay_mode {
            let should_advance = self
                .last_replay_tick
                .map(|t| t.elapsed() >= std::time::Duration::from_millis(800))
                .unwrap_or(true);
            if should_advance {
                self.tree_state.key_down();
                self.details_scroll = 0;
                self.last_replay_tick = Some(std::time::Instant::now());
            }
        }

        // Check if we should execute a debounced search
        if let Some(last_input) = self.last_search_input_time {
            if last_input.elapsed() > std::time::Duration::from_millis(150) {
                // Check if we need to execute search (query has changed)
                let should_execute = if let Some(ref search) = self.search_state {
                    let current_query = search.query.clone();
                    self.last_search_query.as_ref() != Some(&current_query)
                } else {
                    false
                };

                if should_execute {
                    self.execute_search();
                    if let Some(ref search) = self.search_state {
                        self.last_search_query = Some(search.query.clone());
                    }
                }
                self.last_search_input_time = None;
            }
        }
    }

    /// Jump to next search match
    pub fn next_search_match(&mut self) {
        let info = if let Some(ref mut search) = self.search_state {
            search.next_match();
            search.current_match_uuid().map(|uuid| {
                (
                    uuid.to_string(),
                    search.current_match + 1,
                    search.matches.len(),
                )
            })
        } else {
            None
        };
        if let Some((uuid, current, total)) = info {
            let path = self.build_selection_path(&uuid);
            self.tree_state.select(path);
            self.details_scroll = 0;
            self.status_message = format!("Match {}/{}", current, total);
        }
    }

    /// Jump to previous search match
    pub fn prev_search_match(&mut self) {
        let info = if let Some(ref mut search) = self.search_state {
            search.prev_match();
            search.current_match_uuid().map(|uuid| {
                (
                    uuid.to_string(),
                    search.current_match + 1,
                    search.matches.len(),
                )
            })
        } else {
            None
        };
        if let Some((uuid, current, total)) = info {
            let path = self.build_selection_path(&uuid);
            self.tree_state.select(path);
            self.details_scroll = 0;
            self.status_message = format!("Match {}/{}", current, total);
        }
    }

    /// Cancel search
    pub fn cancel_search(&mut self) {
        self.search_state = None;
        self.input_mode = false;
        self.rebuild_tree_items();
        self.status_message = "Search cancelled".to_string();
    }

    /// Rebuild tree items (used when search state changes)
    pub fn rebuild_tree_items(&mut self) {
        self.tree_items = build_tree_items(
            &self.tree_roots,
            &self.search_state,
            &self.tool_correlation,
            &self.prompt_scores,
        );
    }

    /// Handle keyboard input
    pub fn handle_key(&mut self, key: KeyEvent) -> Result<()> {
        // Handle input mode separately
        if self.input_mode {
            return self.handle_input_key(key);
        }

        // ── #14: Error summary overlay intercepts keys ────────────────────
        if self.show_error_summary {
            return self.handle_error_summary_key(key);
        }

        // ── #17: Any key (except P) exits replay mode ────────────────────
        if self.replay_mode && !matches!(key.code, KeyCode::Char('p')) {
            self.replay_mode = false;
            self.last_replay_tick = None;
            self.status_message = "■ Replay stopped".to_string();
            return Ok(());
        }

        match (key.code, key.modifiers) {
            // Quit
            (KeyCode::Char('q'), KeyModifiers::NONE) => {
                self.should_quit = true;
            }
            (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                self.should_quit = true;
            }

            // Start type filter search
            (KeyCode::Char('/'), KeyModifiers::NONE) => {
                self.start_search();
            }

            // Start keyword content search
            (KeyCode::Char('?'), KeyModifiers::SHIFT) => {
                self.start_keyword_search();
            }

            // Next match
            (KeyCode::Char('n'), KeyModifiers::NONE) => {
                if self.search_state.is_some() {
                    self.next_search_match();
                }
            }

            // Previous match
            (KeyCode::Char('N'), KeyModifiers::SHIFT) => {
                if self.search_state.is_some() {
                    self.prev_search_match();
                }
            }

            // Clear search (Alt+c)
            (KeyCode::Char('c'), KeyModifiers::ALT) => {
                if self.search_state.is_some() {
                    self.cancel_search();
                }
            }

            // Navigation - depends on focus mode
            (KeyCode::Char('j'), KeyModifiers::NONE) | (KeyCode::Down, _) => {
                match self.focus_mode {
                    FocusMode::Tree => {
                        self.tree_state.key_down();
                        self.details_scroll = 0; // Reset scroll when changing nodes
                    }
                    FocusMode::Details => {
                        self.details_scroll = self.details_scroll.saturating_add(1);
                    }
                }
            }
            (KeyCode::Char('k'), KeyModifiers::NONE) | (KeyCode::Up, _) => {
                match self.focus_mode {
                    FocusMode::Tree => {
                        self.tree_state.key_up();
                        self.details_scroll = 0; // Reset scroll when changing nodes
                    }
                    FocusMode::Details => {
                        self.details_scroll = self.details_scroll.saturating_sub(1);
                    }
                }
            }

            // Half-page scroll (Ctrl+d/u)
            (KeyCode::Char('d'), KeyModifiers::CONTROL) => {
                if self.focus_mode == FocusMode::Details {
                    self.details_scroll = self.details_scroll.saturating_add(15);
                    self.status_message = "↓ Half page".to_string();
                }
            }
            (KeyCode::Char('u'), KeyModifiers::CONTROL) => {
                if self.focus_mode == FocusMode::Details {
                    self.details_scroll = self.details_scroll.saturating_sub(15);
                    self.status_message = "↑ Half page".to_string();
                }
            }

            // Full-page scroll (Ctrl+f/b or PageDown/PageUp)
            (KeyCode::Char('f'), KeyModifiers::CONTROL) | (KeyCode::PageDown, _) => {
                if self.focus_mode == FocusMode::Details {
                    self.details_scroll = self.details_scroll.saturating_add(30);
                    self.status_message = "↓ Full page".to_string();
                }
            }
            (KeyCode::Char('b'), KeyModifiers::CONTROL) | (KeyCode::PageUp, _) => {
                if self.focus_mode == FocusMode::Details {
                    self.details_scroll = self.details_scroll.saturating_sub(30);
                    self.status_message = "↑ Full page".to_string();
                }
            }

            // Focus switching
            (KeyCode::Tab, KeyModifiers::NONE) => {
                self.focus_mode = match self.focus_mode {
                    FocusMode::Tree => {
                        self.status_message = "Focus: Details (use j/k to scroll)".to_string();
                        FocusMode::Details
                    }
                    FocusMode::Details => {
                        self.status_message = "Focus: List".to_string();
                        FocusMode::Tree
                    }
                };
            }

            // Home/End
            (KeyCode::Home, _) | (KeyCode::Char('g'), KeyModifiers::NONE) => {
                self.tree_state.select_first();
                self.status_message = "↑ Top".to_string();
            }
            (KeyCode::End, _) | (KeyCode::Char('G'), KeyModifiers::SHIFT) => {
                self.tree_state.select_last();
                self.status_message = "↓ Bottom".to_string();
            }

            // ── #7: Error navigation ──────────────────────────────────────
            (KeyCode::Char('e'), KeyModifiers::NONE) => self.jump_to_next_error(),
            (KeyCode::Char('E'), KeyModifiers::SHIFT) => self.jump_to_prev_error(),

            // ── #8: Clipboard ─────────────────────────────────────────────
            (KeyCode::Char('y'), KeyModifiers::NONE) => self.copy_node_to_clipboard(),

            // ── #9: Raw JSON view ─────────────────────────────────────────
            (KeyCode::Char('J'), KeyModifiers::SHIFT) => self.toggle_raw_json(),

            // ── #14: Error summary overlay ────────────────────────────────
            (KeyCode::Char('x'), KeyModifiers::NONE)
            | (KeyCode::Char('X'), KeyModifiers::SHIFT) => {
                self.toggle_error_summary();
            }

            // ── #17: Replay mode ──────────────────────────────────────────
            (KeyCode::Char('p'), KeyModifiers::NONE) => self.toggle_replay(),

            // ── #18: Diff view ────────────────────────────────────────────
            (KeyCode::Char('d'), KeyModifiers::NONE) => self.toggle_diff(),

            _ => {}
        }

        Ok(())
    }

    /// Handle keyboard input in input mode (for search)
    fn handle_input_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Enter => {
                // Immediate execution on Enter (no debouncing)
                self.execute_search();
                self.last_search_input_time = None;
                if let Some(ref search) = self.search_state {
                    self.last_search_query = Some(search.query.clone());
                }
            }
            KeyCode::Esc => {
                self.cancel_search();
                self.last_search_input_time = None;
                self.last_search_query = None;
            }
            KeyCode::Backspace => {
                if let Some(ref mut search) = self.search_state {
                    search.query.pop();
                    self.status_message = format!("Search: {}", search.query);
                    // Set debounce timer for search execution
                    self.last_search_input_time = Some(std::time::Instant::now());
                }
            }
            KeyCode::Char(c) => {
                if let Some(ref mut search) = self.search_state {
                    search.query.push(c);
                    self.status_message = format!("Search: {}", search.query);
                    // Set debounce timer for search execution
                    self.last_search_input_time = Some(std::time::Instant::now());
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Get the currently selected node
    pub fn selected_node(&self) -> Option<&TreeNode> {
        // Get selected UUID from tree state
        let selected = self.tree_state.selected();
        if let Some(uuid) = selected.first() {
            // Look up node by UUID
            self.uuid_to_node.get(uuid)
        } else {
            None
        }
    }
}

/// Build UUID-to-node mapping recursively
fn collect_uuid_mapping(node: &TreeNode, mapping: &mut HashMap<String, TreeNode>) {
    if let Some(ref uuid) = node.node.uuid {
        mapping.insert(uuid.clone(), node.clone());
    }
    for child in &node.children {
        collect_uuid_mapping(child, mapping);
    }
}

/// Compact token count formatter for tree badges (e.g. 1234 → "1k", 1500000 → "1.5M")
fn fmt_compact(n: i64) -> String {
    match n {
        n if n < 1_000 => format!("{}", n),
        n if n < 1_000_000 => format!("{:.0}k", n as f64 / 1_000.0),
        n => format!("{:.1}M", n as f64 / 1_000_000.0),
    }
}

/// Build tree items for tui-tree-widget using UUIDs as identifiers
fn build_tree_items(
    roots: &[TreeNode],
    search_state: &Option<SearchState>,
    correlation: &HashMap<String, String>,
    prompt_scores: &HashMap<String, u8>,
) -> Vec<TreeItem<'static, String>> {
    // Use the first root node's timestamp as session start for latency deltas
    let session_start = roots.iter().filter_map(|n| n.node.timestamp).next();
    roots
        .iter()
        .filter_map(|root| {
            build_tree_item(root, search_state, correlation, session_start, prompt_scores)
        })
        .collect()
}

/// Recursively build a tree item using UUID as identifier
/// Returns None if the node and all its children are filtered out
fn build_tree_item(
    node: &TreeNode,
    search_state: &Option<SearchState>,
    correlation: &HashMap<String, String>,
    parent_timestamp: Option<i64>,
    prompt_scores: &HashMap<String, u8>,
) -> Option<TreeItem<'static, String>> {
    use ratatui::style::{Color, Style};
    use ratatui::text::{Line, Span};

    // Check if this node matches the filter
    let matches_filter = search_state
        .as_ref()
        .map(|s| s.matches_node(node))
        .unwrap_or(true);

    // Build children with this node's timestamp as their parent reference
    let children: Vec<TreeItem<String>> = node
        .children
        .iter()
        .filter_map(|child| {
            build_tree_item(
                child,
                search_state,
                correlation,
                node.node.timestamp,
                prompt_scores,
            )
        })
        .collect();

    // If this node doesn't match AND has no matching children, filter it out
    if !matches_filter && children.is_empty() {
        return None;
    }

    // Use UUID as identifier (or generate one for nodes without UUID)
    let identifier = node
        .node
        .uuid
        .clone()
        .unwrap_or_else(|| format!("no-uuid-{}", node.node.node_type));

    // Get smart label with color (pass correlation so ToolResult shows tool name)
    let (label_text, color_name) =
        crate::analyzer::smart_label::get_node_label(node, Some(correlation));

    // Map color name to ratatui Color
    let color = match color_name {
        "cyan" => Color::Cyan,
        "green" => Color::Green,
        "yellow" => Color::Yellow,
        "blue" => Color::Blue,
        "magenta" => Color::Magenta,
        "red" => Color::Red,
        "gray" => Color::Gray,
        _ => Color::White,
    };

    // Start with the label span
    let mut spans = vec![Span::styled(label_text, Style::default().fg(color))];

    // Prompt score badge for user nodes with score >= 40
    if let Some(uuid) = &node.node.uuid {
        if let Some(&score) = prompt_scores.get(uuid) {
            if score >= prompt_detect::PROMPT_THRESHOLD {
                spans.push(Span::styled(
                    format!(" P:{}%", score),
                    Style::default().fg(Color::Green),
                ));
            }
        }
    }

    // Latency delta badge: show +Xs gap from parent/session-start (only if >= 500ms)
    if let (Some(node_ts), Some(parent_ts)) = (node.node.timestamp, parent_timestamp) {
        let delta_ms = node_ts.saturating_sub(parent_ts);
        if delta_ms >= 500 {
            spans.push(Span::styled(
                format!(" +{:.1}s", delta_ms as f64 / 1000.0),
                Style::default().fg(Color::DarkGray),
            ));
        }
    }

    // Token badge if this node has usage data
    if let Some(tu) = node.node.effective_token_usage() {
        let total = tu.total();
        if total > 0 {
            spans.push(Span::styled(
                format!(
                    "  {}↑ {}↓",
                    fmt_compact(tu.total_input()),
                    fmt_compact(tu.total_output())
                ),
                Style::default().fg(Color::DarkGray),
            ));
        }
    }

    // Model badge for assistant messages that carry a model field
    if let Some(model_name) = node.node.message.as_ref().and_then(|m| m.model_short()) {
        spans.push(Span::styled(
            format!(" [{}]", model_name),
            Style::default().fg(Color::DarkGray),
        ));
    }

    let styled_line = Line::from(spans);

    // Safety: identifiers are UUIDs (or unique "no-uuid-{type}" strings), so
    // duplicate siblings within the same parent cannot occur.
    Some(TreeItem::new(identifier, styled_line, children).expect("duplicate tree identifier"))
}
