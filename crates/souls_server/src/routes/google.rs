//! REST endpoints for Google Universe Bridge & NotebookLM Reverse Engineering.
//!
//! Provides encrypted local session vault inspection, cookie updates,
//! Wiz `batchexecute` RPC dispatching, and Dual-Layer Workspace syncing.

use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::{debug, error, info};

use souls_core::google_bridge::{
    DualLayerWorkspace, GoogleSessionCredentials, GoogleSessionVault, JulesMigrationRouter,
    LegacyJulesCommand, WizRpcClient, WizRpcId, WizRpcPayload,
};
use souls_core::harness::AgenticHarnessDispatcher;

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
    // Fallback if running outside of Z:\ or in unit tests
    PathBuf::from(r".souls_data\google_sessions.vault")
}

/// GET /api/v1/google/session
///
/// Returns current Google session status from the local encrypted vault.
pub async fn get_session_handler() -> impl IntoResponse {
    let vault_path = resolve_google_vault_path();
    match GoogleSessionVault::load(&vault_path) {
        Ok(Some(creds)) => {
            let now = chrono::Utc::now().timestamp();
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
            (StatusCode::OK, Json(serde_json::to_value(status).unwrap_or_default()))
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
            (StatusCode::OK, Json(serde_json::to_value(status).unwrap_or_default()))
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
            Json(serde_json::json!({
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
                Json(serde_json::json!({
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
                Json(serde_json::json!({
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
pub async fn wiz_rpc_handler(
    Json(payload): Json<WizRpcRequestDto>,
) -> impl IntoResponse {
    let vault_path = resolve_google_vault_path();
    let creds = match GoogleSessionVault::load(&vault_path) {
        Ok(Some(c)) if c.is_configured() => c,
        _ => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "status": "unauthorized",
                    "message": "Google session credentials not found or unconfigured in vault. Please authenticate via POST /api/v1/google/session."
                })),
            );
        }
    };

    let rpc_id = match payload.rpc_id.to_ascii_lowercase().as_str() {
        "wxbhsf" | "create_note" => WizRpcId::CreateNote,
        "rlm1ne" | "read_notes" => WizRpcId::ReadNotes,
        "izaodd" | "ingest_source" => WizRpcId::IngestSource,
        "qa9ei" | "deep_research" => WizRpcId::DeepResearch,
        "r7cb6c" | "studio_create" => WizRpcId::StudioCreate,
        "yyryje" | "studio_status" => WizRpcId::StudioStatus,
        "orqrsd" | "query_rag" => WizRpcId::QueryRAG,
        "s07lfd" | "list_notebooks" => WizRpcId::ListNotebooks,
        _ => WizRpcId::Custom(payload.rpc_id.clone()),
    };

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
            Json(serde_json::json!({
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
                Json(serde_json::json!({
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
        Json(serde_json::json!({
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
    let local_path = payload.local_path.unwrap_or_else(|| r"Z:\souls_engine".to_string());
    let mut ws = DualLayerWorkspace::new(PathBuf::from(local_path));

    if let Some(folder) = payload.drive_folder_id {
        ws = ws.with_drive_folder(folder);
    }
    if let Some(doc) = payload.linked_document_id {
        ws = ws.with_linked_document(doc);
    }

    (
        StatusCode::OK,
        Json(serde_json::json!({
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
            Json(serde_json::json!({
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
        Json(serde_json::json!({
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
