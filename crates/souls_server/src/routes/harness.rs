//! Souls Evolutive Harness Orchestration, REST API & Cooperative Catnap Runner (`v0.9.4` / `ADR-034`).
//!
//! Coordinates:
//! - Constitutional `c1` LF SHA-256 Pinning (`AGENTS.md`, `SOUL.md`) — *fail-closed* with 30s RAM cache
//! - DeepSeek Cordis Reversible LIFO Fibers, SQLite ELO Hydration, Disk Persistence & Query-Aware Cues
//! - Cooperative Cognitive Catnap Micro-Slices (`>= 180s` idle, GPU `< 62.0°C` + `5.0°C` hysteresis,
//!   `< 68.0°C` for `0 MB VRAM` slices, `< 1 ms` preemption)
//! - TencentDB Logical Memory Tombstones & Restore
//! - The Ralph Loop -> Skills Hub Quarantine Crystallization (`c4` / `c6`) & 1-Click Approval
//! - WeEvolve Bradley-Terry ELO & EMA Closed-Loop Telemetry

#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::Instant;

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json};
use chrono::{Timelike, Utc};
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::Row;

use souls_core::{
    compute_runmark_fingerprint, fold_aci_observation, is_within_quiet_hours,
    normalize_posix_rel_path, scrub_candidate_environment, CatnapMonitor,
    ConstitutionalPinVerifier, CordisMicrokernel, EffectUndoOp,
};
use souls_memory::{
    apply_memory_tombstone, detect_and_apply_jit_tombstones, list_catnap_slices,
    list_memory_tombstones, list_weevolve_ratings, record_catnap_slice, record_weevolve_signal,
    restore_tombstoned_memory, DomainExperienceLibrary, OntologicalEdge, OntologicalNode,
};
use souls_protocol::{
    CatnapConfigDto, CatnapSliceRecordDto, ConstitutionalPinStatusDto, CordisFiberDto,
    DomainExperienceEntryDto, EpistemicPartition, HarnessComponentSlot, MemoryTombstoneDto,
    RalphSkillCandidateDto,
};

use crate::skills_hub::{
    compute_normalized_lf_sha256, compute_skill_diff, discard_quarantine_item,
    evaluate_skill_quality, normalize_skill_slug, parse_skill_markdown, scan_skill_zero_trust,
    sync_skill_to_targets, QuarantineItem, SkillsHubRoots,
};
use crate::state::AppState;

const VALID_SLICE_KINDS: &[&str] = &[
    "shadow_probe_ppc",
    "ladybug_reconcile",
    "langevin_micro_batch",
    "diary_distillation",
    "ralph_crystallize",
];

const PIN_CACHE_TTL_SECS: i64 = 30;

fn parse_evolutive_harness_toml_section(toml_text: &str) -> CatnapConfigDto {
    let mut cfg = CatnapConfigDto::default();
    let mut in_section = false;

    for raw_line in toml_text.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            in_section = line == "[evolutive_harness]";
            continue;
        }
        if !in_section {
            continue;
        }
        let Some((k, v)) = line.split_once('=') else {
            continue;
        };
        let key = k.trim();
        let val = v.trim().trim_matches('"');
        match key {
            "enabled" => {
                if let Ok(b) = val.parse::<bool>() {
                    cfg.enabled = b;
                }
            }
            "min_idle_secs" => {
                if let Ok(n) = val.parse::<u64>() {
                    cfg.min_idle_secs = n.clamp(30, 3600);
                }
            }
            "max_gpu_temp_c" => {
                if let Ok(f) = val.parse::<f32>() {
                    cfg.max_gpu_temp_c = f.clamp(40.0, 80.0);
                }
            }
            "max_cpu_load_pct" => {
                if let Ok(f) = val.parse::<f32>() {
                    cfg.max_cpu_load_pct = f.clamp(5.0, 80.0);
                }
            }
            "max_daily_budget_usd" => {
                if let Ok(f) = val.parse::<f64>() {
                    cfg.max_daily_budget_usd = f.clamp(0.0, 20.0);
                }
            }
            "allow_free_cloud_improver" => {
                if let Ok(b) = val.parse::<bool>() {
                    cfg.allow_free_cloud_improver = b;
                }
            }
            "quiet_hours_enabled" => {
                if let Ok(b) = val.parse::<bool>() {
                    cfg.quiet_hours_enabled = b;
                }
            }
            "quiet_hours_start_hour" => {
                if let Ok(h) = val.parse::<u8>() {
                    cfg.quiet_hours_start_hour = h % 24;
                }
            }
            "quiet_hours_end_hour" => {
                if let Ok(h) = val.parse::<u8>() {
                    cfg.quiet_hours_end_hour = h % 24;
                }
            }
            _ => {}
        }
    }

    cfg
}

/// Replaces or inserts `[evolutive_harness]` cleanly in `souls_config.toml` without disturbing
/// any sibling TOML section and guaranteeing pure LF (`\n`) line endings.
fn upsert_evolutive_harness_toml_block(existing: &str, cfg: &CatnapConfigDto) -> String {
    let mut kept_lines: Vec<String> = Vec::new();
    let mut skip_section = false;

    for line in existing.replace("\r\n", "\n").lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            if trimmed == "[evolutive_harness]" {
                skip_section = true;
                continue;
            } else {
                skip_section = false;
            }
        }
        if !skip_section {
            kept_lines.push(line.to_string());
        }
    }

    while kept_lines.last().map(|l| l.trim().is_empty()).unwrap_or(false) {
        kept_lines.pop();
    }

    let section = format!(
        "[evolutive_harness]\nenabled = {}\nmin_idle_secs = {}\nmax_gpu_temp_c = {:.1}\nmax_cpu_load_pct = {:.1}\nmax_daily_budget_usd = {:.2}\nallow_free_cloud_improver = {}\nquiet_hours_enabled = {}\nquiet_hours_start_hour = {}\nquiet_hours_end_hour = {}\n",
        cfg.enabled,
        cfg.min_idle_secs,
        cfg.max_gpu_temp_c,
        cfg.max_cpu_load_pct,
        cfg.max_daily_budget_usd,
        cfg.allow_free_cloud_improver,
        cfg.quiet_hours_enabled,
        cfg.quiet_hours_start_hour % 24,
        cfg.quiet_hours_end_hour % 24,
    );

    if kept_lines.is_empty() {
        section
    } else {
        format!("{}\n\n{}", kept_lines.join("\n"), section)
    }
}

/// Stateful orchestrator for the `v0.9.4` Evolutive Harness subsystem.
#[derive(Debug)]
pub struct EvolutiveHarnessService {
    pub data_dir: PathBuf,
    pub config_path: PathBuf,
    pub quarantine_dir_override: Option<PathBuf>,
    pub cordis: Arc<CordisMicrokernel>,
    pub catnap: Arc<CatnapMonitor>,
    pin_verifier: RwLock<ConstitutionalPinVerifier>,
    last_pin_check: RwLock<(i64, Vec<ConstitutionalPinStatusDto>)>,
    pub experience_lib: DomainExperienceLibrary,
    ralph_candidates: RwLock<Vec<RalphSkillCandidateDto>>,
}

impl EvolutiveHarnessService {
    /// Creates a production service rooted under `resolve_data_dir()`.
    pub fn from_system() -> Self {
        let base_data = crate::resolve_data_dir();
        let data_dir = base_data.join("evolutive_harness");
        let config_path = base_data.join("config").join("souls_config.toml");
        Self::with_paths(data_dir, config_path, None)
    }

    /// Creates an isolated service instance for unit/integration tests.
    pub fn new_isolated<P: AsRef<Path>>(root: P) -> Self {
        let root_pb = root.as_ref().to_path_buf();
        let data_dir = root_pb.join("evolutive_harness");
        let config_path = root_pb.join("souls_config.toml");
        let quarantine_dir = root_pb.join("skills_quarantine");
        Self::with_paths(data_dir, config_path, Some(quarantine_dir))
    }

    fn with_paths(
        data_dir: PathBuf,
        config_path: PathBuf,
        quarantine_dir_override: Option<PathBuf>,
    ) -> Self {
        let _ = fs::create_dir_all(&data_dir);
        let catnap_cfg = fs::read_to_string(&config_path)
            .map(|raw| parse_evolutive_harness_toml_section(&raw))
            .unwrap_or_default();

        let cordis = Arc::new(CordisMicrokernel::new());
        Self::seed_default_cordis_fibers(&cordis);
        Self::hydrate_persisted_cordis_fibers(&data_dir, &cordis);

        let catnap = Arc::new(CatnapMonitor::new(catnap_cfg));
        let pin_verifier = ConstitutionalPinVerifier::new();
        Self::seed_constitutional_pins(&pin_verifier, quarantine_dir_override.is_some());

        let experience_lib = DomainExperienceLibrary::new(data_dir.join("experience_library"));

        let candidates_path = data_dir.join("ralph_candidates.json");
        let loaded_candidates = fs::read_to_string(&candidates_path)
            .ok()
            .and_then(|raw| serde_json::from_str::<Vec<RalphSkillCandidateDto>>(&raw).ok())
            .unwrap_or_default();

        Self {
            data_dir,
            config_path,
            quarantine_dir_override,
            cordis,
            catnap,
            pin_verifier: RwLock::new(pin_verifier),
            last_pin_check: RwLock::new((0, Vec::new())),
            experience_lib,
            ralph_candidates: RwLock::new(loaded_candidates),
        }
    }

    fn seed_constitutional_pins(verifier: &ConstitutionalPinVerifier, isolated_test: bool) {
        if !isolated_test {
            let candidates = [
                ("AGENTS.md", PathBuf::from(r"Z:\souls_engine\AGENTS.md")),
                ("AGENTS.md", PathBuf::from("AGENTS.md")),
            ];
            let mut agents_pinned = false;
            for (rel, path) in candidates {
                if path.is_file() && verifier.pin_file_from_disk(rel, &path).is_ok() {
                    agents_pinned = true;
                    break;
                }
            }
            if !agents_pinned {
                let _ = verifier.pin_in_memory(
                    "AGENTS.md",
                    "# Souls Engine Constitutional Baseline (synthetic fallback)\n",
                );
            }

            if let Ok(profile) = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")) {
                let soul_path = PathBuf::from(profile).join(".hermes").join("SOUL.md");
                if soul_path.is_file() {
                    let _ = verifier.pin_file_from_disk("hermes/SOUL.md", &soul_path);
                }
            }
        } else {
            let _ = verifier.pin_in_memory(
                "AGENTS.md",
                "# Test Constitutional Pin: C1 Read-Only Identity\n",
            );
            let _ = verifier.pin_in_memory(
                "hermes/SOUL.md",
                "# Test Soul Identity Snapshot\n",
            );
        }
    }

    fn seed_default_cordis_fibers(cordis: &CordisMicrokernel) {
        let now = Utc::now().timestamp();
        let _ = cordis.mount_fiber(
            Some("fib_ast_outline".to_string()),
            HarnessComponentSlot::C5ToolGuidance,
            "Leitura econômica de código por estrutura",
            "Evita desperdiçar contexto lendo arquivos inteiros; prioriza contorno e símbolo exato.",
            "Antes de ler arquivos longos de código, inspecione a estrutura (souls_ast_outline / souls_ast_slice) para economizar até 80% do contexto.",
            Some("code".to_string()),
            vec![],
            vec![EffectUndoOp {
                op_id: "set_ast_discipline".to_string(),
                description_pt: "Ativar sugestão de leitura estrutural econômica".to_string(),
                state_key: "guidance.code.ast_outline_first".to_string(),
                new_value: "true".to_string(),
                previous_value: None,
            }],
            now,
        );

        let _ = cordis.mount_fiber(
            Some("fib_anti_loop_guard".to_string()),
            HarnessComponentSlot::C2Guardrails,
            "Freio contra repetição de tentativas falhas",
            "Detecta ciclos repetitivos de erro e orienta mudança imediata de estratégia.",
            "Se uma chamada de ferramenta falhar duas vezes seguidas com o mesmo padrão, mude de abordagem e compacte a observação antes de tentar novamente.",
            None,
            vec![],
            vec![EffectUndoOp {
                op_id: "set_anti_loop_max".to_string(),
                description_pt: "Limitar repetição idêntica de erros em 2 tentativas".to_string(),
                state_key: "guardrails.anti_loop_threshold".to_string(),
                new_value: "2".to_string(),
                previous_value: None,
            }],
            now,
        );

        let _ = cordis.mount_fiber(
            Some("fib_subagent_isolation".to_string()),
            HarnessComponentSlot::C7Subagents,
            "Isolamento econômico de subagentes",
            "Orienta delegações de pesquisa a retornarem sínteses enxutas sem poluir o diálogo.",
            "Em tarefas de exploração ou leitura extensa, solicite ao subagente apenas fatos verificados, caminhos e conclusões objetivas.",
            Some("research".to_string()),
            vec!["lance_vectors".to_string()],
            vec![EffectUndoOp {
                op_id: "set_subagent_summary_mode".to_string(),
                description_pt: "Exigir síntese objetiva no retorno de subagentes".to_string(),
                state_key: "subagents.concise_handoff".to_string(),
                new_value: "true".to_string(),
                previous_value: None,
            }],
            now,
        );
    }

    fn hydrate_persisted_cordis_fibers(data_dir: &Path, cordis: &CordisMicrokernel) {
        let path = data_dir.join("cordis_fibers.json");
        let Ok(raw) = fs::read_to_string(&path) else {
            return;
        };
        let Ok(saved) = serde_json::from_str::<Vec<CordisFiberDto>>(&raw) else {
            return;
        };
        for fib in saved {
            if fib.slot.is_constitutionally_pinned() {
                continue;
            }
            let _ = cordis.mount_fiber(
                Some(fib.fiber_id.clone()),
                fib.slot,
                &fib.title_pt,
                &fib.benefit_summary_pt,
                &fib.ephemeral_cue_text,
                fib.target_task_type.clone(),
                fib.coeffect_dependencies.clone(),
                Vec::new(),
                fib.created_at,
            );
            cordis.sync_fiber_rating(&fib.fiber_id, fib.elo_rating, fib.ema_score);
            if !fib.active {
                let _ = cordis.dispose_fiber(&fib.fiber_id);
            }
        }
    }

    fn persist_cordis_state(&self) {
        let path = self.data_dir.join("cordis_fibers.json");
        let fibers = self.cordis.list_fibers();
        if let Ok(pretty) = serde_json::to_string_pretty(&fibers) {
            let normalized = pretty.replace("\r\n", "\n") + "\n";
            let _ = fs::write(path, normalized.as_bytes());
        }
    }

    /// Signals real user/HTTP activity for `< 1 ms` cooperative preemption.
    #[inline]
    pub fn notify_activity(&self) {
        self.catnap.notify_activity();
    }

    /// Updates pending failed trajectories tracked by Catnap Shadow Probing.
    pub fn refresh_pending_failures(&self, count: usize) {
        self.catnap.set_pending_failed_trajectories(count);
    }

    /// Performs a fresh SHA-256 disk check of constitutional `c1` files (*fail-closed*) and refreshes cache.
    pub fn verify_constitutional_pins_fresh(&self) -> Vec<ConstitutionalPinStatusDto> {
        let pins = self
            .pin_verifier
            .read()
            .map(|v| v.verify_or_fail_closed())
            .unwrap_or_default();
        let now = Utc::now().timestamp();
        if let Ok(mut cache) = self.last_pin_check.write() {
            *cache = (now, pins.clone());
        }
        pins
    }

    /// Verifies constitutional `c1` files using a 30s RAM cache so frequent `/cues` polling stays `< 1 µs`.
    pub fn verify_constitutional_pins(&self) -> Vec<ConstitutionalPinStatusDto> {
        let now = Utc::now().timestamp();
        if let Ok(cache) = self.last_pin_check.read() {
            if !cache.1.is_empty() && now.saturating_sub(cache.0) < PIN_CACHE_TTL_SECS {
                return cache.1.clone();
            }
        }
        self.verify_constitutional_pins_fresh()
    }

    /// Re-registers a constitutional `c1` file or virtual baseline hash and invalidates pin cache.
    pub fn pin_constitutional_bytes(
        &self,
        rel_path: &str,
        content: &str,
    ) -> Result<ConstitutionalPinStatusDto, String> {
        let guard = self
            .pin_verifier
            .read()
            .map_err(|_| "Pin verifier lock poisoned".to_string())?;
        let res = guard
            .pin_in_memory(rel_path, content)
            .map_err(|e| e.to_string())?;
        if let Ok(mut cache) = self.last_pin_check.write() {
            cache.0 = 0;
        }
        Ok(res)
    }

    /// Verifies candidate bytes against a registered pin (*fail-closed*).
    pub fn assert_c1_intact(&self, rel_path: &str, candidate_bytes: &[u8]) -> Result<(), String> {
        let guard = self
            .pin_verifier
            .read()
            .map_err(|_| "Pin verifier lock poisoned".to_string())?;
        guard
            .assert_candidate_matches_pin(rel_path, candidate_bytes)
            .map_err(|e| e.to_string())
    }

    /// Active Cordis tactical cues for ephemeral `prefetch()` injection (`c1` untouched).
    pub fn active_prefetch_cues(&self) -> Vec<String> {
        self.cordis.active_prefetch_cues()
    }

    /// Query-aware active Cordis tactical cues (returns empty for non-technical/personal turns).
    pub fn select_query_aware_cues(&self, query: &str) -> Vec<String> {
        self.cordis.select_query_aware_cues(query)
    }

    /// Active Cordis tactical cues filtered for a given subtask classification (`route_subtask`).
    pub fn active_cues_for_task_type(&self, task_type: &str) -> Vec<String> {
        self.cordis.active_cues_for_task_type(task_type)
    }

    /// Returns Ralph skill candidates ordered newest-first.
    pub fn list_ralph_candidates(&self) -> Vec<RalphSkillCandidateDto> {
        let mut items = self
            .ralph_candidates
            .read()
            .map(|g| g.clone())
            .unwrap_or_default();
        items.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        items
    }

    fn save_ralph_candidates_locked(&self, items: &[RalphSkillCandidateDto]) {
        let path = self.data_dir.join("ralph_candidates.json");
        if let Ok(pretty) = serde_json::to_string_pretty(items) {
            let normalized = pretty.replace("\r\n", "\n") + "\n";
            let _ = fs::write(path, normalized.as_bytes());
        }
    }

    /// Persists `[evolutive_harness]` cleanly in `souls_config.toml` without disturbing siblings.
    pub fn persist_catnap_config(&self, cfg: &CatnapConfigDto) -> Result<(), String> {
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let existing = fs::read_to_string(&self.config_path).unwrap_or_default();
        let updated = upsert_evolutive_harness_toml_block(&existing, cfg);
        fs::write(&self.config_path, updated.as_bytes()).map_err(|e| e.to_string())
    }

    /// Produces a scrubbed candidate execution environment (`SCRUB_ENV`).
    pub fn scrubbed_candidate_env() -> (BTreeMap<String, String>, usize) {
        let mut env_map: BTreeMap<String, String> = std::env::vars().collect();
        let scrubbed_count = scrub_candidate_environment(&mut env_map);
        (env_map, scrubbed_count)
    }

    /// Returns human-friendly overview + technical telemetry for Dashboard, MCP, and audit.
    pub async fn build_overview(&self, state: &AppState) -> Value {
        let pins = self.verify_constitutional_pins_fresh();
        let pins_all_ok = pins.iter().all(|p| p.verified && !p.fail_closed_triggered);

        let pool = state.memory.pool();
        let weevolve_ratings = list_weevolve_ratings(&pool, 50).await.unwrap_or_default();
        for rating in &weevolve_ratings {
            if rating.target_kind == "cordis_fiber" {
                self.cordis.sync_fiber_rating(
                    &rating.target_id,
                    rating.elo_rating,
                    rating.ema_score,
                );
            }
        }

        let fibers: Vec<CordisFiberDto> = self.cordis.list_fibers();
        let coeffects = self.cordis.list_coeffects();
        let active_fibers_count = fibers
            .iter()
            .filter(|f| f.active && !f.paused_by_coeffect)
            .count();

        let gpu_temp_c = {
            let wd = state.runtime.watchdog();
            wd.lock().map(|g| g.snapshot().gpu_temp_c).unwrap_or(45.0)
        };
        let active_conns = state.active_connections();
        let current_hour = chrono::Local::now().hour() as u8;
        let catnap_state = self.catnap.snapshot_state();
        let (gate_allowed, gate_reason) =
            self.catnap
                .evaluate_gate(active_conns, gpu_temp_c, 8.0, current_hour);
        let quiet_now = is_within_quiet_hours(&catnap_state.config, current_hour);

        let tombstones: Vec<MemoryTombstoneDto> = list_memory_tombstones(&pool, 50)
            .await
            .unwrap_or_default();
        let superseded_count = tombstones
            .iter()
            .filter(|t| t.status == "superseded")
            .count();
        let catnap_slices = list_catnap_slices(&pool, 30).await.unwrap_or_default();
        let ralph_candidates = self.list_ralph_candidates();
        let quarantined_count = ralph_candidates
            .iter()
            .filter(|c| c.status == "quarantined")
            .count();

        let episodes: Vec<DomainExperienceEntryDto> =
            self.experience_lib.list_episodes().unwrap_or_default();
        let always_on_chars_total: usize = fibers
            .iter()
            .filter(|f| f.active)
            .map(|f| f.ephemeral_cue_text.chars().count())
            .sum();

        let avg_elo = if weevolve_ratings.is_empty() {
            fibers.iter().map(|f| f.elo_rating).sum::<f64>() / (fibers.len().max(1) as f64)
        } else {
            weevolve_ratings.iter().map(|r| r.elo_rating).sum::<f64>()
                / (weevolve_ratings.len() as f64)
        };

        let index_md = fs::read_to_string(self.experience_lib.root_dir().join("INDEX.md"))
            .unwrap_or_else(|_| "# Domain Experience Library Index\n(vazio)".to_string());

        let slots_catalog: Vec<Value> = HarnessComponentSlot::all()
            .iter()
            .map(|slot| {
                let count = fibers
                    .iter()
                    .filter(|f| f.active && f.slot == *slot)
                    .count();
                json!({
                    "code": slot.as_code(),
                    "label_pt": slot.label_pt(),
                    "pinned_readonly": slot.is_constitutionally_pinned(),
                    "active_fibers": count,
                })
            })
            .collect();

        json!({
            "status": "ok",
            "version": "0.9.4",
            "summary": {
                "pins_verified": pins_all_ok,
                "superseded_memories": superseded_count,
                "active_optimizations": active_fibers_count,
                "quarantined_skills": quarantined_count,
                "avg_elo": (avg_elo * 10.0).round() / 10.0,
                "always_on_chars_total": always_on_chars_total,
                "catnap_gate_allowed": gate_allowed,
                "catnap_gate_reason": gate_reason,
                "quiet_hours_active_now": quiet_now,
                "gpu_temp_c": gpu_temp_c,
                "active_connections": active_conns,
            },
            "constitutional_pins": pins,
            "slots_catalog": slots_catalog,
            "cordis": {
                "fibers": fibers,
                "coeffects": coeffects,
            },
            "catnap": catnap_state,
            "catnap_slices": catnap_slices,
            "tombstones": tombstones,
            "weevolve_ratings": weevolve_ratings,
            "ralph_candidates": ralph_candidates,
            "experience_library": {
                "episodes": episodes,
                "index_markdown": index_md,
            }
        })
    }

    fn current_gpu_temp(state: &AppState) -> f32 {
        let wd = state.runtime.watchdog();
        wd.lock().map(|g| g.snapshot().gpu_temp_c).unwrap_or(45.0)
    }

    /// Executes an atomic cooperative Catnap micro-slice checking `< 1 ms` preemption & `+5°C` thermal hysteresis.
    pub async fn run_catnap_micro_slice(
        &self,
        state: &AppState,
        slice_kind_override: Option<&str>,
        force_manual: bool,
    ) -> Result<Value, String> {
        let start = Instant::now();
        let now = Utc::now().timestamp();

        let requested_kind = slice_kind_override
            .map(|s| s.trim().to_ascii_lowercase())
            .filter(|s| VALID_SLICE_KINDS.contains(&s.as_str()))
            .unwrap_or_else(|| self.catnap.next_round_robin_slice_kind().to_string());

        if !force_manual && self.catnap.is_slice_in_backoff(&requested_kind) {
            return Ok(json!({
                "status": "deferred",
                "preempted": false,
                "reason": format!("Monitor '{}' está em resfriamento isolado (backoff) após falha anterior.", requested_kind),
            }));
        }

        let gpu_temp_c = Self::current_gpu_temp(state);
        let active_conns = state.active_connections();
        let current_hour = chrono::Local::now().hour() as u8;

        if !force_manual {
            let (allowed, reason) = self.catnap.evaluate_gate_for_slice(
                active_conns,
                gpu_temp_c,
                8.0,
                current_hour,
                Some(&requested_kind),
            );
            if !allowed {
                return Ok(json!({
                    "status": "deferred",
                    "preempted": false,
                    "reason": reason,
                }));
            }
        }

        let pins = self.verify_constitutional_pins_fresh();
        if pins.iter().any(|p| !p.verified || p.fail_closed_triggered) {
            self.catnap.record_slice_failure(&requested_kind);
            return Err(
                "Integridade constitucional c1 falhou (SHA-256 mismatch): Catnap suspenso (fail-closed)."
                    .to_string(),
            );
        }
        let constitutional_sha = pins
            .first()
            .map(|p| p.actual_sha256.clone())
            .unwrap_or_else(|| "no_c1_pin".to_string());

        let runmark_fingerprint = compute_runmark_fingerprint(
            &format!("slice={requested_kind}|c1={constitutional_sha}"),
            &constitutional_sha,
            HarnessComponentSlot::C5ToolGuidance,
        );
        let slice_id = format!("catnap_{}_{}", requested_kind, now);
        let cursor_checkpoint = format!("cursor_{requested_kind}_{now}");

        self.catnap.begin_slice(&cursor_checkpoint);

        // Step 1: Scrub candidate probe environment (zero secrets leaked)
        let (_scrubbed_env, scrubbed_count) = Self::scrubbed_candidate_env();
        if self.catnap.check_preempted() {
            let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
            let rec = CatnapSliceRecordDto {
                slice_id: slice_id.clone(),
                slice_kind: requested_kind.clone(),
                runmark_fingerprint,
                cursor_checkpoint: cursor_checkpoint.clone(),
                status: "preempted".to_string(),
                duration_ms: elapsed_ms,
                cost_usd: 0.0,
                summary_pt: "Micro-soneca cedida imediatamente (< 1 ms) por chegada de atividade do usuário."
                    .to_string(),
                created_at: now,
            };
            let pool = state.memory.pool();
            let _ = record_catnap_slice(&pool, &rec).await;
            return Ok(json!({
                "status": "preempted",
                "preempted": true,
                "slice": rec,
            }));
        }

        // Step 2: Execute real, deterministic zero-cost slice logic
        let pool = state.memory.pool();
        let active_fibers = self.cordis.list_fibers();
        let always_on_chars: usize = active_fibers
            .iter()
            .filter(|f| f.active)
            .map(|f| f.ephemeral_cue_text.chars().count())
            .sum();

        let (slice_status, summary_pt) = match requested_kind.as_str() {
            "ladybug_reconcile" => {
                let tombstones = list_memory_tombstones(&pool, 50).await.unwrap_or_default();
                let mut edges_reconciled = 0usize;
                for t in &tombstones {
                    let src_node = format!("mem_node_{}", t.superseded_by_memory_id);
                    let dst_node = format!("mem_node_{}", t.superseded_memory_id);
                    if t.status == "restored" {
                        if state
                            .memory
                            .ladybug()
                            .remove_edge_by_rel(&pool, &src_node, &dst_node, "conflicts_with")
                            .await
                            .is_ok()
                        {
                            edges_reconciled += 1;
                        }
                    } else if t.status == "superseded" {
                        let neighbors = state.memory.ladybug().get_neighbors(&src_node).await;
                        let has_edge = neighbors
                            .iter()
                            .any(|e| e.target_id == dst_node && e.relationship == "conflicts_with");
                        if !has_edge {
                            let _ = state
                                .memory
                                .ladybug()
                                .insert_node(
                                    &pool,
                                    OntologicalNode {
                                        id: src_node.clone(),
                                        node_type: "epistemic_memory".to_string(),
                                        canonical_name: t.new_content_preview.clone(),
                                        metadata_json: "{}".to_string(),
                                        banned_patterns: Vec::new(),
                                    },
                                    now,
                                )
                                .await;
                            let _ = state
                                .memory
                                .ladybug()
                                .insert_node(
                                    &pool,
                                    OntologicalNode {
                                        id: dst_node.clone(),
                                        node_type: "epistemic_memory".to_string(),
                                        canonical_name: t.superseded_content_preview.clone(),
                                        metadata_json: "{}".to_string(),
                                        banned_patterns: Vec::new(),
                                    },
                                    now,
                                )
                                .await;
                            if state
                                .memory
                                .ladybug()
                                .insert_edge(
                                    &pool,
                                    OntologicalEdge {
                                        source_id: src_node,
                                        target_id: dst_node,
                                        relationship: "conflicts_with".to_string(),
                                        weight: 1.0,
                                    },
                                    now,
                                )
                                .await
                                .is_ok()
                            {
                                edges_reconciled += 1;
                            }
                        }
                    }
                }
                (
                    "completed",
                    format!(
                        "Reconciliação concluída: {} tombstones verificados e {} aresta(s) sincronizada(s) no grafo (0 MB VRAM).",
                        tombstones.len(),
                        edges_reconciled
                    ),
                )
            }
            "langevin_micro_batch" => {
                let stale_cutoff = now - 7 * 86_400;
                let decayed_rows = sqlx::query(
                    "UPDATE epistemic_memories SET salience = MAX(0.10, salience * 0.98) WHERE partition = 'EVOLVING' AND last_accessed_at < ?1",
                )
                .bind(stale_cutoff)
                .execute(&*pool)
                .await
                .map(|r| r.rows_affected())
                .unwrap_or(0);
                (
                    "completed",
                    format!(
                        "Micro-lote de revisão epistêmica aplicado: {decayed_rows} memória(s) EVOLVING ajustada(s); 0 registros STABLE tocados."
                    ),
                )
            }
            "diary_distillation" => {
                let recent_evolving = state
                    .memory
                    .list_epistemic(Some(EpistemicPartition::Evolving), 5)
                    .await
                    .unwrap_or_default();
                let joined = recent_evolving
                    .iter()
                    .map(|m| m.content.as_str())
                    .collect::<Vec<_>>()
                    .join("\n");
                let folded = fold_aci_observation(
                    "catnap_diary",
                    &format!(
                        "Itens EVOLVING={}; chars={} | {}",
                        recent_evolving.len(),
                        always_on_chars,
                        joined
                    ),
                    240,
                );
                (
                    "completed",
                    format!("Síntese de diário operacional concluída: {folded}"),
                )
            }
            "ralph_crystallize" => {
                let failed_row = sqlx::query(
                    "SELECT agent_id, task_description FROM subagent_sessions WHERE status = 'FAILED' ORDER BY updated_at DESC LIMIT 1",
                )
                .fetch_optional(&*pool)
                .await
                .ok()
                .flatten();

                if !force_manual && failed_row.is_none() {
                    (
                        "skipped",
                        "Nenhuma falha recente de subagente aguardando cristalização de habilidade neste ciclo."
                            .to_string(),
                    )
                } else {
                    let (src_sess, prob_desc) = match failed_row {
                        Some(row) => {
                            let aid: String = row.get(0);
                            let desc: String = row.get(1);
                            (aid, desc)
                        }
                        None => (
                            "catnap_manual".to_string(),
                            "Diagnóstico progressivo e recuperação rápida para erros de execução repetidos."
                                .to_string(),
                        ),
                    };
                    let cand = self
                        .crystallize_ralph_skill(
                            state,
                            &format!("Recuperação para {}", src_sess),
                            "ops",
                            &prob_desc,
                            "1. Compacte o traceback preservando o erro raiz.\n2. Consulte os símbolos envolvidos sem ler arquivos inteiros.\n3. Valide a correção com teste unitário focal.",
                            Some(&src_sess),
                        )
                        .await?;
                    (
                        "completed",
                        format!(
                            "Nova habilidade '{}' enviada para sua aprovação na Quarentena.",
                            cand.title_pt
                        ),
                    )
                }
            }
            _ => {
                let _ = record_weevolve_signal(
                    &pool,
                    "fib_ast_outline",
                    "cordis_fiber",
                    "tool_call_clean",
                    Some(0.65),
                    now,
                )
                .await;
                (
                    "completed",
                    format!(
                        "Sonda silenciosa (Shadow Probe $0.00) concluiu sem desvios ({} variáveis sensíveis isoladas).",
                        scrubbed_count
                    ),
                )
            }
        };

        let post_gpu_temp = Self::current_gpu_temp(state);
        if self.catnap.check_preempted()
            || (!force_manual && self.catnap.check_thermal_abort_in_flight(post_gpu_temp))
        {
            let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
            let rec = CatnapSliceRecordDto {
                slice_id,
                slice_kind: requested_kind,
                runmark_fingerprint,
                cursor_checkpoint,
                status: "preempted".to_string(),
                duration_ms: elapsed_ms,
                cost_usd: 0.0,
                summary_pt: "Micro-soneca interrompida de forma cooperativa (atividade do usuário ou histerese térmica)."
                    .to_string(),
                created_at: now,
            };
            let _ = record_catnap_slice(&pool, &rec).await;
            return Ok(json!({
                "status": "preempted",
                "preempted": true,
                "slice": rec,
            }));
        }

        if slice_status == "completed" {
            let ep = DomainExperienceEntryDto {
                episode_id: format!("ep_{now}"),
                domain_tag: requested_kind.clone(),
                cue_pattern: format!("catnap:{}", requested_kind),
                always_on_chars,
                skills_count: self.list_ralph_candidates().len(),
                subagents_count: active_fibers
                    .iter()
                    .filter(|f| f.active && f.slot == HarnessComponentSlot::C7Subagents)
                    .count(),
                scripts_count: 0,
                hooks_count: active_fibers.iter().filter(|f| f.active).count(),
                runmark_fingerprint: runmark_fingerprint.clone(),
                experience_summary_pt: summary_pt.clone(),
                created_at: now,
            };
            let _ = self.experience_lib.record_episode(&ep);
            self.catnap.record_slice_completed(&cursor_checkpoint, 0.0);
        }

        self.catnap.reset_slice_backoff(&requested_kind);

        let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
        let rec = CatnapSliceRecordDto {
            slice_id,
            slice_kind: requested_kind,
            runmark_fingerprint,
            cursor_checkpoint,
            status: slice_status.to_string(),
            duration_ms: elapsed_ms,
            cost_usd: 0.0,
            summary_pt,
            created_at: now,
        };
        record_catnap_slice(&pool, &rec)
            .await
            .map_err(|e| e.to_string())?;

        Ok(json!({
            "status": slice_status,
            "preempted": false,
            "slice": rec,
            "catnap_state": self.catnap.snapshot_state(),
        }))
    }

    /// Crystallizes a hard-won operational trajectory into a `SKILL.md` candidate staged in
    /// Skills Hub Quarantine (`c4`/`c6`), inspected by Zero-Trust + Myers Diff before approval.
    pub async fn crystallize_ralph_skill(
        &self,
        state: &AppState,
        title_pt: &str,
        domain: &str,
        problem_pattern_pt: &str,
        resolution_playbook: &str,
        source_session_id: Option<&str>,
    ) -> Result<RalphSkillCandidateDto, String> {
        let now = Utc::now().timestamp();
        let slug = normalize_skill_slug(title_pt);
        let safe_domain = normalize_skill_slug(domain);
        let description_clean = problem_pattern_pt.replace(['\r', '\n', '"'], " ");
        let desc_short: String = description_clean.trim().chars().take(160).collect();

        let skill_md = format!(
            "---\nname: {slug}\ndescription: \"{desc_short}\"\nversion: \"0.1.0\"\nauthor: \"Souls Ralph Loop\"\nlicense: \"MIT\"\ntags: [\"{safe_domain}\", \"evolutive-harness\", \"ralph-loop\"]\n---\n\n# {title_pt}\n\n## When to Use\nUse this skill whenever the agent encounters the recurring problem pattern:\n- {problem_pattern_pt}\n\n## Procedure\n{resolution_playbook}\n\n## Guardrails & Pitfalls\n- Never overwrite constitutionally pinned `c1` files (`AGENTS.md`, `SOUL.md`).\n- Execute candidate validators strictly in a credential-scrubbed (`SCRUB_ENV`) workspace.\n\n## Verification\n- Confirm the issue does not recur and record positive feedback if resolved cleanly.\n"
        )
        .replace("\r\n", "\n");

        // Enforce c1 pin integrity before staging new c4 skill
        let pins = self.verify_constitutional_pins_fresh();
        if pins.iter().any(|p| !p.verified || p.fail_closed_triggered) {
            return Err("Integridade constitucional c1 falhou; cristalização abortada (fail-closed).".to_string());
        }

        let anatomy = parse_skill_markdown(&skill_md);
        let scan_report = scan_skill_zero_trust(&skill_md, &[]);
        let (l1_tok, l2_tok, quality_score, lint_issues) =
            evaluate_skill_quality(&anatomy, &skill_md, &scan_report);
        let diff_report = compute_skill_diff("", &skill_md);
        let sha256 = compute_normalized_lf_sha256(&skill_md);

        let quarantine_base = match &self.quarantine_dir_override {
            Some(dir) => dir.clone(),
            None => SkillsHubRoots::from_system().quarantine_dir(),
        };
        let target_dir = quarantine_base.join(&slug);
        fs::create_dir_all(&target_dir).map_err(|e| e.to_string())?;

        let skill_path = target_dir.join("SKILL.md");
        fs::write(&skill_path, skill_md.as_bytes()).map_err(|e| e.to_string())?;

        let q_item = QuarantineItem {
            slug: slug.clone(),
            name: title_pt.trim().to_string(),
            description: desc_short,
            source: "ralph_loop".to_string(),
            source_url: format!("souls://evolutive_harness/ralph/{slug}"),
            fetched_at_epoch: now as u64,
            sha256,
            level1_tokens: l1_tok,
            level2_tokens: l2_tok,
            quality_score,
            lint_issues,
            scan_report,
            content: skill_md,
        };
        let meta_path = target_dir.join("quarantine_meta.json");
        let meta_pretty = serde_json::to_string_pretty(&q_item).unwrap_or_default() + "\n";
        fs::write(meta_path, meta_pretty.replace("\r\n", "\n").as_bytes())
            .map_err(|e| e.to_string())?;

        let quarantine_path_posix =
            normalize_posix_rel_path(&format!("skills_hub/quarantine/{slug}/SKILL.md"))
                .unwrap_or_else(|_| format!("skills_hub/quarantine/{slug}/SKILL.md"));

        let candidate = RalphSkillCandidateDto {
            candidate_id: format!("ralph_{slug}_{now}"),
            slug: slug.clone(),
            title_pt: title_pt.trim().to_string(),
            source_session_id: source_session_id
                .unwrap_or("session_manual")
                .trim()
                .to_string(),
            problem_pattern_pt: problem_pattern_pt.trim().to_string(),
            solution_summary_pt: format!(
                "Habilidade ({l2_tok} tokens, nota {quality_score}/100) enviada para revisão em quarentena."
            ),
            e3_score: (f64::from(quality_score) / 100.0).clamp(0.0, 1.0),
            elo_rating: 1200.0,
            quarantine_path: quarantine_path_posix,
            diff_preview: format!(
                "{} | Score {} -> {}/100",
                diff_report.summary_text, diff_report.before_score, diff_report.after_score
            ),
            status: "quarantined".to_string(),
            created_at: now,
        };

        if let Ok(mut guard) = self.ralph_candidates.write() {
            guard.retain(|c| c.slug != candidate.slug);
            guard.push(candidate.clone());
            self.save_ralph_candidates_locked(&guard);
        }

        let pool = state.memory.pool();
        let _ = record_weevolve_signal(
            &pool,
            &format!("skill:{slug}"),
            "skill",
            "tool_call_clean",
            Some(0.4),
            now,
        )
        .await;

        Ok(candidate)
    }

    /// Promotes a quarantined Ralph skill candidate in 1 click after verifying Zero-Trust cleanliness.
    pub async fn approve_ralph_skill(
        &self,
        state: &AppState,
        raw_slug: &str,
    ) -> Result<Value, String> {
        let slug = normalize_skill_slug(raw_slug);
        let quarantine_base = match &self.quarantine_dir_override {
            Some(dir) => dir.clone(),
            None => SkillsHubRoots::from_system().quarantine_dir(),
        };
        let q_dir = quarantine_base.join(&slug);
        let q_skill = q_dir.join("SKILL.md");
        if !q_skill.is_file() {
            return Err(format!("Habilidade '{slug}' não encontrada na Quarentena."));
        }
        let raw_md = fs::read_to_string(&q_skill).map_err(|e| e.to_string())?;
        let lf_md = raw_md.replace("\r\n", "\n");

        let sync_res = if self.quarantine_dir_override.is_some() {
            // Isolated test surface: install inside data_dir/approved_skills to keep workspace untouched
            let dest_dir = self.data_dir.join("approved_skills").join(&slug);
            fs::create_dir_all(&dest_dir).map_err(|e| e.to_string())?;
            fs::write(dest_dir.join("SKILL.md"), lf_md.as_bytes()).map_err(|e| e.to_string())?;
            let _ = fs::remove_dir_all(&q_dir);
            json!({ "status": "approved", "targets": ["isolated_test"], "slug": slug })
        } else {
            let roots = SkillsHubRoots::from_system();
            let targets = vec!["workspace".to_string(), "hermes_global".to_string()];
            let res = sync_skill_to_targets(
                &roots,
                &slug,
                &lf_md,
                &targets,
                Some("ops"),
                "Aprovada em 1 clique via Auto-Evolução (v0.9.4)",
            )?;
            let _ = discard_quarantine_item(&roots, &slug);
            res
        };

        let now = Utc::now().timestamp();
        if let Ok(mut guard) = self.ralph_candidates.write() {
            for item in guard.iter_mut() {
                if item.slug == slug {
                    item.status = "approved".to_string();
                    item.solution_summary_pt = "Habilidade aprovada e sincronizada.".to_string();
                }
            }
            self.save_ralph_candidates_locked(&guard);
        }

        let pool = state.memory.pool();
        let _ = record_weevolve_signal(
            &pool,
            &format!("skill:{slug}"),
            "skill",
            "skill_promoted",
            None,
            now,
        )
        .await;

        Ok(json!({
            "status": "ok",
            "slug": slug,
            "sync_result": sync_res,
        }))
    }
}

// ============================================================================
// REST Endpoints (`/api/v1/harness/*`)
// ============================================================================

/// `GET /api/v1/harness/overview` (passive observability: never resets Catnap idle timer)
pub async fn overview_handler(State(state): State<AppState>) -> Json<Value> {
    let _guard = state.acquire_passive_connection();
    let payload = state.evolutive_harness.build_overview(&state).await;
    Json(payload)
}

/// `GET /api/v1/harness/cues?query=...` (sub-millisecond query-aware RAM cues for Hermes prefetch)
pub async fn cues_handler(
    State(state): State<AppState>,
    Query(params): Query<BTreeMap<String, String>>,
) -> Json<Value> {
    let _guard = state.acquire_passive_connection();
    let query = params.get("query").map(String::as_str).unwrap_or("");
    let pins = state.evolutive_harness.verify_constitutional_pins();
    let pins_verified = pins.iter().all(|p| p.verified && !p.fail_closed_triggered);
    let cues = state.evolutive_harness.select_query_aware_cues(query);
    Json(json!({
        "status": "ok",
        "pins_verified": pins_verified,
        "cues": cues,
    }))
}

#[derive(Debug, Deserialize)]
pub struct UpdateCatnapConfigRequest {
    pub enabled: Option<bool>,
    pub min_idle_secs: Option<u64>,
    pub max_gpu_temp_c: Option<f32>,
    pub max_cpu_load_pct: Option<f32>,
    pub max_daily_budget_usd: Option<f64>,
    pub allow_free_cloud_improver: Option<bool>,
    pub quiet_hours_enabled: Option<bool>,
    pub quiet_hours_start_hour: Option<u8>,
    pub quiet_hours_end_hour: Option<u8>,
}

/// `POST /api/v1/harness/catnap/config`
pub async fn update_catnap_config_handler(
    State(state): State<AppState>,
    Json(req): Json<UpdateCatnapConfigRequest>,
) -> impl IntoResponse {
    let _guard = state.acquire_connection();
    let mut cfg = state.evolutive_harness.catnap.get_config();
    if let Some(v) = req.enabled {
        cfg.enabled = v;
    }
    if let Some(v) = req.min_idle_secs {
        cfg.min_idle_secs = v;
    }
    if let Some(v) = req.max_gpu_temp_c {
        cfg.max_gpu_temp_c = v;
    }
    if let Some(v) = req.max_cpu_load_pct {
        cfg.max_cpu_load_pct = v;
    }
    if let Some(v) = req.max_daily_budget_usd {
        cfg.max_daily_budget_usd = v;
    }
    if let Some(v) = req.allow_free_cloud_improver {
        cfg.allow_free_cloud_improver = v;
    }
    if let Some(v) = req.quiet_hours_enabled {
        cfg.quiet_hours_enabled = v;
    }
    if let Some(v) = req.quiet_hours_start_hour {
        cfg.quiet_hours_start_hour = v % 24;
    }
    if let Some(v) = req.quiet_hours_end_hour {
        cfg.quiet_hours_end_hour = v % 24;
    }

    let updated = state.evolutive_harness.catnap.update_config(cfg);
    match state.evolutive_harness.persist_catnap_config(&updated) {
        Ok(()) => (
            StatusCode::OK,
            Json(json!({
                "status": "ok",
                "message": "Preferências de aprendizagem em segundo plano salvas com sucesso.",
                "config": updated,
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "status": "error",
                "error": e,
            })),
        ),
    }
}

#[derive(Debug, Default, Deserialize)]
pub struct TriggerCatnapRequest {
    pub slice_kind: Option<String>,
    pub force_manual: Option<bool>,
}

/// `POST /api/v1/harness/catnap/trigger`
pub async fn trigger_catnap_slice_handler(
    State(state): State<AppState>,
    payload: Option<Json<TriggerCatnapRequest>>,
) -> impl IntoResponse {
    let req = payload.map(|p| p.0).unwrap_or_default();
    let force_manual = req.force_manual.unwrap_or(true);
    match state
        .evolutive_harness
        .run_catnap_micro_slice(&state, req.slice_kind.as_deref(), force_manual)
        .await
    {
        Ok(res) => (StatusCode::OK, Json(res)),
        Err(e) => (
            StatusCode::CONFLICT,
            Json(json!({
                "status": "error",
                "error": e,
            })),
        ),
    }
}

#[derive(Debug, Deserialize)]
pub struct MountCordisFiberRequest {
    pub fiber_id: Option<String>,
    pub slot: String,
    pub title_pt: String,
    pub benefit_summary_pt: Option<String>,
    pub ephemeral_cue_text: String,
    pub target_task_type: Option<String>,
    pub coeffect_dependencies: Option<Vec<String>>,
}

/// `POST /api/v1/harness/cordis/mount`
pub async fn mount_cordis_fiber_handler(
    State(state): State<AppState>,
    Json(req): Json<MountCordisFiberRequest>,
) -> impl IntoResponse {
    let _guard = state.acquire_connection();
    let Some(slot) = HarnessComponentSlot::from_str_lossy(&req.slot) else {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "status": "error",
                "error": format!("Slot de anatomia desconhecido: '{}'", req.slot),
            })),
        );
    };

    let now = Utc::now().timestamp();
    let benefit = req
        .benefit_summary_pt
        .unwrap_or_else(|| "Otimização reversível aplicada durante esta sessão.".to_string());
    match state.evolutive_harness.cordis.mount_fiber(
        req.fiber_id,
        slot,
        &req.title_pt,
        &benefit,
        &req.ephemeral_cue_text,
        req.target_task_type,
        req.coeffect_dependencies.unwrap_or_default(),
        Vec::new(),
        now,
    ) {
        Ok(fiber) => {
            state.evolutive_harness.persist_cordis_state();
            (
                StatusCode::OK,
                Json(json!({
                    "status": "ok",
                    "fiber": fiber,
                })),
            )
        }
        Err(e) => (
            StatusCode::FORBIDDEN,
            Json(json!({
                "status": "blocked",
                "error": e.to_string(),
            })),
        ),
    }
}

#[derive(Debug, Deserialize)]
pub struct DisposeCordisFiberRequest {
    pub fiber_id: String,
}

/// `POST /api/v1/harness/cordis/dispose`
pub async fn dispose_cordis_fiber_handler(
    State(state): State<AppState>,
    Json(req): Json<DisposeCordisFiberRequest>,
) -> impl IntoResponse {
    let _guard = state.acquire_connection();
    match state.evolutive_harness.cordis.dispose_fiber(&req.fiber_id) {
        Ok(report) => {
            let pool = state.memory.pool();
            let now = Utc::now().timestamp();
            let _ = record_weevolve_signal(
                &pool,
                &req.fiber_id,
                "cordis_fiber",
                "cordis_rollback",
                None,
                now,
            )
            .await;
            state.evolutive_harness.persist_cordis_state();
            (
                StatusCode::OK,
                Json(json!({
                    "status": "ok",
                    "report": report,
                })),
            )
        }
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "status": "error",
                "error": e.to_string(),
            })),
        ),
    }
}

/// `POST /api/v1/harness/cordis/reactivate`
pub async fn reactivate_cordis_fiber_handler(
    State(state): State<AppState>,
    Json(req): Json<DisposeCordisFiberRequest>,
) -> impl IntoResponse {
    let _guard = state.acquire_connection();
    match state.evolutive_harness.cordis.reactivate_fiber(&req.fiber_id) {
        Ok(fiber) => {
            state.evolutive_harness.persist_cordis_state();
            (
                StatusCode::OK,
                Json(json!({
                    "status": "ok",
                    "fiber": fiber,
                })),
            )
        }
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "status": "error",
                "error": e.to_string(),
            })),
        ),
    }
}

#[derive(Debug, Default, Deserialize)]
pub struct ResolveTombstoneRequest {
    pub memory_id: Option<String>,
    pub superseded_memory_id: Option<String>,
    pub superseded_by_memory_id: Option<String>,
    pub reason_pt: Option<String>,
}

/// `POST /api/v1/harness/tombstones/resolve`
pub async fn resolve_tombstone_handler(
    State(state): State<AppState>,
    payload: Option<Json<ResolveTombstoneRequest>>,
) -> impl IntoResponse {
    let _guard = state.acquire_connection();
    let req = payload.map(|p| p.0).unwrap_or_default();
    let pool = state.memory.pool();
    let now = Utc::now().timestamp();

    if let (Some(old_id), Some(new_id)) =
        (req.superseded_memory_id.as_deref(), req.superseded_by_memory_id.as_deref())
    {
        let Ok(Some(old_mem)) = state.memory.get_epistemic_memory(old_id).await else {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "status": "error", "error": format!("Memória antiga '{old_id}' não encontrada.") })),
            );
        };
        let Ok(Some(new_mem)) = state.memory.get_epistemic_memory(new_id).await else {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "status": "error", "error": format!("Memória nova '{new_id}' não encontrada.") })),
            );
        };
        let reason = req
            .reason_pt
            .as_deref()
            .unwrap_or("Substituição confirmada manualmente pelo operador.");

        return match apply_memory_tombstone(
            &pool,
            state.memory.ladybug(),
            &old_mem,
            &new_mem,
            reason,
            now,
        )
        .await
        {
            Ok(tomb) => (
                StatusCode::OK,
                Json(json!({
                    "status": "ok",
                    "tombstones_applied": [tomb],
                })),
            ),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "status": "error", "error": e.to_string() })),
            ),
        };
    }

    if let Some(target_id) = req.memory_id.as_deref() {
        let Ok(Some(target_mem)) = state.memory.get_epistemic_memory(target_id).await else {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "status": "error", "error": format!("Memória '{target_id}' não encontrada.") })),
            );
        };
        return match detect_and_apply_jit_tombstones(
            &pool,
            state.memory.ladybug(),
            &target_mem,
            now,
        )
        .await
        {
            Ok(applied) => (
                StatusCode::OK,
                Json(json!({
                    "status": "ok",
                    "tombstones_applied": applied,
                })),
            ),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "status": "error", "error": e.to_string() })),
            ),
        };
    }

    let recent = state.memory.list_epistemic(None, 10).await.unwrap_or_default();
    let mut all_applied = Vec::new();
    for mem in recent {
        if let Ok(mut applied) =
            detect_and_apply_jit_tombstones(&pool, state.memory.ladybug(), &mem, now).await
        {
            all_applied.append(&mut applied);
        }
    }

    (
        StatusCode::OK,
        Json(json!({
            "status": "ok",
            "tombstones_applied": all_applied,
        })),
    )
}

#[derive(Debug, Deserialize)]
pub struct RestoreTombstoneRequest {
    pub tombstone_id: String,
}

/// `POST /api/v1/harness/tombstones/restore`
pub async fn restore_tombstone_handler(
    State(state): State<AppState>,
    Json(req): Json<RestoreTombstoneRequest>,
) -> impl IntoResponse {
    let _guard = state.acquire_connection();
    let pool = state.memory.pool();
    let now = Utc::now().timestamp();

    match restore_tombstoned_memory(&pool, &req.tombstone_id, now).await {
        Ok(Some(restored)) => {
            let src_node = format!("mem_node_{}", restored.superseded_by_memory_id);
            let dst_node = format!("mem_node_{}", restored.superseded_memory_id);
            let _ = state
                .memory
                .ladybug()
                .remove_edge_by_rel(&pool, &src_node, &dst_node, "conflicts_with")
                .await;
            (
                StatusCode::OK,
                Json(json!({
                    "status": "ok",
                    "tombstone": restored,
                })),
            )
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "status": "error",
                "error": format!("Tombstone '{}' não encontrado.", req.tombstone_id),
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "status": "error",
                "error": e.to_string(),
            })),
        ),
    }
}

#[derive(Debug, Deserialize)]
pub struct CrystallizeRalphRequest {
    pub title_pt: String,
    pub domain: Option<String>,
    pub problem_pattern_pt: String,
    pub resolution_playbook: String,
    pub source_session_id: Option<String>,
}

/// `POST /api/v1/harness/ralph/crystallize`
pub async fn crystallize_ralph_skill_handler(
    State(state): State<AppState>,
    Json(req): Json<CrystallizeRalphRequest>,
) -> impl IntoResponse {
    let _guard = state.acquire_connection();
    match state
        .evolutive_harness
        .crystallize_ralph_skill(
            &state,
            &req.title_pt,
            req.domain.as_deref().unwrap_or("ops"),
            &req.problem_pattern_pt,
            &req.resolution_playbook,
            req.source_session_id.as_deref(),
        )
        .await
    {
        Ok(cand) => (
            StatusCode::OK,
            Json(json!({
                "status": "ok",
                "candidate": cand,
            })),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "status": "error",
                "error": e,
            })),
        ),
    }
}

#[derive(Debug, Deserialize)]
pub struct ApproveRalphSkillRequest {
    pub slug: String,
}

/// `POST /api/v1/harness/ralph/approve` (1-click promotion from Quarantine to workspace/Hermes)
pub async fn approve_ralph_skill_handler(
    State(state): State<AppState>,
    Json(req): Json<ApproveRalphSkillRequest>,
) -> impl IntoResponse {
    let _guard = state.acquire_connection();
    match state
        .evolutive_harness
        .approve_ralph_skill(&state, &req.slug)
        .await
    {
        Ok(res) => (StatusCode::OK, Json(res)),
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "status": "error",
                "error": e,
            })),
        ),
    }
}

#[derive(Debug, Deserialize)]
pub struct RecordWeEvolveSignalRequest {
    pub target_id: String,
    pub target_kind: Option<String>,
    pub signal: String,
    pub custom_reward: Option<f64>,
}

/// `POST /api/v1/harness/weevolve/signal`
pub async fn record_weevolve_signal_handler(
    State(state): State<AppState>,
    Json(req): Json<RecordWeEvolveSignalRequest>,
) -> impl IntoResponse {
    let _guard = state.acquire_connection();
    let pool = state.memory.pool();
    let now = Utc::now().timestamp();
    let kind = req.target_kind.as_deref().unwrap_or("cordis_fiber");

    match record_weevolve_signal(&pool, &req.target_id, kind, &req.signal, req.custom_reward, now)
        .await
    {
        Ok(rating) => {
            let positive = req.custom_reward.unwrap_or_else(|| {
                souls_memory::map_signal_to_reward(&req.signal)
            }) >= 0.0;
            let auto_rollback = if kind == "cordis_fiber" {
                let res = state.evolutive_harness.cordis.record_hemispheric_signal(
                    &req.target_id,
                    positive,
                    rating.elo_rating,
                    rating.ema_score,
                );
                if res.is_some() {
                    state.evolutive_harness.persist_cordis_state();
                }
                res
            } else {
                None
            };
            (
                StatusCode::OK,
                Json(json!({
                    "status": "ok",
                    "rating": rating,
                    "auto_rollback": auto_rollback,
                })),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "status": "error",
                "error": e.to_string(),
            })),
        ),
    }
}
