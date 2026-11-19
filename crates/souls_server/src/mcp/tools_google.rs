//! MCP Tool Implementations for `google.*` (14 tools) and `inbox.*` (3 tools).
//!
//! Enforces BMAD governance across every MCP boundary:
//! - External Google reads mark the active MCP session as tainted in `SessionTaintTracker`.
//! - Untrusted text snippets/bodies are wrapped with randomized nonce delimiters (`wrap_untrusted_content`).
//! - Mutable operations evaluate `evaluate_action_policy` and escalate to `AgentInboxStore`
//!   with Myers Diff (`souls_ast`) and Gravity Lock when required by Tier 2 or Taint Guard.
//! - Primary account `StreamGenerate` is strictly denied (`McpDomainError::HitlDenied`).

#![forbid(unsafe_code)]

use std::path::PathBuf;

use chrono::{Duration as ChronoDuration, Utc};
use serde_json::{json, Value};

use souls_core::error::CoreError;
use souls_core::google_bridge::{
    build_secondary_stream_generate_payload, check_keep_eligibility, dispatch_gem_official_api,
    evaluate_action_policy, import_gws_credentials, quick_capture_to_obsidian, run_google_doctor,
    upcoming_birthdays, wrap_untrusted_content, GoogleAccountRecord, GoogleSessionCredentials,
    GoogleTasksClient, GoogleWorkspaceClient, NewProposalInput, NlmCliBridge, PeopleClient,
    PolicyVerdict, RoutesV2Client, WizRpcClient, WizRpcId, WizRpcPayload, YouTubeClient,
};
use souls_protocol::error::McpDomainError;
use souls_protocol::google::{
    AccountPermissionPolicy, BlastRadiusTier, BmadProposalDto, ContactCardDto, GemProfileDto,
    GoogleAccountKind, ProposalStatus,
};

use crate::routes::google::resolve_wiz_rpc_id;
use crate::state::AppState;

/// Dispatches any of the 17 Google/Inbox MCP tools. Returns `None` if `tool_name` is unrelated.
pub async fn try_dispatch_google_tool(
    state: &AppState,
    tool_name: &str,
    args: &serde_json::Map<String, Value>,
    session_id: Option<&str>,
) -> Option<Result<Value, McpDomainError>> {
    let sess = args
        .get("session_id")
        .and_then(Value::as_str)
        .or(session_id)
        .unwrap_or("mcp_default");

    let res = match tool_name {
        "google_accounts" => handle_google_accounts(state, args).await,
        "google_briefing" => handle_google_briefing(state, args).await,
        "google_gmail" => handle_google_gmail(state, args, sess).await,
        "google_calendar" => handle_google_calendar(state, args, sess).await,
        "google_drive" => handle_google_drive(state, args, sess).await,
        "google_docs_scratchpad" => handle_google_docs_scratchpad(state, args, sess).await,
        "google_sheets" => handle_google_sheets(state, args, sess).await,
        "google_slides" => handle_google_slides(state, args, sess).await,
        "google_tasks" => handle_google_tasks(state, args, sess).await,
        "google_contacts" => handle_google_contacts(state, args, sess).await,
        "google_commute" => handle_google_commute(args).await,
        "google_youtube" => handle_google_youtube(state, args, sess).await,
        "google_notebooklm" => handle_google_notebooklm(state, args, sess).await,
        "google_gems" => handle_google_gems(state, args).await,
        "inbox_list" => handle_inbox_list(state, args),
        "inbox_inspect_diff" => handle_inbox_inspect_diff(state, args),
        "inbox_propose" => handle_inbox_propose(state, args, sess),
        _ => return None,
    };

    Some(res)
}

// ============================================================================
// 1. google_accounts
// ============================================================================

async fn handle_google_accounts(
    state: &AppState,
    args: &serde_json::Map<String, Value>,
) -> Result<Value, McpDomainError> {
    let action = args
        .get("action")
        .and_then(Value::as_str)
        .unwrap_or("list")
        .to_ascii_lowercase();

    match action.as_str() {
        "list" => {
            let vault = state.google_hub.vault.read().await;
            let profiles = vault.list_profiles();
            Ok(mcp_ok(
                format!("{} Google account(s) registered in GV2 vault.", profiles.len()),
                json!({ "count": profiles.len(), "accounts": profiles }),
            ))
        }
        "doctor" => {
            let vault = state.google_hub.vault.read().await;
            let report = run_google_doctor(&state.google_hub.data_dir, &vault);
            Ok(mcp_ok(
                format!(
                    "Google Doctor status: {} ({} account(s), {} warning(s))",
                    report.overall_status,
                    report.accounts_count,
                    report.warnings.len()
                ),
                serde_json::to_value(report).unwrap_or_default(),
            ))
        }
        "import_gws" => {
            let imported = import_gws_credentials()
                .map_err(|_| McpDomainError::StructuredFailure)?;
            let Some(cfg) = imported else {
                return Ok(mcp_ok(
                    "No local gws credentials found in ~/.config/gws.",
                    json!({ "status": "not_found" }),
                ));
            };
            let email = cfg.email.clone().unwrap_or_default();
            let acct_id = args
                .get("account_id")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(ToString::to_string)
                .unwrap_or_else(|| {
                    if email.is_empty() {
                        "gws_imported".to_string()
                    } else {
                        sanitize_id(&email)
                    }
                });
            let mut vault = state.google_hub.vault.write().await;
            let mut rec = vault
                .get_account(&acct_id)
                .cloned()
                .unwrap_or_else(|| GoogleAccountRecord::new(acct_id.clone(), email.clone(), infer_kind(&email)));
            rec.client_id = cfg.client_id.or(rec.client_id);
            rec.client_secret = cfg.client_secret.or(rec.client_secret);
            rec.oauth_access_token = cfg.access_token.or(rec.oauth_access_token);
            rec.oauth_refresh_token = cfg.refresh_token.or(rec.oauth_refresh_token);
            if !cfg.scopes.is_empty() {
                rec.scopes = cfg.scopes;
            }
            if vault.accounts().is_empty() {
                rec.is_primary = true;
            }
            rec.updated_at = Utc::now().timestamp();
            let dto = rec.to_public_dto();
            vault
                .upsert_account(rec)
                .map_err(|_| McpDomainError::StructuredFailure)?;
            Ok(mcp_ok(
                format!("Imported gws credentials for account '{}'.", dto.account_id),
                json!({ "status": "imported", "account": dto }),
            ))
        }
        "set_policy" => {
            let acct_id = args
                .get("account_id")
                .and_then(Value::as_str)
                .ok_or(McpDomainError::InvalidInputParameters)?;
            let policy_val = args
                .get("policy")
                .cloned()
                .ok_or(McpDomainError::InvalidInputParameters)?;
            let policy: AccountPermissionPolicy =
                serde_json::from_value(policy_val).map_err(|_| McpDomainError::InvalidInputParameters)?;

            let mut vault = state.google_hub.vault.write().await;
            let Some(mut rec) = vault.get_account(acct_id).cloned() else {
                return Err(McpDomainError::SymbolNotFound);
            };
            rec.policy = policy;
            if let Some(wl) = args.get("write_whitelist_ids").and_then(Value::as_array) {
                rec.write_whitelist_ids = wl
                    .iter()
                    .filter_map(|v| v.as_str().map(ToString::to_string))
                    .collect();
            }
            rec.updated_at = Utc::now().timestamp();
            let dto = rec.to_public_dto();
            vault
                .upsert_account(rec)
                .map_err(|_| McpDomainError::StructuredFailure)?;
            Ok(mcp_ok(
                format!("Updated policy for '{}' to {:?}.", dto.account_id, dto.policy),
                json!({ "status": "updated", "account": dto }),
            ))
        }
        "upsert" => {
            let email = args.get("email").and_then(Value::as_str).unwrap_or("");
            let acct_id = args
                .get("account_id")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(ToString::to_string)
                .or_else(|| (!email.is_empty()).then(|| sanitize_id(email)))
                .ok_or(McpDomainError::InvalidInputParameters)?;
            let mut vault = state.google_hub.vault.write().await;
            let mut rec = vault
                .get_account(&acct_id)
                .cloned()
                .unwrap_or_else(|| GoogleAccountRecord::new(acct_id.clone(), email, infer_kind(email)));
            if let Some(lbl) = args.get("label").and_then(Value::as_str) {
                rec.label = lbl.to_string();
            }
            if let Some(kind_val) = args.get("kind").cloned() {
                rec.kind = serde_json::from_value(kind_val)
                    .map_err(|_| McpDomainError::InvalidInputParameters)?;
            }
            if let Some(pol_val) = args.get("policy").cloned() {
                rec.policy = serde_json::from_value(pol_val)
                    .map_err(|_| McpDomainError::InvalidInputParameters)?;
            }
            if let Some(is_p) = args.get("is_primary").and_then(Value::as_bool) {
                rec.is_primary = is_p;
            }
            if rec.is_primary {
                rec.secondary_web_opt_in = false;
            }
            if let Some(wl) = args.get("write_whitelist_ids").and_then(Value::as_array) {
                rec.write_whitelist_ids = wl
                    .iter()
                    .filter_map(|v| v.as_str().map(ToString::to_string))
                    .collect();
            }
            rec.updated_at = Utc::now().timestamp();
            let dto = rec.to_public_dto();
            vault
                .upsert_account(rec)
                .map_err(|_| McpDomainError::StructuredFailure)?;
            Ok(mcp_ok(
                format!("Upserted Google account '{}'.", dto.account_id),
                json!({ "status": "upserted", "account": dto }),
            ))
        }
        "delete" => {
            let acct_id = args
                .get("account_id")
                .and_then(Value::as_str)
                .ok_or(McpDomainError::InvalidInputParameters)?;
            let mut vault = state.google_hub.vault.write().await;
            let removed = vault
                .remove_account(acct_id)
                .map_err(|_| McpDomainError::StructuredFailure)?;
            Ok(mcp_ok(
                format!("Remove account '{acct_id}': {removed}"),
                json!({ "removed": removed, "account_id": acct_id }),
            ))
        }
        _ => Err(McpDomainError::InvalidInputParameters),
    }
}

// ============================================================================
// 2. google_briefing
// ============================================================================

async fn handle_google_briefing(
    state: &AppState,
    args: &serde_json::Map<String, Value>,
) -> Result<Value, McpDomainError> {
    let account_id = args.get("account_id").and_then(Value::as_str);
    let briefing = state.google_hub.build_daily_briefing(account_id).await;
    let summary = format!(
        "Daily Briefing: oauth_connected={}, events={}, focus_slots={}, priority_emails={}, pending_tasks={}, pending_inbox={}",
        briefing.oauth_connected,
        briefing.upcoming_events.len(),
        briefing.focus_slots.len(),
        briefing.priority_emails.len(),
        briefing.pending_tasks.len(),
        briefing.pending_inbox_count
    );
    Ok(mcp_ok(
        summary,
        serde_json::to_value(briefing).unwrap_or_default(),
    ))
}

// ============================================================================
// 3. google_gmail
// ============================================================================

async fn handle_google_gmail(
    state: &AppState,
    args: &serde_json::Map<String, Value>,
    session_id: &str,
) -> Result<Value, McpDomainError> {
    let action = args
        .get("action")
        .and_then(Value::as_str)
        .unwrap_or("triage")
        .to_ascii_lowercase();
    let account_id_arg = args.get("account_id").and_then(Value::as_str);
    let base_override = args.get("api_base_override").and_then(Value::as_str);
    let (acct_id, policy, token_opt) = resolve_account_ctx(state, account_id_arg).await;
    let is_tainted = state.google_hub.taint_tracker.is_tainted(session_id);

    match action.as_str() {
        "triage" => {
            let Some(token) = token_opt else {
                return Ok(unauthenticated_response("gmail_triage"));
            };
            let q = args.get("query").and_then(Value::as_str).unwrap_or("in:inbox");
            let max_res = args.get("max_results").and_then(Value::as_u64).unwrap_or(20) as usize;
            let ws = ws_client(base_override);
            let msgs = ws
                .list_messages(&token, q, max_res)
                .await
                .map_err(|_| McpDomainError::StructuredFailure)?;
            state
                .google_hub
                .taint_tracker
                .mark_tainted(session_id, "gmail:triage");
            let triaged = GoogleWorkspaceClient::triage_inbox_deterministically(&msgs);
            state.google_hub.record_audit_event(
                session_id,
                &acct_id,
                "google_gmail.triage",
                BlastRadiusTier::Tier0ReadOnly,
                true,
                "allowed",
                &format!("Triaged {} Gmail message(s)", triaged.total_scanned),
            );
            Ok(mcp_ok(
                format!(
                    "Gmail Triage complete: {} scanned, {} priority, {} newsletters (session marked tainted).",
                    triaged.total_scanned,
                    triaged.priority_messages.len(),
                    triaged.newsletter_count
                ),
                serde_json::to_value(triaged).unwrap_or_default(),
            ))
        }
        "read" => {
            let Some(token) = token_opt else {
                return Ok(unauthenticated_response("gmail_read"));
            };
            let msg_id = args
                .get("message_id")
                .and_then(Value::as_str)
                .ok_or(McpDomainError::InvalidInputParameters)?;
            let ws = ws_client(base_override);
            let framed = ws
                .get_message_framed(&token, msg_id)
                .await
                .map_err(|_| McpDomainError::StructuredFailure)?;
            state
                .google_hub
                .taint_tracker
                .mark_tainted(session_id, "gmail:read");
            Ok(mcp_ok(
                framed.clone(),
                json!({ "message_id": msg_id, "tainted": true, "framed_content": framed }),
            ))
        }
        "draft" => {
            let to = args.get("to").and_then(Value::as_str).unwrap_or("");
            let subject = args.get("subject").and_then(Value::as_str).unwrap_or("(Draft)");
            let body = args.get("body").and_then(Value::as_str).unwrap_or("");
            match evaluate_action_policy(
                BlastRadiusTier::Tier1SafeReversible,
                policy,
                is_tainted,
                false,
            ) {
                PolicyVerdict::Denied { .. } => Err(McpDomainError::HitlDenied),
                PolicyVerdict::RequireBmadInbox { .. } => {
                    let input = NewProposalInput {
                        account_id: acct_id,
                        service: "gmail".to_string(),
                        action: "create_draft".to_string(),
                        title: format!("Gmail Draft -> {to}"),
                        summary: format!("Subject: {subject}"),
                        tier: BlastRadiusTier::Tier1SafeReversible,
                        tainted_origin: is_tainted,
                        target_resource: to.to_string(),
                        before_snapshot: Some(String::new()),
                        after_snapshot: Some(format!("To: {to}\nSubject: {subject}\n\n{body}")),
                        myers_diff: None,
                        payload_json: json!({
                            "to": to,
                            "subject": subject,
                            "body": body,
                            "api_base_override": base_override
                        }),
                        ttl_secs: 600,
                    };
                    propose_and_mcp_ok(state, session_id, "google_gmail.draft", is_tainted, input)
                }
                PolicyVerdict::AllowDirect => {
                    let Some(token) = token_opt else {
                        return Ok(unauthenticated_response("gmail_draft"));
                    };
                    let ws = ws_client(base_override);
                    let resp = ws
                        .create_draft(&token, to, subject, body)
                        .await
                        .map_err(|_| McpDomainError::StructuredFailure)?;
                    Ok(mcp_ok(
                        format!("Created Gmail draft for {to}."),
                        json!({ "status": "executed", "draft": resp }),
                    ))
                }
            }
        }
        "bulk" => {
            let ids = json_str_vec(args.get("ids"));
            if ids.is_empty() {
                return Err(McpDomainError::InvalidInputParameters);
            }
            let add_labels = json_str_vec(args.get("add_labels"));
            let remove_labels = json_str_vec(args.get("remove_labels"));
            let destructive = add_labels
                .iter()
                .chain(remove_labels.iter())
                .any(|l| l.eq_ignore_ascii_case("TRASH") || l.eq_ignore_ascii_case("SPAM"));
            let tier = if destructive {
                BlastRadiusTier::Tier2ExternalDestructive
            } else {
                BlastRadiusTier::Tier1SafeReversible
            };

            match evaluate_action_policy(tier, policy, is_tainted, false) {
                PolicyVerdict::Denied { .. } => Err(McpDomainError::HitlDenied),
                PolicyVerdict::RequireBmadInbox { .. } => {
                    let input = NewProposalInput {
                        account_id: acct_id,
                        service: "gmail".to_string(),
                        action: "bulk_modify".to_string(),
                        title: format!("Gmail Bulk Modify ({} msgs)", ids.len()),
                        summary: format!("+{add_labels:?} -{remove_labels:?}"),
                        tier,
                        tainted_origin: is_tainted,
                        target_resource: format!("{} message(s)", ids.len()),
                        before_snapshot: Some(ids.join("\n")),
                        after_snapshot: Some(format!(
                            "ids: {}\n+labels: {add_labels:?}\n-labels: {remove_labels:?}",
                            ids.join(", ")
                        )),
                        myers_diff: None,
                        payload_json: json!({
                            "ids": ids,
                            "add_labels": add_labels,
                            "remove_labels": remove_labels,
                            "api_base_override": base_override
                        }),
                        ttl_secs: 600,
                    };
                    propose_and_mcp_ok(state, session_id, "google_gmail.bulk", is_tainted, input)
                }
                PolicyVerdict::AllowDirect => {
                    let Some(token) = token_opt else {
                        return Ok(unauthenticated_response("gmail_bulk"));
                    };
                    let ws = ws_client(base_override);
                    let resp = ws
                        .bulk_modify_labels(&token, &ids, &add_labels, &remove_labels)
                        .await
                        .map_err(|_| McpDomainError::StructuredFailure)?;
                    Ok(mcp_ok(
                        format!("Bulk modified {} Gmail message(s).", ids.len()),
                        json!({ "status": "executed", "result": resp }),
                    ))
                }
            }
        }
        "send" => {
            let to = args
                .get("to")
                .and_then(Value::as_str)
                .ok_or(McpDomainError::InvalidInputParameters)?;
            let subject = args.get("subject").and_then(Value::as_str).unwrap_or("(No Subject)");
            let body = args.get("body").and_then(Value::as_str).unwrap_or("");
            if matches!(policy, AccountPermissionPolicy::ReadOnly) {
                return Err(McpDomainError::HitlDenied);
            }
            let input = NewProposalInput {
                account_id: acct_id,
                service: "gmail".to_string(),
                action: "send_message".to_string(),
                title: format!("Send Email -> {to}"),
                summary: format!("Subject: {subject}"),
                tier: BlastRadiusTier::Tier2ExternalDestructive,
                tainted_origin: is_tainted,
                target_resource: to.to_string(),
                before_snapshot: Some(String::new()),
                after_snapshot: Some(format!("To: {to}\nSubject: {subject}\n\n{body}")),
                myers_diff: None,
                payload_json: json!({
                    "to": to,
                    "subject": subject,
                    "body": body,
                    "api_base_override": base_override
                }),
                ttl_secs: 600,
            };
            propose_and_mcp_ok(state, session_id, "google_gmail.send", is_tainted, input)
        }
        _ => Err(McpDomainError::InvalidInputParameters),
    }
}

// ============================================================================
// 4. google_calendar
// ============================================================================

async fn handle_google_calendar(
    state: &AppState,
    args: &serde_json::Map<String, Value>,
    session_id: &str,
) -> Result<Value, McpDomainError> {
    let action = args
        .get("action")
        .and_then(Value::as_str)
        .unwrap_or("list")
        .to_ascii_lowercase();
    let account_id_arg = args.get("account_id").and_then(Value::as_str);
    let base_override = args.get("api_base_override").and_then(Value::as_str);
    let (acct_id, policy, token_opt) = resolve_account_ctx(state, account_id_arg).await;
    let is_tainted = state.google_hub.taint_tracker.is_tainted(session_id);

    let now = Utc::now();
    match action.as_str() {
        "list" => {
            let Some(token) = token_opt else {
                return Ok(unauthenticated_response("calendar_list"));
            };
            let time_min = args
                .get("time_min")
                .and_then(Value::as_str)
                .map(ToString::to_string)
                .unwrap_or_else(|| now.to_rfc3339());
            let time_max = args
                .get("time_max")
                .and_then(Value::as_str)
                .map(ToString::to_string)
                .unwrap_or_else(|| (now + ChronoDuration::days(7)).to_rfc3339());
            let ws = ws_client(base_override);
            let events = ws
                .list_events(&token, &time_min, &time_max)
                .await
                .map_err(|_| McpDomainError::StructuredFailure)?;
            state
                .google_hub
                .taint_tracker
                .mark_tainted(session_id, "calendar:list");
            Ok(mcp_ok(
                format!("Retrieved {} Calendar event(s).", events.len()),
                json!({ "count": events.len(), "events": events }),
            ))
        }
        "focus_slots" => {
            let work_start = args
                .get("work_start")
                .and_then(Value::as_str)
                .map(ToString::to_string)
                .unwrap_or_else(|| now.to_rfc3339());
            let work_end = args
                .get("work_end")
                .and_then(Value::as_str)
                .map(ToString::to_string)
                .unwrap_or_else(|| (now + ChronoDuration::hours(8)).to_rfc3339());
            let min_mins = args
                .get("min_duration_mins")
                .and_then(Value::as_u64)
                .unwrap_or(60) as u32;
            let events = if let Some(token) = token_opt {
                ws_client(base_override)
                    .list_events(&token, &work_start, &work_end)
                    .await
                    .unwrap_or_default()
            } else {
                Vec::new()
            };
            let slots = GoogleWorkspaceClient::find_deep_focus_slots(
                &events,
                &work_start,
                &work_end,
                min_mins,
            );
            Ok(mcp_ok(
                format!("Found {} deep-focus slot(s).", slots.len()),
                json!({ "count": slots.len(), "focus_slots": slots }),
            ))
        }
        "create" => {
            let summary = args
                .get("summary")
                .and_then(Value::as_str)
                .ok_or(McpDomainError::InvalidInputParameters)?;
            let start = args
                .get("start_rfc3339")
                .and_then(Value::as_str)
                .ok_or(McpDomainError::InvalidInputParameters)?;
            let end = args
                .get("end_rfc3339")
                .and_then(Value::as_str)
                .ok_or(McpDomainError::InvalidInputParameters)?;
            let location = args.get("location").and_then(Value::as_str);
            let attendees = json_str_vec(args.get("attendees"));
            let send_updates = args
                .get("send_updates")
                .and_then(Value::as_str)
                .unwrap_or("none");

            let tier = if send_updates.eq_ignore_ascii_case("none") {
                BlastRadiusTier::Tier1SafeReversible
            } else {
                BlastRadiusTier::Tier2ExternalDestructive
            };

            match evaluate_action_policy(tier, policy, is_tainted, false) {
                PolicyVerdict::Denied { .. } => Err(McpDomainError::HitlDenied),
                PolicyVerdict::RequireBmadInbox { .. } => {
                    let input = NewProposalInput {
                        account_id: acct_id,
                        service: "calendar".to_string(),
                        action: "create_event".to_string(),
                        title: format!("Calendar Event: {summary}"),
                        summary: format!("{start} -> {end} (sendUpdates={send_updates})"),
                        tier,
                        tainted_origin: is_tainted,
                        target_resource: "primary".to_string(),
                        before_snapshot: Some(String::new()),
                        after_snapshot: Some(format!(
                            "Summary: {summary}\nStart: {start}\nEnd: {end}\nAttendees: {attendees:?}\nsendUpdates: {send_updates}"
                        )),
                        myers_diff: None,
                        payload_json: json!({
                            "summary": summary,
                            "start_rfc3339": start,
                            "end_rfc3339": end,
                            "location": location,
                            "attendees": attendees,
                            "send_updates": send_updates,
                            "api_base_override": base_override
                        }),
                        ttl_secs: 600,
                    };
                    propose_and_mcp_ok(state, session_id, "google_calendar.create", is_tainted, input)
                }
                PolicyVerdict::AllowDirect => {
                    let Some(token) = token_opt else {
                        return Ok(unauthenticated_response("calendar_create"));
                    };
                    let ev = ws_client(base_override)
                        .create_event(&token, summary, start, end, location, &attendees, Some(send_updates))
                        .await
                        .map_err(|_| McpDomainError::StructuredFailure)?;
                    Ok(mcp_ok(
                        format!("Created Calendar event '{summary}'."),
                        json!({ "status": "executed", "event": ev }),
                    ))
                }
            }
        }
        "delete" => {
            let event_id = args
                .get("event_id")
                .and_then(Value::as_str)
                .ok_or(McpDomainError::InvalidInputParameters)?;
            if matches!(policy, AccountPermissionPolicy::ReadOnly) {
                return Err(McpDomainError::HitlDenied);
            }
            let input = NewProposalInput {
                account_id: acct_id,
                service: "calendar".to_string(),
                action: "delete_event".to_string(),
                title: format!("Delete Calendar Event: {event_id}"),
                summary: format!("Remove event '{event_id}'"),
                tier: BlastRadiusTier::Tier2ExternalDestructive,
                tainted_origin: is_tainted,
                target_resource: event_id.to_string(),
                before_snapshot: Some(format!("event_id: {event_id}")),
                after_snapshot: Some("(deleted)".to_string()),
                myers_diff: None,
                payload_json: json!({
                    "event_id": event_id,
                    "api_base_override": base_override
                }),
                ttl_secs: 600,
            };
            propose_and_mcp_ok(state, session_id, "google_calendar.delete", is_tainted, input)
        }
        _ => Err(McpDomainError::InvalidInputParameters),
    }
}

// ============================================================================
// 5. google_drive
// ============================================================================

async fn handle_google_drive(
    state: &AppState,
    args: &serde_json::Map<String, Value>,
    session_id: &str,
) -> Result<Value, McpDomainError> {
    let action = args
        .get("action")
        .and_then(Value::as_str)
        .unwrap_or("search")
        .to_ascii_lowercase();
    let account_id_arg = args.get("account_id").and_then(Value::as_str);
    let base_override = args.get("api_base_override").and_then(Value::as_str);
    let (acct_id, policy, token_opt) = resolve_account_ctx(state, account_id_arg).await;
    let is_tainted = state.google_hub.taint_tracker.is_tainted(session_id);

    match action.as_str() {
        "search" => {
            let Some(token) = token_opt else {
                return Ok(unauthenticated_response("drive_search"));
            };
            let q = args
                .get("query")
                .and_then(Value::as_str)
                .unwrap_or("trashed = false");
            let page_size = args.get("page_size").and_then(Value::as_u64).unwrap_or(20) as usize;
            let files = ws_client(base_override)
                .search_files(&token, q, page_size)
                .await
                .map_err(|_| McpDomainError::StructuredFailure)?;
            state
                .google_hub
                .taint_tracker
                .mark_tainted(session_id, "drive:search");
            Ok(mcp_ok(
                format!("Found {} Drive file(s).", files.len()),
                json!({ "count": files.len(), "files": files }),
            ))
        }
        "metadata" => {
            let Some(token) = token_opt else {
                return Ok(unauthenticated_response("drive_metadata"));
            };
            let file_id = args
                .get("file_id")
                .and_then(Value::as_str)
                .ok_or(McpDomainError::InvalidInputParameters)?;
            let meta = ws_client(base_override)
                .get_file_metadata(&token, file_id)
                .await
                .map_err(|_| McpDomainError::StructuredFailure)?;
            state
                .google_hub
                .taint_tracker
                .mark_tainted(session_id, "drive:metadata");
            Ok(mcp_ok(
                format!("Drive metadata for '{}': {}", meta.name, meta.mime_type),
                serde_json::to_value(meta).unwrap_or_default(),
            ))
        }
        "download_sandbox" => {
            let Some(token) = token_opt else {
                return Ok(unauthenticated_response("drive_download"));
            };
            let file_id = args
                .get("file_id")
                .and_then(Value::as_str)
                .ok_or(McpDomainError::InvalidInputParameters)?;
            let filename = args
                .get("filename")
                .and_then(Value::as_str)
                .unwrap_or("drive_download.bin");
            let exchange_dir = state.google_hub.data_dir.join("exchange");
            let path = ws_client(base_override)
                .download_to_sandbox(&token, file_id, filename, &exchange_dir)
                .await
                .map_err(|e| match e {
                    CoreError::RefsPathLeakViolation(_) => McpDomainError::RefsPathLeakViolation,
                    _ => McpDomainError::StructuredFailure,
                })?;
            state
                .google_hub
                .taint_tracker
                .mark_tainted(session_id, "drive:download");
            Ok(mcp_ok(
                format!("Downloaded Drive file into sandbox: {}", path.display()),
                json!({ "status": "downloaded", "sandbox_path": path.display().to_string() }),
            ))
        }
        "share" | "delete" => {
            let file_id = args
                .get("file_id")
                .and_then(Value::as_str)
                .ok_or(McpDomainError::InvalidInputParameters)?;
            if matches!(policy, AccountPermissionPolicy::ReadOnly) {
                return Err(McpDomainError::HitlDenied);
            }
            let act_norm = if action == "delete" {
                "delete_file"
            } else {
                "update_permissions"
            };
            let after_text = if act_norm == "delete_file" {
                format!("DELETE Drive file '{file_id}'")
            } else {
                format!(
                    "SHARE Drive file '{file_id}' -> role={:?}, email={:?}",
                    args.get("role").and_then(Value::as_str),
                    args.get("email_address").and_then(Value::as_str)
                )
            };
            let input = NewProposalInput {
                account_id: acct_id,
                service: "drive".to_string(),
                action: act_norm.to_string(),
                title: format!("Drive {act_norm}: {file_id}"),
                summary: after_text.clone(),
                tier: BlastRadiusTier::Tier2ExternalDestructive,
                tainted_origin: is_tainted,
                target_resource: file_id.to_string(),
                before_snapshot: Some(format!("file_id: {file_id}")),
                after_snapshot: Some(after_text),
                myers_diff: None,
                payload_json: json!({
                    "file_id": file_id,
                    "role": args.get("role").and_then(Value::as_str),
                    "perm_type": args.get("perm_type").and_then(Value::as_str),
                    "email_address": args.get("email_address").and_then(Value::as_str),
                    "api_base_override": base_override
                }),
                ttl_secs: 600,
            };
            propose_and_mcp_ok(state, session_id, &format!("google_drive.{act_norm}"), is_tainted, input)
        }
        _ => Err(McpDomainError::InvalidInputParameters),
    }
}

// ============================================================================
// 6. google_docs_scratchpad
// ============================================================================

async fn handle_google_docs_scratchpad(
    state: &AppState,
    args: &serde_json::Map<String, Value>,
    session_id: &str,
) -> Result<Value, McpDomainError> {
    let action = args
        .get("action")
        .and_then(Value::as_str)
        .unwrap_or("load")
        .to_ascii_lowercase();
    let doc_id = args
        .get("doc_id")
        .and_then(Value::as_str)
        .ok_or(McpDomainError::InvalidInputParameters)?;
    let title = args.get("title").and_then(Value::as_str).unwrap_or(doc_id);

    state
        .google_hub
        .scratchpads
        .load_or_create(doc_id, title)
        .map_err(|e| match e {
            CoreError::RefsPathLeakViolation(_) => McpDomainError::RefsPathLeakViolation,
            _ => McpDomainError::StructuredFailure,
        })?;

    match action.as_str() {
        "load" => {
            let doc = state
                .google_hub
                .scratchpads
                .load_or_create(doc_id, title)
                .map_err(|_| McpDomainError::StructuredFailure)?;
            let text = doc.lines.join("\n");
            Ok(mcp_ok(
                format!("Scratchpad '{}' rev {} ({} line(s)).", doc.doc_id, doc.revision, doc.lines.len()),
                json!({ "scratchpad": doc, "rendered_text": text }),
            ))
        }
        "insert" => {
            let at_line = args.get("at_line").and_then(Value::as_u64).unwrap_or(1) as usize;
            let lines = json_str_vec(args.get("lines"));
            let doc = state
                .google_hub
                .scratchpads
                .insert_lines(doc_id, at_line, lines)
                .map_err(|_| McpDomainError::InvalidInputParameters)?;
            let text = doc.lines.join("\n");
            Ok(mcp_ok(
                format!("Inserted lines at {at_line}; scratchpad rev {} ({} total lines).", doc.revision, doc.lines.len()),
                json!({ "scratchpad": doc, "rendered_text": text }),
            ))
        }
        "replace" => {
            let start = args.get("start_line").and_then(Value::as_u64).unwrap_or(1) as usize;
            let end = args.get("end_line").and_then(Value::as_u64).unwrap_or(start as u64) as usize;
            let lines = json_str_vec(args.get("lines"));
            let doc = state
                .google_hub
                .scratchpads
                .replace_lines(doc_id, start, end, lines)
                .map_err(|_| McpDomainError::InvalidInputParameters)?;
            let text = doc.lines.join("\n");
            Ok(mcp_ok(
                format!("Replaced lines {start}..={end}; scratchpad rev {}.", doc.revision),
                json!({ "scratchpad": doc, "rendered_text": text }),
            ))
        }
        "delete" => {
            let start = args.get("start_line").and_then(Value::as_u64).unwrap_or(1) as usize;
            let end = args.get("end_line").and_then(Value::as_u64).unwrap_or(start as u64) as usize;
            let doc = state
                .google_hub
                .scratchpads
                .delete_lines(doc_id, start, end)
                .map_err(|_| McpDomainError::InvalidInputParameters)?;
            let text = doc.lines.join("\n");
            Ok(mcp_ok(
                format!("Deleted lines {start}..={end}; scratchpad rev {}.", doc.revision),
                json!({ "scratchpad": doc, "rendered_text": text }),
            ))
        }
        "render" => {
            let text = state
                .google_hub
                .scratchpads
                .render_text(doc_id)
                .map_err(|_| McpDomainError::StructuredFailure)?;
            Ok(mcp_ok(text.clone(), json!({ "doc_id": doc_id, "rendered_text": text })))
        }
        "push" => {
            let rendered = state
                .google_hub
                .scratchpads
                .render_text(doc_id)
                .map_err(|_| McpDomainError::StructuredFailure)?;
            let baseline = args
                .get("remote_baseline")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            let account_id_arg = args.get("account_id").and_then(Value::as_str);
            let base_override = args.get("api_base_override").and_then(Value::as_str);
            let acct_opt = state.google_hub.resolve_account(account_id_arg).await;
            let acct_id = acct_opt
                .as_ref()
                .map(|a| a.account_id.clone())
                .unwrap_or_else(|| "default".to_string());
            let policy = acct_opt
                .as_ref()
                .map(|a| a.policy)
                .unwrap_or(AccountPermissionPolicy::BalancedHotlHitl);
            let is_whitelisted = acct_opt
                .as_ref()
                .is_some_and(|a| a.write_whitelist_ids.iter().any(|w| w == doc_id));
            let token_opt = acct_opt
                .as_ref()
                .and_then(|a| a.oauth_access_token.as_deref())
                .map(str::trim)
                .filter(|t| !t.is_empty())
                .map(ToString::to_string);
            let is_tainted = state.google_hub.taint_tracker.is_tainted(session_id);

            match evaluate_action_policy(
                BlastRadiusTier::Tier2ExternalDestructive,
                policy,
                is_tainted,
                is_whitelisted,
            ) {
                PolicyVerdict::Denied { .. } => Err(McpDomainError::HitlDenied),
                PolicyVerdict::AllowDirect if token_opt.is_some() => {
                    let token = token_opt.unwrap_or_default();
                    let resp = ws_client(base_override)
                        .push_docs_content(&token, doc_id, &rendered)
                        .await
                        .map_err(|_| McpDomainError::StructuredFailure)?;
                    Ok(mcp_ok(
                        format!("Pushed whitelisted scratchpad '{doc_id}' directly to Google Docs."),
                        json!({ "status": "pushed_direct", "doc_id": doc_id, "remote_response": resp }),
                    ))
                }
                _ => {
                    let input = NewProposalInput {
                        account_id: acct_id,
                        service: "docs".to_string(),
                        action: "push_scratchpad".to_string(),
                        title: format!("Push Docs Scratchpad: {doc_id}"),
                        summary: format!("Sync {} line(s) to Google Doc '{doc_id}'", rendered.lines().count()),
                        tier: BlastRadiusTier::Tier2ExternalDestructive,
                        tainted_origin: is_tainted,
                        target_resource: doc_id.to_string(),
                        before_snapshot: Some(baseline),
                        after_snapshot: Some(rendered),
                        myers_diff: None,
                        payload_json: json!({
                            "doc_id": doc_id,
                            "api_base_override": base_override
                        }),
                        ttl_secs: 600,
                    };
                    propose_and_mcp_ok(state, session_id, "google_docs_scratchpad.push", is_tainted, input)
                }
            }
        }
        _ => Err(McpDomainError::InvalidInputParameters),
    }
}

// ============================================================================
// 7. google_sheets
// ============================================================================

async fn handle_google_sheets(
    state: &AppState,
    args: &serde_json::Map<String, Value>,
    session_id: &str,
) -> Result<Value, McpDomainError> {
    let action = args
        .get("action")
        .and_then(Value::as_str)
        .unwrap_or("analyze")
        .to_ascii_lowercase();
    let spreadsheet_id = args
        .get("spreadsheet_id")
        .and_then(Value::as_str)
        .ok_or(McpDomainError::InvalidInputParameters)?;
    let range = args
        .get("range")
        .and_then(Value::as_str)
        .ok_or(McpDomainError::InvalidInputParameters)?;
    let base_override = args.get("api_base_override").and_then(Value::as_str);
    let (acct_id, policy, token_opt) =
        resolve_account_ctx(state, args.get("account_id").and_then(Value::as_str)).await;
    let is_tainted = state.google_hub.taint_tracker.is_tainted(session_id);

    match action.as_str() {
        "read" => {
            let Some(token) = token_opt else {
                return Ok(unauthenticated_response("sheets_read"));
            };
            let rows = ws_client(base_override)
                .read_range(&token, spreadsheet_id, range)
                .await
                .map_err(|_| McpDomainError::StructuredFailure)?;
            state
                .google_hub
                .taint_tracker
                .mark_tainted(session_id, "sheets:read");
            Ok(mcp_ok(
                format!("Read {} row(s) from {spreadsheet_id}!{range}.", rows.len()),
                json!({ "spreadsheet_id": spreadsheet_id, "range": range, "rows": rows }),
            ))
        }
        "analyze" => {
            let rows: Vec<Vec<String>> = if let Some(rows_val) = args.get("rows") {
                serde_json::from_value(rows_val.clone())
                    .map_err(|_| McpDomainError::InvalidInputParameters)?
            } else {
                let Some(token) = token_opt else {
                    return Ok(unauthenticated_response("sheets_analyze"));
                };
                let fetched = ws_client(base_override)
                    .read_range(&token, spreadsheet_id, range)
                    .await
                    .map_err(|_| McpDomainError::StructuredFailure)?;
                state
                    .google_hub
                    .taint_tracker
                    .mark_tainted(session_id, "sheets:read");
                fetched
            };
            let analysis =
                GoogleWorkspaceClient::analyze_financial_rows(spreadsheet_id, range, &rows);
            Ok(mcp_ok(
                format!(
                    "Financial analysis for {}!{}: {} data row(s), {} anomalies.",
                    spreadsheet_id,
                    range,
                    analysis.row_count,
                    analysis.anomalies.len()
                ),
                serde_json::to_value(analysis).unwrap_or_default(),
            ))
        }
        "write" => {
            if matches!(policy, AccountPermissionPolicy::ReadOnly) {
                return Err(McpDomainError::HitlDenied);
            }
            let values: Vec<Vec<Value>> = args
                .get("values")
                .cloned()
                .and_then(|v| serde_json::from_value(v).ok())
                .ok_or(McpDomainError::InvalidInputParameters)?;
            let rendered_rows = serde_json::to_string_pretty(&values)
                .unwrap_or_default()
                .replace("\r\n", "\n");
            let input = NewProposalInput {
                account_id: acct_id,
                service: "sheets".to_string(),
                action: "write_range".to_string(),
                title: format!("Sheets Write: {spreadsheet_id}!{range}"),
                summary: format!("Write {} row(s) into '{range}'", values.len()),
                tier: BlastRadiusTier::Tier2ExternalDestructive,
                tainted_origin: is_tainted,
                target_resource: format!("{spreadsheet_id}:{range}"),
                before_snapshot: Some(format!("Range {range}")),
                after_snapshot: Some(rendered_rows),
                myers_diff: None,
                payload_json: json!({
                    "spreadsheet_id": spreadsheet_id,
                    "range": range,
                    "values": values,
                    "api_base_override": base_override
                }),
                ttl_secs: 600,
            };
            propose_and_mcp_ok(state, session_id, "google_sheets.write", is_tainted, input)
        }
        _ => Err(McpDomainError::InvalidInputParameters),
    }
}

// ============================================================================
// 8. google_slides
// ============================================================================

async fn handle_google_slides(
    state: &AppState,
    args: &serde_json::Map<String, Value>,
    session_id: &str,
) -> Result<Value, McpDomainError> {
    let action = args
        .get("action")
        .and_then(Value::as_str)
        .unwrap_or("plan")
        .to_ascii_lowercase();
    let topic = args
        .get("topic")
        .and_then(Value::as_str)
        .ok_or(McpDomainError::InvalidInputParameters)?;
    let briefing = args.get("briefing").and_then(Value::as_str).unwrap_or("");
    let target_slides = args.get("target_slides").and_then(Value::as_u64).unwrap_or(8) as usize;
    let plan = GoogleWorkspaceClient::build_slide_deck_plan(topic, briefing, target_slides);

    if action == "plan" {
        return Ok(mcp_ok(
            format!("Planned {} slides for '{topic}'.", plan.slides.len()),
            json!({ "status": "planned", "plan": plan }),
        ));
    }

    if action == "create" {
        let base_override = args.get("api_base_override").and_then(Value::as_str);
        let (acct_id, policy, token_opt) =
            resolve_account_ctx(state, args.get("account_id").and_then(Value::as_str)).await;
        let is_tainted = state.google_hub.taint_tracker.is_tainted(session_id);
        match evaluate_action_policy(BlastRadiusTier::Tier1SafeReversible, policy, is_tainted, false)
        {
            PolicyVerdict::Denied { .. } => Err(McpDomainError::HitlDenied),
            PolicyVerdict::RequireBmadInbox { .. } => {
                let plan_text = serde_json::to_string_pretty(&plan)
                    .unwrap_or_default()
                    .replace("\r\n", "\n");
                let input = NewProposalInput {
                    account_id: acct_id,
                    service: "slides".to_string(),
                    action: "create_presentation".to_string(),
                    title: format!("Create Slide Deck: {topic}"),
                    summary: format!("{} slides", plan.slides.len()),
                    tier: BlastRadiusTier::Tier1SafeReversible,
                    tainted_origin: is_tainted,
                    target_resource: topic.to_string(),
                    before_snapshot: Some(String::new()),
                    after_snapshot: Some(plan_text),
                    myers_diff: None,
                    payload_json: json!({ "plan": plan, "api_base_override": base_override }),
                    ttl_secs: 600,
                };
                propose_and_mcp_ok(state, session_id, "google_slides.create", is_tainted, input)
            }
            PolicyVerdict::AllowDirect => {
                let Some(token) = token_opt else {
                    return Ok(unauthenticated_response("slides_create"));
                };
                let created = ws_client(base_override)
                    .create_presentation(&token, &plan)
                    .await
                    .map_err(|_| McpDomainError::StructuredFailure)?;
                Ok(mcp_ok(
                    format!("Created presentation '{topic}'."),
                    json!({ "status": "executed", "plan": created }),
                ))
            }
        }
    } else {
        Err(McpDomainError::InvalidInputParameters)
    }
}

// ============================================================================
// 9. google_tasks
// ============================================================================

async fn handle_google_tasks(
    state: &AppState,
    args: &serde_json::Map<String, Value>,
    session_id: &str,
) -> Result<Value, McpDomainError> {
    let action = args
        .get("action")
        .and_then(Value::as_str)
        .unwrap_or("list")
        .to_ascii_lowercase();
    let base_override = args.get("api_base_override").and_then(Value::as_str);
    let account_id_arg = args.get("account_id").and_then(Value::as_str);
    let (acct_id, policy, token_opt) = resolve_account_ctx(state, account_id_arg).await;
    let is_tainted = state.google_hub.taint_tracker.is_tainted(session_id);

    match action.as_str() {
        "list" => {
            let Some(token) = token_opt else {
                return Ok(unauthenticated_response("tasks_list"));
            };
            let list_id = args.get("tasklist_id").and_then(Value::as_str).unwrap_or("@default");
            let show_completed = args.get("show_completed").and_then(Value::as_bool).unwrap_or(false);
            let client = match base_override {
                Some(b) => GoogleTasksClient::with_base_url(b),
                None => GoogleTasksClient::new(),
            };
            let tasks = client
                .list_tasks(&token, list_id, show_completed)
                .await
                .map_err(|_| McpDomainError::StructuredFailure)?;
            Ok(mcp_ok(
                format!("Listed {} task(s).", tasks.len()),
                json!({ "count": tasks.len(), "tasks": tasks }),
            ))
        }
        "create" | "complete" => {
            match evaluate_action_policy(
                BlastRadiusTier::Tier1SafeReversible,
                policy,
                is_tainted,
                false,
            ) {
                PolicyVerdict::Denied { .. } => Err(McpDomainError::HitlDenied),
                PolicyVerdict::RequireBmadInbox { .. } => {
                    let act_norm = if action == "complete" {
                        "complete_task"
                    } else {
                        "create_task"
                    };
                    let list_id = args.get("tasklist_id").and_then(Value::as_str).unwrap_or("@default");
                    let title = args.get("title").and_then(Value::as_str).unwrap_or("Google Task");
                    let task_id = args.get("task_id").and_then(Value::as_str);
                    let input = NewProposalInput {
                        account_id: acct_id,
                        service: "tasks".to_string(),
                        action: act_norm.to_string(),
                        title: format!("Google Task ({act_norm}): {title}"),
                        summary: format!("Tasklist: {list_id}"),
                        tier: BlastRadiusTier::Tier1SafeReversible,
                        tainted_origin: is_tainted,
                        target_resource: task_id.unwrap_or(list_id).to_string(),
                        before_snapshot: Some(String::new()),
                        after_snapshot: Some(format!("action={act_norm}\ntitle={title}")),
                        myers_diff: None,
                        payload_json: json!({
                            "tasklist_id": list_id,
                            "title": title,
                            "notes": args.get("notes").and_then(Value::as_str),
                            "due_rfc3339": args.get("due_rfc3339").and_then(Value::as_str),
                            "task_id": task_id,
                            "api_base_override": base_override
                        }),
                        ttl_secs: 600,
                    };
                    propose_and_mcp_ok(state, session_id, &format!("google_tasks.{act_norm}"), is_tainted, input)
                }
                PolicyVerdict::AllowDirect => {
                    let Some(token) = token_opt else {
                        return Ok(unauthenticated_response("tasks_mutate"));
                    };
                    let list_id = args.get("tasklist_id").and_then(Value::as_str).unwrap_or("@default");
                    let client = match base_override {
                        Some(b) => GoogleTasksClient::with_base_url(b),
                        None => GoogleTasksClient::new(),
                    };
                    if action == "complete" {
                        let task_id = args
                            .get("task_id")
                            .and_then(Value::as_str)
                            .ok_or(McpDomainError::InvalidInputParameters)?;
                        let task = client
                            .complete_task(&token, list_id, task_id)
                            .await
                            .map_err(|_| McpDomainError::StructuredFailure)?;
                        Ok(mcp_ok(
                            format!("Completed task '{task_id}'."),
                            json!({ "status": "executed", "task": task }),
                        ))
                    } else {
                        let title = args
                            .get("title")
                            .and_then(Value::as_str)
                            .ok_or(McpDomainError::InvalidInputParameters)?;
                        let task = client
                            .create_task(
                                &token,
                                list_id,
                                title,
                                args.get("notes").and_then(Value::as_str),
                                args.get("due_rfc3339").and_then(Value::as_str),
                            )
                            .await
                            .map_err(|_| McpDomainError::StructuredFailure)?;
                        Ok(mcp_ok(
                            format!("Created task '{title}'."),
                            json!({ "status": "executed", "task": task }),
                        ))
                    }
                }
            }
        }
        "quick_capture" => {
            let title = args
                .get("title")
                .and_then(Value::as_str)
                .ok_or(McpDomainError::InvalidInputParameters)?;
            let body = args.get("body").and_then(Value::as_str).unwrap_or("");
            let tags = json_str_vec(args.get("tags"));
            let dir = std::env::var("OBSIDIAN_VAULT_PATH")
                .ok()
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty())
                .map(|v| PathBuf::from(v).join("raw").join("inbox"))
                .unwrap_or_else(|| state.google_hub.data_dir.join("quick_capture"));
            let note_path = quick_capture_to_obsidian(&dir, title, body, &tags)
                .map_err(|_| McpDomainError::StructuredFailure)?;
            Ok(mcp_ok(
                format!("Quick-capture note saved to {}.", note_path.display()),
                json!({ "status": "captured", "note_path": note_path.display().to_string() }),
            ))
        }
        "check_keep_eligibility" => {
            let acct = state.google_hub.resolve_account(account_id_arg).await;
            let kind = acct.map(|a| a.kind).unwrap_or(GoogleAccountKind::PersonalGmail);
            match check_keep_eligibility(kind) {
                Ok(()) => Ok(mcp_ok(
                    "Account is eligible for Workspace Enterprise Google Keep API.",
                    json!({ "eligible": true, "kind": kind }),
                )),
                Err(err) => Ok(mcp_ok(
                    err.to_string(),
                    json!({ "eligible": false, "kind": kind, "fallback": "google_tasks + quick_capture" }),
                )),
            }
        }
        _ => Err(McpDomainError::InvalidInputParameters),
    }
}

// ============================================================================
// 10. google_contacts
// ============================================================================

async fn handle_google_contacts(
    state: &AppState,
    args: &serde_json::Map<String, Value>,
    session_id: &str,
) -> Result<Value, McpDomainError> {
    let action = args
        .get("action")
        .and_then(Value::as_str)
        .unwrap_or("search")
        .to_ascii_lowercase();
    let base_override = args.get("api_base_override").and_then(Value::as_str);
    let (_, _, token_opt) =
        resolve_account_ctx(state, args.get("account_id").and_then(Value::as_str)).await;

    let contacts: Vec<ContactCardDto> = if let Some(inline) = args.get("contacts") {
        serde_json::from_value(inline.clone())
            .map_err(|_| McpDomainError::InvalidInputParameters)?
    } else {
        let Some(token) = token_opt else {
            return Ok(unauthenticated_response("google_contacts"));
        };
        let client = match base_override {
            Some(b) => PeopleClient::with_base_url(b),
            None => PeopleClient::new(),
        };
        let q = args.get("query").and_then(Value::as_str).unwrap_or("");
        let page_size = args.get("page_size").and_then(Value::as_u64).unwrap_or(20) as usize;
        let fetched = client
            .search_contacts(&token, q, page_size)
            .await
            .map_err(|_| McpDomainError::StructuredFailure)?;
        state
            .google_hub
            .taint_tracker
            .mark_tainted(session_id, "contacts:read");
        fetched
    };

    if action == "upcoming_birthdays" {
        let within_days = args.get("within_days").and_then(Value::as_u64).unwrap_or(14) as u32;
        let now = Utc::now();
        let m = now.format("%m").to_string().parse::<u32>().unwrap_or(1);
        let d = now.format("%d").to_string().parse::<u32>().unwrap_or(1);
        let bdays = upcoming_birthdays(&contacts, m, d, within_days);
        return Ok(mcp_ok(
            format!("{} upcoming birthday(s) in next {within_days} day(s).", bdays.len()),
            json!({ "count": bdays.len(), "contacts": bdays }),
        ));
    }

    Ok(mcp_ok(
        format!("Retrieved {} contact(s).", contacts.len()),
        json!({ "count": contacts.len(), "contacts": contacts }),
    ))
}

// ============================================================================
// 11. google_commute
// ============================================================================

async fn handle_google_commute(args: &serde_json::Map<String, Value>) -> Result<Value, McpDomainError> {
    let origin = args
        .get("origin")
        .and_then(Value::as_str)
        .ok_or(McpDomainError::InvalidInputParameters)?;
    let destination = args
        .get("destination")
        .and_then(Value::as_str)
        .ok_or(McpDomainError::InvalidInputParameters)?;
    let api_key = args
        .get("api_key")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToString::to_string)
        .or_else(|| std::env::var("ROUTES_API_KEY").ok().filter(|s| !s.trim().is_empty()))
        .or_else(|| std::env::var("GOOGLE_MAPS_API_KEY").ok().filter(|s| !s.trim().is_empty()))
        .ok_or(McpDomainError::InvalidInputParameters)?;

    let client = match args.get("api_base_override").and_then(Value::as_str) {
        Some(b) => RoutesV2Client::with_base_url(b),
        None => RoutesV2Client::new(),
    };
    let comp = client
        .compute_route(&api_key, origin, destination)
        .await
        .map_err(|_| McpDomainError::StructuredFailure)?;
    let event_start = args
        .get("event_start_rfc3339")
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .unwrap_or_else(|| (Utc::now() + ChronoDuration::hours(2)).to_rfc3339());
    let alert = souls_core::google_bridge::compute_commute_alert(
        args.get("event_id").and_then(Value::as_str).unwrap_or("commute_mcp"),
        args.get("event_title").and_then(Value::as_str).unwrap_or("Commute"),
        &event_start,
        destination,
        comp.duration_seconds,
        comp.static_duration_seconds,
        comp.distance_meters,
        args.get("buffer_mins").and_then(Value::as_u64).unwrap_or(15) as u32,
    )
    .map_err(|_| McpDomainError::InvalidInputParameters)?;

    Ok(mcp_ok(
        alert.advisory.clone(),
        serde_json::to_value(alert).unwrap_or_default(),
    ))
}

// ============================================================================
// 12. google_youtube
// ============================================================================

async fn handle_google_youtube(
    state: &AppState,
    args: &serde_json::Map<String, Value>,
    session_id: &str,
) -> Result<Value, McpDomainError> {
    let url_or_id = args
        .get("url_or_id")
        .and_then(Value::as_str)
        .ok_or(McpDomainError::InvalidInputParameters)?;
    let lang = args
        .get("preferred_lang")
        .and_then(Value::as_str)
        .unwrap_or("en");
    let client = match args.get("api_base_override").and_then(Value::as_str) {
        Some(b) => YouTubeClient::with_base_url(b),
        None => YouTubeClient::new(),
    };
    let mut summary = client
        .fetch_transcript_with_cache(&state.google_hub.data_dir, url_or_id, lang)
        .await
        .map_err(|_| McpDomainError::StructuredFailure)?;
    state
        .google_hub
        .taint_tracker
        .mark_tainted(session_id, "youtube:transcript");
    summary.transcript_text =
        wrap_untrusted_content(&format!("youtube:{}", summary.video_id), &summary.transcript_text);
    Ok(mcp_ok(
        summary.transcript_text.clone(),
        serde_json::to_value(summary).unwrap_or_default(),
    ))
}

// ============================================================================
// 13. google_notebooklm
// ============================================================================

async fn handle_google_notebooklm(
    state: &AppState,
    args: &serde_json::Map<String, Value>,
    session_id: &str,
) -> Result<Value, McpDomainError> {
    let action = args
        .get("action")
        .and_then(Value::as_str)
        .unwrap_or("list")
        .to_ascii_lowercase();
    let notebook_id = args.get("notebook_id").and_then(Value::as_str);
    let endpoint_override = args.get("endpoint_override").and_then(Value::as_str);

    if action == "list" {
        let nlm = NlmCliBridge::new();
        if let Ok(nbs) = nlm.list_notebooks().await {
            return Ok(mcp_ok(
                format!("Listed {} NotebookLM notebooks via nlm CLI.", nbs.len()),
                json!({ "backend": "nlm_cli", "notebooks": nbs }),
            ));
        }
    }

    let Some(creds) = resolve_nlm_creds(state).await else {
        return Ok(mcp_ok(
            "NotebookLM offline: neither nlm CLI nor __Secure-1PSID cookies configured.",
            json!({ "status": "offline", "notebooks": [] }),
        ));
    };

    let mut client = WizRpcClient::new(creds);
    if let Some(ep) = endpoint_override {
        client = client.with_endpoint(ep);
    }

    let (rpc_id, payload_args) = match action.as_str() {
        "list" => (WizRpcId::ListNotebooks, json!([null, 1, null, [2]])),
        "get" => {
            let id = notebook_id.ok_or(McpDomainError::InvalidInputParameters)?;
            state
                .google_hub
                .taint_tracker
                .mark_tainted(session_id, &format!("notebooklm:{id}"));
            (WizRpcId::GetNotebook, json!([id, null, [2], null, 0]))
        }
        "add_source" => {
            let id = notebook_id.ok_or(McpDomainError::InvalidInputParameters)?;
            let url = args.get("source_url").and_then(Value::as_str);
            (WizRpcId::AddSource, json!([[[null, null, url]], id]))
        }
        "check_freshness" => {
            let src_id = args
                .get("source_id")
                .and_then(Value::as_str)
                .ok_or(McpDomainError::InvalidInputParameters)?;
            (WizRpcId::CheckFreshness, json!([src_id]))
        }
        "sync_drive" => {
            let src_id = args
                .get("source_id")
                .and_then(Value::as_str)
                .or(notebook_id)
                .ok_or(McpDomainError::InvalidInputParameters)?;
            (WizRpcId::SyncDriveSource, json!([src_id]))
        }
        "query" => {
            let id = notebook_id.ok_or(McpDomainError::InvalidInputParameters)?;
            let query = args
                .get("query")
                .and_then(Value::as_str)
                .ok_or(McpDomainError::InvalidInputParameters)?;
            state
                .google_hub
                .taint_tracker
                .mark_tainted(session_id, &format!("notebooklm:{id}"));
            (WizRpcId::QueryRAG, json!([id, query]))
        }
        "studio_create" => {
            let id = notebook_id.ok_or(McpDomainError::InvalidInputParameters)?;
            let prompt = args.get("prompt").and_then(Value::as_str);
            (WizRpcId::StudioCreate, json!([[2], id, [null, null, prompt]]))
        }
        "studio_status" => {
            let id = notebook_id.ok_or(McpDomainError::InvalidInputParameters)?;
            (WizRpcId::StudioStatus, json!([id]))
        }
        "mindmap" => {
            let id = notebook_id.ok_or(McpDomainError::InvalidInputParameters)?;
            (WizRpcId::GenerateMindmap, json!([id]))
        }
        other => (resolve_wiz_rpc_id(other), args.get("arguments").cloned().unwrap_or(json!([]))),
    };

    let rpc = WizRpcPayload::new(rpc_id, payload_args);
    let resp = client
        .execute(&rpc)
        .await
        .map_err(|_| McpDomainError::StructuredFailure)?;
    Ok(mcp_ok(
        format!("Executed NotebookLM RPC '{}'.", resp.rpc_id),
        json!({ "rpc_id": resp.rpc_id, "success": resp.success, "data": resp.payload }),
    ))
}

// ============================================================================
// 14. google_gems
// ============================================================================

async fn handle_google_gems(
    state: &AppState,
    args: &serde_json::Map<String, Value>,
) -> Result<Value, McpDomainError> {
    let action = args
        .get("action")
        .and_then(Value::as_str)
        .unwrap_or("list")
        .to_ascii_lowercase();

    match action.as_str() {
        "list" => {
            let gems = state.google_hub.gems.list();
            Ok(mcp_ok(
                format!("{} Custom Gem(s) configured.", gems.len()),
                json!({ "count": gems.len(), "gems": gems }),
            ))
        }
        "upsert" => {
            let id = args
                .get("id")
                .and_then(Value::as_str)
                .ok_or(McpDomainError::InvalidInputParameters)?;
            let gem = GemProfileDto {
                id: id.to_string(),
                name: args.get("name").and_then(Value::as_str).unwrap_or(id).to_string(),
                description: args.get("description").and_then(Value::as_str).unwrap_or("").to_string(),
                system_instruction: args
                    .get("system_instruction")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string(),
                model: args
                    .get("model")
                    .and_then(Value::as_str)
                    .unwrap_or("gemini-2.5-flash")
                    .to_string(),
                backend: args
                    .get("backend")
                    .and_then(Value::as_str)
                    .unwrap_or("gemini_api")
                    .to_string(),
                remote_gem_id: args
                    .get("remote_gem_id")
                    .and_then(Value::as_str)
                    .map(ToString::to_string),
            };
            let saved = state
                .google_hub
                .gems
                .upsert(gem)
                .map_err(|_| McpDomainError::StructuredFailure)?;
            Ok(mcp_ok(
                format!("Saved Gem '{}'.", saved.id),
                json!({ "status": "upserted", "gem": saved }),
            ))
        }
        "delete" => {
            let id = args
                .get("id")
                .and_then(Value::as_str)
                .ok_or(McpDomainError::InvalidInputParameters)?;
            let removed = state
                .google_hub
                .gems
                .remove(id)
                .map_err(|_| McpDomainError::StructuredFailure)?;
            Ok(mcp_ok(
                format!("Deleted Gem '{id}': {removed}"),
                json!({ "removed": removed }),
            ))
        }
        "dispatch" | "secondary_stream_generate" => {
            let prompt = args
                .get("prompt")
                .and_then(Value::as_str)
                .ok_or(McpDomainError::InvalidInputParameters)?;
            let id = args.get("id").and_then(Value::as_str).unwrap_or("adhoc_gem");
            let gem = state.google_hub.gems.get(id).unwrap_or(GemProfileDto {
                id: id.to_string(),
                name: id.to_string(),
                description: String::new(),
                system_instruction: args
                    .get("system_instruction")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string(),
                model: args
                    .get("model")
                    .and_then(Value::as_str)
                    .unwrap_or("gemini-2.5-flash")
                    .to_string(),
                backend: args
                    .get("backend")
                    .and_then(Value::as_str)
                    .unwrap_or("gemini_api")
                    .to_string(),
                remote_gem_id: args
                    .get("remote_gem_id")
                    .and_then(Value::as_str)
                    .map(ToString::to_string),
            });

            if action == "secondary_stream_generate" || gem.backend == "secondary_web" {
                let acct = state
                    .google_hub
                    .resolve_account_strict(args.get("account_id").and_then(Value::as_str))
                    .await
                    .ok_or(McpDomainError::HitlDenied)?;
                let remote_id = gem.remote_gem_id.as_deref().unwrap_or(&gem.id);
                let model_hash = args
                    .get("model_hash")
                    .and_then(Value::as_str)
                    .unwrap_or("9d8ca3786ebdfbea");
                let payload = build_secondary_stream_generate_payload(&acct, remote_id, prompt, model_hash)
                    .map_err(|e| match e {
                        CoreError::DangerousCommandBlocked(_) => McpDomainError::HitlDenied,
                        _ => McpDomainError::StructuredFailure,
                    })?;
                return Ok(mcp_ok(
                    format!("Built gated secondary StreamGenerate payload for '{}'.", acct.account_id),
                    json!({ "backend": "secondary_web", "payload": payload }),
                ));
            }

            let api_key = args
                .get("api_key")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(ToString::to_string)
                .or_else(|| std::env::var("GEMINI_API_KEY").ok().filter(|s| !s.trim().is_empty()))
                .ok_or(McpDomainError::InvalidInputParameters)?;
            let base_url = args.get("base_url").and_then(Value::as_str).unwrap_or("");
            let text = dispatch_gem_official_api(&reqwest::Client::new(), base_url, &api_key, &gem, prompt)
                .await
                .map_err(|_| McpDomainError::StructuredFailure)?;
            Ok(mcp_ok(
                text.clone(),
                json!({ "backend": "gemini_api", "gem_id": gem.id, "response_text": text }),
            ))
        }
        _ => Err(McpDomainError::InvalidInputParameters),
    }
}

// ============================================================================
// 15. inbox_list
// ============================================================================

fn handle_inbox_list(
    state: &AppState,
    args: &serde_json::Map<String, Value>,
) -> Result<Value, McpDomainError> {
    let status_filter = match args
        .get("status")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        None | Some("all") => None,
        Some("pending") => Some(ProposalStatus::Pending),
        Some("approved") => Some(ProposalStatus::Approved),
        Some("executed") => Some(ProposalStatus::Executed),
        Some("rejected") => Some(ProposalStatus::Rejected),
        Some("failed") => Some(ProposalStatus::Failed),
        Some("expired") => Some(ProposalStatus::Expired),
        Some(_) => return Err(McpDomainError::InvalidInputParameters),
    };

    let limit = args.get("limit").and_then(Value::as_u64).unwrap_or(50) as usize;
    let proposals: Vec<Value> = state
        .google_hub
        .inbox
        .list(status_filter)
        .into_iter()
        .take(limit.max(1))
        .map(|p| sanitize_proposal_for_mcp(&p))
        .collect();
    let pending_count = state.google_hub.inbox.pending_count();

    Ok(mcp_ok(
        format!(
            "Agent Inbox: {} proposal(s) returned ({} pending approval).",
            proposals.len(),
            pending_count
        ),
        json!({
            "pending_count": pending_count,
            "total": proposals.len(),
            "proposals": proposals
        }),
    ))
}

// ============================================================================
// 16. inbox_inspect_diff
// ============================================================================

fn handle_inbox_inspect_diff(
    state: &AppState,
    args: &serde_json::Map<String, Value>,
) -> Result<Value, McpDomainError> {
    let proposal_id = args
        .get("proposal_id")
        .and_then(Value::as_str)
        .ok_or(McpDomainError::InvalidInputParameters)?;

    let Some(prop) = state.google_hub.inbox.get(proposal_id) else {
        return Err(McpDomainError::SymbolNotFound);
    };

    let diff_text = prop
        .myers_diff
        .clone()
        .unwrap_or_else(|| "(no diff available)".to_string());

    Ok(mcp_ok(
        format!(
            "Proposal '{}' [{}] ({}.{} -> {}):\n{}",
            prop.id,
            format!("{:?}", prop.status).to_ascii_lowercase(),
            prop.service,
            prop.action,
            prop.target_resource,
            diff_text
        ),
        sanitize_proposal_for_mcp(&prop),
    ))
}

// ============================================================================
// 17. inbox_propose
// ============================================================================

fn handle_inbox_propose(
    state: &AppState,
    args: &serde_json::Map<String, Value>,
    session_id: &str,
) -> Result<Value, McpDomainError> {
    let service = args
        .get("service")
        .and_then(Value::as_str)
        .ok_or(McpDomainError::InvalidInputParameters)?;
    let action = args
        .get("action")
        .and_then(Value::as_str)
        .ok_or(McpDomainError::InvalidInputParameters)?;
    let title = args
        .get("title")
        .and_then(Value::as_str)
        .ok_or(McpDomainError::InvalidInputParameters)?;
    let target_resource = args
        .get("target_resource")
        .and_then(Value::as_str)
        .ok_or(McpDomainError::InvalidInputParameters)?;

    let tier = match args.get("tier").cloned() {
        Some(v) => serde_json::from_value(v).map_err(|_| McpDomainError::InvalidInputParameters)?,
        None => BlastRadiusTier::Tier1SafeReversible,
    };

    let account_id = args
        .get("account_id")
        .and_then(Value::as_str)
        .unwrap_or("default")
        .to_string();
    let summary = args
        .get("summary")
        .and_then(Value::as_str)
        .unwrap_or(title)
        .to_string();
    let before_snapshot = args
        .get("before_snapshot")
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let after_snapshot = args
        .get("after_snapshot")
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let myers_diff = args
        .get("myers_diff")
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let payload_json = args.get("payload_json").cloned().unwrap_or_else(|| json!({}));
    let ttl_secs = args.get("ttl_secs").and_then(Value::as_i64).unwrap_or(600);

    let is_tainted = state.google_hub.taint_tracker.is_tainted(session_id);
    let input = NewProposalInput {
        account_id: account_id.clone(),
        service: service.to_string(),
        action: action.to_string(),
        title: title.to_string(),
        summary,
        tier,
        tainted_origin: is_tainted,
        target_resource: target_resource.to_string(),
        before_snapshot,
        after_snapshot,
        myers_diff,
        payload_json,
        ttl_secs,
    };

    let prop = state
        .google_hub
        .propose_with_diff(input)
        .map_err(|_| McpDomainError::StructuredFailure)?;

    state.google_hub.record_audit_event(
        session_id,
        &account_id,
        "inbox_propose",
        tier,
        is_tainted,
        "proposed",
        &format!("Proposed '{}' ({}.{})", prop.id, service, action),
    );

    Ok(mcp_ok(
        format!(
            "Queued BMAD proposal '{}' (gravity_lock_required={}, tainted_origin={}).",
            prop.id, prop.gravity_lock_required, prop.tainted_origin
        ),
        json!({
            "status": "proposed",
            "proposal_id": prop.id,
            "tainted_origin": prop.tainted_origin,
            "gravity_lock_required": prop.gravity_lock_required,
            "myers_diff": prop.myers_diff,
            "proposal": sanitize_proposal_for_mcp(&prop)
        }),
    ))
}

// ============================================================================
// SHARED HELPERS
// ============================================================================

async fn resolve_account_ctx(
    state: &AppState,
    account_id: Option<&str>,
) -> (String, AccountPermissionPolicy, Option<String>) {
    match state.google_hub.resolve_account(account_id).await {
        Some(a) => {
            let tok = a
                .oauth_access_token
                .as_deref()
                .map(str::trim)
                .filter(|t| !t.is_empty())
                .map(ToString::to_string);
            (a.account_id, a.policy, tok)
        }
        None => (
            "default".to_string(),
            AccountPermissionPolicy::BalancedHotlHitl,
            None,
        ),
    }
}

async fn resolve_nlm_creds(state: &AppState) -> Option<GoogleSessionCredentials> {
    let vault = state.google_hub.vault.read().await;
    vault
        .accounts()
        .iter()
        .find(|a| {
            a.has_nlm_bridge
                || a.psid.as_deref().is_some_and(|s| !s.is_empty())
                || a.sapisid.as_deref().is_some_and(|s| !s.is_empty())
        })
        .map(GoogleAccountRecord::to_legacy_credentials)
}

fn ws_client(base_override: Option<&str>) -> GoogleWorkspaceClient {
    match base_override.map(str::trim).filter(|s| !s.is_empty()) {
        Some(b) => GoogleWorkspaceClient::with_base_urls(b, b, b, b, b, b),
        None => GoogleWorkspaceClient::new(),
    }
}

fn propose_and_mcp_ok(
    state: &AppState,
    session_id: &str,
    tool_name: &str,
    is_tainted: bool,
    input: NewProposalInput,
) -> Result<Value, McpDomainError> {
    let acct_id = input.account_id.clone();
    let tier = input.tier;
    let prop = state
        .google_hub
        .propose_with_diff(input)
        .map_err(|_| McpDomainError::StructuredFailure)?;

    state.google_hub.record_audit_event(
        session_id,
        &acct_id,
        tool_name,
        tier,
        is_tainted,
        "queued_bmad",
        &format!("Queued BMAD proposal '{}' ({})", prop.id, prop.title),
    );

    Ok(mcp_ok(
        format!(
            "Action queued in BMAD Agent Inbox as '{}' (gravity_lock_required={}, escalated_by_taint={}).",
            prop.id, prop.gravity_lock_required, is_tainted
        ),
        json!({
            "status": "queued_for_approval",
            "proposal_id": prop.id,
            "escalated_by_taint": is_tainted,
            "gravity_lock_required": prop.gravity_lock_required,
            "myers_diff": prop.myers_diff,
            "proposal": sanitize_proposal_for_mcp(&prop)
        }),
    ))
}

fn sanitize_proposal_for_mcp(prop: &BmadProposalDto) -> Value {
    let mut val = serde_json::to_value(prop).unwrap_or_default();
    if let Some(obj) = val.as_object_mut() {
        if obj
            .get("gravity_lock_token")
            .and_then(Value::as_str)
            .is_some()
        {
            obj.insert(
                "gravity_lock_token".to_string(),
                json!("[HUMAN_APPROVAL_ONLY]"),
            );
        }
    }
    val
}

fn unauthenticated_response(op: &str) -> Value {
    mcp_ok(
        format!("Google OAuth2 access token not configured for '{op}'. Connect an account via OAuth2 PKCE or import gws credentials."),
        json!({
            "status": "unauthenticated",
            "oauth_connected": false,
            "operation": op
        }),
    )
}

fn mcp_ok(text: impl Into<String>, structured: Value) -> Value {
    json!({
        "content": [{ "type": "text", "text": text.into() }],
        "structuredContent": structured,
        "isError": false
    })
}

fn json_str_vec(val: Option<&Value>) -> Vec<String> {
    val.and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(ToString::to_string))
                .collect()
        })
        .unwrap_or_default()
}

fn sanitize_id(raw: &str) -> String {
    let mut out = String::new();
    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
        } else if !out.ends_with('_') && !out.is_empty() {
            out.push('_');
        }
    }
    let trimmed = out.trim_matches('_').to_string();
    if trimmed.is_empty() {
        "google_account".to_string()
    } else {
        trimmed
    }
}

fn infer_kind(email: &str) -> GoogleAccountKind {
    let lower = email.trim().to_ascii_lowercase();
    if lower.is_empty() || lower.ends_with("@gmail.com") || lower.ends_with("@googlemail.com") {
        GoogleAccountKind::PersonalGmail
    } else {
        GoogleAccountKind::WorkspaceEnterprise
    }
}
