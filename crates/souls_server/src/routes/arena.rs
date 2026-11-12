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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SpeculativeCompanionInfo {
    pub status: String,
    pub companion_id: Option<String>,
    pub recommended_companion: Option<String>,
    pub reason: String,
}

fn is_sub1b_or_draft_tag(s: &str) -> bool {
    let s_lower = s.to_lowercase();
    s_lower.contains("0.5b")
        || s_lower.contains("0-5b")
        || s_lower.contains("0_5b")
        || s_lower.contains("135m")
        || s_lower.contains("360m")
        || s_lower.contains("draft")
}

/// Core resolution logic accepting raw parameters for reuse across arena and dashboard.
pub fn resolve_speculative_companion_parts(
    mid: &str,
    name: &str,
    size_mb: f64,
    is_vlm: bool,
    candidates: &[(String, String)], // Vec of (model_id, display_name)
) -> SpeculativeCompanionInfo {
    let mid_lower = mid.to_lowercase();
    let name_lower = name.to_lowercase();

    // 1. Isenção Multimodal (VisionExempt)
    let is_multimodal = is_vlm
        || mid_lower.contains("vl")
        || name_lower.contains("vl")
        || mid_lower.contains("vision")
        || name_lower.contains("vision");

    if is_multimodal {
        return SpeculativeCompanionInfo {
            status: "VisionExempt".to_string(),
            companion_id: None,
            recommended_companion: None,
            reason: "Modelos multimodais de visão operam com encoder visual dedicado sem draft de texto.".to_string(),
        };
    }

    // 2. Native MTP check
    if mid_lower.contains("mtp") || name_lower.contains("mtp") {
        return SpeculativeCompanionInfo {
            status: "NativeMtp".to_string(),
            companion_id: Some("Embedded-MTP-Heads".to_string()),
            recommended_companion: None,
            reason: "Modelo com cabeças de predição Multi-Token (MTP) embutidas nativamente.".to_string(),
        };
    }

    // 3. Sub-1B check (< 600MB or explicit 135M / 360M / 0.5B tags with hyphens, dots, underscores)
    let is_sub1b = (size_mb > 0.0 && size_mb < 600.0)
        || is_sub1b_or_draft_tag(&mid_lower)
        || is_sub1b_or_draft_tag(&name_lower);

    if is_sub1b {
        return SpeculativeCompanionInfo {
            status: "Sub1bExempt".to_string(),
            companion_id: None,
            recommended_companion: None,
            reason: "Modelos sub-1B já operam no limite de velocidade do silício (100+ tok/s). Rascunhos adicionais adicionariam overhead sem ganho real.".to_string(),
        };
    }

    // 4. Qwen Family Resolution
    let is_qwen = mid_lower.contains("qwen") || name_lower.contains("qwen");
    if is_qwen {
        let is_qwen3 = mid_lower.contains("qwen3")
            || mid_lower.contains("qwen-3")
            || mid_lower.contains("qwen_3")
            || name_lower.contains("qwen3")
            || name_lower.contains("qwen-3")
            || name_lower.contains("qwen_3");

        if is_qwen3 {
            // Modelos Qwen 3/3.5 usam NativeMtp ou draft da própria geração Qwen 3, nunca Qwen2.5!
            let qwen3_draft = candidates.iter().find(|(c_mid, c_name)| {
                let cm = c_mid.to_lowercase();
                let cn = c_name.to_lowercase();
                let is_c_q3 = cm.contains("qwen3") || cm.contains("qwen-3") || cm.contains("qwen_3")
                    || cn.contains("qwen3") || cn.contains("qwen-3") || cn.contains("qwen_3");
                is_c_q3 && (is_sub1b_or_draft_tag(&cm) || is_sub1b_or_draft_tag(&cn))
            });

            if let Some(d) = qwen3_draft {
                return SpeculativeCompanionInfo {
                    status: "PairedAvailable".to_string(),
                    companion_id: Some(d.0.clone()),
                    recommended_companion: Some("Qwen3-Draft".to_string()),
                    reason: format!("Par rascunho compatível detectado: {}", d.1),
                };
            } else {
                return SpeculativeCompanionInfo {
                    status: "NativeMtp".to_string(),
                    companion_id: Some("Embedded-MTP-Heads".to_string()),
                    recommended_companion: None,
                    reason: "Modelo Qwen 3/3.5 com predição especulativa nativa (Native MTP).".to_string(),
                };
            }
        } else {
            // Qwen 2 / 2.5
            let draft = candidates.iter().find(|(c_mid, c_name)| {
                let cm = c_mid.to_lowercase();
                let cn = c_name.to_lowercase();
                let is_c_qwen = (cm.contains("qwen") || cn.contains("qwen"))
                    && !(cm.contains("qwen3") || cm.contains("qwen-3") || cm.contains("qwen_3")
                        || cn.contains("qwen3") || cn.contains("qwen-3") || cn.contains("qwen_3"));
                is_c_qwen && (is_sub1b_or_draft_tag(&cm) || is_sub1b_or_draft_tag(&cn))
            });

            if let Some(d) = draft {
                return SpeculativeCompanionInfo {
                    status: "PairedAvailable".to_string(),
                    companion_id: Some(d.0.clone()),
                    recommended_companion: Some("Qwen2.5-Coder-0.5B-Instruct".to_string()),
                    reason: format!("Par rascunho compatível detectado: {}", d.1),
                };
            } else {
                return SpeculativeCompanionInfo {
                    status: "CompanionMissing".to_string(),
                    companion_id: None,
                    recommended_companion: Some("Qwen2.5-Coder-0.5B-Instruct-Q4_K_M.gguf".to_string()),
                    reason: "Requer par draft Qwen 0.5B instalado (disponível no Hub)".to_string(),
                };
            }
        }
    }

    // 5. Llama Family Resolution
    let is_llama = mid_lower.contains("llama") || name_lower.contains("llama");
    if is_llama {
        let draft = candidates.iter().find(|(c_mid, c_name)| {
            let cm = c_mid.to_lowercase();
            let cn = c_name.to_lowercase();
            let is_c_llama = cm.contains("llama") || cn.contains("llama");
            let is_c_draft = cm.contains("1b") || cm.contains("1-b") || cm.contains("1_b")
                || cn.contains("1b") || cn.contains("1-b") || cn.contains("1_b")
                || cm.contains("draft") || cn.contains("draft");
            is_c_llama && is_c_draft
        });

        if let Some(d) = draft {
            return SpeculativeCompanionInfo {
                status: "PairedAvailable".to_string(),
                companion_id: Some(d.0.clone()),
                recommended_companion: Some("Llama-3.2-1B-Instruct".to_string()),
                reason: format!("Par rascunho compatível detectado: {}", d.1),
            };
        } else {
            return SpeculativeCompanionInfo {
                status: "CompanionMissing".to_string(),
                companion_id: None,
                recommended_companion: Some("Llama-3.2-1B-Instruct-Q4_K_M.gguf".to_string()),
                reason: "Requer par draft Llama-3.2-1B instalado (disponível no Hub)".to_string(),
            };
        }
    }

    SpeculativeCompanionInfo {
        status: "CompanionMissing".to_string(),
        companion_id: None,
        recommended_companion: None,
        reason: "Nenhum modelo rascunho compatível com o mesmo vocabulário encontrado no inventário.".to_string(),
    }
}

/// Resolves the speculative drafting eligibility and companion model for a given target model.
pub fn resolve_speculative_companion(
    model: &souls_llm_local_arena::domain::model::ModelDescriptor,
    all_models: &[souls_llm_local_arena::domain::model::ModelDescriptor],
) -> SpeculativeCompanionInfo {
    let candidates: Vec<(String, String)> = all_models
        .iter()
        .map(|m| (m.model_id.clone(), m.name.clone()))
        .collect();

    resolve_speculative_companion_parts(
        &model.model_id,
        &model.name,
        model.size_mb,
        model.has_mmproj,
        &candidates,
    )
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
        .iter()
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

            let latest_tpot_ms = latest_tps.map(|t| if t > 0.0 { (1000.0 / t).min(9999.0) } else { 0.0 });
            let is_vlm = m.has_mmproj || m.model_id.to_lowercase().contains("vl") || m.name.to_lowercase().contains("vl");
            let mut capabilities = vec!["chat".to_string()];
            if is_vlm {
                capabilities.push("vision".to_string());
            }
            let mid_lower = m.model_id.to_lowercase();
            if mid_lower.contains("coder") || mid_lower.contains("code") || mid_lower.contains("arch") {
                capabilities.push("code".to_string());
            }
            if mid_lower.contains("reasoning") || mid_lower.contains("cot") || mid_lower.contains("smollm") || mid_lower.contains("phi") {
                capabilities.push("reasoning".to_string());
            }

            let spec_info = resolve_speculative_companion(&m, &discovered);

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
                "mmproj_path": m.mmproj_path.as_ref().map(|p| p.to_string_lossy().to_string()),
                "status": status,
                "latest_tps": latest_tps,
                "latest_ttft_ms": latest_ttft_ms,
                "latest_tpot_ms": latest_tpot_ms,
                "latest_vram_peak_mb": latest_vram_peak_mb,
                "last_benchmarked": last_benchmarked,
                "e3_score": e3_score,
                "capabilities": capabilities,
                "recommended_silicon": if m.size_mb < 5120.0 { "dGPU NVIDIA (CUDA)" } else { "Host CPU (AVX2)" },
                "speculative_info": spec_info
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
    let target = discovered.iter().find(|m| {
        m.model_id.to_lowercase().contains(&pat)
            || m.file_path.to_string_lossy().to_lowercase().contains(&pat)
            || m.name.to_lowercase().contains(&pat)
    }).cloned();

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
    let vault_conn = runner.connect_vault().await;

    match b_type {
        1 => {
            // Type 1: HardwareProfile
            match runner.run_hardware_profile(&model, engine_type).await {
                Ok(res) => {
                    let mut persisted = false;
                    if let Ok(ref vault) = vault_conn {
                        if let Err(ve) = vault.record_hardware_profile(&res).await {
                            tracing::warn!("Failed to record hardware profile in vault: {}", ve);
                        } else {
                            persisted = true;
                            tracing::info!("HardwareProfile for '{}' successfully persisted in Heuristic Vault.", res.model_id);
                        }
                    }
                    Json(json!({
                        "status": "success",
                        "benchmark_type": "HardwareProfile",
                        "model_id": res.model_id,
                        "engine": res.engine,
                        "persisted": persisted,
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
                    }))
                }
                Err(e) => Json(json!({
                    "status": "error",
                    "message": format!("HardwareProfile benchmark failed: {}", e)
                })),
            }
        }
        2 => {
            // Type 2: CognitiveEval (Rust AST track)
            match runner.run_cognitive_eval(&model, engine_type, "2A_RustAst").await {
                Ok(res) => {
                    let mut persisted = false;
                    if let Ok(ref vault) = vault_conn {
                        if let Err(ve) = vault.record_cognitive_eval(&res).await {
                            tracing::warn!("Failed to record cognitive eval in vault: {}", ve);
                        } else {
                            persisted = true;
                            tracing::info!("CognitiveEval for '{}' successfully persisted in Heuristic Vault.", res.model_id);
                        }
                    }
                    Json(json!({
                        "status": "success",
                        "benchmark_type": "CognitiveEval",
                        "model_id": res.model_id,
                        "track": res.track,
                        "persisted": persisted,
                        "metrics": {
                            "accuracy_pct": res.accuracy_pct,
                            "avg_tps": res.avg_tps,
                            "avg_ttft_ms": res.avg_ttft_ms,
                            "e3_score": res.e3_score,
                            "verdict": res.status
                        }
                    }))
                }
                Err(e) => Json(json!({
                    "status": "error",
                    "message": format!("CognitiveEval benchmark failed: {}", e)
                })),
            }
        }
        3 => {
            // Type 3: MultimodalVision
            match runner.run_multimodal_vision(&model, engine_type).await {
                Ok(res) => {
                    let mut persisted = false;
                    if let Ok(ref vault) = vault_conn {
                        if let Err(ve) = vault.record_multimodal_vision(&res).await {
                            tracing::warn!("Failed to record multimodal vision in vault: {}", ve);
                        } else {
                            persisted = true;
                            tracing::info!("MultimodalVision for '{}' successfully persisted in Heuristic Vault.", res.model_id);
                        }
                    }
                    Json(json!({
                        "status": "success",
                        "benchmark_type": "MultimodalVision",
                        "model_id": res.model_id,
                        "persisted": persisted,
                        "metrics": {
                            "sidecar_path": res.sidecar_path,
                            "multimodal_ttft_ms": res.multimodal_ttft_ms,
                            "projection_latency_ms": res.projection_latency_ms,
                            "verdict": res.status
                        }
                    }))
                }
                Err(e) => Json(json!({
                    "status": "error",
                    "message": format!("MultimodalVision benchmark failed: {}", e)
                })),
            }
        }
        4 => {
            // Type 4: SpeculativeDraft
            let spec_info = resolve_speculative_companion(&model, &discovered);
            match spec_info.status.as_str() {
                "Sub1bExempt" => {
                    Json(json!({
                        "status": "error",
                        "message": format!("Modelo '{}' é sub-1B e está isento de aceleração especulativa: {}", model.model_id, spec_info.reason)
                    }))
                }
                "VisionExempt" => {
                    Json(json!({
                        "status": "error",
                        "message": format!("Modelo '{}' é multimodal (visão) e está isento de aceleração especulativa por texto: {}", model.model_id, spec_info.reason)
                    }))
                }
                "CompanionMissing" => {
                    Json(json!({
                        "status": "error",
                        "message": format!("Aceleração especulativa indisponível para '{}': {}", model.model_id, spec_info.reason)
                    }))
                }
                "PairedAvailable" => {
                    let drafter_id = spec_info.companion_id.as_deref().unwrap_or_default();
                    let drafter = discovered.iter().find(|m| m.model_id == drafter_id);

                    match runner.run_speculative_draft(&model, engine_type, drafter).await {
                        Ok(res) => {
                            let mut persisted = false;
                            if let Ok(ref vault) = vault_conn {
                                if let Err(ve) = vault.record_speculative_draft(&res).await {
                                    tracing::warn!("Failed to record speculative draft in vault: {}", ve);
                                } else {
                                    persisted = true;
                                    tracing::info!("SpeculativeDraft for '{}' with drafter '{:?}' successfully persisted in Heuristic Vault.", res.model_id, drafter_id);
                                }
                            }
                            Json(json!({
                                "status": "success",
                                "benchmark_type": "SpeculativeDraft",
                                "model_id": res.model_id,
                                "persisted": persisted,
                                "metrics": {
                                    "drafter_id": res.drafter_id,
                                    "draft_type": res.draft_type,
                                    "speculative_tps": res.speculative_tps,
                                    "speedup_ratio": res.speedup_ratio,
                                    "acceptance_rate_alpha": res.acceptance_rate_alpha,
                                    "verdict": res.verdict
                                }
                            }))
                        }
                        Err(e) => Json(json!({
                            "status": "error",
                            "message": format!("SpeculativeDraft benchmark failed: {}", e)
                        })),
                    }
                }
                "NativeMtp" => {
                    match runner.run_speculative_draft(&model, engine_type, None).await {
                        Ok(res) => {
                            let mut persisted = false;
                            if let Ok(ref vault) = vault_conn {
                                if let Err(ve) = vault.record_speculative_draft(&res).await {
                                    tracing::warn!("Failed to record speculative draft in vault: {}", ve);
                                } else {
                                    persisted = true;
                                    tracing::info!("SpeculativeDraft (Native MTP) for '{}' successfully persisted in Heuristic Vault.", res.model_id);
                                }
                            }
                            Json(json!({
                                "status": "success",
                                "benchmark_type": "SpeculativeDraft",
                                "model_id": res.model_id,
                                "persisted": persisted,
                                "metrics": {
                                    "drafter_id": res.drafter_id,
                                    "draft_type": res.draft_type,
                                    "speculative_tps": res.speculative_tps,
                                    "speedup_ratio": res.speedup_ratio,
                                    "acceptance_rate_alpha": res.acceptance_rate_alpha,
                                    "verdict": res.verdict
                                }
                            }))
                        }
                        Err(e) => Json(json!({
                            "status": "error",
                            "message": format!("SpeculativeDraft benchmark failed: {}", e)
                        })),
                    }
                }
                _ => Json(json!({
                    "status": "error",
                    "message": "Status de acelerador especulativo não suportado."
                })),
            }
        }
        5 => {
            // Type 5: ContextPressure
            match runner.run_context_pressure(&model, engine_type).await {
                Ok(res) => {
                    let mut persisted = false;
                    if let Ok(ref vault) = vault_conn {
                        if let Err(ve) = vault.record_context_pressure(&res).await {
                            tracing::warn!("Failed to record context pressure in vault: {}", ve);
                        } else {
                            persisted = true;
                            tracing::info!("ContextPressure for '{}' successfully persisted in Heuristic Vault.", res.model_id);
                        }
                    }
                    Json(json!({
                        "status": "success",
                        "benchmark_type": "ContextPressure",
                        "model_id": res.model_id,
                        "persisted": persisted,
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
                    }))
                }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_speculative_companion_vlm_vision_exempt() {
        let candidates = vec![
            ("qwen2.5-coder-0.5b-instruct".to_string(), "Qwen2.5-Coder-0.5B-Instruct".to_string()),
        ];

        // Case 1: has_mmproj is true
        let res1 = resolve_speculative_companion_parts(
            "custom-model",
            "Custom Model",
            2500.0,
            true,
            &candidates,
        );
        assert_eq!(res1.status, "VisionExempt");
        assert!(res1.reason.contains("multimodais de visão"));

        // Case 2: "vl" in name / mid
        let res2 = resolve_speculative_companion_parts(
            "qwen2-5-vl-3b",
            "Qwen2.5-VL-3B-Instruct",
            3000.0,
            false,
            &candidates,
        );
        assert_eq!(res2.status, "VisionExempt");

        // Case 3: "vision" in name / mid
        let res3 = resolve_speculative_companion_parts(
            "minicpm-vision",
            "MiniCPM Vision 2.6",
            5000.0,
            false,
            &candidates,
        );
        assert_eq!(res3.status, "VisionExempt");
    }

    #[test]
    fn test_resolve_speculative_companion_sub1b_variants() {
        let candidates = vec![];

        // Small size < 600MB
        let res1 = resolve_speculative_companion_parts(
            "some-model",
            "Some Model",
            500.0,
            false,
            &candidates,
        );
        assert_eq!(res1.status, "Sub1bExempt");

        // Explicit 0.5b, 0-5b, 0_5b with size >= 600MB
        for tag in &["0.5b", "0-5b", "0_5b", "135m", "360m"] {
            let res = resolve_speculative_companion_parts(
                &format!("test-model-{}", tag),
                &format!("Test Model {}", tag),
                800.0,
                false,
                &candidates,
            );
            assert_eq!(res.status, "Sub1bExempt", "Failed for tag: {}", tag);
        }
    }

    #[test]
    fn test_resolve_speculative_companion_qwen3_native_mtp_not_qwen25() {
        // Only Qwen 2.5 draft candidate available
        let candidates = vec![
            ("qwen2.5-coder-0.5b-instruct".to_string(), "Qwen2.5-Coder-0.5B-Instruct".to_string()),
        ];

        let res = resolve_speculative_companion_parts(
            "qwen3-5-4b-agentic-coder-v4",
            "Qwen3.5-4B-Agentic-Coder-V4",
            3000.0,
            false,
            &candidates,
        );
        // Must NOT pair with Qwen 2.5! Must return NativeMtp
        assert_eq!(res.status, "NativeMtp");
        assert_eq!(res.companion_id, Some("Embedded-MTP-Heads".to_string()));
        assert!(res.reason.contains("Native MTP"));

        // If a real Qwen 3 draft candidate is present, pair with it
        let candidates_with_q3 = vec![
            ("qwen2.5-coder-0.5b-instruct".to_string(), "Qwen2.5-Coder-0.5B-Instruct".to_string()),
            ("qwen3-draft-0.5b".to_string(), "Qwen3-Draft-0.5B-GGUF".to_string()),
        ];
        let res2 = resolve_speculative_companion_parts(
            "qwen3-5-4b-agentic-coder-v4",
            "Qwen3.5-4B-Agentic-Coder-V4",
            3000.0,
            false,
            &candidates_with_q3,
        );
        assert_eq!(res2.status, "PairedAvailable");
        assert_eq!(res2.companion_id, Some("qwen3-draft-0.5b".to_string()));
        assert!(res2.reason.contains("Qwen3-Draft-0.5B-GGUF"));
    }

    #[test]
    fn test_resolve_speculative_companion_qwen25_paired_with_hyphenated_draft() {
        let candidates = vec![
            ("qwen2-5-coder-0-5b-instruct-q4-k-m".to_string(), "Qwen2.5-Coder-0.5B-Instruct-Q4_K_M.gguf".to_string()),
        ];

        let res = resolve_speculative_companion_parts(
            "qwen2.5-coder-7b-instruct",
            "Qwen2.5-Coder-7B-Instruct",
            4500.0,
            false,
            &candidates,
        );
        assert_eq!(res.status, "PairedAvailable");
        assert_eq!(res.companion_id, Some("qwen2-5-coder-0-5b-instruct-q4-k-m".to_string()));
        assert!(res.reason.contains("Qwen2.5-Coder-0.5B-Instruct-Q4_K_M.gguf"));
    }
}
