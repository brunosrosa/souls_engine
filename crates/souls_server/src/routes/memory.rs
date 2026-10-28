//! # Bidirectional Memory Bridge & Frozen Snapshot Endpoints (`/api/v1/memory/*`)
//!
//! Exposes the bare-metal L3 Hippocampus (`SoulsMemoryStore`) to Hermes Agent's
//! `MemoryProvider` lifecycle hooks and the Souls Engine Web Dashboard:
//! - `GET  /api/v1/memory/snapshot`: Inspects active `FrozenMemorySnapshot` & partition counts.
//! - `POST /api/v1/memory/snapshot/pin`: Pins or refreshes a session's `FrozenMemorySnapshot`.
//! - `POST /api/v1/memory/recall`: Executes deduplicated RRF Hybrid Recall with **Adaptive Recall**
//!   ("Terceira Via": `lean` vs `expanded` + LadybugDB ontology neighborhood) and `PiiRedactor`.
//! - `POST /api/v1/memory/mirror`: Mirrors Hermes `memory` tool writes (`add`/`replace`/`remove`) into `STABLE`.
//! - `POST /api/v1/memory/turn`: Persists `sync_turn` and `on_delegation` insights into `EVOLVING`.
//! - `POST /api/v1/memory/checkpoint`: Persists `on_pre_compress` Audit Log v2 checkpoints.
//! - `GET  /api/v1/memory/overview`: Consolidated metrics and audit view for the Web Dashboard.

use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::Row;

use souls_core::pii_redactor::PiiRedactor;
use souls_memory::{
    sha256_hex, EpistemicMemoryRecord, EpistemicPartition, SessionCheckpointRecord, UnifiedMatch,
};

use crate::state::AppState;

fn now_epoch_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(1_700_000_000)
}

#[derive(Debug, Deserialize, Default)]
pub struct SnapshotQuery {
    pub session_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PinSnapshotRequest {
    pub session_id: String,
    #[serde(default)]
    pub force_refresh: bool,
    pub hermes_home: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RecallRequest {
    pub query: String,
    #[serde(default = "default_session_id")]
    pub session_id: String,
    /// Recall mode: `"adaptive"` (default, Terceira Via), `"lean"`, or `"expanded"`.
    #[serde(default = "default_recall_mode")]
    pub mode: String,
    pub top_k: Option<usize>,
    pub max_chars: Option<usize>,
    /// Whether to apply local `PiiRedactor` before returning recall blocks to cloud models.
    #[serde(default = "default_true")]
    pub redact_pii: bool,
}

fn default_session_id() -> String {
    "default".to_string()
}

fn default_recall_mode() -> String {
    "adaptive".to_string()
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OntologyRelationSnippet {
    pub src_label: String,
    pub src_kind: String,
    pub relation: String,
    pub dst_label: String,
    pub dst_kind: String,
    pub weight: f64,
}

#[derive(Debug, Deserialize)]
pub struct MirrorWriteRequest {
    /// Action performed by Hermes `memory` tool: `"add"`, `"replace"`, or `"remove"`.
    pub action: String,
    /// Target file: `"memory"` (`MEMORY.md`), `"user"` (`USER.md`), or `"soul"` (`SOUL.md`).
    pub target: String,
    pub content: String,
    pub old_text: Option<String>,
    pub session_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TurnSyncRequest {
    pub session_id: String,
    pub user_content: String,
    pub assistant_content: String,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub salience: Option<f32>,
}

#[derive(Debug, Deserialize)]
pub struct PreCompressCheckpointRequest {
    pub session_id: String,
    #[serde(default)]
    pub messages: Vec<serde_json::Value>,
    pub summary_override: Option<String>,
    #[serde(default)]
    pub pre_compress_tokens: i64,
}

fn redact_string(redactor: &PiiRedactor, input: &str) -> String {
    let out = redactor.redact(input.as_bytes());
    String::from_utf8(out).unwrap_or_else(|_| input.to_string())
}

/// Converts a natural-language user prompt into a clean FTS5 `OR` query of meaningful tokens,
/// preventing FTS5 syntax errors on punctuation (`?`, `-`, etc.) and allowing multi-topic
/// prompts to recall memories across all mentioned topics ranked by BM25.
pub fn build_fts5_or_query(raw_query: &str) -> String {
    const STOP_WORDS: &[&str] = &[
        "como", "para", "qual", "quais", "com", "por", "que", "uma", "uns", "umas", "dos", "das",
        "nos", "nas", "isso", "isto", "esse", "essa", "este", "esta", "sobre", "entre", "the",
        "and", "for", "with", "from", "that", "this", "what", "how", "why", "are", "was",
    ];

    let mut tokens: Vec<String> = Vec::new();
    for part in raw_query.split(|c: char| !c.is_alphanumeric() && c != '_') {
        let trimmed = part.trim();
        if trimmed.len() < 3 {
            continue;
        }
        let lower = trimmed.to_ascii_lowercase();
        if STOP_WORDS.contains(&lower.as_str()) {
            continue;
        }
        if !tokens.iter().any(|t| t.eq_ignore_ascii_case(trimmed)) {
            tokens.push(trimmed.to_string());
        }
        if tokens.len() >= 12 {
            break;
        }
    }

    if tokens.is_empty() {
        raw_query
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace() || *c == '_')
            .collect::<String>()
            .trim()
            .to_string()
    } else {
        tokens.join(" OR ")
    }
}

/// Evaluates whether an `"adaptive"` recall request should run in `"expanded"` or `"lean"` mode.
///
/// Heuristics calibrated for multi-topic associative thinking (ADHD + AH/SD) vs direct commands:
/// - Prompts >= 140 chars, OR containing >= 2 question marks / semicolons / line breaks,
///   OR referencing architectural/cross-domain keywords trigger `"expanded"` (`top_k=8`, `max_chars=2400` + Ontology Graph).
/// - Short, single-focus operational prompts trigger `"lean"` (`top_k=4`, `max_chars=800`).
pub fn resolve_effective_recall_mode(
    requested_mode: &str,
    query: &str,
) -> (&'static str, usize, usize) {
    match requested_mode.trim().to_ascii_lowercase().as_str() {
        "lean" | "enxuto" => ("lean", 4, 800),
        "expanded" | "extendido" | "estendido" => ("expanded", 8, 2400),
        _ => {
            let trimmed = query.trim();
            let char_len = trimmed.chars().count();
            let question_count = trimmed.matches('?').count();
            let clause_separators = trimmed.matches(';').count()
                + trimmed.matches('\n').count()
                + trimmed.matches("...").count();

            let lower = trimmed.to_ascii_lowercase();
            let deep_keywords = [
                "arquitetura",
                "architecture",
                "design",
                "refactor",
                "refatorar",
                "crate",
                "crates",
                "ontologia",
                "ontology",
                "ladybug",
                "lancedb",
                "sqlite",
                "pareto",
                "switchboard",
                "subagent",
                "subagente",
                "hermes",
                "relacao",
                "relação",
                "impacto",
                "histórico",
                "historico",
                "lembra",
                "remember",
                "comparar",
                "compare",
                "por que",
                "why",
                "como funciona",
            ];
            let keyword_hits = deep_keywords
                .iter()
                .filter(|kw| lower.contains(**kw))
                .count();

            if char_len >= 140
                || question_count >= 2
                || clause_separators >= 2
                || keyword_hits >= 2
                || (char_len >= 75 && keyword_hits >= 1)
            {
                ("expanded", 8, 2400)
            } else {
                ("lean", 4, 800)
            }
        }
    }
}

/// Queries the LadybugDB ontological graph (`ladybug_edges` + `ladybug_nodes`) for relations
/// matching tokens from the query or recalled memories.
async fn query_ontology_neighborhood(
    state: &AppState,
    query: &str,
    matches: &[UnifiedMatch],
    limit: usize,
) -> Vec<OntologyRelationSnippet> {
    let mut terms: Vec<String> = query
        .split(|c: char| !c.is_alphanumeric() && c != '_' && c != '-')
        .map(|s| s.trim().to_ascii_lowercase())
        .filter(|s| s.len() >= 4)
        .take(5)
        .collect();

    for m in matches.iter().take(3) {
        for token in m
            .content
            .split(|c: char| !c.is_alphanumeric() && c != '_' && c != '-')
            .map(|s| s.trim().to_ascii_lowercase())
            .filter(|s| s.len() >= 5)
            .take(2)
        {
            if !terms.contains(&token) && terms.len() < 8 {
                terms.push(token);
            }
        }
    }

    let pool = state.memory.pool();
    let rows = sqlx::query(
        r#"
        SELECT
            s.canonical_name AS src_label,
            s.node_type      AS src_kind,
            e.relationship   AS relation,
            d.canonical_name AS dst_label,
            d.node_type      AS dst_kind,
            e.weight         AS weight
        FROM ladybug_edges e
        JOIN ladybug_nodes s ON s.id = e.source_id
        JOIN ladybug_nodes d ON d.id = e.target_id
        ORDER BY e.weight DESC, e.created_at DESC
        LIMIT 30
        "#,
    )
    .fetch_all(&*pool)
    .await
    .unwrap_or_default();

    let mut matched = Vec::new();
    let mut fallback = Vec::new();

    for row in rows {
        let snippet = OntologyRelationSnippet {
            src_label: row.get("src_label"),
            src_kind: row.get("src_kind"),
            relation: row.get("relation"),
            dst_label: row.get("dst_label"),
            dst_kind: row.get("dst_kind"),
            weight: row.get("weight"),
        };

        let hay = format!(
            "{} {} {}",
            snippet.src_label.to_ascii_lowercase(),
            snippet.relation.to_ascii_lowercase(),
            snippet.dst_label.to_ascii_lowercase()
        );

        if terms.is_empty() || terms.iter().any(|t| hay.contains(t)) {
            matched.push(snippet);
        } else if fallback.len() < limit {
            fallback.push(snippet);
        }

        if matched.len() >= limit {
            break;
        }
    }

    if matched.is_empty() {
        fallback.into_iter().take(limit).collect()
    } else {
        matched
    }
}

/// Formats recalled matches and optional ontological relations into a clean XML/Markdown fenced block
/// ready for Hermes `MemoryProvider.prefetch()` injection.
fn format_recall_context_block(
    mode_used: &str,
    matches: &[UnifiedMatch],
    relations: &[OntologyRelationSnippet],
) -> String {
    if matches.is_empty() && relations.is_empty() {
        return String::new();
    }

    let mut out = String::new();
    out.push_str(&format!(
        "<souls-l3-recall mode=\"{}\" count=\"{}\">\n",
        mode_used,
        matches.len()
    ));

    for m in matches {
        out.push_str(&format!(
            "- [{} | rrf={:.4}] {}\n",
            m.category, m.rrf_score, m.content
        ));
    }

    if !relations.is_empty() {
        out.push_str("<ontology-graph>\n");
        for rel in relations {
            out.push_str(&format!(
                "- ({}:{}) --[{}]--> ({}:{})\n",
                rel.src_kind, rel.src_label, rel.relation, rel.dst_kind, rel.dst_label
            ));
        }
        out.push_str("</ontology-graph>\n");
    }

    out.push_str("</souls-l3-recall>");
    out
}

/// `GET /api/v1/memory/snapshot`
pub async fn get_snapshot_handler(
    State(state): State<AppState>,
    Query(query): Query<SnapshotQuery>,
) -> impl IntoResponse {
    let (stable_count, evolving_count) = state.memory.partition_counts().await.unwrap_or((0, 0));

    let snapshot = if let Some(ref sid) = query.session_id {
        state
            .memory
            .snapshots()
            .get_session_snapshot(sid)
            .unwrap_or_else(|| state.memory.snapshots().current_global())
    } else {
        state.memory.snapshots().current_global()
    };

    Json(serde_json::json!({
        "status": "success",
        "stable_count": stable_count,
        "evolving_count": evolving_count,
        "pinned_sessions_count": state.memory.snapshots().pinned_sessions_count(),
        "snapshot": snapshot,
    }))
}

/// `POST /api/v1/memory/snapshot/pin`
pub async fn pin_snapshot_handler(
    State(state): State<AppState>,
    Json(req): Json<PinSnapshotRequest>,
) -> impl IntoResponse {
    let now = now_epoch_secs();
    let hermes_home = req.hermes_home.as_ref().map(PathBuf::from);

    match state
        .memory
        .pin_session_snapshot(
            &req.session_id,
            req.force_refresh,
            hermes_home.as_deref(),
            now,
        )
        .await
    {
        Ok(snapshot) => {
            let (stable_count, evolving_count) =
                state.memory.partition_counts().await.unwrap_or((0, 0));
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "status": "success",
                    "stable_count": stable_count,
                    "evolving_count": evolving_count,
                    "pinned_sessions_count": state.memory.snapshots().pinned_sessions_count(),
                    "snapshot": snapshot,
                })),
            )
        }
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "status": "error",
                "message": err.to_string(),
            })),
        ),
    }
}

/// `POST /api/v1/memory/recall`
pub async fn recall_handler(
    State(state): State<AppState>,
    Json(req): Json<RecallRequest>,
) -> impl IntoResponse {
    if req.query.trim().is_empty() {
        return (
            StatusCode::OK,
            Json(serde_json::json!({
                "status": "success",
                "mode_used": "lean",
                "count": 0,
                "matches": [],
                "ontology_relations": [],
                "formatted_block": "",
            })),
        );
    }

    let (mode_used, default_top_k, default_max_chars) =
        resolve_effective_recall_mode(&req.mode, &req.query);
    let top_k = req.top_k.unwrap_or(default_top_k).clamp(1, 20);
    let max_chars = req.max_chars.unwrap_or(default_max_chars).clamp(120, 8000);
    let fts_query = build_fts5_or_query(&req.query);

    match state
        .memory
        .recall_for_session(&req.session_id, &fts_query, None, top_k, max_chars)
        .await
    {
        Ok(mut matches) => {
            let redactor = PiiRedactor::new(req.redact_pii);

            if req.redact_pii {
                for m in &mut matches {
                    m.content = redact_string(&redactor, &m.content);
                }
            }

            let ontology_relations = if mode_used == "expanded" {
                let mut rels = query_ontology_neighborhood(&state, &req.query, &matches, 5).await;
                if req.redact_pii {
                    for r in &mut rels {
                        r.src_label = redact_string(&redactor, &r.src_label);
                        r.dst_label = redact_string(&redactor, &r.dst_label);
                    }
                }
                rels
            } else {
                Vec::new()
            };

            let total_redactions = redactor.token_count();
            let formatted_block =
                format_recall_context_block(mode_used, &matches, &ontology_relations);

            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "status": "success",
                    "session_id": req.session_id,
                    "mode_requested": req.mode,
                    "mode_used": mode_used,
                    "top_k": top_k,
                    "max_chars": max_chars,
                    "pii_redactions": total_redactions,
                    "count": matches.len(),
                    "matches": matches,
                    "ontology_relations": ontology_relations,
                    "formatted_block": formatted_block,
                })),
            )
        }
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "status": "error",
                "message": err.to_string(),
            })),
        ),
    }
}

/// `POST /api/v1/memory/mirror`
pub async fn mirror_handler(
    State(state): State<AppState>,
    Json(req): Json<MirrorWriteRequest>,
) -> impl IntoResponse {
    let now = now_epoch_secs();
    let session_id = req.session_id.as_deref().unwrap_or("hermes_mirror");
    let effective_content = if req.action == "remove" && req.content.trim().is_empty() {
        req.old_text.as_deref().unwrap_or("")
    } else {
        req.content.as_str()
    };

    match state
        .memory
        .mirror_hermes_write(&req.action, &req.target, effective_content, session_id, now)
        .await
    {
        Ok(record_id) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "status": "success",
                "action": req.action,
                "target": req.target,
                "session_id": session_id,
                "record_id": record_id,
                "partition": "STABLE",
            })),
        ),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "status": "error",
                "message": err.to_string(),
            })),
        ),
    }
}

/// `POST /api/v1/memory/turn`
pub async fn turn_handler(
    State(state): State<AppState>,
    Json(req): Json<TurnSyncRequest>,
) -> impl IntoResponse {
    let user_trimmed = req.user_content.trim();
    let assistant_trimmed = req.assistant_content.trim();

    if user_trimmed.is_empty() && assistant_trimmed.is_empty() {
        return (
            StatusCode::OK,
            Json(serde_json::json!({
                "status": "skipped",
                "reason": "empty_turn",
            })),
        );
    }

    let now = now_epoch_secs();
    let category = req
        .category
        .unwrap_or_else(|| "EVOLVING:turn_insight".to_string());

    let combined = if assistant_trimmed.chars().count() > 600 {
        let snippet: String = assistant_trimmed.chars().take(600).collect();
        format!("Q: {} | A: {}...", user_trimmed, snippet)
    } else {
        format!("Q: {} | A: {}", user_trimmed, assistant_trimmed)
    };

    let seed = format!("{}:{}:{}", req.session_id, now, combined);
    let hash = sha256_hex(&seed);
    let record_id = format!("evolving_turn_{}", &hash[..16]);

    let record = EpistemicMemoryRecord {
        id: record_id.clone(),
        session_id: req.session_id.clone(),
        category: category.clone(),
        content: combined,
        partition: EpistemicPartition::Evolving,
        salience: req.salience.unwrap_or(0.75).clamp(0.0, 1.0),
        access_count: 1,
        source_ref: Some("hermes:sync_turn".to_string()),
        created_at: now,
        last_accessed_at: now,
        updated_at: now,
    };

    match state.memory.upsert_epistemic_memory(&record).await {
        Ok(()) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "status": "success",
                "id": record_id,
                "category": category,
                "partition": "EVOLVING",
            })),
        ),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "status": "error",
                "message": err.to_string(),
            })),
        ),
    }
}

/// `POST /api/v1/memory/checkpoint`
pub async fn checkpoint_handler(
    State(state): State<AppState>,
    Json(req): Json<PreCompressCheckpointRequest>,
) -> impl IntoResponse {
    let now = now_epoch_secs();
    let message_count = req.messages.len();

    let summary_text = if let Some(override_text) = req.summary_override {
        override_text
    } else {
        let mut extracted = Vec::new();
        for msg in req.messages.iter().rev().take(6).rev() {
            let role = msg
                .get("role")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let content = msg.get("content").and_then(|v| v.as_str()).unwrap_or("");
            let trimmed = content.trim();
            if !trimmed.is_empty() {
                let snippet: String = trimmed.chars().take(180).collect();
                extracted.push(format!("[{}] {}", role, snippet));
            }
        }
        if extracted.is_empty() {
            format!(
                "Pre-compress checkpoint for session {} ({} messages, {} tokens).",
                req.session_id, message_count, req.pre_compress_tokens
            )
        } else {
            extracted.join(" || ")
        }
    };

    let raw_messages_blob =
        serde_json::to_string(&req.messages).unwrap_or_else(|_| "[]".to_string());
    let seed = format!(
        "{}:{}:{}:{}",
        req.session_id, now, message_count, raw_messages_blob
    );
    let sha = sha256_hex(&seed);
    let checkpoint_id = format!("chk_{}", &sha[..16]);

    let record = SessionCheckpointRecord {
        id: checkpoint_id.clone(),
        session_id: req.session_id.clone(),
        checkpoint_version: 2,
        raw_messages_blob,
        message_count: message_count as i64,
        pre_compress_tokens: req.pre_compress_tokens,
        hash_sha256: sha.clone(),
        created_at: now,
    };

    if let Err(err) = state.memory.save_pre_compress_checkpoint(&record).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "status": "error",
                "message": err.to_string(),
            })),
        );
    }

    // Also persist a condensed summary into EVOLVING so it remains recallable after context compression
    let summary_record = EpistemicMemoryRecord {
        id: format!("evolving_{checkpoint_id}"),
        session_id: req.session_id.clone(),
        category: "EVOLVING:pre_compress_summary".to_string(),
        content: summary_text.clone(),
        partition: EpistemicPartition::Evolving,
        salience: 0.82_f32,
        access_count: 1,
        source_ref: Some(format!("checkpoint:{checkpoint_id}")),
        created_at: now,
        last_accessed_at: now,
        updated_at: now,
    };
    let _ = state.memory.upsert_epistemic_memory(&summary_record).await;

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "success",
            "checkpoint_id": checkpoint_id,
            "session_id": req.session_id,
            "message_count": message_count,
            "pre_compress_tokens": req.pre_compress_tokens,
            "sha256_hash": sha,
            "summary_preview": summary_text,
        })),
    )
}

/// `GET /api/v1/memory/overview`
pub async fn overview_handler(State(state): State<AppState>) -> impl IntoResponse {
    let (stable_count, evolving_count) = state.memory.partition_counts().await.unwrap_or((0, 0));
    let recent_checkpoints = state
        .memory
        .list_checkpoints(None, 10)
        .await
        .unwrap_or_default();
    let recent_memories = state
        .memory
        .list_epistemic(None, 15)
        .await
        .unwrap_or_default();

    let pool = state.memory.pool();
    let ladybug_nodes_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM ladybug_nodes")
        .fetch_one(&*pool)
        .await
        .unwrap_or(0);
    let ladybug_edges_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM ladybug_edges")
        .fetch_one(&*pool)
        .await
        .unwrap_or(0);

    Json(serde_json::json!({
        "status": "success",
        "stable_count": stable_count,
        "evolving_count": evolving_count,
        "checkpoints_count": recent_checkpoints.len(),
        "ladybug_nodes_count": ladybug_nodes_count,
        "ladybug_edges_count": ladybug_edges_count,
        "pinned_sessions_count": state.memory.snapshots().pinned_sessions_count(),
        "snapshot": state.memory.snapshots().current_global(),
        "recent_checkpoints": recent_checkpoints,
        "recent_memories": recent_memories,
    }))
}
