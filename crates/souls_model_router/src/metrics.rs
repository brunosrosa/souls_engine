//! FinOps efficiency metrics and $E^3$ formula calculation.

use serde::{Deserialize, Serialize};

/// Small epsilon preventing division by zero when direct cost is 0.0 (e.g. local silicon).
pub const E3_EPSILON: f64 = 1e-6;

/// Calculates the $E^3$ (Efficacy, Economics, Execution) metric:
///
/// $$E^3 = \frac{\text{structural\_score}}{\text{direct\_cost} \cdot \text{latency\_sec} + \epsilon}$$
///
/// - For local tiers (Tiers 0, 0.5, 1, 2), `direct_cost = 0.0`, so $E^3 = \frac{\text{structural\_score}}{\epsilon}$.
///   With structural_score = 1.0, $E^3 = 1{,}000{,}000.0$, heavily rewarding zero-cost local execution.
/// - For cloud tiers, $E^3$ scales inversely with direct cost in USD and execution latency in seconds.
pub fn calculate_e3_metric(structural_score: f64, direct_cost: f64, latency_sec: f64) -> f64 {
    let score = structural_score.clamp(0.0, 1.0);
    let cost = direct_cost.max(0.0);
    let latency = latency_sec.max(0.0001);

    score / ((cost * latency) + E3_EPSILON)
}

/// Observed task outcome report submitted by subagent executors.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaskOutcome {
    /// Associated task ID.
    pub task_id: String,
    /// Tier or arm used (e.g. "tier1", "tier0", "tier3").
    pub tier_name: String,
    /// Task category (e.g. "code", "reasoning", "auxiliary.approval").
    pub task_category: String,
    /// Boolean success flag (e.g. clean compilation, zero exceptions).
    pub success: bool,
    /// Structural correctness score $\in [0.0, 1.0]$ (e.g. test passing ratio).
    pub structural_score: f64,
    /// Direct FinOps API cost incurred in USD (0.0 for local execution).
    pub direct_cost_usd: f64,
    /// Total wall-clock latency in milliseconds.
    pub latency_ms: f64,
    /// Actual input tokens processed.
    pub tokens_input: u32,
    /// Actual output tokens generated.
    pub tokens_output: u32,
}

impl TaskOutcome {
    /// Computes the $E^3$ metric for this outcome.
    pub fn compute_e3(&self) -> f64 {
        let latency_sec = (self.latency_ms / 1000.0).max(0.0001);
        calculate_e3_metric(self.structural_score, self.direct_cost_usd, latency_sec)
    }

    /// Computes the continuous fractional reward $r \in [0.0, 1.0]$ for Bayesian prior updates.
    ///
    /// If execution failed catastrophically, reward is 0.0.
    /// Otherwise, uses the normalized structural score.
    pub fn fractional_reward(&self) -> f64 {
        if !self.success {
            0.0
        } else {
            self.structural_score.clamp(0.0, 1.0)
        }
    }
}
