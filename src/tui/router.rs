//! Navigation router for TUI views
//!
//! Manages navigation between projects, sessions, and session detail views.

use crate::config::Config;
use crate::error::Result;
use crate::parser::parse_session;
use crate::storage::SessionIndex;
use crate::tui::app::App;
use crate::tui::dashboard_view::{DashboardAction, DashboardView};
use crate::tui::projects_view::{ProjectAction, ProjectsView};
use crate::tui::prompts_view::{PromptAction, PromptsView};
use crate::tui::search_modal::{SearchAction, SearchContext, SearchModal};
use crate::tui::sessions_view::{SessionAction, SessionsView};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;

/// Current view mode
#[derive(Debug, Clone)]
pub enum ViewMode {
    Projects,
    Dashboard,
    Prompts,
    Sessions(String), // Project name
    #[allow(dead_code)]
    SessionDetail(String), // Session ID
}

/// Main router application
pub struct Router {
    /// Current view mode
    pub view_mode: ViewMode,

    /// View stack for back navigation
    pub view_stack: Vec<ViewMode>,

    /// Projects view
    pub projects_view: Option<ProjectsView>,

    /// Dashboard view
    pub dashboard_view: Option<DashboardView>,

    /// Prompts view
    pub prompts_view: Option<PromptsView>,

    /// Sessions view
    pub sessions_view: Option<SessionsView>,

    /// Session detail view
    pub session_detail_view: Option<App>,

    /// Search modal (overlay)
    pub search_modal: SearchModal,

    /// Whether to quit
    pub should_quit: bool,

    /// Application configuration
    pub config: Config,
}

impl Router {
    /// Create a new router starting at projects view
    pub fn new() -> Result<Self> {
        let config = Config::load()?;

        // Validate config
        if let Err(e) = config.validate() {
            eprintln!("Warning: Invalid configuration: {}", e);
            eprintln!("Using default configuration.");
        }

        let projects_view = Some(ProjectsView::new(&config)?);
        let search_modal = SearchModal::new(SearchContext::Global);

        Ok(Router {
            view_mode: ViewMode::Projects,
            view_stack: vec![],
            projects_view,
            dashboard_view: None,
            prompts_view: None,
            sessions_view: None,
            session_detail_view: None,
            search_modal,
            should_quit: false,
            config,
        })
    }

    /// Create a router that goes directly to a session
    #[allow(dead_code)]
    pub fn new_with_session(session_id: String) -> Result<Self> {
        let index = SessionIndex::new()?;
        let session_file = index
            .find_by_id(&session_id)?
            .ok_or_else(|| crate::error::HindsightError::SessionNotFound(session_id.clone()))?;

        let session = parse_session(&session_file.path)?;
        let session_detail_view = Some(App::new(session));
        let config = Config::load()?;
        let search_modal = SearchModal::new(SearchContext::Session(session_id.clone()));

        Ok(Router {
            view_mode: ViewMode::SessionDetail(session_id),
            view_stack: vec![],
            projects_view: None,
            dashboard_view: None,
            prompts_view: None,
            sessions_view: None,
            session_detail_view,
            search_modal,
            should_quit: false,
            config,
        })
    }

    /// Handle keyboard input
    /// Process periodic updates (debounced search, etc.)
    pub fn tick(&mut self) {
        // Call tick on session detail view to handle debounced search
        if let ViewMode::SessionDetail(_) = &self.view_mode {
            if let Some(ref mut view) = self.session_detail_view {
                view.tick();
            }
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Result<()> {
        // If search modal is active, let it handle the key first
        if self.search_modal.is_active {
            return self.handle_search_key(key);
        }

        // Check for global '/' key to open search (like vim/less)
        if matches!(key.code, KeyCode::Char('/')) {
            self.activate_search();
            return Ok(());
        }

        match &self.view_mode {
            ViewMode::Projects => {
                // D key opens dashboard from projects view
                if key.code == KeyCode::Char('d') || key.code == KeyCode::Char('D') {
                    self.navigate_to_dashboard()?;
                    return Ok(());
                }
                // P key opens prompts view from projects view
                if key.code == KeyCode::Char('p') || key.code == KeyCode::Char('P') {
                    self.navigate_to_prompts()?;
                    return Ok(());
                }
                if let Some(ref mut view) = self.projects_view {
                    match view.handle_key(key)? {
                        ProjectAction::None => {}
                        ProjectAction::SelectProject(project_name) => {
                            self.navigate_to_sessions(project_name)?;
                        }
                        ProjectAction::Quit => {
                            self.should_quit = true;
                        }
                    }
                }
            }

            ViewMode::Dashboard => {
                if let Some(ref mut view) = self.dashboard_view {
                    match view.handle_key(key)? {
                        DashboardAction::None => {}
                        DashboardAction::Back => {
                            self.navigate_back()?;
                        }
                        DashboardAction::Quit => {
                            self.should_quit = true;
                        }
                    }
                }
            }

            ViewMode::Prompts => {
                if let Some(ref mut view) = self.prompts_view {
                    match view.handle_key(key)? {
                        PromptAction::None => {}
                        PromptAction::SelectSession(session_id) => {
                            self.navigate_to_session_detail(session_id)?;
                        }
                        PromptAction::Back => {
                            self.navigate_back()?;
                        }
                        PromptAction::Quit => {
                            self.should_quit = true;
                        }
                    }
                }
            }

            ViewMode::Sessions(_) => {
                if let Some(ref mut view) = self.sessions_view {
                    match view.handle_key(key)? {
                        SessionAction::None => {}
                        SessionAction::SelectSession(session_id) => {
                            self.navigate_to_session_detail(session_id)?;
                        }
                        SessionAction::Back => {
                            self.navigate_back()?;
                        }
                        SessionAction::Quit => {
                            self.should_quit = true;
                        }
                    }
                }
            }

            ViewMode::SessionDetail(_) => {
                let (should_go_back, should_quit) =
                    if let Some(ref mut view) = self.session_detail_view {
                        view.handle_key(key)?;
                        (
                            view.should_quit && !self.view_stack.is_empty(),
                            view.should_quit && self.view_stack.is_empty(),
                        )
                    } else {
                        (false, false)
                    };

                if should_go_back {
                    self.navigate_back()?;
                    // Reset quit flag
                    if let Some(ref mut view) = self.session_detail_view {
                        view.should_quit = false;
                    }
                } else if should_quit {
                    self.should_quit = true;
                }
            }
        }

        Ok(())
    }

    /// Navigate to dashboard
    fn navigate_to_dashboard(&mut self) -> Result<()> {
        self.view_stack.push(self.view_mode.clone());
        self.dashboard_view = Some(DashboardView::new()?);
        self.view_mode = ViewMode::Dashboard;
        Ok(())
    }

    /// Navigate to prompts view
    fn navigate_to_prompts(&mut self) -> Result<()> {
        self.view_stack.push(self.view_mode.clone());
        self.prompts_view = Some(PromptsView::new()?);
        self.view_mode = ViewMode::Prompts;
        Ok(())
    }

    /// Navigate to sessions view for a project
    fn navigate_to_sessions(&mut self, project_name: String) -> Result<()> {
        // Push current view to stack
        self.view_stack.push(self.view_mode.clone());

        // Create sessions view
        self.sessions_view = Some(SessionsView::new(project_name.clone(), &self.config)?);

        // Update view mode
        self.view_mode = ViewMode::Sessions(project_name);

        Ok(())
    }

    /// Navigate to session detail view
    fn navigate_to_session_detail(&mut self, session_id: String) -> Result<()> {
        // Push current view to stack
        self.view_stack.push(self.view_mode.clone());

        // Load session
        let index = SessionIndex::new()?;
        let session_file = index
            .find_by_id(&session_id)?
            .ok_or_else(|| crate::error::HindsightError::SessionNotFound(session_id.clone()))?;

        let session = parse_session(&session_file.path)?;

        // Create session detail view
        self.session_detail_view = Some(App::new(session));

        // Update view mode
        self.view_mode = ViewMode::SessionDetail(session_id);

        Ok(())
    }

    /// Navigate back to previous view
    fn navigate_back(&mut self) -> Result<()> {
        if let Some(prev_view) = self.view_stack.pop() {
            self.view_mode = prev_view;

            // Re-create the view if needed
            match &self.view_mode {
                ViewMode::Projects => {
                    if self.projects_view.is_none() {
                        self.projects_view = Some(ProjectsView::new(&self.config)?);
                    } else if let Some(ref mut view) = self.projects_view {
                        view.refresh()?;
                    }
                }
                ViewMode::Dashboard => {
                    if self.dashboard_view.is_none() {
                        self.dashboard_view = Some(DashboardView::new()?);
                    }
                }
                ViewMode::Prompts => {
                    if self.prompts_view.is_none() {
                        self.prompts_view = Some(PromptsView::new()?);
                    }
                }
                ViewMode::Sessions(project_name) => {
                    if self.sessions_view.is_none() {
                        self.sessions_view =
                            Some(SessionsView::new(project_name.clone(), &self.config)?);
                    } else if let Some(ref mut view) = self.sessions_view {
                        view.refresh()?;
                    }
                }
                ViewMode::SessionDetail(_) => {
                    // Session detail view should already exist
                }
            }
        }

        Ok(())
    }

    /// Activate search modal with appropriate context
    fn activate_search(&mut self) {
        let context = match &self.view_mode {
            ViewMode::Projects | ViewMode::Dashboard | ViewMode::Prompts => SearchContext::Global,
            ViewMode::Sessions(project) => SearchContext::Project(project.clone()),
            ViewMode::SessionDetail(session_id) => SearchContext::Session(session_id.clone()),
        };

        self.search_modal.context = context;
        self.search_modal.activate();
    }

    /// Handle key when search modal is active
    fn handle_search_key(&mut self, key: KeyEvent) -> Result<()> {
        match self.search_modal.handle_key(key)? {
            SearchAction::None => Ok(()),
            SearchAction::Cancel => Ok(()),
            SearchAction::SelectSession(session_id) => {
                // Navigate to the selected session
                self.navigate_to_session_detail(session_id)?;
                Ok(())
            }
            SearchAction::SelectNode(node_uuid) => {
                // Jump to the selected node in session detail view
                if let Some(ref mut view) = self.session_detail_view {
                    view.select_node_by_uuid(&node_uuid);
                }
                Ok(())
            }
        }
    }

    /// Render the current view
    pub fn render(&mut self, f: &mut Frame) {
        // Render the main view
        match &self.view_mode {
            ViewMode::Projects => {
                if let Some(ref mut view) = self.projects_view {
                    view.render(f, f.area());
                }
            }

            ViewMode::Dashboard => {
                if let Some(ref view) = self.dashboard_view {
                    view.render(f, f.area());
                }
            }

            ViewMode::Prompts => {
                if let Some(ref mut view) = self.prompts_view {
                    view.render(f, f.area());
                }
            }

            ViewMode::Sessions(_) => {
                if let Some(ref mut view) = self.sessions_view {
                    view.render(f, f.area());
                }
            }

            ViewMode::SessionDetail(_) => {
                if let Some(ref mut view) = self.session_detail_view {
                    crate::tui::ui::draw(f, view);
                }
            }
        }

        // Render search modal as overlay (if active)
        self.search_modal.render(f, f.area());
    }
}
