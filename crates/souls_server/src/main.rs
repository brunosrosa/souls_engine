//! # Souls Server (`souls_server`)
//!
//! Bare-metal Headless Daemon for the Souls Engine v7.
//! Listens on `127.0.0.1:9123` via Axum and Tokio multithread runtime.
//! Zero UI, Zero Systray, strictly ReFS/NVMe bare-metal execution.

mod mcp;
mod routes;
mod shutdown;
mod state;

use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;

use tracing::info;

use souls_inference_runtime::InferenceRuntime;
use souls_memory::SoulsMemoryStore;
use souls_model_router::bandit::{ParetoBanditRouter, ParetoWeights};

use crate::routes::build_router;
use crate::routes::models::ensure_canonical_models_seeded;
use crate::shutdown::{execute_graceful_shutdown, shutdown_signal};
use crate::state::AppState;

/// Canonical port for the Souls Server headless daemon.
pub const CANONICAL_SERVER_PORT: u16 = 9123;

/// Resolves data directory based on strict precedence:
/// 1. CLI flag `--data-dir <path>` or `--data-dir=<path>`
/// 2. Environment variable `SOULS_DATA_DIR`
/// 3. Default Dev Drive ReFS path `Z:\souls_engine\.souls_data`
pub fn resolve_data_dir() -> PathBuf {
    let args: Vec<String> = std::env::args().collect();
    for i in 0..args.len() {
        if args[i] == "--data-dir" && i + 1 < args.len() {
            return PathBuf::from(&args[i + 1]);
        }
        if let Some(val) = args[i].strip_prefix("--data-dir=") {
            return PathBuf::from(val);
        }
    }

    if let Ok(val) = std::env::var("SOULS_DATA_DIR") {
        let trimmed = val.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed);
        }
    }

    PathBuf::from("Z:/souls_engine/.souls_data")
}

/// Initializes engine subsystems and creates the shared `AppState`.
pub async fn bootstrap_engine(data_dir: &Path) -> Result<AppState, Box<dyn std::error::Error + Send + Sync>> {
    info!("Bootstrapping Souls Engine bare-metal subsystems at: {:?}", data_dir);

    // 1. Bootstrap SoulsMemoryStore (FrankenSQLite STRICT WAL + Ladybug graph)
    let memory = if data_dir == Path::new(":memory:") {
        SoulsMemoryStore::new_in_memory().await?
    } else {
        std::fs::create_dir_all(data_dir)?;
        let db_path = data_dir.join("souls_memory.db");
        SoulsMemoryStore::new(&db_path, 16).await?
    };

    let pool = memory.pool();

    // 2. Ensure router and registry tables exist, then seed canonical models
    ensure_canonical_models_seeded(&pool).await?;

    // 3. Bootstrap ParetoBanditRouter with O(1) atomic telemetry
    let atomic_telemetry = Arc::new(AtomicU64::new(0));
    let router = ParetoBanditRouter::new((*pool).clone(), atomic_telemetry, ParetoWeights::default());

    // 4. Bootstrap InferenceRuntime
    let runtime = InferenceRuntime::new();

    // 5. Bootstrap CloudArenaCoordinator (Nuvem, 5 perfis cognitivos e Fronteira de Pareto)
    let cloud_config = if data_dir == Path::new(":memory:") {
        souls_llm_cloud_arena::CloudArenaConfig {
            database_path: PathBuf::from(":memory:"),
            ..souls_llm_cloud_arena::CloudArenaConfig::load()
        }
    } else {
        souls_llm_cloud_arena::CloudArenaConfig::load()
    };

    let cloud_arena = if data_dir == Path::new(":memory:") {
        souls_llm_cloud_arena::CloudArenaCoordinator::from_pool(cloud_config, (*pool).clone())
    } else {
        souls_llm_cloud_arena::CloudArenaCoordinator::new(cloud_config).await?
    };

    let cloud_arena_arc = Arc::new(cloud_arena);

    // Dispara a sincronização diária em background (não-bloqueante no boot do daemon)
    let cloud_bg = Arc::clone(&cloud_arena_arc);
    tokio::spawn(async move {
        if let Err(err) = cloud_bg.sync_if_needed().await {
            tracing::warn!("Cloud Arena background sync notice: {err}");
        }
    });

    let state = AppState::new(
        Arc::new(memory),
        Arc::new(runtime),
        Arc::new(router),
        cloud_arena_arc,
    );
    info!("All 10 workspace crates successfully bound into AppState.");

    Ok(state)
}

#[tokio::main]
async fn main() -> ExitCode {
    // 1. Initialize structured logging
    let _guard = souls_core::logging::init_tracing("info");
    info!("=== SOULS ENGINE v7 — Headless Daemon (:9123) Boot Sequence ===");

    let data_dir = resolve_data_dir();
    let state = match bootstrap_engine(&data_dir).await {
        Ok(s) => s,
        Err(err) => {
            eprintln!("CRITICAL: Failed to bootstrap engine subsystems: {err}");
            return ExitCode::from(1);
        }
    };

    let app = build_router(state.clone());
    let addr = SocketAddr::from(([127, 0, 0, 1], CANONICAL_SERVER_PORT));

    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => {
            info!("Listening strictly on http://{} (Zero-Trust Loopback Bare-Metal)", addr);
            l
        }
        Err(err) => {
            // Probing if an active healthy instance is already operating
            let client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_millis(800))
                .build()
                .ok();

            if let Some(c) = client {
                let health_url = format!("http://{}/health", addr);
                if let Ok(resp) = c.get(&health_url).send().await {
                    if resp.status().is_success() {
                        info!("NOTICE: Another active Souls Engine instance is already operating healthy on {addr}. Redundant launch exited cleanly.");
                        println!("Souls Engine já está em execução e respondendo normalmente em http://{addr}.");
                        return ExitCode::SUCCESS;
                    }
                }
            }

            eprintln!("CRITICAL: Failed to bind TCP listener to {addr}: {err}");
            eprintln!("Dica: A porta {addr} pode estar presa por um processo órfão ou zumbi. Execute `stop_souls.bat` para forçar a liberação.");
            return ExitCode::from(1);
        }
    };

    // Record PID file for process tracking and zombie cleanup
    let pid_file = data_dir.join("souls_server.pid");
    let _ = std::fs::write(&pid_file, std::process::id().to_string());

    // Run Axum with graceful shutdown on interrupt signal
    if let Err(err) = axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
    {
        eprintln!("Server runtime error: {err}");
    }

    // Execute atomic 5-phase shutdown
    if let Err(err) = execute_graceful_shutdown(&state).await {
        eprintln!("Shutdown protocol warning: {err}");
    }

    // Clean up PID file
    let _ = std::fs::remove_file(&pid_file);

    info!("=== SOULS ENGINE v7 Daemon Shutdown Cleanly (ExitCode::SUCCESS) ===");
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::{to_bytes, Body};
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    async fn create_test_state() -> AppState {
        let memory = SoulsMemoryStore::new_in_memory()
            .await
            .expect("In-memory store creation failed");
        let pool = memory.pool();
        ensure_canonical_models_seeded(&pool)
            .await
            .expect("Seeding models failed");

        let atomic_telemetry = Arc::new(AtomicU64::new(0));
        let router = ParetoBanditRouter::new((*pool).clone(), atomic_telemetry, ParetoWeights::default());
        let runtime = InferenceRuntime::new();

        let cloud_config = souls_llm_cloud_arena::CloudArenaConfig {
            database_path: std::path::PathBuf::from(":memory:"),
            ..souls_llm_cloud_arena::CloudArenaConfig::load()
        };
        let cloud_arena = Arc::new(souls_llm_cloud_arena::CloudArenaCoordinator::from_pool(
            cloud_config,
            (*pool).clone(),
        ));

        AppState::new(Arc::new(memory), Arc::new(runtime), Arc::new(router), cloud_arena)
    }

    #[tokio::test]
    async fn test_health_endpoint() {
        let state = create_test_state().await;
        let app = build_router(state);

        let req = Request::builder()
            .uri("/health")
            .method("GET")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body_bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let val: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

        assert_eq!(val["status"], "healthy");
        assert_eq!(val["sqlite_status"], "ok");
        assert!(val["hardware"]["vram_free_mb"].as_u64().unwrap() > 0);
        assert_eq!(val["hardware"]["operational_zone"], "Nominal");
    }

    #[tokio::test]
    async fn test_models_endpoint() {
        let state = create_test_state().await;
        let app = build_router(state);

        let req = Request::builder()
            .uri("/v1/models")
            .method("GET")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body_bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let val: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

        assert_eq!(val["object"], "list");
        let list = val["data"].as_array().unwrap();
        assert!(list.len() >= 3, "Expected at least 3 seeded canonical models");

        let ids: Vec<&str> = list
            .iter()
            .filter_map(|m| m["id"].as_str())
            .collect();
        assert!(ids.contains(&"tier0_onnx_cpu"));
        assert!(ids.contains(&"tier05_llama_cpu"));
        assert!(ids.contains(&"tier1_qwen_coder_gpu"));
    }

    #[tokio::test]
    async fn test_chat_completions_sync() {
        let state = create_test_state().await;
        let app = build_router(state);

        let payload = serde_json::json!({
            "model": "tier1_qwen_coder_gpu",
            "messages": [
                {"role": "user", "content": "Write a rust function for entropy."}
            ],
            "max_tokens": 16,
            "stream": false
        });

        let req = Request::builder()
            .uri("/v1/chat/completions")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body_bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let val: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

        assert_eq!(val["object"], "chat.completion");
        assert_eq!(val["model"], "tier1_qwen_coder_gpu");
        assert_eq!(val["choices"][0]["finish_reason"], "stop");
        let content = val["choices"][0]["message"]["content"].as_str().unwrap();
        assert!(!content.is_empty());
    }

    #[tokio::test]
    async fn test_chat_completions_sync_json_schema_healing() {
        let state = create_test_state().await;
        let app = build_router(state);

        let payload = serde_json::json!({
            "model": "tier1_qwen_coder_gpu",
            "messages": [
                {"role": "user", "content": "Return json format status"}
            ],
            "response_format": {
                "type": "json_object"
            },
            "max_tokens": 16,
            "stream": false
        });

        let req = Request::builder()
            .uri("/v1/chat/completions")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body_bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let val: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

        let content = val["choices"][0]["message"]["content"].as_str().unwrap();
        // Assert content is valid JSON healed
        let parsed: serde_json::Value = serde_json::from_str(content).expect("Response should be valid JSON");
        assert!(parsed.is_object());
    }

    #[tokio::test]
    async fn test_chat_completions_streaming_sse() {
        let state = create_test_state().await;
        let app = build_router(state);

        let payload = serde_json::json!({
            "model": "tier1_qwen_coder_gpu",
            "messages": [
                {"role": "user", "content": "Hello streaming world"}
            ],
            "max_tokens": 16,
            "stream": true
        });

        let req = Request::builder()
            .uri("/v1/chat/completions")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(resp.headers().get("content-type").unwrap(), "text/event-stream");

        let body_bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let body_str = String::from_utf8_lossy(&body_bytes);

        assert!(body_str.contains("data: "));
        assert!(body_str.contains("[DONE]"), "SSE stream must end with [DONE]");
    }

    #[tokio::test]
    async fn test_connection_raii_guard() {
        let counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 0);

        {
            let _g1 = crate::state::ConnectionGuard::new(Arc::clone(&counter));
            assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 1);
            {
                let _g2 = crate::state::ConnectionGuard::new(Arc::clone(&counter));
                assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 2);
            }
            assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 1);
        }
        assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn test_sse_disconnect_cancellation() {
        let state = create_test_state().await;
        let app = build_router(state);

        let payload = serde_json::json!({
            "model": "tier1_qwen_coder_gpu",
            "messages": [
                {"role": "user", "content": "Generate a long sentence with many words to test disconnect"}
            ],
            "stream": true
        });

        let req = Request::builder()
            .uri("/v1/chat/completions")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // Intentionally drop the response body immediately simulating client TCP abort
        drop(resp);

        // Give worker background task a moment to notice tx.is_closed() and abort cleanly
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    }

    #[tokio::test]
    async fn test_graceful_shutdown_orchestration() {
        let state = create_test_state().await;
        let res = execute_graceful_shutdown(&state).await;
        assert!(res.is_ok(), "5-phase graceful shutdown must succeed");
    }

    #[test]
    fn test_dynamic_data_dir_resolution() {
        // Without env or args, fallback should be Z:\souls_engine\.souls_data
        let dir = resolve_data_dir();
        assert!(dir.to_str().unwrap().contains(".souls_data"));
    }

    #[tokio::test]
    async fn test_mcp_initialize_streamable_http() {
        let state = create_test_state().await;
        let app = build_router(state);

        let payload = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05"
            }
        });

        let req = Request::builder()
            .uri("/mcp")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body_bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let val: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

        assert_eq!(val["jsonrpc"], "2.0");
        assert_eq!(val["id"], 1);
        assert_eq!(val["result"]["protocolVersion"], "2024-11-05");
        assert_eq!(val["result"]["serverInfo"]["name"], "souls_mcp");
        assert_eq!(val["result"]["serverInfo"]["version"], "7.2.0");
        assert!(val["result"]["capabilities"]["tools"]["listChanged"].as_bool().unwrap());
    }

    #[tokio::test]
    async fn test_mcp_tools_list_progressive_disclosure() {
        let state = create_test_state().await;
        let app = build_router(state);

        // 1. Level 0 default compact boot (7 tools)
        let payload_compact = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/list",
            "params": {}
        });

        let req = Request::builder()
            .uri("/mcp")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&payload_compact).unwrap()))
            .unwrap();

        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body_bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let val: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
        let tools_compact = val["result"]["tools"].as_array().unwrap();
        assert_eq!(tools_compact.len(), 7, "Level 0 must expose exactly 7 boot tools");

        // 2. Full mode (52 tools)
        let payload_full = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/list",
            "params": {
                "mode": "full"
            }
        });

        let req_full = Request::builder()
            .uri("/mcp")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&payload_full).unwrap()))
            .unwrap();

        let resp_full = app.oneshot(req_full).await.unwrap();
        assert_eq!(resp_full.status(), StatusCode::OK);
        let body_bytes_full = to_bytes(resp_full.into_body(), usize::MAX).await.unwrap();
        let val_full: serde_json::Value = serde_json::from_slice(&body_bytes_full).unwrap();
        let tools_full = val_full["result"]["tools"].as_array().unwrap();
        assert_eq!(tools_full.len(), 53, "Full mode must expose all 53 canonical tools");
    }

    #[tokio::test]
    async fn test_mcp_tool_search_meta() {
        let state = create_test_state().await;
        let app = build_router(state);

        let payload = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "tools/call",
            "params": {
                "name": "tool_search",
                "arguments": {
                    "namespace": "ops",
                    "query": "route"
                }
            }
        });

        let req = Request::builder()
            .uri("/mcp")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body_bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let val: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
        assert_eq!(val["id"], 4);
        assert!(!val["result"]["isError"].as_bool().unwrap());
        let matched = val["result"]["structuredContent"]["matched_count"].as_u64().unwrap();
        assert!(matched >= 1, "tool_search should find route_subtask in ops");
    }

    #[tokio::test]
    async fn test_mcp_route_subtask_execution() {
        let state = create_test_state().await;
        let app = build_router(state);

        let payload = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 5,
            "method": "tools/call",
            "params": {
                "name": "souls_mcp.route_subtask",
                "arguments": {
                    "task_type": "coding",
                    "estimated_tokens": 800
                }
            }
        });

        let req = Request::builder()
            .uri("/mcp")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body_bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let val: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
        assert_eq!(val["id"], 5);
        assert!(!val["result"]["isError"].as_bool().unwrap());
        let model_id = val["result"]["structuredContent"]["model_id"].as_str().unwrap();
        assert!(!model_id.is_empty());
    }

    #[tokio::test]
    async fn test_mcp_refs_path_containment_violation() {
        let state = create_test_state().await;
        let app = build_router(state);

        let payload = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 6,
            "method": "tools/call",
            "params": {
                "name": "outline",
                "arguments": {
                    "path": "C:\\Windows\\System32\\calc.exe"
                }
            }
        });

        let req = Request::builder()
            .uri("/mcp")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body_bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let val: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
        assert_eq!(val["id"], 6);
        assert_eq!(val["error"]["code"], -32008, "Escape to C: must yield RefsPathLeakViolation (-32008)");
    }

    #[tokio::test]
    async fn test_mcp_legacy_sse_handshake() {
        use tokio_stream::StreamExt;

        let state = create_test_state().await;
        let app = build_router(state);

        // GET /sse initiates stream
        let req = Request::builder()
            .uri("/sse")
            .method("GET")
            .body(Body::empty())
            .unwrap();

        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(resp.headers().get("content-type").unwrap(), "text/event-stream");

        let mut stream = resp.into_body().into_data_stream();
        let first_chunk = stream.next().await.unwrap().unwrap();
        let body_str = String::from_utf8_lossy(&first_chunk);
        assert!(body_str.contains("event: endpoint"));
        assert!(body_str.contains("/messages?sessionId=mcp-"));

        // Extract session ID from endpoint data
        let session_prefix = "/messages?sessionId=";
        let idx = body_str.find(session_prefix).unwrap();
        let rest = &body_str[idx + session_prefix.len()..];
        let session_id = rest.lines().next().unwrap().trim();

        // Send POST to /messages with the session ID
        let payload = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 7,
            "method": "ping"
        });

        let post_req = Request::builder()
            .uri(format!("/messages?sessionId={session_id}"))
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap();

        let post_resp = app.oneshot(post_req).await.unwrap();
        assert_eq!(post_resp.status(), StatusCode::ACCEPTED);
    }

    #[tokio::test]
    async fn test_mcp_knowledge_and_sqlite_query() {
        let state = create_test_state().await;
        let app = build_router(state);

        // 1. Create epistemic knowledge
        let payload = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 10,
            "method": "tools/call",
            "params": {
                "name": "knowledge",
                "arguments": {
                    "id": "fact_hermes_001",
                    "session_id": "sess_test",
                    "category": "architecture",
                    "content": "Souls Engine communicates with Hermes Agent on port 9123.",
                    "partition": "STABLE"
                }
            }
        });

        let req = Request::builder()
            .uri("/mcp")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap();

        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body_bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let val: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
        assert_eq!(val["id"], 10);
        assert!(!val["result"]["isError"].as_bool().unwrap());

        // 2. Query via sqlite_query (read-only)
        let query_payload = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 11,
            "method": "tools/call",
            "params": {
                "name": "sqlite_query",
                "arguments": {
                    "query": "SELECT id, category, content FROM epistemic_memories WHERE id = 'fact_hermes_001'"
                }
            }
        });

        let query_req = Request::builder()
            .uri("/mcp")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&query_payload).unwrap()))
            .unwrap();

        let query_resp = app.clone().oneshot(query_req).await.unwrap();
        assert_eq!(query_resp.status(), StatusCode::OK);
        let query_bytes = to_bytes(query_resp.into_body(), usize::MAX).await.unwrap();
        let query_val: serde_json::Value = serde_json::from_slice(&query_bytes).unwrap();
        assert_eq!(query_val["id"], 11);
        let rows = query_val["result"]["structuredContent"]["rows"].as_array().unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0]["id"], "fact_hermes_001");

        // 3. Search via semantic_search (FTS5)
        let search_payload = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 12,
            "method": "tools/call",
            "params": {
                "name": "semantic_search",
                "arguments": {
                    "query": "Hermes Agent",
                    "limit": 5
                }
            }
        });

        let search_req = Request::builder()
            .uri("/mcp")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&search_payload).unwrap()))
            .unwrap();

        let search_resp = app.oneshot(search_req).await.unwrap();
        assert_eq!(search_resp.status(), StatusCode::OK);
        let search_bytes = to_bytes(search_resp.into_body(), usize::MAX).await.unwrap();
        let search_val: serde_json::Value = serde_json::from_slice(&search_bytes).unwrap();
        assert_eq!(search_val["id"], 12);
        let matches = search_val["result"]["structuredContent"]["matches"].as_array().unwrap();
        assert!(!matches.is_empty(), "FTS5 search should find indexed Hermes memory");
    }

    #[tokio::test]
    async fn test_mcp_sub_agent_and_thinking_cycle() {
        let state = create_test_state().await;
        let app = build_router(state);

        // 1. Spawn sub-agent
        let spawn_payload = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 20,
            "method": "tools/call",
            "params": {
                "name": "sub_agent",
                "arguments": {
                    "action": "spawn",
                    "parent_session_id": "test_sess_01",
                    "task_description": "Analyze compiler logs",
                    "assigned_tier": "Tier1"
                }
            }
        });

        let req = Request::builder()
            .uri("/mcp")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&spawn_payload).unwrap()))
            .unwrap();

        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body_bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let val: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
        assert_eq!(val["id"], 20);
        let agent_id = val["result"]["structuredContent"]["agent_id"].as_str().unwrap();

        // 2. Register socratic thinking
        let think_payload = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 21,
            "method": "tools/call",
            "params": {
                "name": "thinking",
                "arguments": {
                    "session_id": "test_sess_01",
                    "thought": "Initial hypothesis regarding borrow checker constraint.",
                    "thought_number": 1,
                    "total_thoughts": 2,
                    "next_thought_needed": true
                }
            }
        });

        let think_req = Request::builder()
            .uri("/mcp")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&think_payload).unwrap()))
            .unwrap();

        let think_resp = app.clone().oneshot(think_req).await.unwrap();
        assert_eq!(think_resp.status(), StatusCode::OK);

        // 3. Analyze session coherence
        let analyze_payload = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 22,
            "method": "tools/call",
            "params": {
                "name": "analyze_session",
                "arguments": {
                    "session_id": "test_sess_01"
                }
            }
        });

        let analyze_req = Request::builder()
            .uri("/mcp")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&analyze_payload).unwrap()))
            .unwrap();

        let analyze_resp = app.clone().oneshot(analyze_req).await.unwrap();
        assert_eq!(analyze_resp.status(), StatusCode::OK);
        let an_bytes = to_bytes(analyze_resp.into_body(), usize::MAX).await.unwrap();
        let an_val: serde_json::Value = serde_json::from_slice(&an_bytes).unwrap();
        assert_eq!(an_val["result"]["structuredContent"]["total_thoughts"], 1);

        // 4. Create handoff
        let handoff_payload = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 23,
            "method": "tools/call",
            "params": {
                "name": "handoff",
                "arguments": {
                    "source_session_id": "test_sess_01",
                    "target_agent_id": agent_id,
                    "context_summary": "Handing off analysis results.",
                    "variables": { "exit_code": 0 }
                }
            }
        });

        let handoff_req = Request::builder()
            .uri("/mcp")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&handoff_payload).unwrap()))
            .unwrap();

        let handoff_resp = app.oneshot(handoff_req).await.unwrap();
        assert_eq!(handoff_resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_mcp_ladybug_entities_and_relations() {
        let state = create_test_state().await;
        let app = build_router(state);

        // 1. Create entity
        let entity_payload = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 30,
            "method": "tools/call",
            "params": {
                "name": "mem_create_entities",
                "arguments": {
                    "name": "HermesOrchestrator",
                    "entity_type": "daemon",
                    "metadata_json": "{\"language\":\"python\"}"
                }
            }
        });

        let req = Request::builder()
            .uri("/mcp")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&entity_payload).unwrap()))
            .unwrap();

        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // 2. Create target entity
        let target_payload = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 31,
            "method": "tools/call",
            "params": {
                "name": "mem_create_entities",
                "arguments": {
                    "name": "SoulsEngineServer",
                    "entity_type": "daemon",
                    "metadata_json": "{\"language\":\"rust\"}"
                }
            }
        });

        let target_req = Request::builder()
            .uri("/mcp")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&target_payload).unwrap()))
            .unwrap();

        let target_resp = app.clone().oneshot(target_req).await.unwrap();
        assert_eq!(target_resp.status(), StatusCode::OK);

        // 3. Create relation
        let rel_payload = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 32,
            "method": "tools/call",
            "params": {
                "name": "mem_create_relations",
                "arguments": {
                    "source": "HermesOrchestrator",
                    "target": "SoulsEngineServer",
                    "relationship": "communicates_with"
                }
            }
        });

        let rel_req = Request::builder()
            .uri("/mcp")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&rel_payload).unwrap()))
            .unwrap();

        let rel_resp = app.oneshot(rel_req).await.unwrap();
        assert_eq!(rel_resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_mcp_ast_outline_and_symbol_integration() {
        let state = create_test_state().await;
        let app = build_router(state);

        let test_file_path = "Z:/souls_engine/crates/souls_server/src/test_fixture_ast.rs";
        let test_code = r#"
pub struct FixtureService {
    pub name: String,
}

impl FixtureService {
    pub fn process_data(&self) -> bool {
        let _temp = 100;
        let _val = 200;
        true
    }
}
"#;
        std::fs::write(test_file_path, test_code).expect("write test fixture");

        // 1. Outline call
        let outline_payload = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 40,
            "method": "tools/call",
            "params": {
                "name": "outline",
                "arguments": {
                    "path": test_file_path,
                    "include_private": true
                }
            }
        });

        let req = Request::builder()
            .uri("/mcp")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&outline_payload).unwrap()))
            .unwrap();

        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body_bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let val: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
        assert_eq!(val["id"], 40);
        assert!(!val["result"]["isError"].as_bool().unwrap());
        assert!(val["result"]["structuredContent"]["saved_tokens_pct"].as_f64().unwrap() > 0.0);

        // 2. Symbol call
        let symbol_payload = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 41,
            "method": "tools/call",
            "params": {
                "name": "symbol",
                "arguments": {
                    "path": test_file_path,
                    "symbol_query": "process_data"
                }
            }
        });

        let sym_req = Request::builder()
            .uri("/mcp")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&symbol_payload).unwrap()))
            .unwrap();

        let sym_resp = app.clone().oneshot(sym_req).await.unwrap();
        assert_eq!(sym_resp.status(), StatusCode::OK);
        let sym_bytes = to_bytes(sym_resp.into_body(), usize::MAX).await.unwrap();
        let sym_val: serde_json::Value = serde_json::from_slice(&sym_bytes).unwrap();
        assert_eq!(sym_val["id"], 41);
        assert_eq!(sym_val["result"]["structuredContent"]["symbol_name"], "process_data");

        // Clean up fixture file
        let _ = std::fs::remove_file(test_file_path);
    }

    #[tokio::test]
    async fn test_mcp_intent_probing_integration() {
        let state = create_test_state().await;
        let app = build_router(state);

        let payload = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 42,
            "method": "tools/call",
            "params": {
                "name": "intent",
                "arguments": {
                    "prompt": "pub fn refactor_ast_slice(tree: &SyntaxTree) -> Result<()>"
                }
            }
        });

        let req = Request::builder()
            .uri("/mcp")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body_bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let val: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
        assert_eq!(val["id"], 42);
        assert!(!val["result"]["isError"].as_bool().unwrap());
        assert_eq!(val["result"]["structuredContent"]["intent"], "code");
        assert_eq!(val["result"]["structuredContent"]["vram_mb"], 0);
    }

    #[tokio::test]
    async fn test_cloud_champions_rest_and_mcp_integration() {
        let state = create_test_state().await;
        // Seed initial sync on test state
        let _ = state.cloud_arena.force_sync(Some("2026-10-12")).await;

        let app = build_router(state);

        // 1. REST GET /api/v1/cloud/champions
        let req = Request::builder()
            .uri("/api/v1/cloud/champions?date=2026-10-12")
            .method("GET")
            .body(Body::empty())
            .unwrap();

        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body_bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let val: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
        assert_eq!(val["status"], "success");
        assert_eq!(val["profiles_count"], 5);

        // 2. MCP tools/call cloud_champions
        let mcp_payload = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 50,
            "method": "tools/call",
            "params": {
                "name": "cloud_champions",
                "arguments": {
                    "profile": "coding",
                    "date": "2026-10-12"
                }
            }
        });

        let mcp_req = Request::builder()
            .uri("/mcp")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&mcp_payload).unwrap()))
            .unwrap();

        let mcp_resp = app.clone().oneshot(mcp_req).await.unwrap();
        assert_eq!(mcp_resp.status(), StatusCode::OK);
        let mcp_bytes = to_bytes(mcp_resp.into_body(), usize::MAX).await.unwrap();
        let mcp_val: serde_json::Value = serde_json::from_slice(&mcp_bytes).unwrap();
        assert_eq!(mcp_val["id"], 50);
        assert!(!mcp_val["result"]["isError"].as_bool().unwrap());
    }

    #[tokio::test]
    async fn test_dashboard_html_and_telemetry_endpoints() {
        let state = create_test_state().await;
        let app = build_router(state);

        // 1. Testa GET / (SPA HTML)
        let req_root = Request::builder()
            .uri("/")
            .method("GET")
            .body(Body::empty())
            .unwrap();
        let resp_root = app.clone().oneshot(req_root).await.unwrap();
        assert_eq!(resp_root.status(), StatusCode::OK);
        let bytes_root = to_bytes(resp_root.into_body(), usize::MAX).await.unwrap();
        let html = String::from_utf8(bytes_root.to_vec()).unwrap();
        assert!(html.contains("Souls Engine v7"));
        assert!(html.contains("Silício Físico"));

        // 2. Testa GET /dashboard
        let req_dash = Request::builder()
            .uri("/dashboard")
            .method("GET")
            .body(Body::empty())
            .unwrap();
        let resp_dash = app.clone().oneshot(req_dash).await.unwrap();
        assert_eq!(resp_dash.status(), StatusCode::OK);

        // 3. Testa GET /dashboard/icon/cyan
        let req_icon = Request::builder()
            .uri("/dashboard/icon/cyan")
            .method("GET")
            .body(Body::empty())
            .unwrap();
        let resp_icon = app.clone().oneshot(req_icon).await.unwrap();
        assert_eq!(resp_icon.status(), StatusCode::OK);
        assert_eq!(resp_icon.headers().get("content-type").unwrap(), "image/png");

        // 4. Testa GET /api/v1/telemetry
        let req_telemetry = Request::builder()
            .uri("/api/v1/telemetry")
            .method("GET")
            .body(Body::empty())
            .unwrap();
        let resp_telemetry = app.clone().oneshot(req_telemetry).await.unwrap();
        assert_eq!(resp_telemetry.status(), StatusCode::OK);
        let bytes_telemetry = to_bytes(resp_telemetry.into_body(), usize::MAX).await.unwrap();
        let json_telemetry: serde_json::Value = serde_json::from_slice(&bytes_telemetry).unwrap();
        assert_eq!(json_telemetry["status"], "online");
        assert_eq!(json_telemetry["port"], 9123);
        assert_eq!(json_telemetry["vram_limit_mb"], 5294);

        // 5. Testa GET /api/v1/models/local
        let req_models = Request::builder()
            .uri("/api/v1/models/local")
            .method("GET")
            .body(Body::empty())
            .unwrap();
        let resp_models = app.clone().oneshot(req_models).await.unwrap();
        assert_eq!(resp_models.status(), StatusCode::OK);
        let bytes_models = to_bytes(resp_models.into_body(), usize::MAX).await.unwrap();
        let json_models: serde_json::Value = serde_json::from_slice(&bytes_models).unwrap();
        assert!(json_models["models"].as_array().unwrap().len() >= 4);

        // 6. Testa POST /api/v1/settings/hermes-preset
        let req_preset = Request::builder()
            .uri("/api/v1/settings/hermes-preset")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"preset_id":"balanced"}"#))
            .unwrap();
        let resp_preset = app.clone().oneshot(req_preset).await.unwrap();
        assert_eq!(resp_preset.status(), StatusCode::OK);

        // 7. Testa POST /api/v1/system/checkpoint
        let req_ckpt = Request::builder()
            .uri("/api/v1/system/checkpoint")
            .method("POST")
            .body(Body::empty())
            .unwrap();
        let resp_ckpt = app.clone().oneshot(req_ckpt).await.unwrap();
        assert_eq!(resp_ckpt.status(), StatusCode::OK);

        // 8. Testa GET /api/v1/settings/config (souls_config.toml)
        let req_cfg = Request::builder()
            .uri("/api/v1/settings/config")
            .method("GET")
            .body(Body::empty())
            .unwrap();
        let resp_cfg = app.clone().oneshot(req_cfg).await.unwrap();
        assert_eq!(resp_cfg.status(), StatusCode::OK);
        let bytes_cfg = to_bytes(resp_cfg.into_body(), usize::MAX).await.unwrap();
        let json_cfg: serde_json::Value = serde_json::from_slice(&bytes_cfg).unwrap();
        assert_eq!(json_cfg["status"], "success");
        assert!(json_cfg["raw_toml"].as_str().unwrap().contains("port = 9123"));

        // 9. Testa POST /api/v1/settings/config (atualização atômica)
        let req_post_cfg = Request::builder()
            .uri("/api/v1/settings/config")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(r##"{"raw_toml":"# souls_config.toml\n[server]\nport = 9123\n"}"##))
            .unwrap();
        let resp_post_cfg = app.clone().oneshot(req_post_cfg).await.unwrap();
        assert_eq!(resp_post_cfg.status(), StatusCode::OK);

        // 10. Testa GET /api/v1/telemetry/stream (Server-Sent Events)
        let req_stream = Request::builder()
            .uri("/api/v1/telemetry/stream")
            .method("GET")
            .body(Body::empty())
            .unwrap();
        let resp_stream = app.clone().oneshot(req_stream).await.unwrap();
        assert_eq!(resp_stream.status(), StatusCode::OK);
        assert_eq!(resp_stream.headers().get("content-type").unwrap(), "text/event-stream");
    }

    #[tokio::test]
    async fn test_mcp_domain6_and_extended_tools() {
        let state = create_test_state().await;
        let app = build_router(state);

        // Helper macro/closure to call tools
        let call_tool = |name: &'static str, args: serde_json::Value| {
            let app_clone = app.clone();
            async move {
                let payload = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": 100,
                    "method": "tools/call",
                    "params": {
                        "name": name,
                        "arguments": args
                    }
                });

                let req = Request::builder()
                    .uri("/mcp")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                    .unwrap();

                let resp = app_clone.oneshot(req).await.unwrap();
                assert_eq!(resp.status(), StatusCode::OK);
                let bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
                let json_val: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
                assert_eq!(json_val["id"], 100);
                assert!(!json_val["result"]["isError"].as_bool().unwrap());
                json_val["result"].clone()
            }
        };

        // 1. Testa 'compress'
        let res_compress = call_tool("compress", serde_json::json!({
            "text": "// Comment line\nfn test() {\n    let a = 1; // inline comment\n\n\n    return a;\n}"
        })).await;
        let text_comp = res_compress["content"][0]["text"].as_str().unwrap();
        assert!(text_comp.contains("fn test()"));
        assert!(!text_comp.contains("// Comment line"));

        // 2. Testa 'dedup' e 'fill'
        let repeated_text = "line 1\nline 2\nline 3\nline 4\nline 5\ncode\nline 1\nline 2\nline 3\nline 4\nline 5";
        let res_dedup = call_tool("dedup", serde_json::json!({
            "text": repeated_text,
            "min_lines": 5
        })).await;
        let dedup_out = res_dedup["content"][0]["text"].as_str().unwrap();
        assert!(dedup_out.contains("DEDUP_BLOCK_"));

        let res_fill = call_tool("fill", serde_json::json!({
            "text": dedup_out
        })).await;
        let fill_out = res_fill["content"][0]["text"].as_str().unwrap();
        assert_eq!(fill_out, repeated_text);

        // 3. Testa 'tree'
        let res_tree = call_tool("tree", serde_json::json!({
            "path": "Z:/souls_engine",
            "max_depth": 2
        })).await;
        assert!(res_tree["content"][0]["text"].as_str().unwrap().contains("crates/"));

        // 4. Testa 'search'
        let res_search = call_tool("search", serde_json::json!({
            "path": "Z:/souls_engine/crates/souls_server",
            "query": "build_router",
            "file_pattern": "*.rs"
        })).await;
        assert!(res_search["content"][0]["text"].as_str().unwrap().contains("build_router"));

        // 5. Testa 'session'
        let res_session = call_tool("session", serde_json::json!({
            "action": "status"
        })).await;
        assert_eq!(res_session["structuredContent"]["status"], "active");

        // 6. Testa 'feedback'
        let res_fb = call_tool("feedback", serde_json::json!({})).await;
        assert!(res_fb["structuredContent"]["vram_free_mb"].as_u64().unwrap() > 0);

        // 7. Testa 'mem_read_graph'
        let res_graph = call_tool("mem_read_graph", serde_json::json!({
            "limit": 10
        })).await;
        assert!(res_graph["structuredContent"]["nodes"].is_array());
    }
}
