//! Search result types and ranking
#![allow(dead_code)]

use crate::storage::SessionFile;
use chrono::{Local, TimeZone};

/// A search result representing a matching session
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// Session ID
    pub session_id: String,

    /// Project name
    pub project_name: String,

    /// File path
    pub file_path: String,

    /// File size in bytes
    pub file_size: u64,

    /// Last modified timestamp (seconds since epoch)
    pub modified_at: i64,

    /// Whether session has subagents
    pub has_subagents: bool,

    /// Relevance score (higher = more relevant)
    /// For Phase 1, this is based on recency
    pub relevance: f64,
}

impl SearchResult {
    /// Create a search result from a session file
    pub fn from_session_file(session: SessionFile) -> Self {
        // Calculate relevance based on recency
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        let age_seconds = (now - session.modified_at).max(0) as f64;
        let age_days = age_seconds / 86400.0;

        // Exponential decay: score = 100 * e^(-age_days / 7)
        // Sessions from today: ~100, 1 week ago: ~37, 1 month ago: ~2
        let relevance = 100.0 * (-age_days / 7.0).exp();

        SearchResult {
            session_id: session.session_id,
            project_name: session.project_name,
            file_path: session.path.to_string_lossy().to_string(),
            file_size: session.file_size,
            modified_at: session.modified_at,
            has_subagents: session.has_subagents,
            relevance,
        }
    }

    /// Format the modified_at timestamp as a human-readable string
    pub fn format_time(&self) -> String {
        let dt = Local.timestamp_opt(self.modified_at, 0);
        match dt.single() {
            Some(datetime) => {
                let now = Local::now();
                let duration = now.signed_duration_since(datetime);

                if duration.num_days() == 0 {
                    if duration.num_hours() == 0 {
                        if duration.num_minutes() == 0 {
                            "just now".to_string()
                        } else {
                            format!("{}m ago", duration.num_minutes())
                        }
                    } else {
                        format!("{}h ago", duration.num_hours())
                    }
                } else if duration.num_days() == 1 {
                    "1d ago".to_string()
                } else if duration.num_days() < 7 {
                    format!("{}d ago", duration.num_days())
                } else if duration.num_days() < 30 {
                    format!("{}w ago", duration.num_days() / 7)
                } else {
                    datetime.format("%Y-%m-%d").to_string()
                }
            }
            None => "unknown".to_string(),
        }
    }

    /// Format file size as human-readable string
    pub fn format_size(&self) -> String {
        let size = self.file_size as f64;

        if size < 1024.0 {
            format!("{} B", size)
        } else if size < 1024.0 * 1024.0 {
            format!("{:.1} KB", size / 1024.0)
        } else if size < 1024.0 * 1024.0 * 1024.0 {
            format!("{:.1} MB", size / (1024.0 * 1024.0))
        } else {
            format!("{:.1} GB", size / (1024.0 * 1024.0 * 1024.0))
        }
    }
}
