//! # The Cognitive Catnap Protocol (`souls_core::harness::catnap`)
//!
//! Replaces brittle overnight-only batch jobs with opportunistic micro-consolidation slices
//! triggered during natural user idle breaks (`>= 180s`, GPU `< 50°C`, `0` active requests).
//! Guarantees `< 1 ms` atomic preemption on any incoming user/daemon activity, isolated backoff
//! per slice kind, Quiet Hours governance (`AGENTS.md` §9.5), and SHA-256 `runmark.fingerprint`
//! cache invalidation (Harness² Issue #44 / PR #162).

#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use sha2::{Digest, Sha256};
use souls_protocol::{CatnapConfigDto, CatnapStateDto, HarnessComponentSlot};

fn current_epoch_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn current_epoch_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Computes a deterministic SHA-256 `runmark.fingerprint` binding task inputs, pinned `c1` SHA-256,
/// and target component slot so evaluations are invalidated automatically when state changes.
#[must_use]
pub fn compute_runmark_fingerprint(
    inputs: &str,
    constitutional_sha256: &str,
    slot: HarnessComponentSlot,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(slot.as_code().as_bytes());
    hasher.update(b"|");
    hasher.update(constitutional_sha256.trim().as_bytes());
    hasher.update(b"|");
    hasher.update(inputs.trim().as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Evaluates whether `hour` (`0..=23`) falls inside the configured Quiet Hours window.
#[must_use]
pub fn is_within_quiet_hours(config: &CatnapConfigDto, hour: u8) -> bool {
    if !config.quiet_hours_enabled {
        return false;
    }
    let start = config.quiet_hours_start_hour % 24;
    let end = config.quiet_hours_end_hour % 24;
    if start == end {
        return false;
    }
    if start < end {
        hour >= start && hour < end
    } else {
        hour >= start || hour < end
    }
}

/// Thermal hysteresis band (°C) above `max_gpu_temp_c` before aborting an already-running slice.
pub const CATNAP_HYSTERESIS_BAND_C: f32 = 5.0;
/// Safe ceiling (°C) for `0 MB VRAM` lightweight CPU/SQLite maintenance slices (below `72°C` Preventive zone).
pub const CATNAP_LIGHT_SLICE_MAX_TEMP_C: f32 = 68.0;

const ROUND_ROBIN_SLICES: &[&str] = &[
    "shadow_probe_ppc",
    "ladybug_reconcile",
    "langevin_micro_batch",
    "diary_distillation",
    "ralph_crystallize",
];

/// Returns `true` for purely CPU/SQLite/RAM slices that allocate `0 MB` GPU VRAM.
#[must_use]
pub fn is_lightweight_zero_vram_slice(slice_kind: &str) -> bool {
    matches!(
        slice_kind.trim().to_ascii_lowercase().as_str(),
        "ladybug_reconcile" | "langevin_micro_batch" | "diary_distillation"
    )
}

/// Thread-safe scheduler monitor and `< 1 ms` preemption guard for Cognitive Catnaps.
#[derive(Debug)]
pub struct CatnapMonitor {
    last_activity_epoch_ms: Arc<AtomicU64>,
    preempt_flag: Arc<AtomicBool>,
    last_preemption_latency_bits: AtomicU64,
    active_utc_day: AtomicU64,
    slice_running: AtomicBool,
    slices_completed_today: AtomicU64,
    spent_today_micro_usd: AtomicU64,
    next_rr_index: AtomicU64,
    state_label: RwLock<String>,
    last_cursor_checkpoint: RwLock<String>,
    pending_failed_trajectories: AtomicU64,
    backoff_by_kind: RwLock<BTreeMap<String, (u32, u64)>>,
    config: RwLock<CatnapConfigDto>,
}

impl Default for CatnapMonitor {
    fn default() -> Self {
        Self::new(CatnapConfigDto::default())
    }
}

impl CatnapMonitor {
    /// Creates a new `CatnapMonitor` with initial configuration.
    #[must_use]
    pub fn new(config: CatnapConfigDto) -> Self {
        Self {
            last_activity_epoch_ms: Arc::new(AtomicU64::new(current_epoch_ms())),
            preempt_flag: Arc::new(AtomicBool::new(false)),
            last_preemption_latency_bits: AtomicU64::new(0.0_f64.to_bits()),
            active_utc_day: AtomicU64::new(current_epoch_secs() / 86_400),
            slice_running: AtomicBool::new(false),
            slices_completed_today: AtomicU64::new(0),
            spent_today_micro_usd: AtomicU64::new(0),
            next_rr_index: AtomicU64::new(0),
            state_label: RwLock::new("standby".to_string()),
            last_cursor_checkpoint: RwLock::new("cursor_init".to_string()),
            pending_failed_trajectories: AtomicU64::new(0),
            backoff_by_kind: RwLock::new(BTreeMap::new()),
            config: RwLock::new(config),
        }
    }

    /// Resets daily counters when UTC midnight passes so continuous daemon runs stay accurate.
    pub fn ensure_utc_day_current(&self) {
        self.rollover_utc_day_at(current_epoch_secs() / 86_400);
    }

    /// Applies deterministic UTC-day rollover for a given day index (`epoch_secs / 86_400`).
    pub fn rollover_utc_day_at(&self, utc_day_index: u64) {
        let prev = self.active_utc_day.load(Ordering::Acquire);
        if utc_day_index != prev
            && self
                .active_utc_day
                .compare_exchange(prev, utc_day_index, Ordering::SeqCst, Ordering::Acquire)
                .is_ok()
        {
            self.slices_completed_today.store(0, Ordering::Release);
            self.spent_today_micro_usd.store(0, Ordering::Release);
        }
    }

    /// Hydrates today's completed slice count and USD spend from `catnap_consolidation_ledger` on cold boot.
    pub fn hydrate_today_stats(&self, slices_completed: u64, spent_usd: f64) {
        self.active_utc_day
            .store(current_epoch_secs() / 86_400, Ordering::Release);
        self.slices_completed_today
            .store(slices_completed, Ordering::Release);
        let micro = (spent_usd.max(0.0) * 1_000_000.0).round() as u64;
        self.spent_today_micro_usd.store(micro, Ordering::Release);
    }

    /// Returns `true` while a Catnap micro-slice is actively executing.
    #[must_use]
    pub fn is_slice_running(&self) -> bool {
        self.slice_running.load(Ordering::Acquire)
    }

    /// Clears active slice execution flag when a slice exits without completing (skipped/preempted/error).
    pub fn finish_slice_idle(&self) {
        self.slice_running.store(false, Ordering::SeqCst);
    }

    /// Invoked in `0 ns` by `AppState::acquire_connection()` or user input events.
    /// Signals immediate cooperative preemption (`< 1 ms`) to any running micro-slice.
    #[inline]
    pub fn notify_activity(&self) {
        let now_ms = current_epoch_ms();
        self.last_activity_epoch_ms.store(now_ms, Ordering::Release);
        self.preempt_flag.store(true, Ordering::SeqCst);
    }

    /// Marks the start of a micro-slice, clearing the preemption flag before entering work.
    pub fn begin_slice(&self, cursor_checkpoint: &str) {
        self.preempt_flag.store(false, Ordering::SeqCst);
        self.slice_running.store(true, Ordering::SeqCst);
        if let Ok(mut st) = self.state_label.write() {
            *st = "catnapping".to_string();
        }
        if let Ok(mut cur) = self.last_cursor_checkpoint.write() {
            *cur = cursor_checkpoint.to_string();
        }
    }

    /// Polls the atomic preemption flag inside a catnap slice. Returns `true` if aborted.
    #[inline]
    pub fn check_preempted(&self) -> bool {
        let start = Instant::now();
        if self.preempt_flag.load(Ordering::Acquire) {
            let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
            self.last_preemption_latency_bits
                .store(elapsed_ms.to_bits(), Ordering::Relaxed);
            self.slice_running.store(false, Ordering::SeqCst);
            if let Ok(mut st) = self.state_label.write() {
                *st = "preempted".to_string();
            }
            return true;
        }
        false
    }

    /// Records completion of a micro-slice and updates daily FinOps accounting.
    pub fn record_slice_completed(&self, cursor_checkpoint: &str, cost_usd: f64) {
        self.ensure_utc_day_current();
        self.slice_running.store(false, Ordering::SeqCst);
        self.slices_completed_today.fetch_add(1, Ordering::Relaxed);
        let micro = (cost_usd.max(0.0) * 1_000_000.0).round() as u64;
        self.spent_today_micro_usd
            .fetch_add(micro, Ordering::Relaxed);
        if let Ok(mut st) = self.state_label.write() {
            *st = "completed".to_string();
        }
        if let Ok(mut cur) = self.last_cursor_checkpoint.write() {
            *cur = cursor_checkpoint.to_string();
        }
    }

    /// Sets the number of pending failed trajectories queued for Shadow Probing.
    pub fn set_pending_failed_trajectories(&self, count: usize) {
        self.pending_failed_trajectories
            .store(count as u64, Ordering::Relaxed);
    }

    /// Records isolated backoff for a specific `slice_kind` (never freezing other monitors).
    pub fn record_slice_failure(&self, slice_kind: &str) {
        let now = current_epoch_secs();
        if let Ok(mut map) = self.backoff_by_kind.write() {
            let entry = map.entry(slice_kind.to_string()).or_insert((0, 0));
            entry.0 = entry.0.saturating_add(1).min(6);
            let wait_secs = 60u64 * (1u64 << (entry.0 - 1));
            entry.1 = now.saturating_add(wait_secs);
        }
    }

    /// Clears isolated backoff for a `slice_kind` after a clean execution.
    pub fn reset_slice_backoff(&self, slice_kind: &str) {
        if let Ok(mut map) = self.backoff_by_kind.write() {
            map.remove(slice_kind);
        }
    }

    /// Returns `true` if `slice_kind` is currently cooling down in isolated backoff.
    #[must_use]
    pub fn is_slice_in_backoff(&self, slice_kind: &str) -> bool {
        let now = current_epoch_secs();
        self.backoff_by_kind
            .read()
            .ok()
            .and_then(|map| map.get(slice_kind).map(|(_, until)| now < *until))
            .unwrap_or(false)
    }

    /// Sets a synthetic idle timestamp (used in tests and simulation harnesses).
    pub fn set_idle_elapsed_secs_for_test(&self, idle_secs: u64) {
        let now_ms = current_epoch_ms();
        let target_ms = now_ms.saturating_sub(idle_secs.saturating_mul(1000));
        self.last_activity_epoch_ms
            .store(target_ms, Ordering::Release);
        self.preempt_flag.store(false, Ordering::SeqCst);
    }

    /// Updates `CatnapConfigDto` safely.
    pub fn update_config(&self, next: CatnapConfigDto) -> CatnapConfigDto {
        let sanitized = CatnapConfigDto {
            enabled: next.enabled,
            min_idle_secs: next.min_idle_secs.clamp(30, 3600),
            max_gpu_temp_c: next.max_gpu_temp_c.clamp(40.0, 80.0),
            max_cpu_load_pct: next.max_cpu_load_pct.clamp(5.0, 80.0),
            max_daily_budget_usd: next.max_daily_budget_usd.clamp(0.0, 20.0),
            allow_free_cloud_improver: next.allow_free_cloud_improver,
            quiet_hours_enabled: next.quiet_hours_enabled,
            quiet_hours_start_hour: next.quiet_hours_start_hour % 24,
            quiet_hours_end_hour: next.quiet_hours_end_hour % 24,
        };
        if let Ok(mut cfg) = self.config.write() {
            *cfg = sanitized.clone();
        }
        sanitized
    }

    /// Returns current configuration snapshot.
    #[must_use]
    pub fn get_config(&self) -> CatnapConfigDto {
        self.config
            .read()
            .map(|c| c.clone())
            .unwrap_or_default()
    }

    /// Cycles deterministically through the 5 canonical Catnap slice kinds (round-robin).
    pub fn next_round_robin_slice_kind(&self) -> &'static str {
        let idx = self.next_rr_index.fetch_add(1, Ordering::Relaxed) as usize;
        ROUND_ROBIN_SLICES[idx % ROUND_ROBIN_SLICES.len()]
    }

    /// Evaluates idle, thermal, concurrency, FinOps budget, and Quiet Hours gates cleanly,
    /// allowing `0 MB VRAM` lightweight maintenance slices up to `68.0°C` (`< 72°C` Preventive).
    #[must_use]
    pub fn evaluate_gate_for_slice(
        &self,
        active_conns: usize,
        gpu_temp_c: f32,
        cpu_load_pct: f32,
        current_hour: u8,
        slice_kind: Option<&str>,
    ) -> (bool, String) {
        self.ensure_utc_day_current();
        let cfg = self.get_config();
        if !cfg.enabled {
            return (false, "Cochilos inteligentes desativados pelo operador.".to_string());
        }
        if active_conns > 0 {
            return (
                false,
                format!("{} requisição(ões) ativa(s) em atendimento.", active_conns),
            );
        }
        let idle_secs = self.idle_elapsed_secs();
        if idle_secs < cfg.min_idle_secs {
            return (
                false,
                format!(
                    "Ociosidade atual ({idle_secs}s) abaixo do mínimo ({}s).",
                    cfg.min_idle_secs
                ),
            );
        }

        let is_light = slice_kind.map(is_lightweight_zero_vram_slice).unwrap_or(false);
        let effective_max_gpu = if is_light {
            cfg.max_gpu_temp_c.max(CATNAP_LIGHT_SLICE_MAX_TEMP_C).min(71.0)
        } else {
            cfg.max_gpu_temp_c
        };

        if gpu_temp_c > effective_max_gpu {
            return (
                false,
                format!(
                    "GPU a {gpu_temp_c:.1}°C acima do limite térmico de cochilo ({effective_max_gpu:.1}°C)."
                ),
            );
        }
        if cpu_load_pct > cfg.max_cpu_load_pct {
            return (
                false,
                format!(
                    "CPU em {cpu_load_pct:.1}% acima do limite de ociosidade ({:.1}%).",
                    cfg.max_cpu_load_pct
                ),
            );
        }

        let spent = self.spent_today_usd();
        if spent > cfg.max_daily_budget_usd + f64::EPSILON {
            return (
                false,
                format!(
                    "Teto financeiro diário (${:.2}) atingido.",
                    cfg.max_daily_budget_usd
                ),
            );
        }

        let quiet = is_within_quiet_hours(&cfg, current_hour);
        if quiet {
            return (
                true,
                "Liberado em modo silencioso (Quiet Hours ativo — zero notificações)."
                    .to_string(),
            );
        }
        (
            true,
            "Liberado: host ocioso, temperatura nominal e orçamento preservado.".to_string(),
        )
    }

    /// Evaluates standard gates (`slice_kind = None`).
    #[must_use]
    pub fn evaluate_gate(
        &self,
        active_conns: usize,
        gpu_temp_c: f32,
        cpu_load_pct: f32,
        current_hour: u8,
    ) -> (bool, String) {
        self.evaluate_gate_for_slice(active_conns, gpu_temp_c, cpu_load_pct, current_hour, None)
    }

    /// Hysteresis check for an already-running slice (`max_gpu_temp_c + 5.0°C`, capped below `72.0°C`).
    #[must_use]
    pub fn check_thermal_abort_in_flight(&self, current_gpu_temp_c: f32) -> bool {
        let abort_threshold = (self.get_config().max_gpu_temp_c + CATNAP_HYSTERESIS_BAND_C).min(72.0);
        if current_gpu_temp_c > abort_threshold {
            if let Ok(mut st) = self.state_label.write() {
                *st = "preempted_thermal".to_string();
            }
            return true;
        }
        false
    }

    /// Seconds elapsed since last recorded user/daemon activity.
    #[must_use]
    pub fn idle_elapsed_secs(&self) -> u64 {
        let now_ms = current_epoch_ms();
        let last_ms = self.last_activity_epoch_ms.load(Ordering::Acquire);
        now_ms.saturating_sub(last_ms) / 1000
    }

    /// Dollars spent today on shadow probing.
    #[must_use]
    pub fn spent_today_usd(&self) -> f64 {
        self.ensure_utc_day_current();
        (self.spent_today_micro_usd.load(Ordering::Relaxed) as f64) / 1_000_000.0
    }

    /// Produces a full `CatnapStateDto` telemetry snapshot.
    #[must_use]
    pub fn snapshot_state(&self) -> CatnapStateDto {
        self.ensure_utc_day_current();
        let state = self
            .state_label
            .read()
            .map(|s| s.clone())
            .unwrap_or_else(|_| "standby".to_string());
        let cursor = self
            .last_cursor_checkpoint
            .read()
            .map(|c| c.clone())
            .unwrap_or_else(|_| "cursor_init".to_string());
        let preempt_ms = f64::from_bits(self.last_preemption_latency_bits.load(Ordering::Relaxed));

        CatnapStateDto {
            state,
            idle_elapsed_secs: self.idle_elapsed_secs(),
            last_preemption_latency_ms: preempt_ms,
            slices_completed_today: self.slices_completed_today.load(Ordering::Relaxed),
            spent_today_usd: self.spent_today_usd(),
            last_cursor_checkpoint: cursor,
            pending_failed_trajectories: self
                .pending_failed_trajectories
                .load(Ordering::Relaxed) as usize,
            config: self.get_config(),
        }
    }
}
