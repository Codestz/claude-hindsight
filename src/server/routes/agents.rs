//! Agent & Skill API routes
//!
//! GET /api/agents          → Vec<AgentGroup>
//! GET /api/agents/:name    → AgentGroup
//! GET /api/skills          → Vec<SkillGroup>
//! GET /api/skills/:name    → SkillGroup

use crate::agents::{discover_agents, discover_skills, group_agents, group_skills, AgentGroup, SkillGroup};
use crate::server::{error::ApiError, AppState};
use axum::{
    extract::{Path, State},
    Json,
};

pub async fn list_agents(
    State(_): State<AppState>,
) -> Result<Json<Vec<AgentGroup>>, ApiError> {
    let agents = tokio::task::spawn_blocking(discover_agents)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    Ok(Json(group_agents(agents)))
}

pub async fn get_agent(
    State(_): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<AgentGroup>, ApiError> {
    let agents = tokio::task::spawn_blocking(discover_agents)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let items: Vec<_> = agents.into_iter().filter(|a| a.name == name).collect();
    if items.is_empty() {
        return Err(ApiError::NotFound(format!("Agent '{}' not found", name)));
    }
    let identical = items.windows(2).all(|w| w[0].body == w[1].body);
    Ok(Json(AgentGroup { name, identical, items }))
}

pub async fn list_skills(
    State(_): State<AppState>,
) -> Result<Json<Vec<SkillGroup>>, ApiError> {
    let skills = tokio::task::spawn_blocking(discover_skills)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    Ok(Json(group_skills(skills)))
}

pub async fn get_skill(
    State(_): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<SkillGroup>, ApiError> {
    let skills = tokio::task::spawn_blocking(discover_skills)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let items: Vec<_> = skills.into_iter().filter(|s| s.name == name).collect();
    if items.is_empty() {
        return Err(ApiError::NotFound(format!("Skill '{}' not found", name)));
    }
    let identical = items.windows(2).all(|w| w[0].body == w[1].body);
    Ok(Json(SkillGroup { name, identical, items }))
}
