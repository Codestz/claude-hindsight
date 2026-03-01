//! Route modules

pub mod agents;
pub mod analytics;
pub mod events;
pub mod hooks;
pub mod otel;
pub mod otel_query;
pub mod projects;
pub mod prompts;
pub mod search;
pub mod sessions;
pub mod telemetry;

/// Shared helper: open a `SessionIndex`, run `f` inside `spawn_blocking`, return JSON.
pub async fn with_index<T, F>(f: F) -> Result<axum::Json<T>, super::error::ApiError>
where
    T: serde::Serialize + Send + 'static,
    F: FnOnce(crate::storage::SessionIndex) -> crate::error::Result<T> + Send + 'static,
{
    let result = tokio::task::spawn_blocking(move || {
        let index = crate::storage::SessionIndex::new()?;
        f(index)
    })
    .await
    .map_err(|e| super::error::ApiError::Internal(e.to_string()))??;
    Ok(axum::Json(result))
}
