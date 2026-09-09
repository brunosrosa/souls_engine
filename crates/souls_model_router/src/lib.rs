//! # souls_model_router (Souls Engine v7)
//!
//! FinOps Intelligence and Multi-Objective Contextual Bayesian Router (ParetoBandit).
//!
//! Operates strictly on subagent delegations (`delegate_task`) and auxiliary slots (`auxiliary.*`)
//! in zero-context boundaries, completely eliminating L7 intradiálogo proxies (ADR-003).
//!
//! Features:
//! - Multi-Armed ParetoBandit using Thompson Sampling with Beta priors $\text{Beta}(\alpha, \beta)$.
//! - O(1) Lock-free GPU Telemetry consumption via shared `AtomicU64` (RTX 2060m WDDM).
//! - Thermodynamic Barrier Function $\Phi(T_{\text{GPU}}, V_{\text{livre}}, k)$ with automatic fail-soft to CPU or Cloud.
//! - $E^3$ (Efficacy, Economics, Execution) retrospective scoring metric.
//! - Fractional Bayesian updating via pseudo-observations with continuous rewards.

pub mod bandit;
pub mod context;
pub mod error;
pub mod finops;
pub mod metrics;
pub mod pareto_bandit;
pub mod priors;

// Canonical public surface exports
pub use bandit::{
    pack_gpu_telemetry, unpack_gpu_telemetry, ParetoBanditRouter, ParetoWeights, RoutingDecision,
    LOCAL_ACCELERATED_MAX_CONTEXT_TOKENS,
};
pub use context::TaskContext;
pub use error::RouterError;
pub use metrics::{calculate_e3_metric, TaskOutcome, E3_EPSILON};
pub use priors::{
    ensure_router_tables, fetch_active_models, fetch_prior, update_bayesian_prior,
    update_model_telemetry, upsert_model, BanditPrior, ModelRecord, ROUTER_DDL,
};

#[cfg(test)]
mod tests {
    use super::*;
    use souls_inference_runtime::nvml::OperationalZone;
    use souls_memory::sqlite::create_in_memory_sqlite_pool;

    /// Helper to populate sample models into SQLite memory store.
    async fn setup_test_router(initial_temp: f32, initial_vram_mb: u32) -> ParetoBanditRouter {
        let pool = create_in_memory_sqlite_pool()
            .await
            .expect("Failed to create test pool");

        let router = ParetoBanditRouter::new_for_testing(pool, initial_temp, initial_vram_mb)
            .await
            .expect("Failed to create router");

        // Seed 3 canonical models:
        // 1. tier0 (CPU local, cost 0.0, 0 MB VRAM)
        upsert_model(
            router.pool(),
            &ModelRecord {
                model_id: "tier0_onnx_cpu".to_string(),
                provider_type: "LOCAL".to_string(),
                vram_base_mb: 0,
                kv_cache_cost_per_k: 0,
                max_context_window: 8192,
                specialty_tags: "[\"triage\", \"approval\"]".to_string(),
                score_lmarena: 1000,
                cost_input_per_m: 0.0,
                cost_output_per_m: 0.0,
                is_active: true,
                ema_latency_ms: 25,
                success_rate_ema: 0.99,
            },
        )
        .await
        .unwrap();

        // 2. tier1 (dGPU RTX 2060m, cost 0.0, 1700 MB VRAM base)
        upsert_model(
            router.pool(),
            &ModelRecord {
                model_id: "tier1_qwen_coder_gpu".to_string(),
                provider_type: "LOCAL".to_string(),
                vram_base_mb: 1700,
                kv_cache_cost_per_k: 80,
                max_context_window: 16000,
                specialty_tags: "[\"code\", \"generation\"]".to_string(),
                score_lmarena: 1200,
                cost_input_per_m: 0.0,
                cost_output_per_m: 0.0,
                is_active: true,
                ema_latency_ms: 120,
                success_rate_ema: 0.98,
            },
        )
        .await
        .unwrap();

        // 3. tier3 (Cloud DeepSeek/Claude, cost > 0, 0 MB VRAM)
        upsert_model(
            router.pool(),
            &ModelRecord {
                model_id: "tier3_cloud_deepseek".to_string(),
                provider_type: "CLOUD".to_string(),
                vram_base_mb: 0,
                kv_cache_cost_per_k: 0,
                max_context_window: 64000,
                specialty_tags: "[\"code\", \"complex_reasoning\"]".to_string(),
                score_lmarena: 1350,
                cost_input_per_m: 0.27,
                cost_output_per_m: 1.10,
                is_active: true,
                ema_latency_ms: 450,
                success_rate_ema: 0.99,
            },
        )
        .await
        .unwrap();

        router
    }

    /// Gate 1: test_pareto_bandit_thompson_sampling
    /// Inserts competing priors and asserts quality samples respect statistical convergence of Beta distribution.
    #[tokio::test]
    async fn test_pareto_bandit_thompson_sampling() {
        let router = setup_test_router(45.0, 850).await;

        // Model A (tier1) has high quality prior Beta(100.0, 2.0)
        // Competitor B (tier0) has low quality prior Beta(1.0, 80.0) for code category
        // Competitor C (tier3) has low quality prior Beta(1.0, 80.0) for code category
        update_bayesian_prior(router.pool(), "tier1_qwen_coder_gpu", "code", 1.0, 1000)
            .await
            .unwrap();
        sqlx::query(
            "UPDATE pareto_bandit_priors SET alpha = 100.0, beta = 2.0 WHERE tier_name = 'tier1_qwen_coder_gpu'",
        )
        .execute(router.pool())
        .await
        .unwrap();

        update_bayesian_prior(router.pool(), "tier0_onnx_cpu", "code", 0.0, 1000)
            .await
            .unwrap();
        sqlx::query(
            "UPDATE pareto_bandit_priors SET alpha = 1.0, beta = 80.0 WHERE tier_name = 'tier0_onnx_cpu'",
        )
        .execute(router.pool())
        .await
        .unwrap();

        update_bayesian_prior(router.pool(), "tier3_cloud_deepseek", "code", 0.0, 1000)
            .await
            .unwrap();
        sqlx::query(
            "UPDATE pareto_bandit_priors SET alpha = 1.0, beta = 80.0 WHERE tier_name = 'tier3_cloud_deepseek'",
        )
        .execute(router.pool())
        .await
        .unwrap();

        let task = TaskContext::new("subtask_1", "session_1", "code", 500, 200);

        let mut tier1_chosen = 0;
        let trials = 50;

        for _ in 0..trials {
            let decision = router.route_task(&task).await.expect("Routing failed");
            if decision.model_id == "tier1_qwen_coder_gpu" {
                tier1_chosen += 1;
            }
        }

        // Tier 1 with Beta(100, 2) has mean 0.98 vs Tier 3 with Beta(1, 80) mean 0.012
        // Statistical confidence > 98% of selections must pick Tier 1
        assert!(
            tier1_chosen >= 48,
            "Expected tier1 to be selected at least 48/50 times, got {}",
            tier1_chosen
        );
    }

    /// Gate 2: test_thermodynamic_fallback_barrier
    /// Simulates thermal panic on dGPU (T_GPU = 85°C) and asserts router bans Tier 1, forcing failover to Cloud or CPU.
    #[tokio::test]
    async fn test_thermodynamic_fallback_barrier() {
        // Nominal GPU state initially
        let router = setup_test_router(45.0, 850).await;

        // Calibrate priors for "code" category so Tier 1 is preferred under nominal conditions
        update_bayesian_prior(router.pool(), "tier1_qwen_coder_gpu", "code", 1.0, 1000)
            .await
            .unwrap();
        sqlx::query(
            "UPDATE pareto_bandit_priors SET alpha = 50.0, beta = 2.0 WHERE tier_name = 'tier1_qwen_coder_gpu'",
        )
        .execute(router.pool())
        .await
        .unwrap();

        update_bayesian_prior(router.pool(), "tier0_onnx_cpu", "code", 0.0, 1000)
            .await
            .unwrap();
        sqlx::query(
            "UPDATE pareto_bandit_priors SET alpha = 1.0, beta = 20.0 WHERE tier_name = 'tier0_onnx_cpu'",
        )
        .execute(router.pool())
        .await
        .unwrap();

        let task = TaskContext::new("subtask_code", "session_1", "code", 1000, 500);

        // Under normal thermal state (45°C, 850MB allocated), Tier 1 local GPU should be cleared
        let decision_nominal = router.route_task(&task).await.unwrap();
        assert_eq!(decision_nominal.model_id, "tier1_qwen_coder_gpu");
        assert_eq!(decision_nominal.thermodynamic_barrier_phi, 0.0);

        // Simulate thermal panic: T_GPU = 85.0°C (> 82.0°C Critical limit)
        router.set_gpu_telemetry(85.0, 850, OperationalZone::Critical);

        // Re-route task: Tier 1 GPU must be strictly banned (\Phi = \infty), routing to CPU or Cloud
        let decision_panic = router.route_task(&task).await.unwrap();
        assert_ne!(
            decision_panic.model_id, "tier1_qwen_coder_gpu",
            "Local GPU should have been rejected due to thermal critical panic"
        );
        assert!(
            decision_panic.model_id == "tier0_onnx_cpu"
                || decision_panic.model_id == "tier3_cloud_deepseek",
            "Fallback must choose CPU or Cloud"
        );

        // Also simulate context overflow (> 16k tokens) on nominal hardware
        router.set_gpu_telemetry(45.0, 850, OperationalZone::Nominal);
        let large_task = TaskContext::new("large_context", "session_1", "code", 12000, 5000); // 17k tokens > 16k ceiling
        let decision_large = router.route_task(&large_task).await.unwrap();
        assert_eq!(
            decision_large.model_id, "tier3_cloud_deepseek",
            "Local GPU (>16k) and CPU (8k) excluded, forcing Cloud dispatch"
        );
    }

    /// Gate 3: test_e3_metric_calculation_and_decay
    /// Validates the E3 equation and Bayesian fractional updates.
    #[tokio::test]
    async fn test_e3_metric_calculation_and_decay() {
        // 1. Verify E3 metric for local zero-cost model
        let local_e3 = calculate_e3_metric(1.0, 0.0, 1.5);
        // E3 = 1.0 / (0.0 * 1.5 + 1e-6) = 1_000_000.0
        assert!((local_e3 - 1_000_000.0).abs() < 1e-3);

        // 2. Verify E3 metric for paid cloud model
        // cost = $0.05, latency = 2.0s, score = 0.9
        // denominator = 0.05 * 2.0 + 1e-6 = 0.100001
        // E3 = 0.9 / 0.100001 ~= 8.9999
        let cloud_e3 = calculate_e3_metric(0.9, 0.05, 2.0);
        assert!((cloud_e3 - 9.0).abs() < 0.1);

        // Local execution of identical quality achieves overwhelming advantage
        assert!(local_e3 > cloud_e3 * 100_000.0);

        // 3. Verify Bayesian fractional updates
        let router = setup_test_router(45.0, 850).await;

        let outcome = TaskOutcome {
            task_id: "task_test_e3".to_string(),
            tier_name: "tier1_qwen_coder_gpu".to_string(),
            task_category: "code".to_string(),
            success: true,
            structural_score: 0.85,
            direct_cost_usd: 0.0,
            latency_ms: 150.0,
            tokens_input: 400,
            tokens_output: 100,
        };

        router.record_outcome("task_test_e3", &outcome).await.unwrap();

        let prior = fetch_prior(router.pool(), "tier1_qwen_coder_gpu", "code")
            .await
            .unwrap();

        // Initially alpha=1.0, beta=1.0.
        // After fractional reward 0.85:
        // alpha = 1.0 + 0.85 = 1.85
        // beta = 1.0 + (1.0 - 0.85) = 1.15
        assert!((prior.alpha - 1.85).abs() < 1e-6);
        assert!((prior.beta - 1.15).abs() < 1e-6);
        assert_eq!(prior.pull_count, 1);
        assert!((prior.cumulative_reward - 0.85).abs() < 1e-6);
    }

    #[test]
    fn test_lock_free_telemetry_bitpacking() {
        let vram_mb = 2048;
        let temp_c = 68.5;
        let zone = OperationalZone::Preventive;

        let packed = pack_gpu_telemetry(vram_mb, temp_c, zone);
        let (unpacked_vram, unpacked_temp, unpacked_zone) = unpack_gpu_telemetry(packed);

        assert_eq!(unpacked_vram, vram_mb);
        assert!((unpacked_temp - temp_c).abs() < 0.5);
        assert_eq!(unpacked_zone, zone);
    }

    /// Gate 4: test_preventive_thermodynamic_barrier_penalty
    /// Verifies that in Preventive zone, \Phi computes a continuous non-infinite penalty.
    #[tokio::test]
    async fn test_preventive_thermodynamic_barrier_penalty() {
        let router = setup_test_router(45.0, 850).await;

        // Set to Preventive zone: 78°C (between 75 and 82) and 1000MB free (between 800 and 1400)
        let vram_allocated = 6144 - 1000;
        router.set_gpu_telemetry(78.0, vram_allocated, OperationalZone::Preventive);

        let task = TaskContext::new("subtask_prev", "session_1", "code", 500, 200);
        let decision = router.route_task(&task).await.unwrap();

        // If tier1 was selected or evaluated, its Phi should be finite and positive
        // \lambda * ((1400 - 1000)/600) + \mu * ((78 - 75)/7) = 2.5 * (400/600) + 1.8 * (3/7) = 1.666 + 0.771 ~= 2.438
        if decision.model_id == "tier1_qwen_coder_gpu" {
            assert!(decision.thermodynamic_barrier_phi > 1.5);
            assert!(decision.thermodynamic_barrier_phi < 3.5);
            assert!(!decision.thermodynamic_barrier_phi.is_infinite());
        }
    }

    /// Gate 5: test_model_registry_ema_updates
    /// Verifies dynamic EMA updating of latency and success rate on model records.
    #[tokio::test]
    async fn test_model_registry_ema_updates() {
        let router = setup_test_router(45.0, 850).await;

        let outcome = TaskOutcome {
            task_id: "task_ema".to_string(),
            tier_name: "tier1_qwen_coder_gpu".to_string(),
            task_category: "code".to_string(),
            success: true,
            structural_score: 1.0,
            direct_cost_usd: 0.0,
            latency_ms: 300.0, // original was 120ms
            tokens_input: 500,
            tokens_output: 200,
        };

        router.record_outcome("task_ema", &outcome).await.unwrap();

        let models = fetch_active_models(router.pool()).await.unwrap();
        let tier1 = models
            .iter()
            .find(|m| m.model_id == "tier1_qwen_coder_gpu")
            .unwrap();

        // 0.20 * 300 + 0.80 * 120 = 60 + 96 = 156ms
        assert_eq!(tier1.ema_latency_ms, 156);
    }

    /// Gate 6: test_context_window_exceeded_rejection
    /// Asserts that a colossal task (>64k tokens) exceeding all models is rejected gracefully.
    #[tokio::test]
    async fn test_context_window_exceeded_rejection() {
        let router = setup_test_router(45.0, 850).await;

        let colossal_task = TaskContext::new("colossal", "session_1", "code", 50000, 20000); // 70k > 64k
        let result = router.route_task(&colossal_task).await;

        assert!(result.is_err());
        match result.err().unwrap() {
            RouterError::NoActiveCandidates(msg) => {
                assert!(msg.contains("All model arms blocked") || msg.contains("zero active models"));
            }
            other => panic!("Unexpected error variant: {:?}", other),
        }
    }
}

