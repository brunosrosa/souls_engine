//! MCP Tool Implementations for Souls Evolutive Harness (`v0.9.4` / `ADR-034`).
//!
//! Exposes 5 canonical Progressive-Disclosure tools:
//! - `harness_status`
//! - `harness_catnap_pulse`
//! - `harness_cordis_rollback`
//! - `harness_tombstone_resolve`
//! - `harness_ralph_crystallize`

#![forbid(unsafe_code)]

use chrono::Utc;
use serde_json::{json, Value};

use souls_memory::{
    apply_memory_tombstone, detect_and_apply_jit_tombstones, list_memory_tombstones,
    preview_jit_tombstones, record_weevolve_signal, restore_tombstoned_memory,
};
use souls_protocol::{error::McpDomainError, EpistemicPartition};

use crate::state::AppState;

fn mcp_ok(text: impl Into<String>, structured: Value) -> Value {
    json!({
        "content": [{ "type": "text", "text": text.into() }],
        "structuredContent": structured,
        "isError": false
    })
}

/// Dispatches any of the 5 `harness_*` MCP tools. Returns `None` if `tool_name` is unrelated.
pub async fn try_dispatch_harness_tool(
    state: &AppState,
    tool_name: &str,
    args: &serde_json::Map<String, Value>,
    session_id: Option<&str>,
) -> Option<Result<Value, McpDomainError>> {
    let norm = tool_name.trim().replace('.', "_");
    match norm.as_str() {
        "harness_status" => {
            let include_index = args
                .get("include_index")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            let mut overview = state.evolutive_harness.build_overview(state).await;
            if !include_index {
                if let Some(lib) = overview.get_mut("experience_library").and_then(Value::as_object_mut) {
                    lib.remove("index_markdown");
                }
            }
            let pins_ok = overview["summary"]["pins_verified"]
                .as_bool()
                .unwrap_or(false);
            let active_opt = overview["summary"]["active_optimizations"]
                .as_u64()
                .unwrap_or(0);
            let superseded = overview["summary"]["superseded_memories"]
                .as_u64()
                .unwrap_or(0);
            let quarantined = overview["summary"]["quarantined_skills"]
                .as_u64()
                .unwrap_or(0);
            Some(Ok(mcp_ok(
                format!(
                    "Evolutive Harness v0.9.4+2: c1 pins={}, {} otimizações ativas, {} memórias substituídas, {} habilidades em quarentena.",
                    if pins_ok { "íntegros" } else { "ALERTA" },
                    active_opt,
                    superseded,
                    quarantined
                ),
                overview,
            )))
        }
        "harness_catnap_pulse" => {
            let slice_kind = args.get("slice_kind").and_then(Value::as_str);
            let force_manual = args
                .get("force_manual")
                .and_then(Value::as_bool)
                .unwrap_or(true);
            match state
                .evolutive_harness
                .run_catnap_micro_slice(state, slice_kind, force_manual)
                .await
            {
                Ok(res) => Some(Ok(mcp_ok(
                    format!(
                        "Pulso Catnap finalizado (status={}, preempted={}).",
                        res["status"].as_str().unwrap_or("ok"),
                        res["preempted"].as_bool().unwrap_or(false)
                    ),
                    res,
                ))),
                Err(e) => Some(Ok(json!({
                    "content": [{ "type": "text", "text": e }],
                    "structuredContent": { "status": "error", "error": e },
                    "isError": true
                }))),
            }
        }
        "harness_cordis_rollback" => {
            let fiber_id = args
                .get("fiber_id")
                .and_then(Value::as_str)
                .unwrap_or("")
                .trim();
            if fiber_id.is_empty() {
                return Some(Err(McpDomainError::InvalidInputParameters));
            }
            match state.evolutive_harness.cordis.dispose_fiber(fiber_id) {
                Ok(report) => {
                    let pool = state.memory.pool();
                    let now = Utc::now().timestamp();
                    let _ = record_weevolve_signal(
                        &pool,
                        fiber_id,
                        "cordis_fiber",
                        "cordis_rollback",
                        None,
                        now,
                    )
                    .await;
                    Some(Ok(mcp_ok(
                        format!(
                            "Otimização Cordis '{}' revertida em {} µs ({} passos LIFO).",
                            report.fiber_id,
                            report.elapsed_us,
                            report.reverted_operations.len()
                        ),
                        json!({
                            "status": "ok",
                            "report": report,
                        }),
                    )))
                }
                Err(e) => Some(Ok(json!({
                    "content": [{ "type": "text", "text": e.to_string() }],
                    "structuredContent": { "status": "error", "error": e.to_string() },
                    "isError": true
                }))),
            }
        }
        "harness_tombstone_resolve" => {
            let action = args
                .get("action")
                .and_then(Value::as_str)
                .unwrap_or("preview")
                .trim()
                .to_ascii_lowercase();
            let pool = state.memory.pool();
            let now = Utc::now().timestamp();

            match action.as_str() {
                "list" => {
                    let limit = args.get("limit").and_then(Value::as_u64).unwrap_or(30) as usize;
                    let items = list_memory_tombstones(&pool, limit)
                        .await
                        .unwrap_or_default();
                    Some(Ok(mcp_ok(
                        format!("{} tombstones epistêmicos listados.", items.len()),
                        json!({ "status": "ok", "mutated": false, "tombstones": items }),
                    )))
                }
                "preview" | "dry_run" => {
                    if let Some(mem_id) = args.get("memory_id").and_then(Value::as_str) {
                        let Ok(Some(mem)) = state.memory.get_epistemic_memory(mem_id).await else {
                            return Some(Err(McpDomainError::ResourceNotFound));
                        };
                        let candidates = preview_jit_tombstones(&pool, &mem, now)
                            .await
                            .unwrap_or_default();
                        return Some(Ok(mcp_ok(
                            format!(
                                "Pré-visualização somente leitura para '{}': {} possível(is) contradição(ões) em EVOLVING (nenhuma mutação aplicada).",
                                mem_id,
                                candidates.len()
                            ),
                            json!({
                                "status": "ok",
                                "mutated": false,
                                "candidates": candidates
                            }),
                        )));
                    }

                    let recent = state
                        .memory
                        .list_epistemic(Some(EpistemicPartition::Evolving), 20)
                        .await
                        .unwrap_or_default();
                    let mut seen_pairs = std::collections::HashSet::new();
                    let mut all_candidates = Vec::new();
                    for mem in recent {
                        if let Ok(found) = preview_jit_tombstones(&pool, &mem, now).await {
                            for cand in found {
                                let pair = (
                                    cand.superseded_memory_id.clone(),
                                    cand.superseded_by_memory_id.clone(),
                                );
                                if seen_pairs.insert(pair) {
                                    all_candidates.push(cand);
                                }
                            }
                        }
                    }
                    Some(Ok(mcp_ok(
                        format!(
                            "Pré-visualização somente leitura concluída: {} possível(is) contradição(ões) detectada(s) em EVOLVING (nenhuma mutação aplicada).",
                            all_candidates.len()
                        ),
                        json!({
                            "status": "ok",
                            "mutated": false,
                            "candidates": all_candidates
                        }),
                    )))
                }
                "restore" => {
                    let tomb_id = args
                        .get("tombstone_id")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .trim();
                    if tomb_id.is_empty() {
                        return Some(Err(McpDomainError::InvalidInputParameters));
                    }
                    match restore_tombstoned_memory(&pool, tomb_id, now).await {
                        Ok(Some(restored)) => {
                            let src_node = format!("mem_node_{}", restored.superseded_by_memory_id);
                            let dst_node = format!("mem_node_{}", restored.superseded_memory_id);
                            let _ = state
                                .memory
                                .ladybug()
                                .remove_edge_by_rel(&pool, &src_node, &dst_node, "conflicts_with")
                                .await;
                            Some(Ok(mcp_ok(
                                format!("Memória '{}' restaurada com sucesso.", restored.superseded_memory_id),
                                json!({ "status": "ok", "mutated": true, "tombstone": restored }),
                            )))
                        }
                        Ok(None) => Some(Err(McpDomainError::ResourceNotFound)),
                        Err(_) => Some(Err(McpDomainError::InternalEnginePanic)),
                    }
                }
                "apply" => {
                    let old_id = args.get("superseded_memory_id").and_then(Value::as_str).unwrap_or("").trim();
                    let new_id = args.get("superseded_by_memory_id").and_then(Value::as_str).unwrap_or("").trim();
                    if old_id.is_empty() || new_id.is_empty() {
                        return Some(Err(McpDomainError::InvalidInputParameters));
                    }
                    let Ok(Some(old_mem)) = state.memory.get_epistemic_memory(old_id).await else {
                        return Some(Err(McpDomainError::ResourceNotFound));
                    };
                    let Ok(Some(new_mem)) = state.memory.get_epistemic_memory(new_id).await else {
                        return Some(Err(McpDomainError::ResourceNotFound));
                    };
                    let reason = args
                        .get("reason_pt")
                        .and_then(Value::as_str)
                        .unwrap_or("Contradição marcada via MCP harness_tombstone_resolve.");
                    match apply_memory_tombstone(&pool, state.memory.ladybug(), &old_mem, &new_mem, reason, now).await {
                        Ok(tomb) => Some(Ok(mcp_ok(
                            format!("Tombstone lógico '{}' aplicado sem remover registro STABLE.", tomb.tombstone_id),
                            json!({ "status": "ok", "mutated": true, "tombstone": tomb }),
                        ))),
                        Err(_) => Some(Err(McpDomainError::InternalEnginePanic)),
                    }
                }
                "scan" => {
                    if let Some(mem_id) = args.get("memory_id").and_then(Value::as_str) {
                        let Ok(Some(mem)) = state.memory.get_epistemic_memory(mem_id).await else {
                            return Some(Err(McpDomainError::ResourceNotFound));
                        };
                        let applied = detect_and_apply_jit_tombstones(
                            &pool,
                            state.memory.ladybug(),
                            &mem,
                            now,
                        )
                        .await
                        .unwrap_or_default();
                        return Some(Ok(mcp_ok(
                            format!("Varredura JIT mutante concluída para '{}' (STABLE preservado): {} contradição(ões) resolvida(s).", mem_id, applied.len()),
                            json!({
                                "status": "ok",
                                "mutated": true,
                                "warning_pt": "Varredura mutante aplicada sobre memórias EVOLVING (registros STABLE preservados).",
                                "tombstones_applied": applied
                            }),
                        )));
                    }
                    let recent = state
                        .memory
                        .list_epistemic(Some(EpistemicPartition::Evolving), 20)
                        .await
                        .unwrap_or_default();
                    let mut all_applied = Vec::new();
                    let mut seen_superseded = std::collections::HashSet::new();
                    for mem in recent {
                        if seen_superseded.contains(&mem.id) {
                            continue;
                        }
                        if let Ok(mut applied) = detect_and_apply_jit_tombstones(
                            &pool,
                            state.memory.ladybug(),
                            &mem,
                            now,
                        )
                        .await
                        {
                            for t in &applied {
                                seen_superseded.insert(t.superseded_memory_id.clone());
                            }
                            all_applied.append(&mut applied);
                        }
                    }
                    Some(Ok(mcp_ok(
                        format!("Varredura JIT mutante geral concluída (EVOLVING apenas, STABLE intacto): {} contradição(ões) resolvida(s).", all_applied.len()),
                        json!({
                            "status": "ok",
                            "mutated": true,
                            "warning_pt": "Varredura mutante aplicada sobre memórias EVOLVING (registros STABLE preservados).",
                            "tombstones_applied": all_applied
                        }),
                    )))
                }
                _ => Some(Err(McpDomainError::InvalidInputParameters)),
            }
        }
        "harness_ralph_crystallize" => {
            let title_pt = args.get("title_pt").and_then(Value::as_str).unwrap_or("").trim();
            let problem_pattern_pt = args
                .get("problem_pattern_pt")
                .and_then(Value::as_str)
                .unwrap_or("")
                .trim();
            let resolution_playbook = args
                .get("resolution_playbook")
                .and_then(Value::as_str)
                .unwrap_or("")
                .trim();
            if title_pt.is_empty() || problem_pattern_pt.is_empty() || resolution_playbook.is_empty() {
                return Some(Err(McpDomainError::InvalidInputParameters));
            }
            let domain = args.get("domain").and_then(Value::as_str).unwrap_or("ops");
            let src_sess = args
                .get("source_session_id")
                .and_then(Value::as_str)
                .or(session_id);

            match state
                .evolutive_harness
                .crystallize_ralph_skill(
                    state,
                    title_pt,
                    domain,
                    problem_pattern_pt,
                    resolution_playbook,
                    src_sess,
                )
                .await
            {
                Ok(cand) => Some(Ok(mcp_ok(
                    format!(
                        "Habilidade '{}' cristalizada em Quarentena para revisão humana ({}).",
                        cand.title_pt, cand.quarantine_path
                    ),
                    json!({ "status": "ok", "candidate": cand }),
                ))),
                Err(e) => Some(Ok(json!({
                    "content": [{ "type": "text", "text": e }],
                    "structuredContent": { "status": "error", "error": e },
                    "isError": true
                }))),
            }
        }
        _ => None,
    }
}
