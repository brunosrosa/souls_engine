//! # MCP Tool Handlers (`souls_server::mcp::handlers`)
//!
//! Executes active operational tools and serves graceful structured fallbacks
//! for tools awaiting subsequent roadmap migration sessions.

use serde_json::{json, Value};
use souls_model_router::bandit::unpack_gpu_telemetry;
use souls_model_router::context::TaskContext;
use souls_protocol::error::McpDomainError;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::oneshot;

use crate::state::AppState;

/// Handles `route_subtask` via `ParetoBanditRouter`.
pub async fn handle_route_subtask(
    state: &AppState,
    args: &serde_json::Map<String, Value>,
) -> Result<Value, McpDomainError> {
    let task_type = args
        .get("task_type")
        .and_then(Value::as_str)
        .unwrap_or("coding");

    let estimated_tokens = args
        .get("estimated_tokens")
        .and_then(Value::as_u64)
        .unwrap_or(500) as u32;

    let requires_tools = args
        .get("requires_tools")
        .and_then(Value::as_bool)
        .unwrap_or(false);

    let requires_code = args
        .get("requires_code")
        .and_then(Value::as_bool)
        .unwrap_or_else(|| task_type.contains("code") || task_type.contains("coding"));

    let task_ctx = TaskContext::new(
        format!("task_{:x}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis()),
        "subagent_mcp",
        task_type,
        estimated_tokens / 2,
        estimated_tokens / 2,
    )
    .with_tools(requires_tools)
    .with_code_generation(requires_code, Some(5));

    // Execute route_task on dedicated thread to isolate local ThreadRng future
    let router = Arc::clone(&state.router);
    let (tx, rx) = oneshot::channel();
    std::thread::spawn(move || {
        let rt = match tokio::runtime::Builder::new_current_thread().enable_all().build() {
            Ok(r) => r,
            Err(_) => return,
        };
        let res = rt.block_on(async move {
            router.route_task(&task_ctx).await
        });
        let _ = tx.send(res);
    });

    let decision = rx
        .await
        .map_err(|_| McpDomainError::InternalEnginePanic)?
        .map_err(|_| McpDomainError::VramThermalThrottled)?;

    let packed = state.router.gpu_telemetry_atomic().load(Ordering::Acquire);
    let (vram_allocated_mb, gpu_temp_c, zone) = unpack_gpu_telemetry(packed);

    Ok(json!({
        "content": [{
            "type": "text",
            "text": format!(
                "Optimal Model: {} (Tier: {}, Provider: {}, Utility: {:.4}, Latency: {:.1}ms, VRAM: {}MB, Temp: {:.1}°C, Zone: {:?})",
                decision.model_id,
                decision.tier_name,
                decision.provider_type,
                decision.expected_utility,
                decision.projected_latency_ms,
                vram_allocated_mb,
                gpu_temp_c,
                zone
            )
        }],
        "structuredContent": {
            "model_id": decision.model_id,
            "tier_name": decision.tier_name,
            "provider_type": decision.provider_type,
            "expected_utility": decision.expected_utility,
            "sampled_quality": decision.sampled_quality,
            "projected_cost_usd": decision.projected_cost_usd,
            "projected_latency_ms": decision.projected_latency_ms,
            "vram_allocated_mb": vram_allocated_mb,
            "gpu_temp_c": gpu_temp_c,
            "operational_zone": format!("{zone:?}"),
            "thermodynamic_barrier_phi": decision.thermodynamic_barrier_phi,
            "rationale": decision.rationale
        },
        "isError": false
    }))
}

/// Handles `sys_time` query.
pub fn handle_sys_time() -> Value {
    let now = SystemTime::now();
    let epoch_micros = now
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_micros();

    let secs = (epoch_micros / 1_000_000) as u64;
    let subsec_micros = (epoch_micros % 1_000_000) as u32;

    json!({
        "content": [{
            "type": "text",
            "text": format!("Unix Epoch: {secs}.{subsec_micros:06}s")
        }],
        "structuredContent": {
            "epoch_seconds": secs,
            "epoch_micros": epoch_micros,
            "timezone": "UTC"
        },
        "isError": false
    })
}

/// Handles `metrics` hardware watchdog and active connections.
pub fn handle_metrics(state: &AppState) -> Value {
    let (vram_free_mb, vram_total_mb, gpu_temp_c, zone_str, barrier_phi) = {
        let wd = state.runtime.watchdog();
        let wd_guard = wd.lock().unwrap();
        let snap = wd_guard.snapshot();
        let zone_str = match snap.operational_zone {
            souls_inference_runtime::OperationalZone::Nominal => "Nominal",
            souls_inference_runtime::OperationalZone::Preventive => "Preventive",
            souls_inference_runtime::OperationalZone::Critical => "Critical",
        };
        (
            snap.vram_free_mb,
            snap.vram_total_mb,
            snap.gpu_temp_c,
            zone_str.to_string(),
            snap.barrier_phi,
        )
    };

    json!({
        "content": [{
            "type": "text",
            "text": format!(
                "Hardware Telemetry: Free VRAM: {vram_free_mb}/{vram_total_mb} MB | GPU Temp: {gpu_temp_c:.1}°C | Zone: {zone_str} | Barrier Phi: {barrier_phi:.2} | Active HTTP Connections: {}",
                state.active_connections()
            )
        }],
        "structuredContent": {
            "vram_free_mb": vram_free_mb,
            "vram_total_mb": vram_total_mb,
            "gpu_temp_c": gpu_temp_c,
            "operational_zone": zone_str,
            "barrier_phi": barrier_phi,
            "active_connections": state.active_connections(),
            "uptime_secs": state.uptime_secs()
        },
        "isError": false
    })
}

/// Handles `routes` inspection query.
pub fn handle_routes() -> Value {
    json!({
        "content": [{
            "type": "text",
            "text": "Active Endpoints:\n- /health (GET)\n- /v1/models (GET)\n- /v1/chat/completions (POST)\n- /mcp (GET/POST Streamable HTTP 2025/2026)\n- /sse (GET Legacy SSE 2024-11-05)\n- /messages (POST Legacy SSE Messages)"
        }],
        "structuredContent": {
            "endpoints": [
                { "path": "/health", "method": "GET", "protocol": "REST" },
                { "path": "/v1/models", "method": "GET", "protocol": "OpenAI" },
                { "path": "/v1/chat/completions", "method": "POST", "protocol": "OpenAI" },
                { "path": "/mcp", "method": "GET/POST", "protocol": "MCP Streamable HTTP (2025-03-26+)" },
                { "path": "/sse", "method": "GET", "protocol": "MCP Legacy SSE (2024-11-05)" },
                { "path": "/messages", "method": "POST", "protocol": "MCP Legacy SSE (2024-11-05)" }
            ]
        },
        "isError": false
    })
}

/// Handles `tool_search` meta-tool query.
pub fn handle_tool_search(args: &serde_json::Map<String, Value>) -> Value {
    let namespace = args.get("namespace").and_then(Value::as_str);
    let query = args.get("query").and_then(Value::as_str);
    let result = crate::mcp::registry::search_tools(namespace, query);

    json!({
        "content": [{
            "type": "text",
            "text": serde_json::to_string_pretty(&result).unwrap_or_default()
        }],
        "structuredContent": result,
        "isError": false
    })
}

/// Graceful structured staging response for tools whose engines belong to roadmap sessions 2 and 4.
pub fn handle_staging_fallback(tool_name: &str) -> Value {
    let target_session = match tool_name {
        "handoff" | "sub_agent" | "knowledge" | "sqlite_query" | "semantic_search"
        | "mem_search" | "mem_open_nodes" | "mem_read_graph" | "mem_create_entities"
        | "mem_create_relations" | "mem_add_observations" | "mem_delete_entities"
        | "mem_delete_relations" | "mem_delete_observations" => "Session 2 (L3 Deep Memory & SQLite STRICT WAL)",
        "outline" | "symbol" | "callers" | "callees" | "get_ast" | "edit" | "replace"
        | "delta_diff" | "repo_heatmap" | "repo_impact" => "Session 4 (Tree-sitter AST & Myers Diff)",
        _ => "Subsequent Roadmap Session",
    };

    json!({
        "content": [{
            "type": "text",
            "text": format!("[Staging] Tool '{tool_name}' scheduled for {target_session}. Envelope and contract fully validated.")
        }],
        "structuredContent": {
            "tool_name": tool_name,
            "status": "staging_ready",
            "roadmap_session": target_session
        },
        "isError": false,
        "staging": true
    })
}
