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
use std::time::Instant;

use axum::http::StatusCode;
use chrono::{Duration as ChronoDuration, Local, Timelike, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::sync::{broadcast, RwLock};
use tracing::{info, warn};

use souls_ast::{compute_safe_myers_diff, DiffTag};
use souls_core::error::CoreError;
use souls_core::google_bridge::{
    check_chat_eligibility, compute_commute_alert, compute_form_stats, default_google_data_dir,
    dispatch_external_pulse_alert, is_within_quiet_hours, prepare_google_corpus_tier0,
    refresh_access_token, run_google_doctor, should_dispatch_external, AgentInboxStore,
    DocsScratchpadStore, GemsCatalogStore, GoogleAccountRecord, GoogleAppsScriptClient,
    GoogleChatClient, GoogleFormsClient, GoogleMultiAccountVault, GoogleSessionCredentials,
    GoogleSessionVault, GoogleTasksClient, GoogleWorkspaceClient, NewProposalInput, NlmCliBridge,
    OAuth2Config, ProactivePulseStore, RoutesV2Client, SessionTaintTracker, WizRpcClient,
    WizRpcId, WizRpcPayload,
};
use souls_inference_runtime::{
    resolve_llama_completion_binary, MinionBatchOrchestrator, MinionTask, MinionTaskKind,
};
use souls_llm_inventory::{
    probe_hardware, resolve_canonical_models_dir, scan_local_models, ModelFormat,
};
use souls_protocol::google::{
    AccountPermissionPolicy, AppsScriptFileDto, AuditEventDto, BlastRadiusTier, BmadProposalDto,
    DailyBriefingDto, DriveNotebookSyncLinkDto, GoogleAccountKind, GoogleDoctorReportDto,
    GoogleFormResponseDto, MinionAnalyzeMode, MinionGoogleAnalysisDto, MinionGoogleSourceKind,
    PulseCategory, PulseEventDto, PulseSnapshotDto, PulseUrgency, SlideDeckPlanDto,
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
    pub pulse: Arc<ProactivePulseStore>,
    pub pulse_tx: broadcast::Sender<PulseEventDto>,
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
            "minions_cache",
        ] {
            let _ = std::fs::create_dir_all(data_dir.join(sub));
        }

        let vault = GoogleMultiAccountVault::load_or_init(&data_dir)
            .expect("Failed to initialize GoogleMultiAccountVault");
        let inbox = AgentInboxStore::open(&data_dir).expect("Failed to open AgentInboxStore");
        let scratchpads =
            DocsScratchpadStore::open(&data_dir).expect("Failed to open DocsScratchpadStore");
        let gems = GemsCatalogStore::open(&data_dir).expect("Failed to open GemsCatalogStore");
        let pulse =
            ProactivePulseStore::open(&data_dir).expect("Failed to open ProactivePulseStore");
        let (pulse_tx, _) = broadcast::channel(128);
        let audio_cache_dir = data_dir.join("audio_cache");

        Self {
            data_dir,
            vault: Arc::new(RwLock::new(vault)),
            inbox: Arc::new(inbox),
            taint_tracker: Arc::new(SessionTaintTracker::new()),
            scratchpads: Arc::new(scratchpads),
            gems: Arc::new(gems),
            pulse: Arc::new(pulse),
            pulse_tx,
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

        let is_chat_webhook_with_url = approved.service.eq_ignore_ascii_case("chat")
            && approved.action.eq_ignore_ascii_case("webhook")
            && approved
                .payload_json
                .get("webhook_url")
                .and_then(Value::as_str)
                .is_some_and(|u| !u.trim().is_empty());

        let effective_token = if is_chat_webhook_with_url {
            Some(token_opt.unwrap_or_default())
        } else {
            token_opt
        };

        let mut remote_succeeded = false;
        let receipt = if let (Some(token), false) = (effective_token, dry_run) {
            match self.dispatch_proposal_remote(&approved, &token).await {
                Ok(remote_result) => {
                    remote_succeeded = true;
                    json!({
                        "status": "executed",
                        "service": approved.service,
                        "action": approved.action,
                        "target_resource": approved.target_resource,
                        "account_id": approved.account_id,
                        "executed_at": Utc::now().timestamp(),
                        "remote_response": remote_result
                    })
                }
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

        let mut executed = self
            .inbox
            .mark_executed(proposal_id, receipt)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        if remote_succeeded {
            let svc_l = executed.service.to_ascii_lowercase();
            let act_l = executed.action.to_ascii_lowercase();
            let is_docs_push =
                svc_l == "docs" && matches!(act_l.as_str(), "push_scratchpad" | "push" | "push_docs_content");
            let is_sheets_write =
                svc_l == "sheets" && matches!(act_l.as_str(), "write_range" | "write");
            if is_docs_push || is_sheets_write {
                let file_id = executed
                    .payload_json
                    .get("doc_id")
                    .or_else(|| executed.payload_json.get("spreadsheet_id"))
                    .and_then(Value::as_str)
                    .unwrap_or(&executed.target_resource)
                    .trim()
                    .to_string();
                if !file_id.is_empty() {
                    let sync_outcomes = self.trigger_drive_notebook_autosync(&file_id).await;
                    if !sync_outcomes.is_empty() {
                        if let Some(obj) = executed
                            .execution_receipt
                            .as_mut()
                            .and_then(Value::as_object_mut)
                        {
                            obj.insert("notebook_autosync".to_string(), Value::Array(sync_outcomes));
                        }
                    }
                }
            }
        }

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
            ("apps_script", "push_content" | "push") => {
                let script_client = match base_override {
                    Some(base) => GoogleAppsScriptClient::with_base_url(base),
                    None => GoogleAppsScriptClient::new(),
                };
                let script_id = payload
                    .get("script_id")
                    .and_then(Value::as_str)
                    .unwrap_or(&proposal.target_resource);
                let files: Vec<AppsScriptFileDto> = payload
                    .get("files")
                    .cloned()
                    .and_then(|v| serde_json::from_value(v).ok())
                    .ok_or_else(|| "Apps Script push payload missing valid 'files' array".to_string())?;
                let updated = script_client
                    .update_content(access_token, script_id, &files)
                    .await
                    .map_err(|e| e.to_string())?;
                serde_json::to_value(updated).map_err(|e| e.to_string())
            }
            ("apps_script", "deploy") => {
                let script_client = match base_override {
                    Some(base) => GoogleAppsScriptClient::with_base_url(base),
                    None => GoogleAppsScriptClient::new(),
                };
                let script_id = payload
                    .get("script_id")
                    .and_then(Value::as_str)
                    .unwrap_or(&proposal.target_resource);
                let desc = payload
                    .get("description")
                    .and_then(Value::as_str)
                    .filter(|s| !s.trim().is_empty());
                let version_number = script_client
                    .create_version(access_token, script_id, desc)
                    .await
                    .map_err(|e| e.to_string())?;
                let dep = script_client
                    .create_deployment(access_token, script_id, version_number, desc)
                    .await
                    .map_err(|e| e.to_string())?;
                serde_json::to_value(dep).map_err(|e| e.to_string())
            }
            ("apps_script", "run_function" | "run") => {
                let script_client = match base_override {
                    Some(base) => GoogleAppsScriptClient::with_base_url(base),
                    None => GoogleAppsScriptClient::new(),
                };
                let script_id = payload
                    .get("script_id")
                    .and_then(Value::as_str)
                    .unwrap_or(&proposal.target_resource);
                let function_name = payload
                    .get("function_name")
                    .or_else(|| payload.get("function"))
                    .and_then(Value::as_str)
                    .ok_or_else(|| "Apps Script run payload missing 'function_name'".to_string())?;
                let parameters = payload
                    .get("parameters")
                    .and_then(Value::as_array)
                    .cloned();
                let dev_mode = payload
                    .get("dev_mode")
                    .and_then(Value::as_bool)
                    .unwrap_or(false);
                script_client
                    .run_function(access_token, script_id, function_name, parameters, dev_mode)
                    .await
                    .map_err(|e| e.to_string())
            }
            ("forms", "create_form" | "create") => {
                let forms_client = match base_override {
                    Some(base) => GoogleFormsClient::with_base_url(base),
                    None => GoogleFormsClient::new(),
                };
                let title = payload
                    .get("title")
                    .and_then(Value::as_str)
                    .unwrap_or(&proposal.title);
                let doc_title = payload.get("document_title").and_then(Value::as_str);
                let created = forms_client
                    .create_form(access_token, title, doc_title)
                    .await
                    .map_err(|e| e.to_string())?;
                let batch_updates = payload
                    .get("requests")
                    .cloned()
                    .or_else(|| payload.get("batch_requests").cloned());
                let batch_result = if let Some(reqs) = batch_updates {
                    Some(
                        forms_client
                            .batch_update_form(access_token, &created.form_id, reqs)
                            .await
                            .map_err(|e| e.to_string())?,
                    )
                } else {
                    None
                };
                Ok(json!({
                    "form": created,
                    "batch_update": batch_result
                }))
            }
            ("chat", "create_message" | "send") => {
                let acct = self
                    .resolve_account(Some(&proposal.account_id))
                    .await
                    .ok_or_else(|| {
                        format!(
                            "Google account '{}' not found for Google Chat",
                            proposal.account_id
                        )
                    })?;
                check_chat_eligibility(acct.kind).map_err(|e| e.to_string())?;
                let chat_client = match base_override {
                    Some(base) => GoogleChatClient::with_base_url(base),
                    None => GoogleChatClient::new(),
                };
                let space_name = payload
                    .get("space_name")
                    .and_then(Value::as_str)
                    .unwrap_or(&proposal.target_resource);
                let text = payload
                    .get("text")
                    .and_then(Value::as_str)
                    .unwrap_or(&proposal.summary);
                let msg = chat_client
                    .create_message(access_token, space_name, text)
                    .await
                    .map_err(|e| e.to_string())?;
                serde_json::to_value(msg).map_err(|e| e.to_string())
            }
            ("chat", "webhook") => {
                let webhook_url = payload
                    .get("webhook_url")
                    .and_then(Value::as_str)
                    .ok_or_else(|| "Google Chat webhook payload missing 'webhook_url'".to_string())?;
                let text = payload
                    .get("text")
                    .and_then(Value::as_str)
                    .unwrap_or(&proposal.summary);
                let allow_local = payload
                    .get("allow_local_test_webhook")
                    .and_then(Value::as_bool)
                    .unwrap_or(false)
                    || base_override.is_some();
                if allow_local {
                    GoogleChatClient::send_incoming_webhook_with_allow_local(
                        webhook_url,
                        text,
                        true,
                    )
                    .await
                    .map_err(|e| e.to_string())
                } else {
                    GoogleChatClient::send_incoming_webhook(webhook_url, text)
                        .await
                        .map_err(|e| e.to_string())
                }
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

    /// Resolves NotebookLM Wiz credentials from GV2 multi-account vault or legacy GV1 vault.
    pub async fn resolve_wiz_credentials(&self) -> Option<GoogleSessionCredentials> {
        {
            let vault = self.vault.read().await;
            if let Some(acct) = vault.accounts().iter().find(|a| {
                a.has_nlm_bridge
                    || a.psid.as_deref().is_some_and(|s| !s.is_empty())
                    || a.sapisid.as_deref().is_some_and(|s| !s.is_empty())
            }) {
                return Some(acct.to_legacy_credentials());
            }
        }
        let legacy_path = self.data_dir.parent().map_or_else(
            GoogleSessionVault::default_vault_path,
            |p| p.join("google_sessions.vault"),
        );
        GoogleSessionVault::load(&legacy_path)
            .ok()
            .flatten()
            .filter(|c| c.is_configured())
    }

    /// Persists a pulse event (deduped), broadcasts it over SSE, and optionally spawns
    /// an external Webhook/Telegram alert outside Quiet Hours and cooldown windows.
    pub async fn emit_pulse_event(&self, event: PulseEventDto) -> bool {
        let inserted = match self.pulse.push_event(event.clone()) {
            Ok(flag) => flag,
            Err(err) => {
                warn!(
                    target: "souls_server::google_hub",
                    "Failed persisting pulse event '{}': {err}",
                    event.id
                );
                false
            }
        };

        if !inserted {
            return false;
        }

        let _ = self.pulse_tx.send(event.clone());

        let now_ts = Utc::now().timestamp();
        let cfg = self.pulse.config();
        let local_hour = Local::now().hour() as u8;
        let quiet_active = is_within_quiet_hours(
            cfg.quiet_hours_enabled,
            local_hour,
            cfg.quiet_hours_start,
            cfg.quiet_hours_end,
        );
        let cat_key = pulse_category_key(event.category);
        let cooldown_secs = (cfg.external_cooldown_mins as i64) * 60;
        let last_sent_ts = self
            .pulse
            .runtime_state()
            .external_cooldowns
            .get(cat_key)
            .copied();

        let has_channel = cfg.webhook_configured || cfg.telegram_configured;
        if has_channel
            && should_dispatch_external(
                event.urgency,
                quiet_active,
                cooldown_secs,
                last_sent_ts,
                now_ts,
            )
            && self.pulse.can_dispatch_external(cat_key, now_ts)
        {
            let secrets = self.pulse.secrets();
            let pulse_store = Arc::clone(&self.pulse);
            let ev_clone = event.clone();
            let ev_id = event.id.clone();
            let cat = cat_key.to_string();
            tokio::spawn(async move {
                let http = reqwest::Client::builder()
                    .timeout(std::time::Duration::from_secs(15))
                    .build()
                    .unwrap_or_default();
                match dispatch_external_pulse_alert(&http, &secrets, &ev_clone, None).await {
                    Ok(channels) if !channels.is_empty() => {
                        pulse_store.mark_external_dispatched(&ev_id, &cat, Utc::now().timestamp());
                    }
                    Ok(_) => {}
                    Err(err) => {
                        warn!(
                            target: "souls_server::google_hub",
                            "External pulse alert delivery failed: {err}"
                        );
                    }
                }
            });
        }

        true
    }

    /// Builds a consolidated snapshot of the Proactive Google Pulse state.
    pub async fn build_pulse_snapshot(&self) -> PulseSnapshotDto {
        let cfg = self.pulse.config();
        let rt = self.pulse.runtime_state();
        let local_hour = Local::now().hour() as u8;
        let in_quiet_hours = is_within_quiet_hours(
            cfg.quiet_hours_enabled,
            local_hour,
            cfg.quiet_hours_start,
            cfg.quiet_hours_end,
        );
        let next_run_at = if cfg.enabled {
            rt.last_run_at
                .map(|ts| ts.saturating_add((cfg.interval_mins as i64) * 60))
                .or_else(|| Some(Utc::now().timestamp()))
        } else {
            None
        };

        PulseSnapshotDto {
            config: cfg.clone(),
            running: cfg.enabled,
            in_quiet_hours,
            tick_count: rt.tick_count,
            last_run_at: rt.last_run_at,
            next_run_at,
            unacked_count: self.pulse.unacked_count(),
            recent_events: self.pulse.list_events(30),
            monitors: self.pulse.monitor_health_list(),
            sync_links_count: self.pulse.list_sync_links().len(),
        }
    }

    /// Executes one multi-cadence Proactive Google Pulse tick across all 5 monitors
    /// (`oauth_health`, `commute`, `gmail_vip`, `forms`, `drive_sync`) with isolated backoff.
    pub async fn run_proactive_pulse_cycle(&self, force: bool) -> PulseSnapshotDto {
        let now_ts = Utc::now().timestamp();
        let tick = self.pulse.begin_tick(now_ts);
        let cfg = self.pulse.config();

        // 1. OAuth Health Monitor (1x base cadence)
        if cfg.monitor_oauth_health
            && self
                .pulse
                .should_run_monitor("oauth_health", tick, 1, force, now_ts)
        {
            self.run_monitor_oauth_health(&cfg, now_ts).await;
        }

        // 2. Commute Monitor (1x base cadence; zero network calls if unconfigured or no nearby events)
        if cfg.monitor_commute
            && self
                .pulse
                .should_run_monitor("commute", tick, 1, force, now_ts)
        {
            self.run_monitor_commute(&cfg, now_ts).await;
        }

        // 3. Gmail VIP Monitor (2x default cadence = 10 min)
        if cfg.monitor_gmail_vip
            && self.pulse.should_run_monitor(
                "gmail_vip",
                tick,
                cfg.gmail_tick_multiplier,
                force,
                now_ts,
            )
        {
            self.run_monitor_gmail_vip(&cfg, now_ts).await;
        }

        // 4. Google Forms Monitor (3x default cadence = 15 min)
        if cfg.monitor_forms
            && self.pulse.should_run_monitor(
                "forms",
                tick,
                cfg.forms_sync_tick_multiplier,
                force,
                now_ts,
            )
        {
            self.run_monitor_forms(&cfg, now_ts).await;
        }

        // 5. Drive <-> NotebookLM Sync Monitor (3x default cadence = 15 min)
        if cfg.monitor_drive_sync
            && self.pulse.should_run_monitor(
                "drive_sync",
                tick,
                cfg.forms_sync_tick_multiplier,
                force,
                now_ts,
            )
        {
            self.run_monitor_drive_sync(&cfg, now_ts).await;
        }

        self.build_pulse_snapshot().await
    }

    async fn run_monitor_oauth_health(&self, cfg: &souls_protocol::google::PulseConfigDto, now_ts: i64) {
        let accounts: Vec<GoogleAccountRecord> = self.vault.read().await.accounts().to_vec();
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .unwrap_or_default();

        let mut last_err: Option<String> = None;

        for mut acct in accounts {
            let mut refreshed_ok = false;
            if let Some(exp) = acct.token_expires_at {
                if exp.saturating_sub(now_ts) <= 600 {
                    if let (Some(rt), Some(cid)) = (
                        acct.oauth_refresh_token
                            .as_deref()
                            .map(str::trim)
                            .filter(|s| !s.is_empty())
                            .map(ToString::to_string),
                        acct.client_id
                            .as_deref()
                            .map(str::trim)
                            .filter(|s| !s.is_empty())
                            .map(ToString::to_string),
                    ) {
                        let oauth_cfg = OAuth2Config::google_default(
                            cid,
                            acct.client_secret.clone(),
                            "http://127.0.0.1:9123/api/v1/google/oauth/callback",
                            acct.scopes.clone(),
                        );
                        match refresh_access_token(&http, &oauth_cfg, &rt).await {
                            Ok(tok) => {
                                acct.oauth_access_token = Some(tok.access_token);
                                if let Some(new_rt) = tok.refresh_token {
                                    if !new_rt.trim().is_empty() {
                                        acct.oauth_refresh_token = Some(new_rt);
                                    }
                                }
                                if let Some(secs) = tok.expires_in {
                                    acct.token_expires_at = Some(now_ts.saturating_add(secs));
                                }
                                acct.updated_at = now_ts;
                                let mut vault_guard = self.vault.write().await;
                                if let Err(err) = vault_guard.upsert_account(acct.clone()) {
                                    last_err = Some(err.to_string());
                                } else {
                                    refreshed_ok = true;
                                }
                            }
                            Err(err) => {
                                let msg = err.to_string();
                                last_err = Some(msg.clone());
                                let ev = make_pulse_event(
                                    PulseCategory::OAuthExpiry,
                                    PulseUrgency::High,
                                    format!("Falha ao renovar acesso de '{}'", acct.label),
                                    format!(
                                        "A renovacao automatica da conta '{}' falhou: {}. Reconecte em Conexoes.",
                                        acct.email, msg
                                    ),
                                    Some(acct.account_id.clone()),
                                    Some(acct.account_id.clone()),
                                    Some(format!("reconnect_oauth:{}", acct.account_id)),
                                    format!("oauth_refresh_fail:{}", acct.account_id),
                                );
                                self.emit_pulse_event(ev).await;
                            }
                        }
                    } else if exp <= now_ts {
                        let ev = make_pulse_event(
                            PulseCategory::OAuthExpiry,
                            PulseUrgency::High,
                            format!("Sessao Google expirada: {}", acct.label),
                            format!(
                                "O acesso da conta '{}' expirou e nao ha refresh token salvo. Clique para reconectar.",
                                acct.email
                            ),
                            Some(acct.account_id.clone()),
                            Some(acct.account_id.clone()),
                            Some(format!("reconnect_oauth:{}", acct.account_id)),
                            format!("oauth_expired:{}", acct.account_id),
                        );
                        self.emit_pulse_event(ev).await;
                    }
                }
            }

            let has_oauth = acct.oauth_access_token.is_some() || acct.oauth_refresh_token.is_some();
            if !refreshed_ok && acct.oauth_testing_mode_warning && has_oauth {
                let near_expiry = acct
                    .token_expires_at
                    .is_some_and(|exp| exp.saturating_sub(now_ts) <= 600);
                let aged_testing = now_ts.saturating_sub(acct.updated_at) >= 5 * 86_400;
                if near_expiry || aged_testing {
                    let ev = make_pulse_event(
                        PulseCategory::OAuthExpiry,
                        PulseUrgency::High,
                        format!("Aviso de modo Testing: {}", acct.label),
                        "Seu app Google Cloud esta em modo Testing; autorizacoes expiram a cada 7 dias. Renove ou publique o app."
                            .to_string(),
                        Some(acct.account_id.clone()),
                        Some(acct.account_id.clone()),
                        Some(format!("reconnect_oauth:{}", acct.account_id)),
                        format!("oauth_testing_mode:{}", acct.account_id),
                    );
                    self.emit_pulse_event(ev).await;
                }
            }
        }

        if let Some(err) = last_err {
            self.pulse
                .record_monitor_error("oauth_health", cfg.interval_mins, &err, now_ts);
        } else {
            self.pulse
                .record_monitor_ok("oauth_health", cfg.interval_mins, now_ts);
        }
    }

    async fn run_monitor_commute(&self, cfg: &souls_protocol::google::PulseConfigDto, now_ts: i64) {
        let acct = self.resolve_account(None).await;
        let token_opt = acct
            .as_ref()
            .and_then(|a| a.oauth_access_token.as_deref())
            .map(str::trim)
            .filter(|t| !t.is_empty());
        let origin_opt = cfg
            .commute_origin
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(ToString::to_string)
            .or_else(|| {
                std::env::var("SOULS_COMMUTE_ORIGIN")
                    .ok()
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
            });
        let api_key_opt = std::env::var("ROUTES_API_KEY")
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .or_else(|| {
                std::env::var("GOOGLE_MAPS_API_KEY")
                    .ok()
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
            });

        let (Some(token), Some(origin), Some(api_key)) = (token_opt, origin_opt, api_key_opt) else {
            self.pulse
                .record_monitor_ok("commute", cfg.interval_mins, now_ts);
            return;
        };

        let now = Utc::now();
        let time_min = now.to_rfc3339();
        let time_max = (now + ChronoDuration::minutes(90)).to_rfc3339();
        let ws = GoogleWorkspaceClient::new();

        let events = match ws.list_events(token, &time_min, &time_max).await {
            Ok(evs) => evs,
            Err(err) => {
                self.pulse.record_monitor_error(
                    "commute",
                    cfg.interval_mins,
                    &err.to_string(),
                    now_ts,
                );
                return;
            }
        };

        let geolocated: Vec<_> = events
            .into_iter()
            .filter(|ev| {
                ev.location
                    .as_deref()
                    .is_some_and(|loc| !loc.trim().is_empty())
            })
            .take(3)
            .collect();

        if geolocated.is_empty() {
            self.pulse
                .record_monitor_ok("commute", cfg.interval_mins, now_ts);
            return;
        }

        let routes_client = RoutesV2Client::new();
        let mut last_err: Option<String> = None;
        let account_id = acct.as_ref().map(|a| a.account_id.clone());

        for ev in geolocated {
            let Some(ref dest) = ev.location else {
                continue;
            };
            match routes_client.compute_route(&api_key, &origin, dest).await {
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
                        if let Ok(dep_dt) =
                            chrono::DateTime::parse_from_rfc3339(&alert.suggested_departure_rfc3339)
                        {
                            let mins_until_dep =
                                (dep_dt.with_timezone(&Utc) - now).num_minutes();
                            if mins_until_dep <= 30 {
                                let urgency = if mins_until_dep <= 10 {
                                    PulseUrgency::Critical
                                } else {
                                    PulseUrgency::High
                                };
                                let ev_dto = make_pulse_event(
                                    PulseCategory::CommuteAlert,
                                    urgency,
                                    format!("Hora de sair para '{}'", alert.event_title),
                                    alert.advisory.clone(),
                                    account_id.clone(),
                                    Some(ev.id.clone()),
                                    Some(format!("open_event:{}", ev.id)),
                                    format!("commute:{}:{}", ev.id, ev.start_rfc3339),
                                );
                                self.emit_pulse_event(ev_dto).await;
                            }
                        }
                    }
                }
                Err(err) => {
                    last_err = Some(err.to_string());
                }
            }
        }

        if let Some(err) = last_err {
            self.pulse
                .record_monitor_error("commute", cfg.interval_mins, &err, now_ts);
        } else {
            self.pulse
                .record_monitor_ok("commute", cfg.interval_mins, now_ts);
        }
    }

    async fn run_monitor_gmail_vip(&self, cfg: &souls_protocol::google::PulseConfigDto, now_ts: i64) {
        let cadence = cfg.interval_mins.saturating_mul(cfg.gmail_tick_multiplier.max(1));
        let acct = self.resolve_account(None).await;
        let Some(token) = acct
            .as_ref()
            .and_then(|a| a.oauth_access_token.as_deref())
            .map(str::trim)
            .filter(|t| !t.is_empty())
        else {
            self.pulse.record_monitor_ok("gmail_vip", cadence, now_ts);
            return;
        };

        let last_seen = self.pulse.runtime_state().gmail_last_seen_ts.unwrap_or(0);
        let ws = GoogleWorkspaceClient::new();
        let messages = match ws.list_messages(token, "is:unread in:inbox", 12).await {
            Ok(list) => list,
            Err(err) => {
                self.pulse
                    .record_monitor_error("gmail_vip", cadence, &err.to_string(), now_ts);
                return;
            }
        };

        let acct_id = acct.as_ref().map(|a| a.account_id.clone());
        for msg in &messages {
            if let Some(msg_ts) = parse_email_epoch(&msg.date) {
                if msg_ts <= last_seen {
                    continue;
                }
            }
            let from_l = msg.from.to_ascii_lowercase();
            let subj_l = msg.subject.to_ascii_lowercase();
            let is_vip_sender = cfg.vip_senders.iter().any(|vip| {
                let v = vip.trim().to_ascii_lowercase();
                !v.is_empty() && (from_l.contains(&v) || subj_l.contains(&v))
            });
            let is_priority = msg.priority_score >= 70.0 || msg.requires_reply;
            if is_vip_sender || is_priority {
                let ev = make_pulse_event(
                    PulseCategory::EmailVip,
                    PulseUrgency::High,
                    format!("Email prioritario de {}", msg.from),
                    format!("{} — {}", msg.subject, msg.snippet),
                    acct_id.clone(),
                    Some(msg.id.clone()),
                    Some(format!("gmail:{}", msg.id)),
                    format!("gmail_vip:{}", msg.id),
                );
                self.emit_pulse_event(ev).await;
            }
        }

        self.pulse.set_gmail_cursor(now_ts);
        self.pulse.record_monitor_ok("gmail_vip", cadence, now_ts);
    }

    async fn run_monitor_forms(&self, cfg: &souls_protocol::google::PulseConfigDto, now_ts: i64) {
        let cadence = cfg
            .interval_mins
            .saturating_mul(cfg.forms_sync_tick_multiplier.max(1));
        if cfg.monitored_form_ids.is_empty() {
            self.pulse.record_monitor_ok("forms", cadence, now_ts);
            return;
        }

        let acct = self.resolve_account(None).await;
        let Some(token) = acct
            .as_ref()
            .and_then(|a| a.oauth_access_token.as_deref())
            .map(str::trim)
            .filter(|t| !t.is_empty())
        else {
            self.pulse.record_monitor_ok("forms", cadence, now_ts);
            return;
        };

        let forms_client = GoogleFormsClient::new();
        let acct_id = acct.as_ref().map(|a| a.account_id.clone());
        let mut last_err: Option<String> = None;

        for form_id in &cfg.monitored_form_ids {
            let fid = form_id.trim();
            if fid.is_empty() {
                continue;
            }
            match forms_client.list_responses(token, fid).await {
                Ok(responses) => {
                    let ids: Vec<String> =
                        responses.iter().map(|r| r.response_id.clone()).collect();
                    let unseen = self.pulse.seen_form_responses_diff(fid, &ids);
                    if !unseen.is_empty() {
                        let ev = make_pulse_event(
                            PulseCategory::FormResponse,
                            PulseUrgency::Medium,
                            format!("{} nova(s) resposta(s) no formulario", unseen.len()),
                            format!(
                                "O formulario '{fid}' recebeu {} nova(s) resposta(s) (total {}).",
                                unseen.len(),
                                responses.len()
                            ),
                            acct_id.clone(),
                            Some(fid.to_string()),
                            Some(format!("view_form:{fid}")),
                            format!("form:{fid}:{}", unseen.join(",")),
                        );
                        self.emit_pulse_event(ev).await;
                    }
                }
                Err(err) => {
                    last_err = Some(err.to_string());
                }
            }
        }

        if let Some(err) = last_err {
            self.pulse
                .record_monitor_error("forms", cadence, &err, now_ts);
        } else {
            self.pulse.record_monitor_ok("forms", cadence, now_ts);
        }
    }

    async fn run_monitor_drive_sync(&self, cfg: &souls_protocol::google::PulseConfigDto, now_ts: i64) {
        let cadence = cfg
            .interval_mins
            .saturating_mul(cfg.forms_sync_tick_multiplier.max(1));
        let links = self.pulse.list_sync_links();
        if links.is_empty() {
            self.pulse.record_monitor_ok("drive_sync", cadence, now_ts);
            return;
        }

        let nlm = NlmCliBridge::new();
        let wiz_creds = self.resolve_wiz_credentials().await;
        let mut last_err: Option<String> = None;

        for mut link in links {
            let mut is_stale = false;
            let mut checked = false;

            if wiz_creds.is_some() {
                if let Some(ref creds) = wiz_creds {
                    let client = WizRpcClient::new(creds.clone());
                    let rpc = WizRpcPayload::new(WizRpcId::CheckFreshness, json!([link.source_id]));
                    match client.execute(&rpc).await {
                        Ok(resp) => {
                            checked = true;
                            is_stale = resp
                                .payload
                                .as_ref()
                                .is_some_and(wiz_freshness_indicates_stale);
                        }
                        Err(err) => {
                            last_err = Some(err.to_string());
                        }
                    }
                }
            } else {
                match nlm.list_stale_sources(&link.notebook_id).await {
                    Ok(stale_sources) => {
                        checked = true;
                        is_stale = stale_sources.iter().any(|s| {
                            s.source_id == link.source_id
                                || s.is_stale_drive == Some(true)
                                || s.url
                                    .as_deref()
                                    .is_some_and(|u| u.contains(&link.drive_file_id))
                        });
                    }
                    Err(err) => {
                        let msg = err.to_string();
                        if !msg.contains("not installed") {
                            last_err = Some(msg);
                        }
                    }
                }
            }

            if !checked {
                continue;
            }

            link.last_checked_at = Some(now_ts);
            if is_stale {
                if link.auto_sync_enabled {
                    let sync_res = self.execute_link_sync(&nlm, wiz_creds.as_ref(), &link).await;
                    match sync_res {
                        Ok(_) => {
                            link.last_synced_at = Some(now_ts);
                            link.last_status = "auto_synced".to_string();
                            let _ = self.pulse.upsert_sync_link(link.clone());
                            let ev = make_pulse_event(
                                PulseCategory::DriveSync,
                                PulseUrgency::Low,
                                format!("Caderno sincronizado: {}", link.drive_file_name),
                                format!(
                                    "O arquivo '{}' foi atualizado e sincronizado automaticamente com '{}'.",
                                    link.drive_file_name, link.notebook_title
                                ),
                                None,
                                Some(link.drive_file_id.clone()),
                                None,
                                format!("drive_autosync:{}:{}", link.id, now_ts / 300),
                            );
                            self.emit_pulse_event(ev).await;
                        }
                        Err(err) => {
                            link.last_status = format!("sync_error: {err}");
                            let _ = self.pulse.upsert_sync_link(link);
                            last_err = Some(err);
                        }
                    }
                } else {
                    link.last_status = "stale_pending_user".to_string();
                    let _ = self.pulse.upsert_sync_link(link.clone());
                    let ev = make_pulse_event(
                        PulseCategory::DriveSync,
                        PulseUrgency::Medium,
                        format!("Arquivo alterado no Drive: {}", link.drive_file_name),
                        format!(
                            "A fonte vinculada ao caderno '{}' mudou no Google Drive. Clique para sincronizar.",
                            link.notebook_title
                        ),
                        None,
                        Some(link.drive_file_id.clone()),
                        Some(format!("sync_notebook:{}", link.notebook_id)),
                        format!("drive_stale:{}:{}", link.id, link.notebook_id),
                    );
                    self.emit_pulse_event(ev).await;
                }
            } else {
                link.last_status = "up_to_date".to_string();
                let _ = self.pulse.upsert_sync_link(link);
            }
        }

        if let Some(err) = last_err {
            self.pulse
                .record_monitor_error("drive_sync", cadence, &err, now_ts);
        } else {
            self.pulse
                .record_monitor_ok("drive_sync", cadence, now_ts);
        }
    }

    async fn execute_link_sync(
        &self,
        nlm: &NlmCliBridge,
        wiz_creds: Option<&GoogleSessionCredentials>,
        link: &DriveNotebookSyncLinkDto,
    ) -> Result<Value, String> {
        if let Ok(val) = nlm.sync_sources(&link.notebook_id).await {
            return Ok(json!({ "backend": "nlm_cli", "result": val }));
        }
        if let Some(creds) = wiz_creds {
            let client = WizRpcClient::new(creds.clone());
            let rpc = WizRpcPayload::new(WizRpcId::SyncDriveSource, json!([link.source_id]));
            let resp = client.execute(&rpc).await.map_err(|e| e.to_string())?;
            return Ok(json!({ "backend": "wiz_rpc", "payload": resp.payload }));
        }
        Err("Neither nlm CLI nor NotebookLM Wiz cookies are configured.".to_string())
    }

    /// Triggers automatic NotebookLM synchronization for any `DriveNotebookSyncLinkDto`
    /// tied to `drive_file_id` with `auto_sync_enabled == true`.
    pub async fn trigger_drive_notebook_autosync(&self, drive_file_id: &str) -> Vec<Value> {
        let links: Vec<DriveNotebookSyncLinkDto> = self
            .pulse
            .sync_links_for_drive_file(drive_file_id)
            .into_iter()
            .filter(|l| l.auto_sync_enabled)
            .collect();

        if links.is_empty() {
            return Vec::new();
        }

        let now_ts = Utc::now().timestamp();
        let nlm = NlmCliBridge::new();
        let wiz_creds = self.resolve_wiz_credentials().await;
        let mut outcomes = Vec::new();

        for mut link in links {
            link.last_checked_at = Some(now_ts);
            match self
                .execute_link_sync(&nlm, wiz_creds.as_ref(), &link)
                .await
            {
                Ok(detail) => {
                    link.last_synced_at = Some(now_ts);
                    link.last_status = "auto_synced".to_string();
                    let _ = self.pulse.upsert_sync_link(link.clone());
                    let ev = make_pulse_event(
                        PulseCategory::DriveSync,
                        PulseUrgency::Low,
                        format!("Caderno sincronizado: {}", link.drive_file_name),
                        format!(
                            "Alteracao enviada para '{}' sincronizou automaticamente o caderno '{}'.",
                            link.drive_file_name, link.notebook_title
                        ),
                        None,
                        Some(link.drive_file_id.clone()),
                        None,
                        format!("drive_postpush:{}:{}", link.id, now_ts),
                    );
                    self.emit_pulse_event(ev).await;
                    outcomes.push(json!({
                        "link_id": link.id,
                        "notebook_id": link.notebook_id,
                        "status": "synced",
                        "detail": detail,
                    }));
                }
                Err(err) => {
                    link.last_status = format!("autosync_pending: {err}");
                    let _ = self.pulse.upsert_sync_link(link.clone());
                    let ev = make_pulse_event(
                        PulseCategory::DriveSync,
                        PulseUrgency::Medium,
                        format!("Sincronizacao pendente: {}", link.drive_file_name),
                        format!(
                            "Arquivo salvo no Google Drive, mas a sincronizacao com '{}' precisa de atencao: {}",
                            link.notebook_title, err
                        ),
                        None,
                        Some(link.drive_file_id.clone()),
                        Some(format!("sync_notebook:{}", link.notebook_id)),
                        format!("drive_postpush_err:{}:{}", link.id, now_ts / 300),
                    );
                    self.emit_pulse_event(ev).await;
                    outcomes.push(json!({
                        "link_id": link.id,
                        "notebook_id": link.notebook_id,
                        "status": "deferred",
                        "error": err,
                    }));
                }
            }
        }

        outcomes
    }

    /// Executes the 2-tier MinionS deep-analysis cascade (`Tier 0 BM25 + SHA-256 disk cache`,
    /// escalating to `Tier 1 local SLM` when an installed `.gguf` + `llama-completion` exist).
    pub async fn run_google_minions_analysis(
        &self,
        req: GoogleMinionsRequest,
    ) -> Result<MinionGoogleAnalysisDto, (StatusCode, String)> {
        let start = Instant::now();
        let mode = req.mode.unwrap_or(MinionAnalyzeMode::Auto);
        let top_k = req.top_k.unwrap_or(8).clamp(1, 20);
        let query = req.query.trim().to_string();
        if query.is_empty() {
            return Err((
                StatusCode::BAD_REQUEST,
                "Campo 'query' e obrigatorio para analise MinionS.".to_string(),
            ));
        }

        let (source_ref, raw_bytes) = self.resolve_minions_corpus(&req).await?;

        let data_dir = self.data_dir.clone();
        let source_kind = req.source_kind;
        let source_ref_for_prep = source_ref.clone();
        let query_for_prep = query.clone();
        let prep = tokio::task::spawn_blocking(move || {
            prepare_google_corpus_tier0(
                &data_dir,
                source_kind,
                &source_ref_for_prep,
                &raw_bytes,
                &query_for_prep,
                top_k,
            )
        })
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Falha interna ao preparar corpus MinionS: {e}"),
            )
        })?
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

        if mode == MinionAnalyzeMode::FastDeterministic {
            return Ok(MinionGoogleAnalysisDto {
                source_kind: prep.source_kind,
                source_ref: prep.source_ref,
                query: prep.query,
                mode_requested: mode,
                mode_used: "tier0_deterministic".to_string(),
                executed_device: "CPU Host (Tier 0 BM25)".to_string(),
                cache_hit: prep.cache_hit,
                raw_chars: prep.raw_chars,
                retained_chars: prep.retained_chars,
                estimated_token_savings_pct: prep.estimated_token_savings_pct,
                chunks_total: prep.chunks_total,
                chunks_selected: prep.chunks_selected,
                executive_summary: prep.executive_summary,
                top_passages: prep.top_passages,
                factual_claims: prep.factual_claims,
                statistical_data: prep.statistical_data,
                direct_quotes: prep.direct_quotes,
                markdown_export: prep.markdown_export,
                latency_ms: start.elapsed().as_millis() as u64,
            });
        }

        let model_opt = resolve_local_minions_model();
        let bin_opt = resolve_llama_completion_binary();

        match (model_opt, bin_opt) {
            (Some(model_path), Some(_bin_path)) => {
                let framed_context = prep.framed_context_for_slm.clone();
                let query_brief = query.clone();
                let slm_result = tokio::task::spawn_blocking(move || {
                    let orchestrator = MinionBatchOrchestrator::new().with_model_path(&model_path);
                    let task = MinionTask {
                        task_id: format!("google_minions_{}", Utc::now().timestamp()),
                        kind: MinionTaskKind::SynthesizeSection,
                        passage_text: framed_context,
                        context_brief: Some(query_brief),
                        max_tokens: 768,
                    };
                    orchestrator.execute_task(&task)
                })
                .await
                .map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Falha na thread de inferencia MinionS: {e}"),
                    )
                })?;

                match slm_result {
                    Ok(slm_out) => {
                        let executive_summary = if slm_out.executive_summary.trim().is_empty() {
                            prep.executive_summary.clone()
                        } else {
                            slm_out.executive_summary
                        };
                        let factual_claims = if slm_out.factual_claims.is_empty() {
                            prep.factual_claims.clone()
                        } else {
                            slm_out
                                .factual_claims
                                .into_iter()
                                .filter_map(|c| serde_json::to_value(c).ok())
                                .collect()
                        };
                        let statistical_data = if slm_out.statistical_data.is_empty() {
                            prep.statistical_data.clone()
                        } else {
                            slm_out
                                .statistical_data
                                .into_iter()
                                .filter_map(|s| serde_json::to_value(s).ok())
                                .collect()
                        };
                        let direct_quotes = if slm_out.direct_quotes.is_empty() {
                            prep.direct_quotes.clone()
                        } else {
                            slm_out.direct_quotes
                        };
                        let markdown_export = format!(
                            "# Analise MinionS (Tier 1 Local SLM)\n\n- **Fonte:** `{}`\n- **Pergunta:** {}\n- **Economia estimada de tokens:** {:.1}%\n\n## Resumo Executivo\n{}\n\n{}",
                            prep.source_ref,
                            prep.query,
                            prep.estimated_token_savings_pct,
                            executive_summary,
                            prep.markdown_export,
                        );
                        Ok(MinionGoogleAnalysisDto {
                            source_kind: prep.source_kind,
                            source_ref: prep.source_ref,
                            query: prep.query,
                            mode_requested: mode,
                            mode_used: "tier1_local_slm".to_string(),
                            executed_device: slm_out.executed_device.as_str().to_string(),
                            cache_hit: prep.cache_hit,
                            raw_chars: prep.raw_chars,
                            retained_chars: prep.retained_chars,
                            estimated_token_savings_pct: prep.estimated_token_savings_pct,
                            chunks_total: prep.chunks_total,
                            chunks_selected: prep.chunks_selected,
                            executive_summary,
                            top_passages: prep.top_passages,
                            factual_claims,
                            statistical_data,
                            direct_quotes,
                            markdown_export,
                            latency_ms: start.elapsed().as_millis() as u64,
                        })
                    }
                    Err(err) => {
                        if mode == MinionAnalyzeMode::ForceLocalSlm {
                            return Err((
                                StatusCode::UNPROCESSABLE_ENTITY,
                                format!("Execucao local Tier 1 MinionS falhou: {err}"),
                            ));
                        }
                        Ok(MinionGoogleAnalysisDto {
                            source_kind: prep.source_kind,
                            source_ref: prep.source_ref,
                            query: prep.query,
                            mode_requested: mode,
                            mode_used: "tier0_bm25_fallback".to_string(),
                            executed_device: "CPU Host (Tier 0 BM25)".to_string(),
                            cache_hit: prep.cache_hit,
                            raw_chars: prep.raw_chars,
                            retained_chars: prep.retained_chars,
                            estimated_token_savings_pct: prep.estimated_token_savings_pct,
                            chunks_total: prep.chunks_total,
                            chunks_selected: prep.chunks_selected,
                            executive_summary: prep.executive_summary,
                            top_passages: prep.top_passages,
                            factual_claims: prep.factual_claims,
                            statistical_data: prep.statistical_data,
                            direct_quotes: prep.direct_quotes,
                            markdown_export: prep.markdown_export,
                            latency_ms: start.elapsed().as_millis() as u64,
                        })
                    }
                }
            }
            _ => {
                if mode == MinionAnalyzeMode::ForceLocalSlm {
                    return Err((
                        StatusCode::UNPROCESSABLE_ENTITY,
                        "Nenhum modelo .gguf local ou binario llama-completion instalado. Baixe um modelo pequeno em Modelos & Hardware ou selecione o modo Automatico/Economico.".to_string(),
                    ));
                }
                Ok(MinionGoogleAnalysisDto {
                    source_kind: prep.source_kind,
                    source_ref: prep.source_ref,
                    query: prep.query,
                    mode_requested: mode,
                    mode_used: "tier0_bm25_fallback".to_string(),
                    executed_device: "CPU Host (Tier 0 BM25)".to_string(),
                    cache_hit: prep.cache_hit,
                    raw_chars: prep.raw_chars,
                    retained_chars: prep.retained_chars,
                    estimated_token_savings_pct: prep.estimated_token_savings_pct,
                    chunks_total: prep.chunks_total,
                    chunks_selected: prep.chunks_selected,
                    executive_summary: prep.executive_summary,
                    top_passages: prep.top_passages,
                    factual_claims: prep.factual_claims,
                    statistical_data: prep.statistical_data,
                    direct_quotes: prep.direct_quotes,
                    markdown_export: prep.markdown_export,
                    latency_ms: start.elapsed().as_millis() as u64,
                })
            }
        }
    }

    async fn resolve_minions_corpus(
        &self,
        req: &GoogleMinionsRequest,
    ) -> Result<(String, Vec<u8>), (StatusCode, String)> {
        let trimmed_ref = req.source_ref.trim();

        if let Some(inline) = req.text.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
            let src = if trimmed_ref.is_empty() {
                "inline_text".to_string()
            } else {
                trimmed_ref.to_string()
            };
            return Ok((src, inline.as_bytes().to_vec()));
        }

        if req.source_kind == MinionGoogleSourceKind::RawText {
            if trimmed_ref.is_empty() {
                return Err((
                    StatusCode::BAD_REQUEST,
                    "Forneca 'text' para analisar texto bruto no MinionS.".to_string(),
                ));
            }
            return Ok(("inline_text".to_string(), trimmed_ref.as_bytes().to_vec()));
        }

        if trimmed_ref.is_empty() {
            return Err((
                StatusCode::BAD_REQUEST,
                "Campo 'source_ref' e obrigatorio para fontes Drive, Gmail ou Forms.".to_string(),
            ));
        }

        let acct = self
            .resolve_account(req.account_id.as_deref())
            .await
            .ok_or_else(|| {
                (
                    StatusCode::UNAUTHORIZED,
                    "Nenhuma conta Google OAuth configurada para baixar esta fonte.".to_string(),
                )
            })?;
        let token = acct
            .oauth_access_token
            .as_deref()
            .map(str::trim)
            .filter(|t| !t.is_empty())
            .ok_or_else(|| {
                (
                    StatusCode::UNAUTHORIZED,
                    "Conta Google sem access token OAuth ativo.".to_string(),
                )
            })?;

        let base = req.api_base_override.as_deref();
        let ws = match base {
            Some(ep) => GoogleWorkspaceClient::with_base_urls(ep, ep, ep, ep, ep, ep),
            None => GoogleWorkspaceClient::new(),
        };

        match req.source_kind {
            MinionGoogleSourceKind::DriveDoc => {
                let txt = ws
                    .export_drive_file_text(token, trimmed_ref, "text/plain")
                    .await
                    .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
                Ok((trimmed_ref.to_string(), txt.into_bytes()))
            }
            MinionGoogleSourceKind::DrivePdf => {
                let bytes = ws
                    .download_drive_file_bytes(token, trimmed_ref, 25 * 1024 * 1024)
                    .await
                    .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
                Ok((trimmed_ref.to_string(), bytes))
            }
            MinionGoogleSourceKind::GmailThread => {
                let txt = ws
                    .get_gmail_thread_text(token, trimmed_ref)
                    .await
                    .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
                Ok((trimmed_ref.to_string(), txt.into_bytes()))
            }
            MinionGoogleSourceKind::GoogleForm => {
                let forms_client = match base {
                    Some(ep) => GoogleFormsClient::with_base_url(ep),
                    None => GoogleFormsClient::new(),
                };
                let form = forms_client
                    .get_form(token, trimmed_ref)
                    .await
                    .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
                let responses: Vec<GoogleFormResponseDto> = forms_client
                    .list_responses(token, trimmed_ref)
                    .await
                    .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
                let stats = compute_form_stats(&form, &responses);
                Ok((trimmed_ref.to_string(), stats.markdown_summary.into_bytes()))
            }
            MinionGoogleSourceKind::RawText => unreachable!(),
        }
    }
}

/// Request payload for `GoogleHubService::run_google_minions_analysis`.
#[derive(Debug, Clone, Deserialize)]
pub struct GoogleMinionsRequest {
    pub source_kind: MinionGoogleSourceKind,
    #[serde(default)]
    pub source_ref: String,
    pub query: String,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub mode: Option<MinionAnalyzeMode>,
    #[serde(default)]
    pub top_k: Option<usize>,
    #[serde(default)]
    pub account_id: Option<String>,
    #[serde(default)]
    pub api_base_override: Option<String>,
}

fn resolve_local_minions_model() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("SOULS_MINIONS_MODEL_PATH") {
        let trimmed = path.trim();
        if !trimmed.is_empty() {
            let candidate = PathBuf::from(trimmed);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    let root = resolve_canonical_models_dir();
    let diag = probe_hardware();
    let models = scan_local_models(&root, &diag).unwrap_or_default();
    models
        .into_iter()
        .find(|m| {
            m.format == ModelFormat::Gguf
                && !m.file_name.to_ascii_lowercase().contains("mmproj")
                && m.absolute_path.is_file()
        })
        .map(|m| m.absolute_path)
}

fn pulse_category_key(cat: PulseCategory) -> &'static str {
    match cat {
        PulseCategory::EmailVip => "email_vip",
        PulseCategory::CommuteAlert => "commute_alert",
        PulseCategory::OAuthExpiry => "oauth_expiry",
        PulseCategory::DriveSync => "drive_sync",
        PulseCategory::FormResponse => "form_response",
        PulseCategory::AppsScript => "apps_script",
        PulseCategory::System => "system",
    }
}

fn make_pulse_event(
    category: PulseCategory,
    urgency: PulseUrgency,
    title: String,
    summary: String,
    account_id: Option<String>,
    source_ref: Option<String>,
    action_hint: Option<String>,
    dedup_key: String,
) -> PulseEventDto {
    let now = Utc::now();
    PulseEventDto {
        id: format!(
            "pulse_{}_{}",
            now.timestamp_millis(),
            now.timestamp_subsec_nanos() % 100_000
        ),
        created_at: now.timestamp(),
        category,
        urgency,
        title,
        summary,
        account_id,
        source_ref,
        action_hint,
        acknowledged: false,
        delivered_to_hermes: false,
        delivered_externally: false,
        dedup_key,
    }
}

fn parse_email_epoch(date_str: &str) -> Option<i64> {
    let trimmed = date_str.trim();
    if trimmed.is_empty() {
        return None;
    }
    chrono::DateTime::parse_from_rfc2822(trimmed)
        .or_else(|_| chrono::DateTime::parse_from_rfc3339(trimmed))
        .ok()
        .map(|dt| dt.timestamp())
}

fn wiz_freshness_indicates_stale(payload: &Value) -> bool {
    let serialized = payload.to_string().to_ascii_lowercase();
    serialized.contains("stale")
        || payload.as_bool() == Some(true)
        || payload
            .as_array()
            .is_some_and(|arr| arr.iter().any(|v| v.as_bool() == Some(true)))
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
