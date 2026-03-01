//! Hook event API routes

use crate::server::{error::ApiError, AppState};
use crate::storage::{
    HookCompactionEvent, HookLifecycleEvent, HookPermissionEvent, HookSubagentEvent,
    HookToolEvent,
};
use axum::{
    extract::{Query, State},
    Json,
};
use serde::{Deserialize, Serialize};

use super::with_index;

#[derive(Deserialize)]
pub struct SessionEventQuery {
    pub session_id: Option<String>,
    pub event: Option<String>,
    pub limit: Option<usize>,
}

pub async fn get_tool_events(
    State(_): State<AppState>,
    Query(q): Query<SessionEventQuery>,
) -> Result<Json<Vec<HookToolEvent>>, ApiError> {
    let limit = q.limit.unwrap_or(500);
    with_index(move |idx| match q.session_id {
        Some(sid) => idx.get_tool_events(&sid, q.event.as_deref()),
        None => idx.get_global_tool_events(q.event.as_deref(), limit),
    })
    .await
}

pub async fn get_tool_failures(
    State(_): State<AppState>,
    Query(q): Query<SessionEventQuery>,
) -> Result<Json<Vec<HookToolEvent>>, ApiError> {
    let limit = q.limit.unwrap_or(100);
    with_index(move |idx| match q.session_id {
        Some(sid) => idx.get_tool_failures(&sid),
        None => idx.get_global_tool_failures(limit),
    })
    .await
}

pub async fn get_subagent_events(
    State(_): State<AppState>,
    Query(q): Query<SessionEventQuery>,
) -> Result<Json<Vec<HookSubagentEvent>>, ApiError> {
    let limit = q.limit.unwrap_or(500);
    with_index(move |idx| match q.session_id {
        Some(sid) => idx.get_subagent_events(&sid),
        None => idx.get_global_subagent_events(limit),
    })
    .await
}

pub async fn get_compaction_events(
    State(_): State<AppState>,
    Query(q): Query<SessionEventQuery>,
) -> Result<Json<Vec<HookCompactionEvent>>, ApiError> {
    let limit = q.limit.unwrap_or(500);
    with_index(move |idx| match q.session_id {
        Some(sid) => idx.get_compaction_events(&sid),
        None => idx.get_global_compaction_events(limit),
    })
    .await
}

pub async fn get_permission_events(
    State(_): State<AppState>,
    Query(q): Query<SessionEventQuery>,
) -> Result<Json<Vec<HookPermissionEvent>>, ApiError> {
    let limit = q.limit.unwrap_or(500);
    with_index(move |idx| match q.session_id {
        Some(sid) => idx.get_permission_events(&sid),
        None => idx.get_global_permission_events(limit),
    })
    .await
}

pub async fn get_lifecycle_events(
    State(_): State<AppState>,
    Query(q): Query<SessionEventQuery>,
) -> Result<Json<Vec<HookLifecycleEvent>>, ApiError> {
    let limit = q.limit.unwrap_or(500);
    with_index(move |idx| match q.session_id {
        Some(sid) => idx.get_lifecycle_events(&sid, q.event.as_deref()),
        None => idx.get_global_lifecycle_events(q.event.as_deref(), limit),
    })
    .await
}

/// Activity summary across all sessions
#[derive(Serialize)]
pub struct HookActivitySummary {
    pub total_tool_events: usize,
    pub total_subagent_events: usize,
    pub total_lifecycle_events: usize,
    pub total_permission_events: usize,
    pub tool_event_counts: Vec<(String, usize)>,
    pub recent_errors: usize,
}

pub async fn get_activity_summary(
    State(_): State<AppState>,
) -> Result<Json<HookActivitySummary>, ApiError> {
    with_index(|idx| idx.get_activity_summary()).await
}
