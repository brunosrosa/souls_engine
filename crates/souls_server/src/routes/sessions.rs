//! # Sub-Agent Sessions & Handoff REST Handlers (`souls_server::routes::sessions`)
//!
//! Exposes HTTP REST endpoints for inspecting and governing sub-agent lifecycles,
//! in-RAM operational diaries, and atomic handoff transactions in SQLite STRICT WAL.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::Row;

use crate::state::AppState;

/// Query parameters for listing sub-agent sessions.
#[derive(Debug, Deserialize)]
pub struct ListSubagentsQuery {
    pub parent_session_id: Option<String>,
    pub status: Option<String>,
}

/// Request payload for spawning a new sub-agent session.
#[derive(Debug, Deserialize)]
pub struct SpawnSubagentRequest {
    pub parent_session_id: Option<String>,
    pub task_description: String,
    pub assigned_tier: Option<String>,
}

/// Request payload for updating sub-agent status.
#[derive(Debug, Deserialize)]
pub struct UpdateStatusRequest {
    pub status: String,
}

/// Request payload for recording a sub-agent diary thought.
#[derive(Debug, Deserialize)]
pub struct RecordDiaryRequest {
    pub content: String,
    pub session_id: Option<String>,
}

/// Request payload for closing the sub-agent loop and returning a handoff payload.
#[derive(Debug, Deserialize)]
pub struct CompleteSubagentRequest {
    pub result_summary: String,
    pub variables: Option<Value>,
}

/// Request payload for creating a handoff transaction directly.
#[derive(Debug, Deserialize)]
pub struct CreateHandoffRequest {
    pub source_session_id: String,
    pub target_agent_id: String,
    pub context_summary: String,
    pub variables: Option<Value>,
}

/// Lists sub-agents, optionally filtered by parent session or status.
///
/// `GET /api/v1/sessions/subagents`
pub async fn list_subagents_handler(
    State(state): State<AppState>,
    Query(query): Query<ListSubagentsQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let sub_mgr = state.memory.sub_agent_manager();

    if let Some(ref parent) = query.parent_session_id {
        let list = sub_mgr
            .list_subagents_by_parent(parent)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "status": "error", "message": e.to_string() }))))?;

        let filtered: Vec<_> = if let Some(ref st) = query.status {
            list.into_iter().filter(|r| r.status.as_str().eq_ignore_ascii_case(st)).collect()
        } else {
            list
        };

        return Ok(Json(json!({
            "status": "success",
            "total": filtered.len(),
            "subagents": filtered
        })));
    }

    // Global recent sub-agents query directly from FrankenSQLite
    let pool = state.memory.pool();
    let rows = sqlx::query(
        r#"
        SELECT agent_id, parent_session_id, task_description, assigned_tier, status, created_at, completed_at
        FROM sub_agent_sessions
        ORDER BY created_at DESC
        LIMIT 50
        "#,
    )
    .fetch_all(&*pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "status": "error", "message": e.to_string() }))))?;

    let mut list = Vec::with_capacity(rows.len());
    for r in rows {
        let status_str: String = r.get(4);
        let status = souls_memory::SubAgentStatus::from_str_strict(&status_str)
            .unwrap_or(souls_memory::SubAgentStatus::Failed);

        let record = souls_memory::SubAgentSessionRecord {
            agent_id: r.get(0),
            parent_session_id: r.get(1),
            task_description: r.get(2),
            assigned_tier: r.get(3),
            status,
            created_at: r.get(5),
            completed_at: r.get(6),
        };

        if let Some(ref st) = query.status {
            if !record.status.as_str().eq_ignore_ascii_case(st) {
                continue;
            }
        }

        list.push(record);
    }

    Ok(Json(json!({
        "status": "success",
        "total": list.len(),
        "subagents": list
    })))
}

/// Retrieves a sub-agent session with its full diary trail and handoff payload.
///
/// `GET /api/v1/sessions/subagents/:id`
pub async fn get_subagent_details_handler(
    State(state): State<AppState>,
    Path(agent_id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let sub_mgr = state.memory.sub_agent_manager();

    let session = sub_mgr
        .get_subagent(&agent_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "status": "error", "message": e.to_string() }))))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, Json(json!({ "status": "error", "message": format!("Subagent '{}' not found", agent_id) }))))?;

    let diaries = sub_mgr
        .list_diaries(&agent_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "status": "error", "message": e.to_string() }))))?;

    let received_handoff = sub_mgr
        .get_handoff(&agent_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "status": "error", "message": e.to_string() }))))?;

    // Look for return handoff created by this agent
    let return_handoff = sub_mgr
        .get_handoff(&session.parent_session_id)
        .await
        .ok()
        .flatten()
        .filter(|h| h.source_session_id == session.agent_id);

    Ok(Json(json!({
        "status": "success",
        "subagent": session,
        "diaries": diaries,
        "received_handoff": received_handoff,
        "return_handoff": return_handoff,
    })))
}

/// Spawns a new sub-agent session.
///
/// `POST /api/v1/sessions/subagents/spawn`
pub async fn spawn_subagent_handler(
    State(state): State<AppState>,
    Json(payload): Json<SpawnSubagentRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let sub_mgr = state.memory.sub_agent_manager();
    let parent = payload.parent_session_id.as_deref().unwrap_or("main");
    let tier = payload.assigned_tier.as_deref().unwrap_or("Tier1");

    let record = sub_mgr
        .spawn_subagent(parent, &payload.task_description, tier)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "status": "error", "message": e.to_string() }))))?;

    Ok(Json(json!({
        "status": "success",
        "subagent": record
    })))
}

/// Updates sub-agent status.
///
/// `POST /api/v1/sessions/subagents/:id/status`
pub async fn update_status_handler(
    State(state): State<AppState>,
    Path(agent_id): Path<String>,
    Json(payload): Json<UpdateStatusRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let sub_mgr = state.memory.sub_agent_manager();
    let status = souls_memory::SubAgentStatus::from_str_strict(&payload.status)
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(json!({ "status": "error", "message": e.to_string() }))))?;

    let record = sub_mgr
        .update_status(&agent_id, status)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "status": "error", "message": e.to_string() }))))?;

    Ok(Json(json!({
        "status": "success",
        "subagent": record
    })))
}

/// Records a diary note for a sub-agent.
///
/// `POST /api/v1/sessions/subagents/:id/diary`
pub async fn record_diary_handler(
    State(state): State<AppState>,
    Path(agent_id): Path<String>,
    Json(payload): Json<RecordDiaryRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let sub_mgr = state.memory.sub_agent_manager();
    let sess_id = payload.session_id.as_deref().unwrap_or("subagent_internal");

    let record = sub_mgr
        .record_diary(&agent_id, sess_id, &payload.content)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "status": "error", "message": e.to_string() }))))?;

    Ok(Json(json!({
        "status": "success",
        "diary": record
    })))
}

/// Finalizes a sub-agent session and generates return handoff for the parent.
///
/// `POST /api/v1/sessions/subagents/:id/complete`
pub async fn complete_subagent_handler(
    State(state): State<AppState>,
    Path(agent_id): Path<String>,
    Json(payload): Json<CompleteSubagentRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let sub_mgr = state.memory.sub_agent_manager();

    // 1. Mark as completed
    let session = sub_mgr
        .update_status(&agent_id, souls_memory::SubAgentStatus::Completed)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "status": "error", "message": e.to_string() }))))?;

    // 2. Generate atomic return handoff payload targeting the parent session
    let variables = payload.variables.unwrap_or(json!({}));
    let return_handoff = sub_mgr
        .create_handoff(
            &session.agent_id,
            &session.parent_session_id,
            &payload.result_summary,
            variables,
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "status": "error", "message": e.to_string() }))))?;

    Ok(Json(json!({
        "status": "success",
        "subagent": session,
        "return_handoff": return_handoff
    })))
}

/// Retrieves the most recent handoff payload for a target agent.
///
/// `GET /api/v1/sessions/handoffs/:agent_id`
pub async fn get_handoff_handler(
    State(state): State<AppState>,
    Path(agent_id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let sub_mgr = state.memory.sub_agent_manager();

    let handoff = sub_mgr
        .get_handoff(&agent_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "status": "error", "message": e.to_string() }))))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, Json(json!({ "status": "error", "message": format!("No handoff found for '{}'", agent_id) }))))?;

    Ok(Json(json!({
        "status": "success",
        "handoff": handoff
    })))
}

/// Creates a handoff transaction directly.
///
/// `POST /api/v1/sessions/handoffs`
pub async fn create_handoff_handler(
    State(state): State<AppState>,
    Json(payload): Json<CreateHandoffRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let sub_mgr = state.memory.sub_agent_manager();
    let vars = payload.variables.unwrap_or(json!({}));

    let record = sub_mgr
        .create_handoff(
            &payload.source_session_id,
            &payload.target_agent_id,
            &payload.context_summary,
            vars,
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "status": "error", "message": e.to_string() }))))?;

    Ok(Json(json!({
        "status": "success",
        "handoff": record
    })))
}
