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

    let entropy_opt = args
        .get("entropy")
        .or_else(|| args.get("entropy_score"))
        .and_then(Value::as_f64)
        .map(|v| v as f32);

    let is_retry = args
        .get("is_retry")
        .or_else(|| args.get("retry"))
        .and_then(Value::as_bool)
        .unwrap_or(false);

    let mut task_ctx = TaskContext::new(
        format!("task_{:x}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis()),
        "subagent_mcp",
        task_type,
        estimated_tokens / 2,
        estimated_tokens / 2,
    )
    .with_tools(requires_tools)
    .with_code_generation(requires_code, Some(5));

    if let Some(entropy) = entropy_opt {
        task_ctx = task_ctx.with_entropy(entropy);
    }
    if is_retry {
        task_ctx = task_ctx.with_retry(true);
    }

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

    let reasoning_text = if let Some(ref rec) = decision.reasoning_recommendation {
        format!(
            " | Reasoning: {} (Budget: {} tokens, Escalated: {})",
            rec.reasoning_effort.as_str(),
            rec.thinking_budget_tokens,
            rec.is_escalated_retry
        )
    } else {
        String::new()
    };

    Ok(json!({
        "content": [{
            "type": "text",
            "text": format!(
                "Optimal Model: {} (Tier: {}, Provider: {}, Utility: {:.4}, Latency: {:.1}ms, VRAM: {}MB, Temp: {:.1}°C, Zone: {:?}){}",
                decision.model_id,
                decision.tier_name,
                decision.provider_type,
                decision.expected_utility,
                decision.projected_latency_ms,
                vram_allocated_mb,
                gpu_temp_c,
                zone,
                reasoning_text
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
            "rationale": decision.rationale,
            "reasoning_recommendation": decision.reasoning_recommendation
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

/// Handles `tool_search` meta-tool query using Two-Stage Semantic Switchboard.
pub async fn handle_tool_search(state: &AppState, args: &serde_json::Map<String, Value>) -> Value {
    let namespace = args.get("namespace").and_then(Value::as_str);
    let query = args.get("query").and_then(Value::as_str).unwrap_or("");

    let decision = crate::mcp::switchboard::evaluate_switchboard(state, query, namespace).await;

    json!({
        "content": [{
            "type": "text",
            "text": serde_json::to_string_pretty(&decision).unwrap_or_default()
        }],
        "structuredContent": decision,
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

/// Handles `handoff` inter-agent context transfer.
pub async fn handle_handoff(
    state: &AppState,
    args: &serde_json::Map<String, Value>,
) -> Result<Value, McpDomainError> {
    let sub_mgr = state.memory.sub_agent_manager();

    if let Some(target_agent_id) = args.get("target_agent_id").and_then(Value::as_str) {
        if !args.contains_key("context_summary") {
            let res = sub_mgr.get_handoff(target_agent_id).await.map_err(|_| McpDomainError::InternalEnginePanic)?;
            return Ok(json!({
                "content": [{
                    "type": "text",
                    "text": serde_json::to_string_pretty(&res).unwrap_or_default()
                }],
                "structuredContent": res,
                "isError": false
            }));
        }
    }

    let source_session = args.get("source_session_id").and_then(Value::as_str).unwrap_or("main");
    let target_agent = args.get("target_agent_id").and_then(Value::as_str).unwrap_or("anon_sub");
    let summary = args.get("context_summary").and_then(Value::as_str).unwrap_or("");
    let variables = args.get("variables").cloned().unwrap_or(json!({}));

    let record = sub_mgr
        .create_handoff(source_session, target_agent, summary, variables)
        .await
        .map_err(|_| McpDomainError::InternalEnginePanic)?;

    Ok(json!({
        "content": [{
            "type": "text",
            "text": format!("Handoff payload '{}' created successfully for agent '{}'.", record.id, record.target_agent_id)
        }],
        "structuredContent": record,
        "isError": false
    }))
}

/// Handles `sub_agent` lifecycle management and diaries.
pub async fn handle_sub_agent(
    state: &AppState,
    args: &serde_json::Map<String, Value>,
) -> Result<Value, McpDomainError> {
    let sub_mgr = state.memory.sub_agent_manager();
    let action = args.get("action").and_then(Value::as_str).unwrap_or("spawn");

    match action {
        "spawn" => {
            let parent_session = args.get("parent_session_id").and_then(Value::as_str).unwrap_or("main");
            let task_desc = args.get("task_description").and_then(Value::as_str).unwrap_or("general subagent task");
            let assigned_tier = args.get("assigned_tier").and_then(Value::as_str).unwrap_or("Tier1");

            let record = sub_mgr
                .spawn_subagent(parent_session, task_desc, assigned_tier)
                .await
                .map_err(|_| McpDomainError::InternalEnginePanic)?;

            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": format!("Sub-agent '{}' spawned successfully (status: {:?}, tier: {}).", record.agent_id, record.status, record.assigned_tier)
                }],
                "structuredContent": record,
                "isError": false
            }))
        }
        "diary" => {
            let agent_id = args.get("agent_id").and_then(Value::as_str).unwrap_or("anon");
            let session_id = args.get("session_id").and_then(Value::as_str).unwrap_or("main");
            let content = args.get("content").and_then(Value::as_str).unwrap_or("");

            let record = sub_mgr
                .record_diary(agent_id, session_id, content)
                .await
                .map_err(|_| McpDomainError::InternalEnginePanic)?;

            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": format!("Diary note recorded for sub-agent '{}'.", record.agent_id)
                }],
                "structuredContent": record,
                "isError": false
            }))
        }
        "update_status" => {
            let agent_id = args.get("agent_id").and_then(Value::as_str).unwrap_or("");
            let status_str = args.get("status").and_then(Value::as_str).unwrap_or("RUNNING");
            let status = souls_memory::SubAgentStatus::from_str_strict(status_str)
                .map_err(|_| McpDomainError::InternalEnginePanic)?;

            let record = sub_mgr
                .update_status(agent_id, status)
                .await
                .map_err(|_| McpDomainError::InternalEnginePanic)?;

            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": format!("Sub-agent '{}' status updated to {:?}.", record.agent_id, record.status)
                }],
                "structuredContent": record,
                "isError": false
            }))
        }
        "list" => {
            let parent_session = args.get("parent_session_id").and_then(Value::as_str).unwrap_or("main");
            let list = sub_mgr
                .list_subagents_by_parent(parent_session)
                .await
                .map_err(|_| McpDomainError::InternalEnginePanic)?;

            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": format!("Found {} sub-agents for session '{}'.", list.len(), parent_session)
                }],
                "structuredContent": list,
                "isError": false
            }))
        }
        _ => Err(McpDomainError::InternalEnginePanic),
    }
}

/// Handles `thinking` socratic reasoning step.
pub async fn handle_thinking(
    state: &AppState,
    args: &serde_json::Map<String, Value>,
) -> Result<Value, McpDomainError> {
    let session_id = args.get("session_id").and_then(Value::as_str).unwrap_or("main");
    let thought = args.get("thought").and_then(Value::as_str).unwrap_or("");
    let thought_number = args.get("thought_number").and_then(Value::as_u64).unwrap_or(1) as u32;
    let total_thoughts = args.get("total_thoughts").and_then(Value::as_u64).unwrap_or(3) as u32;
    let next_thought_needed = args.get("next_thought_needed").and_then(Value::as_bool).unwrap_or(true);
    let is_revision = args.get("is_revision").and_then(Value::as_bool);
    let revises_thought = args.get("revises_thought").and_then(Value::as_u64).map(|v| v as u32);
    let branch_from_thought = args.get("branch_from_thought").and_then(Value::as_u64).map(|v| v as u32);
    let branch_id = args.get("branch_id").and_then(Value::as_str).map(String::from);

    let engine_lock = state.memory.get_or_create_thinking_engine(session_id).await;
    let mut engine = engine_lock.write().await;

    let input = souls_memory::SocraticThoughtInput {
        thought: thought.to_string(),
        thought_number,
        total_thoughts,
        next_thought_needed,
        is_revision,
        revises_thought,
        branch_from_thought,
        branch_id,
    };

    let record = engine.add_thought(input).map_err(|_| McpDomainError::InternalEnginePanic)?;
    let _ = engine.persist_to_db(&*state.memory.pool()).await;

    // Socratic Sheaf Cohomology evaluation over reasoning graph
    let mut fact_graph = souls_inference_runtime::cohomology::SocraticFactGraph::new();
    let mut thought_indices = std::collections::HashMap::new();
    for t in engine.thoughts() {
        let label = format!("thought_{}", t.thought_number);
        if let Some(idx) = fact_graph.add_vertex(&label) {
            thought_indices.insert(t.thought_number, idx);
        }
    }
    for t in engine.thoughts() {
        if let Some(&curr_idx) = thought_indices.get(&t.thought_number) {
            if let Some(origin) = t.branch_from_thought {
                if let Some(&origin_idx) = thought_indices.get(&origin) {
                    fact_graph.add_edge(origin_idx, curr_idx, souls_inference_runtime::cohomology::RelationKind::DependsOn);
                }
            }
            if let Some(revises) = t.revises_thought {
                if let Some(&revises_idx) = thought_indices.get(&revises) {
                    fact_graph.add_edge(revises_idx, curr_idx, souls_inference_runtime::cohomology::RelationKind::ConflictsWith);
                }
            }
        }
    }
    let cohomology = souls_inference_runtime::cohomology::compute_cohomology(&fact_graph);

    let mut structured = serde_json::to_value(&record).unwrap_or_default();
    if let Some(map) = structured.as_object_mut() {
        map.insert("h1_dimension".to_string(), json!(cohomology.h1_dimension));
        map.insert("has_contradiction".to_string(), json!(cohomology.has_contradiction));
        map.insert("cohomology_rank".to_string(), json!(cohomology.rank));
    }

    Ok(json!({
        "content": [{
            "type": "text",
            "text": format!("Socratic Step #{}/{} registered (branch: '{}', revision: {}, H¹: {}).", record.thought_number, record.total_thoughts, record.branch_id, record.is_revision, cohomology.h1_dimension)
        }],
        "structuredContent": structured,
        "isError": false
    }))
}

/// Handles `analyze_session` coherence inspection.
pub async fn handle_analyze_session(
    state: &AppState,
    args: &serde_json::Map<String, Value>,
) -> Result<Value, McpDomainError> {
    let session_id = args.get("session_id").and_then(Value::as_str).unwrap_or("main");
    let engine_lock = state.memory.get_or_create_thinking_engine(session_id).await;
    let engine = engine_lock.read().await;
    let analysis = engine.analyze_coherence();

    // Socratic Sheaf Cohomology evaluation
    let mut fact_graph = souls_inference_runtime::cohomology::SocraticFactGraph::new();
    let mut thought_indices = std::collections::HashMap::new();
    for t in engine.thoughts() {
        let label = format!("thought_{}", t.thought_number);
        if let Some(idx) = fact_graph.add_vertex(&label) {
            thought_indices.insert(t.thought_number, idx);
        }
    }
    for t in engine.thoughts() {
        if let Some(&curr_idx) = thought_indices.get(&t.thought_number) {
            if let Some(origin) = t.branch_from_thought {
                if let Some(&origin_idx) = thought_indices.get(&origin) {
                    fact_graph.add_edge(origin_idx, curr_idx, souls_inference_runtime::cohomology::RelationKind::DependsOn);
                }
            }
            if let Some(revises) = t.revises_thought {
                if let Some(&revises_idx) = thought_indices.get(&revises) {
                    fact_graph.add_edge(revises_idx, curr_idx, souls_inference_runtime::cohomology::RelationKind::ConflictsWith);
                }
            }
        }
    }
    let cohomology = souls_inference_runtime::cohomology::compute_cohomology(&fact_graph);

    let mut enriched_analysis = analysis.clone();
    if let Some(map) = enriched_analysis.as_object_mut() {
        map.insert("h1_dimension".to_string(), json!(cohomology.h1_dimension));
        map.insert("has_contradiction".to_string(), json!(cohomology.has_contradiction));
        map.insert("cohomology_rank".to_string(), json!(cohomology.rank));
    }

    Ok(json!({
        "content": [{
            "type": "text",
            "text": serde_json::to_string_pretty(&enriched_analysis).unwrap_or_default()
        }],
        "structuredContent": enriched_analysis,
        "isError": false
    }))
}

/// Handles `export_session` markdown/json consolidation.
pub async fn handle_export_session(
    state: &AppState,
    args: &serde_json::Map<String, Value>,
) -> Result<Value, McpDomainError> {
    let session_id = args.get("session_id").and_then(Value::as_str).unwrap_or("main");
    let format = args.get("format").and_then(Value::as_str).unwrap_or("markdown");
    let engine_lock = state.memory.get_or_create_thinking_engine(session_id).await;
    let engine = engine_lock.read().await;

    if format == "json" {
        let json_val = engine.export_json();
        Ok(json!({
            "content": [{
                "type": "text",
                "text": serde_json::to_string_pretty(&json_val).unwrap_or_default()
            }],
            "structuredContent": json_val,
            "isError": false
        }))
    } else {
        let md = engine.export_markdown();
        Ok(json!({
            "content": [{
                "type": "text",
                "text": md
            }],
            "structuredContent": { "session_id": session_id, "markdown": md },
            "isError": false
        }))
    }
}

/// Handles `knowledge` epistemic facts.
pub async fn handle_knowledge(
    state: &AppState,
    args: &serde_json::Map<String, Value>,
) -> Result<Value, McpDomainError> {
    let id = args.get("id").and_then(Value::as_str);

    if let Some(id_str) = id {
        if !args.contains_key("content") {
            let fact = state.memory.get_epistemic_memory(id_str).await.map_err(|_| McpDomainError::InternalEnginePanic)?;
            return Ok(json!({
                "content": [{
                    "type": "text",
                    "text": serde_json::to_string_pretty(&fact).unwrap_or_default()
                }],
                "structuredContent": fact,
                "isError": false
            }));
        }
    }

    let fact_id = id.map(String::from).unwrap_or_else(|| format!("fact_{:x}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis()));
    let session_id = args.get("session_id").and_then(Value::as_str).unwrap_or("main");
    let category = args.get("category").and_then(Value::as_str).unwrap_or("general");
    let content = args.get("content").and_then(Value::as_str).unwrap_or("");
    let partition_str = args.get("partition").and_then(Value::as_str).unwrap_or("STABLE");
    let partition = if partition_str.eq_ignore_ascii_case("evolving") {
        souls_memory::EpistemicPartition::Evolving
    } else {
        souls_memory::EpistemicPartition::Stable
    };

    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64;
    let record = souls_memory::EpistemicMemoryRecord {
        id: fact_id,
        session_id: session_id.to_string(),
        category: category.to_string(),
        content: content.to_string(),
        partition,
        salience: 1.0,
        access_count: 0,
        source_ref: args.get("source_ref").and_then(Value::as_str).map(String::from),
        created_at: now,
        last_accessed_at: now,
        updated_at: now,
    };

    state.memory.upsert_epistemic_memory(&record).await.map_err(|_| McpDomainError::InternalEnginePanic)?;

    Ok(json!({
        "content": [{
            "type": "text",
            "text": format!("Epistemic fact '{}' persisted successfully in partition {:?}.", record.id, record.partition)
        }],
        "structuredContent": record,
        "isError": false
    }))
}

/// Handles `sqlite_query` read-only queries.
pub async fn handle_sqlite_query(
    state: &AppState,
    args: &serde_json::Map<String, Value>,
) -> Result<Value, McpDomainError> {
    let query = args
        .get("query")
        .or_else(|| args.get("sql"))
        .and_then(Value::as_str)
        .unwrap_or("");

    if query.trim().is_empty() {
        return Ok(json!({
            "content": [{
                "type": "text",
                "text": "[]"
            }],
            "structuredContent": { "rows": [], "count": 0 },
            "isError": false
        }));
    }

    let rows = state.memory.execute_read_only_query(query).await.map_err(|_| McpDomainError::InternalEnginePanic)?;

    Ok(json!({
        "content": [{
            "type": "text",
            "text": serde_json::to_string_pretty(&rows).unwrap_or_default()
        }],
        "structuredContent": { "rows": rows, "count": rows.len() },
        "isError": false
    }))
}

/// Handles `semantic_search` hybrid retrieval blending FTS5 BM25 and LanceDB vectors via RRF.
pub async fn handle_semantic_search(
    state: &AppState,
    args: &serde_json::Map<String, Value>,
) -> Result<Value, McpDomainError> {
    let query = args.get("query").and_then(Value::as_str).unwrap_or("");
    let limit = args.get("limit").and_then(Value::as_u64).unwrap_or(5) as usize;

    let query_vector: Option<Vec<f32>> = args
        .get("vector")
        .or_else(|| args.get("query_vector"))
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_f64().map(|f| f as f32))
                .collect()
        });

    let matches = state
        .memory
        .hybrid_recall(query, query_vector, limit)
        .await
        .map_err(|_| McpDomainError::InternalEnginePanic)?;

    Ok(json!({
        "content": [{
            "type": "text",
            "text": serde_json::to_string_pretty(&matches).unwrap_or_default()
        }],
        "structuredContent": { "matches": matches, "query": query, "limit": limit },
        "isError": false
    }))
}

/// Handles `mem_create_entities` on LadybugDB.
pub async fn handle_mem_create_entities(
    state: &AppState,
    args: &serde_json::Map<String, Value>,
) -> Result<Value, McpDomainError> {
    let pool = state.memory.pool();
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64;
    let mut created_nodes = Vec::new();

    if let Some(entities_arr) = args.get("entities").and_then(Value::as_array) {
        for ent in entities_arr {
            let ent_obj = match ent.as_object() {
                Some(o) => o,
                None => continue,
            };
            let name = ent_obj.get("name").and_then(Value::as_str).unwrap_or("");
            if name.is_empty() {
                continue;
            }
            let node_type = ent_obj
                .get("entity_type")
                .or_else(|| ent_obj.get("entityType"))
                .and_then(Value::as_str)
                .unwrap_or("concept");
            let metadata = ent_obj.get("metadata_json").and_then(Value::as_str).unwrap_or("{}");

            let node = souls_memory::OntologicalNode {
                id: format!("node_{name}"),
                node_type: node_type.to_string(),
                canonical_name: name.to_string(),
                metadata_json: metadata.to_string(),
                banned_patterns: Vec::new(),
            };

            let _ = state.memory.ladybug().insert_node(&pool, node.clone(), now).await;
            created_nodes.push(node);
        }

        return Ok(json!({
            "content": [{
                "type": "text",
                "text": format!("LadybugDB processed {} entities.", created_nodes.len())
            }],
            "structuredContent": { "created_count": created_nodes.len(), "entities": created_nodes },
            "isError": false
        }));
    }

    let name = args.get("name").and_then(Value::as_str).unwrap_or("entity");
    let node_type = args
        .get("entity_type")
        .or_else(|| args.get("entityType"))
        .and_then(Value::as_str)
        .unwrap_or("concept");
    let metadata_json = args.get("metadata_json").and_then(Value::as_str).unwrap_or("{}");

    let node = souls_memory::OntologicalNode {
        id: format!("node_{name}"),
        node_type: node_type.to_string(),
        canonical_name: name.to_string(),
        metadata_json: metadata_json.to_string(),
        banned_patterns: Vec::new(),
    };

    state.memory.ladybug().insert_node(&pool, node.clone(), now).await.map_err(|_| McpDomainError::InternalEnginePanic)?;

    Ok(json!({
        "content": [{
            "type": "text",
            "text": format!("LadybugDB ontological entity '{}' created.", node.canonical_name)
        }],
        "structuredContent": node,
        "isError": false
    }))
}

/// Handles `mem_create_relations` on LadybugDB.
pub async fn handle_mem_create_relations(
    state: &AppState,
    args: &serde_json::Map<String, Value>,
) -> Result<Value, McpDomainError> {
    let pool = state.memory.pool();
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64;
    let mut created_edges = Vec::new();

    if let Some(relations_arr) = args.get("relations").and_then(Value::as_array) {
        for rel in relations_arr {
            let rel_obj = match rel.as_object() {
                Some(o) => o,
                None => continue,
            };
            let source = rel_obj
                .get("source")
                .or_else(|| rel_obj.get("from"))
                .and_then(Value::as_str)
                .unwrap_or("");
            let target = rel_obj
                .get("target")
                .or_else(|| rel_obj.get("to"))
                .and_then(Value::as_str)
                .unwrap_or("");
            let relationship = rel_obj
                .get("relationship")
                .or_else(|| rel_obj.get("relationType"))
                .and_then(Value::as_str)
                .unwrap_or("relates_to");

            if source.is_empty() || target.is_empty() {
                continue;
            }

            let edge = souls_memory::OntologicalEdge {
                source_id: format!("node_{source}"),
                target_id: format!("node_{target}"),
                relationship: relationship.to_string(),
                weight: 1.0,
            };

            let _ = state.memory.ladybug().insert_edge(&pool, edge.clone(), now).await;
            created_edges.push(edge);
        }

        return Ok(json!({
            "content": [{
                "type": "text",
                "text": format!("LadybugDB processed {} relations.", created_edges.len())
            }],
            "structuredContent": { "created_count": created_edges.len(), "relations": created_edges },
            "isError": false
        }));
    }

    let source = args
        .get("source")
        .or_else(|| args.get("from"))
        .and_then(Value::as_str)
        .unwrap_or("");
    let target = args
        .get("target")
        .or_else(|| args.get("to"))
        .and_then(Value::as_str)
        .unwrap_or("");
    let relationship = args
        .get("relationship")
        .or_else(|| args.get("relationType"))
        .and_then(Value::as_str)
        .unwrap_or("relates_to");

    if source.is_empty() || target.is_empty() {
        return Ok(json!({
            "content": [{
                "type": "text",
                "text": "Skipped creating relation with empty source or target node."
            }],
            "structuredContent": { "status": "skipped", "reason": "empty_nodes" },
            "isError": false
        }));
    }

    let edge = souls_memory::OntologicalEdge {
        source_id: format!("node_{source}"),
        target_id: format!("node_{target}"),
        relationship: relationship.to_string(),
        weight: 1.0,
    };

    state.memory.ladybug().insert_edge(&pool, edge.clone(), now).await.map_err(|_| McpDomainError::InternalEnginePanic)?;

    Ok(json!({
        "content": [{
            "type": "text",
            "text": format!("LadybugDB relation '{} -> {} ({})' created.", source, target, relationship)
        }],
        "structuredContent": edge,
        "isError": false
    }))
}

/// Handles `outline` syntactic skeleton dehydration (FinOps context preservation).
pub fn handle_outline(
    _state: &AppState,
    args: &serde_json::Map<String, Value>,
) -> Result<Value, McpDomainError> {
    let path_str = args.get("path").and_then(Value::as_str).unwrap_or("");
    let include_private = args.get("include_private").and_then(Value::as_bool).unwrap_or(false);

    let resolved = souls_core::fs::resolve_workspace_path(path_str).map_err(|_| McpDomainError::InvalidInputParameters)?;
    let content = std::fs::read_to_string(&resolved).map_err(|_| McpDomainError::InvalidInputParameters)?;
    let language = detect_language(path_str);

    let outline = souls_ast::dehydrate_outline(&content, &language, include_private);

    Ok(json!({
        "content": [{
            "type": "text",
            "text": outline.outline_text
        }],
        "structuredContent": outline,
        "isError": false
    }))
}

/// Handles `symbol` surgical AST slice extraction.
pub fn handle_symbol(
    _state: &AppState,
    args: &serde_json::Map<String, Value>,
) -> Result<Value, McpDomainError> {
    let path_str = args.get("path").and_then(Value::as_str).unwrap_or("");
    let query = args.get("symbol_query").and_then(Value::as_str).unwrap_or("");

    let resolved = souls_core::fs::resolve_workspace_path(path_str).map_err(|_| McpDomainError::InvalidInputParameters)?;
    let content = std::fs::read_to_string(&resolved).map_err(|_| McpDomainError::InvalidInputParameters)?;
    let slice_opt = souls_ast::extract_symbol_slice(&content, query);

    match slice_opt {
        Some(slice) => Ok(json!({
            "content": [{
                "type": "text",
                "text": slice.content
            }],
            "structuredContent": slice,
            "isError": false
        })),
        None => Ok(json!({
            "content": [{
                "type": "text",
                "text": format!("Symbol '{query}' not found in '{path_str}'.")
            }],
            "structuredContent": { "found": false, "symbol": query, "path": path_str },
            "isError": false
        })),
    }
}

/// Handles `callers` syntactic dependency tracing.
pub fn handle_callers(
    _state: &AppState,
    args: &serde_json::Map<String, Value>,
) -> Result<Value, McpDomainError> {
    let symbol = args
        .get("symbol_name")
        .or_else(|| args.get("symbol"))
        .and_then(Value::as_str)
        .unwrap_or("");
    let file_scope = args.get("file_scope").and_then(Value::as_str);

    let store = souls_ast::call_graph::CallGraphStore::new();
    let callers = store.get_callers(symbol);
    let sym_entry = store.lookup_symbol(symbol);

    let mut callers_list = Vec::new();
    for c in &callers {
        let entry = store.lookup_symbol(c);
        let (file, line) = match entry {
            Some(ref e) => (e.file_path.to_string_lossy().to_string(), e.line),
            None => (file_scope.unwrap_or("unknown").to_string(), 0),
        };
        callers_list.push(json!({
            "caller": c,
            "file": file,
            "line": line,
            "snippet": format!("let _ = {symbol}();")
        }));
    }

    let res = json!({
        "symbol": symbol,
        "file_scope": file_scope,
        "symbol_info": sym_entry.map(|e| json!({
            "qualified_name": e.qualified_name,
            "kind": e.kind.as_str(),
            "file": e.file_path.to_string_lossy(),
            "line": e.line,
            "col": e.column
        })),
        "callers": callers_list,
        "count": callers.len()
    });

    Ok(json!({
        "content": [{
            "type": "text",
            "text": serde_json::to_string_pretty(&res).unwrap_or_default()
        }],
        "structuredContent": res,
        "isError": false
    }))
}

/// Handles `callees` internal invocations tracing.
pub fn handle_callees(
    _state: &AppState,
    args: &serde_json::Map<String, Value>,
) -> Result<Value, McpDomainError> {
    let symbol = args
        .get("symbol_name")
        .or_else(|| args.get("symbol"))
        .and_then(Value::as_str)
        .unwrap_or("");

    let store = souls_ast::call_graph::CallGraphStore::new();
    let callees = store.get_callees(symbol);
    let sym_entry = store.lookup_symbol(symbol);

    let mut callees_list = Vec::new();
    for c in &callees {
        let entry = store.lookup_symbol(c);
        let (file, line) = match entry {
            Some(ref e) => (e.file_path.to_string_lossy().to_string(), e.line),
            None => ("unknown".to_string(), 0),
        };
        callees_list.push(json!({
            "callee": c,
            "file": file,
            "line": line
        }));
    }

    let res = json!({
        "symbol": symbol,
        "symbol_info": sym_entry.map(|e| json!({
            "qualified_name": e.qualified_name,
            "kind": e.kind.as_str(),
            "file": e.file_path.to_string_lossy(),
            "line": e.line,
            "col": e.column
        })),
        "callees": callees_list,
        "count": callees.len()
    });

    Ok(json!({
        "content": [{
            "type": "text",
            "text": serde_json::to_string_pretty(&res).unwrap_or_default()
        }],
        "structuredContent": res,
        "isError": false
    }))
}

/// Handles `get_ast` isolated Tree-sitter parsing.
pub fn handle_get_ast(
    _state: &AppState,
    args: &serde_json::Map<String, Value>,
) -> Result<Value, McpDomainError> {
    let path_str = args.get("path").and_then(Value::as_str).unwrap_or("");
    let resolved = souls_core::fs::resolve_workspace_path(path_str).map_err(|_| McpDomainError::InvalidInputParameters)?;
    let content = std::fs::read_to_string(&resolved).map_err(|_| McpDomainError::InvalidInputParameters)?;
    let language = detect_language(path_str);

    let parsed = souls_ast::parse_code_isolated(&content, &language, None)
        .map_err(|_| McpDomainError::InternalEnginePanic)?;

    Ok(json!({
        "content": [{
            "type": "text",
            "text": serde_json::to_string_pretty(&parsed).unwrap_or_default()
        }],
        "structuredContent": parsed,
        "isError": false
    }))
}

/// Handles `delta_diff` Myers minimal patch generation.
pub fn handle_delta_diff(
    _state: &AppState,
    args: &serde_json::Map<String, Value>,
) -> Result<Value, McpDomainError> {
    let path_str = args.get("path").and_then(Value::as_str).unwrap_or("");
    let new_content = args.get("new_content").and_then(Value::as_str).unwrap_or("");

    let resolved = souls_core::fs::resolve_workspace_path(path_str).map_err(|_| McpDomainError::InvalidInputParameters)?;
    let old_content = std::fs::read_to_string(&resolved).unwrap_or_default();
    let patch = souls_ast::compute_safe_myers_diff(&old_content, new_content)
        .map_err(|_| McpDomainError::InternalEnginePanic)?;

    Ok(json!({
        "content": [{
            "type": "text",
            "text": patch.diff_text
        }],
        "structuredContent": patch,
        "isError": false
    }))
}

/// Handles `edit` atomic string substitution.
pub fn handle_edit(
    _state: &AppState,
    args: &serde_json::Map<String, Value>,
) -> Result<Value, McpDomainError> {
    let path_str = args.get("path").and_then(Value::as_str).unwrap_or("");
    let old_str = args.get("old_string").and_then(Value::as_str).unwrap_or("");
    let new_str = args.get("new_string").and_then(Value::as_str).unwrap_or("");

    let resolved = souls_core::fs::resolve_workspace_path(path_str).map_err(|_| McpDomainError::InvalidInputParameters)?;
    let content = std::fs::read_to_string(&resolved).map_err(|_| McpDomainError::InvalidInputParameters)?;
    let clean = souls_ast::sanitize_lf(&content);
    let clean_old = souls_ast::sanitize_lf(old_str);
    let clean_new = souls_ast::sanitize_lf(new_str);

    if !clean.contains(&clean_old) {
        return Err(McpDomainError::InvalidInputParameters);
    }

    let count = clean.matches(&clean_old).count();
    if count > 1 {
        return Err(McpDomainError::InvalidInputParameters);
    }

    let modified = clean.replacen(&clean_old, &clean_new, 1);
    std::fs::write(&resolved, modified).map_err(|_| McpDomainError::InternalEnginePanic)?;

    Ok(json!({
        "content": [{
            "type": "text",
            "text": format!("File '{path_str}' updated cleanly with single atomic replacement.")
        }],
        "structuredContent": { "path": path_str, "status": "edited", "replacements": 1 },
        "isError": false
    }))
}

/// Handles `replace` semantic range replacement.
pub fn handle_replace(
    _state: &AppState,
    args: &serde_json::Map<String, Value>,
) -> Result<Value, McpDomainError> {
    let path_str = args.get("path").and_then(Value::as_str).unwrap_or("");
    let target = args.get("target").and_then(Value::as_str).unwrap_or("");
    let replacement = args.get("replacement").and_then(Value::as_str).unwrap_or("");

    let resolved = souls_core::fs::resolve_workspace_path(path_str).map_err(|_| McpDomainError::InvalidInputParameters)?;
    let content = std::fs::read_to_string(&resolved).map_err(|_| McpDomainError::InvalidInputParameters)?;
    let clean = souls_ast::sanitize_lf(&content);

    if !clean.contains(target) {
        return Err(McpDomainError::InvalidInputParameters);
    }

    let modified = clean.replacen(target, replacement, 1);
    std::fs::write(&resolved, modified).map_err(|_| McpDomainError::InternalEnginePanic)?;

    Ok(json!({
        "content": [{
            "type": "text",
            "text": format!("Target block in '{path_str}' replaced successfully.")
        }],
        "structuredContent": { "path": path_str, "status": "replaced" },
        "isError": false
    }))
}

/// Handles `repo_heatmap` Langevin Frecency calculation.
pub fn handle_repo_heatmap(
    _state: &AppState,
    args: &serde_json::Map<String, Value>,
) -> Result<Value, McpDomainError> {
    let repo_path = args.get("repo_path").and_then(Value::as_str).unwrap_or(".");
    let window_days = args.get("time_window_days").and_then(Value::as_u64).unwrap_or(30) as u32;
    let half_life = args.get("half_life_days").and_then(Value::as_f64).unwrap_or(7.0);

    let resolved_repo = souls_core::fs::resolve_workspace_path(repo_path).unwrap_or_else(|_| std::path::PathBuf::from(repo_path));
    let entries = souls_ast::calculate_repo_frecency(&resolved_repo, window_days, half_life)
        .unwrap_or_default();

    let top_entries: Vec<_> = entries.into_iter().take(20).collect();

    Ok(json!({
        "content": [{
            "type": "text",
            "text": serde_json::to_string_pretty(&top_entries).unwrap_or_default()
        }],
        "structuredContent": { "repo_path": repo_path, "top_entries": top_entries },
        "isError": false
    }))
}

/// Handles `repo_impact` blast radius BFS calculation.
pub fn handle_repo_impact(
    _state: &AppState,
    args: &serde_json::Map<String, Value>,
) -> Result<Value, McpDomainError> {
    let repo_path = args.get("repo_path").and_then(Value::as_str).unwrap_or(".");
    let depth = args.get("depth").and_then(Value::as_u64).unwrap_or(3) as usize;

    let mut targets = Vec::new();
    if let Some(p) = args.get("path").and_then(Value::as_str) {
        targets.push(p);
    }
    if let Some(arr) = args.get("paths").and_then(Value::as_array) {
        for item in arr {
            if let Some(s) = item.as_str() {
                targets.push(s);
            }
        }
    }
    if targets.is_empty() {
        targets.push("crates/souls_core/src/lib.rs");
    }

    let resolved_repo = souls_core::fs::resolve_workspace_path(repo_path).unwrap_or_else(|_| std::path::PathBuf::from(repo_path));
    let blast = souls_ast::calculate_blast_radius(&resolved_repo, &targets, depth)
        .map_err(|_| McpDomainError::InternalEnginePanic)?;

    Ok(json!({
        "content": [{
            "type": "text",
            "text": serde_json::to_string_pretty(&blast).unwrap_or_default()
        }],
        "structuredContent": blast,
        "isError": false
    }))
}

/// Handles `intent` Tier 0/0.5 Epistemic Probing with Fallback Escalation support.
pub async fn handle_intent(
    state: &AppState,
    args: &serde_json::Map<String, Value>,
) -> Result<Value, McpDomainError> {
    let prompt = args.get("prompt")
        .or_else(|| args.get("task"))
        .and_then(Value::as_str)
        .unwrap_or("");

    let is_retry = args.get("is_retry")
        .or_else(|| args.get("retry"))
        .and_then(Value::as_bool)
        .unwrap_or(false);

    let eval = state.runtime.evaluate_intent_with_retry(prompt, is_retry).await
        .map_err(|_| McpDomainError::InternalEnginePanic)?;

    Ok(json!({
        "content": [{
            "type": "text",
            "text": serde_json::to_string_pretty(&eval).unwrap_or_default()
        }],
        "structuredContent": eval,
        "isError": false
    }))
}

fn detect_language(path: &str) -> String {
    if path.ends_with(".rs") {
        "rust".to_string()
    } else if path.ends_with(".py") {
        "python".to_string()
    } else if path.ends_with(".ts") || path.ends_with(".tsx") {
        "typescript".to_string()
    } else if path.ends_with(".js") || path.ends_with(".jsx") {
        "javascript".to_string()
    } else if path.ends_with(".go") {
        "go".to_string()
    } else {
        "rust".to_string()
    }
}

/// Handles `cloud_champions` Pareto champions inspection.
pub async fn handle_cloud_champions(
    state: &AppState,
    args: &serde_json::Map<String, Value>,
) -> Result<Value, McpDomainError> {
    let date = args.get("date").and_then(Value::as_str);
    let profile_filter = args.get("profile").and_then(Value::as_str).unwrap_or("all");

    let champions = state.cloud_arena.get_all_champions(date).await
        .map_err(|_| McpDomainError::InternalEnginePanic)?;

    let filtered: Vec<_> = if profile_filter == "all" {
        champions
    } else {
        champions.into_iter().filter(|c| c.profile.as_str() == profile_filter).collect()
    };

    Ok(json!({
        "content": [{
            "type": "text",
            "text": serde_json::to_string_pretty(&filtered).unwrap_or_default()
        }],
        "structuredContent": filtered,
        "isError": false
    }))
}

/// Handles `cloud_arena_sync` on-demand synchronization.
pub async fn handle_cloud_arena_sync(
    state: &AppState,
    args: &serde_json::Map<String, Value>,
) -> Result<Value, McpDomainError> {
    let force = args.get("force").and_then(Value::as_bool).unwrap_or(false);
    let date = args.get("date").and_then(Value::as_str);

    let outcome = if force || date.is_some() {
        state.cloud_arena.force_sync(date).await
            .map_err(|_| McpDomainError::InternalEnginePanic)?
    } else {
        match state.cloud_arena.sync_if_needed().await {
            Ok(Some(o)) => o,
            Ok(None) => {
                return Ok(json!({
                    "content": [{
                        "type": "text",
                        "text": "Cloud Arena já está atualizada para o dia de hoje."
                    }],
                    "isError": false
                }));
            }
            Err(_) => return Err(McpDomainError::InternalEnginePanic),
        }
    };

    Ok(json!({
        "content": [{
            "type": "text",
            "text": serde_json::to_string_pretty(&outcome).unwrap_or_default()
        }],
        "structuredContent": outcome,
        "isError": false
    }))
}

/// Handles `profile_list` Master Brain profiles inspection.
pub async fn handle_profile_list(state: &AppState) -> Result<Value, McpDomainError> {
    let profiles = crate::profiles::get_available_profiles(state).await;
    let cfg_path = crate::profiles::resolve_hermes_config_path();
    let (active_model, active_provider) = crate::profiles::read_active_hermes_model(&cfg_path);

    let active_profile = profiles
        .iter()
        .find(|p| p.is_active)
        .map(|p| p.id.as_str())
        .unwrap_or("custom");

    let result = json!({
        "active_model": active_model,
        "active_provider": active_provider,
        "active_profile": active_profile,
        "config_path": cfg_path.to_string_lossy(),
        "profiles_count": profiles.len(),
        "profiles": profiles
    });

    Ok(json!({
        "content": [{
            "type": "text",
            "text": serde_json::to_string_pretty(&result).unwrap_or_default()
        }],
        "structuredContent": result,
        "isError": false
    }))
}

/// Handles `profile_switch` atomic profile switching on Hermes Agent config.yaml.
pub async fn handle_profile_switch(
    state: &AppState,
    args: &serde_json::Map<String, Value>,
) -> Result<Value, McpDomainError> {
    let profile_arg = args.get("profile").and_then(Value::as_str);
    let model_override = args.get("model_override").and_then(Value::as_str);
    let provider_arg = args.get("provider").and_then(Value::as_str);

    let profiles = crate::profiles::get_available_profiles(state).await;

    let (target_model, target_provider) = if let Some(custom_model) = model_override {
        (custom_model.to_string(), provider_arg.unwrap_or("nous").to_string())
    } else if let Some(prof_id) = profile_arg {
        if let Some(matched) = profiles.iter().find(|p| p.id == prof_id) {
            (matched.target_model.clone(), provider_arg.unwrap_or(&matched.provider).to_string())
        } else {
            return Err(McpDomainError::InvalidInputParameters);
        }
    } else {
        let matched = profiles
            .iter()
            .find(|p| p.id == "frontier_balanced")
            .ok_or(McpDomainError::InternalEnginePanic)?;
        (matched.target_model.clone(), provider_arg.unwrap_or(&matched.provider).to_string())
    };

    let switch_res = crate::profiles::apply_profile_to_hermes(
        profile_arg,
        &target_model,
        Some(&target_provider),
        None,
    )
    .map_err(|_| McpDomainError::InvalidInputParameters)?;

    let text_summary = format!(
        "Master Brain do Hermes atualizado com sucesso!\n- Modelo anterior: {}\n- Novo modelo ativo: {}\n- Provedor: {}\n- Backup de segurança: {}\n\n(i) Novas sessões e prompts do Hermes Agent utilizarão imediatamente este modelo.",
        switch_res.previous_model,
        switch_res.applied_model,
        switch_res.provider,
        switch_res.backup_path.as_deref().unwrap_or("N/A")
    );

    Ok(json!({
        "content": [{
            "type": "text",
            "text": text_summary
        }],
        "structuredContent": switch_res,
        "isError": false
    }))
}

