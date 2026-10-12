//! # Handler para endpoints REST de Cloud Arena (`/api/v1/cloud/*`)
//!
//! Exposição de campeões de Pareto da nuvem e sincronização sob demanda.

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::Json;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct CloudQueryParams {
    pub date: Option<String>,
    pub force: Option<bool>,
}

/// Handler para `GET /api/v1/cloud/champions`
pub async fn cloud_champions_handler(
    State(state): State<AppState>,
    Query(params): Query<CloudQueryParams>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let _guard = state.acquire_connection();

    match state.cloud_arena.get_all_champions(params.date.as_deref()).await {
        Ok(champions) => Ok(Json(json!({
            "status": "success",
            "date": params.date.unwrap_or_else(|| chrono::Utc::now().format("%Y-%m-%d").to_string()),
            "profiles_count": champions.len(),
            "champions": champions
        }))),
        Err(err) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "status": "error", "message": err.to_string() })),
        )),
    }
}

/// Handler para `POST /api/v1/cloud/sync`
pub async fn cloud_sync_handler(
    State(state): State<AppState>,
    Query(params): Query<CloudQueryParams>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let _guard = state.acquire_connection();

    let outcome_res = if params.force.unwrap_or(false) || params.date.is_some() {
        state.cloud_arena.force_sync(params.date.as_deref()).await
    } else {
        match state.cloud_arena.sync_if_needed().await {
            Ok(Some(outcome)) => Ok(outcome),
            Ok(None) => {
                return Ok(Json(json!({
                    "status": "skipped",
                    "message": "Cloud Arena já está atualizada para o dia de hoje."
                })));
            }
            Err(e) => Err(e),
        }
    };

    match outcome_res {
        Ok(outcome) => Ok(Json(json!({
            "status": "success",
            "outcome": outcome
        }))),
        Err(err) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "status": "error", "message": err.to_string() })),
        )),
    }
}
