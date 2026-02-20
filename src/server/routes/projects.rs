//! Project routes: GET /api/projects

use crate::server::dto::ProjectStatsDto;
use crate::server::{error::ApiError, AppState};
use crate::storage::SessionIndex;
use axum::{extract::State, Json};

pub async fn list_projects(
    State(_state): State<AppState>,
) -> Result<Json<Vec<ProjectStatsDto>>, ApiError> {
    let result = tokio::task::spawn_blocking(move || {
        let index = SessionIndex::new()?;
        index.get_all_project_stats()
    })
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))??;

    Ok(Json(
        result.into_iter().map(ProjectStatsDto::from).collect(),
    ))
}
