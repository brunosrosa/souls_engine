//! # JSON-RPC 2.0 Dispatcher & ReFS Validator (`souls_server::mcp::dispatcher`)
//!
//! Central Model Context Protocol dispatch engine with alias normalization,
//! Dev Drive ReFS containment validation, panic encapsulation, and async telemetry.

use std::path::Path;
use std::time::Instant;
use serde_json::{json, Value};
use tracing::{debug, warn};

use souls_protocol::error::{JsonRpcError, JsonRpcResponse, McpDomainError};

use crate::mcp::handlers;
use crate::mcp::registry::{get_canonical_tools, list_tools};
use crate::state::AppState;

/// Standard MCP protocol version.
pub const MCP_PROTOCOL_VERSION: &str = "2024-11-05";

/// Normalizes aliases and prefixes to canonical tool names (ADR-026 / Zero-Brand).
pub fn normalize_tool_name(mut name: &str) -> &str {
    loop {
        if let Some(rest) = name.strip_prefix("souls_mcp.") {
            name = rest;
        } else if let Some(rest) = name.strip_prefix("souls_") {
            name = rest;
        } else if let Some(rest) = name.strip_prefix("ctx_") {
            name = rest;
        } else {
            break;
        }
    }
    match name {
        "core_think" | "sequential_thinking" | "sequentialthinking" => "thinking",
        "ast_outline" => "outline",
        "ast_slice" => "symbol",
        "myers_diff" => "delta_diff",
        "summon_tool" | "summon" => "tool_search",
        other => other,
    }
}

/// Validates that candidate file path does not escape the Dev Drive ReFS boundary (ADR-005 / Red Line 4).
pub fn validate_refs_path(path_str: &str) -> Result<(), McpDomainError> {
    let trimmed = path_str.trim();
    if trimmed.is_empty() {
        return Ok(());
    }

    // Prohibit toxic drive escapes and env var references
    if trimmed.contains("..\\..")
        || trimmed.contains("../..")
        || trimmed.starts_with("C:")
        || trimmed.starts_with("c:")
        || trimmed.contains("%APPDATA%")
        || trimmed.contains("%LOCALAPPDATA%")
        || trimmed.contains("/temp/")
        || trimmed.contains("\\temp\\")
    {
        warn!("RefsPathLeakViolation intercepted: path escape attempt '{trimmed}'");
        return Err(McpDomainError::RefsPathLeakViolation);
    }

    // If path is absolute, must strictly begin with Z:\ or Z:/
    let p = Path::new(trimmed);
    if p.is_absolute() {
        let normalized = trimmed.replace('/', "\\");
        if !normalized.to_uppercase().starts_with("Z:\\") {
            warn!("RefsPathLeakViolation intercepted: absolute path outside Z: drive '{trimmed}'");
            return Err(McpDomainError::RefsPathLeakViolation);
        }
    }

    Ok(())
}

/// Validates all path-like fields inside tool arguments.
fn validate_tool_arguments_paths(args: &serde_json::Map<String, Value>) -> Result<(), McpDomainError> {
    for (k, v) in args {
        if (k.contains("path") || k == "dir" || k == "file") && v.is_string() {
            if let Some(path_str) = v.as_str() {
                validate_refs_path(path_str)?;
            }
        } else if k == "paths" && v.is_array() {
            if let Some(arr) = v.as_array() {
                for item in arr {
                    if let Some(path_str) = item.as_str() {
                        validate_refs_path(path_str)?;
                    }
                }
            }
        }
    }
    Ok(())
}

/// Dispatches an incoming JSON-RPC 2.0 request.
pub async fn dispatch_jsonrpc(
    state: &AppState,
    payload: Value,
    session_id: Option<&str>,
) -> Option<Value> {
    let start_time = Instant::now();

    let method = payload.get("method").and_then(Value::as_str);
    let id = payload.get("id").cloned().unwrap_or(Value::Null);

    let method_name = match method {
        Some(m) => m,
        None => {
            let err = JsonRpcError::new(-32600, "Invalid Request: missing method field");
            return Some(serde_json::to_value(JsonRpcResponse::<()>::error(id, err)).unwrap_or_default());
        }
    };

    debug!("Dispatching JSON-RPC method: {method_name} (id: {id:?})");

    let params = payload.get("params").cloned().unwrap_or(Value::Null);

    let result: Result<Value, JsonRpcError> = match method_name {
        "initialize" => {
            let client_protocol = params
                .get("protocolVersion")
                .and_then(Value::as_str)
                .unwrap_or(MCP_PROTOCOL_VERSION);

            debug!("MCP client initialized with protocol version: {client_protocol}");

            Ok(json!({
                "protocolVersion": client_protocol,
                "capabilities": {
                    "tools": { "listChanged": true },
                    "resources": { "subscribe": false, "listChanged": false },
                    "prompts": { "listChanged": false }
                },
                "serverInfo": {
                    "name": "souls_mcp",
                    "version": "7.2.0"
                }
            }))
        }

        "notifications/initialized" => {
            // Notifications do not emit a response
            return None;
        }

        "ping" => Ok(json!({})),

        "tools/list" => {
            let full_mode = params
                .get("mode")
                .and_then(Value::as_str)
                .map(|m| m.eq_ignore_ascii_case("full"))
                .or_else(|| params.get("full").and_then(Value::as_bool))
                .unwrap_or(false);

            Ok(list_tools(full_mode))
        }

        "tools/call" => {
            let params_obj = match params.as_object() {
                Some(obj) => obj,
                None => {
                    return Some(serde_json::to_value(JsonRpcResponse::<()>::error(
                        id,
                        JsonRpcError::new(-32602, "Invalid params: expected object"),
                    )).unwrap_or_default());
                }
            };

            let raw_tool_name = match params_obj.get("name").and_then(Value::as_str) {
                Some(name) => name,
                None => {
                    return Some(serde_json::to_value(JsonRpcResponse::<()>::error(
                        id,
                        JsonRpcError::new(-32602, "Missing tool name"),
                    )).unwrap_or_default());
                }
            };

            let norm_tool = normalize_tool_name(raw_tool_name);
            let empty_map = serde_json::Map::new();
            let args = params_obj
                .get("arguments")
                .and_then(Value::as_object)
                .unwrap_or(&empty_map);

            // 1. Validate ReFS path containment
            if let Err(domain_err) = validate_tool_arguments_paths(args) {
                return Some(serde_json::to_value(JsonRpcResponse::<()>::error(id, domain_err.into())).unwrap_or_default());
            }

            // 2. Dispatch to matching tool handler
            let call_result = match norm_tool {
                "route_subtask" => {
                    handlers::handle_route_subtask(state, args)
                        .await
                        .map_err(|e| e.to_jsonrpc_error())
                }
                "sys_time" => Ok(handlers::handle_sys_time()),
                "metrics" => Ok(handlers::handle_metrics(state)),
                "routes" => Ok(handlers::handle_routes()),
                "tool_search" => Ok(handlers::handle_tool_search(args)),
                "cloud_champions" => {
                    handlers::handle_cloud_champions(state, args)
                        .await
                        .map_err(|e| e.to_jsonrpc_error())
                }
                "cloud_arena_sync" => {
                    handlers::handle_cloud_arena_sync(state, args)
                        .await
                        .map_err(|e| e.to_jsonrpc_error())
                }
                "handoff" => {
                    handlers::handle_handoff(state, args)
                        .await
                        .map_err(|e| e.to_jsonrpc_error())
                }
                "sub_agent" => {
                    handlers::handle_sub_agent(state, args)
                        .await
                        .map_err(|e| e.to_jsonrpc_error())
                }
                "thinking" => {
                    handlers::handle_thinking(state, args)
                        .await
                        .map_err(|e| e.to_jsonrpc_error())
                }
                "analyze_session" => {
                    handlers::handle_analyze_session(state, args)
                        .await
                        .map_err(|e| e.to_jsonrpc_error())
                }
                "export_session" => {
                    handlers::handle_export_session(state, args)
                        .await
                        .map_err(|e| e.to_jsonrpc_error())
                }
                "knowledge" => {
                    handlers::handle_knowledge(state, args)
                        .await
                        .map_err(|e| e.to_jsonrpc_error())
                }
                "sqlite_query" => {
                    handlers::handle_sqlite_query(state, args)
                        .await
                        .map_err(|e| e.to_jsonrpc_error())
                }
                "semantic_search" => {
                    handlers::handle_semantic_search(state, args)
                        .await
                        .map_err(|e| e.to_jsonrpc_error())
                }
                "mem_create_entities" => {
                    handlers::handle_mem_create_entities(state, args)
                        .await
                        .map_err(|e| e.to_jsonrpc_error())
                }
                "mem_create_relations" => {
                    handlers::handle_mem_create_relations(state, args)
                        .await
                        .map_err(|e| e.to_jsonrpc_error())
                }
                "outline" => handlers::handle_outline(state, args).map_err(|e| e.to_jsonrpc_error()),
                "symbol" => handlers::handle_symbol(state, args).map_err(|e| e.to_jsonrpc_error()),
                "callers" => handlers::handle_callers(state, args).map_err(|e| e.to_jsonrpc_error()),
                "callees" => handlers::handle_callees(state, args).map_err(|e| e.to_jsonrpc_error()),
                "get_ast" => handlers::handle_get_ast(state, args).map_err(|e| e.to_jsonrpc_error()),
                "delta_diff" => handlers::handle_delta_diff(state, args).map_err(|e| e.to_jsonrpc_error()),
                "edit" => handlers::handle_edit(state, args).map_err(|e| e.to_jsonrpc_error()),
                "replace" => handlers::handle_replace(state, args).map_err(|e| e.to_jsonrpc_error()),
                "repo_heatmap" => handlers::handle_repo_heatmap(state, args).map_err(|e| e.to_jsonrpc_error()),
                "repo_impact" => handlers::handle_repo_impact(state, args).map_err(|e| e.to_jsonrpc_error()),
                "intent" => {
                    handlers::handle_intent(state, args)
                        .await
                        .map_err(|e| e.to_jsonrpc_error())
                }
                other => {
                    // Check if known canonical tool
                    let tools = get_canonical_tools();
                    if tools.iter().any(|t| t.name == other) {
                        Ok(handlers::handle_staging_fallback(other))
                    } else {
                        Err(McpDomainError::ToolNotFoundInNamespace.to_jsonrpc_error())
                    }
                }
            };

            // 3. Asynchronously record non-blocking telemetry in SQLite
            let duration_ms = start_time.elapsed().as_millis() as u64;
            let is_error = call_result.is_err();
            let pool = (*state.memory.pool()).clone();
            let sess_str = session_id.unwrap_or("anon").to_string();
            let tool_str = norm_tool.to_string();

            tokio::spawn(async move {
                let _ = sqlx::query(
                    "INSERT INTO telemetry_events (id, timestamp, tool_name, duration_ms, is_error, session_id)
                     VALUES ($1, strftime('%s','now'), $2, $3, $4, $5)"
                )
                .bind(format!("tel_{:x}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos()))
                .bind(&tool_str)
                .bind(duration_ms as i64)
                .bind(if is_error { 1 } else { 0 })
                .bind(&sess_str)
                .execute(&pool)
                .await;
            });

            call_result
        }

        "resources/list" => Ok(json!({ "resources": [] })),

        "prompts/list" => Ok(json!({ "prompts": [] })),

        other => {
            Err(JsonRpcError::new(-32601, format!("Method not found: {other}")))
        }
    };

    let response = match result {
        Ok(val) => JsonRpcResponse::success(id, val),
        Err(err) => JsonRpcResponse::error(id, err),
    };

    Some(serde_json::to_value(response).unwrap_or_default())
}
