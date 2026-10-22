//! HTTP REST handlers for `souls_llm_local_arena` (Local Silicon Benchmarks).
//!
//! Provides bare-metal hardware benchmarking, cognitive evaluations,
//! telemetry extraction and sync with `souls_heuristic_vault.db`.

use axum::extract::{Json, State};
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::{sqlite::SqlitePoolOptions, Row};
use std::collections::HashMap;

use souls_llm_local_arena::{
    resolve_vault_db_path, EngineType, ModelTier,
};

use crate::state::AppState;

/// DTO for requesting a benchmark execution on a local model.
#[derive(Debug, Deserialize)]
pub struct RunBenchmarkPayload {
    pub model_id: String,
    pub benchmark_type: Option<u8>,
    pub engine: Option<String>,
}

/// GET /api/v1/arena/models
/// Lists all local models detected by the Local Arena scanner along with their latest vault benchmarks.
pub async fn arena_models_handler(State(state): State<AppState>) -> Json<Value> {
    let _guard = state.acquire_connection();
    let runner = state.local_arena.lock().await;
    let discovered = runner.discover_models();

    // Query vault model_registry for historical benchmarks
    let vault_path = resolve_vault_db_path();
    let mut registry_map: HashMap<String, (f64, f64, f64, i64, f64)> = HashMap::new();

    if let Ok(pool) = SqlitePoolOptions::new()
        .max_connections(2)
        .connect(&format!("sqlite://{}?mode=ro", vault_path.to_string_lossy().replace('\\', "/")))
        .await
    {
        if let Ok(rows) = sqlx::query(
            "SELECT COALESCE(model_id, model_name) as model_id,
                    COALESCE(tps, 0.0) as tps,
                    COALESCE(ttft_ms, 0.0) as ttft_ms,
                    COALESCE(vram_peak_mb, 0.0) as vram_peak_mb,
                    COALESCE(last_benchmarked_epoch, 0) as last_epoch,
                    COALESCE(e3_score, 0.0) as e3_score
             FROM model_registry;"
        )
        .fetch_all(&pool)
        .await
        {
            for r in rows {
                let m_id: String = r.get("model_id");
                let tps: f64 = r.get("tps");
                let ttft: f64 = r.get("ttft_ms");
                let vram: f64 = r.get("vram_peak_mb");
                let epoch: i64 = r.get("last_epoch");
                let e3: f64 = r.get("e3_score");
                registry_map.insert(m_id.to_lowercase(), (tps, ttft, vram, epoch, e3));
            }
        }
    }

    let models_json: Vec<Value> = discovered
        .into_iter()
        .map(|m| {
            let key = m.model_id.to_lowercase();
            let bench_data = registry_map.get(&key).copied();

            let (latest_tps, latest_ttft_ms, latest_vram_peak_mb, last_benchmarked, e3_score) = match bench_data {
                Some((tps, ttft, vram, ep, e3)) => (
                    if tps > 0.0 { Some(tps) } else { None },
                    if ttft > 0.0 { Some(ttft) } else { None },
                    if vram > 0.0 { Some(vram) } else { None },
                    if ep > 0 { Some(ep) } else { None },
                    if e3 > 0.0 { Some(e3) } else { None },
                ),
                None => (None, None, None, None, None),
            };

            let status = if latest_tps.is_some() {
                "BENCHMARKED"
            } else {
                "PENDING"
            };

            json!({
                "model_id": m.model_id,
                "name": m.name,
                "file_path": m.file_path.to_string_lossy(),
                "tier": m.tier.as_str(),
                "parameters": m.parameters,
                "quantization": m.quant,
                "size_mb": m.size_mb,
                "context_length": m.ctx_max,
                "has_mmproj": m.has_mmproj,
                "mmproj_path": m.mmproj_path.map(|p| p.to_string_lossy().to_string()),
                "status": status,
                "latest_tps": latest_tps,
                "latest_ttft_ms": latest_ttft_ms,
                "latest_vram_peak_mb": latest_vram_peak_mb,
                "last_benchmarked": last_benchmarked,
                "e3_score": e3_score
            })
        })
        .collect();

    Json(json!({
        "status": "success",
        "total": models_json.len(),
        "models": models_json
    }))
}

/// GET /api/v1/arena/benchmarks
/// Returns historical telemetry logs and multi-dimensional task profiles from the Heuristic Vault.
pub async fn arena_benchmarks_handler(State(state): State<AppState>) -> Json<Value> {
    let _guard = state.acquire_connection();
    let vault_path = resolve_vault_db_path();

    let pool = match SqlitePoolOptions::new()
        .max_connections(2)
        .connect(&format!("sqlite://{}?mode=ro", vault_path.to_string_lossy().replace('\\', "/")))
        .await
    {
        Ok(p) => p,
        Err(e) => {
            return Json(json!({
                "status": "error",
                "message": format!("Failed to open Heuristic Vault database: {}", e),
                "history": [],
                "profiles": []
            }));
        }
    };

    // Query recent 50 telemetry runs
    let telemetry_rows = sqlx::query(
        "SELECT id, COALESCE(model_id, 'unknown') as model_id,
                COALESCE(benchmark_type, 'HardwareProfile') as benchmark_type,
                COALESCE(engine, 'llama-cuda') as engine,
                COALESCE(ttft_ms, 0.0) as ttft_ms,
                COALESCE(tpot_ms, 0.0) as tpot_ms,
                COALESCE(tps, 0.0) as tps,
                COALESCE(vram_peak_mb, 0.0) as vram_peak_mb,
                COALESCE(ram_mb, 0.0) as ram_mb,
                COALESCE(e3_score, 0.0) as e3_score,
                COALESCE(status, 'OK') as status,
                COALESCE(created_at, '') as created_at
         FROM arena_telemetry
         ORDER BY id DESC
         LIMIT 50;"
    )
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    let history: Vec<Value> = telemetry_rows
        .into_iter()
        .map(|r| {
            let id: i64 = r.get("id");
            let model_id: String = r.get("model_id");
            let benchmark_type: String = r.get("benchmark_type");
            let engine: String = r.get("engine");
            let ttft_ms: f64 = r.get("ttft_ms");
            let tpot_ms: f64 = r.get("tpot_ms");
            let tps: f64 = r.get("tps");
            let vram_peak_mb: f64 = r.get("vram_peak_mb");
            let ram_mb: f64 = r.get("ram_mb");
            let e3_score: f64 = r.get("e3_score");
            let status: String = r.get("status");
            let created_at: String = r.get("created_at");

            json!({
                "id": id,
                "model_id": model_id,
                "benchmark_type": benchmark_type,
                "engine": engine,
                "ttft_ms": ttft_ms,
                "tpot_ms": tpot_ms,
                "tps": tps,
                "vram_peak_mb": vram_peak_mb,
                "ram_mb": ram_mb,
                "e3_score": e3_score,
                "status": status,
                "created_at": created_at
            })
        })
        .collect();

    // Query task profiles and pareto classifications
    let profile_rows = sqlx::query(
        "SELECT model_id, tier, parameters, quantization,
                score_coding, score_agentic, score_reasoning, score_fast,
                silicon_cost, roi_coding, roi_agentic, roi_reasoning, roi_fast,
                is_pareto_coding, is_pareto_agentic, is_pareto_reasoning, is_pareto_fast,
                pareto_tag_coding, pareto_tag_agentic, pareto_tag_reasoning, pareto_tag_fast,
                updated_at
         FROM model_task_profiles
         ORDER BY score_coding DESC;"
    )
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    let profiles: Vec<Value> = profile_rows
        .into_iter()
        .map(|r| {
            let model_id: String = r.get("model_id");
            let tier: String = r.get("tier");
            let parameters: String = r.get("parameters");
            let quantization: String = r.get("quantization");
            let score_coding: f64 = r.get("score_coding");
            let score_agentic: f64 = r.get("score_agentic");
            let score_reasoning: f64 = r.get("score_reasoning");
            let score_fast: f64 = r.get("score_fast");
            let silicon_cost: f64 = r.get("silicon_cost");
            let roi_coding: f64 = r.get("roi_coding");
            let is_p_code: i64 = r.get("is_pareto_coding");
            let tag_code: String = r.get("pareto_tag_coding");

            json!({
                "model_id": model_id,
                "tier": tier,
                "parameters": parameters,
                "quantization": quantization,
                "score_coding": score_coding,
                "score_agentic": score_agentic,
                "score_reasoning": score_reasoning,
                "score_fast": score_fast,
                "silicon_cost": silicon_cost,
                "roi_coding": roi_coding,
                "is_pareto_coding": is_p_code == 1,
                "pareto_tag_coding": tag_code
            })
        })
        .collect();

    Json(json!({
        "status": "success",
        "history_count": history.len(),
        "profiles_count": profiles.len(),
        "history": history,
        "profiles": profiles
    }))
}

/// POST /api/v1/arena/benchmark/run
/// Runs an empirical benchmark battery on the real physical silicon for a given local model.
pub async fn arena_run_benchmark_handler(
    State(state): State<AppState>,
    Json(payload): Json<RunBenchmarkPayload>,
) -> Json<Value> {
    let _guard = state.acquire_connection();
    let mut runner = state.local_arena.lock().await;

    let discovered = runner.discover_models();
    let pat = payload.model_id.to_lowercase();
    let target = discovered.into_iter().find(|m| {
        m.model_id.to_lowercase().contains(&pat)
            || m.file_path.to_string_lossy().to_lowercase().contains(&pat)
            || m.name.to_lowercase().contains(&pat)
    });

    let model = match target {
        Some(m) => m,
        None => {
            return Json(json!({
                "status": "error",
                "message": format!("Model '{}' not found in local scanned directories.", payload.model_id)
            }));
        }
    };

    // Determine engine strictly: production defaults to real silicon (CUDA for Tier 1+, CPU AVX2 for Tier 0/0.5)
    let engine_type = if let Some(ref e) = payload.engine {
        EngineType::parse(e).unwrap_or(EngineType::LlamaCppCuda)
    } else {
        match model.tier {
            ModelTier::Tier0Bootstrap | ModelTier::Tier05Epistemic => EngineType::LlamaCppCpu,
            _ => EngineType::LlamaCppCuda,
        }
    };

    let b_type = payload.benchmark_type.unwrap_or(1);

    match b_type {
        1 => {
            // Type 1: HardwareProfile
            match runner.run_hardware_profile(&model, engine_type).await {
                Ok(res) => Json(json!({
                    "status": "success",
                    "benchmark_type": "HardwareProfile",
                    "model_id": res.model_id,
                    "engine": res.engine,
                    "metrics": {
                        "tps": res.tps,
                        "ttft_ms": res.ttft_ms,
                        "tpot_ms": res.tpot_ms,
                        "peak_vram_mb": res.peak_vram_mb,
                        "ram_mb": res.ram_mb,
                        "cold_load_ms": res.cold_load_ms,
                        "e3_score": res.e3_score,
                        "verdict": res.status
                    }
                })),
                Err(e) => Json(json!({
                    "status": "error",
                    "message": format!("HardwareProfile benchmark failed: {}", e)
                })),
            }
        }
        2 => {
            // Type 2: CognitiveEval (Rust AST track)
            match runner.run_cognitive_eval(&model, engine_type, "2A_RustAst").await {
                Ok(res) => Json(json!({
                    "status": "success",
                    "benchmark_type": "CognitiveEval",
                    "model_id": res.model_id,
                    "track": res.track,
                    "metrics": {
                        "accuracy_pct": res.accuracy_pct,
                        "avg_tps": res.avg_tps,
                        "avg_ttft_ms": res.avg_ttft_ms,
                        "e3_score": res.e3_score,
                        "verdict": res.status
                    }
                })),
                Err(e) => Json(json!({
                    "status": "error",
                    "message": format!("CognitiveEval benchmark failed: {}", e)
                })),
            }
        }
        3 => {
            // Type 3: MultimodalVision
            match runner.run_multimodal_vision(&model, engine_type).await {
                Ok(res) => Json(json!({
                    "status": "success",
                    "benchmark_type": "MultimodalVision",
                    "model_id": res.model_id,
                    "metrics": {
                        "sidecar_path": res.sidecar_path,
                        "multimodal_ttft_ms": res.multimodal_ttft_ms,
                        "projection_latency_ms": res.projection_latency_ms,
                        "verdict": res.status
                    }
                })),
                Err(e) => Json(json!({
                    "status": "error",
                    "message": format!("MultimodalVision benchmark failed: {}", e)
                })),
            }
        }
        4 => {
            // Type 4: SpeculativeDraft
            match runner.run_speculative_draft(&model, engine_type, None).await {
                Ok(res) => Json(json!({
                    "status": "success",
                    "benchmark_type": "SpeculativeDraft",
                    "model_id": res.model_id,
                    "metrics": {
                        "speculative_tps": res.speculative_tps,
                        "speedup_ratio": res.speedup_ratio,
                        "acceptance_rate_alpha": res.acceptance_rate_alpha,
                        "verdict": res.verdict
                    }
                })),
                Err(e) => Json(json!({
                    "status": "error",
                    "message": format!("SpeculativeDraft benchmark failed: {}", e)
                })),
            }
        }
        5 => {
            // Type 5: ContextPressure
            match runner.run_context_pressure(&model, engine_type).await {
                Ok(res) => Json(json!({
                    "status": "success",
                    "benchmark_type": "ContextPressure",
                    "model_id": res.model_id,
                    "metrics": {
                        "max_tested_context": res.max_tested_context,
                        "max_effective_context": res.max_effective_context,
                        "needle_at_4k": res.needle_at_4k,
                        "needle_at_8k": res.needle_at_8k,
                        "needle_at_16k": res.needle_at_16k,
                        "needle_at_32k": res.needle_at_32k,
                        "degradation_detected": res.degradation_detected,
                        "verdict": res.status
                    }
                })),
                Err(e) => Json(json!({
                    "status": "error",
                    "message": format!("ContextPressure benchmark failed: {}", e)
                })),
            }
        }
        _ => Json(json!({
            "status": "error",
            "message": format!("Invalid benchmark_type '{}'. Valid range: 1 to 5.", b_type)
        })),
    }
}

/// POST /api/v1/arena/sync
/// Synchronizes local discovered models to the SQLite Vault and ensures Hermes HardLinks are current.
pub async fn arena_sync_handler(State(state): State<AppState>) -> Json<Value> {
    let _guard = state.acquire_connection();
    let runner = state.local_arena.lock().await;

    let sync_res = runner.sync_models_to_vault().await;
    let hermes_res = runner.sync_hermes_links(None);

    match (sync_res, hermes_res) {
        (Ok(count), Ok(report)) => Json(json!({
            "status": "success",
            "synced_vault_models": count,
            "hermes_links": {
                "created": report.links_created,
                "verified": report.links_verified,
                "removed": report.links_removed
            }
        })),
        (Err(e), _) => Json(json!({
            "status": "error",
            "message": format!("Failed to sync models to vault: {}", e)
        })),
        (_, Err(e)) => Json(json!({
            "status": "error",
            "message": format!("Failed to sync Hermes hardlinks: {}", e)
        })),
    }
}
