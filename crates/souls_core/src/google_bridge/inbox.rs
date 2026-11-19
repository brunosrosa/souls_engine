//! BMAD Agent Inbox proposal store with Gravity Lock enforcement and redacted audit logging.
//!
//! Proposals are persisted atomically as pure-LF JSON files in `<data_dir>/inbox/{proposal_id}.json`
//! and security events are appended to `<data_dir>/audit_log.jsonl`.

use super::security::redact_google_secrets;
use super::session::{generate_entropy_bytes, write_json_atomic_lf};
use crate::error::CoreError;
use chrono::Utc;
use souls_protocol::google::{AuditEventDto, BlastRadiusTier, BmadProposalDto, ProposalStatus};
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, RwLock};

/// Input descriptor for enqueuing a new BMAD change proposal.
#[derive(Debug, Clone)]
pub struct NewProposalInput {
    pub account_id: String,
    pub service: String,
    pub action: String,
    pub title: String,
    pub summary: String,
    pub tier: BlastRadiusTier,
    pub tainted_origin: bool,
    pub target_resource: String,
    pub before_snapshot: Option<String>,
    pub after_snapshot: Option<String>,
    pub myers_diff: Option<String>,
    pub payload_json: serde_json::Value,
    pub ttl_secs: i64,
}

/// Persistent BMAD Agent Inbox and audit log store.
pub struct AgentInboxStore {
    data_dir: PathBuf,
    inbox_dir: PathBuf,
    audit_path: PathBuf,
    proposals: RwLock<HashMap<String, BmadProposalDto>>,
    audit_lock: Mutex<()>,
}

impl AgentInboxStore {
    /// Opens (or initializes) an `AgentInboxStore` anchored at `data_dir`.
    pub fn open(data_dir: &Path) -> Result<Self, CoreError> {
        let inbox_dir = data_dir.join("inbox");
        fs::create_dir_all(&inbox_dir).map_err(|e| {
            CoreError::SubprocessError(format!(
                "Failed to create inbox directory '{}': {e}",
                inbox_dir.display()
            ))
        })?;

        let audit_path = data_dir.join("audit_log.jsonl");
        let mut map = HashMap::new();

        for entry in fs::read_dir(&inbox_dir).map_err(|e| {
            CoreError::SubprocessError(format!(
                "Failed to scan inbox directory '{}': {e}",
                inbox_dir.display()
            ))
        })? {
            let entry = entry.map_err(CoreError::Io)?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            if path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with('.'))
            {
                continue;
            }
            let content = fs::read_to_string(&path).map_err(|e| {
                CoreError::SubprocessError(format!(
                    "Failed to read proposal file '{}': {e}",
                    path.display()
                ))
            })?;
            if let Ok(prop) = serde_json::from_str::<BmadProposalDto>(&content) {
                map.insert(prop.id.clone(), prop);
            }
        }

        Ok(Self {
            data_dir: data_dir.to_path_buf(),
            inbox_dir,
            audit_path,
            proposals: RwLock::new(map),
            audit_lock: Mutex::new(()),
        })
    }

    /// Returns the base data directory for this store.
    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    /// Enqueues a new proposal, generating a Gravity Lock token when required (`Tier2` or `tainted_origin`).
    pub fn enqueue(&self, input: NewProposalInput) -> Result<BmadProposalDto, CoreError> {
        let now = Utc::now().timestamp();
        let ttl = if input.ttl_secs > 0 {
            input.ttl_secs
        } else {
            600
        };
        let expires_at = now.saturating_add(ttl);

        let id = format!("prop_{}_{}", now, hex_entropy::<6>());
        let gravity_lock_required =
            matches!(input.tier, BlastRadiusTier::Tier2ExternalDestructive) || input.tainted_origin;
        let gravity_lock_token = if gravity_lock_required {
            Some(format!("glock_{}", hex_entropy::<12>()))
        } else {
            None
        };

        let myers_diff = input.myers_diff.or_else(|| {
            match (&input.before_snapshot, &input.after_snapshot) {
                (Some(before), Some(after)) => Some(compute_line_diff(before, after)),
                _ => None,
            }
        });

        let proposal = BmadProposalDto {
            id: id.clone(),
            created_at: now,
            expires_at,
            account_id: input.account_id,
            service: input.service,
            action: input.action,
            title: input.title,
            summary: input.summary,
            tier: input.tier,
            tainted_origin: input.tainted_origin,
            target_resource: input.target_resource,
            before_snapshot: input.before_snapshot,
            after_snapshot: input.after_snapshot,
            myers_diff,
            payload_json: input.payload_json,
            status: ProposalStatus::Pending,
            gravity_lock_required,
            gravity_lock_token,
            rejection_reason: None,
            execution_receipt: None,
        };

        let path = self.proposal_path(&id)?;
        write_json_atomic_lf(&path, &proposal)?;

        if let Ok(mut guard) = self.proposals.write() {
            guard.insert(id, proposal.clone());
        }

        Ok(proposal)
    }

    /// Lists proposals optionally filtered by status, sorted newest first.
    pub fn list(&self, status_filter: Option<ProposalStatus>) -> Vec<BmadProposalDto> {
        let mut items: Vec<BmadProposalDto> = self
            .proposals
            .read()
            .map(|guard| {
                guard
                    .values()
                    .filter(|p| status_filter.is_none_or(|f| p.status == f))
                    .cloned()
                    .collect()
            })
            .unwrap_or_default();

        items.sort_by(|a, b| {
            b.created_at
                .cmp(&a.created_at)
                .then_with(|| b.id.cmp(&a.id))
        });
        items
    }

    /// Returns the count of active (unexpired) pending proposals.
    pub fn pending_count(&self) -> usize {
        let now = Utc::now().timestamp();
        self.proposals
            .read()
            .map(|guard| {
                guard
                    .values()
                    .filter(|p| p.status == ProposalStatus::Pending && now <= p.expires_at)
                    .count()
            })
            .unwrap_or(0)
    }

    /// Retrieves a proposal by ID.
    pub fn get(&self, id: &str) -> Option<BmadProposalDto> {
        self.proposals
            .read()
            .ok()
            .and_then(|guard| guard.get(id).cloned())
    }

    /// Approves a pending proposal, verifying expiration and Gravity Lock token when required.
    pub fn approve(
        &self,
        id: &str,
        provided_gravity_token: Option<&str>,
    ) -> Result<BmadProposalDto, CoreError> {
        let mut guard = self.proposals.write().map_err(|_| {
            CoreError::SubprocessError("Inbox proposal lock poisoned".to_string())
        })?;

        let prop = guard.get_mut(id).ok_or_else(|| {
            CoreError::SubprocessError(format!("Proposal '{id}' not found"))
        })?;

        if prop.status != ProposalStatus::Pending {
            return Err(CoreError::SubprocessError(format!(
                "Proposal '{id}' cannot be approved in state {:?}",
                prop.status
            )));
        }

        let now = Utc::now().timestamp();
        if now > prop.expires_at {
            prop.status = ProposalStatus::Expired;
            let path = self.proposal_path(id)?;
            write_json_atomic_lf(&path, prop)?;
            return Err(CoreError::SubprocessError(format!(
                "Proposal '{id}' has expired at {}",
                prop.expires_at
            )));
        }

        if prop.gravity_lock_required {
            let expected = prop.gravity_lock_token.as_deref().unwrap_or("");
            let provided = provided_gravity_token.unwrap_or("");
            if expected.is_empty() || !constant_time_str_eq(expected, provided) {
                return Err(CoreError::DangerousCommandBlocked(format!(
                    "Gravity Lock verification failed for proposal '{id}': missing or invalid token"
                )));
            }
        }

        prop.status = ProposalStatus::Approved;
        let updated = prop.clone();
        let path = self.proposal_path(id)?;
        write_json_atomic_lf(&path, &updated)?;
        Ok(updated)
    }

    /// Marks an approved proposal as executed and records its execution receipt.
    pub fn mark_executed(
        &self,
        id: &str,
        receipt: serde_json::Value,
    ) -> Result<BmadProposalDto, CoreError> {
        let mut guard = self.proposals.write().map_err(|_| {
            CoreError::SubprocessError("Inbox proposal lock poisoned".to_string())
        })?;

        let prop = guard.get_mut(id).ok_or_else(|| {
            CoreError::SubprocessError(format!("Proposal '{id}' not found"))
        })?;

        if prop.status != ProposalStatus::Approved {
            return Err(CoreError::SubprocessError(format!(
                "Proposal '{id}' must be Approved before marking Executed (current: {:?})",
                prop.status
            )));
        }

        prop.status = ProposalStatus::Executed;
        prop.execution_receipt = Some(receipt);
        let updated = prop.clone();
        let path = self.proposal_path(id)?;
        write_json_atomic_lf(&path, &updated)?;
        Ok(updated)
    }

    /// Rejects a pending or approved proposal with a human-readable reason.
    pub fn reject(&self, id: &str, reason: &str) -> Result<BmadProposalDto, CoreError> {
        let mut guard = self.proposals.write().map_err(|_| {
            CoreError::SubprocessError("Inbox proposal lock poisoned".to_string())
        })?;

        let prop = guard.get_mut(id).ok_or_else(|| {
            CoreError::SubprocessError(format!("Proposal '{id}' not found"))
        })?;

        if !matches!(prop.status, ProposalStatus::Pending | ProposalStatus::Approved) {
            return Err(CoreError::SubprocessError(format!(
                "Proposal '{id}' cannot be rejected in state {:?}",
                prop.status
            )));
        }

        prop.status = ProposalStatus::Rejected;
        prop.rejection_reason = Some(reason.to_string());
        let updated = prop.clone();
        let path = self.proposal_path(id)?;
        write_json_atomic_lf(&path, &updated)?;
        Ok(updated)
    }

    /// Appends a redacted audit event to `<data_dir>/audit_log.jsonl` using pure LF endings.
    pub fn record_audit(&self, mut event: AuditEventDto) -> Result<(), CoreError> {
        event.redacted_summary = redact_google_secrets(&event.redacted_summary);

        let mut line = serde_json::to_string(&event).map_err(|e| {
            CoreError::SubprocessError(format!("Failed to serialize audit event: {e}"))
        })?;
        if line.contains('\r') {
            line = line.replace(['\r', '\n'], " ");
        }
        line.push('\n');

        let _lock = self.audit_lock.lock().map_err(|_| {
            CoreError::SubprocessError("Audit log mutex poisoned".to_string())
        })?;

        if let Some(parent) = self.audit_path.parent() {
            fs::create_dir_all(parent).map_err(CoreError::Io)?;
        }

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.audit_path)
            .map_err(|e| {
                CoreError::SubprocessError(format!(
                    "Failed to open audit log '{}': {e}",
                    self.audit_path.display()
                ))
            })?;

        file.write_all(line.as_bytes()).map_err(|e| {
            CoreError::SubprocessError(format!(
                "Failed to append to audit log '{}': {e}",
                self.audit_path.display()
            ))
        })?;

        Ok(())
    }

    /// Reads up to `limit` most recent audit events from `<data_dir>/audit_log.jsonl` (newest first).
    pub fn list_audit(&self, limit: usize) -> Vec<AuditEventDto> {
        if limit == 0 || !self.audit_path.exists() {
            return Vec::new();
        }
        let Ok(_lock) = self.audit_lock.lock() else {
            return Vec::new();
        };
        let Ok(content) = fs::read_to_string(&self.audit_path) else {
            return Vec::new();
        };

        let mut events = Vec::new();
        for line in content.lines().rev() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            if let Ok(evt) = serde_json::from_str::<AuditEventDto>(trimmed) {
                events.push(evt);
                if events.len() >= limit {
                    break;
                }
            }
        }
        events
    }

    fn proposal_path(&self, id: &str) -> Result<PathBuf, CoreError> {
        if id.is_empty()
            || id.contains("..")
            || id.contains('/')
            || id.contains('\\')
            || id.contains(':')
        {
            return Err(CoreError::RefsPathLeakViolation(format!(
                "Invalid proposal identifier '{id}'"
            )));
        }
        Ok(self.inbox_dir.join(format!("{id}.json")))
    }
}

fn hex_entropy<const N: usize>() -> String {
    let bytes = generate_entropy_bytes::<N>();
    let mut s = String::with_capacity(N * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

fn constant_time_str_eq(a: &str, b: &str) -> bool {
    let a_bytes = a.as_bytes();
    let b_bytes = b.as_bytes();
    if a_bytes.len() != b_bytes.len() {
        return false;
    }
    let mut diff = 0u8;
    for (&x, &y) in a_bytes.iter().zip(b_bytes.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

/// Computes a concise line-by-line diff (`-`/`+`/` `) using LCS.
fn compute_line_diff(before: &str, after: &str) -> String {
    let a: Vec<&str> = before.lines().collect();
    let b: Vec<&str> = after.lines().collect();

    if a.len().saturating_mul(b.len()) > 40_000 {
        let mut fallback = Vec::new();
        for l in &a {
            fallback.push(format!("- {l}"));
        }
        for l in &b {
            fallback.push(format!("+ {l}"));
        }
        return fallback.join("\n");
    }

    let m = a.len();
    let n = b.len();
    let mut dp = vec![vec![0usize; n + 1]; m + 1];
    for i in (0..m).rev() {
        for j in (0..n).rev() {
            dp[i][j] = if a[i] == b[j] {
                dp[i + 1][j + 1] + 1
            } else {
                dp[i + 1][j].max(dp[i][j + 1])
            };
        }
    }

    let mut lines = Vec::new();
    let (mut i, mut j) = (0usize, 0usize);
    while i < m && j < n {
        if a[i] == b[j] {
            lines.push(format!("  {}", a[i]));
            i += 1;
            j += 1;
        } else if dp[i + 1][j] >= dp[i][j + 1] {
            lines.push(format!("- {}", a[i]));
            i += 1;
        } else {
            lines.push(format!("+ {}", b[j]));
            j += 1;
        }
    }
    while i < m {
        lines.push(format!("- {}", a[i]));
        i += 1;
    }
    while j < n {
        lines.push(format!("+ {}", b[j]));
        j += 1;
    }
    lines.join("\n")
}
