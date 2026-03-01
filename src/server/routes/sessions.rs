//! Session routes

use crate::api::responses::{NodeResponse, NodeResponseContext, TreeResponse};
use crate::server::dto::SessionFileDto;
use crate::server::{error::ApiError, AppState};
use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;

use super::with_index;

#[derive(Deserialize)]
pub struct SessionsQuery {
    pub project: Option<String>,
    pub limit: Option<usize>,
}

pub async fn list_sessions(
    State(_state): State<AppState>,
    Query(q): Query<SessionsQuery>,
) -> Result<Json<Vec<SessionFileDto>>, ApiError> {
    let limit = q.limit.unwrap_or(usize::MAX);
    with_index(move |index| {
        let sessions = if let Some(ref project) = q.project {
            index.find_by_project(project)?
        } else {
            index.list_sessions()?
        };
        Ok(sessions
            .into_iter()
            .take(limit)
            .map(SessionFileDto::from)
            .collect())
    })
    .await
}

pub async fn get_session(
    State(_state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<SessionFileDto>, ApiError> {
    let result = tokio::task::spawn_blocking(move || {
        let index = crate::storage::SessionIndex::new()?;
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
    with_index(move |index| {
        let session = index
            .find_by_id(&id)?
            .ok_or_else(|| crate::error::HindsightError::SessionNotFound(id.clone()))?;

        let parsed = crate::parser::parse_session(&session.path)?;
        let roots = crate::analyzer::build_simple_tree(parsed.nodes);

        let mut ctx = NodeResponseContext::new();
        let roots_response: Vec<NodeResponse> = roots
            .iter()
            .map(|r| NodeResponse::from_tree_node_with_context(r, &mut ctx))
            .collect();

        let total_nodes = count_nodes(&roots_response);
        let max_depth = max_depth(&roots_response, 0);

        Ok(TreeResponse {
            roots: roots_response,
            total_nodes,
            max_depth,
        })
    })
    .await
}

/// GET /api/sessions/:id/prompts -- returns nodes with prompt_score >= 40
pub async fn get_session_prompts(
    State(_state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<NodeResponse>>, ApiError> {
    with_index(move |index| {
        let session = index
            .find_by_id(&id)?
            .ok_or_else(|| crate::error::HindsightError::SessionNotFound(id.clone()))?;

        let parsed = crate::parser::parse_session(&session.path)?;
        let roots = crate::analyzer::build_simple_tree(parsed.nodes);

        let mut ctx = NodeResponseContext::new();
        let roots_response: Vec<NodeResponse> = roots
            .iter()
            .map(|r| NodeResponse::from_tree_node_with_context(r, &mut ctx))
            .collect();

        let threshold = crate::analyzer::prompt_detect::PROMPT_THRESHOLD;
        let mut prompts = Vec::new();
        collect_prompts(&roots_response, threshold, &mut prompts);

        Ok(prompts)
    })
    .await
}

fn collect_prompts(nodes: &[NodeResponse], threshold: u8, out: &mut Vec<NodeResponse>) {
    for node in nodes {
        if node.prompt_score.unwrap_or(0) >= threshold {
            let mut flat = node.clone();
            flat.children = Vec::new();
            out.push(flat);
        }
        collect_prompts(&node.children, threshold, out);
    }
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
