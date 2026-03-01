//! Hook event API routes
//!
//! GET /api/hooks/tool-events?session_id=&event=&limit=
//! GET /api/hooks/tool-failures?session_id=&limit=
//! GET /api/hooks/subagent-events?session_id=&limit=
//! GET /api/hooks/compaction-events?session_id=&limit=
//! GET /api/hooks/permission-events?session_id=&limit=
//! GET /api/hooks/lifecycle-events?session_id=&event=&limit=
//! GET /api/hooks/activity-summary

use crate::server::{error::ApiError, AppState};
use crate::storage::{
    HookCompactionEvent, HookLifecycleEvent, HookPermissionEvent, HookSubagentEvent,
    HookToolEvent, SessionIndex,
};
use axum::{
    extract::{Query, State},
    Json,
};
use serde::{Deserialize, Serialize};

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
    let session_id = q.session_id.clone();
    let event = q.event.clone();
    let limit = q.limit.unwrap_or(500);
    let rows = tokio::task::spawn_blocking(move || -> crate::error::Result<Vec<HookToolEvent>> {
        let idx = SessionIndex::new()?;
        match session_id {
            Some(sid) => idx.get_tool_events(&sid, event.as_deref()),
            None => idx.get_global_tool_events(event.as_deref(), limit),
        }
    })
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))??;
    Ok(Json(rows))
}

pub async fn get_tool_failures(
    State(_): State<AppState>,
    Query(q): Query<SessionEventQuery>,
) -> Result<Json<Vec<HookToolEvent>>, ApiError> {
    let session_id = q.session_id.clone();
    let limit = q.limit.unwrap_or(100);
    let rows = tokio::task::spawn_blocking(move || -> crate::error::Result<Vec<HookToolEvent>> {
        let idx = SessionIndex::new()?;
        match session_id {
            Some(sid) => idx.get_tool_failures(&sid),
            None => idx.get_global_tool_failures(limit),
        }
    })
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))??;
    Ok(Json(rows))
}

pub async fn get_subagent_events(
    State(_): State<AppState>,
    Query(q): Query<SessionEventQuery>,
) -> Result<Json<Vec<HookSubagentEvent>>, ApiError> {
    let session_id = q.session_id.clone();
    let limit = q.limit.unwrap_or(500);
    let rows =
        tokio::task::spawn_blocking(move || -> crate::error::Result<Vec<HookSubagentEvent>> {
            let idx = SessionIndex::new()?;
            match session_id {
                Some(sid) => idx.get_subagent_events(&sid),
                None => idx.get_global_subagent_events(limit),
            }
        })
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))??;
    Ok(Json(rows))
}

pub async fn get_compaction_events(
    State(_): State<AppState>,
    Query(q): Query<SessionEventQuery>,
) -> Result<Json<Vec<HookCompactionEvent>>, ApiError> {
    let session_id = q.session_id.clone();
    let limit = q.limit.unwrap_or(500);
    let rows =
        tokio::task::spawn_blocking(move || -> crate::error::Result<Vec<HookCompactionEvent>> {
            let idx = SessionIndex::new()?;
            match session_id {
                Some(sid) => idx.get_compaction_events(&sid),
                None => idx.get_global_compaction_events(limit),
            }
        })
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))??;
    Ok(Json(rows))
}

pub async fn get_permission_events(
    State(_): State<AppState>,
    Query(q): Query<SessionEventQuery>,
) -> Result<Json<Vec<HookPermissionEvent>>, ApiError> {
    let session_id = q.session_id.clone();
    let limit = q.limit.unwrap_or(500);
    let rows =
        tokio::task::spawn_blocking(move || -> crate::error::Result<Vec<HookPermissionEvent>> {
            let idx = SessionIndex::new()?;
            match session_id {
                Some(sid) => idx.get_permission_events(&sid),
                None => idx.get_global_permission_events(limit),
            }
        })
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))??;
    Ok(Json(rows))
}

pub async fn get_lifecycle_events(
    State(_): State<AppState>,
    Query(q): Query<SessionEventQuery>,
) -> Result<Json<Vec<HookLifecycleEvent>>, ApiError> {
    let session_id = q.session_id.clone();
    let event = q.event.clone();
    let limit = q.limit.unwrap_or(500);
    let rows =
        tokio::task::spawn_blocking(move || -> crate::error::Result<Vec<HookLifecycleEvent>> {
            let idx = SessionIndex::new()?;
            match session_id {
                Some(sid) => idx.get_lifecycle_events(&sid, event.as_deref()),
                None => idx.get_global_lifecycle_events(event.as_deref(), limit),
            }
        })
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))??;
    Ok(Json(rows))
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
    let summary = tokio::task::spawn_blocking(move || -> crate::error::Result<HookActivitySummary> {
        let idx = SessionIndex::new()?;
        idx.get_activity_summary()
    })
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))??;
    Ok(Json(summary))
}
