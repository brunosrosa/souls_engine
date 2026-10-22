//! # Switchboard REST Handlers (`souls_server::routes::switchboard`)
//!
//! Exposes HTTP REST endpoints for inspecting and testing the Two-Stage Semantic Switchboard
//! on Host CPU (AVX2, 0 MB VRAM).

use axum::extract::State;
use axum::response::Json;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::mcp::switchboard::{SwitchboardDecision, CANONICAL_SKILLS};
use crate::state::AppState;

/// Request payload for evaluating a prompt/intent via the Switchboard.
#[derive(Debug, Deserialize)]
pub struct SwitchboardEvaluateRequest {
    pub query: String,
    pub namespace: Option<String>,
    pub top_k: Option<usize>,
}

/// Evaluates a prompt or intent against the Two-Stage Semantic Switchboard.
///
/// `POST /api/v1/switchboard/evaluate`
pub async fn evaluate_handler(
    State(state): State<AppState>,
    Json(payload): Json<SwitchboardEvaluateRequest>,
) -> Json<Value> {
    let decision: SwitchboardDecision = crate::mcp::switchboard::evaluate_switchboard(
        &state,
        &payload.query,
        payload.namespace.as_deref(),
        payload.top_k,
    )
    .await;

    Json(json!({
        "status": "success",
        "decision": decision
    }))
}

/// Returns the static catalog of all 15 canonical operational skills.
///
/// `GET /api/v1/switchboard/skills`
pub async fn list_skills_handler() -> Json<Value> {
    Json(json!({
        "status": "success",
        "total": CANONICAL_SKILLS.len(),
        "skills": CANONICAL_SKILLS
    }))
}

/// Diagnostic health check for the Two-Stage Switchboard engine.
///
/// `GET /api/v1/switchboard/health`
pub async fn health_handler() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "mode": "Zero-VRAM Bare-Metal Host CPU AVX2",
        "total_skills": CANONICAL_SKILLS.len(),
        "total_tools": crate::mcp::registry::get_canonical_tools().len(),
        "stage1": "Dense RAM Cosine Vector Match (< 2ms)",
        "stage2": "Tier 0.5 Logit Probing (~8ms)"
    }))
}
