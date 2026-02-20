//! Claude Hindsight - 20/20 hindsight for your Claude Code sessions
//!
//! A powerful observability tool for Claude Code that transforms raw JSONL
//! transcripts into beautiful, interactive visualizations.

use clap::{Parser, Subcommand};
use std::process;

mod analyzer;
mod api;
mod commands;
mod config;
mod error;
mod parser;
mod search;
mod server;
mod storage;
mod tui;
mod watcher;

use error::Result;

#[derive(Parser)]
#[command(name = "hindsight")]
#[command(version, about, long_about = None)]
#[command(author = "Codestz <est.estrada@outlook.com>")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize Claude Hindsight (discover Claude Code sessions)
    Init {
        /// Enable OpenTelemetry integration (optional)
        #[arg(long)]
        enable_otel: bool,
    },

    /// List all Claude Code sessions
    List {
        /// Filter by project name
        #[arg(short, long)]
        project: Option<String>,

        /// Show only sessions with errors
        #[arg(long)]
        errors: bool,

        /// Show last N sessions
        #[arg(short, long)]
        last: Option<usize>,

        /// Show today's sessions only
        #[arg(long)]
        today: bool,

        /// Show sessions with subagents
        #[arg(long)]
        with_subagents: bool,
    },

    /// Watch current session in real-time
    Watch {
        /// Open dashboard in browser
        #[arg(short, long)]
        dashboard: bool,

        /// Session ID (auto-detects current if not provided)
        session_id: Option<String>,
    },

    /// Analyze a session (interactive TUI)
    Show {
        /// Session ID or partial ID
        session_id: String,

        /// Open web dashboard instead of terminal UI
        #[arg(short, long)]
        dashboard: bool,

        /// Port for web dashboard
        #[arg(short, long, default_value = "3939")]
        port: u16,
    },

    /// Search sessions
    Search {
        /// Search query
        query: String,

        /// Filter by tool name
        #[arg(long)]
        tool: Vec<String>,

        /// Show only sessions with errors
        #[arg(long)]
        errors: bool,
    },

    /// Export session to HTML report
    Export {
        /// Session ID
        session_id: String,

        /// Output file path
        #[arg(short, long, default_value = "report.html")]
        output: String,
    },

    /// Manage configuration
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    /// Manage Claude project directories to scan
    Paths {
        #[command(subcommand)]
        action: PathsAction,
    },

    /// Sync index with disk: remove deleted sessions, add new ones, refresh analytics
    Reindex {
        /// Show each session being processed
        #[arg(short, long)]
        verbose: bool,
    },

    /// Start the web dashboard server
    Serve {
        /// Port to listen on
        #[arg(short, long, default_value = "7227")]
        port: u16,

        /// Open browser after starting
        #[arg(long)]
        open: bool,
    },
}

#[derive(Subcommand)]
enum PathsAction {
    /// List all configured scan directories
    List,

    /// Add a directory to scan for Claude sessions
    Add {
        /// Path to add (e.g. ~/.claudep/projects)
        path: String,
    },

    /// Remove a directory (the default ~/.claude/projects cannot be removed)
    Remove {
        /// Path to remove
        path: String,
    },
}

#[derive(Subcommand)]
enum ConfigAction {
    /// Show current configuration
    Show,

    /// Edit configuration in $EDITOR
    Edit,

    /// Reset configuration to defaults
    Reset,

    /// Validate configuration file
    Validate,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    // If no command provided, check config for default view
    let command = match cli.command {
        Some(cmd) => cmd,
        None => {
            // Load config to determine default view
            let config = config::Config::load()?;

            match config.ui.default_view.as_str() {
                "last_session" => {
                    // Find and open the most recent session
                    return run_last_session();
                }
                _ => {
                    // Default to projects hub
                    return run_hub();
                }
            }
        }
    };

    match command {
        Commands::Init { enable_otel } => {
            commands::init::run(enable_otel)?;
        }
        Commands::List {
            project,
            errors,
            last,
            today,
            with_subagents,
        } => {
            commands::list::run(project, errors, last, today, with_subagents)?;
        }
        Commands::Watch {
            dashboard,
            session_id,
        } => {
            commands::watch::run(session_id, dashboard)?;
        }
        Commands::Show {
            session_id,
            dashboard,
            port,
        } => {
            commands::show::run(session_id, dashboard, port)?;
        }
        Commands::Search {
            query,
            tool,
            errors,
        } => {
            commands::search::run(query, tool, errors)?;
        }
        Commands::Export { session_id, output } => {
            commands::export::run(session_id, output)?;
        }
        Commands::Paths { action } => match action {
            PathsAction::List => commands::paths::list()?,
            PathsAction::Add { path } => commands::paths::add(path)?,
            PathsAction::Remove { path } => commands::paths::remove(path)?,
        },
        Commands::Config { action } => match action {
            ConfigAction::Show => commands::config::show_config()?,
            ConfigAction::Edit => commands::config::edit_config()?,
            ConfigAction::Reset => commands::config::reset_config()?,
            ConfigAction::Validate => commands::config::validate_config()?,
        },
        Commands::Reindex { verbose } => {
            commands::reindex::run(verbose)?;
        }
        Commands::Serve { port, open } => {
            commands::serve::run(port, open)?;
        }
    }

    Ok(())
}

/// Run the main TUI hub
fn run_hub() -> Result<()> {
    use tui::router::Router;
    use tui::EventHandler;

    // Initialize router
    let mut router = Router::new()?;

    // Initialize terminal
    let mut terminal = tui::init()?;
    let mut event_handler = EventHandler::new(250);

    // Main loop
    loop {
        terminal.draw(|f| router.render(f))?;

        if let tui::Event::Key(key) = event_handler.poll()? {
            router.handle_key(key)?;
        }

        // Process periodic updates (debounced search, etc.)
        router.tick();

        if router.should_quit {
            break;
        }
    }

    // Restore terminal
    tui::restore()?;
    Ok(())
}

/// Run the TUI opening the most recent session
fn run_last_session() -> Result<()> {
    use storage::SessionIndex;
    use tui::router::Router;
    use tui::EventHandler;

    // Get the most recent session
    let index = SessionIndex::new()?;
    let latest = index.get_latest()?;

    match latest {
        Some(session_file) => {
            // Create router with the session
            let mut router = Router::new_with_session(session_file.session_id)?;

            // Initialize terminal
            let mut terminal = tui::init()?;
            let mut event_handler = EventHandler::new(250);

            // Main loop
            loop {
                terminal.draw(|f| router.render(f))?;

                if let tui::Event::Key(key) = event_handler.poll()? {
                    router.handle_key(key)?;
                }

                // Process periodic updates (debounced search, etc.)
                router.tick();

                if router.should_quit {
                    break;
                }
            }

            // Restore terminal
            tui::restore()?;
            Ok(())
        }
        None => {
            eprintln!("No sessions found. Run 'hindsight init' to discover sessions.");
            eprintln!("Falling back to projects view...");
            run_hub()
        }
    }
}

