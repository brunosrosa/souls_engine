//! Contextual Multi-Armed ParetoBandit with Thompson Sampling and Lock-Free Thermodynamic Watchdog.

use rand_distr::{Beta, Distribution};
use sqlx::{Pool, Sqlite};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tracing::{debug, info, warn};

use souls_inference_runtime::calculate_reasoning_effort;
use souls_inference_runtime::nvml::{
    BARRIER_LAMBDA, BARRIER_MU, OperationalZone, RTX_2060M_VRAM_TOTAL_MB,
    THERMAL_CRITICAL_C, THERMAL_NOMINAL_MAX_C, VRAM_FREE_CRITICAL_MB, VRAM_FREE_NOMINAL_MIN_MB,
};
use souls_memory::SoulsMemoryStore;
use souls_protocol::DynamicReasoningRecommendation;

use crate::context::TaskContext;
use crate::error::RouterError;
use crate::fallback::{CooldownManager, FallbackCandidate, FallbackChain, IncidentKind};
use crate::metrics::{calculate_e3_metric, TaskOutcome};
use crate::priors::{
    current_timestamp_sec, decay_bayesian_prior, ensure_router_tables, fetch_active_models,
    fetch_prior, update_bayesian_prior, update_model_telemetry,
};

/// Maximum context tokens tolerated by local accelerated Tier 1 before mandatory failover to Cloud.
pub const LOCAL_ACCELERATED_MAX_CONTEXT_TOKENS: u32 = 16_000;

/// Asynchronous outcome event sent via MPSC channel to worker flusher.
#[derive(Debug, Clone, PartialEq)]
pub struct OutcomeEvent {
    pub task_id: String,
    pub outcome: TaskOutcome,
}

enum OutcomeMsg {
    Event(OutcomeEvent),
    Flush(tokio::sync::oneshot::Sender<()>),
}

/// Weighting configuration for the multi-objective Pareto utility scalarization:
///
/// $$U(k \mid x_i) = w_q \cdot \hat{Q}_{k,c} - w_c \cdot C_k(x_i) - w_l \cdot \hat{L}_k(x_i) - \Phi$$
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParetoWeights {
    /// Quality weight ($w_q$). Default: 1.0
    pub w_quality: f64,
    /// Direct cost weight ($w_c$). Default: 10.0
    pub w_cost: f64,
    /// Latency weight per ms ($w_l$). Default: 0.001 (1 sec penalty = 1.0)
    pub w_latency: f64,
}

impl Default for ParetoWeights {
    fn default() -> Self {
        Self {
            w_quality: 1.0,
            w_cost: 10.0,
            w_latency: 0.001,
        }
    }
}

/// Final decision produced by the ParetoBandit decision engine.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RoutingDecision {
    /// Selected model identifier.
    pub model_id: String,
    /// Execution tier (e.g. "tier1", "tier0", "tier3").
    pub tier_name: String,
    /// Host provider classification ("LOCAL" or "CLOUD").
    pub provider_type: String,
    /// Scalarized utility score $U(k \mid x_i)$.
    pub expected_utility: f64,
    /// Thompson-sampled quality estimate $\hat{Q}_{k,c} \in [0.0, 1.0]$.
    pub sampled_quality: f64,
    /// Projected FinOps cost in USD ($C_k$).
    pub projected_cost_usd: f64,
    /// Projected latency in milliseconds ($\hat{L}_k$).
    pub projected_latency_ms: f64,
    /// Evaluated thermodynamic barrier penalty $\Phi$.
    pub thermodynamic_barrier_phi: f32,
    /// Audit rationale explaining why this model was chosen.
    pub rationale: String,
    /// Dynamic reasoning and thinking budget recommendation based on Tier 0.5 entropy and FinOps calibration.
    pub reasoning_recommendation: Option<DynamicReasoningRecommendation>,
}

/// Unpacks lock-free GPU telemetry from an atomic u64 state:
/// - Bits 0..19: Allocated VRAM (MB)
/// - Bits 20..29: GPU Temperature * 2.0 (0.5°C precision)
/// - Bits 30..31: Operational Zone (0 = Nominal, 1 = Preventive, 2 = Critical)
#[inline]
pub fn unpack_gpu_telemetry(packed: u64) -> (u32, f32, OperationalZone) {
    let vram_allocated_mb = (packed & 0xFFFFF) as u32;
    let temp_raw = ((packed >> 20) & 0x3FF) as f32;
    let gpu_temp_c = temp_raw / 2.0;
    let zone_val = (packed >> 30) & 0x3;
    let zone = match zone_val {
        0 => OperationalZone::Nominal,
        1 => OperationalZone::Preventive,
        _ => OperationalZone::Critical,
    };
    (vram_allocated_mb, gpu_temp_c, zone)
}

/// Packs telemetry into an atomic u64 for testing and synthetic state generation.
#[inline]
pub fn pack_gpu_telemetry(vram_mb: u32, gpu_temp_c: f32, zone: OperationalZone) -> u64 {
    let vram_part = (vram_mb as u64) & 0xFFFFF;
    let temp_part = (((gpu_temp_c * 2.0).clamp(0.0, 1023.0)) as u64) & 0x3FF;
    let zone_part = match zone {
        OperationalZone::Nominal => 0u64,
        OperationalZone::Preventive => 1u64,
        OperationalZone::Critical => 2u64,
    };
    vram_part | (temp_part << 20) | (zone_part << 30)
}

/// Main Contextual ParetoBandit Router governing zero-context subagent and auxiliary requests.
#[derive(Clone)]
pub struct ParetoBanditRouter {
    pool: Pool<Sqlite>,
    gpu_telemetry_atomic: Arc<AtomicU64>,
    weights: ParetoWeights,
    outcome_tx: tokio::sync::mpsc::Sender<OutcomeMsg>,
    cooldown_manager: Arc<CooldownManager>,
}

impl ParetoBanditRouter {
    /// Creates a new ParetoBanditRouter with explicit components and spawns the background worker.
    pub fn new(
        pool: Pool<Sqlite>,
        gpu_telemetry_atomic: Arc<AtomicU64>,
        weights: ParetoWeights,
    ) -> Self {
        let (outcome_tx, mut outcome_rx) = tokio::sync::mpsc::channel::<OutcomeMsg>(256);
        let pool_worker = pool.clone();
        let cooldown_manager = Arc::new(CooldownManager::new());
        let cooldown_worker = Arc::clone(&cooldown_manager);

        tokio::spawn(async move {
            while let Some(msg) = outcome_rx.recv().await {
                match msg {
                    OutcomeMsg::Event(event) => {
                        let now_sec = current_timestamp_sec();

                        // If transient provider rate limit, apply dynamic quarantine without damaging Bayesian prior (beta)
                        if event.outcome.is_rate_limit {
                            let cd = event.outcome.retry_after_secs.unwrap_or(60);
                            cooldown_worker.record_incident(
                                &event.outcome.tier_name,
                                IncidentKind::RateLimited429,
                                Some(cd),
                            );
                            info!(
                                target: "souls_model_router::bandit",
                                "Arm '{}' dynamically quarantined for {}s due to rate limit; Bayesian priors preserved.",
                                event.outcome.tier_name,
                                cd
                            );
                            continue;
                        }

                        let reward = event.outcome.fractional_reward();
                        if event.outcome.success {
                            cooldown_worker.record_success(&event.outcome.tier_name);
                        }
                        if let Err(err) = update_bayesian_prior(
                            &pool_worker,
                            &event.outcome.tier_name,
                            &event.outcome.task_category,
                            reward,
                            now_sec,
                        )
                        .await
                        {
                            warn!("Asynchronous prior update failed: {}", err);
                        }
                        if let Err(err) = update_model_telemetry(
                            &pool_worker,
                            &event.outcome.tier_name,
                            event.outcome.latency_ms,
                            event.outcome.success,
                        )
                        .await
                        {
                            warn!("Asynchronous model telemetry update failed: {}", err);
                        }
                    }
                    OutcomeMsg::Flush(ack) => {
                        let _ = ack.send(());
                    }
                }
            }
        });

        Self {
            pool,
            gpu_telemetry_atomic,
            weights,
            outcome_tx,
            cooldown_manager,
        }
    }

    /// Initializes a router directly from a SoulsMemoryStore and atomic hardware watchdog handle.
    pub async fn from_memory_store(
        memory_store: &SoulsMemoryStore,
        gpu_telemetry_atomic: Arc<AtomicU64>,
    ) -> Result<Self, RouterError> {
        let pool = memory_store.pool();
        ensure_router_tables(&pool).await?;

        Ok(Self::new(
            (*pool).clone(),
            gpu_telemetry_atomic,
            ParetoWeights::default(),
        ))
    }

    /// Creates a test instance with synthetic in-memory SQLite and initial GPU telemetry.
    pub async fn new_for_testing(
        pool: Pool<Sqlite>,
        initial_temp_c: f32,
        initial_vram_allocated_mb: u32,
    ) -> Result<Self, RouterError> {
        ensure_router_tables(&pool).await?;

        let free = RTX_2060M_VRAM_TOTAL_MB.saturating_sub(initial_vram_allocated_mb);
        let zone = if free <= VRAM_FREE_CRITICAL_MB || initial_temp_c >= THERMAL_CRITICAL_C {
            OperationalZone::Critical
        } else if free <= VRAM_FREE_NOMINAL_MIN_MB || initial_temp_c >= THERMAL_NOMINAL_MAX_C {
            OperationalZone::Preventive
        } else {
            OperationalZone::Nominal
        };

        let packed = pack_gpu_telemetry(initial_vram_allocated_mb, initial_temp_c, zone);
        let atomic = Arc::new(AtomicU64::new(packed));

        Ok(Self::new(pool, atomic, ParetoWeights::default()))
    }

    /// Access the underlying connection pool.
    pub fn pool(&self) -> &Pool<Sqlite> {
        &self.pool
    }

    /// Access the lock-free shared atomic telemetry handle.
    pub fn gpu_telemetry_atomic(&self) -> Arc<AtomicU64> {
        Arc::clone(&self.gpu_telemetry_atomic)
    }

    /// Mutates the shared atomic telemetry directly (ideal for test simulations).
    pub fn set_gpu_telemetry(&self, temp_c: f32, vram_allocated_mb: u32, zone: OperationalZone) {
        let packed = pack_gpu_telemetry(vram_allocated_mb, temp_c, zone);
        self.gpu_telemetry_atomic.store(packed, Ordering::Release);
    }

    /// Access the thread-safe CooldownManager handle.
    pub fn cooldown_manager(&self) -> Arc<CooldownManager> {
        Arc::clone(&self.cooldown_manager)
    }

    /// Records a provider incident (HTTP 429, 500, timeout) quarantining the model arm.
    pub fn record_incident(
        &self,
        model_id: &str,
        kind: IncidentKind,
        custom_cooldown_secs: Option<u64>,
    ) {
        self.cooldown_manager
            .record_incident(model_id, kind, custom_cooldown_secs);
    }

    /// Records a provider incident by inspecting HTTP response headers to compute dynamic quarantine.
    pub fn record_incident_with_headers<I, K, V>(
        &self,
        model_id: &str,
        kind: IncidentKind,
        headers: I,
    ) -> u64
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        self.cooldown_manager.record_incident_with_headers(
            model_id,
            kind,
            headers,
            current_timestamp_sec(),
        )
    }

    /// Explicitly clears quarantine state for a recovered model arm.
    pub fn record_success(&self, model_id: &str) {
        self.cooldown_manager.record_success(model_id);
    }

    /// Re-evaluates and routes a task context to the optimal model arm.
    pub async fn route_task(&self, task: &TaskContext) -> Result<RoutingDecision, RouterError> {
        // 1. Lock-free O(1) telemetry acquisition
        let packed = self.gpu_telemetry_atomic.load(Ordering::Acquire);
        let (vram_allocated_mb, gpu_temp_c, zone) = unpack_gpu_telemetry(packed);
        let vram_free_mb = RTX_2060M_VRAM_TOTAL_MB.saturating_sub(vram_allocated_mb);

        // 2. Fetch all active candidate models from SQLite
        let active_models = fetch_active_models(&self.pool).await?;
        if active_models.is_empty() {
            return Err(RouterError::NoActiveCandidates(
                "model_registry contains zero active models".to_string(),
            ));
        }

        let mut best_decision: Option<RoutingDecision> = None;
        let mut highest_utility = f64::NEG_INFINITY;
        let mut rng = rand::thread_rng();
        let mut excluded_by_context: usize = 0;
        let mut excluded_by_thermal: usize = 0;
        let mut max_active_context_window: u32 = 0;

        // 1.5 Compute dynamic reasoning effort recommendation
        let entropy = task.entropy_score.unwrap_or(0.5);
        let is_retry = task.is_retry.unwrap_or(false);
        let reasoning_rec = calculate_reasoning_effort(entropy, &task.category, is_retry);

        let total_tokens = task.total_projected_tokens();

        for model in active_models {
            // Check authentication credentials for cloud models
            if !model.is_auth_available() {
                debug!(
                    model_id = %model.model_id,
                    "Model excluded: no active authentication credentials found in environment"
                );
                continue;
            }

            let effective_ctx = if model.is_gpu_accelerated() {
                model.max_context_window.min(LOCAL_ACCELERATED_MAX_CONTEXT_TOKENS)
            } else {
                model.max_context_window
            };
            max_active_context_window = max_active_context_window.max(effective_ctx);

            // Check max context window of model
            if task.is_context_exceeded(model.max_context_window) {
                excluded_by_context += 1;
                debug!(
                    model_id = %model.model_id,
                    max_ctx = model.max_context_window,
                    tokens = total_tokens,
                    "Model excluded: task exceeds model context window"
                );
                continue;
            }

            // 3. Compute thermodynamic barrier \Phi(T_GPU, V_free, k)
            let barrier_phi: f32 = if model.is_gpu_accelerated() {
                // Rule 1: Preemption due to context overflow on RTX 2060m (> 16k tokens)
                if total_tokens > LOCAL_ACCELERATED_MAX_CONTEXT_TOKENS {
                    excluded_by_context += 1;
                    warn!(
                        total_tokens,
                        limit = LOCAL_ACCELERATED_MAX_CONTEXT_TOKENS,
                        "Local GPU Tier banned due to context token overflow (>16k tokens)"
                    );
                    f32::INFINITY
                // Rule 2: Critical thermal or VRAM exhaustion
                } else if zone == OperationalZone::Critical
                    || vram_free_mb <= VRAM_FREE_CRITICAL_MB
                    || gpu_temp_c >= THERMAL_CRITICAL_C
                {
                    excluded_by_thermal += 1;
                    warn!(
                        gpu_temp_c,
                        vram_free_mb,
                        "Local GPU Tier banned: RTX 2060m in Critical Zone"
                    );
                    f32::INFINITY
                // Rule 3: Preventive barrier penalty
                } else if zone == OperationalZone::Preventive
                    || vram_free_mb <= VRAM_FREE_NOMINAL_MIN_MB
                    || gpu_temp_c >= THERMAL_NOMINAL_MAX_C
                {
                    let vram_penalty = (1_400.0 - vram_free_mb as f32).max(0.0) / 600.0;
                    let temp_penalty = (gpu_temp_c - 75.0).max(0.0) / 7.0;
                    BARRIER_LAMBDA * vram_penalty + BARRIER_MU * temp_penalty
                } else {
                    0.0
                }
            } else {
                // Tier 0 (CPU ONNX), Tier 0.5 (CPU Probing), or Cloud models
                0.0
            };

            // 4. Thompson Sampling from Beta(\alpha, \beta)
            let prior = fetch_prior(&self.pool, &model.model_id, &task.category).await?;
            let sampled_quality = match Beta::new(prior.alpha, prior.beta) {
                Ok(dist) => dist.sample(&mut rng),
                Err(e) => {
                    warn!("Invalid Beta parameters ({}, {}): {}", prior.alpha, prior.beta, e);
                    (prior.alpha / (prior.alpha + prior.beta)).clamp(0.0, 1.0)
                }
            };

            // 5. Projected cost calculation C_k(x_i) taking caching into account
            let cached_in = task.estimated_cached_input_tokens.unwrap_or(0);
            let projected_cost_usd = model.calculate_projected_cost(
                task.estimated_input_tokens,
                cached_in,
                task.estimated_output_tokens,
            );

            if let Some(budget_cap) = task.max_budget_usd {
                if projected_cost_usd > budget_cap {
                    debug!(
                        model_id = %model.model_id,
                        projected_cost = projected_cost_usd,
                        budget_cap,
                        "Model excluded: projected cost exceeds task budget cap"
                    );
                    continue;
                }
            }

            // 6. Projected latency calculation \hat{L}_k(x_i) with provider scaling factor
            let provider = model.cloud_provider();
            let latency_factor = provider.latency_scaling_factor();
            let projected_latency_ms = (model.ema_latency_ms as f64)
                * (total_tokens as f64 / 1000.0).max(0.2)
                * latency_factor;

            // 6.5 Circuit breaker quarantine check
            let (in_cooldown, _remaining_cd) = self.cooldown_manager.is_quarantined(&model.model_id);
            let cooldown_penalty = if in_cooldown { 10_000.0 } else { 0.0 };

            // 7. Scalarized utility calculation U(k | x_i)
            let utility = if barrier_phi.is_infinite() {
                f64::NEG_INFINITY
            } else {
                (self.weights.w_quality * sampled_quality)
                    - (self.weights.w_cost * projected_cost_usd)
                    - (self.weights.w_latency * projected_latency_ms)
                    - (barrier_phi as f64)
                    - cooldown_penalty
            };

            let rationale = format!(
                "Arm {}: Q_hat={:.3}, Cost=${:.5}, Lat={:.1}ms, Phi={:.2}, CD_pen={:.1} => U={:.3}",
                model.model_id, sampled_quality, projected_cost_usd, projected_latency_ms, barrier_phi, cooldown_penalty, utility
            );

            if utility > highest_utility {
                highest_utility = utility;
                best_decision = Some(RoutingDecision {
                    model_id: model.model_id.clone(),
                    tier_name: model.model_id.clone(),
                    provider_type: model.provider_type.clone(),
                    expected_utility: utility,
                    sampled_quality,
                    projected_cost_usd,
                    projected_latency_ms,
                    thermodynamic_barrier_phi: barrier_phi,
                    rationale,
                    reasoning_recommendation: Some(reasoning_rec.clone()),
                });
            }
        }

        match best_decision {
            Some(decision) if decision.expected_utility > f64::NEG_INFINITY => {
                info!(
                    task_id = %task.task_id,
                    selected_model = %decision.model_id,
                    utility = decision.expected_utility,
                    "Routing decision selected successfully"
                );
                Ok(decision)
            }
            _ if excluded_by_context > 0 && excluded_by_thermal == 0 => {
                Err(RouterError::ContextWindowExceeded(format!(
                    "estimated_tokens={total_tokens} excede o teto ativo de {max_active_context_window} tokens. Faça chunking/map-reduce abaixo de {max_active_context_window} tokens."
                )))
            }
            _ => Err(RouterError::NoActiveCandidates(
                "All model arms blocked by thermodynamic barrier or context limits".to_string(),
            )),
        }
    }

    /// Evaluates all active models and builds a Pareto-optimal contingency fallback chain in <150ms.
    pub async fn resolve_fallback_chain(
        &self,
        task: &TaskContext,
        max_fallbacks: usize,
    ) -> Result<FallbackChain, RouterError> {
        let start = std::time::Instant::now();

        // 1. Lock-free O(1) telemetry acquisition
        let packed = self.gpu_telemetry_atomic.load(Ordering::Acquire);
        let (vram_allocated_mb, gpu_temp_c, zone) = unpack_gpu_telemetry(packed);
        let vram_free_mb = RTX_2060M_VRAM_TOTAL_MB.saturating_sub(vram_allocated_mb);

        // 2. Fetch all active candidate models from SQLite
        let active_models = fetch_active_models(&self.pool).await?;
        if active_models.is_empty() {
            return Err(RouterError::NoActiveCandidates(
                "model_registry contains zero active models".to_string(),
            ));
        }

        let total_tokens = task.total_projected_tokens();
        let cached_in = task.estimated_cached_input_tokens.unwrap_or(0);
        let mut candidates: Vec<FallbackCandidate> = Vec::new();

        for model in active_models {
            // Check authentication credentials for cloud models
            if !model.is_auth_available() {
                continue;
            }

            // Context window check
            if task.is_context_exceeded(model.max_context_window) {
                continue;
            }

            // FinOps cost check
            let projected_cost_usd = model.calculate_projected_cost(
                task.estimated_input_tokens,
                cached_in,
                task.estimated_output_tokens,
            );

            if let Some(budget_cap) = task.max_budget_usd {
                if projected_cost_usd > budget_cap {
                    continue;
                }
            }

            // Thermodynamic barrier for local models
            let barrier_phi: f32 = if model.is_gpu_accelerated() {
                if total_tokens > LOCAL_ACCELERATED_MAX_CONTEXT_TOKENS
                    || zone == OperationalZone::Critical
                    || vram_free_mb <= VRAM_FREE_CRITICAL_MB
                    || gpu_temp_c >= THERMAL_CRITICAL_C
                {
                    f32::INFINITY
                } else if zone == OperationalZone::Preventive
                    || vram_free_mb <= VRAM_FREE_NOMINAL_MIN_MB
                    || gpu_temp_c >= THERMAL_NOMINAL_MAX_C
                {
                    let vram_penalty = (1_400.0 - vram_free_mb as f32).max(0.0) / 600.0;
                    let temp_penalty = (gpu_temp_c - 75.0).max(0.0) / 7.0;
                    BARRIER_LAMBDA * vram_penalty + BARRIER_MU * temp_penalty
                } else {
                    0.0
                }
            } else {
                0.0
            };

            if barrier_phi.is_infinite() {
                continue;
            }

            // Expected quality E[Q] = alpha / (alpha + beta) for deterministic Pareto fallback ranking
            let prior = fetch_prior(&self.pool, &model.model_id, &task.category).await?;
            let expected_quality = (prior.alpha / (prior.alpha + prior.beta)).clamp(0.0, 1.0);

            let provider = model.cloud_provider();
            let latency_factor = provider.latency_scaling_factor();
            let projected_latency_ms = (model.ema_latency_ms as f64)
                * (total_tokens as f64 / 1000.0).max(0.2)
                * latency_factor;

            let (in_cooldown, remaining_cd) = self.cooldown_manager.is_quarantined(&model.model_id);
            let cooldown_penalty = if in_cooldown { 10_000.0 } else { 0.0 };

            let utility = (self.weights.w_quality * expected_quality)
                - (self.weights.w_cost * projected_cost_usd)
                - (self.weights.w_latency * projected_latency_ms)
                - (barrier_phi as f64)
                - cooldown_penalty;

            candidates.push(FallbackCandidate {
                model_id: model.model_id,
                provider,
                tier_name: model.provider_type,
                utility_score: utility,
                projected_cost_usd,
                projected_latency_ms,
                in_cooldown,
                cooldown_remaining_secs: remaining_cd,
            });
        }

        if candidates.is_empty() {
            return Err(RouterError::NoActiveCandidates(
                "No viable model arms available for fallback chain".to_string(),
            ));
        }

        // Sort by health first (healthy models ahead of quarantined models), then by utility descending
        candidates.sort_by(|a, b| {
            match (a.in_cooldown, b.in_cooldown) {
                (false, true) => std::cmp::Ordering::Less,
                (true, false) => std::cmp::Ordering::Greater,
                _ => b
                    .utility_score
                    .partial_cmp(&a.utility_score)
                    .unwrap_or(std::cmp::Ordering::Equal),
            }
        });

        let primary = candidates.remove(0);
        let fallbacks_count = candidates.len().min(max_fallbacks);
        let fallbacks = candidates.into_iter().take(fallbacks_count).collect();

        let resolution_ms = start.elapsed().as_secs_f64() * 1000.0;

        Ok(FallbackChain {
            task_id: task.task_id.clone(),
            primary,
            fallbacks,
            budget_ceiling_usd: task.max_budget_usd,
            resolution_ms,
        })
    }

    /// Asynchronously records task outcome feedback via non-blocking MPSC channel,
    /// completely decoupling caller threads from SQLite disk writes.
    pub async fn record_outcome(
        &self,
        task_id: &str,
        outcome: &TaskOutcome,
    ) -> Result<(), RouterError> {
        let event = OutcomeEvent {
            task_id: task_id.to_string(),
            outcome: outcome.clone(),
        };

        if let Err(tokio::sync::mpsc::error::TrySendError::Full(ev)) =
            self.outcome_tx.try_send(OutcomeMsg::Event(event))
        {
            self.outcome_tx.send(ev).await.map_err(|e| {
                RouterError::InvalidOutcome(format!("Outcome MPSC channel closed: {}", e))
            })?;
        }

        Ok(())
    }

    /// Flushes all pending asynchronous outcomes in the MPSC channel and awaits SQLite persistence.
    pub async fn flush_outcomes(&self) -> Result<(), RouterError> {
        let (ack_tx, ack_rx) = tokio::sync::oneshot::channel();
        self.outcome_tx
            .send(OutcomeMsg::Flush(ack_tx))
            .await
            .map_err(|e| {
                RouterError::InvalidOutcome(format!("Outcome MPSC channel closed: {}", e))
            })?;
        ack_rx
            .await
            .map_err(|e| RouterError::InvalidOutcome(format!("Outcome flush ACK dropped: {}", e)))?;
        Ok(())
    }

    /// Applies geometric decay factor (\gamma) on a stored Beta prior in SQLite:
    /// \alpha' = 1.0 + \gamma * (\alpha - 1.0)
    /// \beta' = 1.0 + \gamma * (\beta - 1.0)
    pub async fn decay_prior(
        &self,
        tier_name: &str,
        task_category: &str,
        gamma: f64,
    ) -> Result<crate::priors::BanditPrior, RouterError> {
        let now_sec = current_timestamp_sec();
        decay_bayesian_prior(&self.pool, tier_name, task_category, gamma, now_sec).await
    }

    /// Helper for computing the $E^3$ metric.
    pub fn calculate_e3_metric(structural_score: f64, direct_cost: f64, latency_sec: f64) -> f64 {
        calculate_e3_metric(structural_score, direct_cost, latency_sec)
    }
}
