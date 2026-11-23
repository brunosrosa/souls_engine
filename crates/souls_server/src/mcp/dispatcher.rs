//! # JSON-RPC 2.0 Dispatcher & ReFS Validator (`souls_server::mcp::dispatcher`)
//!
//! Central Model Context Protocol dispatch engine with alias normalization,
//! Dev Drive ReFS containment validation, panic encapsulation, and async telemetry.

use std::time::Instant;
use serde_json::{json, Value};
use tracing::{debug, warn};

use souls_protocol::error::{JsonRpcError, JsonRpcResponse, McpDomainError};

use crate::mcp::handlers;
use crate::mcp::registry::{get_canonical_tools, list_tools};
use crate::mcp::tools_extended;
use crate::mcp::tools_google;
use crate::mcp::tools_multimodal;
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
        "dispatch" => "tool_dispatch",
        other => other,
    }
}

/// Validates that candidate file path does not escape the Dev Drive ReFS boundary (ADR-005 / Red Line 4).
pub fn validate_refs_path(path_str: &str) -> Result<(), McpDomainError> {
    let trimmed = path_str.trim();
    if trimmed.is_empty() {
        return Ok(());
    }

    souls_core::fs::resolve_workspace_path(trimmed).map_err(|e| {
        warn!("RefsPathLeakViolation intercepted: path escape attempt '{trimmed}': {e}");
        McpDomainError::RefsPathLeakViolation
    })?;

    Ok(())
}

/// Validates all path-like fields inside tool arguments.
fn validate_tool_arguments_paths(args: &serde_json::Map<String, Value>) -> Result<(), McpDomainError> {
    for (k, v) in args {
        let k_lower = k.to_lowercase();
        if (k_lower.contains("path") || k_lower.contains("dir") || k_lower.contains("file")) && v.is_string() {
            if let Some(path_str) = v.as_str() {
                validate_refs_path(path_str)?;
            }
        } else if (k_lower.contains("paths") || k_lower.contains("files")) && v.is_array() {
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

            let query = params
                .get("query")
                .or_else(|| params.get("intent"))
                .and_then(Value::as_str);

            if let Some(q) = query {
                if !q.trim().is_empty() {
                    let namespace = params.get("namespace").and_then(Value::as_str);
                    let top_k = params.get("top_k").and_then(Value::as_u64).map(|k| k as usize);
                    let decision = crate::mcp::switchboard::evaluate_switchboard(state, q, namespace, top_k).await;
                    let tools_json: Vec<Value> = decision
                        .tools
                        .into_iter()
                        .map(|t| {
                            json!({
                                "name": t.name,
                                "description": t.description,
                                "inputSchema": t.schema,
                            })
                        })
                        .collect();

                    Ok(json!({
                        "tools": tools_json,
                        "switchboard": {
                            "stage": decision.stage,
                            "confidence": decision.confidence,
                            "entropy": decision.entropy,
                            "latency_ms": decision.latency_ms,
                            "recommended_skill": decision.recommended_skill,
                        }
                    }))
                } else {
                    Ok(list_tools(full_mode))
                }
            } else {
                Ok(list_tools(full_mode))
            }
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
                "tool_dispatch" => {
                    let target_tool_raw = match args.get("tool").and_then(Value::as_str) {
                        Some(t) => t,
                        None => {
                            return Some(serde_json::to_value(JsonRpcResponse::<()>::error(
                                id,
                                JsonRpcError::new(-32602, "Missing required 'tool' parameter for tool_dispatch"),
                            )).unwrap_or_default());
                        }
                    };

                    let target_norm = normalize_tool_name(target_tool_raw);
                    if target_norm == "tool_dispatch" {
                        return Some(serde_json::to_value(JsonRpcResponse::<()>::error(
                            id,
                            JsonRpcError::new(-32602, "Recursive dispatch of 'tool_dispatch' is forbidden"),
                        )).unwrap_or_default());
                    }

                    let empty_map = serde_json::Map::new();
                    let target_args = args.get("arguments").and_then(Value::as_object).unwrap_or(&empty_map);

                    if let Err(domain_err) = validate_tool_arguments_paths(target_args) {
                        return Some(serde_json::to_value(JsonRpcResponse::<()>::error(id, domain_err.into())).unwrap_or_default());
                    }

                    let forwarded_req = json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "method": "tools/call",
                        "params": {
                            "name": target_norm,
                            "arguments": target_args
                        }
                    });

                    return Box::pin(dispatch_jsonrpc(state, forwarded_req, session_id)).await;
                }
                "route_subtask" => {
                    handlers::handle_route_subtask(state, args)
                        .await
                        .map_err(|e| e.to_jsonrpc_error())
                }
                "sys_time" => Ok(handlers::handle_sys_time()),
                "metrics" => Ok(handlers::handle_metrics(state)),
                "routes" => Ok(handlers::handle_routes()),
                "tool_search" => Ok(handlers::handle_tool_search(state, args).await),
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
                "profile_list" => {
                    handlers::handle_profile_list(state)
                        .await
                        .map_err(|e| e.to_jsonrpc_error())
                }
                "profile_switch" => {
                    handlers::handle_profile_switch(state, args)
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

                // DOMAIN 6: CONTEXT LEAN
                "read" => tools_extended::handle_read(args).await.map_err(|e| e.to_jsonrpc_error()),
                "smart_read" => tools_extended::handle_smart_read(args).await.map_err(|e| e.to_jsonrpc_error()),
                "multi_read" => tools_extended::handle_multi_read(args).await.map_err(|e| e.to_jsonrpc_error()),
                "compress" => tools_extended::handle_compress(args).await.map_err(|e| e.to_jsonrpc_error()),
                "dedup" => tools_extended::handle_dedup(args).await.map_err(|e| e.to_jsonrpc_error()),
                "fill" => tools_extended::handle_fill(args).await.map_err(|e| e.to_jsonrpc_error()),
                "stub_fill" => tools_extended::handle_stub_fill(args).await.map_err(|e| e.to_jsonrpc_error()),
                "headroom_retrieve" => tools_extended::handle_headroom_retrieve(args).await.map_err(|e| e.to_jsonrpc_error()),

                // DOMAIN 6: SYSTEM, SEARCH & SANDBOX
                "tree" => tools_extended::handle_tree(args).await.map_err(|e| e.to_jsonrpc_error()),
                "search" => tools_extended::handle_search(args).await.map_err(|e| e.to_jsonrpc_error()),
                "shell" => tools_extended::handle_shell(args).await.map_err(|e| e.to_jsonrpc_error()),
                "execute" => tools_extended::handle_execute(args).await.map_err(|e| e.to_jsonrpc_error()),
                "fetch_web" => tools_extended::handle_fetch_web(args).await.map_err(|e| e.to_jsonrpc_error()),
                "web_search" => tools_extended::handle_web_search(args).await.map_err(|e| e.to_jsonrpc_error()),
                "repo_meta" => tools_extended::handle_repo_meta(state, args).await.map_err(|e| e.to_jsonrpc_error()),

                // DOMAIN 3: EXTENDED MEMORY & LADYBUG GRAPH
                "mem_search" => tools_extended::handle_mem_search(state, args).await.map_err(|e| e.to_jsonrpc_error()),
                "mem_open_nodes" => tools_extended::handle_mem_open_nodes(state, args).await.map_err(|e| e.to_jsonrpc_error()),
                "mem_read_graph" => tools_extended::handle_mem_read_graph(state, args).await.map_err(|e| e.to_jsonrpc_error()),
                "mem_add_observations" => tools_extended::handle_mem_add_observations(state, args).await.map_err(|e| e.to_jsonrpc_error()),
                "mem_delete_entities" => tools_extended::handle_mem_delete_entities(state, args).await.map_err(|e| e.to_jsonrpc_error()),
                "mem_delete_relations" => tools_extended::handle_mem_delete_relations(state, args).await.map_err(|e| e.to_jsonrpc_error()),
                "mem_delete_observations" => tools_extended::handle_mem_delete_observations(state, args).await.map_err(|e| e.to_jsonrpc_error()),

                // DOMAIN 4 & 5: EXTENDED THINKING & OPS
                "session" => tools_extended::handle_session(state, args).await.map_err(|e| e.to_jsonrpc_error()),
                "merge_sessions" => tools_extended::handle_merge_sessions(state, args).await.map_err(|e| e.to_jsonrpc_error()),
                "feedback" => tools_extended::handle_feedback(state, args).await.map_err(|e| e.to_jsonrpc_error()),

                // DOMAIN 7: RESEARCH HUB & MISSION CONTROL PLANE (v0.7.0)
                "research_claim" => {
                    let agent_id = args.get("agent_id").and_then(Value::as_str).unwrap_or("hermes_agent");
                    let run_id = args.get("run_id").and_then(Value::as_str);
                    match state.research.claim_queued_mission(agent_id, run_id) {
                        Ok(Some(mission)) => {
                            let mission_dir = state.research.roots().mission_dir(&mission.run_id).unwrap_or_default();
                            Ok(json!({
                                "status": "claimed",
                                "run_id": mission.run_id,
                                "slug": mission.slug,
                                "workspace_path": mission_dir.display().to_string(),
                                "brief": mission.brief,
                                "budget": mission.budget,
                                "claim": mission.claim
                            }))
                        }
                        Ok(None) => Ok(json!({
                            "status": "idle",
                            "message": "Nenhuma missão de pesquisa aguardando na fila"
                        })),
                        Err(e) => Err(JsonRpcError::new(-32000, &e)),
                    }
                }

                "research_event" => {
                    let run_id = match args.get("run_id").and_then(Value::as_str) {
                        Some(rid) => rid,
                        None => return Some(serde_json::to_value(JsonRpcResponse::<()>::error(id, JsonRpcError::new(-32602, "Missing run_id"))).unwrap_or_default()),
                    };
                    let phase = args.get("phase").and_then(Value::as_str).unwrap_or("progress");
                    let round = args.get("round").and_then(Value::as_u64).unwrap_or(1) as u32;
                    let summary = args.get("summary").and_then(Value::as_str).unwrap_or("");
                    let t0 = args.get("tokens_tier0").and_then(Value::as_u64);
                    let t1 = args.get("tokens_tier1").and_then(Value::as_u64);
                    let t2 = args.get("tokens_tier2").and_then(Value::as_u64);
                    let t3 = args.get("tokens_tier3").and_then(Value::as_u64);
                    let usd = args.get("usd_spent").and_then(Value::as_f64);
                    let turns = args.get("turns_used").and_then(Value::as_u64).map(|n| n as u32);

                    match state.research.record_agent_event(run_id, phase, round, summary, t0, t1, t2, t3, usd, turns) {
                        Ok(flags) => Ok(json!({
                            "status": "recorded",
                            "run_id": run_id,
                            "flags": flags
                        })),
                        Err(e) => Err(JsonRpcError::new(-32000, &e)),
                    }
                }

                "research_steering" => {
                    let run_id = match args.get("run_id").and_then(Value::as_str) {
                        Some(rid) => rid,
                        None => return Some(serde_json::to_value(JsonRpcResponse::<()>::error(id, JsonRpcError::new(-32602, "Missing run_id"))).unwrap_or_default()),
                    };
                    if let Some(mission) = state.research.get_mission(run_id) {
                        let steering_notes = if let Ok(dir) = state.research.roots().mission_dir(run_id) {
                            std::fs::read_to_string(dir.join("steering.md")).unwrap_or_default()
                        } else {
                            String::new()
                        };
                        Ok(json!({
                            "run_id": run_id,
                            "flags": mission.flags,
                            "steering_notes": steering_notes
                        }))
                    } else {
                        Err(JsonRpcError::new(-32000, "Missão não encontrada"))
                    }
                }

                "research_complete" => {
                    let run_id = match args.get("run_id").and_then(Value::as_str) {
                        Some(rid) => rid,
                        None => return Some(serde_json::to_value(JsonRpcResponse::<()>::error(id, JsonRpcError::new(-32602, "Missing run_id"))).unwrap_or_default()),
                    };
                    let status_str = args.get("status").and_then(Value::as_str).unwrap_or("confirmed");
                    if let Some(mut mission) = state.research.get_mission(run_id) {
                        let old_status = mission.status;
                        mission.status = match status_str {
                            "confirmed" => crate::research_hub::MissionStatus::Confirmed,
                            "blocked" => crate::research_hub::MissionStatus::Blocked,
                            _ => crate::research_hub::MissionStatus::Failed,
                        };
                        match state.research.upsert_and_persist(mission) {
                            Ok(saved) => {
                                state.research.broadcast(crate::research_hub::ResearchStreamEvent::MissionStatusChanged {
                                    run_id: run_id.to_string(),
                                    old_status,
                                    new_status: saved.status,
                                });
                                Ok(json!({
                                    "status": "completed",
                                    "run_id": run_id,
                                    "mission_status": saved.status
                                }))
                            }
                            Err(e) => Err(JsonRpcError::new(-32000, &e)),
                        }
                    } else {
                        Err(JsonRpcError::new(-32000, "Missão não encontrada"))
                    }
                }

                // DOMAIN 8: COGNITIVE VISION & SOVEREIGN VOICE (v0.8.3 + v0.9.3)
                "ui_inspect" | "ui_interact" | "ui_snapshot" | "ui_diff" | "voice_transcribe"
                | "voice_synthesize" | "voice_rapport" => {
                    match tools_multimodal::try_dispatch_multimodal_tool(
                        state,
                        norm_tool,
                        args,
                        session_id,
                    )
                    .await
                    {
                        Some(res) => res.map_err(|e| e.to_jsonrpc_error()),
                        None => Err(McpDomainError::ToolNotFoundInNamespace.to_jsonrpc_error()),
                    }
                }

                // DOMAIN 9: GOOGLE UNIVERSE & NOTEBOOKLM (v0.8.2 + v0.9.2 Canonical RPCs)
                "notebooklm_rpc" => {
                    let rpc_id_str = match args.get("rpc_id").and_then(Value::as_str) {
                        Some(id) => id,
                        None => return Some(serde_json::to_value(JsonRpcResponse::<()>::error(id, JsonRpcError::new(-32602, "Missing rpc_id"))).unwrap_or_default()),
                    };
                    let rpc_args = args.get("arguments").cloned().unwrap_or(json!([]));
                    let vault_path = crate::routes::google::resolve_google_vault_path();
                    let legacy_creds = souls_core::google_bridge::GoogleSessionVault::load(&vault_path)
                        .ok()
                        .flatten()
                        .filter(|c| c.is_configured());
                    let gv2_creds = if legacy_creds.is_none() {
                        let vault = state.google_hub.vault.read().await;
                        vault
                            .accounts()
                            .iter()
                            .find(|a| a.has_nlm_bridge || a.psid.is_some() || a.sapisid.is_some())
                            .map(|a| a.to_legacy_credentials())
                    } else {
                        None
                    };
                    let creds = match legacy_creds.or(gv2_creds) {
                        Some(c) if c.is_configured() => c,
                        _ => return Some(serde_json::to_value(JsonRpcResponse::<()>::error(id, JsonRpcError::new(-32001, "Google session credentials not found in vault. Authenticate via Dashboard."))).unwrap_or_default()),
                    };

                    let rpc_id = crate::routes::google::resolve_wiz_rpc_id(rpc_id_str);
                    let client = souls_core::google_bridge::WizRpcClient::new(creds);
                    let payload = souls_core::google_bridge::WizRpcPayload::new(rpc_id, rpc_args);
                    match client.execute(&payload).await {
                        Ok(resp) => Ok(json!({
                            "status": "success",
                            "rpc_id": resp.rpc_id,
                            "data": resp.payload,
                            "raw_response": resp.raw_response
                        })),
                        Err(e) => Err(JsonRpcError::new(-32000, format!("Wiz RPC failed: {e}"))),
                    }
                }

                // DOMAIN 10: AGENTIC HARNESS DISPATCH (v0.8.1)
                "agentic_dispatch" => {
                    let cli_str = match args.get("cli").and_then(Value::as_str) {
                        Some(c) => c,
                        None => return Some(serde_json::to_value(JsonRpcResponse::<()>::error(id, JsonRpcError::new(-32602, "Missing cli parameter"))).unwrap_or_default()),
                    };
                    let prompt = match args.get("prompt").and_then(Value::as_str) {
                        Some(p) => p,
                        None => return Some(serde_json::to_value(JsonRpcResponse::<()>::error(id, JsonRpcError::new(-32602, "Missing prompt parameter"))).unwrap_or_default()),
                    };
                    let kind = match cli_str.to_ascii_lowercase().as_str() {
                        "agy" | "antigravity" => souls_core::harness::AgenticCliKind::Antigravity,
                        "claude" | "claudecode" => souls_core::harness::AgenticCliKind::ClaudeCode,
                        "codex" => souls_core::harness::AgenticCliKind::Codex,
                        other => souls_core::harness::AgenticCliKind::Custom(other.to_string()),
                    };
                    let cwd = args.get("working_dir")
                        .and_then(Value::as_str)
                        .map(std::path::PathBuf::from)
                        .unwrap_or_else(|| std::path::PathBuf::from(r"Z:\souls_engine"));
                    let timeout = args.get("timeout_secs").and_then(Value::as_u64).unwrap_or(300);
                    let auto_restore = args.get("auto_restore").and_then(Value::as_bool).unwrap_or(true);

                    let cmd = souls_core::harness::AgenticCliCommand::new(kind, prompt, cwd)
                        .with_timeout_secs(timeout)
                        .with_auto_restore(auto_restore);

                    match souls_core::harness::AgenticHarnessDispatcher::dispatch(&cmd).await {
                        Ok(output) => Ok(json!({
                            "status": "success",
                            "output": output
                        })),
                        Err(e) => Err(JsonRpcError::new(-32000, format!("Agentic dispatch error: {e}"))),
                    }
                }

                other => {
                    if let Some(google_res) =
                        tools_google::try_dispatch_google_tool(state, other, args, session_id).await
                    {
                        google_res.map_err(|e| e.to_jsonrpc_error())
                    } else {
                        // Check if known canonical tool
                        let tools = get_canonical_tools();
                        if tools.iter().any(|t| t.name == other) {
                            Ok(handlers::handle_staging_fallback(other))
                        } else {
                            Err(McpDomainError::ToolNotFoundInNamespace.to_jsonrpc_error())
                        }
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
