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
        Ok((effective_date, champions)) => {
            let mut champions_val = serde_json::to_value(champions).unwrap_or_else(|_| json!([]));
            if let Value::Array(ref mut arr) = champions_val {
                // Injeta o perfil primordial: Maestro / Master Brain (Orquestrador)
                let orchestrator_report = json!({
                    "profile": "orchestrator",
                    "quality_leader": {
                        "canonical_id": "anthropic/claude-opus-5.5",
                        "display_name": "Claude Opus 5.5",
                        "provider": "nous_portal",
                        "composite_score": 94.2,
                        "blended_cost": 6.30,
                        "roi_index": 35.8,
                        "throughput_tps": 45.0,
                        "context_window": 1000000
                    },
                    "best_value": {
                        "canonical_id": "openai/gpt-6.1-sol",
                        "display_name": "GPT-6.1 Sol",
                        "provider": "nous_portal",
                        "composite_score": 87.9,
                        "blended_cost": 3.15,
                        "roi_index": 43.4,
                        "throughput_tps": 95.0,
                        "context_window": 1050000
                    },
                    "ultra_low_cost": {
                        "canonical_id": "qwen/qwen3.7-flash",
                        "display_name": "Qwen 3.7 Flash",
                        "provider": "nous_portal",
                        "composite_score": 74.3,
                        "blended_cost": 0.0256,
                        "roi_index": 72.4,
                        "throughput_tps": 130.0,
                        "context_window": 1000000
                    }
                });
                arr.insert(0, orchestrator_report);
            }

            Ok(Json(json!({
                "status": "success",
                "date": effective_date,
                "profiles_count": if let Value::Array(ref a) = champions_val { a.len() } else { 0 },
                "champions": champions_val
            })))
        }
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
/// Retorna o ranking expandido de modelos da Cloud Arena (Top 100) com sub-scores de benchmarks
/// e disponibilidade explícita nos provedores Nous Portal e OpenRouter.
pub async fn cloud_ranked_models_handler(
    State(state): State<AppState>,
    Query(params): Query<CloudModelsQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let _guard = state.acquire_connection();

    let raw_profile = params.profile.unwrap_or_else(|| "orchestrator".to_string()).to_lowercase();
    let limit = params.limit.unwrap_or(100).clamp(10, 200);

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
    let is_orchestrator = raw_profile == "orchestrator" || raw_profile == "brain" || raw_profile == "maestro";

    let rows = if is_orchestrator {
        let query_str = r#"
            SELECT 
                de.canonical_id,
                cm.display_name,
                (
                    0.35 * COALESCE(MAX(CASE WHEN de.profile_task = 'agentic' THEN de.composite_score END), 70.0) +
                    0.35 * COALESCE(MAX(CASE WHEN de.profile_task = 'reasoning' THEN de.composite_score END), 70.0) +
                    0.30 * COALESCE(MAX(CASE WHEN de.profile_task = 'coding' THEN de.composite_score END), 70.0)
                ) as score,
                AVG(de.task_blended_cost) as task_blended_cost,
                AVG(de.roi_index) as roi_index,
                'Leader' as pareto_tag,
                cm.context_window,
                (SELECT COUNT(*) FROM pricing_telemetry_snapshots pts WHERE pts.canonical_id = de.canonical_id AND pts.source LIKE '%nous%') as has_nous,
                (SELECT COUNT(*) FROM pricing_telemetry_snapshots pts WHERE pts.canonical_id = de.canonical_id AND pts.source LIKE '%openrouter%') as has_openrouter,
                (SELECT prompt_price_per_m FROM pricing_telemetry_snapshots pts WHERE pts.canonical_id = de.canonical_id AND pts.source LIKE '%nous%' ORDER BY pts.recorded_at DESC LIMIT 1) as nous_prompt_price,
                (SELECT completion_price_per_m FROM pricing_telemetry_snapshots pts WHERE pts.canonical_id = de.canonical_id AND pts.source LIKE '%nous%' ORDER BY pts.recorded_at DESC LIMIT 1) as nous_completion_price,
                (SELECT prompt_price_per_m FROM pricing_telemetry_snapshots pts WHERE pts.canonical_id = de.canonical_id AND pts.source LIKE '%openrouter%' ORDER BY pts.recorded_at DESC LIMIT 1) as or_prompt_price,
                (SELECT completion_price_per_m FROM pricing_telemetry_snapshots pts WHERE pts.canonical_id = de.canonical_id AND pts.source LIKE '%openrouter%' ORDER BY pts.recorded_at DESC LIMIT 1) as or_completion_price,
                MAX(CASE WHEN de.profile_task = 'coding' THEN de.composite_score END) as eval_coding,
                MAX(CASE WHEN de.profile_task = 'agentic' THEN de.composite_score END) as eval_agentic,
                MAX(CASE WHEN de.profile_task = 'reasoning' THEN de.composite_score END) as eval_reasoning,
                MAX(CASE WHEN de.profile_task = 'fast' THEN de.composite_score END) as eval_fast,
                MAX(CASE WHEN de.profile_task = 'design' THEN de.composite_score END) as eval_design,
                bs.general_elo,
                bs.coding_elo,
                bs.bfcl_score,
                bs.ifeval_score,
                bs.gpqa_score,
                bs.hard_prompts_elo,
                (SELECT throughput_tps FROM pricing_telemetry_snapshots pts WHERE pts.canonical_id = de.canonical_id ORDER BY pts.recorded_at DESC LIMIT 1) as tps
            FROM daily_evaluations de
            JOIN canonical_models cm ON cm.id = de.canonical_id
            LEFT JOIN (
                SELECT canonical_id, general_elo, coding_elo, bfcl_score, ifeval_score, gpqa_score, hard_prompts_elo
                FROM benchmark_snapshots
                GROUP BY canonical_id
            ) bs ON bs.canonical_id = de.canonical_id
            WHERE de.evaluation_date = (
                SELECT MAX(evaluation_date) 
                FROM daily_evaluations
            )
            GROUP BY de.canonical_id
            ORDER BY score DESC
            LIMIT ?;
        "#;

        sqlx::query(query_str)
            .bind(limit)
            .fetch_all(&pool)
            .await
    } else {
        let query_str = r#"
            SELECT 
                de.canonical_id,
                cm.display_name,
                MAX(de.composite_score) as score,
                de.task_blended_cost,
                de.roi_index,
                de.pareto_tag,
                cm.context_window,
                (SELECT COUNT(*) FROM pricing_telemetry_snapshots pts WHERE pts.canonical_id = de.canonical_id AND pts.source LIKE '%nous%') as has_nous,
                (SELECT COUNT(*) FROM pricing_telemetry_snapshots pts WHERE pts.canonical_id = de.canonical_id AND pts.source LIKE '%openrouter%') as has_openrouter,
                (SELECT prompt_price_per_m FROM pricing_telemetry_snapshots pts WHERE pts.canonical_id = de.canonical_id AND pts.source LIKE '%nous%' ORDER BY pts.recorded_at DESC LIMIT 1) as nous_prompt_price,
                (SELECT completion_price_per_m FROM pricing_telemetry_snapshots pts WHERE pts.canonical_id = de.canonical_id AND pts.source LIKE '%nous%' ORDER BY pts.recorded_at DESC LIMIT 1) as nous_completion_price,
                (SELECT prompt_price_per_m FROM pricing_telemetry_snapshots pts WHERE pts.canonical_id = de.canonical_id AND pts.source LIKE '%openrouter%' ORDER BY pts.recorded_at DESC LIMIT 1) as or_prompt_price,
                (SELECT completion_price_per_m FROM pricing_telemetry_snapshots pts WHERE pts.canonical_id = de.canonical_id AND pts.source LIKE '%openrouter%' ORDER BY pts.recorded_at DESC LIMIT 1) as or_completion_price,
                MAX(CASE WHEN de.profile_task = 'coding' THEN de.composite_score END) as eval_coding,
                MAX(CASE WHEN de.profile_task = 'agentic' THEN de.composite_score END) as eval_agentic,
                MAX(CASE WHEN de.profile_task = 'reasoning' THEN de.composite_score END) as eval_reasoning,
                MAX(CASE WHEN de.profile_task = 'fast' THEN de.composite_score END) as eval_fast,
                MAX(CASE WHEN de.profile_task = 'design' THEN de.composite_score END) as eval_design,
                bs.general_elo,
                bs.coding_elo,
                bs.bfcl_score,
                bs.ifeval_score,
                bs.gpqa_score,
                bs.hard_prompts_elo,
                (SELECT throughput_tps FROM pricing_telemetry_snapshots pts WHERE pts.canonical_id = de.canonical_id ORDER BY pts.recorded_at DESC LIMIT 1) as tps
            FROM daily_evaluations de
            JOIN canonical_models cm ON cm.id = de.canonical_id
            LEFT JOIN (
                SELECT canonical_id, general_elo, coding_elo, bfcl_score, ifeval_score, gpqa_score, hard_prompts_elo
                FROM benchmark_snapshots
                GROUP BY canonical_id
            ) bs ON bs.canonical_id = de.canonical_id
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

        sqlx::query(query_str)
            .bind(&raw_profile)
            .bind(&raw_profile)
            .bind(limit)
            .fetch_all(&pool)
            .await
    };

    let rows = match rows {
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
        let has_nous: i64 = r.try_get("has_nous").unwrap_or(0);
        let has_openrouter: i64 = r.try_get("has_openrouter").unwrap_or(0);
        let nous_prompt_price: Option<f64> = r.try_get("nous_prompt_price").ok();
        let nous_completion_price: Option<f64> = r.try_get("nous_completion_price").ok();
        let or_prompt_price: Option<f64> = r.try_get("or_prompt_price").ok();
        let or_completion_price: Option<f64> = r.try_get("or_completion_price").ok();
        let eval_coding: Option<f64> = r.try_get("eval_coding").ok();
        let eval_agentic: Option<f64> = r.try_get("eval_agentic").ok();
        let eval_reasoning: Option<f64> = r.try_get("eval_reasoning").ok();
        let eval_fast: Option<f64> = r.try_get("eval_fast").ok();
        let eval_design: Option<f64> = r.try_get("eval_design").ok();
        let general_elo: Option<f64> = r.try_get("general_elo").ok();
        let coding_elo: Option<f64> = r.try_get("coding_elo").ok();
        let bfcl_score: Option<f64> = r.try_get("bfcl_score").ok();
        let ifeval_score: Option<f64> = r.try_get("ifeval_score").ok();
        let gpqa_score: Option<f64> = r.try_get("gpqa_score").ok();
        let hard_prompts_elo: Option<f64> = r.try_get("hard_prompts_elo").ok();
        let tps: Option<f64> = r.try_get("tps").ok();

        let nous_blended = match (nous_prompt_price, nous_completion_price) {
            (Some(p), Some(c)) => Some(0.25 * p + 0.75 * c),
            _ => None,
        };
        let or_blended = match (or_prompt_price, or_completion_price) {
            (Some(p), Some(c)) => Some(0.25 * p + 0.75 * c),
            _ => None,
        };

        let primary_provider = if has_nous > 0 { "nous_portal" } else { "openrouter_market" };

        models.push(json!({
            "rank": idx + 1,
            "canonical_id": canonical_id,
            "display_name": display_name,
            "composite_score": score,
            "blended_cost": blended_cost,
            "roi_index": roi_index,
            "pareto_tag": pareto_tag,
            "context_window": context_window,
            "provider": primary_provider,
            "has_nous": has_nous > 0,
            "has_openrouter": has_openrouter > 0,
            "nous_prompt_price": nous_prompt_price,
            "nous_completion_price": nous_completion_price,
            "nous_blended": nous_blended,
            "or_prompt_price": or_prompt_price,
            "or_completion_price": or_completion_price,
            "or_blended": or_blended,
            "eval_coding": eval_coding,
            "eval_agentic": eval_agentic,
            "eval_reasoning": eval_reasoning,
            "eval_fast": eval_fast,
            "eval_design": eval_design,
            "general_elo": general_elo,
            "coding_elo": coding_elo,
            "bfcl_score": bfcl_score,
            "ifeval_score": ifeval_score,
            "gpqa_score": gpqa_score,
            "hard_prompts_elo": hard_prompts_elo,
            "throughput_tps": tps
        }));
    }

    Ok(Json(json!({
        "status": "success",
        "profile": raw_profile,
        "count": models.len(),
        "models": models
    })))
}

