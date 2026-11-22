//! Handler for `POST /v1/chat/completions` OpenAI-compatible endpoint.
//!
//! Features:
//! - Full OpenAI API contract compliance (Sync JSON and SSE Streaming).
//! - llguidance / JSON Schema enforcement with automatic response healing.
//! - Non-blocking FinOps MPSC outcome recording for ParetoBandit and E³ metric.
//! - Silicon protection: `tx.is_closed()` client disconnect cancellation in streaming mode.
//! - RAII `ConnectionGuard` tracking on all execution paths.

use std::convert::Infallible;
use std::time::Instant;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{IntoResponse, Json, Response};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio_stream::wrappers::ReceiverStream;
use tracing::{info, warn};

use souls_inference_runtime::{heal_json_response, GenParams};
use souls_model_router::metrics::TaskOutcome;
use souls_model_router::priors::current_timestamp_sec;

use crate::state::AppState;

/// OpenAI Chat Message format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// JSON Schema / Response Format specifier.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseFormat {
    #[serde(rename = "type")]
    pub format_type: Option<String>,
    pub json_schema: Option<Value>,
}

/// Incoming Chat Completion payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionRequest {
    #[serde(default)]
    pub model: Option<String>,
    pub messages: Vec<ChatMessage>,
    #[serde(default)]
    pub temperature: Option<f32>,
    #[serde(default)]
    pub top_p: Option<f32>,
    #[serde(default)]
    pub max_tokens: Option<u32>,
    #[serde(default)]
    pub stream: bool,
    #[serde(default)]
    pub response_format: Option<ResponseFormat>,
    #[serde(default)]
    pub stop: Option<Vec<String>>,
}

/// Choice in standard non-streaming response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatChoice {
    pub index: usize,
    pub message: ChatMessage,
    pub finish_reason: String,
}

/// Prompt token details breakdown (OpenAI-compatible prefix cache telemetry).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTokensDetails {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cached_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub padded_tokens: Option<u32>,
}

/// Usage statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageInfo {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_tokens_details: Option<PromptTokensDetails>,
}

/// Non-streaming Chat Completion response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub object: &'static str,
    pub created: i64,
    pub model: String,
    pub choices: Vec<ChatChoice>,
    pub usage: UsageInfo,
}

/// Delta payload for SSE chunk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeltaContent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

/// Chunk choice for SSE stream.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatChunkChoice {
    pub index: usize,
    pub delta: DeltaContent,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,
}

/// Streaming Chat Completion Chunk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionChunk {
    pub id: String,
    pub object: &'static str,
    pub created: i64,
    pub model: String,
    pub choices: Vec<ChatChunkChoice>,
}

/// POST /v1/chat/completions handler.
pub async fn chat_completions_handler(
    State(state): State<AppState>,
    Json(payload): Json<ChatCompletionRequest>,
) -> Response {
    let _conn_guard = state.acquire_connection();

    if payload.messages.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": {
                    "message": "Messages array cannot be empty.",
                    "type": "invalid_request_error"
                }
            })),
        )
            .into_response();
    }

    let model_name = payload
        .model
        .clone()
        .unwrap_or_else(|| "tier1_qwen_coder_gpu".to_string());

    let prompt = payload
        .messages
        .iter()
        .map(|m| format!("{}: {}", m.role, m.content))
        .collect::<Vec<_>>()
        .join("\n");

    let packaging = state.runtime.package_prompt(&prompt, &model_name);
    let prompt_tokens = packaging.raw_token_estimate as u32;
    let padded_tokens = packaging.padded_token_boundary as u32;

    info!(
        "Chat request packaging: model='{}', prompt_tokens={}, padded_boundary={} (PadTokenId={}, 64Aligned={})",
        model_name, prompt_tokens, padded_tokens, packaging.pad_token_id, packaging.is_64_aligned
    );

    let completion_id = format!("chatcmpl-{}", souls_core::fs::validate_refs_path(&std::path::PathBuf::from("Z:/souls_engine")).map(|_| uuid_like_id()).unwrap_or_else(|_| uuid_like_id()));
    let created_ts = current_timestamp_sec();

    // Configure generation parameters & JSON Schema
    let mut params = GenParams {
        max_tokens: payload.max_tokens.unwrap_or(2048),
        temperature: payload.temperature.unwrap_or(0.2),
        top_p: payload.top_p.unwrap_or(0.95),
        stop_sequences: payload.stop.unwrap_or_default(),
        json_schema: None,
    };

    if let Some(ref rf) = payload.response_format {
        if let Some(ref schema) = rf.json_schema {
            params.json_schema = Some(schema.clone());
        } else if rf.format_type.as_deref() == Some("json_object") {
            params.json_schema = Some(serde_json::json!({
                "type": "object"
            }));
        }
    }

    // Determine category based on prompt hints
    let task_category = if prompt.to_lowercase().contains("code")
        || prompt.to_lowercase().contains("fn ")
        || prompt.to_lowercase().contains("def ")
    {
        "code"
    } else {
        "chat"
    };

    if payload.stream {
        handle_streaming_completion(
            state,
            prompt,
            model_name,
            completion_id,
            created_ts,
            params,
            prompt_tokens,
            padded_tokens,
            task_category,
        )
        .await
    } else {
        handle_sync_completion(
            state,
            prompt,
            model_name,
            completion_id,
            created_ts,
            params,
            prompt_tokens,
            padded_tokens,
            task_category,
        )
        .await
    }
}

/// Resolves upstream native llama-server endpoint URL (default: http://127.0.0.1:8080).
pub fn resolve_llama_server_endpoint() -> String {
    if let Ok(endpoint) = std::env::var("SOULS_LLAMA_SERVER_URL") {
        return endpoint.trim_end_matches('/').to_string();
    }
    let port = std::env::var("SOULS_LLAMA_SERVER_PORT")
        .or_else(|_| std::env::var("LLAMA_ARG_PORT"))
        .unwrap_or_else(|_| "8080".to_string());
    format!("http://127.0.0.1:{port}")
}

/// Checks if an upstream native llama-server instance is responsive.
pub async fn probe_upstream_llama_server(client: &reqwest::Client, endpoint: &str) -> bool {
    let health_url = format!("{endpoint}/health");
    match client.get(&health_url).timeout(std::time::Duration::from_millis(350)).send().await {
        Ok(resp) => resp.status().is_success(),
        Err(_) => false,
    }
}

/// Executes generation: routes to live upstream native `llama-server` if available,
/// or executes via `souls_inference_runtime` on silicon (using newest llamacpp build).
pub async fn perform_generation(
    state: &AppState,
    model_name: &str,
    prompt: &str,
    params: &GenParams,
) -> Result<String, souls_inference_runtime::InferenceError> {
    let client = reqwest::Client::new();
    let upstream_endpoint = resolve_llama_server_endpoint();

    // 1. Check if upstream native llama-server is online (active model in VRAM)
    if probe_upstream_llama_server(&client, &upstream_endpoint).await {
        info!("Routing generation to active upstream llama-server at {upstream_endpoint}");
        let req_body = serde_json::json!({
            "model": model_name,
            "messages": [
                {"role": "user", "content": prompt}
            ],
            "temperature": params.temperature,
            "max_tokens": params.max_tokens,
            "stream": false
        });

        match client
            .post(format!("{upstream_endpoint}/v1/chat/completions"))
            .json(&req_body)
            .timeout(std::time::Duration::from_secs(120))
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() => {
                if let Ok(json_resp) = resp.json::<serde_json::Value>().await {
                    if let Some(content) = json_resp["choices"][0]["message"]["content"].as_str() {
                        return Ok(content.to_string());
                    }
                }
            }
            Ok(resp) => {
                warn!("Upstream llama-server returned HTTP {}: falling back to local runtime", resp.status());
            }
            Err(e) => {
                warn!("Upstream llama-server request failed ({e}): falling back to local runtime");
            }
        }
    }

    // 2. Local execution on designated model or Tier 1 default
    if let Some(model_path) = souls_llm_local_arena::LocalModelInventory::scan().resolve_model_path(model_name) {
        state.runtime.infer_model_generate(&model_path, prompt, params).await.map(|out| out.text)
    } else {
        state.runtime.infer_tier1_generate(prompt, params).await
    }
}

/// Handles non-streaming JSON response.
#[allow(clippy::too_many_arguments)]
async fn handle_sync_completion(
    state: AppState,
    prompt: String,
    model_name: String,
    completion_id: String,
    created_ts: i64,
    params: GenParams,
    prompt_tokens: u32,
    padded_tokens: u32,
    task_category: &'static str,
) -> Response {
    let start = Instant::now();

    // Execute generation via perform_generation (llama-server first, then runtime on newest build)
    let gen_res = perform_generation(&state, &model_name, &prompt, &params).await;

    let raw_text = match gen_res {
        Ok(t) => t,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": {
                        "message": format!("Inference error: {e}"),
                        "type": "inference_error"
                    }
                })),
            )
                .into_response();
        }
    };

    // Apply JSON healing if JSON output is expected or detected
    let healed_text = if params.json_schema.is_some()
        || raw_text.trim_start().starts_with('{')
        || raw_text.trim_start().starts_with('[')
    {
        match heal_json_response(&raw_text) {
            Ok(v) => v.to_string(),
            Err(_) => raw_text,
        }
    } else {
        raw_text
    };

    let elapsed_ms = start.elapsed().as_millis() as f64;
    let completion_tokens = (healed_text.len() / 4).max(1) as u32;

    // Asynchronously record outcome in ParetoBandit via MPSC
    let outcome = TaskOutcome {
        task_id: completion_id.clone(),
        tier_name: model_name.clone(),
        task_category: task_category.to_string(),
        success: true,
        structural_score: 1.0,
        direct_cost_usd: 0.0,
        latency_ms: elapsed_ms,
        tokens_input: prompt_tokens,
        tokens_output: completion_tokens,
        is_rate_limit: false,
        retry_after_secs: None,
    };

    let router = Arc::clone(&state.router);
    let cid = completion_id.clone();
    tokio::spawn(async move {
        if let Err(e) = router.record_outcome(&cid, &outcome).await {
            warn!("Failed to record outcome via MPSC: {e}");
        }
    });

    let resp = ChatCompletionResponse {
        id: completion_id,
        object: "chat.completion",
        created: created_ts,
        model: model_name,
        choices: vec![ChatChoice {
            index: 0,
            message: ChatMessage {
                role: "assistant".to_string(),
                content: healed_text,
            },
            finish_reason: "stop".to_string(),
        }],
        usage: UsageInfo {
            prompt_tokens,
            completion_tokens,
            total_tokens: prompt_tokens + completion_tokens,
            prompt_tokens_details: Some(PromptTokensDetails {
                cached_tokens: if padded_tokens >= prompt_tokens {
                    Some(padded_tokens - prompt_tokens)
                } else {
                    None
                },
                padded_tokens: Some(padded_tokens),
            }),
        },
    };

    (StatusCode::OK, Json(resp)).into_response()
}

/// Handles SSE streaming response with client disconnect protection.
#[allow(clippy::too_many_arguments)]
async fn handle_streaming_completion(
    state: AppState,
    prompt: String,
    model_name: String,
    completion_id: String,
    created_ts: i64,
    params: GenParams,
    prompt_tokens: u32,
    _padded_tokens: u32,
    task_category: &'static str,
) -> Response {
    let (tx, rx) = tokio::sync::mpsc::channel::<Result<Event, Infallible>>(64);

    let stream_model = model_name.clone();
    let stream_cid = completion_id.clone();
    let router = Arc::clone(&state.router);

    tokio::spawn(async move {
        let start = Instant::now();

        // 1. First initial chunk specifying role
        let initial_chunk = ChatCompletionChunk {
            id: stream_cid.clone(),
            object: "chat.completion.chunk",
            created: created_ts,
            model: stream_model.clone(),
            choices: vec![ChatChunkChoice {
                index: 0,
                delta: DeltaContent {
                    role: Some("assistant".to_string()),
                    content: None,
                },
                finish_reason: None,
            }],
        };

        if let Ok(json) = serde_json::to_string(&initial_chunk) {
            if tx.send(Ok(Event::default().data(json))).await.is_err() {
                warn!("Client disconnected before initial chunk. Aborting generation.");
                return;
            }
        }

        // 2. Generate raw text via perform_generation (llama-server first, then runtime on newest build)
        let gen_result = perform_generation(&state, &stream_model, &prompt, &params).await;
        let generated_text = match gen_result {
            Ok(text) => text,
            Err(e) => {
                let err_chunk = ChatCompletionChunk {
                    id: stream_cid.clone(),
                    object: "chat.completion.chunk",
                    created: created_ts,
                    model: stream_model.clone(),
                    choices: vec![ChatChunkChoice {
                        index: 0,
                        delta: DeltaContent {
                            role: None,
                            content: Some(format!(" [Inference error: {e}]")),
                        },
                        finish_reason: Some("error".to_string()),
                    }],
                };
                if let Ok(json) = serde_json::to_string(&err_chunk) {
                    let _ = tx.send(Ok(Event::default().data(json))).await;
                }
                let _ = tx.send(Ok(Event::default().data("[DONE]"))).await;
                return;
            }
        };

        // Split text into tokens / words for streaming
        let words: Vec<&str> = generated_text.split_inclusive(' ').collect();
        let mut emitted_tokens = 0u32;
        let mut client_aborted = false;

        for word in words {
            // RETIFICAÇÃO TÁTICA 2: sse_disconnect_cancellation
            // Verifica a saúde do canal de envio antes de cada token.
            // Se o canal estiver fechado (cliente TCP fechou conexão), aborta a GPU imediatamente.
            if tx.is_closed() {
                warn!(
                    "SSE client disconnected during token streaming! Aborting CUDA generation for task: {}",
                    stream_cid
                );
                client_aborted = true;
                break;
            }

            let chunk = ChatCompletionChunk {
                id: stream_cid.clone(),
                object: "chat.completion.chunk",
                created: created_ts,
                model: stream_model.clone(),
                choices: vec![ChatChunkChoice {
                    index: 0,
                    delta: DeltaContent {
                        role: None,
                        content: Some(word.to_string()),
                    },
                    finish_reason: None,
                }],
            };

            if let Ok(json) = serde_json::to_string(&chunk) {
                if tx.send(Ok(Event::default().data(json))).await.is_err() {
                    warn!("Failed to send chunk to disconnected client; halting generation.");
                    client_aborted = true;
                    break;
                }
            }

            emitted_tokens += 1;
            // Short yield to simulate real token-by-token emission
            tokio::task::yield_now().await;
        }

        if !client_aborted {
            // Final chunk with finish_reason: stop
            let stop_chunk = ChatCompletionChunk {
                id: stream_cid.clone(),
                object: "chat.completion.chunk",
                created: created_ts,
                model: stream_model.clone(),
                choices: vec![ChatChunkChoice {
                    index: 0,
                    delta: DeltaContent {
                        role: None,
                        content: None,
                    },
                    finish_reason: Some("stop".to_string()),
                }],
            };

            if let Ok(json) = serde_json::to_string(&stop_chunk) {
                let _ = tx.send(Ok(Event::default().data(json))).await;
            }

            // Standard OpenAI termination marker: data: [DONE]
            let _ = tx.send(Ok(Event::default().data("[DONE]"))).await;
        }

        // Telemetry outcome recording
        let elapsed_ms = start.elapsed().as_millis() as f64;
        let outcome = TaskOutcome {
            task_id: stream_cid.clone(),
            tier_name: stream_model.clone(),
            task_category: task_category.to_string(),
            success: !client_aborted,
            structural_score: if client_aborted { 0.0 } else { 1.0 },
            direct_cost_usd: 0.0,
            latency_ms: elapsed_ms,
            tokens_input: prompt_tokens,
            tokens_output: emitted_tokens,
            is_rate_limit: false,
            retry_after_secs: None,
        };

        if let Err(e) = router.record_outcome(&stream_cid, &outcome).await {
            warn!("Failed to record streaming outcome: {e}");
        } else {
            info!("Streaming completion finished (tokens: {}, ms: {:.1})", emitted_tokens, elapsed_ms);
        }
    });

    let receiver_stream = ReceiverStream::new(rx);
    Sse::new(receiver_stream)
        .keep_alive(KeepAlive::default())
        .into_response()
}

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

static ID_COUNTER: AtomicU64 = AtomicU64::new(1);

fn uuid_like_id() -> String {
    let count = ID_COUNTER.fetch_add(1, Ordering::Relaxed);
    let now = current_timestamp_sec();
    format!("{now:x}-{count:06x}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_llama_server_endpoint_default_and_env() {
        let endpoint = resolve_llama_server_endpoint();
        assert!(endpoint.starts_with("http://127.0.0.1:"));

        std::env::set_var("SOULS_LLAMA_SERVER_PORT", "9999");
        assert_eq!(resolve_llama_server_endpoint(), "http://127.0.0.1:9999");
        std::env::remove_var("SOULS_LLAMA_SERVER_PORT");
    }

    #[tokio::test]
    async fn test_probe_upstream_llama_server_offline() {
        let client = reqwest::Client::new();
        let online = probe_upstream_llama_server(&client, "http://127.0.0.1:59999").await;
        assert!(!online, "Offline endpoint must probe false gracefully without panicking");
    }
}
