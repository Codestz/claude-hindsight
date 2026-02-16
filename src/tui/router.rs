//! Navigation router for TUI views
//!
//! Manages navigation between projects, sessions, and session detail views.

use crate::config::Config;
use crate::error::Result;
use crate::parser::parse_session;
use crate::storage::SessionIndex;
use crate::tui::app::App;
use crate::tui::projects_view::{ProjectAction, ProjectsView};
use crate::tui::sessions_view::{SessionAction, SessionsView};
use crossterm::event::KeyEvent;
use ratatui::Frame;

/// Current view mode
#[derive(Debug, Clone)]
pub enum ViewMode {
    Projects,
    Sessions(String),      // Project name
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

    /// Sessions view
    pub sessions_view: Option<SessionsView>,

    /// Session detail view
    pub session_detail_view: Option<App>,

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

        Ok(Router {
            view_mode: ViewMode::Projects,
            view_stack: vec![],
            projects_view,
            sessions_view: None,
            session_detail_view: None,
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

        Ok(Router {
            view_mode: ViewMode::SessionDetail(session_id),
            view_stack: vec![],
            projects_view: None,
            sessions_view: None,
            session_detail_view,
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
        match &self.view_mode {
            ViewMode::Projects => {
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
                let (should_go_back, should_quit) = if let Some(ref mut view) = self.session_detail_view {
                    view.handle_key(key)?;
                    (view.should_quit && !self.view_stack.is_empty(), view.should_quit && self.view_stack.is_empty())
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
                ViewMode::Sessions(project_name) => {
                    if self.sessions_view.is_none() {
                        self.sessions_view = Some(SessionsView::new(project_name.clone(), &self.config)?);
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

    /// Render the current view
    pub fn render(&mut self, f: &mut Frame) {
        match &self.view_mode {
            ViewMode::Projects => {
                if let Some(ref mut view) = self.projects_view {
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
    }
}
