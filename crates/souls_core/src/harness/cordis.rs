//! # DeepSeek Cordis Reversible Microkernel (`souls_core::harness::cordis`)
//!
//! Implements Spatiotemporal Composability (`arXiv:2608.25512`):
//! - **Temporal Composability (LIFO `ctx.effect` / `dispose()`):** Tactical adjustments (`c2`, `c5`,
//!   `c7`, `c8`) carry explicit inverse operations undone in strict reverse order in `< 1 ms`.
//! - **Spatial Composability (Reactive Coeffects `inject`):** Fibers declare dependencies on external
//!   capabilities (`local_llamacpp`, `nvml_dgpu`, `google_oauth`, etc.) and pause/resume gracefully.
//! - **Split-Brain Hemispheric Drift Guard:** Auto-reverts a tactical fiber after `>= 3` negative
//!   consecutive signals or `ema_score < 0.40`.

#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::sync::RwLock;
use std::time::Instant;

use souls_protocol::{CordisFiberDto, CordisRollbackReportDto, HarnessComponentSlot};

use crate::error::CoreError;
use crate::harness::surface::ConstitutionalPinVerifier;

/// Single reversible state mutation tracked on a Cordis fiber's LIFO undo stack.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectUndoOp {
    pub op_id: String,
    pub description_pt: String,
    pub state_key: String,
    pub new_value: String,
    pub previous_value: Option<String>,
}

#[derive(Debug, Clone)]
struct CordisFiberRecord {
    dto: CordisFiberDto,
    undo_stack: Vec<EffectUndoOp>,
}

#[derive(Debug, Default)]
struct CordisState {
    fibers: BTreeMap<String, CordisFiberRecord>,
    coeffects: BTreeMap<String, bool>,
    runtime_kv: BTreeMap<String, String>,
    next_seq: u64,
}

/// Thread-safe reversible microkernel governing tactical runtime adaptations.
#[derive(Debug)]
pub struct CordisMicrokernel {
    inner: RwLock<CordisState>,
}

impl Default for CordisMicrokernel {
    fn default() -> Self {
        Self::new()
    }
}

impl CordisMicrokernel {
    /// Creates a new `CordisMicrokernel` with default runtime coeffects active.
    #[must_use]
    pub fn new() -> Self {
        let mut coeffects = BTreeMap::new();
        coeffects.insert("local_llamacpp".to_string(), true);
        coeffects.insert("nvml_dgpu".to_string(), true);
        coeffects.insert("google_oauth".to_string(), true);
        coeffects.insert("audio_pipeline".to_string(), true);
        coeffects.insert("lance_vectors".to_string(), true);

        Self {
            inner: RwLock::new(CordisState {
                fibers: BTreeMap::new(),
                coeffects,
                runtime_kv: BTreeMap::new(),
                next_seq: 1,
            }),
        }
    }

    /// Mounts a tactical fiber (`c2`, `c5`, `c7`, `c8`) with its LIFO undo stack.
    ///
    /// Fails closed on `c1` (constitutional layer) and rejects direct bypass of `c4`/`c6`
    /// (which must go through Skills Hub Quarantine / Myers Diff).
    #[allow(clippy::too_many_arguments)]
    pub fn mount_fiber(
        &self,
        fiber_id: Option<String>,
        slot: HarnessComponentSlot,
        title_pt: &str,
        benefit_summary_pt: &str,
        ephemeral_cue_text: &str,
        target_task_type: Option<String>,
        coeffect_dependencies: Vec<String>,
        undo_ops: Vec<EffectUndoOp>,
        now_epoch: i64,
    ) -> Result<CordisFiberDto, CoreError> {
        ConstitutionalPinVerifier::reject_if_c1_mutated(slot)?;
        if matches!(
            slot,
            HarnessComponentSlot::C4Skills | HarnessComponentSlot::C6Scripts
        ) {
            return Err(CoreError::HarnessGovernanceViolation(format!(
                "Slot '{}' exige aprovação via Quarentena + Myers Diff antes de entrar em produção.",
                slot.as_code()
            )));
        }

        let mut guard = self.inner.write().map_err(|_| {
            CoreError::InternalPanic("CordisMicrokernel write lock poisoned".to_string())
        })?;

        let assigned_id = match fiber_id {
            Some(id) if !id.trim().is_empty() => id.trim().to_string(),
            _ => {
                let seq = guard.next_seq;
                guard.next_seq += 1;
                format!("cordis_fib_{seq:04}")
            }
        };

        // Ensure all referenced coeffects exist in registry (default true if newly declared)
        for dep in &coeffect_dependencies {
            guard.coeffects.entry(dep.clone()).or_insert(true);
        }
        let paused_by_coeffect = coeffect_dependencies
            .iter()
            .any(|dep| !guard.coeffects.get(dep).copied().unwrap_or(true));

        // Apply side effects and snapshot previous values for deterministic LIFO inversion
        let mut tracked_undo = Vec::with_capacity(undo_ops.len().max(1));
        if undo_ops.is_empty() {
            let cue_key = format!("fiber.cue.{assigned_id}");
            let prev = guard
                .runtime_kv
                .insert(cue_key.clone(), ephemeral_cue_text.to_string());
            tracked_undo.push(EffectUndoOp {
                op_id: format!("undo_{assigned_id}_cue"),
                description_pt: format!("Remover diretriz tática '{}'", title_pt),
                state_key: cue_key,
                new_value: ephemeral_cue_text.to_string(),
                previous_value: prev,
            });
        } else {
            for mut op in undo_ops {
                let prev = guard
                    .runtime_kv
                    .insert(op.state_key.clone(), op.new_value.clone());
                if op.previous_value.is_none() {
                    op.previous_value = prev;
                }
                tracked_undo.push(op);
            }
        }

        let dto = CordisFiberDto {
            fiber_id: assigned_id.clone(),
            slot,
            title_pt: title_pt.trim().to_string(),
            benefit_summary_pt: benefit_summary_pt.trim().to_string(),
            ephemeral_cue_text: ephemeral_cue_text.trim().to_string(),
            target_task_type: target_task_type.map(|s| s.trim().to_ascii_lowercase()),
            coeffect_dependencies,
            active: true,
            paused_by_coeffect,
            undo_operations_count: tracked_undo.len(),
            elo_rating: 1200.0,
            ema_score: 0.5,
            drift_streak: 0,
            created_at: now_epoch,
        };

        guard.fibers.insert(
            assigned_id,
            CordisFiberRecord {
                dto: dto.clone(),
                undo_stack: tracked_undo,
            },
        );

        Ok(dto)
    }

    /// Reverts a mounted fiber in strict LIFO order (`ctx.effect` undo stack) in `< 1 ms`.
    pub fn dispose_fiber(&self, fiber_id: &str) -> Result<CordisRollbackReportDto, CoreError> {
        let start = Instant::now();
        let mut guard = self.inner.write().map_err(|_| {
            CoreError::InternalPanic("CordisMicrokernel write lock poisoned".to_string())
        })?;

        let mut undo_items = {
            let rec = guard.fibers.get_mut(fiber_id).ok_or_else(|| {
                CoreError::HarnessGovernanceViolation(format!(
                    "Fibra Cordis '{}' não encontrada para reversão.",
                    fiber_id
                ))
            })?;
            rec.dto.active = false;
            std::mem::take(&mut rec.undo_stack)
        };

        let mut reverted_ops = Vec::with_capacity(undo_items.len());
        while let Some(op) = undo_items.pop() {
            match op.previous_value {
                Some(prev) => {
                    guard.runtime_kv.insert(op.state_key.clone(), prev);
                }
                None => {
                    guard.runtime_kv.remove(&op.state_key);
                }
            }
            reverted_ops.push(format!("{} ({})", op.op_id, op.state_key));
        }

        if let Some(rec) = guard.fibers.get_mut(fiber_id) {
            rec.dto.undo_operations_count = 0;
        }

        Ok(CordisRollbackReportDto {
            fiber_id: fiber_id.to_string(),
            reverted_operations: reverted_ops,
            elapsed_us: start.elapsed().as_micros() as u64,
            restored_cleanly: true,
        })
    }

    /// Reactively updates a coeffect provider state (`inject`), pausing or resuming dependent fibers.
    /// Returns the number of fibers whose pause state changed.
    pub fn update_coeffect(&self, provider: &str, available: bool) -> usize {
        let Ok(mut guard) = self.inner.write() else {
            return 0;
        };
        guard.coeffects.insert(provider.trim().to_string(), available);

        let coeffects_snapshot = guard.coeffects.clone();
        let mut changed = 0usize;
        for rec in guard.fibers.values_mut() {
            if !rec.dto.active {
                continue;
            }
            let should_pause = rec
                .dto
                .coeffect_dependencies
                .iter()
                .any(|dep| !coeffects_snapshot.get(dep).copied().unwrap_or(true));
            if rec.dto.paused_by_coeffect != should_pause {
                rec.dto.paused_by_coeffect = should_pause;
                changed += 1;
            }
        }
        changed
    }

    /// Evaluates Split-Brain hemispheric drift for a fiber; automatically executes `dispose_fiber`
    /// if `drift_streak >= 3` or `new_ema < 0.40`.
    pub fn record_hemispheric_signal(
        &self,
        fiber_id: &str,
        positive: bool,
        new_elo: f64,
        new_ema: f64,
    ) -> Option<CordisRollbackReportDto> {
        let should_rollback = {
            let Ok(mut guard) = self.inner.write() else {
                return None;
            };
            let Some(rec) = guard.fibers.get_mut(fiber_id) else {
                return None;
            };
            if !rec.dto.active {
                return None;
            }
            rec.dto.elo_rating = new_elo;
            rec.dto.ema_score = new_ema;
            if positive {
                rec.dto.drift_streak = 0;
            } else {
                rec.dto.drift_streak = rec.dto.drift_streak.saturating_add(1);
            }
            !positive && (rec.dto.drift_streak >= 3 || new_ema < 0.40)
        };

        if should_rollback {
            self.dispose_fiber(fiber_id).ok()
        } else {
            None
        }
    }

    /// Lists all fibers (active and reverted).
    #[must_use]
    pub fn list_fibers(&self) -> Vec<CordisFiberDto> {
        self.inner
            .read()
            .map(|g| g.fibers.values().map(|r| r.dto.clone()).collect())
            .unwrap_or_default()
    }

    /// Returns a snapshot of current reactive coeffects.
    #[must_use]
    pub fn list_coeffects(&self) -> BTreeMap<String, bool> {
        self.inner
            .read()
            .map(|g| g.coeffects.clone())
            .unwrap_or_default()
    }

    /// Reads an active runtime key managed by Cordis fibers.
    #[must_use]
    pub fn get_runtime_value(&self, key: &str) -> Option<String> {
        self.inner
            .read()
            .ok()
            .and_then(|g| g.runtime_kv.get(key).cloned())
    }

    /// Re-activates a previously disposed fiber cleanly without mutating `c1`, resetting `drift_streak`.
    pub fn reactivate_fiber(&self, fiber_id: &str) -> Result<CordisFiberDto, CoreError> {
        let mut guard = self.inner.write().map_err(|_| {
            CoreError::InternalPanic("CordisMicrokernel write lock poisoned".to_string())
        })?;

        let (cue_text, title_pt) = {
            let rec = guard.fibers.get_mut(fiber_id).ok_or_else(|| {
                CoreError::HarnessGovernanceViolation(format!(
                    "Fibra Cordis '{}' não encontrada para reativação.",
                    fiber_id
                ))
            })?;
            ConstitutionalPinVerifier::reject_if_c1_mutated(rec.dto.slot)?;
            rec.dto.active = true;
            rec.dto.drift_streak = 0;
            (rec.dto.ephemeral_cue_text.clone(), rec.dto.title_pt.clone())
        };

        let cue_key = format!("fiber.cue.{fiber_id}");
        let prev = guard.runtime_kv.insert(cue_key.clone(), cue_text.clone());

        let rec = guard.fibers.get_mut(fiber_id).expect("checked above");
        if rec.undo_stack.is_empty() {
            rec.undo_stack.push(EffectUndoOp {
                op_id: format!("undo_{fiber_id}_cue"),
                description_pt: format!("Remover diretriz tática '{}'", title_pt),
                state_key: cue_key,
                new_value: cue_text,
                previous_value: prev,
            });
        }
        rec.dto.undo_operations_count = rec.undo_stack.len();
        Ok(rec.dto.clone())
    }

    /// Synchronizes hydrated WeEvolve ratings (`elo_rating`, `ema_score`) from SQLite into RAM.
    pub fn sync_fiber_rating(&self, fiber_id: &str, elo_rating: f64, ema_score: f64) {
        if let Ok(mut guard) = self.inner.write() {
            if let Some(rec) = guard.fibers.get_mut(fiber_id) {
                rec.dto.elo_rating = elo_rating;
                rec.dto.ema_score = ema_score;
            }
        }
    }

    /// Selects contextually relevant Cordis cues for `query` (`prefetch()` noise elimination).
    /// Casual/personal queries (agenda, email, greeting) receive an empty list.
    #[must_use]
    pub fn select_query_aware_cues(&self, query: &str) -> Vec<String> {
        let lower = query.trim().to_ascii_lowercase();
        if lower.is_empty() {
            return Vec::new();
        }

        const CODE_MARKERS: &[&str] = &[
            "rust", "python", "cargo", "codigo", "código", "code", "bug", "erro", "error",
            "compile", "compil", "funcao", "função", "struct", "trait", "arquivo", "file",
            "refator", "test", "git", "ast", "outline", "slice", ".rs", ".py", ".js", ".ts",
            ".html", ".css", "sql", "json", "toml",
        ];
        const RESEARCH_MARKERS: &[&str] = &[
            "pesquis", "research", "subagent", "deleg", "storm", "analis", "anális",
            "paper", "arxiv", "estudo", "compar", "arquitetura", "benchmark", "deep",
        ];
        const OPS_ERROR_MARKERS: &[&str] = &[
            "falh", "erro", "trav", "loop", "timeout", "retry", "corrig", "fix", "debug",
        ];

        let is_code = CODE_MARKERS.iter().any(|m| lower.contains(m));
        let is_research = RESEARCH_MARKERS.iter().any(|m| lower.contains(m));
        let is_ops_error = OPS_ERROR_MARKERS.iter().any(|m| lower.contains(m));

        if !is_code && !is_research && !is_ops_error {
            return Vec::new();
        }

        let Ok(guard) = self.inner.read() else {
            return Vec::new();
        };

        guard
            .fibers
            .values()
            .filter(|r| {
                if !r.dto.active || r.dto.paused_by_coeffect || r.dto.ephemeral_cue_text.is_empty() {
                    return false;
                }
                match r.dto.target_task_type.as_deref() {
                    Some("code") | Some("coding") => is_code,
                    Some("research") => is_research,
                    Some(other) if !other.is_empty() && other != "all" => lower.contains(other),
                    _ => is_code || is_research || is_ops_error,
                }
            })
            .map(|r| format!("- {}", r.dto.ephemeral_cue_text))
            .collect()
    }

    /// Returns concise ephemeral cues for `prefetch()` (only active, non-paused fibers).
    #[must_use]
    pub fn active_prefetch_cues(&self) -> Vec<String> {
        let Ok(guard) = self.inner.read() else {
            return Vec::new();
        };
        guard
            .fibers
            .values()
            .filter(|r| {
                r.dto.active && !r.dto.paused_by_coeffect && !r.dto.ephemeral_cue_text.is_empty()
            })
            .map(|r| {
                format!(
                    "- [{}] {}: {}",
                    r.dto.slot.as_code(),
                    r.dto.title_pt,
                    r.dto.ephemeral_cue_text
                )
            })
            .collect()
    }

    /// Returns active tactical cues matching a delegated sub-agent `task_type` (`route_subtask`).
    #[must_use]
    pub fn active_cues_for_task_type(&self, task_type: &str) -> Vec<String> {
        let norm_task = task_type.trim().to_ascii_lowercase();
        let Ok(guard) = self.inner.read() else {
            return Vec::new();
        };
        guard
            .fibers
            .values()
            .filter(|r| {
                if !r.dto.active || r.dto.paused_by_coeffect || r.dto.ephemeral_cue_text.is_empty()
                {
                    return false;
                }
                match &r.dto.target_task_type {
                    None => true,
                    Some(target) => {
                        target.is_empty()
                            || target == "all"
                            || norm_task.contains(target.as_str())
                            || target.contains(norm_task.as_str())
                    }
                }
            })
            .map(|r| {
                format!(
                    "[{} / {}] {}",
                    r.dto.fiber_id, r.dto.title_pt, r.dto.ephemeral_cue_text
                )
            })
            .collect()
    }
}
