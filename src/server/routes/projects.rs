//! Project routes: GET /api/projects

use crate::server::dto::ProjectStatsDto;
use crate::server::{error::ApiError, AppState};
use axum::{extract::State, Json};

use super::with_index;

pub async fn list_projects(
    State(_state): State<AppState>,
) -> Result<Json<Vec<ProjectStatsDto>>, ApiError> {
    with_index(|index| {
        index
            .get_all_project_stats()
            .map(|v| v.into_iter().map(ProjectStatsDto::from).collect())
    })
    .await
}
