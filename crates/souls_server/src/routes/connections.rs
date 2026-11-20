//! REST endpoints for Global Cloud Providers and Headless Agentic CLIs.
//!
//! Provides inspection of Chinese & Global Frontier API gateways, credentials testing,
//! automated host CLI detection (agy, claude, codex), and task dispatch.

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Instant;
use tracing::{error, info};

use souls_core::harness::{
    detect_installed_clis, AgenticCliCommand, AgenticCliKind, AgenticHarnessDispatcher,
};
use souls_model_router::{get_canonical_frontier_providers, CloudProvider};

use crate::state::AppState;

/// Provider configuration status and metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderStatusDto {
    pub slug: String,
    pub name: String,
    pub base_url: String,
    pub auth_header_env: String,
    pub is_configured: bool,
    pub masked_key: Option<String>,
    pub default_cache_discount: f64,
    pub latency_factor: f64,
    pub models_count: usize,
    pub models: Vec<ProviderModelDto>,
    pub console_url: String,
    pub purpose_summary: String,
    pub has_free_tier: bool,
}

fn provider_ux_meta(provider: &CloudProvider) -> (&'static str, &'static str, bool) {
    match provider {
        CloudProvider::OpenRouter => (
            "https://openrouter.ai/keys",
            "Acesso unificado a dezenas de modelos (Claude, GPT, Gemini, DeepSeek) com uma única chave.",
            true,
        ),
        CloudProvider::Gemini => (
            "https://aistudio.google.com/apikey",
            "Modelos Google Gemini Flash e Pro direto pelo AI Studio (ótimo custo-benefício e cota gratuita).",
            true,
        ),
        CloudProvider::DeepSeek => (
            "https://platform.deepseek.com/api_keys",
            "Raciocínio e código de alta eficiência (DeepSeek Chat e R1) com grande desconto de cache.",
            false,
        ),
        CloudProvider::Groq => (
            "https://console.groq.com/keys",
            "Respostas ultrarrápidas em LPU para triagem, classificação e agentes de baixa latência.",
            true,
        ),
        CloudProvider::Cerebras => (
            "https://cloud.cerebras.ai/",
            "Altíssima velocidade de geração (400+ tokens/s) para tarefas rápidas e código.",
            true,
        ),
        CloudProvider::Together => (
            "https://api.together.xyz/settings/api-keys",
            "Hospedagem rápida de modelos open-weight (Llama, Qwen, DeepSeek) na nuvem.",
            false,
        ),
        CloudProvider::OpenAI => (
            "https://platform.openai.com/api-keys",
            "Modelos GPT e família o-series diretamente pela API oficial da OpenAI.",
            false,
        ),
        CloudProvider::Anthropic => (
            "https://console.anthropic.com/settings/keys",
            "Modelos Claude Sonnet, Haiku e Opus diretamente pela API oficial da Anthropic.",
            false,
        ),
        CloudProvider::NousPortal => (
            "https://portal.nousresearch.com/",
            "Acesso aos modelos Hermes e DeepHermes no portal oficial da Nous Research.",
            true,
        ),
        CloudProvider::AlibabaDashScope => (
            "https://dashscope.console.aliyun.com/apiKey",
            "Modelos Qwen (2.5 / 3 / Coder / Max) diretamente pela nuvem Alibaba DashScope.",
            false,
        ),
        CloudProvider::MoonshotKimi => (
            "https://platform.moonshot.ai/console/api-keys",
            "Modelos Kimi com janela longa para análise documental e raciocínio.",
            false,
        ),
        CloudProvider::ZhipuGLM => (
            "https://open.bigmodel.cn/usercenter/apikeys",
            "Modelos da família GLM diretamente pela API da Zhipu AI.",
            false,
        ),
        CloudProvider::MiniMax => (
            "https://platform.minimaxi.com/",
            "Modelos MiniMax de uso geral, código e agentes de contexto longo.",
            false,
        ),
        CloudProvider::SiliconFlow => (
            "https://cloud.siliconflow.cn/account/ak",
            "Gateway asiático econômico para modelos open-weight com suporte a cache.",
            true,
        ),
        CloudProvider::Bedrock => (
            "https://console.aws.amazon.com/bedrock/",
            "Claude, Nova e outros modelos gerenciados dentro da sua conta AWS.",
            false,
        ),
        CloudProvider::AzureOpenAI => (
            "https://portal.azure.com/",
            "Modelos OpenAI e agentes com governança corporativa no Microsoft Azure.",
            false,
        ),
        CloudProvider::LocalSilicon | CloudProvider::Custom(_) => (
            "https://openrouter.ai/keys",
            "Endpoint local ou customizado.",
            true,
        ),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderModelDto {
    pub model_id: String,
    pub display_name: String,
    pub context_window: u32,
    pub cost_input_per_m: f64,
    pub cost_output_per_m: f64,
    pub supports_cache: bool,
    pub supports_tools: bool,
    pub recommended_task: String,
    pub score_lmarena: u32,
}

/// Request payload for testing provider connectivity.
#[derive(Debug, Clone, Deserialize)]
pub struct TestConnectionRequest {
    pub provider: String,
    pub api_key: Option<String>,
    pub endpoint_url: Option<String>,
}

/// Request payload for dispatching a headless task to an agentic CLI.
#[derive(Debug, Clone, Deserialize)]
pub struct DispatchCliRequest {
    pub cli: String, // "agy", "claude", "codex", or custom name
    pub prompt: String,
    pub working_dir: Option<String>,
    pub model: Option<String>,
    pub reasoning_effort: Option<String>,
    pub timeout_secs: Option<u64>,
    pub auto_restore_on_failure: Option<bool>,
}

/// GET /api/v1/connections/providers
///
/// Returns list of all canonical frontier providers and whether an API key is detected.
pub async fn list_providers_handler() -> impl IntoResponse {
    let specs = get_canonical_frontier_providers();
    let mut providers = Vec::with_capacity(specs.len());

    for s in specs {
        let env_key = std::env::var(&s.auth_header_env).ok();
        let is_configured = env_key.as_ref().map(|k| !k.trim().is_empty()).unwrap_or(false);
        let masked_key = env_key.and_then(|k| mask_api_key(&k));

        let models_count = s.models.len();
        let models = s
            .models
            .into_iter()
            .map(|m| ProviderModelDto {
                model_id: m.model_id,
                display_name: m.display_name,
                context_window: m.context_window,
                cost_input_per_m: m.cost_input_per_m,
                cost_output_per_m: m.cost_output_per_m,
                supports_cache: m.supports_cache,
                supports_tools: m.supports_tools,
                recommended_task: m.recommended_task,
                score_lmarena: m.score_lmarena,
            })
            .collect();

        let (console_url, purpose_summary, has_free_tier) = provider_ux_meta(&s.provider);
        providers.push(ProviderStatusDto {
            slug: s.provider.canonical_slug().to_string(),
            name: s.name,
            base_url: s.base_url,
            auth_header_env: s.auth_header_env,
            is_configured,
            masked_key,
            default_cache_discount: s.default_cache_discount,
            latency_factor: s.latency_factor,
            models_count,
            models,
            console_url: console_url.to_string(),
            purpose_summary: purpose_summary.to_string(),
            has_free_tier,
        });
    }

    (StatusCode::OK, Json(serde_json::json!({
        "status": "success",
        "count": providers.len(),
        "providers": providers
    })))
}

/// POST /api/v1/connections/test
///
/// Validates connectivity and responsiveness of a specific provider.
pub async fn test_provider_handler(
    Json(payload): Json<TestConnectionRequest>,
) -> impl IntoResponse {
    let provider_opt = CloudProvider::from_provider_name(&payload.provider);
    let provider = match provider_opt {
        Some(p) => p,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "status": "error",
                    "message": format!("Unknown provider '{}'", payload.provider)
                })),
            );
        }
    };

    let base_url = payload
        .endpoint_url
        .unwrap_or_else(|| provider.default_base_url().to_string());

    let auth_key = payload
        .api_key
        .or_else(|| std::env::var(provider.auth_header_env()).ok());

    let start = Instant::now();
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build();

    let client = match client {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "status": "error",
                    "message": format!("Failed to build HTTP client: {e}")
                })),
            );
        }
    };

    let mut req = client.get(&base_url);
    if let Some(ref key) = auth_key {
        req = req.bearer_auth(key);
    }

    // Ping probe
    match req.send().await {
        Ok(resp) => {
            let latency_ms = start.elapsed().as_millis() as u64;
            let status = resp.status().as_u16();
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "status": "success",
                    "provider": provider.canonical_slug(),
                    "base_url": base_url,
                    "http_status": status,
                    "latency_ms": latency_ms,
                    "authenticated": auth_key.is_some(),
                    "message": format!("Endpoint responded with HTTP {} in {}ms", status, latency_ms)
                })),
            )
        }
        Err(e) => {
            let latency_ms = start.elapsed().as_millis() as u64;
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "status": "warning",
                    "provider": provider.canonical_slug(),
                    "base_url": base_url,
                    "latency_ms": latency_ms,
                    "error": e.to_string(),
                    "message": format!("Connection attempt failed: {e}")
                })),
            )
        }
    }
}

/// GET /api/v1/connections/clis
///
/// Discovers installed agentic CLIs on the host (`agy`, `claude`, `codex`).
pub async fn list_clis_handler() -> impl IntoResponse {
    let clis = detect_installed_clis().await;
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "success",
            "count": clis.len(),
            "clis": clis
        })),
    )
}

/// POST /api/v1/connections/cli/dispatch
///
/// Dispatches a headless task to an installed agentic CLI with streaming NDJSON capture.
pub async fn dispatch_cli_handler(
    State(state): State<AppState>,
    Json(payload): Json<DispatchCliRequest>,
) -> impl IntoResponse {
    let _guard = state.acquire_connection();

    let kind = match payload.cli.to_ascii_lowercase().as_str() {
        "agy" | "antigravity" => AgenticCliKind::Antigravity,
        "claude" | "claudecode" => AgenticCliKind::ClaudeCode,
        "codex" => AgenticCliKind::Codex,
        other => AgenticCliKind::Custom(other.to_string()),
    };

    let working_dir = payload
        .working_dir
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("Z:\\souls_engine"));

    let mut cmd = AgenticCliCommand::new(kind, payload.prompt, working_dir)
        .with_timeout_secs(payload.timeout_secs.unwrap_or(300))
        .with_auto_restore(payload.auto_restore_on_failure.unwrap_or(true));

    if let Some(m) = payload.model {
        cmd = cmd.with_model(m);
    }
    if let Some(e) = payload.reasoning_effort {
        cmd = cmd.with_reasoning_effort(e);
    }

    info!(
        target: "souls_server::connections",
        "Dispatching headless agentic task to {:?}",
        cmd.cli
    );

    match AgenticHarnessDispatcher::dispatch(&cmd).await {
        Ok(output) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "status": "success",
                "output": output
            })),
        ),
        Err(e) => {
            error!(target: "souls_server::connections", "Agentic dispatch failed: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "status": "error",
                    "message": format!("Agentic CLI execution failed: {e}")
                })),
            )
        }
    }
}

/// Masks sensitive API keys showing only the first 4 and last 4 characters.
pub fn mask_api_key(key: &str) -> Option<String> {
    let trimmed = key.trim();
    if trimmed.is_empty() {
        return None;
    }
    if trimmed.len() <= 8 {
        return Some("********".to_string());
    }
    let prefix = &trimmed[..4];
    let suffix = &trimmed[trimmed.len() - 4..];
    Some(format!("{}...{}", prefix, suffix))
}

#[derive(Debug, Deserialize)]
pub struct SaveCredentialRequest {
    pub key: String,
    pub value: String,
}

/// Atomically saves an API key / credential into runtime env, `.souls_data/credentials.env`
/// and `%LOCALAPPDATA%\hermes\.env` without breaking other environment variables.
pub fn save_credential_atomic(key: &str, value: &str) -> Result<String, String> {
    let clean_key = key.trim();
    let clean_val = value.trim();

    if clean_key.is_empty() {
        return Err("O nome da variável de ambiente não pode ser vazio.".to_string());
    }

    // 1. Atualiza no processo atual
    std::env::set_var(clean_key, clean_val);

    // 2. Persiste em Z:\souls_engine\.souls_data\credentials.env
    let data_dir = crate::resolve_data_dir();
    let _ = std::fs::create_dir_all(&data_dir);
    let souls_env_file = data_dir.join("credentials.env");
    let _ = upsert_env_line(&souls_env_file, clean_key, clean_val);

    // 3. Persiste em %LOCALAPPDATA%\hermes\.env (e sub-perfis com .env próprio) se existir
    let hermes_dir = crate::profiles::resolve_hermes_root_dir(None);
    if hermes_dir.exists() {
        let hermes_env_file = hermes_dir.join(".env");
        let _ = upsert_env_line(&hermes_env_file, clean_key, clean_val);
        if let Ok(entries) = std::fs::read_dir(hermes_dir.join("profiles")) {
            for entry in entries.flatten() {
                let prof_env = entry.path().join(".env");
                if prof_env.exists() {
                    let _ = upsert_env_line(&prof_env, clean_key, clean_val);
                }
            }
        }
    }

    let masked = mask_api_key(clean_val).unwrap_or_else(|| "********".to_string());
    info!(key = %clean_key, masked = %masked, "Credencial de API persistida com sucesso");
    Ok(masked)
}

fn upsert_env_line(path: &std::path::Path, key: &str, value: &str) -> Result<(), String> {
    let mut lines = Vec::new();
    let mut found = false;
    if path.exists() {
        if let Ok(content) = std::fs::read_to_string(path) {
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with(&format!("{key}=")) || trimmed.starts_with(&format!("export {key}=")) {
                    lines.push(format!("{key}={value}"));
                    found = true;
                } else {
                    lines.push(line.to_string());
                }
            }
        }
    }
    if !found {
        lines.push(format!("{key}={value}"));
    }
    let content = lines.join("\n") + "\n";
    std::fs::write(path, content.as_bytes())
        .map_err(|e| format!("Falha ao gravar arquivo de ambiente: {e}"))?;
    Ok(())
}

/// POST /api/v1/system/credentials/save
pub async fn save_credential_handler(
    Json(payload): Json<SaveCredentialRequest>,
) -> impl IntoResponse {
    match save_credential_atomic(&payload.key, &payload.value) {
        Ok(masked) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "status": "success",
                "key": payload.key,
                "masked": masked,
                "message": format!("Chave '{}' salva e ativada com sucesso.", payload.key)
            })),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "status": "error",
                "message": e
            })),
        ),
    }
}
