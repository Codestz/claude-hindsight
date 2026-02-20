//! Search route: GET /api/search

use axum::{
    extract::{Query, State},
    Json,
};
use serde::Deserialize;
use crate::server::{AppState, error::ApiError};
use crate::server::dto::SessionFileDto;
use crate::storage::SessionIndex;

#[derive(Deserialize)]
pub struct SearchQuery {
    #[serde(default)]
    pub q: String,
    pub project: Option<String>,
    pub tool: Option<String>,
    #[serde(default)]
    pub errors: bool,
}

pub async fn search_sessions(
    State(_state): State<AppState>,
    Query(q): Query<SearchQuery>,
) -> Result<Json<Vec<SessionFileDto>>, ApiError> {
    let result = tokio::task::spawn_blocking(move || {
        let index = SessionIndex::new()?;
        index.search_sessions(
            &q.q,
            q.project.as_deref(),
            q.errors,
            q.tool.as_deref(),
        )
    })
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))??;

    Ok(Json(result.into_iter().map(SessionFileDto::from).collect()))
}
