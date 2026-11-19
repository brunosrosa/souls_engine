//! REST endpoints for Google Universe Bridge, Life Hub, Multi-Account OAuth2 PKCE,
//! Gemini Notebooks UI, HTTP Range Audio Streaming, Custom Gems, and Companion Doctor.
//!
//! Preserves 100% backward compatibility with v0.8.2 endpoints while exposing the
//! complete v0.9.2 Google Life Hub & BMAD governance surface.

#![forbid(unsafe_code)]

use std::io::SeekFrom;
use std::path::PathBuf;

use axum::body::{Body, Bytes};
use axum::extract::{Path, Query, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse, Response};
use axum::Json;
use chrono::{Duration as ChronoDuration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use tokio_stream::wrappers::ReceiverStream;
use tracing::{debug, error, info};

use souls_ast::compute_safe_myers_diff;
use souls_core::error::CoreError;
use souls_core::google_bridge::{
    build_google_auth_url, build_secondary_stream_generate_payload, compute_commute_alert,
    dispatch_gem_official_api, evaluate_action_policy, exchange_code_for_tokens,
    find_binary_on_path, import_gws_credentials, import_gws_credentials_from_dir,
    parse_google_client_secret_json, quick_capture_to_obsidian, upcoming_birthdays,
    wrap_untrusted_content, DualLayerWorkspace, GoogleAccountRecord, GoogleSessionCredentials,
    GoogleSessionVault, GoogleTasksClient, GoogleWorkspaceClient, JulesMigrationRouter,
    LegacyJulesCommand, NewProposalInput, NlmCliBridge, OAuth2Config, PeopleClient, PkceChallenge,
    PolicyVerdict, RoutesV2Client, WizRpcClient, WizRpcId, WizRpcPayload, YouTubeClient,
};
use souls_core::harness::AgenticHarnessDispatcher;
use souls_protocol::google::{
    AccountPermissionPolicy, BlastRadiusTier, GemProfileDto, GmailTriageResultDto,
    GoogleAccountKind,
};

use crate::google_hub::{is_safe_path_segment, PendingPkce, PkceOAuthMeta};
use crate::state::AppState;

// ============================================================================
// v0.8.2 BACKWARD-COMPATIBLE TYPES & HANDLERS
// ============================================================================

/// DTO reporting Google Session Vault status with redacted tokens.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleSessionStatusDto {
    pub is_authenticated: bool,
    pub account_email: Option<String>,
    pub expires_at: Option<i64>,
    pub is_expired: bool,
    pub has_psid: bool,
    pub masked_psid: Option<String>,
    pub has_psidts: bool,
    pub masked_psidts: Option<String>,
    pub has_sapisid: bool,
    pub masked_sapisid: Option<String>,
    pub has_oauth_token: bool,
    pub masked_oauth_token: Option<String>,
    pub csrf_token: String,
    pub vault_path: String,
}

/// Request payload to save or update Google session credentials.
#[derive(Debug, Clone, Deserialize)]
pub struct SaveGoogleSessionRequest {
    pub psid: Option<String>,
    pub psidts: Option<String>,
    pub sapisid: Option<String>,
    pub csrf_token: Option<String>,
    pub oauth_access_token: Option<String>,
    pub account_email: Option<String>,
    pub expires_at: Option<i64>,
}

/// Request payload to execute a Wiz batchexecute RPC.
#[derive(Debug, Clone, Deserialize)]
pub struct WizRpcRequestDto {
    pub rpc_id: String,
    pub arguments: serde_json::Value,
    pub endpoint_override: Option<String>,
}

/// Request payload to migrate a legacy Jules command line.
#[derive(Debug, Clone, Deserialize)]
pub struct JulesMigrationRequestDto {
    pub command: String,
    pub working_dir: Option<String>,
}

/// Request payload to link a Google Drive resource to a local path in Dual-Layer Workspace.
#[derive(Debug, Clone, Deserialize)]
pub struct WorkspaceLinkRequestDto {
    pub local_path: Option<String>,
    pub drive_folder_id: Option<String>,
    pub linked_document_id: Option<String>,
}

/// Resolves the canonical vault path, prioritizing `SOULS_GOOGLE_VAULT_PATH` if configured.
pub fn resolve_google_vault_path() -> PathBuf {
    if let Ok(p) = std::env::var("SOULS_GOOGLE_VAULT_PATH") {
        return PathBuf::from(p);
    }
    let canonical = GoogleSessionVault::default_vault_path();
    if let Some(parent) = canonical.parent() {
        if parent.exists() {
            return canonical;
        }
    }
    PathBuf::from(r".souls_data\google_sessions.vault")
}

/// Maps any canonical NotebookLM Wiz RPC ID string or alias to `WizRpcId`.
pub fn resolve_wiz_rpc_id(raw: &str) -> WizRpcId {
    match raw.trim().to_ascii_lowercase().as_str() {
        "wxbhsf" | "list_notebooks" => WizRpcId::ListNotebooks,
        "rlm1ne" | "get_notebook" => WizRpcId::GetNotebook,
        "ccqfvf" | "create_notebook" => WizRpcId::CreateNotebook,
        "s0tc2d" | "rename_notebook" | "rename_or_style_notebook" => WizRpcId::RenameOrStyleNotebook,
        "wwinqb" | "delete_notebook" => WizRpcId::DeleteNotebook,
        "jfmdgd" | "share_status" => WizRpcId::ShareStatus,
        "izaodd" | "add_source" | "ingest_source" => WizRpcId::AddSource,
        "ozz5z" | "add_source_v2" => WizRpcId::AddSourceV2,
        "hizojc" | "get_source" => WizRpcId::GetSource,
        "yr9yof" | "check_freshness" => WizRpcId::CheckFreshness,
        "flmjqe" | "sync_drive" | "sync_drive_source" => WizRpcId::SyncDriveSource,
        "b7wfje" | "rename_source" => WizRpcId::RenameSource,
        "tgmbj" | "delete_source" => WizRpcId::DeleteSource,
        "tr032e" | "source_guide" => WizRpcId::SourceGuide,
        "hptbtc" | "list_conversations" => WizRpcId::ListConversations,
        "khqzz" | "get_conversation_turns" => WizRpcId::GetConversationTurns,
        "ljjv0c" | "fast_research" => WizRpcId::FastResearch,
        "qa9ei" | "deep_research" => WizRpcId::DeepResearch,
        "e3bvqc" | "poll_research" => WizRpcId::PollResearch,
        "lbwxtb" | "import_research" => WizRpcId::ImportResearch,
        "r7cb6c" | "studio_create" => WizRpcId::StudioCreate,
        "gartlc" | "studio_status" => WizRpcId::StudioStatus,
        "v9rmvd" | "get_artifact" => WizRpcId::GetArtifact,
        "rc3d8d" | "steer_artifact" => WizRpcId::SteerArtifact,
        "kmckpe" | "revise_slides" => WizRpcId::ReviseSlides,
        "yyryje" | "generate_mindmap" => WizRpcId::GenerateMindmap,
        "cyk0xb" | "save_mindmap" => WizRpcId::SaveMindmap,
        "cfji9" | "list_mindmaps" => WizRpcId::ListMindmaps,
        "orqrsd" | "query_rag" => WizRpcId::QueryRAG,
        "typjf" | "create_note" => WizRpcId::CreateNote,
        "cxjtyd" | "read_notes" => WizRpcId::ReadNotes,
        _ => WizRpcId::Custom(raw.trim().to_string()),
    }
}

/// GET /api/v1/google/session
///
/// Returns current Google session status from the local encrypted vault.
pub async fn get_session_handler() -> impl IntoResponse {
    let vault_path = resolve_google_vault_path();
    match GoogleSessionVault::load(&vault_path) {
        Ok(Some(creds)) => {
            let now = Utc::now().timestamp();
            let masked = creds.masked();
            let status = GoogleSessionStatusDto {
                is_authenticated: creds.is_configured(),
                account_email: creds.account_email.clone(),
                expires_at: creds.expires_at,
                is_expired: creds.is_expired(now),
                has_psid: creds.psid.is_some(),
                masked_psid: masked.psid,
                has_psidts: creds.psidts.is_some(),
                masked_psidts: masked.psidts,
                has_sapisid: creds.sapisid.is_some(),
                masked_sapisid: masked.sapisid,
                has_oauth_token: creds.oauth_access_token.is_some(),
                masked_oauth_token: masked.oauth_access_token,
                csrf_token: creds.csrf_token,
                vault_path: vault_path.to_string_lossy().to_string(),
            };
            (
                StatusCode::OK,
                Json(serde_json::to_value(status).unwrap_or_default()),
            )
        }
        _ => {
            debug!(target: "souls_server::google", "Google session not found or vault empty");
            let status = GoogleSessionStatusDto {
                is_authenticated: false,
                account_email: None,
                expires_at: None,
                is_expired: false,
                has_psid: false,
                masked_psid: None,
                has_psidts: false,
                masked_psidts: None,
                has_sapisid: false,
                masked_sapisid: None,
                has_oauth_token: false,
                masked_oauth_token: None,
                csrf_token: "SNlM0e".to_string(),
                vault_path: vault_path.to_string_lossy().to_string(),
            };
            (
                StatusCode::OK,
                Json(serde_json::to_value(status).unwrap_or_default()),
            )
        }
    }
}

/// POST /api/v1/google/session
///
/// Encrypts and saves Google session credentials into the local Dev Drive vault.
pub async fn save_session_handler(
    Json(payload): Json<SaveGoogleSessionRequest>,
) -> impl IntoResponse {
    if payload.psid.is_none() && payload.oauth_access_token.is_none() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "status": "error",
                "message": "At least one credential token (__Secure-1PSID or oauth_access_token) must be provided."
            })),
        );
    }

    let vault_path = resolve_google_vault_path();
    let mut creds = GoogleSessionCredentials::new();

    if let Some(p) = payload.psid {
        creds = creds.with_psid(p);
    }
    if let Some(pts) = payload.psidts {
        creds = creds.with_psidts(pts);
    }
    if let Some(s) = payload.sapisid {
        creds = creds.with_sapisid(s);
    }
    if let Some(csrf) = payload.csrf_token {
        creds = creds.with_csrf_token(csrf);
    }
    if let Some(o) = payload.oauth_access_token {
        creds = creds.with_oauth_token(o);
    }
    if let Some(e) = payload.account_email {
        creds = creds.with_email(e);
    }
    if let Some(exp) = payload.expires_at {
        creds.expires_at = Some(exp);
    }

    match GoogleSessionVault::save(&vault_path, &creds) {
        Ok(()) => {
            info!(target: "souls_server::google", "Successfully encrypted and saved Google session vault at {}", vault_path.display());
            let masked = creds.masked();
            (
                StatusCode::OK,
                Json(json!({
                    "status": "success",
                    "message": "Google session credentials safely encrypted and stored in vault.",
                    "masked_psid": masked.psid,
                    "account_email": masked.account_email,
                    "vault_path": vault_path.to_string_lossy().to_string()
                })),
            )
        }
        Err(e) => {
            error!(target: "souls_server::google", "Failed to write encrypted Google vault: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "status": "error",
                    "message": format!("Failed to save session credentials: {e}")
                })),
            )
        }
    }
}

/// POST /api/v1/google/notebooklm/rpc
///
/// Dispatches a headless Wiz batchexecute RPC request against Google NotebookLM.
pub async fn wiz_rpc_handler(Json(payload): Json<WizRpcRequestDto>) -> impl IntoResponse {
    let vault_path = resolve_google_vault_path();
    let creds = match GoogleSessionVault::load(&vault_path) {
        Ok(Some(c)) if c.is_configured() => c,
        _ => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "status": "unauthorized",
                    "message": "Google session credentials not found or unconfigured in vault. Please authenticate via POST /api/v1/google/session."
                })),
            );
        }
    };

    let rpc_id = resolve_wiz_rpc_id(&payload.rpc_id);
    let rpc_payload = WizRpcPayload::new(rpc_id.clone(), payload.arguments);

    let mut client = WizRpcClient::new(creds);
    if let Some(ep) = payload.endpoint_override {
        client = client.with_endpoint(ep);
    }

    info!(
        target: "souls_server::google",
        "Dispatching Wiz RPC '{}' ({})",
        rpc_id.as_str(),
        rpc_id.description()
    );

    match client.execute(&rpc_payload).await {
        Ok(resp) => (
            StatusCode::OK,
            Json(json!({
                "status": "success",
                "rpc_id": resp.rpc_id,
                "data": resp.payload,
                "raw_response": resp.raw_response
            })),
        ),
        Err(e) => {
            error!(target: "souls_server::google", "Wiz RPC execution failed: {e}");
            (
                StatusCode::BAD_GATEWAY,
                Json(json!({
                    "status": "error",
                    "rpc_id": rpc_id.as_str(),
                    "message": format!("Wiz RPC execution error: {e}")
                })),
            )
        }
    }
}

/// GET /api/v1/google/workspace/status
///
/// Returns dual-layer workspace state binding Dev Drive ReFS to Google Drive.
pub async fn workspace_status_handler() -> impl IntoResponse {
    let ws = DualLayerWorkspace::new(r"Z:\souls_engine");
    (
        StatusCode::OK,
        Json(json!({
            "status": "active",
            "local_root": ws.local_root,
            "refs_block_cloning_enabled": ws.refs_block_cloning_enabled,
            "google_drive_folder_id": ws.google_drive_folder_id,
            "linked_documents_count": ws.linked_document_ids.len(),
            "jules_migration_status": "migrated_to_antigravity_agy",
            "jules_decommission_date": JulesMigrationRouter::JULES_DECOMMISSION_DATE
        })),
    )
}

/// POST /api/v1/google/workspace/link
///
/// Links a Google Drive folder or Doc/Sheet ID to the dual-layer workspace.
pub async fn workspace_link_handler(
    Json(payload): Json<WorkspaceLinkRequestDto>,
) -> impl IntoResponse {
    let local_path = payload
        .local_path
        .unwrap_or_else(|| r"Z:\souls_engine".to_string());
    let mut ws = DualLayerWorkspace::new(PathBuf::from(local_path));

    if let Some(folder) = payload.drive_folder_id {
        ws = ws.with_drive_folder(folder);
    }
    if let Some(doc) = payload.linked_document_id {
        ws = ws.with_linked_document(doc);
    }

    (
        StatusCode::OK,
        Json(json!({
            "status": "success",
            "workspace": ws
        })),
    )
}

/// POST /api/v1/google/jules/migrate
///
/// Transparently migrates deprecated Google Jules command requests to Antigravity CLI (`agy`).
pub async fn jules_migrate_handler(
    Json(payload): Json<JulesMigrationRequestDto>,
) -> impl IntoResponse {
    let is_jules = JulesMigrationRouter::is_legacy_jules_command(&payload.command);

    if !is_jules {
        return (
            StatusCode::OK,
            Json(json!({
                "is_legacy_jules": false,
                "message": "Command does not reference Google Jules."
            })),
        );
    }

    let working_dir = payload
        .working_dir
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"Z:\souls_engine"));

    let jules_cmd = LegacyJulesCommand {
        prompt: payload.command.clone(),
        working_dir: Some(working_dir.clone()),
        auto_approve: true,
        model: None,
    };

    let agy_cmd = JulesMigrationRouter::migrate_to_antigravity(jules_cmd, &working_dir);
    let target_args = AgenticHarnessDispatcher::build_arguments(&agy_cmd);

    (
        StatusCode::OK,
        Json(json!({
            "is_legacy_jules": true,
            "decommission_date": JulesMigrationRouter::JULES_DECOMMISSION_DATE,
            "migrated_cli": "agy",
            "target_command": target_args,
            "working_dir": agy_cmd.working_dir.to_string_lossy().to_string(),
            "notice": format!(
                "Google Jules was decommissioned on {}. Command transparently mapped to Antigravity CLI (agy).",
                JulesMigrationRouter::JULES_DECOMMISSION_DATE
            )
        })),
    )
}

// ============================================================================
// v0.9.2 MULTI-ACCOUNT VAULT, OAUTH2 PKCE & DOCTOR
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct UpsertAccountRequest {
    pub account_id: Option<String>,
    pub label: Option<String>,
    pub email: Option<String>,
    pub kind: Option<GoogleAccountKind>,
    pub policy: Option<AccountPermissionPolicy>,
    pub is_primary: Option<bool>,
    pub secondary_web_opt_in: Option<bool>,
    pub write_whitelist_ids: Option<Vec<String>>,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub oauth_access_token: Option<String>,
    pub oauth_refresh_token: Option<String>,
    pub token_expires_at: Option<i64>,
    pub scopes: Option<Vec<String>>,
    pub oauth_testing_mode_warning: Option<bool>,
    pub psid: Option<String>,
    pub psidts: Option<String>,
    pub sapisid: Option<String>,
    pub csrf_token: Option<String>,
    pub has_nlm_bridge: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAccountPolicyRequest {
    pub policy: AccountPermissionPolicy,
    pub write_whitelist_ids: Option<Vec<String>>,
    pub secondary_web_opt_in: Option<bool>,
}

#[derive(Debug, Default, Deserialize)]
pub struct OAuthStartRequest {
    pub account_id: Option<String>,
    pub label: Option<String>,
    pub kind: Option<GoogleAccountKind>,
    pub policy: Option<AccountPermissionPolicy>,
    pub redirect_uri: Option<String>,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub scopes: Option<Vec<String>>,
    pub login_hint: Option<String>,
    pub auth_endpoint_override: Option<String>,
    pub token_endpoint_override: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct OAuthCallbackQuery {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct ImportGwsRequest {
    pub gws_dir: Option<String>,
    pub account_id: Option<String>,
    pub label: Option<String>,
    pub is_primary: Option<bool>,
}

/// `GET /api/v1/google/accounts`
pub async fn list_accounts_handler(State(state): State<AppState>) -> impl IntoResponse {
    let vault = state.google_hub.vault.read().await;
    let accounts = vault.list_profiles();
    (
        StatusCode::OK,
        Json(json!({
            "status": "success",
            "count": accounts.len(),
            "accounts": accounts
        })),
    )
}

/// `POST /api/v1/google/accounts`
pub async fn upsert_account_handler(
    State(state): State<AppState>,
    Json(payload): Json<UpsertAccountRequest>,
) -> impl IntoResponse {
    let email_str = payload.email.clone().unwrap_or_default().trim().to_string();
    let derived_id = payload
        .account_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToString::to_string)
        .or_else(|| {
            if !email_str.is_empty() {
                Some(sanitize_account_id(&email_str))
            } else {
                payload
                    .label
                    .as_deref()
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(sanitize_account_id)
            }
        });

    let Some(account_id) = derived_id else {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "status": "error",
                "message": "account_id, email, or label is required"
            })),
        );
    };

    let mut vault = state.google_hub.vault.write().await;
    let mut record = vault
        .get_account(&account_id)
        .cloned()
        .unwrap_or_else(|| {
            let kind = payload.kind.unwrap_or_else(|| infer_kind_from_email(&email_str));
            GoogleAccountRecord::new(account_id.clone(), email_str.clone(), kind)
        });

    if let Some(label) = payload.label.filter(|s| !s.trim().is_empty()) {
        record.label = label.trim().to_string();
    }
    if !email_str.is_empty() {
        record.email = email_str;
    }
    if let Some(kind) = payload.kind {
        record.kind = kind;
    }
    if let Some(policy) = payload.policy {
        record.policy = policy;
    }
    if let Some(is_primary) = payload.is_primary {
        record.is_primary = is_primary;
    }
    if let Some(opt_in) = payload.secondary_web_opt_in {
        record.secondary_web_opt_in = !record.is_primary && opt_in;
    }
    if record.is_primary {
        record.secondary_web_opt_in = false;
    }
    if let Some(wl) = payload.write_whitelist_ids {
        record.write_whitelist_ids = wl;
    }
    if let Some(cid) = payload.client_id {
        record.client_id = Some(cid);
    }
    if let Some(cs) = payload.client_secret {
        record.client_secret = Some(cs);
    }
    if let Some(tok) = payload.oauth_access_token {
        record.oauth_access_token = Some(tok);
    }
    if let Some(rtok) = payload.oauth_refresh_token {
        record.oauth_refresh_token = Some(rtok);
    }
    if let Some(exp) = payload.token_expires_at {
        record.token_expires_at = Some(exp);
    }
    if let Some(scopes) = payload.scopes {
        record.scopes = scopes;
    }
    if let Some(warn_flag) = payload.oauth_testing_mode_warning {
        record.oauth_testing_mode_warning = warn_flag;
    }
    if let Some(psid) = payload.psid {
        record.psid = Some(psid);
        record.has_nlm_bridge = true;
    }
    if let Some(psidts) = payload.psidts {
        record.psidts = Some(psidts);
    }
    if let Some(sapisid) = payload.sapisid {
        record.sapisid = Some(sapisid);
        record.has_nlm_bridge = true;
    }
    if let Some(csrf) = payload.csrf_token {
        record.csrf_token = csrf;
    }
    if let Some(nlm) = payload.has_nlm_bridge {
        record.has_nlm_bridge = nlm;
    }
    record.updated_at = Utc::now().timestamp();

    let public_dto = record.to_public_dto();
    if let Err(err) = vault.upsert_account(record) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "status": "error",
                "message": format!("Failed to persist account in GV2 vault: {err}")
            })),
        );
    }
    drop(vault);
    state.google_hub.invalidate_doctor_cache().await;

    (
        StatusCode::OK,
        Json(json!({
            "status": "success",
            "account": public_dto
        })),
    )
}

/// `DELETE /api/v1/google/accounts/:id`
pub async fn delete_account_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let mut vault = state.google_hub.vault.write().await;
    let res = vault.remove_account(&id);
    drop(vault);
    match res {
        Ok(true) => {
            state.google_hub.invalidate_doctor_cache().await;
            (
                StatusCode::OK,
                Json(json!({
                    "status": "success",
                    "removed": true,
                    "account_id": id
                })),
            )
        }
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "status": "error",
                "message": format!("Google account '{id}' not found")
            })),
        ),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "status": "error",
                "message": format!("Failed to remove account '{id}': {err}")
            })),
        ),
    }
}

/// `POST /api/v1/google/accounts/:id/policy`
pub async fn update_account_policy_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateAccountPolicyRequest>,
) -> impl IntoResponse {
    let mut vault = state.google_hub.vault.write().await;
    let Some(mut record) = vault.get_account(&id).cloned() else {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({
                "status": "error",
                "message": format!("Google account '{id}' not found")
            })),
        );
    };

    if payload.secondary_web_opt_in == Some(true) && record.is_primary {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({
                "status": "error",
                "message": "secondary_web_opt_in cannot be enabled on a primary Google account"
            })),
        );
    }

    record.policy = payload.policy;
    if let Some(wl) = payload.write_whitelist_ids {
        record.write_whitelist_ids = wl;
    }
    if let Some(opt_in) = payload.secondary_web_opt_in {
        record.secondary_web_opt_in = opt_in;
    }
    record.updated_at = Utc::now().timestamp();

    let public_dto = record.to_public_dto();
    if let Err(err) = vault.upsert_account(record) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "status": "error",
                "message": format!("Failed to save policy update: {err}")
            })),
        );
    }

    (
        StatusCode::OK,
        Json(json!({
            "status": "success",
            "account": public_dto
        })),
    )
}

/// `POST /api/v1/google/oauth/start`
pub async fn oauth_start_handler(
    State(state): State<AppState>,
    body: Option<Json<OAuthStartRequest>>,
) -> impl IntoResponse {
    let req = body.map(|Json(b)| b).unwrap_or_default();
    let pkce = PkceChallenge::generate();
    let redirect_uri = req
        .redirect_uri
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("http://127.0.0.1:9123/api/v1/google/oauth/callback")
        .to_string();

    let label = req
        .label
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("Google Account")
        .to_string();
    let kind = req.kind.unwrap_or(GoogleAccountKind::PersonalGmail);
    let policy = req.policy.unwrap_or(AccountPermissionPolicy::BalancedHotlHitl);

    let gws_imported = import_gws_credentials().ok().flatten();
    let configured_client_id = req
        .client_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToString::to_string)
        .or_else(|| env_opt("GOOGLE_OAUTH_CLIENT_ID"))
        .or_else(|| env_opt("GOOGLE_CLIENT_ID"))
        .or_else(|| gws_imported.as_ref().and_then(|g| g.client_id.clone()));

    let client_secret = req
        .client_secret
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToString::to_string)
        .or_else(|| env_opt("GOOGLE_OAUTH_CLIENT_SECRET"))
        .or_else(|| env_opt("GOOGLE_CLIENT_SECRET"))
        .or_else(|| gws_imported.as_ref().and_then(|g| g.client_secret.clone()));

    let scopes = req.scopes.unwrap_or_else(default_google_workspace_scopes);

    let effective_client_id = configured_client_id
        .clone()
        .unwrap_or_else(|| "souls-engine-unconfigured-client".to_string());

    let mut cfg = OAuth2Config::google_default(
        effective_client_id,
        client_secret.clone(),
        redirect_uri.clone(),
        scopes.clone(),
    );
    if req.auth_endpoint_override.is_some() || req.token_endpoint_override.is_some() {
        let auth_ep = req
            .auth_endpoint_override
            .unwrap_or_else(|| cfg.auth_endpoint.clone());
        let token_ep = req
            .token_endpoint_override
            .clone()
            .unwrap_or_else(|| cfg.token_endpoint.clone());
        cfg = cfg.with_endpoints(auth_ep, token_ep);
    }

    let auth_url = build_google_auth_url(&cfg, &pkce, req.login_hint.as_deref());
    let pending = PendingPkce {
        state: pkce.state.clone(),
        code_verifier: pkce.code_verifier,
        redirect_uri: redirect_uri.clone(),
        label,
        kind,
        policy,
        created_at: Utc::now().timestamp(),
    };

    {
        let mut guard = state.google_hub.pkce_states.write().await;
        guard.insert(pkce.state.clone(), pending);
    }
    state
        .google_hub
        .store_pkce_meta(
            &pkce.state,
            PkceOAuthMeta {
                account_id: req.account_id,
                client_id: configured_client_id.clone(),
                client_secret,
                token_endpoint: req.token_endpoint_override,
                scopes,
            },
        )
        .await;

    (
        StatusCode::OK,
        Json(json!({
            "status": "success",
            "auth_url": auth_url,
            "state": pkce.state,
            "redirect_uri": redirect_uri,
            "code_challenge_method": "S256",
            "client_id_configured": configured_client_id.is_some()
        })),
    )
}

/// `GET /api/v1/google/oauth/callback`
pub async fn oauth_callback_handler(
    State(state): State<AppState>,
    Query(query): Query<OAuthCallbackQuery>,
) -> impl IntoResponse {
    if let Some(err) = query.error.as_deref().filter(|s| !s.trim().is_empty()) {
        return (
            StatusCode::BAD_REQUEST,
            Html(render_oauth_html(
                "Autorização Google Cancelada",
                &format!("O provedor OAuth2 retornou um erro: {}", escape_html(err)),
                false,
            )),
        );
    }

    let Some(state_param) = query.state.as_deref().filter(|s| !s.trim().is_empty()) else {
        return (
            StatusCode::BAD_REQUEST,
            Html(render_oauth_html(
                "Estado PKCE Ausente",
                "O parâmetro obrigatório 'state' não foi informado no callback OAuth2.",
                false,
            )),
        );
    };

    let Some((pending, meta)) = state.google_hub.take_pkce(state_param).await else {
        return (
            StatusCode::BAD_REQUEST,
            Html(render_oauth_html(
                "Estado PKCE Inválido ou Expirado",
                "O token CSRF 'state' não corresponde a uma sessão PKCE pendente.",
                false,
            )),
        );
    };

    let now = Utc::now().timestamp();
    if now - pending.created_at > 900 {
        return (
            StatusCode::BAD_REQUEST,
            Html(render_oauth_html(
                "Sessão PKCE Expirada",
                "O desafio PKCE expirou (>15 minutos). Reinicie o fluxo de autorização.",
                false,
            )),
        );
    }

    let Some(code) = query.code.as_deref().filter(|s| !s.trim().is_empty()) else {
        return (
            StatusCode::BAD_REQUEST,
            Html(render_oauth_html(
                "Código de Autorização Ausente",
                "O parâmetro 'code' não foi recebido do Google OAuth2.",
                false,
            )),
        );
    };

    let gws_imported = import_gws_credentials().ok().flatten();
    let client_id = meta
        .as_ref()
        .and_then(|m| m.client_id.clone())
        .or_else(|| env_opt("GOOGLE_OAUTH_CLIENT_ID"))
        .or_else(|| env_opt("GOOGLE_CLIENT_ID"))
        .or_else(|| gws_imported.as_ref().and_then(|g| g.client_id.clone()));

    let client_secret = meta
        .as_ref()
        .and_then(|m| m.client_secret.clone())
        .or_else(|| env_opt("GOOGLE_OAUTH_CLIENT_SECRET"))
        .or_else(|| env_opt("GOOGLE_CLIENT_SECRET"))
        .or_else(|| gws_imported.as_ref().and_then(|g| g.client_secret.clone()));

    let Some(cid) = client_id else {
        return (
            StatusCode::OK,
            Html(render_oauth_html(
                "Desafio PKCE Validado (Client ID Não Configurado)",
                "O state PKCE foi validado com sucesso, porém GOOGLE_OAUTH_CLIENT_ID não está configurado para concluir a troca remota do authorization_code.",
                true,
            )),
        );
    };

    let scopes = meta
        .as_ref()
        .map(|m| m.scopes.clone())
        .unwrap_or_default();
    let mut cfg = OAuth2Config::google_default(cid.clone(), client_secret.clone(), pending.redirect_uri.clone(), scopes.clone());
    if let Some(ref token_ep) = meta.as_ref().and_then(|m| m.token_endpoint.clone()) {
        let auth_ep = cfg.auth_endpoint.clone();
        cfg = cfg.with_endpoints(auth_ep, token_ep.clone());
    }

    let http = reqwest::Client::new();
    let token_resp = match exchange_code_for_tokens(&http, &cfg, code, &pending.code_verifier).await {
        Ok(tok) => tok,
        Err(err) => {
            return (
                StatusCode::BAD_GATEWAY,
                Html(render_oauth_html(
                    "Falha na Troca OAuth2",
                    &format!("Não foi possível trocar o authorization_code por tokens: {}", escape_html(&err.to_string())),
                    false,
                )),
            );
        }
    };

    let acct_id = meta
        .as_ref()
        .and_then(|m| m.account_id.clone())
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| sanitize_account_id(&pending.label));
    let email = query.email.unwrap_or_default();

    let mut vault = state.google_hub.vault.write().await;
    let mut record = vault
        .get_account(&acct_id)
        .cloned()
        .unwrap_or_else(|| GoogleAccountRecord::new(acct_id.clone(), email.clone(), pending.kind));

    record.label = pending.label.clone();
    if !email.trim().is_empty() {
        record.email = email;
    }
    record.kind = pending.kind;
    record.policy = pending.policy;
    record.is_primary = vault.accounts().is_empty() || record.is_primary;
    if record.is_primary {
        record.secondary_web_opt_in = false;
    }
    record.client_id = Some(cid);
    record.client_secret = client_secret;
    record.oauth_access_token = Some(token_resp.access_token);
    if let Some(refresh) = token_resp.refresh_token {
        record.oauth_refresh_token = Some(refresh);
    }
    record.token_expires_at = token_resp.expires_in.map(|secs| now.saturating_add(secs));
    if !scopes.is_empty() {
        record.scopes = scopes;
    }
    record.oauth_testing_mode_warning = true;
    record.updated_at = now;

    if let Err(err) = vault.upsert_account(record) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Html(render_oauth_html(
                "Erro ao Persistir Cofre",
                &format!("Tokens recebidos, mas falha ao gravar o cofre local: {}", escape_html(&err.to_string())),
                false,
            )),
        );
    }
    drop(vault);
    state.google_hub.invalidate_doctor_cache().await;

    (
        StatusCode::OK,
        Html(render_oauth_html(
            "Conta Google Conectada com Sucesso",
            &format!(
                "Sua conta Google '{}' ({}) foi conectada e salva com segurança. Você já pode fechar esta janela e voltar ao painel.",
                escape_html(&pending.label),
                escape_html(&acct_id)
            ),
            true,
        )),
    )
}

#[derive(Debug, Default, Deserialize)]
pub struct DoctorQuery {
    pub refresh: Option<bool>,
}

/// `GET /api/v1/google/doctor`
pub async fn google_doctor_handler(
    State(state): State<AppState>,
    Query(query): Query<DoctorQuery>,
) -> impl IntoResponse {
    let report = state
        .google_hub
        .get_doctor_report(query.refresh.unwrap_or(false))
        .await;
    let gemini_api_key_ready = env_opt("GEMINI_API_KEY").is_some();
    let maps_api_key_ready = env_opt("ROUTES_API_KEY").is_some() || env_opt("GOOGLE_MAPS_API_KEY").is_some();

    let mut val = serde_json::to_value(&report).unwrap_or_default();
    if let Some(obj) = val.as_object_mut() {
        obj.insert("gemini_api_key_ready".to_string(), json!(gemini_api_key_ready));
        obj.insert("maps_api_key_ready".to_string(), json!(maps_api_key_ready));
    }
    (StatusCode::OK, Json(val))
}

#[derive(Debug, Default, Deserialize)]
pub struct NlmLoginRequest {
    pub force: Option<bool>,
}

/// `POST /api/v1/google/nlm/login`
///
/// One-click helper to check or launch the interactive Google login window for NotebookLM (`nlm`).
pub async fn nlm_login_handler(
    State(state): State<AppState>,
    body: Option<Json<NlmLoginRequest>>,
) -> impl IntoResponse {
    let force = body.map(|Json(b)| b.force.unwrap_or(false)).unwrap_or(false);
    let Some(nlm_path) = find_binary_on_path("nlm") else {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({
                "status": "not_installed",
                "message": "A ferramenta nlm (NotebookLM) não foi encontrada neste computador."
            })),
        );
    };

    if !force {
        let path_clone = nlm_path.clone();
        let already_ok = tokio::task::spawn_blocking(move || {
            let mut cmd = std::process::Command::new(&path_clone);
            cmd.args(["login", "--check"])
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped());
            let Ok(mut child) = cmd.spawn() else {
                return false;
            };
            let start = std::time::Instant::now();
            loop {
                match child.try_wait() {
                    Ok(Some(st)) => return st.success(),
                    Ok(None) => {
                        if start.elapsed() >= std::time::Duration::from_secs(8) {
                            let _ = child.kill();
                            let _ = child.wait();
                            return false;
                        }
                        std::thread::sleep(std::time::Duration::from_millis(50));
                    }
                    Err(_) => return false,
                }
            }
        })
        .await
        .unwrap_or(false);

        if already_ok {
            state.google_hub.invalidate_doctor_cache().await;
            return (
                StatusCode::OK,
                Json(json!({
                    "status": "already_authenticated",
                    "message": "NotebookLM já está conectado e pronto para uso."
                })),
            );
        }
    }

    let spawn_res = std::process::Command::new(&nlm_path)
        .arg("login")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn();

    state.google_hub.invalidate_doctor_cache().await;

    match spawn_res {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({
                "status": "browser_opened",
                "message": "Janela do Chrome aberta para você entrar na sua conta Google. Assim que concluir, clique em Atualizar."
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "status": "error",
                "message": format!("Não foi possível abrir o login do NotebookLM: {e}")
            })),
        ),
    }
}

#[derive(Debug, Deserialize)]
pub struct UploadOAuthCredentialsRequest {
    pub client_secret_json: String,
    pub account_id: Option<String>,
    pub label: Option<String>,
    pub email: Option<String>,
    pub kind: Option<GoogleAccountKind>,
    pub policy: Option<AccountPermissionPolicy>,
    pub auto_start_oauth: Option<bool>,
}

/// `POST /api/v1/google/oauth/upload-credentials`
///
fn default_google_workspace_scopes() -> Vec<String> {
    vec![
        "openid".to_string(),
        "email".to_string(),
        "profile".to_string(),
        "https://www.googleapis.com/auth/gmail.modify".to_string(),
        "https://www.googleapis.com/auth/calendar".to_string(),
        "https://www.googleapis.com/auth/drive.readonly".to_string(),
        "https://www.googleapis.com/auth/documents".to_string(),
        "https://www.googleapis.com/auth/spreadsheets".to_string(),
        "https://www.googleapis.com/auth/presentations".to_string(),
        "https://www.googleapis.com/auth/tasks".to_string(),
        "https://www.googleapis.com/auth/contacts.readonly".to_string(),
    ]
}

/// Parses an official Google Cloud `client_secret.json`, persists it in the vault, and optionally
/// starts the OAuth2 PKCE flow returning `auth_url` in a single step.
pub async fn oauth_upload_credentials_handler(
    State(state): State<AppState>,
    Json(payload): Json<UploadOAuthCredentialsRequest>,
) -> impl IntoResponse {
    let parsed = match parse_google_client_secret_json(&payload.client_secret_json) {
        Ok(p) => p,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "status": "error",
                    "message": format!("Arquivo JSON inválido: {e}")
                })),
            );
        }
    };

    let email_str = payload.email.clone().unwrap_or_default().trim().to_string();
    let label_str = payload
        .label
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("Conta Google")
        .to_string();

    let acct_id = payload
        .account_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToString::to_string)
        .unwrap_or_else(|| {
            if !email_str.is_empty() {
                sanitize_account_id(&email_str)
            } else {
                sanitize_account_id(&label_str)
            }
        });

    let kind = payload.kind.unwrap_or_else(|| infer_kind_from_email(&email_str));
    let policy = payload.policy.unwrap_or(AccountPermissionPolicy::BalancedHotlHitl);

    let mut vault = state.google_hub.vault.write().await;
    let mut record = vault
        .get_account(&acct_id)
        .cloned()
        .unwrap_or_else(|| GoogleAccountRecord::new(acct_id.clone(), email_str.clone(), kind));
    record.label = label_str.clone();
    if !email_str.is_empty() {
        record.email = email_str.clone();
    }
    record.kind = kind;
    record.policy = policy;
    record.is_primary = vault.accounts().is_empty() || record.is_primary;
    if record.is_primary {
        record.secondary_web_opt_in = false;
    }
    record.client_id = Some(parsed.client_id.clone());
    if parsed.client_secret.is_some() {
        record.client_secret = parsed.client_secret.clone();
    }
    record.updated_at = Utc::now().timestamp();
    let public_dto = record.to_public_dto();

    if let Err(err) = vault.upsert_account(record) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "status": "error",
                "message": format!("Falha ao salvar credenciais no cofre local: {err}")
            })),
        );
    }
    drop(vault);
    state.google_hub.invalidate_doctor_cache().await;

    let mut auth_url_opt = None;
    let mut state_opt = None;
    let redirect_uri = "http://127.0.0.1:9123/api/v1/google/oauth/callback".to_string();

    if payload.auto_start_oauth.unwrap_or(true) {
        let pkce = PkceChallenge::generate();
        let scopes = default_google_workspace_scopes();
        let cfg = OAuth2Config::google_default(
            parsed.client_id.clone(),
            parsed.client_secret.clone(),
            redirect_uri.clone(),
            scopes.clone(),
        );
        let hint = if email_str.is_empty() { None } else { Some(email_str.as_str()) };
        let url = build_google_auth_url(&cfg, &pkce, hint);
        let pending = PendingPkce {
            state: pkce.state.clone(),
            code_verifier: pkce.code_verifier,
            redirect_uri: redirect_uri.clone(),
            label: label_str,
            kind,
            policy,
            created_at: Utc::now().timestamp(),
        };
        {
            let mut guard = state.google_hub.pkce_states.write().await;
            guard.insert(pkce.state.clone(), pending);
        }
        state
            .google_hub
            .store_pkce_meta(
                &pkce.state,
                PkceOAuthMeta {
                    account_id: Some(acct_id),
                    client_id: Some(parsed.client_id),
                    client_secret: parsed.client_secret,
                    token_endpoint: None,
                    scopes,
                },
            )
            .await;

        auth_url_opt = Some(url);
        state_opt = Some(pkce.state);
    }

    (
        StatusCode::OK,
        Json(json!({
            "status": "success",
            "account": public_dto,
            "auth_url": auth_url_opt,
            "state": state_opt,
            "redirect_uri": redirect_uri
        })),
    )
}

/// `POST /api/v1/google/gws/login`
///
/// Launches `gws auth login` when `client_secret.json` is already available in `~/.config/gws`,
/// or returns friendly guidance to upload `client_secret.json` via the 1-click wizard.
pub async fn gws_login_handler() -> impl IntoResponse {
    let Some(gws_path) = find_binary_on_path("gws") else {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({
                "status": "not_installed",
                "message": "A CLI gws não foi encontrada no computador."
            })),
        );
    };

    let has_client_secret = std::env::var("USERPROFILE")
        .ok()
        .or_else(|| std::env::var("HOME").ok())
        .map(|h| PathBuf::from(h).join(".config").join("gws").join("client_secret.json").exists())
        .unwrap_or(false);

    if !has_client_secret {
        return (
            StatusCode::OK,
            Json(json!({
                "status": "needs_setup",
                "message": "Ainda não há um client_secret.json na pasta do gws. Envie ou cole o arquivo JSON do Google Cloud acima para conectar em 1 clique."
            })),
        );
    }

    let spawn_res = std::process::Command::new(&gws_path)
        .args(["auth", "login"])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn();

    match spawn_res {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({
                "status": "browser_opened",
                "message": "Navegador aberto via gws auth login. Ao finalizar o login, clique em Importar do gws."
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "status": "error",
                "message": format!("Falha ao iniciar gws auth login: {e}")
            })),
        ),
    }
}

/// `POST /api/v1/google/accounts/import-gws`
pub async fn import_gws_handler(
    State(state): State<AppState>,
    body: Option<Json<ImportGwsRequest>>,
) -> impl IntoResponse {
    let req = body.map(|Json(b)| b).unwrap_or_default();
    let imported_res = match req
        .gws_dir
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        Some(dir) => import_gws_credentials_from_dir(&PathBuf::from(dir)),
        None => import_gws_credentials(),
    };

    let imported = match imported_res {
        Ok(Some(cfg)) => cfg,
        Ok(None) => {
            return (
                StatusCode::OK,
                Json(json!({
                    "status": "not_found",
                    "message": "No gws credentials or client_secret.json found."
                })),
            );
        }
        Err(err) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "status": "error",
                    "message": err.to_string()
                })),
            );
        }
    };

    let email = imported.email.clone().unwrap_or_default();
    let acct_id = req
        .account_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToString::to_string)
        .unwrap_or_else(|| {
            if !email.is_empty() {
                sanitize_account_id(&email)
            } else {
                "gws_imported".to_string()
            }
        });

    let mut vault = state.google_hub.vault.write().await;
    let kind = infer_kind_from_email(&email);
    let mut record = vault
        .get_account(&acct_id)
        .cloned()
        .unwrap_or_else(|| GoogleAccountRecord::new(acct_id.clone(), email.clone(), kind));

    if let Some(lbl) = req.label.filter(|s| !s.trim().is_empty()) {
        record.label = lbl;
    } else if record.label.is_empty() {
        record.label = "Imported gws Account".to_string();
    }
    if !email.is_empty() {
        record.email = email;
    }
    if let Some(is_p) = req.is_primary {
        record.is_primary = is_p;
    } else if vault.accounts().is_empty() {
        record.is_primary = true;
    }
    if record.is_primary {
        record.secondary_web_opt_in = false;
    }
    if imported.client_id.is_some() {
        record.client_id = imported.client_id;
    }
    if imported.client_secret.is_some() {
        record.client_secret = imported.client_secret;
    }
    if imported.access_token.is_some() {
        record.oauth_access_token = imported.access_token;
    }
    if imported.refresh_token.is_some() {
        record.oauth_refresh_token = imported.refresh_token;
    }
    if !imported.scopes.is_empty() {
        record.scopes = imported.scopes;
    }
    record.updated_at = Utc::now().timestamp();

    let public_dto = record.to_public_dto();
    if let Err(err) = vault.upsert_account(record) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "status": "error",
                "message": format!("Failed to persist imported gws credentials: {err}")
            })),
        );
    }
    drop(vault);
    state.google_hub.invalidate_doctor_cache().await;

    (
        StatusCode::OK,
        Json(json!({
            "status": "imported",
            "account": public_dto
        })),
    )
}

// ============================================================================
// v0.9.2 LIFE HUB ENDPOINTS
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct BriefingQuery {
    pub account_id: Option<String>,
}

/// `GET /api/v1/google/briefing`
pub async fn briefing_handler(
    State(state): State<AppState>,
    Query(query): Query<BriefingQuery>,
) -> impl IntoResponse {
    let briefing = state
        .google_hub
        .build_daily_briefing(query.account_id.as_deref())
        .await;
    (StatusCode::OK, Json(briefing))
}

#[derive(Debug, Deserialize)]
pub struct GmailTriageQuery {
    pub account_id: Option<String>,
    pub query: Option<String>,
    pub max_results: Option<usize>,
    pub session_id: Option<String>,
    pub api_base_override: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GmailComposeRequest {
    pub account_id: Option<String>,
    pub to: String,
    pub subject: String,
    pub body: String,
    pub session_id: Option<String>,
    pub api_base_override: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GmailBulkRequest {
    pub account_id: Option<String>,
    pub ids: Vec<String>,
    #[serde(default)]
    pub add_labels: Vec<String>,
    #[serde(default)]
    pub remove_labels: Vec<String>,
    pub session_id: Option<String>,
    pub api_base_override: Option<String>,
}

/// `GET /api/v1/google/gmail/triage`
pub async fn gmail_triage_handler(
    State(state): State<AppState>,
    Query(query): Query<GmailTriageQuery>,
) -> impl IntoResponse {
    let account = state
        .google_hub
        .resolve_account(query.account_id.as_deref())
        .await;
    let Some(acct) = account else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "status": "unauthorized",
                "message": "No Google account configured for Gmail triage."
            })),
        );
    };
    let Some(token) = acct
        .oauth_access_token
        .as_deref()
        .map(str::trim)
        .filter(|t| !t.is_empty())
    else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "status": "unauthorized",
                "message": "Account has no active OAuth2 access token for Gmail triage."
            })),
        );
    };

    let ws = workspace_client_from_override(query.api_base_override.as_deref());
    let q = query.query.as_deref().unwrap_or("in:inbox");
    let max_res = query.max_results.unwrap_or(20).clamp(1, 50);

    match ws.list_messages(token, q, max_res).await {
        Ok(msgs) => {
            let sess = query.session_id.as_deref().unwrap_or("web_dashboard");
            state
                .google_hub
                .taint_tracker
                .mark_tainted(sess, "gmail:triage");
            let mut result: GmailTriageResultDto =
                GoogleWorkspaceClient::triage_inbox_deterministically(&msgs);
            result.account_email = acct.email;
            state.google_hub.record_audit_event(
                sess,
                &acct.account_id,
                "gmail.triage",
                BlastRadiusTier::Tier0ReadOnly,
                true,
                "allowed",
                &format!("Triaged {} Gmail message(s)", result.total_scanned),
            );
            (StatusCode::OK, Json(serde_json::to_value(result).unwrap_or_default()))
        }
        Err(err) => (
            StatusCode::BAD_GATEWAY,
            Json(json!({
                "status": "error",
                "message": format!("Gmail triage failed: {err}")
            })),
        ),
    }
}

/// `POST /api/v1/google/gmail/draft`
pub async fn gmail_draft_handler(
    State(state): State<AppState>,
    Json(payload): Json<GmailComposeRequest>,
) -> impl IntoResponse {
    let sess = payload.session_id.as_deref().unwrap_or("web_dashboard");
    let (acct_id, policy, token_opt) = resolve_policy_context(&state, payload.account_id.as_deref()).await;
    let is_tainted = state.google_hub.taint_tracker.is_tainted(sess);

    match evaluate_action_policy(BlastRadiusTier::Tier1SafeReversible, policy, is_tainted, false) {
        PolicyVerdict::Denied { reason } => (
            StatusCode::FORBIDDEN,
            Json(json!({ "status": "forbidden", "message": reason })),
        ),
        PolicyVerdict::RequireBmadInbox { .. } => {
            let after_text = format!(
                "To: {}\nSubject: {}\n\n{}",
                payload.to, payload.subject, payload.body
            );
            let input = NewProposalInput {
                account_id: acct_id.clone(),
                service: "gmail".to_string(),
                action: "create_draft".to_string(),
                title: format!("Gmail Draft -> {}", payload.to),
                summary: format!("Subject: {}", payload.subject),
                tier: BlastRadiusTier::Tier1SafeReversible,
                tainted_origin: is_tainted,
                target_resource: payload.to.clone(),
                before_snapshot: Some(String::new()),
                after_snapshot: Some(after_text),
                myers_diff: None,
                payload_json: json!({
                    "to": payload.to,
                    "subject": payload.subject,
                    "body": payload.body,
                    "api_base_override": payload.api_base_override
                }),
                ttl_secs: 600,
            };
            queue_proposal_response(&state, sess, "gmail.draft", input)
        }
        PolicyVerdict::AllowDirect => {
            let Some(token) = token_opt else {
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(json!({
                        "status": "unauthorized",
                        "message": "No active OAuth token configured for Gmail draft."
                    })),
                );
            };
            let ws = workspace_client_from_override(payload.api_base_override.as_deref());
            match ws
                .create_draft(&token, &payload.to, &payload.subject, &payload.body)
                .await
            {
                Ok(resp) => {
                    state.google_hub.record_audit_event(
                        sess,
                        &acct_id,
                        "gmail.draft",
                        BlastRadiusTier::Tier1SafeReversible,
                        false,
                        "executed",
                        &format!("Created Gmail draft for {}", payload.to),
                    );
                    (
                        StatusCode::OK,
                        Json(json!({ "status": "success", "draft": resp })),
                    )
                }
                Err(err) => (
                    StatusCode::BAD_GATEWAY,
                    Json(json!({ "status": "error", "message": err.to_string() })),
                ),
            }
        }
    }
}

/// `POST /api/v1/google/gmail/bulk`
pub async fn gmail_bulk_handler(
    State(state): State<AppState>,
    Json(payload): Json<GmailBulkRequest>,
) -> impl IntoResponse {
    if payload.ids.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "status": "error", "message": "ids array cannot be empty" })),
        );
    }

    let is_destructive = payload
        .add_labels
        .iter()
        .chain(payload.remove_labels.iter())
        .any(|l| l.eq_ignore_ascii_case("TRASH") || l.eq_ignore_ascii_case("SPAM"));
    let tier = if is_destructive {
        BlastRadiusTier::Tier2ExternalDestructive
    } else {
        BlastRadiusTier::Tier1SafeReversible
    };

    let sess = payload.session_id.as_deref().unwrap_or("web_dashboard");
    let (acct_id, policy, token_opt) = resolve_policy_context(&state, payload.account_id.as_deref()).await;
    let is_tainted = state.google_hub.taint_tracker.is_tainted(sess);

    match evaluate_action_policy(tier, policy, is_tainted, false) {
        PolicyVerdict::Denied { reason } => (
            StatusCode::FORBIDDEN,
            Json(json!({ "status": "forbidden", "message": reason })),
        ),
        PolicyVerdict::RequireBmadInbox { .. } => {
            let input = NewProposalInput {
                account_id: acct_id,
                service: "gmail".to_string(),
                action: "bulk_modify".to_string(),
                title: format!("Gmail Bulk Label Modify ({} msgs)", payload.ids.len()),
                summary: format!(
                    "+labels={:?} -labels={:?}",
                    payload.add_labels, payload.remove_labels
                ),
                tier,
                tainted_origin: is_tainted,
                target_resource: format!("{} message(s)", payload.ids.len()),
                before_snapshot: Some(format!("ids: {}", payload.ids.join(", "))),
                after_snapshot: Some(format!(
                    "ids: {}\n+labels: {:?}\n-labels: {:?}",
                    payload.ids.join(", "),
                    payload.add_labels,
                    payload.remove_labels
                )),
                myers_diff: None,
                payload_json: json!({
                    "ids": payload.ids,
                    "add_labels": payload.add_labels,
                    "remove_labels": payload.remove_labels,
                    "api_base_override": payload.api_base_override
                }),
                ttl_secs: 600,
            };
            queue_proposal_response(&state, sess, "gmail.bulk", input)
        }
        PolicyVerdict::AllowDirect => {
            let Some(token) = token_opt else {
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(json!({
                        "status": "unauthorized",
                        "message": "No active OAuth token configured for Gmail bulk update."
                    })),
                );
            };
            let ws = workspace_client_from_override(payload.api_base_override.as_deref());
            match ws
                .bulk_modify_labels(&token, &payload.ids, &payload.add_labels, &payload.remove_labels)
                .await
            {
                Ok(resp) => (
                    StatusCode::OK,
                    Json(json!({ "status": "success", "result": resp })),
                ),
                Err(err) => (
                    StatusCode::BAD_GATEWAY,
                    Json(json!({ "status": "error", "message": err.to_string() })),
                ),
            }
        }
    }
}

/// `POST /api/v1/google/gmail/send` (Tier 2 -> enfileira no Agent Inbox BMAD!)
pub async fn gmail_send_handler(
    State(state): State<AppState>,
    Json(payload): Json<GmailComposeRequest>,
) -> impl IntoResponse {
    let sess = payload.session_id.as_deref().unwrap_or("web_dashboard");
    let (acct_id, policy, _) = resolve_policy_context(&state, payload.account_id.as_deref()).await;
    let is_tainted = state.google_hub.taint_tracker.is_tainted(sess);

    if let PolicyVerdict::Denied { reason } = evaluate_action_policy(
        BlastRadiusTier::Tier2ExternalDestructive,
        policy,
        is_tainted,
        false,
    ) {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({ "status": "forbidden", "message": reason })),
        );
    }

    let after_text = format!(
        "To: {}\nSubject: {}\n\n{}",
        payload.to, payload.subject, payload.body
    );
    let input = NewProposalInput {
        account_id: acct_id,
        service: "gmail".to_string(),
        action: "send_message".to_string(),
        title: format!("Send Email -> {}", payload.to),
        summary: format!("Subject: {}", payload.subject),
        tier: BlastRadiusTier::Tier2ExternalDestructive,
        tainted_origin: is_tainted,
        target_resource: payload.to.clone(),
        before_snapshot: Some(String::new()),
        after_snapshot: Some(after_text),
        myers_diff: None,
        payload_json: json!({
            "to": payload.to,
            "subject": payload.subject,
            "body": payload.body,
            "api_base_override": payload.api_base_override
        }),
        ttl_secs: 600,
    };

    queue_proposal_response(&state, sess, "gmail.send", input)
}

#[derive(Debug, Deserialize)]
pub struct CalendarListQuery {
    pub account_id: Option<String>,
    pub time_min: Option<String>,
    pub time_max: Option<String>,
    pub session_id: Option<String>,
    pub api_base_override: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CalendarCreateRequest {
    pub account_id: Option<String>,
    pub summary: String,
    pub start_rfc3339: String,
    pub end_rfc3339: String,
    pub location: Option<String>,
    #[serde(default)]
    pub attendees: Vec<String>,
    pub send_updates: Option<String>,
    pub session_id: Option<String>,
    pub api_base_override: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct FocusSlotsQuery {
    pub account_id: Option<String>,
    pub work_start: Option<String>,
    pub work_end: Option<String>,
    pub min_duration_mins: Option<u32>,
    pub api_base_override: Option<String>,
}

/// `GET /api/v1/google/calendar/events`
pub async fn calendar_events_handler(
    State(state): State<AppState>,
    Query(query): Query<CalendarListQuery>,
) -> impl IntoResponse {
    let account = state
        .google_hub
        .resolve_account(query.account_id.as_deref())
        .await;
    let Some(acct) = account else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "status": "unauthorized",
                "message": "No Google account configured for Calendar."
            })),
        );
    };
    let Some(token) = acct
        .oauth_access_token
        .as_deref()
        .map(str::trim)
        .filter(|t| !t.is_empty())
    else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "status": "unauthorized",
                "message": "Account has no active OAuth2 access token for Calendar."
            })),
        );
    };

    let now = Utc::now();
    let time_min = query.time_min.unwrap_or_else(|| now.to_rfc3339());
    let time_max = query
        .time_max
        .unwrap_or_else(|| (now + ChronoDuration::days(7)).to_rfc3339());
    let ws = workspace_client_from_override(query.api_base_override.as_deref());

    match ws.list_events(token, &time_min, &time_max).await {
        Ok(events) => {
            let sess = query.session_id.as_deref().unwrap_or("web_dashboard");
            state
                .google_hub
                .taint_tracker
                .mark_tainted(sess, "calendar:events");
            (
                StatusCode::OK,
                Json(json!({
                    "status": "success",
                    "count": events.len(),
                    "events": events
                })),
            )
        }
        Err(err) => (
            StatusCode::BAD_GATEWAY,
            Json(json!({ "status": "error", "message": err.to_string() })),
        ),
    }
}

/// `POST /api/v1/google/calendar/events`
pub async fn calendar_create_event_handler(
    State(state): State<AppState>,
    Json(payload): Json<CalendarCreateRequest>,
) -> impl IntoResponse {
    let send_updates = payload
        .send_updates
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("none");

    let tier = if send_updates.eq_ignore_ascii_case("none") {
        BlastRadiusTier::Tier1SafeReversible
    } else {
        BlastRadiusTier::Tier2ExternalDestructive
    };

    let sess = payload.session_id.as_deref().unwrap_or("web_dashboard");
    let (acct_id, policy, token_opt) = resolve_policy_context(&state, payload.account_id.as_deref()).await;
    let is_tainted = state.google_hub.taint_tracker.is_tainted(sess);

    match evaluate_action_policy(tier, policy, is_tainted, false) {
        PolicyVerdict::Denied { reason } => (
            StatusCode::FORBIDDEN,
            Json(json!({ "status": "forbidden", "message": reason })),
        ),
        PolicyVerdict::RequireBmadInbox { .. } => {
            let after_text = format!(
                "Summary: {}\nStart: {}\nEnd: {}\nLocation: {}\nAttendees: {:?}\nsendUpdates: {}",
                payload.summary,
                payload.start_rfc3339,
                payload.end_rfc3339,
                payload.location.as_deref().unwrap_or("-"),
                payload.attendees,
                send_updates
            );
            let input = NewProposalInput {
                account_id: acct_id,
                service: "calendar".to_string(),
                action: "create_event".to_string(),
                title: format!("Calendar Event: {}", payload.summary),
                summary: format!("{} -> {} (sendUpdates={})", payload.start_rfc3339, payload.end_rfc3339, send_updates),
                tier,
                tainted_origin: is_tainted,
                target_resource: "primary".to_string(),
                before_snapshot: Some(String::new()),
                after_snapshot: Some(after_text),
                myers_diff: None,
                payload_json: json!({
                    "summary": payload.summary,
                    "start_rfc3339": payload.start_rfc3339,
                    "end_rfc3339": payload.end_rfc3339,
                    "location": payload.location,
                    "attendees": payload.attendees,
                    "send_updates": send_updates,
                    "api_base_override": payload.api_base_override
                }),
                ttl_secs: 600,
            };
            queue_proposal_response(&state, sess, "calendar.create_event", input)
        }
        PolicyVerdict::AllowDirect => {
            let Some(token) = token_opt else {
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(json!({
                        "status": "unauthorized",
                        "message": "No active OAuth token configured for Calendar event creation."
                    })),
                );
            };
            let ws = workspace_client_from_override(payload.api_base_override.as_deref());
            match ws
                .create_event(
                    &token,
                    &payload.summary,
                    &payload.start_rfc3339,
                    &payload.end_rfc3339,
                    payload.location.as_deref(),
                    &payload.attendees,
                    Some(send_updates),
                )
                .await
            {
                Ok(ev) => {
                    state.google_hub.record_audit_event(
                        sess,
                        &acct_id,
                        "calendar.create_event",
                        tier,
                        false,
                        "executed",
                        &format!("Created calendar event '{}'", payload.summary),
                    );
                    (
                        StatusCode::OK,
                        Json(json!({ "status": "success", "event": ev })),
                    )
                }
                Err(err) => (
                    StatusCode::BAD_GATEWAY,
                    Json(json!({ "status": "error", "message": err.to_string() })),
                ),
            }
        }
    }
}

/// `GET /api/v1/google/calendar/focus-slots`
pub async fn calendar_focus_slots_handler(
    State(state): State<AppState>,
    Query(query): Query<FocusSlotsQuery>,
) -> impl IntoResponse {
    let now = Utc::now();
    let work_start = query.work_start.unwrap_or_else(|| now.to_rfc3339());
    let work_end = query
        .work_end
        .unwrap_or_else(|| (now + ChronoDuration::hours(8)).to_rfc3339());
    let min_mins = query.min_duration_mins.unwrap_or(60).max(15);

    let (_, _, token_opt) = resolve_policy_context(&state, query.account_id.as_deref()).await;
    let events = if let Some(token) = token_opt {
        let ws = workspace_client_from_override(query.api_base_override.as_deref());
        ws.list_events(&token, &work_start, &work_end)
            .await
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    let slots =
        GoogleWorkspaceClient::find_deep_focus_slots(&events, &work_start, &work_end, min_mins);
    (
        StatusCode::OK,
        Json(json!({
            "status": "success",
            "count": slots.len(),
            "focus_slots": slots
        })),
    )
}

#[derive(Debug, Deserialize)]
pub struct DriveSearchQuery {
    pub account_id: Option<String>,
    pub query: Option<String>,
    pub page_size: Option<usize>,
    pub session_id: Option<String>,
    pub api_base_override: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DriveActionRequest {
    pub account_id: Option<String>,
    pub action: String,
    pub file_id: String,
    pub filename: Option<String>,
    pub role: Option<String>,
    pub perm_type: Option<String>,
    pub email_address: Option<String>,
    pub session_id: Option<String>,
    pub api_base_override: Option<String>,
}

/// `GET /api/v1/google/drive/search`
pub async fn drive_search_handler(
    State(state): State<AppState>,
    Query(query): Query<DriveSearchQuery>,
) -> impl IntoResponse {
    let account = state
        .google_hub
        .resolve_account(query.account_id.as_deref())
        .await;
    let Some(acct) = account else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "status": "unauthorized", "message": "No Google account configured for Drive." })),
        );
    };
    let Some(token) = acct
        .oauth_access_token
        .as_deref()
        .map(str::trim)
        .filter(|t| !t.is_empty())
    else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "status": "unauthorized", "message": "Account has no active OAuth2 access token for Drive." })),
        );
    };

    let ws = workspace_client_from_override(query.api_base_override.as_deref());
    let q = query.query.as_deref().unwrap_or("trashed = false");
    let size = query.page_size.unwrap_or(20);

    match ws.search_files(token, q, size).await {
        Ok(files) => {
            let sess = query.session_id.as_deref().unwrap_or("web_dashboard");
            state
                .google_hub
                .taint_tracker
                .mark_tainted(sess, "drive:search");
            (
                StatusCode::OK,
                Json(json!({ "status": "success", "count": files.len(), "files": files })),
            )
        }
        Err(err) => (
            StatusCode::BAD_GATEWAY,
            Json(json!({ "status": "error", "message": err.to_string() })),
        ),
    }
}

/// `POST /api/v1/google/drive/action` (delete/share -> Agent Inbox!)
pub async fn drive_action_handler(
    State(state): State<AppState>,
    Json(payload): Json<DriveActionRequest>,
) -> impl IntoResponse {
    let sess = payload.session_id.as_deref().unwrap_or("web_dashboard");
    let (acct_id, policy, token_opt) = resolve_policy_context(&state, payload.account_id.as_deref()).await;
    let is_tainted = state.google_hub.taint_tracker.is_tainted(sess);

    match payload.action.to_ascii_lowercase().as_str() {
        "delete" | "delete_file" | "share" | "update_permissions" => {
            if let PolicyVerdict::Denied { reason } = evaluate_action_policy(
                BlastRadiusTier::Tier2ExternalDestructive,
                policy,
                is_tainted,
                false,
            ) {
                return (
                    StatusCode::FORBIDDEN,
                    Json(json!({ "status": "forbidden", "message": reason })),
                );
            }

            let action_norm = if payload.action.contains("delete") {
                "delete_file"
            } else {
                "update_permissions"
            };
            let after_text = if action_norm == "delete_file" {
                format!("DELETE Drive file '{}'", payload.file_id)
            } else {
                format!(
                    "SHARE Drive file '{}' -> role={}, type={}, email={:?}",
                    payload.file_id,
                    payload.role.as_deref().unwrap_or("reader"),
                    payload.perm_type.as_deref().unwrap_or("user"),
                    payload.email_address
                )
            };

            let input = NewProposalInput {
                account_id: acct_id,
                service: "drive".to_string(),
                action: action_norm.to_string(),
                title: format!("Drive {}: {}", action_norm, payload.file_id),
                summary: after_text.clone(),
                tier: BlastRadiusTier::Tier2ExternalDestructive,
                tainted_origin: is_tainted,
                target_resource: payload.file_id.clone(),
                before_snapshot: Some(format!("file_id: {}", payload.file_id)),
                after_snapshot: Some(after_text),
                myers_diff: None,
                payload_json: json!({
                    "file_id": payload.file_id,
                    "role": payload.role,
                    "perm_type": payload.perm_type,
                    "email_address": payload.email_address,
                    "api_base_override": payload.api_base_override
                }),
                ttl_secs: 600,
            };
            queue_proposal_response(&state, sess, &format!("drive.{action_norm}"), input)
        }
        "download_sandbox" => {
            let Some(token) = token_opt else {
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(json!({ "status": "unauthorized", "message": "No active OAuth token for Drive download." })),
                );
            };
            let filename = payload.filename.as_deref().unwrap_or("drive_download.bin");
            let ws = workspace_client_from_override(payload.api_base_override.as_deref());
            let exchange_dir = state.google_hub.data_dir.join("exchange");
            match ws
                .download_to_sandbox(&token, &payload.file_id, filename, &exchange_dir)
                .await
            {
                Ok(path) => {
                    state
                        .google_hub
                        .taint_tracker
                        .mark_tainted(sess, "drive:download");
                    (
                        StatusCode::OK,
                        Json(json!({
                            "status": "success",
                            "sandbox_path": path.display().to_string()
                        })),
                    )
                }
                Err(CoreError::RefsPathLeakViolation(msg)) => (
                    StatusCode::BAD_REQUEST,
                    Json(json!({ "status": "error", "message": msg })),
                ),
                Err(err) => (
                    StatusCode::BAD_GATEWAY,
                    Json(json!({ "status": "error", "message": err.to_string() })),
                ),
            }
        }
        other => (
            StatusCode::BAD_REQUEST,
            Json(json!({ "status": "error", "message": format!("Unsupported drive action '{other}'") })),
        ),
    }
}

#[derive(Debug, Deserialize)]
pub struct DocsScratchpadGetQuery {
    pub doc_id: String,
    pub title: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DocsScratchpadEditRequest {
    pub doc_id: String,
    pub title: Option<String>,
    pub op: Option<String>,
    pub at_line: Option<usize>,
    pub start_line: Option<usize>,
    pub end_line: Option<usize>,
    #[serde(default)]
    pub lines: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct DocsScratchpadPushRequest {
    pub doc_id: String,
    pub account_id: Option<String>,
    pub remote_baseline: Option<String>,
    pub session_id: Option<String>,
    pub api_base_override: Option<String>,
}

/// `GET /api/v1/google/docs/scratchpad`
pub async fn docs_scratchpad_get_handler(
    State(state): State<AppState>,
    Query(query): Query<DocsScratchpadGetQuery>,
) -> impl IntoResponse {
    let title = query.title.as_deref().unwrap_or(&query.doc_id);
    match state.google_hub.scratchpads.load_or_create(&query.doc_id, title) {
        Ok(doc) => {
            let rendered = doc.lines.join("\n");
            (
                StatusCode::OK,
                Json(json!({
                    "status": "success",
                    "scratchpad": doc,
                    "rendered_text": rendered
                })),
            )
        }
        Err(err) => (
            StatusCode::BAD_REQUEST,
            Json(json!({ "status": "error", "message": err.to_string() })),
        ),
    }
}

/// `POST /api/v1/google/docs/scratchpad`
pub async fn docs_scratchpad_edit_handler(
    State(state): State<AppState>,
    Json(payload): Json<DocsScratchpadEditRequest>,
) -> impl IntoResponse {
    let op = payload
        .op
        .as_deref()
        .unwrap_or("insert")
        .to_ascii_lowercase();
    let title = payload.title.as_deref().unwrap_or(&payload.doc_id);

    // Ensure scratchpad exists prior to line operations
    if let Err(err) = state.google_hub.scratchpads.load_or_create(&payload.doc_id, title) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "status": "error", "message": err.to_string() })),
        );
    }

    let res = match op.as_str() {
        "load_or_create" | "load" => state.google_hub.scratchpads.load_or_create(&payload.doc_id, title),
        "insert" => {
            let at = payload.at_line.unwrap_or(1);
            state
                .google_hub
                .scratchpads
                .insert_lines(&payload.doc_id, at, payload.lines)
        }
        "replace" => {
            let start = payload.start_line.unwrap_or(1);
            let end = payload.end_line.unwrap_or(start);
            state
                .google_hub
                .scratchpads
                .replace_lines(&payload.doc_id, start, end, payload.lines)
        }
        "delete" => {
            let start = payload.start_line.unwrap_or(1);
            let end = payload.end_line.unwrap_or(start);
            state
                .google_hub
                .scratchpads
                .delete_lines(&payload.doc_id, start, end)
        }
        other => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "status": "error", "message": format!("Unsupported scratchpad op '{other}'") })),
            );
        }
    };

    match res {
        Ok(doc) => {
            let rendered = doc.lines.join("\n");
            (
                StatusCode::OK,
                Json(json!({
                    "status": "success",
                    "scratchpad": doc,
                    "rendered_text": rendered
                })),
            )
        }
        Err(err) => (
            StatusCode::BAD_REQUEST,
            Json(json!({ "status": "error", "message": err.to_string() })),
        ),
    }
}

/// `POST /api/v1/google/docs/scratchpad/push`
pub async fn docs_scratchpad_push_handler(
    State(state): State<AppState>,
    Json(payload): Json<DocsScratchpadPushRequest>,
) -> impl IntoResponse {
    let rendered = match state.google_hub.scratchpads.render_text(&payload.doc_id) {
        Ok(text) => text,
        Err(err) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "status": "error", "message": err.to_string() })),
            );
        }
    };

    let baseline = payload.remote_baseline.unwrap_or_default();
    let sess = payload.session_id.as_deref().unwrap_or("web_dashboard");
    let acct_opt = state
        .google_hub
        .resolve_account(payload.account_id.as_deref())
        .await;
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
        .is_some_and(|a| a.write_whitelist_ids.iter().any(|w| w == &payload.doc_id));
    let token_opt = acct_opt
        .as_ref()
        .and_then(|a| a.oauth_access_token.as_deref())
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .map(ToString::to_string);
    let is_tainted = state.google_hub.taint_tracker.is_tainted(sess);

    match evaluate_action_policy(
        BlastRadiusTier::Tier2ExternalDestructive,
        policy,
        is_tainted,
        is_whitelisted,
    ) {
        PolicyVerdict::Denied { reason } => (
            StatusCode::FORBIDDEN,
            Json(json!({ "status": "forbidden", "message": reason })),
        ),
        PolicyVerdict::AllowDirect if token_opt.is_some() => {
            let token = token_opt.unwrap_or_default();
            let diff_preview = compute_safe_myers_diff(&baseline, &rendered)
                .map(|p| p.diff_text)
                .unwrap_or_default();
            let ws = workspace_client_from_override(payload.api_base_override.as_deref());
            match ws.push_docs_content(&token, &payload.doc_id, &rendered).await {
                Ok(resp) => {
                    state.google_hub.record_audit_event(
                        sess,
                        &acct_id,
                        "docs.push_scratchpad",
                        BlastRadiusTier::Tier1SafeReversible,
                        false,
                        "executed_whitelisted",
                        &format!("Pushed whitelisted scratchpad '{}' directly", payload.doc_id),
                    );
                    (
                        StatusCode::OK,
                        Json(json!({
                            "status": "pushed_direct",
                            "doc_id": payload.doc_id,
                            "myers_diff": diff_preview,
                            "remote_response": resp
                        })),
                    )
                }
                Err(err) => (
                    StatusCode::BAD_GATEWAY,
                    Json(json!({ "status": "error", "message": err.to_string() })),
                ),
            }
        }
        _ => {
            let input = NewProposalInput {
                account_id: acct_id,
                service: "docs".to_string(),
                action: "push_scratchpad".to_string(),
                title: format!("Push Docs Scratchpad: {}", payload.doc_id),
                summary: format!(
                    "Synchronize {} line(s) from local scratchpad to Google Doc '{}'",
                    rendered.lines().count(),
                    payload.doc_id
                ),
                tier: BlastRadiusTier::Tier2ExternalDestructive,
                tainted_origin: is_tainted,
                target_resource: payload.doc_id.clone(),
                before_snapshot: Some(baseline),
                after_snapshot: Some(rendered),
                myers_diff: None,
                payload_json: json!({
                    "doc_id": payload.doc_id,
                    "api_base_override": payload.api_base_override
                }),
                ttl_secs: 600,
            };
            queue_proposal_response(&state, sess, "docs.push_scratchpad", input)
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SheetsAnalyzeRequest {
    pub spreadsheet_id: String,
    pub range: String,
    pub account_id: Option<String>,
    pub rows: Option<Vec<Vec<String>>>,
    pub session_id: Option<String>,
    pub api_base_override: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SheetsWriteRequest {
    pub spreadsheet_id: String,
    pub range: String,
    pub values: Vec<Vec<Value>>,
    pub account_id: Option<String>,
    pub session_id: Option<String>,
    pub api_base_override: Option<String>,
}

/// `POST /api/v1/google/sheets/analyze`
pub async fn sheets_analyze_handler(
    State(state): State<AppState>,
    Json(payload): Json<SheetsAnalyzeRequest>,
) -> impl IntoResponse {
    let rows = if let Some(inline_rows) = payload.rows {
        inline_rows
    } else {
        let (_, _, token_opt) = resolve_policy_context(&state, payload.account_id.as_deref()).await;
        let Some(token) = token_opt else {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "status": "unauthorized",
                    "message": "No OAuth token configured and no inline 'rows' provided for Sheets analysis."
                })),
            );
        };
        let ws = workspace_client_from_override(payload.api_base_override.as_deref());
        match ws
            .read_range(&token, &payload.spreadsheet_id, &payload.range)
            .await
        {
            Ok(fetched) => {
                let sess = payload.session_id.as_deref().unwrap_or("web_dashboard");
                state
                    .google_hub
                    .taint_tracker
                    .mark_tainted(sess, "sheets:read");
                fetched
            }
            Err(err) => {
                return (
                    StatusCode::BAD_GATEWAY,
                    Json(json!({ "status": "error", "message": err.to_string() })),
                );
            }
        }
    };

    let analysis =
        GoogleWorkspaceClient::analyze_financial_rows(&payload.spreadsheet_id, &payload.range, &rows);
    (
        StatusCode::OK,
        Json(serde_json::to_value(analysis).unwrap_or_default()),
    )
}

/// `POST /api/v1/google/sheets/write` (Tier 2 -> Agent Inbox!)
pub async fn sheets_write_handler(
    State(state): State<AppState>,
    Json(payload): Json<SheetsWriteRequest>,
) -> impl IntoResponse {
    let sess = payload.session_id.as_deref().unwrap_or("web_dashboard");
    let (acct_id, policy, _) = resolve_policy_context(&state, payload.account_id.as_deref()).await;
    let is_tainted = state.google_hub.taint_tracker.is_tainted(sess);

    if let PolicyVerdict::Denied { reason } = evaluate_action_policy(
        BlastRadiusTier::Tier2ExternalDestructive,
        policy,
        is_tainted,
        false,
    ) {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({ "status": "forbidden", "message": reason })),
        );
    }

    let rendered_rows = serde_json::to_string_pretty(&payload.values)
        .unwrap_or_default()
        .replace("\r\n", "\n");
    let input = NewProposalInput {
        account_id: acct_id,
        service: "sheets".to_string(),
        action: "write_range".to_string(),
        title: format!("Sheets Write: {}!{}", payload.spreadsheet_id, payload.range),
        summary: format!("Write {} row(s) into range '{}'", payload.values.len(), payload.range),
        tier: BlastRadiusTier::Tier2ExternalDestructive,
        tainted_origin: is_tainted,
        target_resource: format!("{}:{}", payload.spreadsheet_id, payload.range),
        before_snapshot: Some(format!("Range {}", payload.range)),
        after_snapshot: Some(rendered_rows),
        myers_diff: None,
        payload_json: json!({
            "spreadsheet_id": payload.spreadsheet_id,
            "range": payload.range,
            "values": payload.values,
            "api_base_override": payload.api_base_override
        }),
        ttl_secs: 600,
    };
    queue_proposal_response(&state, sess, "sheets.write", input)
}

#[derive(Debug, Deserialize)]
pub struct SlidesPlanRequest {
    pub topic: String,
    #[serde(default)]
    pub briefing: String,
    pub target_slides: Option<usize>,
    #[serde(default)]
    pub create_remote: bool,
    pub account_id: Option<String>,
    pub session_id: Option<String>,
    pub api_base_override: Option<String>,
}

/// `POST /api/v1/google/slides/plan`
pub async fn slides_plan_handler(
    State(state): State<AppState>,
    Json(payload): Json<SlidesPlanRequest>,
) -> impl IntoResponse {
    let plan = GoogleWorkspaceClient::build_slide_deck_plan(
        &payload.topic,
        &payload.briefing,
        payload.target_slides.unwrap_or(8),
    );

    if !payload.create_remote {
        return (
            StatusCode::OK,
            Json(json!({ "status": "success", "plan": plan })),
        );
    }

    let sess = payload.session_id.as_deref().unwrap_or("web_dashboard");
    let (acct_id, policy, token_opt) = resolve_policy_context(&state, payload.account_id.as_deref()).await;
    let is_tainted = state.google_hub.taint_tracker.is_tainted(sess);

    match evaluate_action_policy(BlastRadiusTier::Tier1SafeReversible, policy, is_tainted, false) {
        PolicyVerdict::Denied { reason } => (
            StatusCode::FORBIDDEN,
            Json(json!({ "status": "forbidden", "message": reason })),
        ),
        PolicyVerdict::RequireBmadInbox { .. } => {
            let plan_text = serde_json::to_string_pretty(&plan)
                .unwrap_or_default()
                .replace("\r\n", "\n");
            let input = NewProposalInput {
                account_id: acct_id,
                service: "slides".to_string(),
                action: "create_presentation".to_string(),
                title: format!("Create Slide Deck: {}", payload.topic),
                summary: format!("{} slide(s) planned", plan.slides.len()),
                tier: BlastRadiusTier::Tier1SafeReversible,
                tainted_origin: is_tainted,
                target_resource: payload.topic.clone(),
                before_snapshot: Some(String::new()),
                after_snapshot: Some(plan_text),
                myers_diff: None,
                payload_json: json!({
                    "plan": plan,
                    "api_base_override": payload.api_base_override
                }),
                ttl_secs: 600,
            };
            queue_proposal_response(&state, sess, "slides.create", input)
        }
        PolicyVerdict::AllowDirect => {
            let Some(token) = token_opt else {
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(json!({ "status": "unauthorized", "message": "No active OAuth token for Slides creation.", "plan": plan })),
                );
            };
            let ws = workspace_client_from_override(payload.api_base_override.as_deref());
            match ws.create_presentation(&token, &plan).await {
                Ok(created) => (
                    StatusCode::OK,
                    Json(json!({ "status": "success", "plan": created })),
                ),
                Err(err) => (
                    StatusCode::BAD_GATEWAY,
                    Json(json!({ "status": "error", "message": err.to_string(), "plan": plan })),
                ),
            }
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct TasksListQuery {
    pub account_id: Option<String>,
    pub tasklist_id: Option<String>,
    pub show_completed: Option<bool>,
    pub api_base_override: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TasksMutateRequest {
    pub action: Option<String>,
    pub account_id: Option<String>,
    pub tasklist_id: Option<String>,
    pub title: Option<String>,
    pub notes: Option<String>,
    pub due_rfc3339: Option<String>,
    pub task_id: Option<String>,
    pub session_id: Option<String>,
    pub api_base_override: Option<String>,
}

/// `GET /api/v1/google/tasks`
pub async fn tasks_list_handler(
    State(state): State<AppState>,
    Query(query): Query<TasksListQuery>,
) -> impl IntoResponse {
    let (_, _, token_opt) = resolve_policy_context(&state, query.account_id.as_deref()).await;
    let Some(token) = token_opt else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "status": "unauthorized", "message": "No active OAuth token configured for Google Tasks." })),
        );
    };

    let client = match query.api_base_override.as_deref() {
        Some(base) => GoogleTasksClient::with_base_url(base),
        None => GoogleTasksClient::new(),
    };
    let list_id = query.tasklist_id.as_deref().unwrap_or("@default");
    let show_completed = query.show_completed.unwrap_or(false);

    match client.list_tasks(&token, list_id, show_completed).await {
        Ok(tasks) => (
            StatusCode::OK,
            Json(json!({ "status": "success", "count": tasks.len(), "tasks": tasks })),
        ),
        Err(err) => (
            StatusCode::BAD_GATEWAY,
            Json(json!({ "status": "error", "message": err.to_string() })),
        ),
    }
}

/// `POST /api/v1/google/tasks`
pub async fn tasks_mutate_handler(
    State(state): State<AppState>,
    Json(payload): Json<TasksMutateRequest>,
) -> impl IntoResponse {
    let action = payload
        .action
        .as_deref()
        .unwrap_or("create")
        .to_ascii_lowercase();
    let sess = payload.session_id.as_deref().unwrap_or("web_dashboard");
    let (acct_id, policy, token_opt) = resolve_policy_context(&state, payload.account_id.as_deref()).await;
    let is_tainted = state.google_hub.taint_tracker.is_tainted(sess);

    match evaluate_action_policy(BlastRadiusTier::Tier1SafeReversible, policy, is_tainted, false) {
        PolicyVerdict::Denied { reason } => (
            StatusCode::FORBIDDEN,
            Json(json!({ "status": "forbidden", "message": reason })),
        ),
        PolicyVerdict::RequireBmadInbox { .. } => {
            let tasklist_id = payload.tasklist_id.as_deref().unwrap_or("@default");
            let act_norm = if action == "complete" {
                "complete_task"
            } else {
                "create_task"
            };
            let title_str = payload.title.clone().unwrap_or_else(|| "Task Update".to_string());
            let input = NewProposalInput {
                account_id: acct_id,
                service: "tasks".to_string(),
                action: act_norm.to_string(),
                title: format!("Google Task ({act_norm}): {title_str}"),
                summary: format!("Tasklist: {tasklist_id}"),
                tier: BlastRadiusTier::Tier1SafeReversible,
                tainted_origin: is_tainted,
                target_resource: payload
                    .task_id
                    .clone()
                    .unwrap_or_else(|| tasklist_id.to_string()),
                before_snapshot: Some(String::new()),
                after_snapshot: Some(format!(
                    "action={act_norm}\ntitle={title_str}\nnotes={:?}",
                    payload.notes
                )),
                myers_diff: None,
                payload_json: json!({
                    "tasklist_id": tasklist_id,
                    "title": title_str,
                    "notes": payload.notes,
                    "due_rfc3339": payload.due_rfc3339,
                    "task_id": payload.task_id,
                    "api_base_override": payload.api_base_override
                }),
                ttl_secs: 600,
            };
            queue_proposal_response(&state, sess, &format!("tasks.{act_norm}"), input)
        }
        PolicyVerdict::AllowDirect => {
            let Some(token) = token_opt else {
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(json!({ "status": "unauthorized", "message": "No active OAuth token configured for Google Tasks." })),
                );
            };
            let client = match payload.api_base_override.as_deref() {
                Some(base) => GoogleTasksClient::with_base_url(base),
                None => GoogleTasksClient::new(),
            };
            let list_id = payload.tasklist_id.as_deref().unwrap_or("@default");
            if action == "complete" {
                let Some(ref tid) = payload.task_id else {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(json!({ "status": "error", "message": "task_id is required to complete a task" })),
                    );
                };
                match client.complete_task(&token, list_id, tid).await {
                    Ok(t) => (StatusCode::OK, Json(json!({ "status": "success", "task": t }))),
                    Err(err) => (
                        StatusCode::BAD_GATEWAY,
                        Json(json!({ "status": "error", "message": err.to_string() })),
                    ),
                }
            } else {
                let Some(ref title) = payload.title else {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(json!({ "status": "error", "message": "title is required to create a task" })),
                    );
                };
                match client
                    .create_task(
                        &token,
                        list_id,
                        title,
                        payload.notes.as_deref(),
                        payload.due_rfc3339.as_deref(),
                    )
                    .await
                {
                    Ok(t) => (StatusCode::OK, Json(json!({ "status": "success", "task": t }))),
                    Err(err) => (
                        StatusCode::BAD_GATEWAY,
                        Json(json!({ "status": "error", "message": err.to_string() })),
                    ),
                }
            }
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct QuickCaptureRequest {
    pub title: String,
    pub body: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// `POST /api/v1/google/quick-capture`
pub async fn quick_capture_handler(
    State(state): State<AppState>,
    Json(payload): Json<QuickCaptureRequest>,
) -> impl IntoResponse {
    let target_dir = std::env::var("OBSIDIAN_VAULT_PATH")
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .map(|v| PathBuf::from(v).join("raw").join("inbox"))
        .unwrap_or_else(|| state.google_hub.data_dir.join("quick_capture"));

    match quick_capture_to_obsidian(&target_dir, &payload.title, &payload.body, &payload.tags) {
        Ok(path) => (
            StatusCode::OK,
            Json(json!({
                "status": "success",
                "note_path": path.display().to_string(),
                "keep_fallback": "obsidian_quick_capture"
            })),
        ),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "status": "error", "message": err.to_string() })),
        ),
    }
}

#[derive(Debug, Deserialize)]
pub struct ContactsQuery {
    pub account_id: Option<String>,
    pub query: Option<String>,
    pub page_size: Option<usize>,
    pub birthdays_within_days: Option<u32>,
    pub session_id: Option<String>,
    pub api_base_override: Option<String>,
}

/// `GET /api/v1/google/contacts`
pub async fn contacts_handler(
    State(state): State<AppState>,
    Query(query): Query<ContactsQuery>,
) -> impl IntoResponse {
    let (_, _, token_opt) = resolve_policy_context(&state, query.account_id.as_deref()).await;
    let Some(token) = token_opt else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "status": "unauthorized", "message": "No active OAuth token configured for Google Contacts." })),
        );
    };

    let client = match query.api_base_override.as_deref() {
        Some(base) => PeopleClient::with_base_url(base),
        None => PeopleClient::new(),
    };
    let q = query.query.as_deref().unwrap_or("");
    let size = query.page_size.unwrap_or(20);

    match client.search_contacts(&token, q, size).await {
        Ok(contacts) => {
            let sess = query.session_id.as_deref().unwrap_or("web_dashboard");
            state
                .google_hub
                .taint_tracker
                .mark_tainted(sess, "contacts:read");
            let filtered = if let Some(days) = query.birthdays_within_days {
                let now = Utc::now();
                let month = now.format("%m").to_string().parse::<u32>().unwrap_or(1);
                let day = now.format("%d").to_string().parse::<u32>().unwrap_or(1);
                upcoming_birthdays(&contacts, month, day, days)
            } else {
                contacts
            };
            (
                StatusCode::OK,
                Json(json!({ "status": "success", "count": filtered.len(), "contacts": filtered })),
            )
        }
        Err(err) => (
            StatusCode::BAD_GATEWAY,
            Json(json!({ "status": "error", "message": err.to_string() })),
        ),
    }
}

#[derive(Debug, Deserialize)]
pub struct MapsCommuteRequest {
    pub origin: String,
    pub destination: String,
    pub event_id: Option<String>,
    pub event_title: Option<String>,
    pub event_start_rfc3339: Option<String>,
    pub buffer_mins: Option<u32>,
    pub api_key: Option<String>,
    pub api_base_override: Option<String>,
}

/// `POST /api/v1/google/maps/commute`
pub async fn maps_commute_handler(Json(payload): Json<MapsCommuteRequest>) -> impl IntoResponse {
    let api_key = payload
        .api_key
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToString::to_string)
        .or_else(|| env_opt("ROUTES_API_KEY"))
        .or_else(|| env_opt("GOOGLE_MAPS_API_KEY"));

    let Some(key) = api_key else {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "status": "error",
                "message": "Routes API v2 key missing. Set ROUTES_API_KEY, GOOGLE_MAPS_API_KEY, or pass 'api_key'."
            })),
        );
    };

    let client = match payload.api_base_override.as_deref() {
        Some(base) => RoutesV2Client::with_base_url(base),
        None => RoutesV2Client::new(),
    };

    let comp = match client
        .compute_route(&key, &payload.origin, &payload.destination)
        .await
    {
        Ok(c) => c,
        Err(err) => {
            return (
                StatusCode::BAD_GATEWAY,
                Json(json!({ "status": "error", "message": err.to_string() })),
            );
        }
    };

    let event_start = payload
        .event_start_rfc3339
        .unwrap_or_else(|| (Utc::now() + ChronoDuration::hours(2)).to_rfc3339());
    match compute_commute_alert(
        payload.event_id.as_deref().unwrap_or("commute_check"),
        payload.event_title.as_deref().unwrap_or("Commute"),
        &event_start,
        &payload.destination,
        comp.duration_seconds,
        comp.static_duration_seconds,
        comp.distance_meters,
        payload.buffer_mins.unwrap_or(15),
    ) {
        Ok(alert) => (
            StatusCode::OK,
            Json(serde_json::to_value(alert).unwrap_or_default()),
        ),
        Err(err) => (
            StatusCode::BAD_REQUEST,
            Json(json!({ "status": "error", "message": err.to_string() })),
        ),
    }
}

#[derive(Debug, Deserialize)]
pub struct YouTubeTranscriptRequest {
    pub url_or_id: String,
    pub preferred_lang: Option<String>,
    pub session_id: Option<String>,
    pub api_base_override: Option<String>,
}

/// `POST /api/v1/google/youtube/transcript`
pub async fn youtube_transcript_handler(
    State(state): State<AppState>,
    Json(payload): Json<YouTubeTranscriptRequest>,
) -> impl IntoResponse {
    let client = match payload.api_base_override.as_deref() {
        Some(base) => YouTubeClient::with_base_url(base),
        None => YouTubeClient::new(),
    };
    let lang = payload.preferred_lang.as_deref().unwrap_or("en");

    match client
        .fetch_transcript_with_cache(&state.google_hub.data_dir, &payload.url_or_id, lang)
        .await
    {
        Ok(mut summary) => {
            let sess = payload.session_id.as_deref().unwrap_or("web_dashboard");
            state
                .google_hub
                .taint_tracker
                .mark_tainted(sess, "youtube:transcript");
            summary.transcript_text = wrap_untrusted_content(
                &format!("youtube:{}", summary.video_id),
                &summary.transcript_text,
            );
            (
                StatusCode::OK,
                Json(serde_json::to_value(summary).unwrap_or_default()),
            )
        }
        Err(err) => (
            StatusCode::BAD_GATEWAY,
            Json(json!({ "status": "error", "message": err.to_string() })),
        ),
    }
}

#[derive(Debug, Deserialize)]
pub struct GemsMutateRequest {
    pub action: Option<String>,
    pub id: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub system_instruction: Option<String>,
    pub model: Option<String>,
    pub backend: Option<String>,
    pub remote_gem_id: Option<String>,
    pub prompt: Option<String>,
    pub account_id: Option<String>,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub model_hash: Option<String>,
}

/// `GET /api/v1/google/gems`
pub async fn gems_list_handler(State(state): State<AppState>) -> impl IntoResponse {
    let gems = state.google_hub.gems.list();
    (
        StatusCode::OK,
        Json(json!({
            "status": "success",
            "count": gems.len(),
            "gems": gems
        })),
    )
}

/// `POST /api/v1/google/gems` (bloqueando `StreamGenerate` na conta primaria!)
pub async fn gems_mutate_handler(
    State(state): State<AppState>,
    Json(payload): Json<GemsMutateRequest>,
) -> impl IntoResponse {
    let action = payload
        .action
        .as_deref()
        .unwrap_or("upsert")
        .to_ascii_lowercase();

    match action.as_str() {
        "upsert" => {
            let Some(id) = payload.id.filter(|s| !s.trim().is_empty()) else {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({ "status": "error", "message": "Gem id is required" })),
                );
            };
            let gem = GemProfileDto {
                id,
                name: payload.name.unwrap_or_else(|| "Custom Gem".to_string()),
                description: payload.description.unwrap_or_default(),
                system_instruction: payload.system_instruction.unwrap_or_default(),
                model: payload.model.unwrap_or_else(|| "gemini-2.5-flash".to_string()),
                backend: payload.backend.unwrap_or_else(|| "gemini_api".to_string()),
                remote_gem_id: payload.remote_gem_id,
            };
            match state.google_hub.gems.upsert(gem) {
                Ok(saved) => (
                    StatusCode::OK,
                    Json(json!({ "status": "success", "gem": saved })),
                ),
                Err(err) => (
                    StatusCode::BAD_REQUEST,
                    Json(json!({ "status": "error", "message": err.to_string() })),
                ),
            }
        }
        "delete" => {
            let Some(ref id) = payload.id else {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({ "status": "error", "message": "Gem id is required for delete" })),
                );
            };
            match state.google_hub.gems.remove(id) {
                Ok(removed) => (
                    StatusCode::OK,
                    Json(json!({ "status": "success", "removed": removed })),
                ),
                Err(err) => (
                    StatusCode::BAD_REQUEST,
                    Json(json!({ "status": "error", "message": err.to_string() })),
                ),
            }
        }
        "dispatch" | "secondary_stream_generate" => {
            let gem = if let Some(ref id) = payload.id {
                state.google_hub.gems.get(id).unwrap_or(GemProfileDto {
                    id: id.clone(),
                    name: payload.name.unwrap_or_else(|| id.clone()),
                    description: payload.description.unwrap_or_default(),
                    system_instruction: payload.system_instruction.unwrap_or_default(),
                    model: payload.model.unwrap_or_else(|| "gemini-2.5-flash".to_string()),
                    backend: payload.backend.unwrap_or_else(|| "gemini_api".to_string()),
                    remote_gem_id: payload.remote_gem_id,
                })
            } else {
                GemProfileDto {
                    id: "adhoc_gem".to_string(),
                    name: payload.name.unwrap_or_else(|| "Adhoc Gem".to_string()),
                    description: payload.description.unwrap_or_default(),
                    system_instruction: payload.system_instruction.unwrap_or_default(),
                    model: payload.model.unwrap_or_else(|| "gemini-2.5-flash".to_string()),
                    backend: payload.backend.unwrap_or_else(|| "gemini_api".to_string()),
                    remote_gem_id: payload.remote_gem_id,
                }
            };

            let prompt = payload.prompt.as_deref().unwrap_or("");
            if prompt.trim().is_empty() {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({ "status": "error", "message": "prompt is required" })),
                );
            }

            if action == "secondary_stream_generate" || gem.backend == "secondary_web" {
                let Some(acct) = state
                    .google_hub
                    .resolve_account_strict(payload.account_id.as_deref())
                    .await
                else {
                    return (
                        StatusCode::NOT_FOUND,
                        Json(json!({
                            "status": "error",
                            "message": "Explicit secondary sandbox account required for StreamGenerate."
                        })),
                    );
                };

                let remote_id = gem.remote_gem_id.as_deref().unwrap_or(&gem.id);
                let model_hash = payload.model_hash.as_deref().unwrap_or("9d8ca3786ebdfbea");
                match build_secondary_stream_generate_payload(&acct, remote_id, prompt, model_hash) {
                    Ok(stream_payload) => (
                        StatusCode::OK,
                        Json(json!({
                            "status": "success",
                            "backend": "secondary_web",
                            "account_id": acct.account_id,
                            "payload": stream_payload
                        })),
                    ),
                    Err(CoreError::DangerousCommandBlocked(msg)) => (
                        StatusCode::FORBIDDEN,
                        Json(json!({ "status": "forbidden", "message": msg })),
                    ),
                    Err(err) => (
                        StatusCode::BAD_REQUEST,
                        Json(json!({ "status": "error", "message": err.to_string() })),
                    ),
                }
            } else {
                let key = payload
                    .api_key
                    .as_deref()
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(ToString::to_string)
                    .or_else(|| env_opt("GEMINI_API_KEY"));
                let Some(api_key) = key else {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(json!({
                            "status": "error",
                            "message": "GEMINI_API_KEY not configured for official Gemini API dispatch."
                        })),
                    );
                };
                let base_url = payload.base_url.as_deref().unwrap_or("");
                let http = reqwest::Client::new();
                match dispatch_gem_official_api(&http, base_url, &api_key, &gem, prompt).await {
                    Ok(reply) => (
                        StatusCode::OK,
                        Json(json!({
                            "status": "success",
                            "backend": "gemini_api",
                            "gem_id": gem.id,
                            "response_text": reply
                        })),
                    ),
                    Err(err) => (
                        StatusCode::BAD_GATEWAY,
                        Json(json!({ "status": "error", "message": err.to_string() })),
                    ),
                }
            }
        }
        other => (
            StatusCode::BAD_REQUEST,
            Json(json!({ "status": "error", "message": format!("Unsupported gems action '{other}'") })),
        ),
    }
}

// ============================================================================
// v0.9.2 GEMINI NOTEBOOKS UI & HTTP RANGE AUDIO STREAMING
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct NotebookAddSourceRequest {
    pub source_type: Option<String>,
    pub url: Option<String>,
    pub title: Option<String>,
    pub content: Option<String>,
    pub endpoint_override: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct NotebookSyncDriveRequest {
    #[serde(default)]
    pub source_ids: Vec<String>,
    pub endpoint_override: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct NotebookQueryRequest {
    pub query: String,
    pub session_id: Option<String>,
    pub endpoint_override: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct NotebookStudioRequest {
    pub kind: Option<String>,
    pub artifact_type: Option<String>,
    pub prompt: Option<String>,
    pub endpoint_override: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AudioStreamQuery {
    pub download: Option<bool>,
}

/// `GET /api/v1/google/notebooks`
pub async fn notebooks_list_handler(State(state): State<AppState>) -> impl IntoResponse {
    let nlm = NlmCliBridge::new();
    if let Ok(notebooks) = nlm.list_notebooks().await {
        return (
            StatusCode::OK,
            Json(json!({
                "status": "success",
                "backend": "nlm_cli",
                "count": notebooks.len(),
                "notebooks": notebooks
            })),
        );
    }

    if let Some(creds) = resolve_nlm_credentials(&state).await {
        let client = WizRpcClient::new(creds);
        let payload = WizRpcPayload::new(WizRpcId::ListNotebooks, json!([null, 1, null, [2]]));
        if let Ok(resp) = client.execute(&payload).await {
            return (
                StatusCode::OK,
                Json(json!({
                    "status": "success",
                    "backend": "wiz_rpc",
                    "rpc_id": resp.rpc_id,
                    "notebooks": resp.payload.unwrap_or_else(|| json!([]))
                })),
            );
        }
    }

    (
        StatusCode::OK,
        Json(json!({
            "status": "offline",
            "backend": "none",
            "count": 0,
            "notebooks": [],
            "message": "Nenhuma sessão NotebookLM (nlm CLI ou cookies __Secure-1PSID) configurada no momento."
        })),
    )
}

/// `GET /api/v1/google/notebooks/:id`
pub async fn notebook_get_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    if !is_safe_path_segment(&id) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "status": "error", "message": "Invalid notebook identifier" })),
        );
    }

    let cached_audio = list_cached_audio_files(&state.google_hub.audio_cache_dir, &id);
    let nlm = NlmCliBridge::new();
    if let Ok(nb) = nlm.get_notebook(&id).await {
        return (
            StatusCode::OK,
            Json(json!({
                "status": "success",
                "backend": "nlm_cli",
                "notebook_id": id,
                "notebook": nb,
                "cached_audio": cached_audio
            })),
        );
    }

    if let Some(creds) = resolve_nlm_credentials(&state).await {
        let client = WizRpcClient::new(creds);
        let payload = WizRpcPayload::new(WizRpcId::GetNotebook, json!([id, null, [2], null, 0]));
        if let Ok(resp) = client.execute(&payload).await {
            return (
                StatusCode::OK,
                Json(json!({
                    "status": "success",
                    "backend": "wiz_rpc",
                    "notebook_id": id,
                    "notebook": resp.payload,
                    "cached_audio": cached_audio
                })),
            );
        }
    }

    (
        StatusCode::OK,
        Json(json!({
            "status": "offline",
            "backend": "cache_only",
            "notebook_id": id,
            "notebook": null,
            "cached_audio": cached_audio
        })),
    )
}

/// `POST /api/v1/google/notebooks/:id/sources`
pub async fn notebook_add_source_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<NotebookAddSourceRequest>,
) -> impl IntoResponse {
    let Some(creds) = resolve_nlm_credentials(&state).await else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "status": "unauthorized", "message": "NotebookLM cookies not configured in vault." })),
        );
    };

    let mut client = WizRpcClient::new(creds);
    if let Some(ep) = payload.endpoint_override {
        client = client.with_endpoint(ep);
    }
    let source_args = json!([
        [
            [
                null,
                [payload.title.as_deref().unwrap_or("Source"), payload.content.as_deref().unwrap_or("")],
                payload.url.as_deref(),
                payload.source_type.as_deref()
            ]
        ],
        id
    ]);
    let rpc = WizRpcPayload::new(WizRpcId::AddSource, source_args);
    match client.execute(&rpc).await {
        Ok(resp) => (
            StatusCode::OK,
            Json(json!({ "status": "success", "rpc_id": resp.rpc_id, "data": resp.payload })),
        ),
        Err(err) => (
            StatusCode::BAD_GATEWAY,
            Json(json!({ "status": "error", "message": err.to_string() })),
        ),
    }
}

/// `POST /api/v1/google/notebooks/:id/sync-drive`
pub async fn notebook_sync_drive_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
    body: Option<Json<NotebookSyncDriveRequest>>,
) -> impl IntoResponse {
    let req = body.map(|Json(b)| b).unwrap_or(NotebookSyncDriveRequest {
        source_ids: Vec::new(),
        endpoint_override: None,
    });

    let nlm = NlmCliBridge::new();
    if let Ok(sync_out) = nlm.sync_sources(&id).await {
        return (
            StatusCode::OK,
            Json(json!({ "status": "success", "backend": "nlm_cli", "result": sync_out })),
        );
    }

    let Some(creds) = resolve_nlm_credentials(&state).await else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "status": "unauthorized", "message": "Neither nlm CLI nor NotebookLM session cookies are configured." })),
        );
    };

    let mut client = WizRpcClient::new(creds);
    if let Some(ep) = req.endpoint_override {
        client = client.with_endpoint(ep);
    }

    let mut synced = Vec::new();
    for src_id in req.source_ids {
        let check_rpc = WizRpcPayload::new(WizRpcId::CheckFreshness, json!([src_id]));
        let _ = client.execute(&check_rpc).await;
        let sync_rpc = WizRpcPayload::new(WizRpcId::SyncDriveSource, json!([src_id]));
        if let Ok(resp) = client.execute(&sync_rpc).await {
            synced.push(json!({ "source_id": src_id, "data": resp.payload }));
        }
    }

    (
        StatusCode::OK,
        Json(json!({ "status": "success", "backend": "wiz_rpc", "notebook_id": id, "synced": synced })),
    )
}

/// `POST /api/v1/google/notebooks/:id/query`
pub async fn notebook_query_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<NotebookQueryRequest>,
) -> impl IntoResponse {
    let sess = payload.session_id.as_deref().unwrap_or("web_dashboard");

    let nlm = NlmCliBridge::new();
    if let Ok(ans) = nlm.query_notebook(&id, &payload.query).await {
        state
            .google_hub
            .taint_tracker
            .mark_tainted(sess, &format!("notebooklm:{id}"));
        return (
            StatusCode::OK,
            Json(json!({ "status": "success", "backend": "nlm_cli", "notebook_id": id, "answer": ans })),
        );
    }

    let Some(creds) = resolve_nlm_credentials(&state).await else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "status": "unauthorized", "message": "NotebookLM query requires nlm CLI or session cookies." })),
        );
    };
    let mut client = WizRpcClient::new(creds);
    if let Some(ep) = payload.endpoint_override {
        client = client.with_endpoint(ep);
    }
    let rpc = WizRpcPayload::new(WizRpcId::QueryRAG, json!([id, payload.query]));
    match client.execute(&rpc).await {
        Ok(resp) => {
            state
                .google_hub
                .taint_tracker
                .mark_tainted(sess, &format!("notebooklm:{id}"));
            (
                StatusCode::OK,
                Json(json!({ "status": "success", "backend": "wiz_rpc", "notebook_id": id, "data": resp.payload })),
            )
        }
        Err(err) => (
            StatusCode::BAD_GATEWAY,
            Json(json!({ "status": "error", "message": err.to_string() })),
        ),
    }
}

/// `POST /api/v1/google/notebooks/:id/studio`
pub async fn notebook_studio_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<NotebookStudioRequest>,
) -> impl IntoResponse {
    let kind = payload
        .kind
        .or(payload.artifact_type)
        .unwrap_or_else(|| "audio_overview".to_string())
        .to_ascii_lowercase();

    if kind == "audio_overview" || kind == "audio" {
        let nlm = NlmCliBridge::new();
        if let Ok(res) = nlm.create_audio(&id).await {
            return (
                StatusCode::OK,
                Json(json!({ "status": "success", "backend": "nlm_cli", "kind": kind, "result": res })),
            );
        }
    }

    let Some(creds) = resolve_nlm_credentials(&state).await else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "status": "unauthorized", "message": "NotebookLM Studio requires nlm CLI or session cookies." })),
        );
    };

    let mut client = WizRpcClient::new(creds);
    if let Some(ep) = payload.endpoint_override {
        client = client.with_endpoint(ep);
    }

    let (rpc_id, args) = if kind == "mindmap" {
        (WizRpcId::GenerateMindmap, json!([id]))
    } else {
        (
            WizRpcId::StudioCreate,
            json!([[2], id, [null, null, payload.prompt]]),
        )
    };

    let rpc = WizRpcPayload::new(rpc_id, args);
    match client.execute(&rpc).await {
        Ok(resp) => (
            StatusCode::OK,
            Json(json!({ "status": "success", "backend": "wiz_rpc", "rpc_id": resp.rpc_id, "kind": kind, "data": resp.payload })),
        ),
        Err(err) => (
            StatusCode::BAD_GATEWAY,
            Json(json!({ "status": "error", "message": err.to_string() })),
        ),
    }
}

/// `GET /api/v1/google/notebooks/:id/audio/:artifact_id`
///
/// Serves cached NotebookLM audio files (`audio/mpeg`, `audio/mp4`, `audio/wav`, `audio/ogg`)
/// with real HTTP `Range: bytes=start-end` support returning `206 Partial Content` + `Content-Range`
/// for instant HTML5 `<audio>` player seeking.
pub async fn notebook_audio_stream_handler(
    State(state): State<AppState>,
    Path((id, artifact_id)): Path<(String, String)>,
    Query(query): Query<AudioStreamQuery>,
    headers: HeaderMap,
) -> Response {
    if !is_safe_path_segment(&id) || !is_safe_path_segment(&artifact_id) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "status": "error", "message": "Invalid notebook or artifact identifier" })),
        )
            .into_response();
    }

    let mut resolved = state.google_hub.resolve_audio_path(&id, &artifact_id);
    if resolved.is_none() && query.download.unwrap_or(false) {
        let target = state
            .google_hub
            .audio_cache_dir
            .join(&id)
            .join(format!("{artifact_id}.mp3"));
        if let Some(parent) = target.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let nlm = NlmCliBridge::new();
        if nlm.download_audio(&id, &target).await.is_ok() {
            resolved = state.google_hub.resolve_audio_path(&id, &artifact_id);
        }
    }

    let Some(path) = resolved else {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({
                "status": "error",
                "message": format!("Audio artifact '{artifact_id}' not found in cache for notebook '{id}'")
            })),
        )
            .into_response();
    };

    let metadata = match tokio::fs::metadata(&path).await {
        Ok(m) => m,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "status": "error", "message": format!("Failed to stat audio file: {err}") })),
            )
                .into_response();
        }
    };

    let total_len = metadata.len();
    let content_type = audio_mime_for_path(&path);

    if let Some(range_header) = headers.get(header::RANGE).and_then(|v| v.to_str().ok()) {
        let Some((start, end)) = parse_http_byte_range(range_header, total_len) else {
            return Response::builder()
                .status(StatusCode::RANGE_NOT_SATISFIABLE)
                .header(header::ACCEPT_RANGES, "bytes")
                .header(header::CONTENT_RANGE, format!("bytes */{total_len}"))
                .body(Body::empty())
                .unwrap_or_else(|_| StatusCode::RANGE_NOT_SATISFIABLE.into_response());
        };

        let slice_len = end - start + 1;
        let stream = stream_file_slice(path, start, slice_len);
        return Response::builder()
            .status(StatusCode::PARTIAL_CONTENT)
            .header(header::CONTENT_TYPE, content_type)
            .header(header::ACCEPT_RANGES, "bytes")
            .header(
                header::CONTENT_RANGE,
                format!("bytes {start}-{end}/{total_len}"),
            )
            .header(header::CONTENT_LENGTH, slice_len.to_string())
            .body(Body::from_stream(stream))
            .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response());
    }

    let stream = stream_file_slice(path, 0, total_len);
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::ACCEPT_RANGES, "bytes")
        .header(header::CONTENT_LENGTH, total_len.to_string())
        .body(Body::from_stream(stream))
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}

// ============================================================================
// INTERNAL HELPERS
// ============================================================================

async fn resolve_policy_context(
    state: &AppState,
    account_id: Option<&str>,
) -> (String, AccountPermissionPolicy, Option<String>) {
    let acct = state.google_hub.resolve_account(account_id).await;
    match acct {
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

async fn resolve_nlm_credentials(state: &AppState) -> Option<GoogleSessionCredentials> {
    {
        let vault = state.google_hub.vault.read().await;
        if let Some(acct) = vault.accounts().iter().find(|a| {
            a.has_nlm_bridge
                || a.psid.as_deref().is_some_and(|s| !s.is_empty())
                || a.sapisid.as_deref().is_some_and(|s| !s.is_empty())
        }) {
            return Some(acct.to_legacy_credentials());
        }
    }
    let legacy_path = resolve_google_vault_path();
    GoogleSessionVault::load(&legacy_path)
        .ok()
        .flatten()
        .filter(|c| c.is_configured())
}

fn queue_proposal_response(
    state: &AppState,
    session_id: &str,
    tool_name: &str,
    input: NewProposalInput,
) -> (StatusCode, Json<Value>) {
    let account_id = input.account_id.clone();
    let tier = input.tier;
    let tainted = input.tainted_origin;
    match state.google_hub.propose_with_diff(input) {
        Ok(prop) => {
            state.google_hub.record_audit_event(
                session_id,
                &account_id,
                tool_name,
                tier,
                tainted,
                "queued_bmad",
                &format!("Queued proposal '{}' ({})", prop.id, prop.title),
            );
            (
                StatusCode::ACCEPTED,
                Json(json!({
                    "status": "queued_for_approval",
                    "myers_diff": prop.myers_diff,
                    "proposal": prop
                })),
            )
        }
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "status": "error", "message": err })),
        ),
    }
}

fn workspace_client_from_override(base_override: Option<&str>) -> GoogleWorkspaceClient {
    match base_override.map(str::trim).filter(|s| !s.is_empty()) {
        Some(base) => GoogleWorkspaceClient::with_base_urls(base, base, base, base, base, base),
        None => GoogleWorkspaceClient::new(),
    }
}

fn env_opt(key: &str) -> Option<String> {
    std::env::var(key)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn sanitize_account_id(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
        } else if !out.ends_with('_') && !out.is_empty() {
            out.push('_');
        }
        if out.len() >= 48 {
            break;
        }
    }
    let trimmed = out.trim_matches('_').to_string();
    if trimmed.is_empty() {
        "google_account".to_string()
    } else {
        trimmed
    }
}

fn infer_kind_from_email(email: &str) -> GoogleAccountKind {
    let lower = email.trim().to_ascii_lowercase();
    if lower.is_empty() || lower.ends_with("@gmail.com") || lower.ends_with("@googlemail.com") {
        GoogleAccountKind::PersonalGmail
    } else {
        GoogleAccountKind::WorkspaceEnterprise
    }
}

fn audio_mime_for_path(path: &std::path::Path) -> &'static str {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("mp3") => "audio/mpeg",
        Some("m4a") | Some("mp4") => "audio/mp4",
        Some("wav") => "audio/wav",
        Some("ogg") => "audio/ogg",
        _ => "application/octet-stream",
    }
}

fn parse_http_byte_range(header_val: &str, total_len: u64) -> Option<(u64, u64)> {
    if total_len == 0 {
        return None;
    }
    let spec = header_val.trim().strip_prefix("bytes=")?;
    let first_range = spec.split(',').next()?.trim();
    let (start_str, end_str) = first_range.split_once('-')?;

    if start_str.is_empty() {
        let suffix_len = end_str.trim().parse::<u64>().ok()?;
        if suffix_len == 0 {
            return None;
        }
        let take = suffix_len.min(total_len);
        let start = total_len - take;
        return Some((start, total_len - 1));
    }

    let start = start_str.trim().parse::<u64>().ok()?;
    if start >= total_len {
        return None;
    }

    let end = if end_str.trim().is_empty() {
        total_len - 1
    } else {
        let parsed_end = end_str.trim().parse::<u64>().ok()?;
        if parsed_end < start {
            return None;
        }
        parsed_end.min(total_len - 1)
    };

    Some((start, end))
}

fn stream_file_slice(
    path: PathBuf,
    start: u64,
    len: u64,
) -> ReceiverStream<Result<Bytes, std::io::Error>> {
    let (tx, rx) = tokio::sync::mpsc::channel::<Result<Bytes, std::io::Error>>(16);
    tokio::spawn(async move {
        let mut file = match tokio::fs::File::open(&path).await {
            Ok(f) => f,
            Err(err) => {
                let _ = tx.send(Err(err)).await;
                return;
            }
        };
        if start > 0 {
            if let Err(err) = file.seek(SeekFrom::Start(start)).await {
                let _ = tx.send(Err(err)).await;
                return;
            }
        }
        let mut remaining = len;
        let mut buf = vec![0u8; 64 * 1024];
        while remaining > 0 {
            let max_read = (remaining as usize).min(buf.len());
            match file.read(&mut buf[..max_read]).await {
                Ok(0) => break,
                Ok(n) => {
                    remaining -= n as u64;
                    if tx.send(Ok(Bytes::copy_from_slice(&buf[..n]))).await.is_err() {
                        break;
                    }
                }
                Err(err) => {
                    let _ = tx.send(Err(err)).await;
                    break;
                }
            }
        }
    });
    ReceiverStream::new(rx)
}

fn list_cached_audio_files(audio_cache_dir: &std::path::Path, notebook_id: &str) -> Vec<Value> {
    let dir = audio_cache_dir.join(notebook_id);
    if !dir.is_dir() {
        return Vec::new();
    }
    let mut items = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                let size_bytes = entry.metadata().map(|m| m.len()).unwrap_or(0);
                items.push(json!({
                    "artifact_id": stem,
                    "filename": path.file_name().and_then(|n| n.to_str()).unwrap_or_default(),
                    "size_bytes": size_bytes,
                    "stream_url": format!("/api/v1/google/notebooks/{notebook_id}/audio/{stem}")
                }));
            }
        }
    }
    items
}

fn escape_html(raw: &str) -> String {
    raw.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn render_oauth_html(title: &str, message: &str, ok: bool) -> String {
    let accent = if ok { "#34d399" } else { "#f87171" };
    format!(
        "<!DOCTYPE html>\n<html lang=\"pt-BR\">\n<head>\n<meta charset=\"utf-8\">\n<title>{title}</title>\n<style>body{{background:#0d0e12;color:#e5e7eb;font-family:system-ui,-apple-system,sans-serif;display:flex;align-items:center;justify-content:center;min-height:100vh;margin:0}}.card{{max-width:520px;padding:28px;border-radius:14px;background:#161821;border:1px solid {accent}55}}.badge{{color:{accent};font-weight:700;margin-bottom:8px}}</style>\n</head>\n<body>\n<div class=\"card\">\n<div class=\"badge\">{title}</div>\n<p>{message}</p>\n</div>\n</body>\n</html>\n"
    )
}
