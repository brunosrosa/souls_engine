//! # Souls Evolutive Harness Persistence (`souls_memory::evolve`)
//!
//! Implements the persistent organs of `v0.9.4` (`ADR-034`):
//! 1. **JIT Conflict Invalidation (TencentDB Tombstone Pattern):** Logical tombstoning
//!    (`status = 'superseded'`) in SQLite `STRICT` + `conflicts_with` edges in `LadybugDB`
//!    without physically deleting `STABLE` rows (`ADR-006`), plus 1-click restoration.
//! 2. **WeEvolve Closed-Loop Rating Engine:** Transplanted Bradley-Terry ELO & EMA ($\alpha = 0.15$)
//!    feedback mathematics from `_donor/souls_mc_core/src/cognition/learning/`.
//! 3. **Cognitive Catnap Consolidation Ledger & Domain Experience Library:**
//!    Persistent `EXPERIENCE.md` trees and $\mathcal{O}(M)$ `INDEX.md` inventory with
//!    `always_on_chars` census (Google Harness² `library.py`).

#![forbid(unsafe_code)]

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};
use souls_protocol::{
    AuthorizationProvenanceDto, CatnapSliceRecordDto, DomainExperienceEntryDto,
    EpistemicMemoryRecord, MemoryTombstoneDto, WeEvolveRatingDto,
};
use sqlx::{Pool, Row, Sqlite};

use crate::error::MemoryError;
use crate::ladybug::{LadybugOntologyGraph, OntologicalEdge, OntologicalNode};

/// Default Bradley-Terry ELO baseline rating.
pub const WEEVOLVE_BASELINE_ELO: f64 = 1200.0;
/// Default Bradley-Terry K-factor.
pub const WEEVOLVE_K_FACTOR: f64 = 32.0;
/// Default EMA smoothing alpha (`0.15`).
pub const WEEVOLVE_EMA_ALPHA: f64 = 0.15;

const REPLACEMENT_MARKERS: &[&str] = &[
    "substitu",
    "em vez de",
    "ao invés de",
    "ao inves de",
    "não use",
    "nao use",
    "nunca use",
    "agora usamos",
    "descontinuad",
    "trocamos",
    "mudamos para",
    "atualizado para",
    "replaced",
    "instead of",
    "never use",
    "do not use",
    "don't use",
    "deprecated",
    "switched to",
    "changed to",
];

const POSITIVE_POLARITY_MARKERS: &[&str] = &[
    "sempre ",
    "usar ",
    "use ",
    "utilizar ",
    "habilitar ",
    "enable ",
    "adotar ",
    "preferir ",
];

const NEGATIVE_POLARITY_MARKERS: &[&str] = &[
    "nunca ",
    "não usar",
    "nao usar",
    "proibido ",
    "desabilitar ",
    "disable ",
    "evitar ",
    "banir ",
];

const STOPWORDS: &[&str] = &[
    "para", "com", "sem", "por", "uma", "uns", "das", "dos", "que", "como", "mais", "menos",
    "usar", "use", "agora", "sempre", "nunca", "nao", "não", "substituimos", "substituímos",
    "trocamos", "mudamos", "vez", "the", "and", "for", "with", "instead", "never", "always",
    "from", "into", "that", "this", "em", "de", "da", "do", "no", "na", "os", "as", "um",
    // Generic domain nouns that must not trigger single-word contradictions
    "sistema", "projeto", "codigo", "código", "arquivo", "arquivos", "usuario", "usuário",
    "configuracao", "configuração", "dados", "modelo", "modelos", "ferramenta", "ferramentas",
    "engine", "souls", "hermes", "servidor", "banco", "tabela", "aplicacao", "aplicação",
    "servico", "serviço", "funcao", "função",
];

fn preview_text(text: &str, max_chars: usize) -> String {
    let trimmed = text.trim();
    if trimmed.chars().count() <= max_chars {
        trimmed.to_string()
    } else {
        let mut s: String = trimmed.chars().take(max_chars.saturating_sub(3)).collect();
        s.push_str("...");
        s
    }
}

fn short_hash16(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let hex = format!("{:x}", hasher.finalize());
    hex.chars().take(16).collect()
}

fn extract_subject_tokens(text: &str) -> HashSet<String> {
    let mut tokens = HashSet::new();
    for raw in text.split(|c: char| !c.is_alphanumeric() && c != '_' && c != '-') {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            continue;
        }
        let is_single_upper_symbol = trimmed.chars().count() == 1
            && trimmed
                .chars()
                .next()
                .map(|c| c.is_ascii_uppercase())
                .unwrap_or(false);
        let lower = trimmed.to_ascii_lowercase();
        if STOPWORDS.contains(&lower.as_str()) {
            continue;
        }
        if lower.chars().count() >= 3 || is_single_upper_symbol {
            tokens.insert(lower);
        }
    }
    tokens
}

fn split_key_value_prefix(text: &str) -> Option<(String, String)> {
    let idx = text.find(':')?;
    if !(3..=32).contains(&idx) {
        return None;
    }
    if text[..=idx].contains("://") {
        return None;
    }
    let key_slice = text[..idx].trim();
    if key_slice.chars().count() < 3
        || !key_slice
            .chars()
            .all(|c| c.is_alphanumeric() || matches!(c, ' ' | '_' | '-' | '.'))
    {
        return None;
    }
    let val_slice = text[idx + 1..].trim();
    if val_slice.is_empty() || val_slice.starts_with("//") {
        return None;
    }
    Some((key_slice.to_ascii_lowercase(), val_slice.to_ascii_lowercase()))
}

fn detect_contradiction_reason(old_content: &str, new_content: &str) -> Option<String> {
    let old_trimmed = old_content.trim();
    let new_trimmed = new_content.trim();
    if old_trimmed.eq_ignore_ascii_case(new_trimmed) {
        return None;
    }

    let old_lower = old_trimmed.to_ascii_lowercase();
    let new_lower = new_trimmed.to_ascii_lowercase();

    // Pattern A: Explicit key-prefix override ("modelo: deepseek" vs "modelo: claude")
    if let (Some((k1, v1)), Some((k2, v2))) = (
        split_key_value_prefix(old_trimmed),
        split_key_value_prefix(new_trimmed),
    ) {
        if k1 == k2 && v1 != v2 {
            return Some(format!(
                "Valor da diretriz '{}' atualizado de '{}' para nova definição.",
                k1,
                preview_text(&v1, 48)
            ));
        }
    }

    let old_tokens = extract_subject_tokens(old_trimmed);
    let new_tokens = extract_subject_tokens(new_trimmed);
    let shared: Vec<String> = old_tokens.intersection(&new_tokens).cloned().collect();
    if shared.is_empty() {
        return None;
    }

    // Pattern B: Explicit replacement / deprecation directive in new memory
    // Require >= 2 shared specific tokens OR explicit direct naming ("nunca use X", "substituimos X", etc.)
    let new_has_replacement = REPLACEMENT_MARKERS.iter().any(|m| new_lower.contains(m));
    let old_has_replacement = REPLACEMENT_MARKERS.iter().any(|m| old_lower.contains(m));
    if new_has_replacement && !old_has_replacement {
        let direct_named = shared.iter().any(|tok| {
            [
                "substituimos ",
                "substituímos ",
                "nunca use ",
                "nao use ",
                "não use ",
                "never use ",
                "do not use ",
                "don't use ",
                "em vez de ",
                "ao inves de ",
                "ao invés de ",
                "instead of ",
                "descontinuado ",
                "deprecated ",
            ]
            .iter()
            .any(|prefix| new_lower.contains(&format!("{prefix}{tok}")))
        });
        if shared.len() >= 2 || direct_named {
            return Some(format!(
                "Substituição imediata detectada sobre o tema '{}'.",
                shared.join(", ")
            ));
        }
    }

    // Pattern C: Direct polarity reversal on >= 2 shared subject tokens
    if shared.len() >= 2 {
        let old_pos = POSITIVE_POLARITY_MARKERS
            .iter()
            .any(|m| old_lower.contains(m));
        let new_neg = NEGATIVE_POLARITY_MARKERS
            .iter()
            .any(|m| new_lower.contains(m));
        let old_neg = NEGATIVE_POLARITY_MARKERS
            .iter()
            .any(|m| old_lower.contains(m));
        let new_pos = POSITIVE_POLARITY_MARKERS
            .iter()
            .any(|m| new_lower.contains(m));
        if (old_pos && new_neg) || (old_neg && new_pos) {
            return Some(format!(
                "Inversão direta de diretriz detectada sobre '{}'.",
                shared.join(", ")
            ));
        }
    }

    None
}

/// Lists all currently active (`status = 'superseded'`) memory IDs in `0 ms` SQLite B-tree time.
pub async fn list_active_tombstoned_ids(pool: &Pool<Sqlite>) -> Result<HashSet<String>, MemoryError> {
    let rows = sqlx::query(
        "SELECT superseded_memory_id FROM epistemic_memory_tombstones WHERE status = 'superseded'",
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|r| r.get(0)).collect())
}

/// Inserts an explicit logical tombstone and registers a `conflicts_with` edge in `LadybugDB`
/// after ensuring both endpoints exist (`insert_node`) so `NodeNotFound` never triggers.
pub async fn apply_memory_tombstone(
    pool: &Pool<Sqlite>,
    ladybug: &LadybugOntologyGraph,
    superseded_mem: &EpistemicMemoryRecord,
    superseding_mem: &EpistemicMemoryRecord,
    reason_pt: &str,
    now_epoch: i64,
) -> Result<MemoryTombstoneDto, MemoryError> {
    let tombstone_id = format!(
        "tmb_{}",
        short_hash16(&format!("{}->{}", superseding_mem.id, superseded_mem.id))
    );
    let old_prev = preview_text(&superseded_mem.content, 160);
    let new_prev = preview_text(&superseding_mem.content, 160);

    sqlx::query(
        r#"
        INSERT INTO epistemic_memory_tombstones
        (tombstone_id, superseded_memory_id, superseded_by_memory_id, category, superseded_content_preview, new_content_preview, reason_pt, status, created_at, restored_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'superseded', ?8, NULL)
        ON CONFLICT(superseded_memory_id) DO UPDATE SET
            tombstone_id = excluded.tombstone_id,
            superseded_by_memory_id = excluded.superseded_by_memory_id,
            category = excluded.category,
            superseded_content_preview = excluded.superseded_content_preview,
            new_content_preview = excluded.new_content_preview,
            reason_pt = excluded.reason_pt,
            status = 'superseded',
            created_at = excluded.created_at,
            restored_at = NULL
        "#,
    )
    .bind(&tombstone_id)
    .bind(&superseded_mem.id)
    .bind(&superseding_mem.id)
    .bind(&superseding_mem.category)
    .bind(&old_prev)
    .bind(&new_prev)
    .bind(reason_pt)
    .bind(now_epoch)
    .execute(pool)
    .await?;

    // Guarantee both nodes exist in LadybugDB before linking conflicts_with edge
    let new_node_id = format!("mem_node_{}", superseding_mem.id);
    let old_node_id = format!("mem_node_{}", superseded_mem.id);

    ladybug
        .insert_node(
            pool,
            OntologicalNode {
                id: new_node_id.clone(),
                node_type: "epistemic_memory".to_string(),
                canonical_name: format!("memory:{}:{}", superseding_mem.category, short_hash16(&superseding_mem.id)),
                metadata_json: serde_json::json!({
                    "memory_id": superseding_mem.id,
                    "category": superseding_mem.category,
                    "preview": preview_text(&superseding_mem.content, 64),
                })
                .to_string(),
                banned_patterns: Vec::new(),
            },
            now_epoch,
        )
        .await?;

    ladybug
        .insert_node(
            pool,
            OntologicalNode {
                id: old_node_id.clone(),
                node_type: "epistemic_memory".to_string(),
                canonical_name: format!("memory:{}:{}", superseded_mem.category, short_hash16(&superseded_mem.id)),
                metadata_json: serde_json::json!({
                    "memory_id": superseded_mem.id,
                    "category": superseded_mem.category,
                    "tombstone_status": "superseded",
                    "preview": preview_text(&superseded_mem.content, 64),
                })
                .to_string(),
                banned_patterns: Vec::new(),
            },
            now_epoch,
        )
        .await?;

    ladybug
        .insert_edge(
            pool,
            OntologicalEdge {
                source_id: new_node_id,
                target_id: old_node_id,
                relationship: "conflicts_with".to_string(),
                weight: 1.0,
            },
            now_epoch,
        )
        .await?;

    Ok(MemoryTombstoneDto {
        tombstone_id,
        superseded_memory_id: superseded_mem.id.clone(),
        superseded_by_memory_id: superseding_mem.id.clone(),
        category: superseding_mem.category.clone(),
        superseded_content_preview: old_prev,
        new_content_preview: new_prev,
        reason_pt: reason_pt.to_string(),
        status: "superseded".to_string(),
        created_at: now_epoch,
        restored_at: None,
    })
}

/// Finds contradiction candidates strictly within the `EVOLVING` partition and same `category`,
/// enforcing unidirectional temporal causality (`old.updated_at < new.updated_at`, or `id < new.id` tie-break)
/// so `STABLE` is never mutated automatically and mutual/circular tombstones cannot occur.
async fn find_jit_tombstone_candidates(
    pool: &Pool<Sqlite>,
    new_mem: &EpistemicMemoryRecord,
) -> Result<Vec<(EpistemicMemoryRecord, String)>, MemoryError> {
    if new_mem.partition == souls_protocol::EpistemicPartition::Stable {
        return Ok(Vec::new());
    }

    let already_superseded: Option<i64> = sqlx::query_scalar(
        "SELECT 1 FROM epistemic_memory_tombstones WHERE superseded_memory_id = ?1 AND status = 'superseded' LIMIT 1",
    )
    .bind(&new_mem.id)
    .fetch_optional(pool)
    .await?;
    if already_superseded.is_some() {
        return Ok(Vec::new());
    }

    let rows = sqlx::query(
        r#"
        SELECT id, session_id, category, content, partition, salience, access_count, source_ref, created_at, last_accessed_at, updated_at
        FROM epistemic_memories
        WHERE id != ?1
          AND category = ?2
          AND UPPER(partition) != 'STABLE'
          AND (updated_at < ?3 OR (updated_at = ?3 AND id < ?1))
          AND id NOT IN (SELECT superseded_memory_id FROM epistemic_memory_tombstones WHERE status = 'superseded')
        ORDER BY updated_at DESC
        LIMIT 120
        "#,
    )
    .bind(&new_mem.id)
    .bind(&new_mem.category)
    .bind(new_mem.updated_at)
    .fetch_all(pool)
    .await?;

    let mut matches = Vec::new();
    for r in rows {
        let part_str: String = r.get(4);
        let partition = match part_str.as_str() {
            "STABLE" => souls_protocol::EpistemicPartition::Stable,
            _ => souls_protocol::EpistemicPartition::Evolving,
        };
        let old_mem = EpistemicMemoryRecord {
            id: r.get(0),
            session_id: r.get(1),
            category: r.get(2),
            content: r.get(3),
            partition,
            salience: r.get::<f64, _>(5) as f32,
            access_count: r.get(6),
            source_ref: r.get(7),
            created_at: r.get(8),
            last_accessed_at: r.get(9),
            updated_at: r.get(10),
        };

        if let Some(reason_pt) = detect_contradiction_reason(&old_mem.content, &new_mem.content) {
            matches.push((old_mem, reason_pt));
        }
    }

    Ok(matches)
}

/// Strictly read-only dry-run preview of JIT tombstone candidates without mutating SQLite or LadybugDB.
pub async fn preview_jit_tombstones(
    pool: &Pool<Sqlite>,
    new_mem: &EpistemicMemoryRecord,
    now_epoch: i64,
) -> Result<Vec<MemoryTombstoneDto>, MemoryError> {
    let candidates = find_jit_tombstone_candidates(pool, new_mem).await?;
    Ok(candidates
        .into_iter()
        .map(|(old_mem, reason_pt)| MemoryTombstoneDto {
            tombstone_id: format!(
                "preview_{}",
                short_hash16(&format!("{}->{}", new_mem.id, old_mem.id))
            ),
            superseded_memory_id: old_mem.id,
            superseded_by_memory_id: new_mem.id.clone(),
            category: new_mem.category.clone(),
            superseded_content_preview: preview_text(&old_mem.content, 160),
            new_content_preview: preview_text(&new_mem.content, 160),
            reason_pt,
            status: "preview".to_string(),
            created_at: now_epoch,
            restored_at: None,
        })
        .collect())
}

/// Scans active `EVOLVING` memories in the same category for contradictions against `new_mem`
/// and applies logical JIT tombstones (`status = 'superseded'`), never mutating `STABLE` records automatically.
pub async fn detect_and_apply_jit_tombstones(
    pool: &Pool<Sqlite>,
    ladybug: &LadybugOntologyGraph,
    new_mem: &EpistemicMemoryRecord,
    now_epoch: i64,
) -> Result<Vec<MemoryTombstoneDto>, MemoryError> {
    let candidates = find_jit_tombstone_candidates(pool, new_mem).await?;
    let mut applied = Vec::with_capacity(candidates.len());
    for (old_mem, reason_pt) in candidates {
        let tomb = apply_memory_tombstone(pool, ladybug, &old_mem, new_mem, &reason_pt, now_epoch)
            .await?;
        applied.push(tomb);
    }
    Ok(applied)
}

/// Lists recent memory tombstones (`superseded` and `restored`) for Dashboard/MCP audit.
pub async fn list_memory_tombstones(
    pool: &Pool<Sqlite>,
    limit: usize,
) -> Result<Vec<MemoryTombstoneDto>, MemoryError> {
    let rows = sqlx::query(
        r#"
        SELECT tombstone_id, superseded_memory_id, superseded_by_memory_id, category,
               superseded_content_preview, new_content_preview, reason_pt, status, created_at, restored_at
        FROM epistemic_memory_tombstones
        ORDER BY created_at DESC
        LIMIT ?1
        "#,
    )
    .bind(limit.max(1) as i64)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| MemoryTombstoneDto {
            tombstone_id: r.get(0),
            superseded_memory_id: r.get(1),
            superseded_by_memory_id: r.get(2),
            category: r.get(3),
            superseded_content_preview: r.get(4),
            new_content_preview: r.get(5),
            reason_pt: r.get(6),
            status: r.get(7),
            created_at: r.get(8),
            restored_at: r.get(9),
        })
        .collect())
}

/// Restores a previously superseded memory in 1 click (`status = 'restored'`),
/// immediately re-enabling it in RRF Recall and FTS5 search.
pub async fn restore_tombstoned_memory(
    pool: &Pool<Sqlite>,
    tombstone_id: &str,
    now_epoch: i64,
) -> Result<Option<MemoryTombstoneDto>, MemoryError> {
    let updated = sqlx::query(
        r#"
        UPDATE epistemic_memory_tombstones
        SET status = 'restored', restored_at = ?1
        WHERE tombstone_id = ?2 OR superseded_memory_id = ?2
        "#,
    )
    .bind(now_epoch)
    .bind(tombstone_id)
    .execute(pool)
    .await?;

    if updated.rows_affected() == 0 {
        return Ok(None);
    }

    let row = sqlx::query(
        r#"
        SELECT tombstone_id, superseded_memory_id, superseded_by_memory_id, category,
               superseded_content_preview, new_content_preview, reason_pt, status, created_at, restored_at
        FROM epistemic_memory_tombstones
        WHERE tombstone_id = ?1 OR superseded_memory_id = ?1
        LIMIT 1
        "#,
    )
    .bind(tombstone_id)
    .fetch_optional(pool)
    .await?;

    let dto = row.map(|r| MemoryTombstoneDto {
        tombstone_id: r.get(0),
        superseded_memory_id: r.get(1),
        superseded_by_memory_id: r.get(2),
        category: r.get(3),
        superseded_content_preview: r.get(4),
        new_content_preview: r.get(5),
        reason_pt: r.get(6),
        status: r.get(7),
        created_at: r.get(8),
        restored_at: r.get(9),
    });

    if let Some(ref restored) = dto {
        let restored_node_id = format!("mem_node_{}", restored.superseded_memory_id);
        let _ = sqlx::query(
            "DELETE FROM ladybug_edges WHERE relationship = 'conflicts_with' AND target_id = ?1",
        )
        .bind(&restored_node_id)
        .execute(pool)
        .await;
    }

    Ok(dto)
}

// ============================================================================
// WeEvolve Bradley-Terry ELO & EMA Closed-Loop Engine (from _donor/ratings.rs)
// ============================================================================

/// Logistic sigmoid scaling of raw reward into `(0.0, 1.0)`.
#[must_use]
pub fn sigmoid_reward(reward_value: f64) -> f64 {
    1.0 / (1.0 + (-reward_value).exp())
}

/// Calculates Bradley-Terry ELO update against baseline difficulty (`R_baseline = 1200.0`).
#[must_use]
pub fn calculate_bradley_terry_elo(
    r_self: f64,
    r_baseline: f64,
    k: f64,
    reward_value: f64,
) -> (f64, f64) {
    let s = sigmoid_reward(reward_value);
    let e_self = 1.0 / (1.0 + 10.0_f64.powf((r_baseline - r_self) / 400.0));
    let r_new = r_self + k * (s - e_self);
    (r_new, s)
}

/// Exponential moving average smoothing (`EMA_{t+1} = alpha * S + (1 - alpha) * EMA_t`).
#[must_use]
pub fn update_ema(current_ema: f64, s: f64, alpha: f64) -> f64 {
    (alpha * s + (1.0 - alpha) * current_ema).clamp(0.0, 1.0)
}

/// Maps canonical telemetry/feedback action names to numerical WeEvolve rewards.
#[must_use]
pub fn map_signal_to_reward(signal: &str) -> f64 {
    match signal.trim().to_ascii_lowercase().as_str() {
        "user_approved" | "skill_promoted" => 1.4,
        "test_success" => 1.2,
        "subagent_completed" | "tool_call_clean" | "positive" => 1.0,
        "manual_edit_distance" => -0.8,
        "compilation_failure" | "subagent_failed" | "negative" => -1.0,
        "git_rollback" | "user_reverted" | "cordis_rollback" => -1.5,
        _ => 0.0,
    }
}

/// Records an empirical WeEvolve signal in `weevolve_ratings` (`STRICT`) and returns the updated DTO.
pub async fn record_weevolve_signal(
    pool: &Pool<Sqlite>,
    target_id: &str,
    target_kind: &str,
    signal: &str,
    custom_reward: Option<f64>,
    now_epoch: i64,
) -> Result<WeEvolveRatingDto, MemoryError> {
    let valid_kind = match target_kind {
        "cordis_fiber" | "skill" | "tool_guidance" | "subagent_profile" => target_kind,
        _ => "cordis_fiber",
    };

    sqlx::query(
        r#"
        INSERT INTO weevolve_ratings (target_id, target_kind, elo_rating, ema_score, total_matches, last_signal, updated_at)
        VALUES (?1, ?2, ?3, 0.5, 0, 'init', ?4)
        ON CONFLICT(target_id) DO NOTHING
        "#,
    )
    .bind(target_id)
    .bind(valid_kind)
    .bind(WEEVOLVE_BASELINE_ELO)
    .bind(now_epoch)
    .execute(pool)
    .await?;

    let row = sqlx::query(
        "SELECT target_kind, elo_rating, ema_score, total_matches FROM weevolve_ratings WHERE target_id = ?1",
    )
    .bind(target_id)
    .fetch_one(pool)
    .await?;

    let kind_db: String = row.get(0);
    let r_self: f64 = row.get(1);
    let ema_t: f64 = row.get(2);
    let total_matches: i64 = row.get(3);

    let reward = custom_reward.unwrap_or_else(|| map_signal_to_reward(signal));
    let (r_new, s) =
        calculate_bradley_terry_elo(r_self, WEEVOLVE_BASELINE_ELO, WEEVOLVE_K_FACTOR, reward);
    let ema_new = update_ema(ema_t, s, WEEVOLVE_EMA_ALPHA);
    let next_matches = total_matches + 1;

    sqlx::query(
        r#"
        UPDATE weevolve_ratings
        SET elo_rating = ?1, ema_score = ?2, total_matches = ?3, last_signal = ?4, updated_at = ?5
        WHERE target_id = ?6
        "#,
    )
    .bind(r_new)
    .bind(ema_new)
    .bind(next_matches)
    .bind(signal)
    .bind(now_epoch)
    .bind(target_id)
    .execute(pool)
    .await?;

    Ok(WeEvolveRatingDto {
        target_id: target_id.to_string(),
        target_kind: kind_db,
        elo_rating: r_new,
        ema_score: ema_new,
        total_matches: next_matches,
        last_signal: signal.to_string(),
        updated_at: now_epoch,
    })
}

/// Lists WeEvolve ratings ordered by ELO descending.
pub async fn list_weevolve_ratings(
    pool: &Pool<Sqlite>,
    limit: usize,
) -> Result<Vec<WeEvolveRatingDto>, MemoryError> {
    let rows = sqlx::query(
        r#"
        SELECT target_id, target_kind, elo_rating, ema_score, total_matches, last_signal, updated_at
        FROM weevolve_ratings
        ORDER BY elo_rating DESC, updated_at DESC
        LIMIT ?1
        "#,
    )
    .bind(limit.max(1) as i64)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| WeEvolveRatingDto {
            target_id: r.get(0),
            target_kind: r.get(1),
            elo_rating: r.get(2),
            ema_score: r.get(3),
            total_matches: r.get(4),
            last_signal: r.get(5),
            updated_at: r.get(6),
        })
        .collect())
}

// ============================================================================
// Catnap Consolidation Ledger & Domain Experience Library (library.py port)
// ============================================================================

/// Persists an executed Cognitive Catnap micro-slice into `catnap_consolidation_ledger`.
pub async fn record_catnap_slice(
    pool: &Pool<Sqlite>,
    slice: &CatnapSliceRecordDto,
) -> Result<(), MemoryError> {
    let provenance_json = match &slice.authorization_provenance {
        Some(prov) => serde_json::to_string(prov).unwrap_or_else(|_| "{}".to_string()),
        None => "{}".to_string(),
    };
    let trigger_mode = if slice.trigger_mode.trim().is_empty() {
        "auto"
    } else {
        slice.trigger_mode.trim()
    };
    let actor = if slice.actor.trim().is_empty() {
        "catnap_scheduler"
    } else {
        slice.actor.trim()
    };

    sqlx::query(
        r#"
        INSERT INTO catnap_consolidation_ledger
        (slice_id, slice_kind, runmark_fingerprint, cursor_checkpoint, status, duration_ms, cost_usd, summary_pt, trigger_mode, actor, provenance_json, created_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
        ON CONFLICT(slice_id) DO UPDATE SET
            status = excluded.status,
            duration_ms = excluded.duration_ms,
            cost_usd = excluded.cost_usd,
            summary_pt = excluded.summary_pt,
            trigger_mode = excluded.trigger_mode,
            actor = excluded.actor,
            provenance_json = excluded.provenance_json
        "#,
    )
    .bind(&slice.slice_id)
    .bind(&slice.slice_kind)
    .bind(&slice.runmark_fingerprint)
    .bind(&slice.cursor_checkpoint)
    .bind(&slice.status)
    .bind(slice.duration_ms)
    .bind(slice.cost_usd)
    .bind(&slice.summary_pt)
    .bind(trigger_mode)
    .bind(actor)
    .bind(&provenance_json)
    .bind(slice.created_at)
    .execute(pool)
    .await?;

    Ok(())
}

/// Lists recent Cognitive Catnap micro-slices from SQLite.
pub async fn list_catnap_slices(
    pool: &Pool<Sqlite>,
    limit: usize,
) -> Result<Vec<CatnapSliceRecordDto>, MemoryError> {
    let rows = sqlx::query(
        r#"
        SELECT slice_id, slice_kind, runmark_fingerprint, cursor_checkpoint, status, duration_ms, cost_usd, summary_pt, trigger_mode, actor, provenance_json, created_at
        FROM catnap_consolidation_ledger
        ORDER BY created_at DESC
        LIMIT ?1
        "#,
    )
    .bind(limit.max(1) as i64)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| {
            let prov_raw: String = r.get(10);
            let authorization_provenance = if prov_raw.trim().is_empty() || prov_raw.trim() == "{}" {
                None
            } else {
                serde_json::from_str::<AuthorizationProvenanceDto>(&prov_raw).ok()
            };
            CatnapSliceRecordDto {
                slice_id: r.get(0),
                slice_kind: r.get(1),
                runmark_fingerprint: r.get(2),
                cursor_checkpoint: r.get(3),
                status: r.get(4),
                duration_ms: r.get(5),
                cost_usd: r.get(6),
                summary_pt: r.get(7),
                trigger_mode: r.get(8),
                actor: r.get(9),
                authorization_provenance,
                created_at: r.get(11),
            }
        })
        .collect())
}

/// Computes completed Catnap slices and total USD spent since `utc_day_start_epoch`.
pub async fn get_catnap_utc_day_totals(
    pool: &Pool<Sqlite>,
    utc_day_start_epoch: i64,
) -> Result<(u64, f64), MemoryError> {
    let row = sqlx::query(
        r#"
        SELECT COUNT(*), COALESCE(SUM(cost_usd), 0.0)
        FROM catnap_consolidation_ledger
        WHERE status = 'completed' AND created_at >= ?1
        "#,
    )
    .bind(utc_day_start_epoch)
    .fetch_one(pool)
    .await?;

    let count: i64 = row.get(0);
    let spent: f64 = row.get(1);
    Ok((count.max(0) as u64, spent.max(0.0)))
}

/// Returns `true` if a LadybugDB `canonical_name` looks like unvalidated prose/memory residue.
fn is_residue_ladybug_node(node_type: &str, canonical_name: &str) -> bool {
    let trimmed = canonical_name.trim();
    if trimmed.contains("http://")
        || trimmed.contains("https://")
        || trimmed.contains("://")
        || trimmed.contains('\n')
        || trimmed.contains('\r')
        || trimmed.chars().count() > 80
    {
        return true;
    }
    if node_type == "epistemic_memory"
        && (trimmed.contains("Souls Engine (")
            || trimmed.contains("born ")
            || trimmed.ends_with("..."))
    {
        return true;
    }
    false
}

/// Idempotently removes prose/URL residue nodes from LadybugDB (`ladybug_nodes`, `ON DELETE CASCADE` edges, and RAM).
pub async fn cleanup_residue_ladybug_nodes(
    pool: &Pool<Sqlite>,
    ladybug: &LadybugOntologyGraph,
) -> Result<Vec<String>, MemoryError> {
    let rows = sqlx::query("SELECT id, node_type, canonical_name FROM ladybug_nodes")
        .fetch_all(pool)
        .await?;

    let mut removed_ids = Vec::new();
    for r in rows {
        let id: String = r.get(0);
        let node_type: String = r.get(1);
        let canonical_name: String = r.get(2);
        if is_residue_ladybug_node(&node_type, &canonical_name) {
            ladybug.delete_node(pool, &id).await?;
            removed_ids.push(id);
        }
    }
    Ok(removed_ids)
}

/// Persistent episodic domain harness library (`harness2/core/library.py` port)
/// storing `EXPERIENCE.md`, `metadata.json`, and an $\mathcal{O}(M)$ `INDEX.md` with
/// `always_on_chars` census to prevent prompt bloat.
#[derive(Debug, Clone)]
pub struct DomainExperienceLibrary {
    root_dir: PathBuf,
}

impl DomainExperienceLibrary {
    /// Creates a new `DomainExperienceLibrary` rooted at `root_dir`.
    pub fn new<P: AsRef<Path>>(root_dir: P) -> Self {
        Self {
            root_dir: root_dir.as_ref().to_path_buf(),
        }
    }

    /// Returns the directory containing episodic harness archives.
    #[must_use]
    pub fn root_dir(&self) -> &Path {
        &self.root_dir
    }

    /// Persists an episode (`EXPERIENCE.md` + `metadata.json`) and rebuilds `INDEX.md` in O(M).
    /// Deduplicates by `runmark_fingerprint` and caps stored directories at `50`.
    pub fn record_episode(
        &self,
        entry: &DomainExperienceEntryDto,
    ) -> Result<PathBuf, MemoryError> {
        let existing = self.list_episodes().unwrap_or_default();
        let reused_id = existing
            .iter()
            .find(|ep| !entry.runmark_fingerprint.is_empty() && ep.runmark_fingerprint == entry.runmark_fingerprint)
            .map(|ep| ep.episode_id.clone());

        let mut persisted = entry.clone();
        if let Some(id) = reused_id {
            persisted.episode_id = id;
        }

        let ep_dir = self.root_dir.join(&persisted.episode_id);
        fs::create_dir_all(&ep_dir)?;

        let exp_md = format!(
            "# Episode: {}\n\n- **Domain:** {}\n- **Cue:** {}\n- **Always-On Chars:** {}\n- **Inventory:** skills={}, subagents={}, scripts={}, hooks={}\n- **Runmark Fingerprint:** `{}`\n\n## Experience Summary\n{}\n",
            persisted.episode_id,
            persisted.domain_tag,
            persisted.cue_pattern,
            persisted.always_on_chars,
            persisted.skills_count,
            persisted.subagents_count,
            persisted.scripts_count,
            persisted.hooks_count,
            persisted.runmark_fingerprint,
            persisted.experience_summary_pt
        )
        .replace("\r\n", "\n");

        fs::write(ep_dir.join("EXPERIENCE.md"), exp_md.as_bytes())?;
        let meta_json = serde_json::to_string_pretty(&persisted).unwrap_or_default() + "\n";
        fs::write(
            ep_dir.join("metadata.json"),
            meta_json.replace("\r\n", "\n").as_bytes(),
        )?;

        const MAX_RETAINED_EPISODES: usize = 50;
        let all_after = self.list_episodes().unwrap_or_default();
        if all_after.len() > MAX_RETAINED_EPISODES {
            for old_ep in all_after.iter().skip(MAX_RETAINED_EPISODES) {
                let _ = fs::remove_dir_all(self.root_dir.join(&old_ep.episode_id));
            }
        }

        self.rebuild_index_om()?;
        Ok(self.root_dir.join("INDEX.md"))
    }

    /// Rebuilds `INDEX.md` in O(M) single-pass scan across stored episode directories.
    pub fn rebuild_index_om(&self) -> Result<usize, MemoryError> {
        fs::create_dir_all(&self.root_dir)?;
        let entries = self.list_episodes()?;
        let mut lines = Vec::with_capacity(entries.len() + 4);
        lines.push("# Domain Experience Library Index".to_string());
        lines.push(String::new());
        lines.push("| Episode | Domain | Always-On Chars | Skills | Subagents | Scripts | Hooks | Cue |".to_string());
        lines.push("|---|---|---:|---:|---:|---:|---:|---|".to_string());
        for ep in &entries {
            lines.push(format!(
                "| {} | {} | {} | {} | {} | {} | {} | {} |",
                ep.episode_id,
                ep.domain_tag,
                ep.always_on_chars,
                ep.skills_count,
                ep.subagents_count,
                ep.scripts_count,
                ep.hooks_count,
                ep.cue_pattern.replace('|', "/")
            ));
        }
        let body = lines.join("\n") + "\n";
        fs::write(self.root_dir.join("INDEX.md"), body.as_bytes())?;
        Ok(entries.len())
    }

    /// Lists all recorded domain experience episodes ordered newest-first.
    pub fn list_episodes(&self) -> Result<Vec<DomainExperienceEntryDto>, MemoryError> {
        if !self.root_dir.exists() {
            return Ok(Vec::new());
        }
        let mut episodes = Vec::new();
        for dir_entry in fs::read_dir(&self.root_dir)? {
            let dir_entry = dir_entry?;
            let meta_path = dir_entry.path().join("metadata.json");
            if meta_path.is_file() {
                if let Ok(raw) = fs::read_to_string(&meta_path) {
                    if let Ok(parsed) = serde_json::from_str::<DomainExperienceEntryDto>(&raw) {
                        episodes.push(parsed);
                    }
                }
            }
        }
        episodes.sort_by(|a, b| b.created_at.cmp(&a.created_at).then_with(|| b.episode_id.cmp(&a.episode_id)));
        Ok(episodes)
    }
}
