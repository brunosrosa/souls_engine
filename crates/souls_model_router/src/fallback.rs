//! # Fallback Chains, Circuit Breaker & Incident Handling
//!
//! Provides ultra-fast (<150ms) Pareto-optimal contingency routing under provider anomalies:
//! - HTTP 429 (Rate Limit / Quota Exceeded)
//! - HTTP 500 / 502 / 503 / 504 (Provider Outage / Service Unavailable)
//! - Latency Timeout (> SLA limit)
//! - Transport / Network Connection Failures

use serde::{Deserialize, Serialize};

pub use crate::circuit_breaker::{
    current_time_sec, extract_cooldown_from_headers, parse_ratelimit_reset, parse_retry_after,
    CircuitBreakerState, CooldownManager, DynamicQuarantineConfig, IncidentKind,
};
use crate::cloud_provider::CloudProvider;

/// An individual candidate in the resolved fallback chain.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FallbackCandidate {
    /// Model arm identifier.
    pub model_id: String,
    /// Cloud or local provider classification.
    pub provider: CloudProvider,
    /// Execution tier (e.g. "tier1", "tier3", "tier0").
    pub tier_name: String,
    /// Calculated scalar utility score.
    pub utility_score: f64,
    /// Projected FinOps cost taking caching into account.
    pub projected_cost_usd: f64,
    /// Projected latency in milliseconds.
    pub projected_latency_ms: f64,
    /// Whether the model is currently under quarantine / circuit breaker cooldown.
    pub in_cooldown: bool,
    /// Remaining quarantine time in seconds if currently in cooldown.
    pub cooldown_remaining_secs: Option<u64>,
}

/// Complete contingency fallback chain for an execution request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FallbackChain {
    /// Task or request identifier.
    pub task_id: String,
    /// Highest-ranking candidate (primary execution arm).
    pub primary: FallbackCandidate,
    /// Ordered contingency backups if primary fails with 429/500/timeout.
    pub fallbacks: Vec<FallbackCandidate>,
    /// Configured FinOps budget ceiling in USD if present.
    pub budget_ceiling_usd: Option<f64>,
    /// Total resolution duration in milliseconds (< 150ms guarantee).
    pub resolution_ms: f64,
}

impl FallbackChain {
    /// Returns an ordered iterator starting from primary followed by active fallbacks.
    pub fn execution_order(&self) -> impl Iterator<Item = &FallbackCandidate> {
        std::iter::once(&self.primary).chain(self.fallbacks.iter())
    }

    /// Selects the next healthy candidate in the chain that is not in quarantine.
    pub fn select_first_healthy(&self) -> Option<&FallbackCandidate> {
        self.execution_order().find(|c| !c.in_cooldown)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circuit_breaker_quarantine_lifecycle() {
        let manager = CooldownManager::new();
        let model = "deepseek/deepseek-chat";

        // Initially healthy
        let (in_cd, _) = manager.is_quarantined(model);
        assert!(!in_cd);

        // Record 429 rate limit incident with 10s cooldown
        manager.record_incident(model, IncidentKind::RateLimited429, Some(10));
        let (in_cd_after, remaining) = manager.is_quarantined(model);
        assert!(in_cd_after);
        assert!(remaining.unwrap() > 0 && remaining.unwrap() <= 10);

        // Verify active quarantines listing
        let quars = manager.active_quarantines();
        assert_eq!(quars.len(), 1);
        assert_eq!(quars[0].0, model);
        assert_eq!(quars[0].2, IncidentKind::RateLimited429);

        // Record successful completion resets cooldown immediately
        manager.record_success(model);
        let (in_cd_reset, _) = manager.is_quarantined(model);
        assert!(!in_cd_reset);
    }
}

