//! # Handler para endpoints REST de Cloud Arena (`/api/v1/cloud/*`)
//!
//! Exposição de campeões de Pareto da nuvem e sincronização sob demanda.

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use souls_llm_cloud_arena::analytics::{
    get_profile_weights, normalize_agentic_score, normalize_coding_score, normalize_direct_pct,
    normalize_general_score,
};
use souls_llm_cloud_arena::TaskProfileKind;

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
            let champions_val = serde_json::to_value(champions).unwrap_or_else(|_| json!([]));
            let profiles_count = if let Value::Array(ref a) = champions_val { a.len() } else { 0 };

            Ok(Json(json!({
                "status": "success",
                "date": effective_date,
                "profiles_count": profiles_count,
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
    let limit = params.limit.unwrap_or(500).clamp(10, 600);

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
    let profile_kind = TaskProfileKind::from_str_lossy(&raw_profile).unwrap_or(TaskProfileKind::Executive);
    let canonical_profile = profile_kind.as_str();
    let (_, cost_weights) = get_profile_weights(profile_kind);

    let query_str = r#"
        SELECT 
            de.canonical_id,
            cm.display_name,
            cm.developer,
            cm.is_multimodal,
            cm.supports_tools,
            cm.supports_structured_outputs,
            cm.supports_caching,
            de.composite_score as score,
            de.task_blended_cost,
            de.roi_index,
            de.pareto_tag,
            cm.context_window,
            (SELECT COUNT(*) FROM pricing_telemetry_snapshots pts
             WHERE pts.canonical_id = de.canonical_id
               AND pts.source = 'nous_portal'
               AND pts.recorded_at = COALESCE(
                   (SELECT MAX(p2.recorded_at) FROM pricing_telemetry_snapshots p2 WHERE p2.source = 'nous_portal'),
                   pts.recorded_at
               )) as has_nous,
            (SELECT COUNT(*) FROM pricing_telemetry_snapshots pts
             WHERE pts.canonical_id = de.canonical_id
               AND pts.source = 'openrouter_market'
               AND pts.recorded_at = COALESCE(
                   (SELECT MAX(p2.recorded_at) FROM pricing_telemetry_snapshots p2 WHERE p2.source = 'openrouter_market'),
                   pts.recorded_at
               )) as has_openrouter,
            (SELECT prompt_price_per_m FROM pricing_telemetry_snapshots pts WHERE pts.canonical_id = de.canonical_id AND pts.source = 'nous_portal' ORDER BY pts.recorded_at DESC, pts.id DESC LIMIT 1) as nous_prompt_price,
            (SELECT completion_price_per_m FROM pricing_telemetry_snapshots pts WHERE pts.canonical_id = de.canonical_id AND pts.source = 'nous_portal' ORDER BY pts.recorded_at DESC, pts.id DESC LIMIT 1) as nous_completion_price,
            (SELECT cache_read_price_per_m FROM pricing_telemetry_snapshots pts WHERE pts.canonical_id = de.canonical_id AND pts.source = 'nous_portal' ORDER BY pts.recorded_at DESC, pts.id DESC LIMIT 1) as nous_cache_price,
            (SELECT prompt_price_per_m FROM pricing_telemetry_snapshots pts WHERE pts.canonical_id = de.canonical_id AND pts.source = 'openrouter_market' ORDER BY pts.recorded_at DESC, pts.id DESC LIMIT 1) as or_prompt_price,
            (SELECT completion_price_per_m FROM pricing_telemetry_snapshots pts WHERE pts.canonical_id = de.canonical_id AND pts.source = 'openrouter_market' ORDER BY pts.recorded_at DESC, pts.id DESC LIMIT 1) as or_completion_price,
            (SELECT cache_read_price_per_m FROM pricing_telemetry_snapshots pts WHERE pts.canonical_id = de.canonical_id AND pts.source = 'openrouter_market' ORDER BY pts.recorded_at DESC, pts.id DESC LIMIT 1) as or_cache_price,
            ev.eval_executive,
            ev.eval_coding,
            ev.eval_agentic,
            ev.eval_reasoning,
            ev.eval_fast,
            ev.eval_design,
            ev.eval_research,
            ev.eval_security,
            ev.eval_curation,
            bs.general_elo,
            bs.coding_elo,
            bs.bfcl_score,
            bs.ifeval_score,
            bs.gpqa_score,
            bs.hard_prompts_elo,
            bs.vqa_multimodal_score,
            (SELECT throughput_tps FROM pricing_telemetry_snapshots pts WHERE pts.canonical_id = de.canonical_id AND pts.source != 'nous_portal_batch' ORDER BY CASE WHEN pts.source = 'nous_portal' THEN 0 ELSE 1 END, pts.recorded_at DESC, pts.id DESC LIMIT 1) as tps,
            (SELECT ttft_ms FROM pricing_telemetry_snapshots pts WHERE pts.canonical_id = de.canonical_id AND pts.source != 'nous_portal_batch' AND pts.ttft_ms > 0 ORDER BY CASE WHEN pts.source = 'nous_portal' THEN 0 ELSE 1 END, pts.recorded_at DESC, pts.id DESC LIMIT 1) as ttft_ms
        FROM daily_evaluations de
        JOIN canonical_models cm ON cm.id = de.canonical_id
        LEFT JOIN benchmark_snapshots bs ON bs.id = (
            SELECT MAX(b2.id) FROM benchmark_snapshots b2 WHERE b2.canonical_id = de.canonical_id
        )
        LEFT JOIN (
            SELECT
                canonical_id,
                MAX(CASE WHEN profile_task = 'executive' THEN composite_score END) as eval_executive,
                MAX(CASE WHEN profile_task = 'coding' THEN composite_score END) as eval_coding,
                MAX(CASE WHEN profile_task = 'agentic' THEN composite_score END) as eval_agentic,
                MAX(CASE WHEN profile_task = 'reasoning' THEN composite_score END) as eval_reasoning,
                MAX(CASE WHEN profile_task = 'fast' THEN composite_score END) as eval_fast,
                MAX(CASE WHEN profile_task = 'design' THEN composite_score END) as eval_design,
                MAX(CASE WHEN profile_task = 'research' THEN composite_score END) as eval_research,
                MAX(CASE WHEN profile_task = 'security' THEN composite_score END) as eval_security,
                MAX(CASE WHEN profile_task = 'curation' THEN composite_score END) as eval_curation
            FROM daily_evaluations
            WHERE evaluation_date = (
                SELECT MAX(evaluation_date) FROM daily_evaluations WHERE profile_task = ?
            )
            GROUP BY canonical_id
        ) ev ON ev.canonical_id = de.canonical_id
        WHERE de.profile_task = ?
          AND de.evaluation_date = (
              SELECT MAX(evaluation_date)
              FROM daily_evaluations
              WHERE profile_task = ?
          )
          AND cm.context_window >= 4096
          AND LOWER(cm.id) NOT LIKE '%:batch'
          AND LOWER(cm.id) NOT LIKE '%embed%'
          AND LOWER(cm.id) NOT LIKE '%minilm%'
          AND LOWER(cm.id) NOT LIKE '%rerank%'
          AND LOWER(cm.id) NOT LIKE '%whisper%'
          AND LOWER(cm.id) NOT LIKE '%tts%'
        GROUP BY de.canonical_id
        ORDER BY CASE WHEN de.pareto_tag = 'Estimated' THEN 1 ELSE 0 END ASC, score DESC
        LIMIT ?;
    "#;

    let rows = sqlx::query(query_str)
        .bind(canonical_profile)
        .bind(canonical_profile)
        .bind(canonical_profile)
        .bind(limit)
        .fetch_all(&pool)
        .await;

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
        let pareto_tag: String = r.try_get("pareto_tag").unwrap_or_else(|_| "Dominated".to_string());
        let context_window: i64 = r.get("context_window");
        let developer: String = r.try_get("developer").unwrap_or_else(|_| "unknown".to_string());
        let is_multimodal: i64 = r.try_get("is_multimodal").unwrap_or(0);
        let supports_tools: i64 = r.try_get("supports_tools").unwrap_or(0);
        let supports_structured_outputs: i64 = r.try_get("supports_structured_outputs").unwrap_or(0);
        let supports_caching: i64 = r.try_get("supports_caching").unwrap_or(0);
        let has_nous: i64 = r.try_get("has_nous").unwrap_or(0);
        let has_openrouter: i64 = r.try_get("has_openrouter").unwrap_or(0);
        let nous_prompt_price: Option<f64> = r.try_get("nous_prompt_price").ok();
        let nous_completion_price: Option<f64> = r.try_get("nous_completion_price").ok();
        let nous_cache_price: Option<f64> = r.try_get("nous_cache_price").ok();
        let or_prompt_price: Option<f64> = r.try_get("or_prompt_price").ok();
        let or_completion_price: Option<f64> = r.try_get("or_completion_price").ok();
        let or_cache_price: Option<f64> = r.try_get("or_cache_price").ok();
        let eval_executive: Option<f64> = r.try_get("eval_executive").ok();
        let eval_coding: Option<f64> = r.try_get("eval_coding").ok();
        let eval_agentic: Option<f64> = r.try_get("eval_agentic").ok();
        let eval_reasoning: Option<f64> = r.try_get("eval_reasoning").ok();
        let eval_fast: Option<f64> = r.try_get("eval_fast").ok();
        let eval_design: Option<f64> = r.try_get("eval_design").ok();
        let eval_research: Option<f64> = r.try_get("eval_research").ok();
        let eval_security: Option<f64> = r.try_get("eval_security").ok();
        let eval_curation: Option<f64> = r.try_get("eval_curation").ok();
        let general_elo_raw: Option<f64> = r.try_get("general_elo").ok();
        let coding_elo_raw: Option<f64> = r.try_get("coding_elo").ok();
        let bfcl_score_raw: Option<f64> = r.try_get("bfcl_score").ok();
        let ifeval_score_raw: Option<f64> = r.try_get("ifeval_score").ok();
        let gpqa_score_raw: Option<f64> = r.try_get("gpqa_score").ok();
        let hard_prompts_raw: Option<f64> = r.try_get("hard_prompts_elo").ok();
        let vqa_score_raw: Option<f64> = r.try_get("vqa_multimodal_score").ok();
        let tps: Option<f64> = r.try_get("tps").ok();
        let ttft_ms: Option<f64> = r.try_get("ttft_ms").ok();

        // Normaliza todos os sub-scores para a escala unificada 0.0..100.0 antes de enviar à UI
        let general_elo = general_elo_raw.filter(|v| *v > 0.0).map(normalize_general_score);
        let coding_elo = coding_elo_raw.filter(|v| *v > 0.0).map(normalize_coding_score);
        let bfcl_score = bfcl_score_raw.filter(|v| *v > 0.0).map(normalize_agentic_score);
        let ifeval_score = ifeval_score_raw.filter(|v| *v > 0.0).map(normalize_direct_pct);
        let gpqa_score = gpqa_score_raw.filter(|v| *v > 0.0).map(normalize_direct_pct);
        let hard_prompts_elo = hard_prompts_raw.filter(|v| *v > 0.0).map(normalize_general_score);
        let vqa_score = vqa_score_raw.filter(|v| *v > 0.0).map(normalize_direct_pct);

        let is_free_id = canonical_id.ends_with(":free");
        let compute_blended = |prompt: Option<f64>, comp: Option<f64>, cache: Option<f64>, cache_ratio: f64| -> Option<f64> {
            match (prompt, comp) {
                (Some(p), Some(c)) => {
                    if is_free_id || (p <= 0.0 && c <= 0.0) {
                        Some(0.0)
                    } else {
                        let cr = cache.unwrap_or(p * cache_ratio);
                        Some((cost_weights.w_cache * cr + cost_weights.w_in * p + cost_weights.w_out * c).max(0.001))
                    }
                }
                _ => None,
            }
        };
        let nous_blended = compute_blended(nous_prompt_price, nous_completion_price, nous_cache_price, 0.20);
        let or_blended = compute_blended(or_prompt_price, or_completion_price, or_cache_price, 0.25);
        let effective_blended = if is_free_id {
            0.0
        } else if has_nous > 0 {
            nous_blended.unwrap_or(blended_cost)
        } else {
            or_blended.unwrap_or(blended_cost)
        };
        let is_free = is_free_id || effective_blended == 0.0;

        let mut available_gateways: Vec<String> = Vec::new();
        if has_nous > 0 {
            available_gateways.push("nous".to_string());
        }
        if has_openrouter > 0 {
            available_gateways.push("openrouter".to_string());
        }
        if let Some(direct_gw) = direct_gateway_for_model(&canonical_id, &developer) {
            if !available_gateways.contains(&direct_gw) {
                available_gateways.push(direct_gw);
            }
        }
        let primary_provider = if has_nous > 0 { "nous" } else { "openrouter" };

        models.push(json!({
            "rank": idx + 1,
            "canonical_id": canonical_id,
            "display_name": display_name,
            "developer": developer,
            "is_multimodal": is_multimodal > 0,
            "supports_tools": supports_tools > 0,
            "supports_structured_outputs": supports_structured_outputs > 0,
            "supports_caching": supports_caching > 0,
            "is_free": is_free,
            "composite_score": score,
            "blended_cost": effective_blended,
            "roi_index": roi_index,
            "pareto_tag": pareto_tag,
            "is_verified": pareto_tag != "Estimated",
            "context_window": context_window,
            "provider": primary_provider,
            "available_gateways": available_gateways,
            "has_nous": has_nous > 0,
            "has_openrouter": has_openrouter > 0,
            "nous_prompt_price": nous_prompt_price,
            "nous_completion_price": nous_completion_price,
            "nous_blended": nous_blended,
            "or_prompt_price": or_prompt_price,
            "or_completion_price": or_completion_price,
            "or_blended": or_blended,
            "eval_executive": eval_executive,
            "eval_coding": eval_coding,
            "eval_agentic": eval_agentic,
            "eval_reasoning": eval_reasoning,
            "eval_fast": eval_fast,
            "eval_design": eval_design,
            "eval_research": eval_research,
            "eval_security": eval_security,
            "eval_curation": eval_curation,
            "general_elo": general_elo,
            "coding_elo": coding_elo,
            "bfcl_score": bfcl_score,
            "ifeval_score": ifeval_score,
            "gpqa_score": gpqa_score,
            "hard_prompts_elo": hard_prompts_elo,
            "vqa_multimodal_score": vqa_score,
            "throughput_tps": tps,
            "ttft_ms": ttft_ms
        }));
    }

    Ok(Json(json!({
        "status": "success",
        "profile": canonical_profile,
        "count": models.len(),
        "models": models
    })))
}

fn direct_gateway_for_model(canonical_id: &str, developer: &str) -> Option<String> {
    let prefix = canonical_id
        .split('/')
        .next()
        .unwrap_or(developer)
        .to_ascii_lowercase();
    match prefix.as_str() {
        "anthropic" => Some("anthropic".to_string()),
        "openai" => Some("openai-api".to_string()),
        "google" => Some("gemini".to_string()),
        "deepseek" => Some("deepseek".to_string()),
        "z-ai" | "zai" => Some("zai".to_string()),
        "x-ai" | "xai" => Some("xai".to_string()),
        "qwen" | "alibaba" => Some("alibaba".to_string()),
        "xiaomi" => Some("xiaomi".to_string()),
        "minimax" => Some("minimax".to_string()),
        "stepfun" => Some("stepfun".to_string()),
        "upstage" => Some("upstage".to_string()),
        "moonshotai" => Some("kimi-for-coding".to_string()),
        "nvidia" => Some("nvidia".to_string()),
        _ => None,
    }
}

/// GET /api/v1/hermes/customs — Inspeciona autenticação, slugs, base_url, MoA e sessões presas no Hermes Desktop.
pub async fn hermes_customs_handler(State(state): State<AppState>) -> Json<Value> {
    let _guard = state.acquire_connection();
    let hermes_root = crate::profiles::resolve_hermes_root_dir(None);
    let report = crate::profiles::inspect_hermes_customs_with_options(&hermes_root, true).await;
    let mut val = serde_json::to_value(report).unwrap_or_else(|_| json!({}));
    if let Value::Object(ref mut m) = val {
        m.insert("status".to_string(), json!("success"));
    }
    Json(val)
}

#[derive(Debug, Deserialize)]
pub struct HermesRepairRequest {
    pub repair_configs: Option<bool>,
    pub unpin_sessions: Option<bool>,
    pub moa_enabled: Option<bool>,
    pub launch_login_provider: Option<String>,
}

/// POST /api/v1/hermes/customs/repair — Repara configurações inválidas, alterna MoA, destrava sessões ou inicia login interativo.
pub async fn hermes_customs_repair_handler(
    State(state): State<AppState>,
    Json(payload): Json<HermesRepairRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let _guard = state.acquire_connection();
    let hermes_root = crate::profiles::resolve_hermes_root_dir(None);

    let mut login_launched = false;
    let mut login_hint: Option<String> = None;
    if let Some(ref prov) = payload.launch_login_provider {
        let canonical = crate::profiles::normalize_hermes_provider_and_route(prov, "").provider;
        let cmd_str = format!("hermes login --provider {}", canonical);
        login_hint = Some(cmd_str.clone());
        let hermes_venv_bin = hermes_root
            .join("hermes-agent")
            .join(".venv")
            .join("Scripts")
            .join("hermes.exe");
        let binary = if hermes_venv_bin.exists() {
            hermes_venv_bin.to_string_lossy().to_string()
        } else {
            "hermes".to_string()
        };
        #[cfg(target_os = "windows")]
        {
            let ps_cmd = format!("& '{}' login --provider {}", binary.replace('\'', "''"), canonical);
            if std::process::Command::new("cmd")
                .args([
                    "/C",
                    "start",
                    "Hermes Portal Login",
                    "powershell",
                    "-NoExit",
                    "-Command",
                    &ps_cmd,
                ])
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
                .is_ok()
            {
                login_launched = true;
            }
        }
    }

    let do_repair = payload.repair_configs.unwrap_or(payload.launch_login_provider.is_none());
    let do_unpin = payload.unpin_sessions.unwrap_or(payload.launch_login_provider.is_none());
    match crate::profiles::repair_hermes_customs(
        &hermes_root,
        do_repair,
        do_unpin,
        payload.moa_enabled,
    )
    .await
    {
        Ok(mut res) => {
            let report = crate::profiles::inspect_hermes_customs_with_options(&hermes_root, true).await;
            if let Value::Object(ref mut m) = res {
                m.insert("login_launched".to_string(), json!(login_launched));
                if let Some(hint) = login_hint {
                    m.insert("login_command".to_string(), json!(hint));
                }
                m.insert("customs".to_string(), serde_json::to_value(report).unwrap_or_else(|_| json!({})));
            }
            Ok(Json(res))
        }
        Err(err) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "status": "error", "message": err })),
        )),
    }
}

async fn connect_ro_hermes_state_db(db_path: &Path) -> Option<(sqlx::SqlitePool, Option<PathBuf>)> {
    let opts = sqlx::sqlite::SqliteConnectOptions::new()
        .filename(db_path)
        .read_only(true)
        .create_if_missing(false)
        .busy_timeout(std::time::Duration::from_millis(2000));
    if let Ok(pool) = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(opts)
        .await
    {
        return Some((pool, None));
    }

    // Fallback: locked exclusively by another writer — snapshot to a temporary read-only copy
    let tmp_dir = std::env::temp_dir().join(format!(
        "souls_hermes_ro_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    let _ = std::fs::create_dir_all(&tmp_dir);
    let tmp_db = tmp_dir.join("state_ro.db");
    if std::fs::copy(db_path, &tmp_db).is_ok() {
        let wal = db_path.with_extension("db-wal");
        let shm = db_path.with_extension("db-shm");
        if wal.exists() {
            let _ = std::fs::copy(&wal, tmp_dir.join("state_ro.db-wal"));
        }
        if shm.exists() {
            let _ = std::fs::copy(&shm, tmp_dir.join("state_ro.db-shm"));
        }
        let tmp_opts = sqlx::sqlite::SqliteConnectOptions::new()
            .filename(&tmp_db)
            .read_only(true)
            .create_if_missing(false);
        if let Ok(pool) = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(tmp_opts)
            .await
        {
            return Some((pool, Some(tmp_dir)));
        }
    }
    let _ = std::fs::remove_dir_all(&tmp_dir);
    None
}

fn sanitize_hermes_billing_provider(raw: &str, model_id: &str) -> String {
    let low = raw.trim().to_ascii_lowercase();
    if low == "llamacpp" || low == "local_silicon" || low == "ollama" || low == "custom" {
        return "custom".to_string();
    }
    if low.is_empty() {
        if model_id.to_ascii_lowercase().ends_with(".gguf")
            || (!model_id.contains('/') && model_id.to_ascii_lowercase().contains("qwen"))
        {
            return "custom".to_string();
        }
        return "nous".to_string();
    }
    crate::profiles::normalize_hermes_provider_and_route(&low, model_id).provider
}

fn lookup_model_pricing(
    price_map: &HashMap<String, (f64, f64, f64)>,
    model_id: &str,
) -> Option<(f64, f64, f64)> {
    let key = model_id.trim().to_ascii_lowercase();
    if let Some(p) = price_map.get(&key) {
        return Some(*p);
    }
    if let Some(stripped) = key.strip_suffix(":exacto").or_else(|| key.strip_suffix(":nitro")) {
        if let Some(p) = price_map.get(stripped) {
            return Some(*p);
        }
    }
    None
}

/// Synchronizes Hermes Desktop `state.db` (`session_model_usage`) into `hermes_usage_ledger`
/// (inside `souls_heuristic_vault.db`) in strictly non-blocking read-only mode and returns FinOps aggregations.
pub async fn sync_and_summarize_hermes_finops(
    hermes_root: &Path,
    vault_path: &Path,
) -> Result<Value, String> {
    use sqlx::Row;

    if let Some(parent) = vault_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let vault_opts = sqlx::sqlite::SqliteConnectOptions::new()
        .filename(vault_path)
        .create_if_missing(true)
        .busy_timeout(std::time::Duration::from_millis(3000));
    let vault_pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(2)
        .connect_with(vault_opts)
        .await
        .map_err(|e| format!("Falha ao abrir vault FinOps: {e}"))?;

    if let Err(e) = souls_llm_cloud_arena::storage::migrations::run_migrations(&vault_pool).await {
        tracing::warn!("Migrações FinOps emitiram aviso: {e}");
    }

    // 1. Build latest pricing lookup map (Nous preferred over OpenRouter for non-zero entries)
    let mut price_map: HashMap<String, (f64, f64, f64)> = HashMap::new();
    if let Ok(price_rows) = sqlx::query(
        r#"
        SELECT LOWER(canonical_id) as mid, source, prompt_price_per_m, completion_price_per_m, cache_read_price_per_m
        FROM pricing_telemetry_snapshots
        WHERE source != 'nous_portal_batch'
        ORDER BY
            CASE WHEN source = 'nous_portal' THEN 0 WHEN source = 'openrouter_market' THEN 1 ELSE 2 END ASC,
            recorded_at DESC,
            id DESC
        "#,
    )
    .fetch_all(&vault_pool)
    .await
    {
        for row in price_rows {
            let mid: String = row.get("mid");
            let p: f64 = row.try_get("prompt_price_per_m").unwrap_or(0.0);
            let c: f64 = row.try_get("completion_price_per_m").unwrap_or(0.0);
            let cr: f64 = row.try_get::<Option<f64>, _>("cache_read_price_per_m").ok().flatten().unwrap_or(p * 0.20);
            price_map.entry(mid).or_insert((p, c, cr));
        }
    }

    // 2. Collect root state.db + profiles/*/state.db
    let mut dbs = Vec::new();
    let root_db = hermes_root.join("state.db");
    if root_db.exists() {
        dbs.push(("default".to_string(), root_db));
    }
    if let Ok(entries) = std::fs::read_dir(hermes_root.join("profiles")) {
        let mut subs: Vec<(String, PathBuf)> = entries
            .flatten()
            .filter_map(|e| {
                let p = e.path();
                if p.is_dir() {
                    let db = p.join("state.db");
                    if db.exists() {
                        let name = p.file_name()?.to_string_lossy().to_string();
                        return Some((name, db));
                    }
                }
                None
            })
            .collect();
        subs.sort_by(|a, b| a.0.cmp(&b.0));
        dbs.extend(subs);
    }

    let now_rfc = chrono::Utc::now().to_rfc3339();
    let mut synced_rows = 0usize;

    for (prof_label, db_path) in dbs {
        let Some((hpool, tmp_cleanup)) = connect_ro_hermes_state_db(&db_path).await else {
            continue;
        };

        let query_res = sqlx::query(
            r#"
            SELECT
                u.session_id,
                COALESCE(NULLIF(TRIM(s.profile_name), ''), ?) as profile_name,
                COALESCE(NULLIF(TRIM(s.source), ''), 'desktop') as source_channel,
                COALESCE(u.model, s.model, 'unknown') as model_id,
                COALESCE(u.billing_provider, s.billing_provider, '') as billing_provider,
                COALESCE(u.task, '') as task_kind,
                COALESCE(u.api_call_count, 0) as api_call_count,
                COALESCE(u.input_tokens, 0) as input_tokens,
                COALESCE(u.output_tokens, 0) as output_tokens,
                COALESCE(u.cache_read_tokens, 0) as cache_read_tokens,
                COALESCE(u.cache_write_tokens, 0) as cache_write_tokens,
                COALESCE(u.reasoning_tokens, 0) as reasoning_tokens,
                COALESCE(u.estimated_cost_usd, 0.0) as estimated_cost_usd,
                COALESCE(u.actual_cost_usd, 0.0) as actual_cost_usd,
                COALESCE(u.first_seen, s.started_at, 0.0) as first_seen,
                COALESCE(u.last_seen, s.last_activity_at, s.started_at, 0.0) as last_seen
            FROM session_model_usage u
            LEFT JOIN sessions s ON s.id = u.session_id
            "#,
        )
        .bind(&prof_label)
        .fetch_all(&hpool)
        .await;

        if let Ok(rows) = query_res {
            if let Ok(mut tx) = vault_pool.begin().await {
                for r in rows {
                    let session_id: String = r.get("session_id");
                    let profile_name: String = r.get("profile_name");
                    let source_channel: String = r.get("source_channel");
                    let model_id: String = r.get("model_id");
                    let raw_prov: String = r.get("billing_provider");
                    let task_kind: String = r.get("task_kind");
                    let api_call_count: i64 = r.get("api_call_count");
                    let input_tokens: i64 = r.get("input_tokens");
                    let output_tokens: i64 = r.get("output_tokens");
                    let cache_read_tokens: i64 = r.get("cache_read_tokens");
                    let cache_write_tokens: i64 = r.get("cache_write_tokens");
                    let reasoning_tokens: i64 = r.get("reasoning_tokens");
                    let est_cost: f64 = r.get("estimated_cost_usd");
                    let act_cost: f64 = r.get("actual_cost_usd");
                    let first_seen: f64 = r.get("first_seen");
                    let last_seen: f64 = r.get("last_seen");

                    let billing_provider = sanitize_hermes_billing_provider(&raw_prov, &model_id);
                    let usage_key = format!(
                        "{prof_label}::{session_id}::{model_id}::{billing_provider}::{task_kind}"
                    );
                    let is_local_or_free = billing_provider == "custom"
                        || model_id.to_ascii_lowercase().ends_with(":free");
                    let canonical_id = if price_map.contains_key(&model_id.to_ascii_lowercase()) {
                        Some(model_id.clone())
                    } else {
                        None
                    };
                    let reported_cost = if act_cost > 0.0 { act_cost } else { est_cost };

                    let (computed_cost_usd, cache_saved_usd) = if is_local_or_free {
                        (0.0, 0.0)
                    } else if let Some((p_rate, c_rate, cr_rate)) = lookup_model_pricing(&price_map, &model_id) {
                        let calc = (input_tokens as f64 * p_rate
                            + output_tokens as f64 * c_rate
                            + cache_read_tokens as f64 * cr_rate
                            + cache_write_tokens as f64 * (p_rate * 1.25))
                            / 1_000_000.0;
                        let saved = (cache_read_tokens as f64 * (p_rate - cr_rate).max(0.0))
                            / 1_000_000.0;
                        let final_cost = if calc > 0.0 { calc } else { reported_cost };
                        (final_cost, saved)
                    } else {
                        (reported_cost, 0.0)
                    };

                    let _ = sqlx::query(
                        r#"
                        INSERT INTO hermes_usage_ledger (
                            usage_key, session_id, profile_name, source_channel, model_id,
                            canonical_id, billing_provider, task_kind, api_call_count,
                            input_tokens, output_tokens, cache_read_tokens, cache_write_tokens,
                            reasoning_tokens, hermes_reported_cost_usd, computed_cost_usd,
                            cache_saved_usd, first_seen_ts, last_seen_ts, synced_at
                        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                        ON CONFLICT(usage_key) DO UPDATE SET
                            profile_name = excluded.profile_name,
                            source_channel = excluded.source_channel,
                            canonical_id = excluded.canonical_id,
                            api_call_count = excluded.api_call_count,
                            input_tokens = excluded.input_tokens,
                            output_tokens = excluded.output_tokens,
                            cache_read_tokens = excluded.cache_read_tokens,
                            cache_write_tokens = excluded.cache_write_tokens,
                            reasoning_tokens = excluded.reasoning_tokens,
                            hermes_reported_cost_usd = excluded.hermes_reported_cost_usd,
                            computed_cost_usd = excluded.computed_cost_usd,
                            cache_saved_usd = excluded.cache_saved_usd,
                            first_seen_ts = excluded.first_seen_ts,
                            last_seen_ts = excluded.last_seen_ts,
                            synced_at = excluded.synced_at
                        "#,
                    )
                    .bind(&usage_key)
                    .bind(&session_id)
                    .bind(&profile_name)
                    .bind(&source_channel)
                    .bind(&model_id)
                    .bind(&canonical_id)
                    .bind(&billing_provider)
                    .bind(&task_kind)
                    .bind(api_call_count)
                    .bind(input_tokens)
                    .bind(output_tokens)
                    .bind(cache_read_tokens)
                    .bind(cache_write_tokens)
                    .bind(reasoning_tokens)
                    .bind(reported_cost)
                    .bind(computed_cost_usd)
                    .bind(cache_saved_usd)
                    .bind(first_seen)
                    .bind(last_seen)
                    .bind(&now_rfc)
                    .execute(&mut *tx)
                    .await;
                    synced_rows += 1;
                }
                let _ = tx.commit().await;
            }
        }

        hpool.close().await;
        if let Some(tmp_dir) = tmp_cleanup {
            let _ = std::fs::remove_dir_all(tmp_dir);
        }
    }

    // 3. Aggregate periods from hermes_usage_ledger
    let now_epoch = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0);
    let cut_today = now_epoch - 86_400.0;
    let cut_7d = now_epoch - 7.0 * 86_400.0;
    let cut_30d = now_epoch - 30.0 * 86_400.0;

    let query_period = |cutoff: f64| {
        let pool = vault_pool.clone();
        async move {
            let row = sqlx::query(
                r#"
                SELECT
                    COUNT(DISTINCT session_id) as sessions,
                    COALESCE(SUM(api_call_count), 0) as calls,
                    COALESCE(SUM(input_tokens), 0) as input_tokens,
                    COALESCE(SUM(output_tokens), 0) as output_tokens,
                    COALESCE(SUM(cache_read_tokens), 0) as cache_read_tokens,
                    COALESCE(SUM(cache_write_tokens), 0) as cache_write_tokens,
                    COALESCE(SUM(computed_cost_usd), 0.0) as spend_usd,
                    COALESCE(SUM(cache_saved_usd), 0.0) as cache_saved_usd,
                    COALESCE(SUM(CASE WHEN billing_provider = 'custom' OR LOWER(model_id) LIKE '%:free' THEN input_tokens + output_tokens ELSE 0 END), 0) as zero_cost_tokens
                FROM hermes_usage_ledger
                WHERE last_seen_ts >= ?
                "#,
            )
            .bind(cutoff)
            .fetch_one(&pool)
            .await
            .ok();

            if let Some(r) = row {
                let in_t: i64 = r.get("input_tokens");
                let out_t: i64 = r.get("output_tokens");
                let cr_t: i64 = r.get("cache_read_tokens");
                let cw_t: i64 = r.get("cache_write_tokens");
                let total_tokens = in_t + out_t + cr_t + cw_t;
                let denom = (in_t + cr_t).max(1) as f64;
                let cache_hit_pct = (cr_t as f64 / denom) * 100.0;
                json!({
                    "sessions": r.get::<i64, _>("sessions"),
                    "calls": r.get::<i64, _>("calls"),
                    "input_tokens": in_t,
                    "output_tokens": out_t,
                    "cache_read_tokens": cr_t,
                    "cache_write_tokens": cw_t,
                    "total_tokens": total_tokens,
                    "spend_usd": r.get::<f64, _>("spend_usd"),
                    "cache_saved_usd": r.get::<f64, _>("cache_saved_usd"),
                    "cache_hit_pct": (cache_hit_pct * 10.0).round() / 10.0,
                    "zero_cost_tokens": r.get::<i64, _>("zero_cost_tokens")
                })
            } else {
                json!({
                    "sessions": 0, "calls": 0, "input_tokens": 0, "output_tokens": 0,
                    "cache_read_tokens": 0, "cache_write_tokens": 0, "total_tokens": 0,
                    "spend_usd": 0.0, "cache_saved_usd": 0.0, "cache_hit_pct": 0.0,
                    "zero_cost_tokens": 0
                })
            }
        }
    };

    let p_today = query_period(cut_today).await;
    let p_7d = query_period(cut_7d).await;
    let p_30d = query_period(cut_30d).await;
    let p_all = query_period(0.0).await;

    let spend_7d = p_7d.get("spend_usd").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let spend_30d = p_30d.get("spend_usd").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let projected_monthly_run_rate_usd = if spend_7d > 0.0 {
        (spend_7d / 7.0) * 30.0
    } else {
        spend_30d
    };

    // Baseline monthly token estimate for the modal cost simulator (uses 30d or extrapolated 7d volume)
    let t_7d = p_7d.get("total_tokens").and_then(|v| v.as_i64()).unwrap_or(0) as f64;
    let t_30d = p_30d.get("total_tokens").and_then(|v| v.as_i64()).unwrap_or(0) as f64;
    let projected_monthly_input_tokens = {
        let in7 = p_7d.get("input_tokens").and_then(|v| v.as_i64()).unwrap_or(0) as f64;
        let in30 = p_30d.get("input_tokens").and_then(|v| v.as_i64()).unwrap_or(0) as f64;
        if in7 > 0.0 { ((in7 / 7.0) * 30.0).max(in30) } else { in30.max(500_000.0) }
    };
    let projected_monthly_output_tokens = {
        let out7 = p_7d.get("output_tokens").and_then(|v| v.as_i64()).unwrap_or(0) as f64;
        let out30 = p_30d.get("output_tokens").and_then(|v| v.as_i64()).unwrap_or(0) as f64;
        if out7 > 0.0 { ((out7 / 7.0) * 30.0).max(out30) } else { out30.max(150_000.0) }
    };
    let projected_monthly_cache_read_tokens = {
        let cr7 = p_7d.get("cache_read_tokens").and_then(|v| v.as_i64()).unwrap_or(0) as f64;
        let cr30 = p_30d.get("cache_read_tokens").and_then(|v| v.as_i64()).unwrap_or(0) as f64;
        if cr7 > 0.0 { ((cr7 / 7.0) * 30.0).max(cr30) } else { cr30 }
    };
    let monthly_tokens_estimate = if t_7d > 0.0 { ((t_7d / 7.0) * 30.0).max(t_30d) } else { t_30d };

    let mut by_gateway = Vec::new();
    if let Ok(rows) = sqlx::query(
        r#"
        SELECT billing_provider, COUNT(DISTINCT session_id) as sessions, COALESCE(SUM(api_call_count), 0) as calls,
               COALESCE(SUM(input_tokens + output_tokens + cache_read_tokens), 0) as tokens,
               COALESCE(SUM(computed_cost_usd), 0.0) as spend_usd,
               COALESCE(SUM(cache_saved_usd), 0.0) as cache_saved_usd
        FROM hermes_usage_ledger
        GROUP BY billing_provider
        ORDER BY spend_usd DESC, tokens DESC
        "#,
    )
    .fetch_all(&vault_pool)
    .await
    {
        for r in rows {
            by_gateway.push(json!({
                "gateway": r.get::<String, _>("billing_provider"),
                "sessions": r.get::<i64, _>("sessions"),
                "calls": r.get::<i64, _>("calls"),
                "tokens": r.get::<i64, _>("tokens"),
                "spend_usd": r.get::<f64, _>("spend_usd"),
                "cache_saved_usd": r.get::<f64, _>("cache_saved_usd")
            }));
        }
    }

    let mut by_model = Vec::new();
    if let Ok(rows) = sqlx::query(
        r#"
        SELECT model_id, billing_provider, COUNT(DISTINCT session_id) as sessions,
               COALESCE(SUM(api_call_count), 0) as calls,
               COALESCE(SUM(input_tokens), 0) as input_tokens,
               COALESCE(SUM(output_tokens), 0) as output_tokens,
               COALESCE(SUM(cache_read_tokens), 0) as cache_read_tokens,
               COALESCE(SUM(computed_cost_usd), 0.0) as spend_usd,
               COALESCE(SUM(cache_saved_usd), 0.0) as cache_saved_usd,
               MAX(last_seen_ts) as last_seen_ts
        FROM hermes_usage_ledger
        GROUP BY model_id, billing_provider
        ORDER BY spend_usd DESC, (input_tokens + output_tokens + cache_read_tokens) DESC
        LIMIT 25
        "#,
    )
    .fetch_all(&vault_pool)
    .await
    {
        for r in rows {
            by_model.push(json!({
                "model_id": r.get::<String, _>("model_id"),
                "billing_provider": r.get::<String, _>("billing_provider"),
                "sessions": r.get::<i64, _>("sessions"),
                "calls": r.get::<i64, _>("calls"),
                "input_tokens": r.get::<i64, _>("input_tokens"),
                "output_tokens": r.get::<i64, _>("output_tokens"),
                "cache_read_tokens": r.get::<i64, _>("cache_read_tokens"),
                "spend_usd": r.get::<f64, _>("spend_usd"),
                "cache_saved_usd": r.get::<f64, _>("cache_saved_usd"),
                "last_seen_ts": r.get::<f64, _>("last_seen_ts")
            }));
        }
    }

    let mut by_profile = Vec::new();
    if let Ok(rows) = sqlx::query(
        r#"
        SELECT profile_name, COUNT(DISTINCT session_id) as sessions,
               COALESCE(SUM(api_call_count), 0) as calls,
               COALESCE(SUM(input_tokens + output_tokens + cache_read_tokens), 0) as tokens,
               COALESCE(SUM(computed_cost_usd), 0.0) as spend_usd
        FROM hermes_usage_ledger
        GROUP BY profile_name
        ORDER BY spend_usd DESC, tokens DESC
        "#,
    )
    .fetch_all(&vault_pool)
    .await
    {
        for r in rows {
            by_profile.push(json!({
                "profile_name": r.get::<String, _>("profile_name"),
                "sessions": r.get::<i64, _>("sessions"),
                "calls": r.get::<i64, _>("calls"),
                "tokens": r.get::<i64, _>("tokens"),
                "spend_usd": r.get::<f64, _>("spend_usd")
            }));
        }
    }

    let mut recent_sessions = Vec::new();
    if let Ok(rows) = sqlx::query(
        r#"
        SELECT session_id, profile_name, source_channel, model_id, billing_provider, task_kind,
               api_call_count, input_tokens, output_tokens, cache_read_tokens,
               computed_cost_usd, cache_saved_usd, last_seen_ts
        FROM hermes_usage_ledger
        ORDER BY last_seen_ts DESC
        LIMIT 20
        "#,
    )
    .fetch_all(&vault_pool)
    .await
    {
        for r in rows {
            recent_sessions.push(json!({
                "session_id": r.get::<String, _>("session_id"),
                "profile_name": r.get::<String, _>("profile_name"),
                "source_channel": r.get::<String, _>("source_channel"),
                "model_id": r.get::<String, _>("model_id"),
                "billing_provider": r.get::<String, _>("billing_provider"),
                "task_kind": r.get::<String, _>("task_kind"),
                "calls": r.get::<i64, _>("api_call_count"),
                "input_tokens": r.get::<i64, _>("input_tokens"),
                "output_tokens": r.get::<i64, _>("output_tokens"),
                "cache_read_tokens": r.get::<i64, _>("cache_read_tokens"),
                "cost_usd": r.get::<f64, _>("computed_cost_usd"),
                "cache_saved_usd": r.get::<f64, _>("cache_saved_usd"),
                "last_seen_ts": r.get::<f64, _>("last_seen_ts")
            }));
        }
    }

    vault_pool.close().await;

    Ok(json!({
        "status": "success",
        "synced_rows": synced_rows,
        "periods": {
            "today": p_today,
            "last_7d": p_7d,
            "last_30d": p_30d,
            "all_time": p_all,
            "projected_monthly_run_rate_usd": projected_monthly_run_rate_usd,
            "monthly_tokens_estimate": monthly_tokens_estimate,
            "monthly_input_tokens_estimate": projected_monthly_input_tokens,
            "monthly_output_tokens_estimate": projected_monthly_output_tokens,
            "monthly_cache_read_tokens_estimate": projected_monthly_cache_read_tokens
        },
        "by_gateway": by_gateway,
        "by_model": by_model,
        "by_profile": by_profile,
        "recent_sessions": recent_sessions
    }))
}

/// GET /api/v1/cloud/finops — Sincroniza histórico real do Hermes (`state.db`) e retorna visão FinOps atual e projetada.
pub async fn cloud_finops_handler(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let _guard = state.acquire_connection();
    let hermes_root = crate::profiles::resolve_hermes_root_dir(None);
    let vault_path = souls_llm_cloud_arena::resolve_heuristic_vault_path();
    match sync_and_summarize_hermes_finops(&hermes_root, &vault_path).await {
        Ok(res) => Ok(Json(res)),
        Err(err) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "status": "error", "message": err })),
        )),
    }
}

