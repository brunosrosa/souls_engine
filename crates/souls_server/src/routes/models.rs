//! Handler for `GET /v1/models` OpenAI-compatible endpoint.
//!
//! Exposes registered and active model arms from `model_registry`.
//! Seeds canonical local models if the database is in cold-start state.

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json};
use serde::{Deserialize, Serialize};
use tracing::info;

use souls_model_router::priors::{
    ensure_router_tables, fetch_active_models, upsert_model, ModelRecord,
};

use crate::state::AppState;

/// OpenAI model item representation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelItem {
    pub id: String,
    pub object: &'static str,
    pub created: i64,
    pub owned_by: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_context_window: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vram_base_mb: Option<u32>,
}

/// OpenAI list models response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelsListResponse {
    pub object: &'static str,
    pub data: Vec<ModelItem>,
}

/// Seeds canonical local models if none are active in SQLite.
pub async fn ensure_canonical_models_seeded(
    pool: &sqlx::Pool<sqlx::Sqlite>,
) -> Result<(), souls_model_router::error::RouterError> {
    ensure_router_tables(pool).await?;
    let active = fetch_active_models(pool).await?;
    if !active.is_empty() {
        return Ok(());
    }

    info!("Cold-start detected in model_registry: seeding canonical local model arms.");

    // 1. Tier 0: ONNX CPU AVX2 (GLiClass / ModernBERT)
    upsert_model(
        pool,
        &ModelRecord {
            model_id: "tier0_onnx_cpu".to_string(),
            provider_type: "LOCAL".to_string(),
            vram_base_mb: 0,
            kv_cache_cost_per_k: 0,
            max_context_window: 8192,
            specialty_tags: "[\"triage\", \"classifier\"]".to_string(),
            score_lmarena: 1000,
            cost_input_per_m: 0.0,
            cost_output_per_m: 0.0,
            is_active: true,
            ema_latency_ms: 25,
            success_rate_ema: 1.0,
        },
    )
    .await?;

    // 2. Tier 0.5: Llama.cpp CPU Logit Probing
    upsert_model(
        pool,
        &ModelRecord {
            model_id: "tier05_llama_cpu".to_string(),
            provider_type: "LOCAL".to_string(),
            vram_base_mb: 0,
            kv_cache_cost_per_k: 0,
            max_context_window: 8192,
            specialty_tags: "[\"probing\", \"entropy\"]".to_string(),
            score_lmarena: 1100,
            cost_input_per_m: 0.0,
            cost_output_per_m: 0.0,
            is_active: true,
            ema_latency_ms: 80,
            success_rate_ema: 1.0,
        },
    )
    .await?;

    // 3. Tier 1: Upstream Qwen CUDA with Asymmetric KV Cache
    upsert_model(
        pool,
        &ModelRecord {
            model_id: "tier1_qwen_coder_gpu".to_string(),
            provider_type: "LOCAL".to_string(),
            vram_base_mb: 1700,
            kv_cache_cost_per_k: 80,
            max_context_window: 16000,
            specialty_tags: "[\"generation\", \"code\"]".to_string(),
            score_lmarena: 1250,
            cost_input_per_m: 0.0,
            cost_output_per_m: 0.0,
            is_active: true,
            ema_latency_ms: 150,
            success_rate_ema: 1.0,
        },
    )
    .await?;

    Ok(())
}

/// GET /v1/models handler.
pub async fn models_handler(State(state): State<AppState>) -> impl IntoResponse {
    let _guard = state.acquire_connection();
    let pool = state.memory.pool();

    if let Err(e) = ensure_canonical_models_seeded(&pool).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": {
                    "message": format!("Failed to ensure models: {e}"),
                    "type": "server_error"
                }
            })),
        );
    }

    let records = match fetch_active_models(&pool).await {
        Ok(m) => m,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": {
                        "message": format!("Failed to fetch models: {e}"),
                        "type": "server_error"
                    }
                })),
            );
        }
    };

    let items: Vec<ModelItem> = records
        .into_iter()
        .map(|r| ModelItem {
            id: r.model_id,
            object: "model",
            created: 1700000000,
            owned_by: "souls_engine",
            provider_type: Some(r.provider_type),
            max_context_window: Some(r.max_context_window),
            vram_base_mb: Some(r.vram_base_mb),
        })
        .collect();

    (
        StatusCode::OK,
        Json(serde_json::to_value(ModelsListResponse {
            object: "list",
            data: items,
        }).unwrap()),
    )
}
