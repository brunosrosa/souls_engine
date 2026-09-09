//! # Souls Server (`souls_server`)
//!
//! Bare-metal Headless Daemon for the Souls Engine v7.
//! Listens on `127.0.0.1:9123` via Axum and Tokio multithread runtime.
//! Zero UI, Zero Systray, strictly ReFS/NVMe bare-metal execution.

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

    let state = AppState::new(Arc::new(memory), Arc::new(runtime), Arc::new(router));
    info!("All 8 workspace crates successfully bound into AppState.");

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
            eprintln!("CRITICAL: Failed to bind TCP listener to {addr}: {err}");
            return ExitCode::from(1);
        }
    };

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

        AppState::new(Arc::new(memory), Arc::new(runtime), Arc::new(router))
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
}
