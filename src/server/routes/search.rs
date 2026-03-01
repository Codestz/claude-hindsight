//! Search route: GET /api/search

use crate::server::dto::SessionFileDto;
use crate::server::{error::ApiError, AppState};
use axum::{
    extract::{Query, State},
    Json,
};
use serde::Deserialize;

use super::with_index;

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
    with_index(move |index| {
        index
            .search_sessions(&q.q, q.project.as_deref(), q.errors, q.tool.as_deref())
            .map(|v| v.into_iter().map(SessionFileDto::from).collect())
    })
    .await
}
