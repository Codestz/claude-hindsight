//! Cross-session prompts endpoint

use crate::server::dto::PromptEntryDto;
use crate::server::{error::ApiError, AppState};
use crate::storage::SessionIndex;
use axum::{
    extract::{Query, State},
    Json,
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct PromptsQuery {
    pub project: Option<String>,
    pub limit: Option<usize>,
}

/// Minimum character length for a message to qualify as a prompt.
const MIN_PROMPT_LENGTH: usize = 300;

/// Extract the full text of the first real user message from a session file.
/// Returns None if parsing fails or no qualifying prompt exists.
fn extract_first_prompt(path: &std::path::Path) -> Option<String> {
    let parsed = crate::parser::parse_session(path).ok()?;
    parsed
        .nodes
        .iter()
        .filter(|n| n.node_type == "user")
        .find_map(|n| {
            let text = n.message.as_ref()?.text_content();
            let trimmed = text.trim().to_string();
            if trimmed.len() < MIN_PROMPT_LENGTH {
                return None;
            }
            if crate::analyzer::prompt_detect::is_local_command_text(&trimmed) {
                return None;
            }
            Some(trimmed)
        })
}

/// GET /api/prompts — returns first-message prompts across all sessions
pub async fn list_prompts(
    State(_state): State<AppState>,
    Query(q): Query<PromptsQuery>,
) -> Result<Json<Vec<PromptEntryDto>>, ApiError> {
    let result = tokio::task::spawn_blocking(move || -> crate::error::Result<_> {
        let index = SessionIndex::new()?;
        let mut sessions = if let Some(ref project) = q.project {
            index.find_by_project(project)?
        } else {
            index.list_sessions()?
        };

        // Pre-sort so we only parse the top N files
        sessions.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        let limit = q.limit.unwrap_or(100);
        let mut dtos = Vec::with_capacity(limit);

        for s in sessions {
            if dtos.len() >= limit {
                break;
            }
            // Parse the actual session file to get full prompt text
            let prompt_text = match extract_first_prompt(&s.path) {
                Some(t) => t,
                None => continue, // No real user message, skip
            };

            dtos.push(PromptEntryDto {
                session_id: s.session_id,
                project_name: s.project_name,
                prompt_text,
                prompt_score: 50,
                timestamp: Some(s.created_at),
                model: s.model,
            });
        }

        Ok(dtos)
    })
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))??;

    Ok(Json(result))
}
