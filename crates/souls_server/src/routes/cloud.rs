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

    match state.cloud_arena.get_champions_with_effective_date(params.date.as_deref()).await {
        Ok((effective_date, champions)) => Ok(Json(json!({
            "status": "success",
            "date": effective_date,
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

#[derive(Debug, Deserialize)]
pub struct CloudModelsQuery {
    pub profile: Option<String>,
    pub limit: Option<i64>,
}

/// Handler para `GET /api/v1/cloud/models`
/// Retorna o ranking expandido de modelos da Cloud Arena (Top 40+) para o perfil cognitivo especificado.
pub async fn cloud_ranked_models_handler(
    State(state): State<AppState>,
    Query(params): Query<CloudModelsQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let _guard = state.acquire_connection();

    let profile = params.profile.unwrap_or_else(|| "coding".to_string()).to_lowercase();
    let limit = params.limit.unwrap_or(50).clamp(10, 100);

    let vault_path = souls_llm_cloud_arena::resolve_heuristic_vault_path();
    let db_url = format!("sqlite://{}?mode=ro", vault_path.to_string_lossy());

    let pool = match sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(2)
        .connect(&db_url)
        .await
    {
        Ok(p) => p,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "status": "error", "message": format!("Falha ao conectar no SQLite da Cloud Arena: {}", e) })),
            ));
        }
    };

    use sqlx::Row;
    let query_str = r#"
        SELECT 
            de.canonical_id,
            cm.display_name,
            MAX(de.composite_score) as score,
            de.task_blended_cost,
            de.roi_index,
            de.pareto_tag,
            cm.context_window,
            (SELECT source FROM pricing_telemetry_snapshots pts 
             WHERE pts.canonical_id = de.canonical_id 
             ORDER BY pts.recorded_at DESC LIMIT 1) as best_provider,
            (SELECT throughput_tps FROM pricing_telemetry_snapshots pts 
             WHERE pts.canonical_id = de.canonical_id 
             ORDER BY pts.recorded_at DESC LIMIT 1) as tps
        FROM daily_evaluations de
        JOIN canonical_models cm ON cm.id = de.canonical_id
        WHERE de.profile_task = ? 
          AND de.evaluation_date = (
              SELECT MAX(evaluation_date) 
              FROM daily_evaluations 
              WHERE profile_task = ?
          )
        GROUP BY de.canonical_id
        ORDER BY score DESC
        LIMIT ?;
    "#;

    let rows = match sqlx::query(query_str)
        .bind(&profile)
        .bind(&profile)
        .bind(limit)
        .fetch_all(&pool)
        .await
    {
        Ok(r) => r,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "status": "error", "message": format!("Erro na consulta de modelos da arena: {}", e) })),
            ));
        }
    };

    let mut models = Vec::new();
    for (idx, r) in rows.into_iter().enumerate() {
        let canonical_id: String = r.get("canonical_id");
        let display_name: String = r.get("display_name");
        let score: f64 = r.get("score");
        let blended_cost: f64 = r.get("task_blended_cost");
        let roi_index: f64 = r.get("roi_index");
        let pareto_tag: String = r.get("pareto_tag");
        let context_window: i64 = r.get("context_window");
        let provider: String = r.try_get("best_provider").unwrap_or_else(|_| "openrouter_market".to_string());
        let tps: Option<f64> = r.try_get("tps").ok();

        models.push(json!({
            "rank": idx + 1,
            "canonical_id": canonical_id,
            "display_name": display_name,
            "composite_score": score,
            "blended_cost": blended_cost,
            "roi_index": roi_index,
            "pareto_tag": pareto_tag,
            "context_window": context_window,
            "provider": provider,
            "throughput_tps": tps
        }));
    }

    Ok(Json(json!({
        "status": "success",
        "profile": profile,
        "count": models.len(),
        "models": models
    })))
}

