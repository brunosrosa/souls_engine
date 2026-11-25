//! # Harness SurfaceSpec, Constitutional SHA-256 Pinning & ACI Guardrails (`v0.9.4`)
//!
//! Implements the 8-component Harness² anatomy (`c1`–`c8`) protections:
//! - Strict POSIX relative path normalization (`\` -> `/`, rejecting `..`) fixing Windows tree keys.
//! - Fail-closed SHA-256 cryptographic pinning on constitutional `c1` files (`AGENTS.md`, `SOUL.md`).
//! - `SCRUB_ENV` credential scrubbing for candidate `c6` script probing.
//! - SWE-agent ACI observation folding and Inspect AI `timeout_retry = false` idempotency guard.

#![forbid(unsafe_code)]

use std::collections::{BTreeMap, HashMap};
use std::path::Path;
use std::sync::RwLock;

use chrono::Utc;
use sha2::{Digest, Sha256};
use souls_protocol::{ConstitutionalPinStatusDto, HarnessComponentSlot};

use crate::error::CoreError;

/// Environment variables scrubbed before executing any candidate script or probe (`SCRUB_ENV`).
pub const SCRUB_ENV_KEYS: &[&str] = &[
    "GITHUB_TOKEN",
    "GH_TOKEN",
    "OPENAI_API_KEY",
    "ANTHROPIC_API_KEY",
    "GEMINI_API_KEY",
    "GOOGLE_API_KEY",
    "GOOGLE_CLIENT_SECRET",
    "OPENROUTER_API_KEY",
    "DEEPSEEK_API_KEY",
    "NOUS_API_KEY",
    "SOULS_JUDGE_KEY",
    "AWS_SECRET_ACCESS_KEY",
];

/// Normalizes a relative harness component path to strict POSIX `/` separators.
///
/// Prevents Windows backslash mismatches in `c1`–`c8` tree keys (Harness² Issue #151 / PR #159)
/// and blocks path traversal (`..`) or absolute/drive-qualified escapes.
pub fn normalize_posix_rel_path(raw: &str) -> Result<String, CoreError> {
    let replaced = raw.trim().replace('\\', "/");
    let trimmed = replaced.trim_start_matches('/');
    if trimmed.is_empty() {
        return Err(CoreError::HarnessGovernanceViolation(
            "Harness surface path cannot be empty".to_string(),
        ));
    }
    if trimmed.contains(':') {
        return Err(CoreError::RefsPathLeakViolation(format!(
            "Absolute or drive-prefixed path rejected in harness surface: {raw}"
        )));
    }

    let mut segments = Vec::new();
    for seg in trimmed.split('/') {
        if seg.is_empty() || seg == "." {
            continue;
        }
        if seg == ".." {
            return Err(CoreError::RefsPathLeakViolation(format!(
                "Path traversal '..' rejected in harness surface: {raw}"
            )));
        }
        segments.push(seg);
    }

    if segments.is_empty() {
        return Err(CoreError::HarnessGovernanceViolation(format!(
            "Invalid relative harness surface path: {raw}"
        )));
    }

    Ok(segments.join("/"))
}

/// Removes sensitive provider/judge credentials from candidate execution environments.
/// Returns the number of scrubbed entries.
pub fn scrub_candidate_environment(env: &mut BTreeMap<String, String>) -> usize {
    let keys_to_remove: Vec<String> = env
        .keys()
        .filter(|k| {
            let upper = k.to_ascii_uppercase();
            SCRUB_ENV_KEYS.iter().any(|blocked| *blocked == upper)
                || upper.ends_with("_API_KEY")
                || upper.ends_with("_SECRET")
                || upper.ends_with("_TOKEN")
        })
        .cloned()
        .collect();

    let count = keys_to_remove.len();
    for k in keys_to_remove {
        env.remove(&k);
    }
    count
}

/// Computes an LF-normalized SHA-256 hex digest so CRLF/LF differences never drift hashes.
#[must_use]
pub fn compute_lf_sha256_hex(bytes: &[u8]) -> String {
    let text = String::from_utf8_lossy(bytes);
    let normalized = text.replace("\r\n", "\n");
    let mut hasher = Sha256::new();
    hasher.update(normalized.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[derive(Debug, Clone)]
struct PinnedFileTarget {
    file_path_posix: String,
    absolute_path: Option<std::path::PathBuf>,
    expected_sha256: String,
    pin_mode: String,
}

fn format_resolved_path(path: &Path) -> String {
    let raw = std::fs::canonicalize(path)
        .unwrap_or_else(|_| path.to_path_buf())
        .to_string_lossy()
        .replace('\\', "/");
    raw.strip_prefix("//?/").unwrap_or(&raw).to_string()
}

fn normalize_pin_mode(mode: &str) -> String {
    if mode.trim().eq_ignore_ascii_case("watch") {
        "watch".to_string()
    } else {
        "fail_closed".to_string()
    }
}

/// Cryptographic verifier enforcing fail-closed immutability for constitutional (`c1`) files.
#[derive(Debug, Default)]
pub struct ConstitutionalPinVerifier {
    pins: RwLock<HashMap<String, PinnedFileTarget>>,
}

impl ConstitutionalPinVerifier {
    /// Creates an empty verifier.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Rejects any automated optimization targeting constitutionally pinned `c1` slots.
    pub fn reject_if_c1_mutated(slot: HarnessComponentSlot) -> Result<(), CoreError> {
        if slot.is_constitutionally_pinned() {
            return Err(CoreError::HarnessGovernanceViolation(
                "Slot c1 (Identidade & Regras Fundamentais) é constitucionalmente fixado (SHA-256 fail-closed) e não pode ser mutado automaticamente."
                    .to_string(),
            ));
        }
        Ok(())
    }

    /// Registers a file on disk as constitutionally pinned (`fail_closed`), capturing its baseline LF SHA-256.
    pub fn pin_file_from_disk<P: AsRef<Path>>(
        &self,
        rel_path: &str,
        abs_path: P,
    ) -> Result<ConstitutionalPinStatusDto, CoreError> {
        self.pin_file_from_disk_with_mode(rel_path, abs_path, "fail_closed")
    }

    /// Registers a file on disk with explicit governance mode (`"fail_closed"` or `"watch"`).
    pub fn pin_file_from_disk_with_mode<P: AsRef<Path>>(
        &self,
        rel_path: &str,
        abs_path: P,
        pin_mode: &str,
    ) -> Result<ConstitutionalPinStatusDto, CoreError> {
        let bytes = std::fs::read(abs_path.as_ref())?;
        let hash = compute_lf_sha256_hex(&bytes);
        self.pin_file_with_expected_hash(rel_path, abs_path, &hash, pin_mode)
    }

    /// Registers a file on disk against a previously persisted expected LF SHA-256 baseline.
    ///
    /// Preserves drift detection across daemon restarts (particularly for `"watch"` pins like
    /// `MEMORY.md` / `USER.md`) instead of silently overwriting the expected baseline on boot.
    pub fn pin_file_with_expected_hash<P: AsRef<Path>>(
        &self,
        rel_path: &str,
        abs_path: P,
        expected_sha256: &str,
        pin_mode: &str,
    ) -> Result<ConstitutionalPinStatusDto, CoreError> {
        let posix = normalize_posix_rel_path(rel_path)?;
        let abs_buf = abs_path.as_ref().to_path_buf();
        let resolved = Some(format_resolved_path(&abs_buf));
        let actual = match std::fs::read(&abs_buf) {
            Ok(bytes) => compute_lf_sha256_hex(&bytes),
            Err(_) => "missing_or_unreadable".to_string(),
        };
        let expected = expected_sha256.trim().to_ascii_lowercase();
        let drift_detected = actual != expected;
        let verified = !drift_detected;
        let mode = normalize_pin_mode(pin_mode);
        let fail_closed_triggered = (mode != "watch") && drift_detected;
        let now = Utc::now().timestamp();
        let target = PinnedFileTarget {
            file_path_posix: posix.clone(),
            absolute_path: Some(abs_buf),
            expected_sha256: expected.clone(),
            pin_mode: mode.clone(),
        };
        if let Ok(mut guard) = self.pins.write() {
            guard.insert(posix.clone(), target);
        }
        Ok(ConstitutionalPinStatusDto {
            file_path_posix: posix,
            expected_sha256: expected,
            actual_sha256: actual,
            verified,
            fail_closed_triggered,
            pin_mode: mode,
            drift_detected,
            resolved_path: resolved,
            checked_at: now,
        })
    }

    /// Registers an in-memory constitutional content hash (`fail_closed`).
    pub fn pin_in_memory(
        &self,
        rel_path: &str,
        baseline_content: &str,
    ) -> Result<ConstitutionalPinStatusDto, CoreError> {
        self.pin_in_memory_with_mode(rel_path, baseline_content, "fail_closed")
    }

    /// Registers an in-memory constitutional content hash with explicit mode (`"fail_closed"` or `"watch"`).
    pub fn pin_in_memory_with_mode(
        &self,
        rel_path: &str,
        baseline_content: &str,
        pin_mode: &str,
    ) -> Result<ConstitutionalPinStatusDto, CoreError> {
        let posix = normalize_posix_rel_path(rel_path)?;
        let hash = compute_lf_sha256_hex(baseline_content.as_bytes());
        let mode = normalize_pin_mode(pin_mode);
        let now = Utc::now().timestamp();
        let target = PinnedFileTarget {
            file_path_posix: posix.clone(),
            absolute_path: None,
            expected_sha256: hash.clone(),
            pin_mode: mode.clone(),
        };
        if let Ok(mut guard) = self.pins.write() {
            guard.insert(posix.clone(), target);
        }
        Ok(ConstitutionalPinStatusDto {
            file_path_posix: posix,
            expected_sha256: hash.clone(),
            actual_sha256: hash,
            verified: true,
            fail_closed_triggered: false,
            pin_mode: mode,
            drift_detected: false,
            resolved_path: None,
            checked_at: now,
        })
    }

    /// Exports current `file_path_posix -> expected_sha256` baselines for disk persistence.
    #[must_use]
    pub fn export_baselines(&self) -> BTreeMap<String, String> {
        self.pins
            .read()
            .map(|g| {
                g.iter()
                    .filter(|(_, t)| t.absolute_path.is_some())
                    .map(|(k, t)| (k.clone(), t.expected_sha256.clone()))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Re-calculates expected LF SHA-256 baselines for selected disk-backed pins (or all if `None`/empty).
    pub fn resign_disk_pins_selective(
        &self,
        paths: Option<&[String]>,
    ) -> Vec<ConstitutionalPinStatusDto> {
        let filter_set: Option<Vec<String>> = paths
            .filter(|slice| !slice.is_empty())
            .map(|slice| {
                slice
                    .iter()
                    .filter_map(|p| normalize_posix_rel_path(p).ok())
                    .collect()
            });

        if let Ok(mut guard) = self.pins.write() {
            for target in guard.values_mut() {
                if let Some(ref allowed) = filter_set {
                    if !allowed.iter().any(|p| p == &target.file_path_posix) {
                        continue;
                    }
                }
                if let Some(ref path) = target.absolute_path {
                    if let Ok(bytes) = std::fs::read(path) {
                        target.expected_sha256 = compute_lf_sha256_hex(&bytes);
                    }
                }
            }
        }
        self.verify_or_fail_closed()
    }

    /// Re-calculates expected LF SHA-256 baselines from all disk-backed pinned files after human confirmation.
    pub fn resign_all_disk_pins(&self) -> Vec<ConstitutionalPinStatusDto> {
        self.resign_disk_pins_selective(None)
    }

    /// Verifies all registered pinned files against disk (`fail_closed` blocks automations on mismatch;
    /// `watch` audits drift in `drift_detected = true` and `verified = false` without triggering fail-closed freeze).
    pub fn verify_or_fail_closed(&self) -> Vec<ConstitutionalPinStatusDto> {
        let now = Utc::now().timestamp();
        let snapshot: Vec<PinnedFileTarget> = self
            .pins
            .read()
            .map(|g| g.values().cloned().collect())
            .unwrap_or_default();

        let mut results = Vec::with_capacity(snapshot.len());
        for target in snapshot {
            let resolved_path = target.absolute_path.as_ref().map(|p| format_resolved_path(p));
            let actual = match &target.absolute_path {
                Some(path) => match std::fs::read(path) {
                    Ok(bytes) => compute_lf_sha256_hex(&bytes),
                    Err(_) => "missing_or_unreadable".to_string(),
                },
                None => target.expected_sha256.clone(),
            };
            let drift_detected = actual != target.expected_sha256;
            let verified = !drift_detected;
            let fail_closed_triggered = (target.pin_mode != "watch") && drift_detected;
            results.push(ConstitutionalPinStatusDto {
                file_path_posix: target.file_path_posix,
                expected_sha256: target.expected_sha256,
                actual_sha256: actual,
                verified,
                fail_closed_triggered,
                pin_mode: target.pin_mode,
                drift_detected,
                resolved_path,
                checked_at: now,
            });
        }
        results.sort_by(|a, b| a.file_path_posix.cmp(&b.file_path_posix));
        results
    }

    /// Verifies candidate bytes against a registered pin; returns error on mismatch (fail-closed).
    pub fn assert_candidate_matches_pin(
        &self,
        rel_path: &str,
        candidate_bytes: &[u8],
    ) -> Result<(), CoreError> {
        let posix = normalize_posix_rel_path(rel_path)?;
        let expected = self
            .pins
            .read()
            .ok()
            .and_then(|g| g.get(&posix).map(|p| p.expected_sha256.clone()));
        if let Some(exp) = expected {
            let actual = compute_lf_sha256_hex(candidate_bytes);
            if actual != exp {
                return Err(CoreError::HarnessGovernanceViolation(format!(
                    "Fail-closed SHA-256 constitutional mismatch for '{posix}': expected {exp}, got {actual}"
                )));
            }
        }
        Ok(())
    }
}

/// Collapses verbose past tool outputs into a concise SWE-agent ACI observation block,
/// preserving head lines, tail lines, and explicit diagnostic error lines.
#[must_use]
pub fn fold_aci_observation(tool_name: &str, raw_output: &str, max_chars: usize) -> String {
    let trimmed = raw_output.trim();
    if trimmed.is_empty() {
        return format!("[ACI:{tool_name}] Comando concluído sem saída padrão.");
    }
    let budget = max_chars.max(120);
    if trimmed.chars().count() <= budget {
        return trimmed.to_string();
    }

    let lines: Vec<&str> = trimmed.lines().collect();
    let head: String = lines.iter().take(3).copied().collect::<Vec<_>>().join("\n");
    let tail: String = lines
        .iter()
        .rev()
        .take(3)
        .copied()
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>()
        .join("\n");

    let error_hits: Vec<&str> = lines
        .iter()
        .copied()
        .filter(|l| {
            let lower = l.to_ascii_lowercase();
            lower.contains("error")
                || lower.contains("failed")
                || lower.contains("panic")
                || lower.contains("exception")
        })
        .take(3)
        .collect();

    let mut summary = format!(
        "[ACI:{tool_name} dobrado: {} linhas, {} chars originais]\n{}\n...",
        lines.len(),
        trimmed.chars().count(),
        head
    );
    if !error_hits.is_empty() {
        summary.push_str("\n[Sinais de Erro]:\n");
        summary.push_str(&error_hits.join("\n"));
    }
    summary.push_str("\n");
    summary.push_str(&tail);

    if summary.chars().count() > budget {
        summary.chars().take(budget).collect()
    } else {
        summary
    }
}

/// Inspect AI `timeout_retry = false` guard: returns `true` for external side-effecting
/// actions that MUST NEVER be retried automatically on timeout or replayed in Shadow Probing.
#[must_use]
pub fn is_non_idempotent_tool(tool_name: &str, action: Option<&str>) -> bool {
    let tool = tool_name.trim().to_ascii_lowercase();
    let act = action.unwrap_or("").trim().to_ascii_lowercase();

    match tool.as_str() {
        "inbox_resolve" | "google_chat" | "ui_interact" => true,
        "google_gmail" => matches!(act.as_str(), "send" | "draft" | "bulk" | "reply" | "delete"),
        "google_calendar" => matches!(act.as_str(), "create" | "update" | "delete"),
        "google_docs_scratchpad" => matches!(act.as_str(), "push" | "create"),
        "google_sheets" => matches!(act.as_str(), "append" | "update" | "write"),
        "google_slides" => matches!(act.as_str(), "create"),
        "google_tasks" => matches!(act.as_str(), "create" | "complete" | "delete"),
        "google_apps_script" => matches!(act.as_str(), "deploy" | "run" | "create"),
        _ => false,
    }
}
