//! Analytics routes

use crate::server::dto::{GlobalAnalyticsDto, ProjectAnalyticsDto};
use crate::server::{error::ApiError, AppState};
use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;

use super::with_index;

pub async fn global_analytics(
    State(_state): State<AppState>,
) -> Result<Json<GlobalAnalyticsDto>, ApiError> {
    with_index(|index| index.get_global_analytics().map(GlobalAnalyticsDto::from)).await
}

#[derive(Deserialize)]
pub struct SparklineQuery {
    #[serde(default = "default_days")]
    pub days: usize,
}

fn default_days() -> usize {
    14
}

pub async fn global_sparkline(
    State(_state): State<AppState>,
    Query(q): Query<SparklineQuery>,
) -> Result<Json<Vec<u64>>, ApiError> {
    with_index(move |index| index.get_daily_session_counts(q.days)).await
}

pub async fn project_analytics(
    State(_state): State<AppState>,
    Path(project): Path<String>,
) -> Result<Json<ProjectAnalyticsDto>, ApiError> {
    with_index(move |index| index.get_project_analytics(&project).map(ProjectAnalyticsDto::from))
        .await
}

pub async fn global_top_files(
    State(_state): State<AppState>,
) -> Result<Json<Vec<(String, usize)>>, ApiError> {
    with_index(|index| index.get_top_files(12)).await
}

pub async fn project_top_files(
    State(_state): State<AppState>,
    Path(project): Path<String>,
) -> Result<Json<Vec<(String, usize)>>, ApiError> {
    with_index(move |index| index.get_top_files_for_project(&project, 12)).await
}
