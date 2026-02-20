//! Session routes

use crate::api::responses::{NodeResponse, TreeResponse};
use crate::server::dto::SessionFileDto;
use crate::server::{error::ApiError, AppState};
use crate::storage::SessionIndex;
use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct SessionsQuery {
    pub project: Option<String>,
    pub limit: Option<usize>,
}

pub async fn list_sessions(
    State(_state): State<AppState>,
    Query(q): Query<SessionsQuery>,
) -> Result<Json<Vec<SessionFileDto>>, ApiError> {
    let result = tokio::task::spawn_blocking(move || -> crate::error::Result<_> {
        let index = SessionIndex::new()?;
        let sessions = if let Some(ref project) = q.project {
            index.find_by_project(project)?
        } else {
            index.list_sessions()?
        };
        Ok(sessions)
    })
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))??;

    let limit = q.limit.unwrap_or(usize::MAX);
    let dtos: Vec<SessionFileDto> = result
        .into_iter()
        .take(limit)
        .map(SessionFileDto::from)
        .collect();

    Ok(Json(dtos))
}

pub async fn get_session(
    State(_state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<SessionFileDto>, ApiError> {
    let result = tokio::task::spawn_blocking(move || -> crate::error::Result<_> {
        let index = SessionIndex::new()?;
        index.find_by_id(&id)
    })
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))??;

    match result {
        Some(session) => Ok(Json(SessionFileDto::from(session))),
        None => Err(ApiError::NotFound("Session not found".to_string())),
    }
}

pub async fn get_session_nodes(
    State(_state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<TreeResponse>, ApiError> {
    let result = tokio::task::spawn_blocking(move || -> crate::error::Result<_> {
        let index = SessionIndex::new()?;
        let session = index
            .find_by_id(&id)?
            .ok_or_else(|| crate::error::HindsightError::SessionNotFound(id.clone()))?;

        let parsed = crate::parser::parse_session(&session.path)?;
        let roots = crate::analyzer::build_simple_tree(parsed.nodes);

        let roots_response: Vec<NodeResponse> =
            roots.iter().map(NodeResponse::from_tree_node).collect();

        let total_nodes = count_nodes(&roots_response);
        let max_depth = max_depth(&roots_response, 0);

        Ok(TreeResponse {
            roots: roots_response,
            total_nodes,
            max_depth,
        })
    })
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))??;

    Ok(Json(result))
}

fn count_nodes(nodes: &[NodeResponse]) -> usize {
    nodes
        .iter()
        .fold(0, |acc, n| acc + 1 + count_nodes(&n.children))
}

fn max_depth(nodes: &[NodeResponse], current: usize) -> usize {
    nodes.iter().fold(current, |max, n| {
        let child_max = max_depth(&n.children, current + 1);
        max.max(child_max)
    })
}
