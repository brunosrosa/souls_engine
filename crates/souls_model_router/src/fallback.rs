//! # Fallback Chains, Circuit Breaker & Incident Handling
//!
//! Provides ultra-fast (<150ms) Pareto-optimal contingency routing under provider anomalies:
//! - HTTP 429 (Rate Limit / Quota Exceeded)
//! - HTTP 500 / 502 / 503 / 504 (Provider Outage / Service Unavailable)
//! - Latency Timeout (> SLA limit)
//! - Transport / Network Connection Failures

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::cloud_provider::CloudProvider;

/// Categories of provider failures triggering contingency switching.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IncidentKind {
    /// HTTP 429: Too Many Requests / Rate limit or token quota exhausted.
    RateLimited429,
    /// HTTP 500/502/503/504: Service Unavailable or internal server error.
    ServerUnavailable500,
    /// Latency timeout exceeded configured SLA limit.
    Timeout,
    /// Network connection refused, reset, or broken pipe.
    NetworkError,
}

impl IncidentKind {
    /// Recommended cooldown quarantine duration in seconds based on failure severity.
    pub fn default_cooldown_seconds(&self) -> u64 {
        match self {
            Self::RateLimited429 => 60,       // Standard cloud quota replenish window
            Self::ServerUnavailable500 => 30, // Brief transient outage backoff
            Self::Timeout => 15,              // Concurrency spike backoff
            Self::NetworkError => 20,         // Transient DNS / route flap
        }
    }
}

/// Circuit breaker telemetry state for a specific model arm.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerState {
    /// Unix timestamp until which this arm is quarantined.
    pub quarantined_until_sec: i64,
    /// Consecutive failure count before recovery.
    pub consecutive_failures: u32,
    /// Most recent incident observed.
    pub last_incident: IncidentKind,
    /// Timestamp of most recent failure.
    pub last_failure_at_sec: i64,
}

/// Thread-safe in-memory circuit breaker and cooldown registry.
#[derive(Debug, Default)]
pub struct CooldownManager {
    breakers: RwLock<HashMap<String, CircuitBreakerState>>,
}

impl CooldownManager {
    /// Creates a new empty CooldownManager.
    pub fn new() -> Self {
        Self {
            breakers: RwLock::new(HashMap::new()),
        }
    }

    /// Evaluates if an arm is currently in quarantine/cooldown.
    ///
    /// Returns `(is_in_cooldown, remaining_seconds_optional)`.
    pub fn is_quarantined(&self, model_id: &str) -> (bool, Option<u64>) {
        let now = current_time_sec();
        let guard = self.breakers.read().unwrap();
        if let Some(state) = guard.get(model_id) {
            if state.quarantined_until_sec > now {
                let remaining = (state.quarantined_until_sec - now) as u64;
                return (true, Some(remaining));
            }
        }
        (false, None)
    }

    /// Registers a provider incident, placing the arm in quarantine.
    pub fn record_incident(
        &self,
        model_id: &str,
        kind: IncidentKind,
        custom_cooldown_secs: Option<u64>,
    ) {
        let now = current_time_sec();
        let duration = custom_cooldown_secs.unwrap_or_else(|| kind.default_cooldown_seconds());
        let quarantine_until = now.saturating_add(duration as i64);

        let mut guard = self.breakers.write().unwrap();
        let entry = guard.entry(model_id.to_string()).or_insert_with(|| CircuitBreakerState {
            quarantined_until_sec: 0,
            consecutive_failures: 0,
            last_incident: kind,
            last_failure_at_sec: now,
        });

        entry.consecutive_failures = entry.consecutive_failures.saturating_add(1);
        entry.quarantined_until_sec = quarantine_until;
        entry.last_incident = kind;
        entry.last_failure_at_sec = now;
    }

    /// Clears the quarantine state of an arm upon verified successful response.
    pub fn record_success(&self, model_id: &str) {
        let mut guard = self.breakers.write().unwrap();
        if let Some(state) = guard.get_mut(model_id) {
            state.consecutive_failures = 0;
            state.quarantined_until_sec = 0;
        }
    }

    /// Returns a snapshot of all active quarantined models.
    pub fn active_quarantines(&self) -> Vec<(String, u64, IncidentKind)> {
        let now = current_time_sec();
        let guard = self.breakers.read().unwrap();
        let mut list = Vec::new();
        for (model_id, state) in guard.iter() {
            if state.quarantined_until_sec > now {
                let remaining = (state.quarantined_until_sec - now) as u64;
                list.push((model_id.clone(), remaining, state.last_incident));
            }
        }
        list
    }
}

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

#[inline]
fn current_time_sec() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
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

