//! REST and SSE endpoints for `souls_llm_inventory`.
//!
//! Exposes hardware diagnostics, model recommendations, Hugging Face search,
//! sequential download queue control, and live SSE download progress.

use axum::extract::{Path as AxumPath, Query, State};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::convert::Infallible;
use std::time::Duration;
use tokio::task::JoinSet;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::{Stream, StreamExt};

use crate::state::AppState;
use souls_llm_inventory::{
    build_dynamic_hf_catalog_item, delete_model_file, estimate_variant_throughput_tps,
    get_curated_catalog_from_models, get_recommended_triad_from_models, probe_hardware,
    resolve_canonical_models_dir, scan_local_models, HardwareDiagnostic, HardwareFit, HfModelFile,
    HfSearchResult, HuggingFaceClient, HuggingFaceModelItem,
};

fn derive_gpu_architecture(gpu_name: &str) -> &'static str {
    let upper = gpu_name.to_uppercase();
    if upper.contains("SEM DGPU") || upper.contains("CPU HOST") || upper.contains("NONE (CPU") {
        "Host CPU / iGPU"
    } else if upper.contains("LOVELACE")
        || upper.contains("ADA GENERATION")
        || upper.contains("RTX 40")
        || upper.split_whitespace().any(|w| w.trim_matches(|c: char| !c.is_alphanumeric()) == "ADA")
    {
        "Ada Lovelace"
    } else if upper.contains("BLACKWELL") || upper.contains("RTX 50") || upper.contains("B200") || upper.contains("B100") {
        "Blackwell"
    } else if upper.contains("RTX 30") || upper.contains("AMPERE") {
        "Ampere"
    } else if upper.contains("RTX 20") || upper.contains("GTX 16") || upper.contains("TURING") {
        "Turing"
    } else if upper.contains("VOLTA") || upper.contains("TITAN V") {
        "Volta"
    } else if upper.contains("PASCAL") || upper.contains("GTX 10") {
        "Pascal"
    } else if upper.contains("RTX") || upper.contains("GTX") || upper.contains("NVIDIA") || upper.contains("CUDA") {
        "NVIDIA CUDA"
    } else if upper.contains("RADEON") || upper.contains("AMD") {
        "AMD ROCm / RDNA"
    } else if upper.contains("APPLE") || upper.contains("M1") || upper.contains("M2") || upper.contains("M3") || upper.contains("M4") {
        "Apple Silicon"
    } else {
        "Host CPU / iGPU"
    }
}

fn enrich_hardware_diagnostic(diag: souls_llm_inventory::HardwareDiagnostic) -> Value {
    let mut val = serde_json::to_value(&diag).unwrap_or_else(|_| json!({}));
    if let Some(obj) = val.as_object_mut() {
        let (_usable_gb, weights_gb, kv_gb, ceiling_tokens, ceiling_sub, formula_sub) = if diag.has_dedicated_gpu && diag.vram_safe_usable_mb > 0 {
            let usable_mb = diag.vram_safe_usable_mb as f64;
            let usable_gb = usable_mb / 1024.0;
            // Alocação física realista de VRAM: ~72% para pesos do modelo e ~28% para KV-Cache
            let weights = ((usable_gb * 0.72) * 10.0).round() / 10.0;
            let kv = ((usable_gb - weights) * 10.0).round() / 10.0;

            let (tokens, sub, desc) = if usable_mb >= 11_000.0 {
                (32_768, "(até 64k c/ KV Q4)", format!("Baseado em {:.2} GB úteis: seguro p/ 7B/14B FP16; subagentes em fila.", usable_gb))
            } else if usable_mb >= 7_000.0 {
                (16_384, "(até 32k c/ KV Q4)", format!("Baseado em {:.2} GB úteis: seguro p/ 7B (8k FP16 / 16k Q4); subagentes em fila.", usable_gb))
            } else if usable_mb >= 4_500.0 {
                // dGPU intermediária (~5.17 GB úteis sob WDDM)
                (16_384, "(até 32k c/ KV Q4)", format!("Baseado em {:.2} GB úteis: seguro p/ 3B (16k-32k) e 7B c/ KV Q4 (4k-8k FP16). Sessões curtas evitam spillover PCIe.", usable_gb))
            } else {
                (8_192, "(até 16k c/ KV Q4)", format!("Baseado em {:.2} GB úteis: recomendado 3B Q4_K_M com sessões enxutas.", usable_gb))
            };
            (usable_gb, weights, kv, tokens, sub, desc)
        } else {
            let usable_gb = (diag.host_ram_safe_usable_mb as f64) / 1024.0;
            (usable_gb, 4.0, 2.0, 8_192, "(AVX2 Offload)", "Execução via CPU Host AVX2. Sessões contidas para manter latência aceitável.".to_string())
        };

        let (gpu_arch, gpu_cc, gpu_details) = if diag.has_dedicated_gpu {
            let arch = derive_gpu_architecture(&diag.gpu_name);
            let (cc, details) = match arch {
                "Blackwell" => ("10.0", "NVIDIA Blackwell GPU • 5th Gen Tensor Cores"),
                "Ada Lovelace" => ("8.9", "NVIDIA Ada GPU • 4th Gen Tensor Cores"),
                "Ampere" => ("8.6", "NVIDIA Ampere GPU • 3rd Gen Tensor Cores"),
                "Turing" => ("7.5", "NVIDIA Turing GPU • 2nd Gen Tensor Cores"),
                "Volta" => ("7.0", "NVIDIA Volta GPU • 1st Gen Tensor Cores"),
                "Pascal" => ("6.1", "NVIDIA Pascal GPU • CUDA Cores"),
                "NVIDIA CUDA" => ("CUDA", "NVIDIA CUDA Acceleration"),
                "AMD ROCm / RDNA" => ("RDNA", "AMD ROCm Acceleration"),
                "Apple Silicon" => ("Metal", "Apple Silicon Unified Memory / Metal"),
                _ => ("Generic", "GPU Compute Acceleration"),
            };
            (arch, cc, details)
        } else {
            ("Host CPU / iGPU", "N/A", "Host Execution")
        };

        obj.insert("gpu_architecture".to_string(), json!(gpu_arch));
        obj.insert("gpu_compute_capability".to_string(), json!(gpu_cc));
        obj.insert("gpu_details".to_string(), json!(gpu_details));

        let base_cpu = if diag.cpu_name.trim().is_empty() { "Host CPU" } else { diag.cpu_name.trim() };
        obj.insert("cpu_name".to_string(), json!(format!("{} ({} Cores / {} Threads)", base_cpu, diag.cpu_cores_physical, diag.cpu_cores_logical)));

        let cpu_instructions_extended = if diag.has_avx512 {
            "AVX-512 • AVX2 • FMA • BMI2 • SSE4.2 • AES-NI"
        } else if diag.has_avx2 {
            "AVX2 • FMA • BMI2 • SSE4.2 • AES-NI"
        } else {
            "x86-64 • SSE4.2 • Basic Vector Extensions"
        };
        obj.insert("cpu_instructions_extended".to_string(), json!(cpu_instructions_extended));
        obj.insert("weights_budget_gb".to_string(), json!(weights_gb));
        obj.insert("kv_cache_budget_gb".to_string(), json!(kv_gb));
        obj.insert("context_ceiling_tokens".to_string(), json!(ceiling_tokens));
        obj.insert("context_ceiling_display".to_string(), json!(format!("{}.{:03} tokens", ceiling_tokens / 1000, ceiling_tokens % 1000)));
        obj.insert("context_ceiling_sub".to_string(), json!(ceiling_sub));
        obj.insert("context_formula_subtitle".to_string(), json!(formula_sub));
    }
    val
}

/// Hardware diagnostic endpoint.
pub async fn inventory_hardware_handler(State(state): State<AppState>) -> Json<Value> {
    let _guard = state.acquire_connection();
    let diag = probe_hardware();
    Json(json!({
        "status": "success",
        "hardware": enrich_hardware_diagnostic(diag)
    }))
}

#[derive(Debug, Default, Deserialize)]
pub struct RecommendationsQuery {
    pub sync: Option<bool>,
    pub mode: Option<String>,
}

fn is_elite_hf_author(repo_id: &str) -> bool {
    let org = repo_id
        .split('/')
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();
    matches!(
        org.as_str(),
        "qwen"
            | "bartowski"
            | "unsloth"
            | "google"
            | "meta-llama"
            | "microsoft"
            | "nousresearch"
            | "deepseek-ai"
            | "nvidia"
            | "huggingfacetb"
            | "mistralai"
            | "lmstudio-community"
            | "ggml-org"
    )
}

async fn fetch_repo_files_batch(
    hf_client: &HuggingFaceClient,
    repo_ids: &[String],
) -> HashMap<String, Vec<HfModelFile>> {
    let mut set = JoinSet::new();
    for repo in repo_ids.iter().take(10) {
        let client = hf_client.clone();
        let repo_id = repo.clone();
        set.spawn(async move {
            let res = tokio::time::timeout(
                Duration::from_secs(6),
                client.get_repo_gguf_files(&repo_id),
            )
            .await;
            let files = res.ok().and_then(|r| r.ok()).unwrap_or_default();
            (repo_id, files)
        });
    }

    let mut map = HashMap::new();
    while let Some(joined) = set.join_next().await {
        if let Ok((repo_id, files)) = joined {
            if !files.is_empty() {
                map.insert(repo_id.to_ascii_lowercase(), files);
            }
        }
    }
    map
}

/// Deduplicates raw HF search items and builds dynamic catalog items with real file trees for top hits.
async fn hydrate_hf_catalog_items(
    hf_client: &HuggingFaceClient,
    diag: &HardwareDiagnostic,
    raw_items: Vec<HfSearchResult>,
    installed_filenames: &[String],
    is_trending: bool,
    exclude_repo_ids: &HashSet<String>,
    max_items: usize,
) -> Vec<HuggingFaceModelItem> {
    let mut seen = HashSet::new();
    let mut filtered_raw = Vec::new();
    for r in raw_items {
        let key = r.repo_id.to_ascii_lowercase();
        if exclude_repo_ids.contains(&key) || !seen.insert(key) {
            continue;
        }
        if r.downloads < 25 && r.likes < 2 {
            continue;
        }
        filtered_raw.push(r);
        if filtered_raw.len() >= max_items * 2 {
            break;
        }
    }

    // Prioritize elite authors for sync mode while keeping trending order for trending mode
    if !is_trending {
        filtered_raw.sort_by(|a, b| {
            let ea = is_elite_hf_author(&a.repo_id);
            let eb = is_elite_hf_author(&b.repo_id);
            eb.cmp(&ea).then_with(|| b.downloads.cmp(&a.downloads))
        });
    }

    let preselected: Vec<HfSearchResult> = filtered_raw
        .into_iter()
        .filter(|r| {
            build_dynamic_hf_catalog_item(diag, r, installed_filenames, is_trending, None)
                .map(|m| m.parameter_count <= 45_000_000_000 || m.is_moe)
                .unwrap_or(false)
        })
        .take(max_items)
        .collect();

    let repo_ids: Vec<String> = preselected.iter().map(|r| r.repo_id.clone()).collect();
    let files_map = fetch_repo_files_batch(hf_client, &repo_ids).await;

    preselected
        .into_iter()
        .filter_map(|r| {
            let real_files = files_map
                .get(&r.repo_id.to_ascii_lowercase())
                .map(|v| v.as_slice());
            build_dynamic_hf_catalog_item(diag, &r, installed_filenames, is_trending, real_files)
        })
        .collect()
}

/// Curated recommendations, evolutionary 3-tier triad, and optional live HF sync / trending feed.
pub async fn inventory_recommendations_handler(
    State(state): State<AppState>,
    Query(params): Query<RecommendationsQuery>,
) -> Json<Value> {
    let _guard = state.acquire_connection();
    let diag = probe_hardware();
    let root = resolve_canonical_models_dir();

    let installed_models = scan_local_models(&root, &diag).unwrap_or_default();
    let installed_filenames: Vec<String> = installed_models
        .iter()
        .map(|m| m.file_name.clone())
        .collect();

    let triad = get_recommended_triad_from_models(&diag, &installed_models);
    let mut catalog = get_curated_catalog_from_models(&diag, &installed_models);
    let mut existing_repos: HashSet<String> = catalog
        .iter()
        .map(|m| m.repo_id.to_ascii_lowercase())
        .collect();

    let mut synced_live = false;
    if params.sync == Some(true) {
        let (qwen3, r1, gemma3, llama32) = tokio::join!(
            state.hf_client.search_models_ext("Qwen3 GGUF", "downloads", 10),
            state.hf_client.search_models_ext("DeepSeek-R1 GGUF", "downloads", 8),
            state.hf_client.search_models_ext("gemma-3 GGUF", "downloads", 8),
            state.hf_client.search_models_ext("Llama-3.2 GGUF", "downloads", 8),
        );
        let mut combined_raw = Vec::new();
        for res in [qwen3, r1, gemma3, llama32] {
            if let Ok(items) = res {
                combined_raw.extend(items);
            }
        }
        let discovered = hydrate_hf_catalog_items(
            &state.hf_client,
            &diag,
            combined_raw,
            &installed_filenames,
            false,
            &existing_repos,
            10,
        )
        .await;
        if !discovered.is_empty() {
            synced_live = true;
            for item in discovered {
                existing_repos.insert(item.repo_id.to_ascii_lowercase());
                catalog.push(item);
            }
        }
    }

    let mut trending_catalog = Vec::new();
    if params.mode.as_deref() == Some("trending") {
        let (trend_res, recent_qwen, recent_r1) = tokio::join!(
            state.hf_client.search_models_ext("", "trending", 24),
            state.hf_client.search_models_ext("Qwen3", "recent", 8),
            state.hf_client.search_models_ext("DeepSeek", "recent", 6),
        );
        let mut raw_trending = Vec::new();
        for res in [trend_res, recent_qwen, recent_r1] {
            if let Ok(items) = res {
                raw_trending.extend(items);
            }
        }
        let empty_set = HashSet::new();
        trending_catalog = hydrate_hf_catalog_items(
            &state.hf_client,
            &diag,
            raw_trending,
            &installed_filenames,
            true,
            &empty_set,
            16,
        )
        .await;
    }

    Json(json!({
        "status": "success",
        "hardware": enrich_hardware_diagnostic(diag),
        "triad": triad,
        "catalog": catalog,
        "trending_catalog": trending_catalog,
        "synced_live": synced_live,
        "installed_count": installed_filenames.len()
    }))
}

/// Locally installed physical models with zero-copy metadata and hardware fit.
pub async fn inventory_models_handler(State(state): State<AppState>) -> Json<Value> {
    let _guard = state.acquire_connection();
    let diag = probe_hardware();
    let root = resolve_canonical_models_dir();

    let models = scan_local_models(&root, &diag).unwrap_or_default();

    Json(json!({
        "status": "success",
        "root_dir": root.display().to_string(),
        "total_models": models.len(),
        "models": models
    }))
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
    pub query: Option<String>,
    pub param_filter: Option<String>,
    pub task_filter: Option<String>,
    pub sort: Option<String>,
    pub limit: Option<usize>,
    pub autocomplete: Option<bool>,
}

fn matches_search_filters(
    item: &HuggingFaceModelItem,
    raw: &HfSearchResult,
    param_filter: &str,
    task_filter: &str,
) -> bool {
    let t = item.parameter_count as f64 / 1_000_000_000.0;
    let param_ok = match param_filter {
        "all" | "" => true,
        "fit_gpu" => item
            .variants
            .iter()
            .any(|v| v.hardware_fit == HardwareFit::OptimalVram),
        "sub1b" => t < 1.0 && !item.is_moe,
        "1b" => (1.0..2.0).contains(&t) && !item.is_moe,
        "2b" => (2.0..3.0).contains(&t) && !item.is_moe,
        "3b" => (3.0..4.0).contains(&t) && !item.is_moe,
        "4b" => (4.0..5.0).contains(&t) && !item.is_moe,
        "5b" => (5.0..6.0).contains(&t),
        "6b" => (6.0..7.0).contains(&t),
        "7b" => (7.0..8.0).contains(&t),
        "8b" => (8.0..9.0).contains(&t),
        "9b" => (9.0..10.0).contains(&t),
        "10b_12b" => (10.0..=12.5).contains(&t),
        "14b" => t > 12.5 && t <= 16.0,
        "32b" => t > 16.0 && t <= 45.0,
        "70b_plus" => t > 45.0,
        _ => true,
    };
    if !param_ok {
        return false;
    }

    let r = raw.repo_id.to_ascii_lowercase();
    let tags_joined = raw.tags.join(" ").to_ascii_lowercase();
    match task_filter {
        "all" | "" => true,
        "coding" => {
            item.cognitive_profile == "coding"
                || r.contains("code")
                || r.contains("coder")
                || tags_joined.contains("code")
        }
        "reasoning" => {
            item.cognitive_profile == "reasoning"
                || r.contains("r1")
                || r.contains("reason")
                || r.contains("think")
                || r.contains("qwq")
                || tags_joined.contains("reasoning")
        }
        "agentic" | "tools" => {
            item.cognitive_profile == "agentic"
                || r.contains("agent")
                || r.contains("tool")
                || r.contains("xlam")
                || r.contains("hermes")
                || tags_joined.contains("function-calling")
        }
        "vision" => item.is_multimodal_vlm || item.cognitive_profile == "vision",
        "fast" => item.parameter_count <= 2_000_000_000,
        "general" | "chat" => {
            r.contains("instruct")
                || r.contains("chat")
                || tags_joined.contains("conversational")
                || item.cognitive_profile == "agentic"
        }
        _ => true,
    }
}

fn build_hf_search_query(raw_q: &str, param_filter: &str, task_filter: &str) -> String {
    let clean_q = raw_q.trim();
    if !clean_q.is_empty() {
        return clean_q.to_string();
    }

    let mut parts: Vec<&str> = Vec::new();
    match task_filter {
        "coding" => parts.push("coder"),
        "reasoning" => parts.push("R1 reasoning"),
        "agentic" | "tools" => parts.push("function calling"),
        "vision" => parts.push("VL vision"),
        "fast" => parts.push("1B instruct"),
        "general" | "chat" => parts.push("Instruct"),
        _ => {}
    }
    match param_filter {
        "sub1b" => parts.push("0.5B"),
        "1b" => parts.push("1B"),
        "2b" => parts.push("2B"),
        "3b" => parts.push("3B"),
        "4b" => parts.push("4B"),
        "5b" => parts.push("5B"),
        "6b" => parts.push("6B"),
        "7b" => parts.push("7B"),
        "8b" => parts.push("8B"),
        "9b" => parts.push("9B"),
        "10b_12b" => parts.push("12B"),
        "14b" => parts.push("14B"),
        "32b" => parts.push("32B"),
        "70b_plus" => parts.push("70B"),
        "fit_gpu" => {
            if parts.is_empty() {
                parts.push("Qwen3");
            }
        }
        _ => {}
    }

    if parts.is_empty() {
        "Instruct".to_string()
    } else {
        parts.join(" ")
    }
}

/// Combined live search against the Hugging Face Hub with hardware-fit and cognitive enrichment.
pub async fn inventory_search_handler(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> Json<Value> {
    let _guard = state.acquire_connection();
    let search_term = query.q.or(query.query).unwrap_or_default();
    let param_filter = query
        .param_filter
        .as_deref()
        .unwrap_or("all")
        .trim()
        .to_ascii_lowercase();
    let task_filter = query
        .task_filter
        .as_deref()
        .unwrap_or("all")
        .trim()
        .to_ascii_lowercase();
    let sort_mode = query.sort.as_deref().unwrap_or("downloads");
    let limit = query.limit.unwrap_or(24).clamp(1, 50);

    if search_term.trim().is_empty() && param_filter == "all" && task_filter == "all" {
        return Json(json!({
            "status": "success",
            "query": "",
            "results": []
        }));
    }

    let effective_query = build_hf_search_query(&search_term, &param_filter, &task_filter);
    let fetch_limit = if param_filter != "all" || task_filter != "all" {
        48
    } else {
        limit
    };

    let primary_res = state
        .hf_client
        .search_models_ext(&effective_query, sort_mode, fetch_limit)
        .await;

    let mut raw_results = match primary_res {
        Ok(items) => items,
        Err(e) => {
            return Json(json!({
                "status": "error",
                "message": e.to_string(),
                "results": []
            }))
        }
    };

    // When user provided free-text query plus param/task filters, augment with a targeted query so narrow bands still match
    if !search_term.trim().is_empty() && (task_filter != "all" || (param_filter != "all" && param_filter != "fit_gpu")) {
        let task_hint = match task_filter.as_str() {
            "coding" => Some("coder"),
            "reasoning" => Some("reasoning"),
            "vision" => Some("VL"),
            "agentic" | "tools" => Some("tool"),
            _ => None,
        };
        let param_hint = match param_filter.as_str() {
            "sub1b" => Some("0.5B"),
            "1b" => Some("1B"),
            "2b" => Some("2B"),
            "3b" => Some("3B"),
            "4b" => Some("4B"),
            "5b" => Some("5B"),
            "6b" => Some("6B"),
            "7b" => Some("7B"),
            "8b" => Some("8B"),
            "9b" => Some("9B"),
            "10b_12b" => Some("12B"),
            "14b" => Some("14B"),
            "32b" => Some("32B"),
            "70b_plus" => Some("70B"),
            _ => None,
        };
        let mut extra_terms = vec![search_term.trim()];
        if let Some(p) = param_hint {
            if !search_term.to_ascii_lowercase().contains(&p.to_ascii_lowercase()) {
                extra_terms.push(p);
            }
        }
        if let Some(t) = task_hint {
            if !search_term.to_ascii_lowercase().contains(t) {
                extra_terms.push(t);
            }
        }
        if extra_terms.len() > 1 {
            let augmented = extra_terms.join(" ");
            if let Ok(extra) = state
                .hf_client
                .search_models_ext(&augmented, sort_mode, 25)
                .await
            {
                let mut combined = extra;
                combined.extend(raw_results);
                raw_results = combined;
            }
        }
    }

    let diag = probe_hardware();
    let installed_filenames: Vec<String> = if query.autocomplete == Some(true) {
        Vec::new()
    } else {
        let root = resolve_canonical_models_dir();
        scan_local_models(&root, &diag)
            .unwrap_or_default()
            .into_iter()
            .map(|m| m.file_name)
            .collect()
    };

    let is_trending = sort_mode.eq_ignore_ascii_case("trending");
    let mut seen = HashSet::new();
    let mut enriched_results = Vec::new();

    for r in raw_results {
        let key = r.repo_id.to_ascii_lowercase();
        if !seen.insert(key) {
            continue;
        }
        let Some(item) =
            build_dynamic_hf_catalog_item(&diag, &r, &installed_filenames, is_trending, None)
        else {
            continue;
        };
        if !matches_search_filters(&item, &r, &param_filter, &task_filter) {
            continue;
        }

        let primary_var = item
            .variants
            .iter()
            .find(|v| v.is_recommended_fit)
            .or_else(|| item.variants.first());
        let primary_fit = primary_var
            .map(|v| v.hardware_fit)
            .unwrap_or(HardwareFit::PartialCpuOffload);

        let mut val = serde_json::to_value(&r).unwrap_or_else(|_| json!({}));
        if let Some(obj) = val.as_object_mut() {
            obj.insert("hardware_fit".to_string(), json!(primary_fit));
            obj.insert(
                "estimated_tps".to_string(),
                json!(item.benchmark_scores.speed_tokens_sec_estimate),
            );
            obj.insert("parameter_count".to_string(), json!(item.parameter_count));
            obj.insert("is_moe".to_string(), json!(item.is_moe));
            obj.insert("cognitive_profile".to_string(), json!(item.cognitive_profile));
            obj.insert("category_role".to_string(), json!(item.category_role));
            obj.insert("is_downloaded".to_string(), json!(item.is_downloaded));
            obj.insert("local_file_path".to_string(), json!(item.local_file_path));
            obj.insert("catalog_item".to_string(), json!(item));
        }
        enriched_results.push(val);
        if enriched_results.len() >= limit {
            break;
        }
    }

    Json(json!({
        "status": "success",
        "query": search_term.trim(),
        "effective_query": effective_query,
        "results": enriched_results
    }))
}

#[derive(Debug, Deserialize)]
pub struct RepoFilesQuery {
    pub repo_id: String,
}

/// Live inspection of GGUF quantization files in a specific Hugging Face repository.
pub async fn inventory_repo_files_handler(
    State(state): State<AppState>,
    Query(query): Query<RepoFilesQuery>,
) -> Json<Value> {
    let _guard = state.acquire_connection();
    let diag = probe_hardware();
    let root = resolve_canonical_models_dir();
    let installed_models = scan_local_models(&root, &diag).unwrap_or_default();

    match state.hf_client.get_repo_gguf_files(&query.repo_id).await {
        Ok(files) => {
            let files_with_fit: Vec<Value> = files
                .into_iter()
                .map(|f| {
                    let (total_params, active_params, _) =
                        souls_llm_inventory::parse_model_params_from_name(
                            &f.file_name,
                            Some(f.size_bytes),
                        );
                    let eval_params = active_params.unwrap_or(total_params);
                    let fit = souls_llm_inventory::evaluate_model_fit(
                        &diag,
                        f.size_bytes,
                        8192,
                        total_params,
                    );
                    let estimated_tps = estimate_variant_throughput_tps(
                        &diag,
                        fit,
                        f.size_bytes,
                        eval_params,
                    );
                    let matched_local = installed_models.iter().find(|im| {
                        im.file_name.eq_ignore_ascii_case(&f.file_name)
                    });
                    json!({
                        "file_name": f.file_name,
                        "size_bytes": f.size_bytes,
                        "size_gb": f.size_gb,
                        "download_url": f.download_url,
                        "sha256": f.sha256,
                        "quant_tag": f.quant_tag,
                        "hardware_fit": fit,
                        "estimated_tps": estimated_tps,
                        "is_installed": matched_local.is_some(),
                        "installed_file_name": matched_local.map(|m| m.file_name.clone()),
                    })
                })
                .collect();

            Json(json!({
                "status": "success",
                "repo_id": query.repo_id,
                "files": files_with_fit
            }))
        }
        Err(e) => Json(json!({
            "status": "error",
            "message": e.to_string(),
            "files": []
        })),
    }
}

#[derive(Debug, Deserialize)]
pub struct EnqueueDownloadPayload {
    pub repo_id: String,
    pub file_name: String,
    pub download_url: Option<String>,
    pub expected_sha256: Option<String>,
    pub projector_file: Option<String>,
    pub projector_url: Option<String>,
}

/// Enqueues a model for background downloading (including companion VLM projector if requested).
pub async fn inventory_download_handler(
    State(state): State<AppState>,
    Json(payload): Json<EnqueueDownloadPayload>,
) -> Json<Value> {
    let _guard = state.acquire_connection();
    let dest_dir = resolve_canonical_models_dir();

    let download_url = payload.download_url.unwrap_or_else(|| {
        format!(
            "https://huggingface.co/{}/resolve/main/{}",
            payload.repo_id, payload.file_name
        )
    });

    match state
        .inventory_downloader
        .enqueue_download(
            &payload.repo_id,
            &payload.file_name,
            &download_url,
            payload.expected_sha256,
            &dest_dir,
        )
        .await
    {
        Ok(item) => {
            let mut enqueued_items = vec![item];

            // If this is a VLM with companion vision projector, enqueue it automatically
            if let Some(proj_name) = payload.projector_file {
                let proj_url = payload.projector_url.unwrap_or_else(|| {
                    format!(
                        "https://huggingface.co/{}/resolve/main/{}",
                        payload.repo_id, proj_name
                    )
                });
                if let Ok(proj_item) = state
                    .inventory_downloader
                    .enqueue_download(&payload.repo_id, &proj_name, &proj_url, None, &dest_dir)
                    .await
                {
                    enqueued_items.push(proj_item);
                }
            }

            let primary_item = enqueued_items.first().cloned();
            Json(json!({
                "status": "enqueued",
                "item": primary_item,
                "items": enqueued_items,
                "count": enqueued_items.len()
            }))
        }
        Err(e) => Json(json!({
            "status": "error",
            "message": e.to_string()
        })),
    }
}

/// Lists all active, queued, paused, and completed downloads.
pub async fn inventory_list_downloads_handler(State(state): State<AppState>) -> Json<Value> {
    let _guard = state.acquire_connection();
    let downloads = state.inventory_downloader.list_downloads().await;
    Json(json!({
        "status": "success",
        "downloads": downloads
    }))
}

/// Pauses an active or queued download.
pub async fn inventory_pause_download_handler(
    State(state): State<AppState>,
    AxumPath(id): AxumPath<String>,
) -> Json<Value> {
    let _guard = state.acquire_connection();
    match state.inventory_downloader.pause_download(&id).await {
        Ok(_) => Json(json!({ "status": "paused", "id": id })),
        Err(e) => Json(json!({ "status": "error", "message": e.to_string() })),
    }
}

/// Resumes a paused or failed download.
pub async fn inventory_resume_download_handler(
    State(state): State<AppState>,
    AxumPath(id): AxumPath<String>,
) -> Json<Value> {
    let _guard = state.acquire_connection();
    match state.inventory_downloader.resume_download(&id).await {
        Ok(_) => Json(json!({ "status": "resumed", "id": id })),
        Err(e) => Json(json!({ "status": "error", "message": e.to_string() })),
    }
}

/// Cancels a download and purges intermediate .part files.
pub async fn inventory_cancel_download_handler(
    State(state): State<AppState>,
    AxumPath(id): AxumPath<String>,
) -> Json<Value> {
    let _guard = state.acquire_connection();
    match state.inventory_downloader.cancel_download(&id).await {
        Ok(_) => Json(json!({ "status": "cancelled", "id": id })),
        Err(e) => Json(json!({ "status": "error", "message": e.to_string() })),
    }
}

/// Safely removes a model from disk.
pub async fn inventory_delete_model_handler(
    State(state): State<AppState>,
    AxumPath(filename): AxumPath<String>,
) -> Json<Value> {
    let _guard = state.acquire_connection();
    match delete_model_file(&filename) {
        Ok(_) => Json(json!({ "status": "deleted", "filename": filename })),
        Err(e) => Json(json!({ "status": "error", "message": e.to_string() })),
    }
}

/// Real-time Server-Sent Events stream of download progress.
pub async fn inventory_download_stream_handler(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = state.inventory_downloader.subscribe_progress();
    let stream = BroadcastStream::new(rx).filter_map(|msg| match msg {
        Ok(item) => {
            let data = serde_json::to_string(&item).unwrap_or_default();
            Some(Ok(Event::default().event("download_progress").data(data)))
        }
        Err(_) => None,
    });

    Sse::new(stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(10)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use souls_llm_inventory::HardwareDiagnostic;

    #[test]
    fn test_derive_gpu_architecture_mappings() {
        assert_eq!(derive_gpu_architecture("NVIDIA GeForce RTX 5090"), "Blackwell");
        assert_eq!(derive_gpu_architecture("Blackwell B200"), "Blackwell");
        assert_eq!(derive_gpu_architecture("NVIDIA GeForce RTX 4090"), "Ada Lovelace");
        assert_eq!(derive_gpu_architecture("RTX 4080 Mobile (Ada)"), "Ada Lovelace");
        assert_eq!(derive_gpu_architecture("NVIDIA GeForce RTX 3080 Ti"), "Ampere");
        assert_eq!(derive_gpu_architecture("NVIDIA GeForce RTX 2060"), "Turing");
        assert_eq!(derive_gpu_architecture("GeForce GTX 1660 Ti"), "Turing");
        assert_eq!(derive_gpu_architecture("NVIDIA TITAN V"), "Volta");
        assert_eq!(derive_gpu_architecture("GeForce GTX 1080"), "Pascal");
        assert_eq!(derive_gpu_architecture("NVIDIA CUDA Device"), "NVIDIA CUDA");
        assert_eq!(derive_gpu_architecture("AMD Radeon RX 7900 XTX"), "AMD ROCm / RDNA");
        assert_eq!(derive_gpu_architecture("Apple M3 Max"), "Apple Silicon");
        assert_eq!(derive_gpu_architecture("Intel UHD Graphics"), "Host CPU / iGPU");
        assert_eq!(derive_gpu_architecture("CPU Host Mode (Sem dGPU NVIDIA)"), "Host CPU / iGPU");
        assert_eq!(derive_gpu_architecture("Microsoft Basic Display Adapter"), "Host CPU / iGPU");
        assert_eq!(derive_gpu_architecture("NVIDIA GeForce RTX 3080 Display Adapter"), "Ampere");
        assert_eq!(derive_gpu_architecture("NVIDIA GeForce RTX 2060 Display Adapter"), "Turing");
        assert_eq!(derive_gpu_architecture("AMD Radeon RX 7900 Display Adapter"), "AMD ROCm / RDNA");
        assert_eq!(derive_gpu_architecture("NVIDIA RTX 4000 Ada Generation"), "Ada Lovelace");
        assert_eq!(derive_gpu_architecture("AMD Radeon (Adaptive Sync)"), "AMD ROCm / RDNA");
        assert_eq!(derive_gpu_architecture("NVIDIA RTX 5000 Ada Generation"), "Ada Lovelace");
        assert_eq!(derive_gpu_architecture("NVIDIA RTX 500 Ada Generation Laptop GPU"), "Ada Lovelace");
    }

    #[test]
    fn test_enrich_hardware_diagnostic_preserves_real_names() {
        let diag = HardwareDiagnostic {
            has_dedicated_gpu: true,
            gpu_name: "NVIDIA GeForce RTX 4070".to_string(),
            cpu_name: "AMD Ryzen 9 7950X".to_string(),
            vram_total_mb: 12288,
            vram_free_mb: 10000,
            vram_safe_usable_mb: 9500,
            host_ram_total_mb: 65536,
            host_ram_available_mb: 48000,
            host_ram_safe_usable_mb: 32000,
            cpu_cores_physical: 16,
            cpu_cores_logical: 32,
            has_avx2: true,
            has_avx512: true,
        };

        let enriched = enrich_hardware_diagnostic(diag);
        assert_eq!(enriched["gpu_name"], "NVIDIA GeForce RTX 4070");
        assert_eq!(enriched["gpu_architecture"], "Ada Lovelace");
        assert_eq!(enriched["gpu_compute_capability"], "8.9");
        assert_eq!(enriched["cpu_name"], "AMD Ryzen 9 7950X (16 Cores / 32 Threads)");
        assert!(enriched["cpu_instructions_extended"].as_str().unwrap().contains("AVX-512"));
    }

    #[test]
    fn test_enrich_hardware_diagnostic_cpu_only() {
        let diag = HardwareDiagnostic {
            has_dedicated_gpu: false,
            gpu_name: "None (CPU Execution)".to_string(),
            cpu_name: "Intel Core i7-12700H".to_string(),
            vram_total_mb: 0,
            vram_free_mb: 0,
            vram_safe_usable_mb: 0,
            host_ram_total_mb: 32768,
            host_ram_available_mb: 20000,
            host_ram_safe_usable_mb: 16000,
            cpu_cores_physical: 14,
            cpu_cores_logical: 20,
            has_avx2: true,
            has_avx512: false,
        };

        let enriched = enrich_hardware_diagnostic(diag);
        assert_eq!(enriched["gpu_name"], "None (CPU Execution)");
        assert_eq!(enriched["gpu_architecture"], "Host CPU / iGPU");
        assert_eq!(enriched["gpu_compute_capability"], "N/A");
        assert_eq!(enriched["cpu_name"], "Intel Core i7-12700H (14 Cores / 20 Threads)");
        assert!(enriched["cpu_instructions_extended"].as_str().unwrap().contains("AVX2"));
        assert!(!enriched["cpu_instructions_extended"].as_str().unwrap().contains("AVX-512"));
    }

    #[test]
    fn test_repo_files_fit_uses_dynamic_params() {
        let diag = probe_hardware();

        // 0.5B model file (~400 MB)
        let (params_0_5b, _, _) = souls_llm_inventory::parse_model_params_from_name(
            "qwen2-5-coder-0-5b-instruct-q4_k_m.gguf",
            Some(400_000_000),
        );
        assert_eq!(params_0_5b, 500_000_000);
        let fit_0_5b = souls_llm_inventory::evaluate_model_fit(&diag, 400_000_000, 8192, params_0_5b);
        assert!(matches!(fit_0_5b, souls_llm_inventory::HardwareFit::OptimalVram));

        // 70B model file (~40 GB)
        let (params_70b, _, _) = souls_llm_inventory::parse_model_params_from_name(
            "Llama-3.3-70B-Instruct-Q4_K_M.gguf",
            Some(40_000_000_000),
        );
        assert_eq!(params_70b, 70_000_000_000);
        let fit_70b = souls_llm_inventory::evaluate_model_fit(&diag, 40_000_000_000, 8192, params_70b);
        assert!(!matches!(fit_70b, souls_llm_inventory::HardwareFit::OptimalVram));
    }

    #[test]
    fn test_combined_hf_search_query_and_fit_filters() {
        assert_eq!(build_hf_search_query("bartowski", "4b", "coding"), "bartowski");
        assert_eq!(build_hf_search_query("", "4b", "coding"), "coder 4B");
        assert_eq!(build_hf_search_query("", "fit_gpu", "all"), "Qwen3");

        let diag = HardwareDiagnostic {
            has_dedicated_gpu: true,
            gpu_name: "NVIDIA GeForce RTX 2060".to_string(),
            cpu_name: "Intel Core i9".to_string(),
            vram_total_mb: 6144,
            vram_free_mb: 5000,
            vram_safe_usable_mb: 5294,
            host_ram_total_mb: 32768,
            host_ram_available_mb: 24000,
            host_ram_safe_usable_mb: 20000,
            cpu_cores_physical: 8,
            cpu_cores_logical: 16,
            has_avx2: true,
            has_avx512: false,
        };

        let raw_4b = HfSearchResult {
            repo_id: "Qwen/Qwen3-4B-Coder-GGUF".to_string(),
            author: Some("Qwen".to_string()),
            downloads: 12000,
            likes: 90,
            last_modified: None,
            pipeline_tag: Some("text-generation".to_string()),
            tags: vec!["gguf".to_string(), "code".to_string()],
        };
        let item_4b = build_dynamic_hf_catalog_item(&diag, &raw_4b, &[], false, None).unwrap();
        assert!(matches_search_filters(&item_4b, &raw_4b, "fit_gpu", "coding"));
        assert!(matches_search_filters(&item_4b, &raw_4b, "4b", "coding"));
        assert!(!matches_search_filters(&item_4b, &raw_4b, "14b", "coding"));

        let raw_32b = HfSearchResult {
            repo_id: "Qwen/Qwen2.5-Coder-32B-Instruct-GGUF".to_string(),
            author: Some("Qwen".to_string()),
            downloads: 50000,
            likes: 300,
            last_modified: None,
            pipeline_tag: Some("text-generation".to_string()),
            tags: vec!["gguf".to_string()],
        };
        let item_32b = build_dynamic_hf_catalog_item(&diag, &raw_32b, &[], false, None).unwrap();
        assert!(!matches_search_filters(&item_32b, &raw_32b, "fit_gpu", "all"));
        assert!(matches_search_filters(&item_32b, &raw_32b, "32b", "coding"));
    }
}
