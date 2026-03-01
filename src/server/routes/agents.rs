//! Agent & Skill API routes
//!
//! GET /api/agents          → Vec<AgentConfig>
//! GET /api/agents/:name    → AgentConfig
//! GET /api/skills          → Vec<SkillConfig>
//! GET /api/skills/:name    → SkillConfig

use crate::agents::{discover_agents, discover_skills, AgentConfig, SkillConfig};
use crate::server::{error::ApiError, AppState};
use axum::{
    extract::{Path, State},
    Json,
};

pub async fn list_agents(
    State(_): State<AppState>,
) -> Result<Json<Vec<AgentConfig>>, ApiError> {
    let agents = tokio::task::spawn_blocking(discover_agents)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    Ok(Json(agents))
}

pub async fn get_agent(
    State(_): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<AgentConfig>, ApiError> {
    let agents = tokio::task::spawn_blocking(discover_agents)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    agents
        .into_iter()
        .find(|a| a.name == name)
        .map(Json)
        .ok_or_else(|| ApiError::NotFound(format!("Agent '{}' not found", name)))
}

pub async fn list_skills(
    State(_): State<AppState>,
) -> Result<Json<Vec<SkillConfig>>, ApiError> {
    let skills = tokio::task::spawn_blocking(discover_skills)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    Ok(Json(skills))
}

pub async fn get_skill(
    State(_): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<SkillConfig>, ApiError> {
    let skills = tokio::task::spawn_blocking(discover_skills)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    skills
        .into_iter()
        .find(|s| s.name == name)
        .map(Json)
        .ok_or_else(|| ApiError::NotFound(format!("Skill '{}' not found", name)))
}
