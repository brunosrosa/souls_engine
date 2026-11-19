//! # Google Universe Hub Service (`souls_server::google_hub`)
//!
//! Coordinates the multi-account GV2 encrypted vault, BMAD Agent Inbox
//! (with Myers Diff via `souls_ast` and Gravity Lock enforcement),
//! Session Taint Tracker, incremental Docs Scratchpads, Custom Gems catalog,
//! OAuth2 PKCE flow state, Daily Briefing aggregation, and audio cache resolution.

#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use axum::http::StatusCode;
use chrono::{Duration as ChronoDuration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::sync::RwLock;
use tracing::{info, warn};

use souls_ast::{compute_safe_myers_diff, DiffTag};
use souls_core::error::CoreError;
use souls_core::google_bridge::{
    compute_commute_alert, default_google_data_dir, run_google_doctor, AgentInboxStore,
    DocsScratchpadStore, GemsCatalogStore, GoogleAccountRecord, GoogleMultiAccountVault,
    GoogleTasksClient, GoogleWorkspaceClient, NewProposalInput, RoutesV2Client, SessionTaintTracker,
};
use souls_protocol::google::{
    AccountPermissionPolicy, AuditEventDto, BlastRadiusTier, BmadProposalDto, DailyBriefingDto,
    GoogleAccountKind, GoogleDoctorReportDto, SlideDeckPlanDto,
};

/// Pending RFC 7636 OAuth2 PKCE challenge waiting for callback completion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingPkce {
    pub state: String,
    pub code_verifier: String,
    pub redirect_uri: String,
    pub label: String,
    pub kind: GoogleAccountKind,
    pub policy: AccountPermissionPolicy,
    pub created_at: i64,
}

/// Companion OAuth client metadata retained privately alongside `PendingPkce`.
#[derive(Debug, Clone)]
pub(crate) struct PkceOAuthMeta {
    pub account_id: Option<String>,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub token_endpoint: Option<String>,
    pub scopes: Vec<String>,
}

/// Central stateful service backing Google Life Hub, NotebookLM, Gems, and BMAD Agent Inbox.
pub struct GoogleHubService {
    pub data_dir: PathBuf,
    pub vault: Arc<RwLock<GoogleMultiAccountVault>>,
    pub inbox: Arc<AgentInboxStore>,
    pub taint_tracker: Arc<SessionTaintTracker>,
    pub scratchpads: Arc<DocsScratchpadStore>,
    pub gems: Arc<GemsCatalogStore>,
    pub audio_cache_dir: PathBuf,
    pub pkce_states: Arc<RwLock<HashMap<String, PendingPkce>>>,
    pkce_meta: Arc<RwLock<HashMap<String, PkceOAuthMeta>>>,
    doctor_cache: Arc<RwLock<Option<(i64, GoogleDoctorReportDto)>>>,
}

impl GoogleHubService {
    /// Initializes a `GoogleHubService` anchored at `data_dir` and creates required subdirectories.
    pub fn new(data_dir: PathBuf) -> Self {
        let _ = std::fs::create_dir_all(&data_dir);
        for sub in [
            "inbox",
            "scratchpads",
            "audio_cache",
            "exchange",
            "youtube_cache",
        ] {
            let _ = std::fs::create_dir_all(data_dir.join(sub));
        }

        let vault = GoogleMultiAccountVault::load_or_init(&data_dir)
            .expect("Failed to initialize GoogleMultiAccountVault");
        let inbox = AgentInboxStore::open(&data_dir).expect("Failed to open AgentInboxStore");
        let scratchpads =
            DocsScratchpadStore::open(&data_dir).expect("Failed to open DocsScratchpadStore");
        let gems = GemsCatalogStore::open(&data_dir).expect("Failed to open GemsCatalogStore");
        let audio_cache_dir = data_dir.join("audio_cache");

        Self {
            data_dir,
            vault: Arc::new(RwLock::new(vault)),
            inbox: Arc::new(inbox),
            taint_tracker: Arc::new(SessionTaintTracker::new()),
            scratchpads: Arc::new(scratchpads),
            gems: Arc::new(gems),
            audio_cache_dir,
            pkce_states: Arc::new(RwLock::new(HashMap::new())),
            pkce_meta: Arc::new(RwLock::new(HashMap::new())),
            doctor_cache: Arc::new(RwLock::new(None)),
        }
    }

    /// Clears the cached Google Doctor probe report so the next call re-probes host CLIs.
    pub async fn invalidate_doctor_cache(&self) {
        let mut guard = self.doctor_cache.write().await;
        *guard = None;
    }

    /// Returns a cached Google Doctor report (45s TTL) or executes a fresh parallel CLI probe.
    pub async fn get_doctor_report(&self, force_refresh: bool) -> GoogleDoctorReportDto {
        let now = Utc::now().timestamp();
        if !force_refresh {
            let guard = self.doctor_cache.read().await;
            if let Some((cached_at, ref report)) = *guard {
                if now - cached_at < 45 {
                    return report.clone();
                }
            }
        }

        let data_dir = self.data_dir.clone();
        let vault_snapshot = self.vault.read().await.clone();
        let report = tokio::task::spawn_blocking(move || run_google_doctor(&data_dir, &vault_snapshot))
            .await
            .unwrap_or_else(|_| GoogleDoctorReportDto {
                overall_status: "degraded".to_string(),
                clis: Vec::new(),
                accounts_count: 0,
                oauth_production_guidance: String::new(),
                keep_enterprise_note: String::new(),
                routes_v2_ready: false,
                warnings: vec!["Google Doctor probe task failed.".to_string()],
            });

        let mut guard = self.doctor_cache.write().await;
        *guard = Some((Utc::now().timestamp(), report.clone()));
        report
    }

    /// Initializes `GoogleHubService` using the canonical Dev Drive Google data directory.
    pub fn from_system() -> Self {
        Self::new(default_google_data_dir())
    }

    /// Stores auxiliary OAuth client metadata keyed by CSRF `state`.
    pub(crate) async fn store_pkce_meta(&self, state: &str, meta: PkceOAuthMeta) {
        let mut guard = self.pkce_meta.write().await;
        guard.insert(state.to_string(), meta);
    }

    /// Atomically removes and returns a pending PKCE challenge and its companion client metadata.
    pub(crate) async fn take_pkce(
        &self,
        state: &str,
    ) -> Option<(PendingPkce, Option<PkceOAuthMeta>)> {
        let pending = {
            let mut guard = self.pkce_states.write().await;
            guard.remove(state)
        }?;
        let meta = {
            let mut guard = self.pkce_meta.write().await;
            guard.remove(state)
        };
        Some((pending, meta))
    }

    /// Resolves a configured account by `account_id`/email, falling back to primary or default.
    pub async fn resolve_account(&self, account_id: Option<&str>) -> Option<GoogleAccountRecord> {
        let vault = self.vault.read().await;
        if let Some(id) = account_id.map(str::trim).filter(|s| !s.is_empty()) {
            vault
                .get_account(id)
                .cloned()
                .or_else(|| vault.get_primary_or_default().cloned())
        } else {
            vault.get_primary_or_default().cloned()
        }
    }

    /// Resolves a strictly matching account when an explicit `account_id` is supplied,
    /// or falls back to primary/default when `account_id` is `None`.
    pub async fn resolve_account_strict(
        &self,
        account_id: Option<&str>,
    ) -> Option<GoogleAccountRecord> {
        let vault = self.vault.read().await;
        match account_id.map(str::trim).filter(|s| !s.is_empty()) {
            Some(id) => vault.get_account(id).cloned(),
            None => vault.get_primary_or_default().cloned(),
        }
    }

    /// Records a redacted BMAD audit event into `<data_dir>/audit_log.jsonl`.
    pub fn record_audit_event(
        &self,
        session_id: &str,
        account_id: &str,
        tool_name: &str,
        tier: BlastRadiusTier,
        tainted: bool,
        outcome: &str,
        summary: &str,
    ) {
        let event = AuditEventDto {
            timestamp: Utc::now().timestamp(),
            session_id: session_id.to_string(),
            account_id: account_id.to_string(),
            tool_name: tool_name.to_string(),
            tier,
            tainted,
            outcome: outcome.to_string(),
            redacted_summary: summary.to_string(),
        };
        if let Err(err) = self.inbox.record_audit(event) {
            warn!(target: "souls_server::google_hub", "Failed to write audit event: {err}");
        }
    }

    /// Enqueues a BMAD change proposal, computing a real Myers Diff via `souls_ast`
    /// whenever `before_snapshot` and `after_snapshot` are present and `myers_diff` is empty.
    pub fn propose_with_diff(&self, mut input: NewProposalInput) -> Result<BmadProposalDto, String> {
        let needs_diff = input
            .myers_diff
            .as_deref()
            .is_none_or(|s| s.trim().is_empty());

        if needs_diff {
            if let (Some(before), Some(after)) = (&input.before_snapshot, &input.after_snapshot) {
                let patch = compute_safe_myers_diff(before, after).map_err(|e| e.to_string())?;
                let formatted = if patch.stats.unchanged {
                    "(no changes)".to_string()
                } else {
                    let mut diff_lines = Vec::with_capacity(patch.changes.len() + 1);
                    diff_lines.push(format!(
                        "@@ Myers Diff (+{} / -{}) @@",
                        patch.stats.additions, patch.stats.deletions
                    ));
                    for change in &patch.changes {
                        match change.tag {
                            DiffTag::Delete => diff_lines.push(format!("- {}", change.text)),
                            DiffTag::Insert => diff_lines.push(format!("+ {}", change.text)),
                            DiffTag::Equal => diff_lines.push(format!("  {}", change.text)),
                        }
                    }
                    diff_lines.join("\n")
                };
                input.myers_diff = Some(formatted);
            }
        }

        self.inbox.enqueue(input).map_err(|e| e.to_string())
    }

    /// Approves a pending proposal (validating Gravity Lock) and executes it against
    /// Google Workspace APIs when OAuth is configured, or records a clear `approved_dry_run` receipt.
    pub async fn execute_approved_proposal(
        &self,
        proposal_id: &str,
        gravity_lock_token: Option<&str>,
    ) -> Result<BmadProposalDto, (StatusCode, String)> {
        let approved = self
            .inbox
            .approve(proposal_id, gravity_lock_token)
            .map_err(map_approve_error)?;

        let account = self.resolve_account(Some(&approved.account_id)).await;
        let token_opt = account
            .as_ref()
            .and_then(|a| a.oauth_access_token.as_deref())
            .map(str::trim)
            .filter(|t| !t.is_empty())
            .map(ToString::to_string);

        let dry_run = approved
            .payload_json
            .get("dry_run")
            .and_then(Value::as_bool)
            .unwrap_or(false);

        let receipt = if let (Some(token), false) = (token_opt, dry_run) {
            match self.dispatch_proposal_remote(&approved, &token).await {
                Ok(remote_result) => json!({
                    "status": "executed",
                    "service": approved.service,
                    "action": approved.action,
                    "target_resource": approved.target_resource,
                    "account_id": approved.account_id,
                    "executed_at": Utc::now().timestamp(),
                    "remote_response": remote_result
                }),
                Err(err_msg) => {
                    self.record_audit_event(
                        "inbox_approval",
                        &approved.account_id,
                        &format!("{}.{}", approved.service, approved.action),
                        approved.tier,
                        approved.tainted_origin,
                        "execution_failed",
                        &err_msg,
                    );
                    return Err((StatusCode::BAD_GATEWAY, err_msg));
                }
            }
        } else {
            json!({
                "status": "approved_dry_run",
                "service": approved.service,
                "action": approved.action,
                "target_resource": approved.target_resource,
                "account_id": approved.account_id,
                "executed_at": Utc::now().timestamp(),
                "note": "No live OAuth access token configured (or dry_run requested); approval recorded locally."
            })
        };

        let executed = self
            .inbox
            .mark_executed(proposal_id, receipt)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        self.record_audit_event(
            "inbox_approval",
            &executed.account_id,
            &format!("{}.{}", executed.service, executed.action),
            executed.tier,
            executed.tainted_origin,
            "executed",
            &format!(
                "Executed proposal '{}' ({} -> {})",
                executed.id, executed.service, executed.target_resource
            ),
        );

        info!(
            target: "souls_server::google_hub",
            "Proposal '{}' approved and executed (service={}, action={})",
            executed.id,
            executed.service,
            executed.action
        );

        Ok(executed)
    }

    async fn dispatch_proposal_remote(
        &self,
        proposal: &BmadProposalDto,
        access_token: &str,
    ) -> Result<Value, String> {
        let base_override = proposal
            .payload_json
            .get("api_base_override")
            .and_then(Value::as_str);

        let ws = match base_override {
            Some(base) => GoogleWorkspaceClient::with_base_urls(base, base, base, base, base, base),
            None => GoogleWorkspaceClient::new(),
        };

        let service = proposal.service.to_ascii_lowercase();
        let action = proposal.action.to_ascii_lowercase();
        let payload = &proposal.payload_json;

        match (service.as_str(), action.as_str()) {
            ("gmail", "send_message" | "send") => {
                let to = payload
                    .get("to")
                    .and_then(Value::as_str)
                    .ok_or_else(|| "Gmail send payload missing 'to'".to_string())?;
                let subject = payload
                    .get("subject")
                    .and_then(Value::as_str)
                    .unwrap_or("(No Subject)");
                let body = payload.get("body").and_then(Value::as_str).unwrap_or("");
                ws.send_message(access_token, to, subject, body)
                    .await
                    .map_err(|e| e.to_string())
            }
            ("gmail", "create_draft" | "draft") => {
                let to = payload.get("to").and_then(Value::as_str).unwrap_or("");
                let subject = payload
                    .get("subject")
                    .and_then(Value::as_str)
                    .unwrap_or("(Draft)");
                let body = payload.get("body").and_then(Value::as_str).unwrap_or("");
                ws.create_draft(access_token, to, subject, body)
                    .await
                    .map_err(|e| e.to_string())
            }
            ("gmail", "bulk_modify" | "bulk") => {
                let ids = json_str_array(payload.get("ids"));
                let add_labels = json_str_array(payload.get("add_labels"));
                let remove_labels = json_str_array(payload.get("remove_labels"));
                ws.bulk_modify_labels(access_token, &ids, &add_labels, &remove_labels)
                    .await
                    .map_err(|e| e.to_string())
            }
            ("calendar", "create_event" | "invite") => {
                let summary = payload
                    .get("summary")
                    .and_then(Value::as_str)
                    .unwrap_or(&proposal.title);
                let start = payload
                    .get("start_rfc3339")
                    .and_then(Value::as_str)
                    .ok_or_else(|| "Calendar create payload missing 'start_rfc3339'".to_string())?;
                let end = payload
                    .get("end_rfc3339")
                    .and_then(Value::as_str)
                    .ok_or_else(|| "Calendar create payload missing 'end_rfc3339'".to_string())?;
                let location = payload.get("location").and_then(Value::as_str);
                let attendees = json_str_array(payload.get("attendees"));
                let send_updates = payload.get("send_updates").and_then(Value::as_str);
                let ev = ws
                    .create_event(
                        access_token,
                        summary,
                        start,
                        end,
                        location,
                        &attendees,
                        send_updates,
                    )
                    .await
                    .map_err(|e| e.to_string())?;
                serde_json::to_value(ev).map_err(|e| e.to_string())
            }
            ("calendar", "delete_event" | "delete") => {
                let event_id = payload
                    .get("event_id")
                    .and_then(Value::as_str)
                    .unwrap_or(&proposal.target_resource);
                ws.delete_event(access_token, event_id)
                    .await
                    .map_err(|e| e.to_string())?;
                Ok(json!({ "deleted": true, "event_id": event_id }))
            }
            ("drive", "update_permissions" | "share") => {
                let file_id = payload
                    .get("file_id")
                    .and_then(Value::as_str)
                    .unwrap_or(&proposal.target_resource);
                let role = payload
                    .get("role")
                    .and_then(Value::as_str)
                    .unwrap_or("reader");
                let perm_type = payload
                    .get("perm_type")
                    .or_else(|| payload.get("type"))
                    .and_then(Value::as_str)
                    .unwrap_or("user");
                let email = payload.get("email_address").and_then(Value::as_str);
                ws.update_permissions(access_token, file_id, role, perm_type, email)
                    .await
                    .map_err(|e| e.to_string())
            }
            ("drive", "delete_file" | "delete") => {
                let file_id = payload
                    .get("file_id")
                    .and_then(Value::as_str)
                    .unwrap_or(&proposal.target_resource);
                ws.delete_file(access_token, file_id)
                    .await
                    .map_err(|e| e.to_string())?;
                Ok(json!({ "deleted": true, "file_id": file_id }))
            }
            ("docs", "push_scratchpad" | "push" | "push_docs_content") => {
                let doc_id = payload
                    .get("doc_id")
                    .and_then(Value::as_str)
                    .unwrap_or(&proposal.target_resource);
                let content = if let Some(c) = payload.get("content").and_then(Value::as_str) {
                    c.to_string()
                } else if let Some(after) = proposal.after_snapshot.as_ref() {
                    after.clone()
                } else {
                    self.scratchpads
                        .render_text(doc_id)
                        .map_err(|e| e.to_string())?
                };
                ws.push_docs_content(access_token, doc_id, &content)
                    .await
                    .map_err(|e| e.to_string())
            }
            ("sheets", "write_range" | "write") => {
                let spreadsheet_id = payload
                    .get("spreadsheet_id")
                    .and_then(Value::as_str)
                    .unwrap_or(&proposal.target_resource);
                let range = payload
                    .get("range")
                    .and_then(Value::as_str)
                    .unwrap_or("Sheet1!A1");
                let values: Vec<Vec<Value>> = payload
                    .get("values")
                    .cloned()
                    .and_then(|v| serde_json::from_value(v).ok())
                    .unwrap_or_default();
                ws.write_range(access_token, spreadsheet_id, range, values)
                    .await
                    .map_err(|e| e.to_string())
            }
            ("slides", "create_presentation" | "create") => {
                let plan: SlideDeckPlanDto = if let Some(plan_val) = payload.get("plan") {
                    serde_json::from_value(plan_val.clone()).map_err(|e| e.to_string())?
                } else {
                    let topic = payload
                        .get("topic")
                        .and_then(Value::as_str)
                        .unwrap_or(&proposal.title);
                    let briefing = payload
                        .get("briefing")
                        .and_then(Value::as_str)
                        .unwrap_or(&proposal.summary);
                    let target_slides = payload
                        .get("target_slides")
                        .and_then(Value::as_u64)
                        .unwrap_or(8) as usize;
                    GoogleWorkspaceClient::build_slide_deck_plan(topic, briefing, target_slides)
                };
                let created = ws
                    .create_presentation(access_token, &plan)
                    .await
                    .map_err(|e| e.to_string())?;
                serde_json::to_value(created).map_err(|e| e.to_string())
            }
            ("tasks", "create_task" | "create") => {
                let tasks_client = match base_override {
                    Some(base) => GoogleTasksClient::with_base_url(base),
                    None => GoogleTasksClient::new(),
                };
                let tasklist_id = payload
                    .get("tasklist_id")
                    .and_then(Value::as_str)
                    .unwrap_or("@default");
                let title = payload
                    .get("title")
                    .and_then(Value::as_str)
                    .unwrap_or(&proposal.title);
                let notes = payload.get("notes").and_then(Value::as_str);
                let due = payload.get("due_rfc3339").and_then(Value::as_str);
                let created = tasks_client
                    .create_task(access_token, tasklist_id, title, notes, due)
                    .await
                    .map_err(|e| e.to_string())?;
                serde_json::to_value(created).map_err(|e| e.to_string())
            }
            ("tasks", "complete_task" | "complete") => {
                let tasks_client = match base_override {
                    Some(base) => GoogleTasksClient::with_base_url(base),
                    None => GoogleTasksClient::new(),
                };
                let tasklist_id = payload
                    .get("tasklist_id")
                    .and_then(Value::as_str)
                    .unwrap_or("@default");
                let task_id = payload
                    .get("task_id")
                    .and_then(Value::as_str)
                    .unwrap_or(&proposal.target_resource);
                let completed = tasks_client
                    .complete_task(access_token, tasklist_id, task_id)
                    .await
                    .map_err(|e| e.to_string())?;
                serde_json::to_value(completed).map_err(|e| e.to_string())
            }
            _ => Ok(json!({
                "status": "approved_recorded",
                "service": proposal.service,
                "action": proposal.action,
                "target_resource": proposal.target_resource
            })),
        }
    }

    /// Aggregates Calendar, Deep Focus Slots, Routes v2 Commute Alerts, Gmail Priority,
    /// Google Tasks, and pending Agent Inbox items with graceful offline degradation.
    pub async fn build_daily_briefing(&self, account_id: Option<&str>) -> DailyBriefingDto {
        let now = Utc::now();
        let pending_inbox_count = self.inbox.pending_count();
        let account = self.resolve_account(account_id).await;

        let Some(acct) = account else {
            return DailyBriefingDto {
                generated_at: now.timestamp(),
                account_email: None,
                oauth_connected: false,
                upcoming_events: Vec::new(),
                focus_slots: Vec::new(),
                commute_alerts: Vec::new(),
                priority_emails: Vec::new(),
                pending_tasks: Vec::new(),
                pending_inbox_count,
                notes: vec![
                    "Nenhuma conta Google configurada. Use POST /api/v1/google/oauth/start ou importe credenciais gws para ativar o Life Hub."
                        .to_string(),
                ],
            };
        };

        let account_email = if acct.email.trim().is_empty() {
            None
        } else {
            Some(acct.email.clone())
        };

        let Some(token) = acct
            .oauth_access_token
            .as_deref()
            .map(str::trim)
            .filter(|t| !t.is_empty())
        else {
            return DailyBriefingDto {
                generated_at: now.timestamp(),
                account_email,
                oauth_connected: false,
                upcoming_events: Vec::new(),
                focus_slots: Vec::new(),
                commute_alerts: Vec::new(),
                priority_emails: Vec::new(),
                pending_tasks: Vec::new(),
                pending_inbox_count,
                notes: vec![
                    "Conta registrada sem token OAuth2 ativo. Inicie POST /api/v1/google/oauth/start para sincronizar Calendar, Gmail e Tasks."
                        .to_string(),
                ],
            };
        };

        let ws = GoogleWorkspaceClient::new();
        let tasks_client = GoogleTasksClient::new();
        let time_min = now.to_rfc3339();
        let time_max = (now + ChronoDuration::hours(24)).to_rfc3339();

        let mut notes = Vec::new();

        let upcoming_events = match ws.list_events(token, &time_min, &time_max).await {
            Ok(evs) => evs,
            Err(err) => {
                notes.push(format!("Calendar unavailable: {err}"));
                Vec::new()
            }
        };

        let focus_slots =
            GoogleWorkspaceClient::find_deep_focus_slots(&upcoming_events, &time_min, &time_max, 60);

        let mut commute_alerts = Vec::new();
        let geolocated: Vec<_> = upcoming_events
            .iter()
            .filter(|ev| {
                ev.location
                    .as_deref()
                    .is_some_and(|loc| !loc.trim().is_empty())
            })
            .take(3)
            .collect();

        if !geolocated.is_empty() {
            let api_key = std::env::var("ROUTES_API_KEY")
                .ok()
                .filter(|s| !s.trim().is_empty())
                .or_else(|| {
                    std::env::var("GOOGLE_MAPS_API_KEY")
                        .ok()
                        .filter(|s| !s.trim().is_empty())
                });
            let origin = std::env::var("SOULS_COMMUTE_ORIGIN")
                .ok()
                .filter(|s| !s.trim().is_empty());

            if let (Some(key), Some(orig)) = (api_key, origin) {
                let routes_client = RoutesV2Client::new();
                for ev in geolocated {
                    if let Some(ref dest) = ev.location {
                        match routes_client.compute_route(&key, &orig, dest).await {
                            Ok(comp) => {
                                if let Ok(alert) = compute_commute_alert(
                                    &ev.id,
                                    &ev.summary,
                                    &ev.start_rfc3339,
                                    dest,
                                    comp.duration_seconds,
                                    comp.static_duration_seconds,
                                    comp.distance_meters,
                                    15,
                                ) {
                                    commute_alerts.push(alert);
                                }
                            }
                            Err(err) => {
                                notes.push(format!("Routes v2 lookup failed for '{}': {err}", ev.summary));
                            }
                        }
                    }
                }
            } else {
                notes.push(
                    "Geolocated events detected; configure ROUTES_API_KEY and SOULS_COMMUTE_ORIGIN for live traffic-aware departure alerts."
                        .to_string(),
                );
            }
        }

        let priority_emails = match ws.list_messages(token, "in:inbox is:unread", 15).await {
            Ok(msgs) => {
                let triaged = GoogleWorkspaceClient::triage_inbox_deterministically(&msgs);
                triaged.priority_messages.into_iter().take(10).collect()
            }
            Err(err) => {
                notes.push(format!("Gmail triage unavailable: {err}"));
                Vec::new()
            }
        };

        let pending_tasks = match tasks_client.list_tasks(token, "@default", false).await {
            Ok(tasks) => tasks,
            Err(err) => {
                notes.push(format!("Google Tasks unavailable: {err}"));
                Vec::new()
            }
        };

        DailyBriefingDto {
            generated_at: now.timestamp(),
            account_email,
            oauth_connected: true,
            upcoming_events,
            focus_slots,
            commute_alerts,
            priority_emails,
            pending_tasks,
            pending_inbox_count,
            notes,
        }
    }

    /// Resolves a cached NotebookLM audio artifact path under `audio_cache_dir`,
    /// rejecting any path traversal tokens (`..`, `/`, `\`, `:`).
    pub fn resolve_audio_path(&self, notebook_id: &str, artifact_id: &str) -> Option<PathBuf> {
        if !is_safe_path_segment(notebook_id) || !is_safe_path_segment(artifact_id) {
            return None;
        }

        let extensions = ["mp3", "m4a", "wav", "ogg"];
        let nb_dir = self.audio_cache_dir.join(notebook_id);

        if has_allowed_audio_extension(artifact_id, &extensions) {
            let exact_nb = nb_dir.join(artifact_id);
            if exact_nb.is_file() && is_contained_within(&exact_nb, &self.audio_cache_dir) {
                return Some(exact_nb);
            }
            let exact_root = self.audio_cache_dir.join(artifact_id);
            if exact_root.is_file() && is_contained_within(&exact_root, &self.audio_cache_dir) {
                return Some(exact_root);
            }
        }

        for ext in extensions {
            let candidate_nb = nb_dir.join(format!("{artifact_id}.{ext}"));
            if candidate_nb.is_file() && is_contained_within(&candidate_nb, &self.audio_cache_dir) {
                return Some(candidate_nb);
            }
        }

        for ext in extensions {
            let candidate_root = self.audio_cache_dir.join(format!("{artifact_id}.{ext}"));
            if candidate_root.is_file() && is_contained_within(&candidate_root, &self.audio_cache_dir)
            {
                return Some(candidate_root);
            }
        }

        None
    }
}

/// Validates a single identifier segment against path traversal attacks.
pub fn is_safe_path_segment(segment: &str) -> bool {
    let trimmed = segment.trim();
    !trimmed.is_empty()
        && trimmed != "."
        && trimmed != ".."
        && !trimmed.contains("..")
        && !trimmed.contains('/')
        && !trimmed.contains('\\')
        && !trimmed.contains(':')
        && !trimmed.contains('\0')
}

fn has_allowed_audio_extension(name: &str, extensions: &[&str]) -> bool {
    let lower = name.to_ascii_lowercase();
    extensions
        .iter()
        .any(|ext| lower.ends_with(&format!(".{ext}")))
}

fn is_contained_within(candidate: &Path, root: &Path) -> bool {
    match (candidate.canonicalize(), root.canonicalize()) {
        (Ok(c), Ok(r)) => c.starts_with(r),
        _ => candidate.starts_with(root),
    }
}

fn map_approve_error(err: CoreError) -> (StatusCode, String) {
    match err {
        CoreError::DangerousCommandBlocked(msg) => (StatusCode::FORBIDDEN, msg),
        CoreError::RefsPathLeakViolation(msg) => (StatusCode::BAD_REQUEST, msg),
        other => {
            let msg = other.to_string();
            if msg.contains("not found") {
                (StatusCode::NOT_FOUND, msg)
            } else {
                (StatusCode::BAD_REQUEST, msg)
            }
        }
    }
}

fn json_str_array(val: Option<&Value>) -> Vec<String> {
    val.and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(ToString::to_string))
                .collect()
        })
        .unwrap_or_default()
}
