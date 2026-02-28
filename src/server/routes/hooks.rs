//! Hook event API routes
//!
//! GET /api/hooks/tool-events?session_id=&event=
//! GET /api/hooks/tool-failures?session_id=
//! GET /api/hooks/subagent-events?session_id=
//! GET /api/hooks/compaction-events?session_id=
//! GET /api/hooks/permission-events?session_id=
//! GET /api/hooks/lifecycle-events?session_id=&event=

use crate::server::{error::ApiError, AppState};
use crate::storage::{
    HookCompactionEvent, HookLifecycleEvent, HookPermissionEvent, HookSubagentEvent,
    HookToolEvent, SessionIndex,
};
use axum::{
    extract::{Query, State},
    Json,
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct SessionEventQuery {
    pub session_id: String,
    pub event: Option<String>,
}

pub async fn get_tool_events(
    State(_): State<AppState>,
    Query(q): Query<SessionEventQuery>,
) -> Result<Json<Vec<HookToolEvent>>, ApiError> {
    let session_id = q.session_id.clone();
    let event = q.event.clone();
    let rows = tokio::task::spawn_blocking(move || -> crate::error::Result<Vec<HookToolEvent>> {
        let idx = SessionIndex::new()?;
        idx.get_tool_events(&session_id, event.as_deref())
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
    let rows = tokio::task::spawn_blocking(move || -> crate::error::Result<Vec<HookToolEvent>> {
        let idx = SessionIndex::new()?;
        idx.get_tool_failures(&session_id)
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
    let rows =
        tokio::task::spawn_blocking(move || -> crate::error::Result<Vec<HookSubagentEvent>> {
            let idx = SessionIndex::new()?;
            idx.get_subagent_events(&session_id)
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
    let rows =
        tokio::task::spawn_blocking(move || -> crate::error::Result<Vec<HookCompactionEvent>> {
            let idx = SessionIndex::new()?;
            idx.get_compaction_events(&session_id)
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
    let rows =
        tokio::task::spawn_blocking(move || -> crate::error::Result<Vec<HookPermissionEvent>> {
            let idx = SessionIndex::new()?;
            idx.get_permission_events(&session_id)
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
    let rows =
        tokio::task::spawn_blocking(move || -> crate::error::Result<Vec<HookLifecycleEvent>> {
            let idx = SessionIndex::new()?;
            idx.get_lifecycle_events(&session_id, event.as_deref())
        })
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))??;
    Ok(Json(rows))
}
