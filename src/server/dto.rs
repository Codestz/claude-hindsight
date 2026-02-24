//! DTO (Data Transfer Object) layer for the web API
//!
//! Adds Serialize to existing storage structs via wrapper types with From impls.

use crate::storage::{GlobalAnalytics, ProjectAnalytics, ProjectStats, SessionFile};
use serde::{Deserialize, Serialize};

/// Serializable project stats
#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectStatsDto {
    pub project_name: String,
    pub session_count: usize,
    pub total_size: u64,
    pub last_activity: Option<i64>,
}

impl From<ProjectStats> for ProjectStatsDto {
    fn from(s: ProjectStats) -> Self {
        Self {
            project_name: s.project_name,
            session_count: s.session_count,
            total_size: s.total_size,
            last_activity: s.last_activity,
        }
    }
}

/// Serializable global analytics
#[derive(Debug, Serialize, Deserialize)]
pub struct GlobalAnalyticsDto {
    pub total_sessions: usize,
    pub sessions_this_week: usize,
    pub sessions_today: usize,
    pub total_size: u64,
    pub total_projects: usize,
    pub subagent_count: usize,
    pub avg_session_size: u64,
    pub most_active_project: Option<String>,
    pub top_tools: Vec<(String, usize)>,
    pub total_errors: usize,
}

impl From<GlobalAnalytics> for GlobalAnalyticsDto {
    fn from(a: GlobalAnalytics) -> Self {
        Self {
            total_sessions: a.total_sessions,
            sessions_this_week: a.sessions_this_week,
            sessions_today: a.sessions_today,
            total_size: a.total_size,
            total_projects: a.total_projects,
            subagent_count: a.subagent_count,
            avg_session_size: a.avg_session_size,
            most_active_project: a.most_active_project,
            top_tools: a.top_tools,
            total_errors: a.total_errors,
        }
    }
}

/// Serializable project analytics
#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectAnalyticsDto {
    pub project_name: String,
    pub total_sessions: usize,
    pub sessions_this_week: usize,
    pub sessions_today: usize,
    pub total_size: u64,
    pub subagent_count: usize,
    pub avg_session_size: u64,
    pub top_tools: Vec<(String, usize)>,
    pub last_activity: Option<i64>,
    pub total_errors: usize,
}

impl From<ProjectAnalytics> for ProjectAnalyticsDto {
    fn from(a: ProjectAnalytics) -> Self {
        Self {
            project_name: a.project_name,
            total_sessions: a.total_sessions,
            sessions_this_week: a.sessions_this_week,
            sessions_today: a.sessions_today,
            total_size: a.total_size,
            subagent_count: a.subagent_count,
            avg_session_size: a.avg_session_size,
            top_tools: a.top_tools,
            last_activity: a.last_activity,
            total_errors: a.total_errors,
        }
    }
}

/// Serializable session file (path excluded — internal detail)
#[derive(Debug, Serialize, Deserialize)]
pub struct SessionFileDto {
    pub session_id: String,
    pub project_name: String,
    pub file_size: u64,
    pub created_at: i64,
    pub modified_at: i64,
    pub has_subagents: bool,
    pub model: Option<String>,
    pub error_count: usize,
    pub first_message: Option<String>,
    pub source_dir: String,
    pub subagent_models: Option<Vec<String>>,
}

/// Cross-session prompt entry
#[derive(Debug, Serialize, Deserialize)]
pub struct PromptEntryDto {
    pub session_id: String,
    pub project_name: String,
    pub prompt_text: String,
    pub prompt_score: u8,
    pub timestamp: Option<i64>,
    pub model: Option<String>,
}

impl From<SessionFile> for SessionFileDto {
    fn from(s: SessionFile) -> Self {
        let subagent_models = s.subagent_models.map(|m| {
            m.split(',').map(str::to_string).collect()
        });
        Self {
            session_id: s.session_id,
            project_name: s.project_name,
            file_size: s.file_size,
            created_at: s.created_at,
            modified_at: s.modified_at,
            has_subagents: s.has_subagents,
            model: s.model,
            error_count: s.error_count,
            first_message: s.first_message,
            source_dir: s.source_dir,
            subagent_models,
        }
    }
}
