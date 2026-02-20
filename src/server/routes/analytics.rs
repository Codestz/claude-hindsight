//! Analytics routes

use crate::server::dto::{GlobalAnalyticsDto, ProjectAnalyticsDto};
use crate::server::{error::ApiError, AppState};
use crate::storage::SessionIndex;
use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;

pub async fn global_analytics(
    State(_state): State<AppState>,
) -> Result<Json<GlobalAnalyticsDto>, ApiError> {
    let result = tokio::task::spawn_blocking(move || {
        let index = SessionIndex::new()?;
        index.get_global_analytics()
    })
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))??;

    Ok(Json(GlobalAnalyticsDto::from(result)))
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
    let result = tokio::task::spawn_blocking(move || {
        let index = SessionIndex::new()?;
        index.get_daily_session_counts(q.days)
    })
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))??;

    Ok(Json(result))
}

pub async fn project_analytics(
    State(_state): State<AppState>,
    Path(project): Path<String>,
) -> Result<Json<ProjectAnalyticsDto>, ApiError> {
    let result = tokio::task::spawn_blocking(move || {
        let index = SessionIndex::new()?;
        index.get_project_analytics(&project)
    })
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))??;

    Ok(Json(ProjectAnalyticsDto::from(result)))
}

pub async fn global_top_files(
    State(_state): State<AppState>,
) -> Result<Json<Vec<(String, usize)>>, ApiError> {
    let result = tokio::task::spawn_blocking(move || {
        let index = SessionIndex::new()?;
        index.get_top_files(12)
    })
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))??;

    Ok(Json(result))
}

pub async fn project_top_files(
    State(_state): State<AppState>,
    Path(project): Path<String>,
) -> Result<Json<Vec<(String, usize)>>, ApiError> {
    let result = tokio::task::spawn_blocking(move || {
        let index = SessionIndex::new()?;
        index.get_top_files_for_project(&project, 12)
    })
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))??;

    Ok(Json(result))
}
