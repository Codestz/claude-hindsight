//! Claude Hindsight - 20/20 hindsight for your Claude Code sessions
//!
//! A powerful observability tool for Claude Code that transforms raw JSONL
//! transcripts into beautiful, interactive visualizations.

use clap::{Parser, Subcommand};
use std::process;

mod analyzer;
mod parser;
mod commands;
mod error;
mod storage;
mod tui;

use error::Result;

#[derive(Parser)]
#[command(name = "hindsight")]
#[command(version, about, long_about = None)]
#[command(author = "Codestz <est.estrada@outlook.com>")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
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

    /// Show quick session statistics
    Stats {
        /// Session ID or partial ID
        session_id: String,
    },

    /// Analyze session costs
    Costs {
        /// Session ID or partial ID
        session_id: String,

        /// Show cost breakdown by tool
        #[arg(long)]
        by_tool: bool,

        /// Show cost breakdown by time period
        #[arg(long)]
        by_time: bool,
    },

    /// Debug session errors
    Errors {
        /// Session ID or partial ID
        session_id: String,
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

    /// Compare two sessions
    Compare {
        /// First session ID
        session_a: String,

        /// Second session ID
        session_b: String,
    },

    /// Export session to HTML report
    Export {
        /// Session ID
        session_id: String,

        /// Output file path
        #[arg(short, long, default_value = "report.html")]
        output: String,
    },
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
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
        Commands::Stats { session_id } => {
            commands::stats::run(session_id)?;
        }
        Commands::Costs {
            session_id,
            by_tool,
            by_time,
        } => {
            commands::costs::run(session_id, by_tool, by_time)?;
        }
        Commands::Errors { session_id } => {
            commands::errors::run(session_id)?;
        }
        Commands::Search {
            query,
            tool,
            errors,
        } => {
            commands::search::run(query, tool, errors)?;
        }
        Commands::Compare {
            session_a,
            session_b,
        } => {
            commands::compare::run(session_a, session_b)?;
        }
        Commands::Export {
            session_id,
            output,
        } => {
            commands::export::run(session_id, output)?;
        }
    }

    Ok(())
}
