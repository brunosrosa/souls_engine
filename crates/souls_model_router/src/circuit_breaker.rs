//! # Dynamic Provider Quarantine & Circuit Breaker (`circuit_breaker.rs`)
//!
//! Provides intelligent header-aware circuit breaking:
//! - Inspects `retry-after` (integer seconds and HTTP-date RFC 2822/7231)
//! - Inspects `x-ratelimit-reset`, `ratelimit-reset`, and reset deltas
//! - Dynamically calculates quarantine: `max(header_cooldown_sec, default_backoff_sec)`
//! - Protects Bayesian priors ($\alpha, \beta$) from degradation during rate-limiting.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, info};

/// Categories of provider failures triggering contingency switching and quarantine.
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
    /// Recommended baseline cooldown duration in seconds based on failure severity.
    #[must_use]
    pub fn default_cooldown_seconds(&self) -> u64 {
        match self {
            Self::RateLimited429 => 60,       // Standard cloud quota replenishment window
            Self::ServerUnavailable500 => 30, // Transient server bounce backoff
            Self::Timeout => 15,              // Concurrency spike relief
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
    /// Whether the quarantine was dynamically instructed by response headers.
    pub from_header: bool,
}

/// Dynamic quarantine configuration thresholds.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DynamicQuarantineConfig {
    pub min_backoff_sec: u64,
    pub max_backoff_sec: u64,
}

impl Default for DynamicQuarantineConfig {
    fn default() -> Self {
        Self {
            min_backoff_sec: 5,
            max_backoff_sec: 3600, // 1 hour ceiling
        }
    }
}

/// Thread-safe in-memory circuit breaker and dynamic cooldown registry.
#[derive(Debug)]
pub struct CooldownManager {
    breakers: RwLock<HashMap<String, CircuitBreakerState>>,
    config: DynamicQuarantineConfig,
}

impl Default for CooldownManager {
    fn default() -> Self {
        Self::new()
    }
}

impl CooldownManager {
    /// Creates a new CooldownManager with default config.
    #[must_use]
    pub fn new() -> Self {
        Self {
            breakers: RwLock::new(HashMap::new()),
            config: DynamicQuarantineConfig::default(),
        }
    }

    /// Creates a CooldownManager with customized threshold config.
    #[must_use]
    pub fn with_config(config: DynamicQuarantineConfig) -> Self {
        Self {
            breakers: RwLock::new(HashMap::new()),
            config,
        }
    }

    /// Evaluates if an arm is currently in quarantine/cooldown.
    ///
    /// Returns `(is_in_cooldown, remaining_seconds_optional)`.
    #[must_use]
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
        let duration = custom_cooldown_secs
            .unwrap_or_else(|| kind.default_cooldown_seconds())
            .clamp(self.config.min_backoff_sec, self.config.max_backoff_sec);
        let quarantine_until = now.saturating_add(duration as i64);

        let mut guard = self.breakers.write().unwrap();
        let entry = guard.entry(model_id.to_string()).or_insert_with(|| CircuitBreakerState {
            quarantined_until_sec: 0,
            consecutive_failures: 0,
            last_incident: kind,
            last_failure_at_sec: now,
            from_header: false,
        });

        entry.consecutive_failures = entry.consecutive_failures.saturating_add(1);
        entry.quarantined_until_sec = quarantine_until;
        entry.last_incident = kind;
        entry.last_failure_at_sec = now;
        entry.from_header = custom_cooldown_secs.is_some();

        debug!(
            target: "souls_model_router::circuit_breaker",
            "Model '{}' quarantined for {}s until epoch {} (incident: {:?})",
            model_id,
            duration,
            quarantine_until,
            kind
        );
    }

    /// Inspects HTTP response headers (e.g. `retry-after`, `x-ratelimit-reset`)
    /// and sets an optimal dynamic quarantine window.
    pub fn record_incident_with_headers<I, K, V>(
        &self,
        model_id: &str,
        kind: IncidentKind,
        headers: I,
        now_epoch_sec: i64,
    ) -> u64
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        let header_cd = extract_cooldown_from_headers(headers, now_epoch_sec);
        let default_cd = kind.default_cooldown_seconds();
        let effective_cd = header_cd
            .unwrap_or(default_cd)
            .max(default_cd)
            .clamp(self.config.min_backoff_sec, self.config.max_backoff_sec);

        self.record_incident(model_id, kind, Some(effective_cd));

        if header_cd.is_some() {
            info!(
                target: "souls_model_router::circuit_breaker",
                "Dynamic quarantine of {}s applied to '{}' from HTTP headers",
                effective_cd,
                model_id
            );
        }

        effective_cd
    }

    /// Clears the quarantine state of an arm upon verified successful response.
    pub fn record_success(&self, model_id: &str) {
        let mut guard = self.breakers.write().unwrap();
        if let Some(state) = guard.get_mut(model_id) {
            state.consecutive_failures = 0;
            state.quarantined_until_sec = 0;
            state.from_header = false;
        }
    }

    /// Returns a snapshot of all active quarantined models.
    #[must_use]
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

/// Extracts retry-after or rate-limit reset cooldown in seconds from HTTP headers.
///
/// Supported headers:
/// - `retry-after`: Seconds (e.g. "60") or HTTP-date RFC 2822 / RFC 7231
/// - `x-ratelimit-reset`: Unix epoch timestamp or seconds delta
/// - `ratelimit-reset`: Same as above
/// - `x-ratelimit-reset-requests`: Reset delta
/// - `x-ratelimit-reset-tokens`: Reset delta
#[must_use]
pub fn extract_cooldown_from_headers<I, K, V>(headers: I, now_epoch_sec: i64) -> Option<u64>
where
    I: IntoIterator<Item = (K, V)>,
    K: AsRef<str>,
    V: AsRef<str>,
{
    let mut max_cooldown: Option<u64> = None;

    for (k, v) in headers {
        let key = k.as_ref().to_ascii_lowercase();
        let val = v.as_ref().trim();

        if key == "retry-after" {
            if let Some(cd) = parse_retry_after(val, now_epoch_sec) {
                max_cooldown = Some(max_cooldown.unwrap_or(0).max(cd));
            }
        } else if key == "x-ratelimit-reset"
            || key == "ratelimit-reset"
            || key == "x-ratelimit-reset-requests"
            || key == "x-ratelimit-reset-tokens"
        {
            if let Some(cd) = parse_ratelimit_reset(val, now_epoch_sec) {
                max_cooldown = Some(max_cooldown.unwrap_or(0).max(cd));
            }
        }
    }

    max_cooldown
}

/// Parses a `retry-after` header value (seconds or RFC 2822 / RFC 7231 date).
#[must_use]
pub fn parse_retry_after(val: &str, now_epoch_sec: i64) -> Option<u64> {
    if val.is_empty() {
        return None;
    }

    // 1. Integer seconds (e.g. "120")
    if let Ok(secs) = val.parse::<u64>() {
        return Some(secs);
    }

    // 2. Float seconds (e.g. "2.5")
    if let Ok(secs_f) = val.parse::<f64>() {
        if secs_f >= 0.0 {
            return Some(secs_f.ceil() as u64);
        }
    }

    // 3. RFC 2822 Date (e.g. "Wed, 21 Oct 2026 07:28:00 GMT")
    if let Ok(dt) = chrono::DateTime::parse_from_rfc2822(val) {
        let target = dt.timestamp();
        return Some(if target > now_epoch_sec {
            (target - now_epoch_sec) as u64
        } else {
            1
        });
    }

    // 4. IMF-fixdate (RFC 7231): "%a, %d %b %Y %H:%M:%S GMT"
    if let Ok(naive) = chrono::NaiveDateTime::parse_from_str(val, "%a, %d %b %Y %H:%M:%S GMT") {
        let target = naive.and_utc().timestamp();
        return Some(if target > now_epoch_sec {
            (target - now_epoch_sec) as u64
        } else {
            1
        });
    }

    None
}

/// Parses an `x-ratelimit-reset` header value (epoch timestamp or relative delta seconds).
#[must_use]
pub fn parse_ratelimit_reset(val: &str, now_epoch_sec: i64) -> Option<u64> {
    if val.is_empty() {
        return None;
    }

    // 1. Integer parse
    if let Ok(num) = val.parse::<i64>() {
        if num > 1_000_000_000 {
            // Absolute epoch timestamp
            return Some(if num > now_epoch_sec {
                (num - now_epoch_sec) as u64
            } else {
                1
            });
        } else if num > 0 {
            // Relative seconds
            return Some(num as u64);
        }
    }

    // 2. Float parse
    if let Ok(num_f) = val.parse::<f64>() {
        if num_f > 1_000_000_000.0 {
            let target = num_f as i64;
            return Some(if target > now_epoch_sec {
                (target - now_epoch_sec) as u64
            } else {
                1
            });
        } else if num_f > 0.0 {
            return Some(num_f.ceil() as u64);
        }
    }

    None
}

#[inline]
pub fn current_time_sec() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_cooldown_integer_seconds() {
        let headers = vec![("retry-after", "120")];
        let cd = extract_cooldown_from_headers(headers, 1000).unwrap();
        assert_eq!(cd, 120);
    }

    #[test]
    fn test_extract_cooldown_http_date() {
        let now = 1_700_000_000;
        let future_dt = chrono::DateTime::from_timestamp(now + 180, 0).unwrap();
        let rfc2822_str = future_dt.to_rfc2822();

        let headers = vec![("Retry-After", rfc2822_str.as_str())];
        let cd = extract_cooldown_from_headers(headers, now).unwrap();
        assert_eq!(cd, 180);
    }

    #[test]
    fn test_extract_cooldown_ratelimit_reset_epoch() {
        let now = 1_700_000_000;
        let reset_epoch = now + 45;
        let headers = vec![("x-ratelimit-reset", reset_epoch.to_string())];
        let cd = extract_cooldown_from_headers(headers, now).unwrap();
        assert_eq!(cd, 45);
    }

    #[test]
    fn test_record_incident_with_headers_takes_max() {
        let manager = CooldownManager::new();
        let model = "deepseek/deepseek-chat";

        // Headers specify 150s, default 429 is 60s => effective should be 150s
        let headers = vec![("retry-after", "150")];
        let eff = manager.record_incident_with_headers(
            model,
            IncidentKind::RateLimited429,
            headers,
            current_time_sec(),
        );
        assert_eq!(eff, 150);

        let (quarantined, remaining) = manager.is_quarantined(model);
        assert!(quarantined);
        assert!(remaining.unwrap() >= 148 && remaining.unwrap() <= 150);
    }
}

