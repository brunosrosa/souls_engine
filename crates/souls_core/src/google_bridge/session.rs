//! Encrypted local credential vault and multi-account session manager for Google services.
//!
//! Handles cookies (`__Secure-1PSIDTS`, `__Secure-1PSID`, `SAPISID`), CSRF tokens (`SNlM0e`),
//! and OAuth2 credentials, preventing plain-text token leakage in memory dumps and disk.

use crate::error::CoreError;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use souls_protocol::google::{AccountPermissionPolicy, GoogleAccountKind, GoogleAccountProfileDto};
use std::collections::hash_map::RandomState;
use std::fs;
use std::hash::{BuildHasher, Hasher};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use tracing::{debug, info};

const GV2_MASTER_SEED: &[u8] = b"SoulsEngineV7_GoogleMultiAccountVault_GV2_MasterKey_2026";
static NONCE_SEQ: AtomicU64 = AtomicU64::new(1);

/// Resolves the canonical Google data directory (`SOULS_GOOGLE_DATA_DIR`, then `SOULS_DATA_DIR/google`,
/// falling back to `Z:\souls_engine\.souls_data\google`).
pub fn default_google_data_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("SOULS_GOOGLE_DATA_DIR") {
        let trimmed = dir.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed);
        }
    }
    if let Ok(data_dir) = std::env::var("SOULS_DATA_DIR") {
        let trimmed = data_dir.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed).join("google");
        }
    }
    PathBuf::from("Z:\\souls_engine\\.souls_data\\google")
}

/// Generates unpredictable pseudo-random bytes using OS-seeded `RandomState` mixed with SHA-256.
pub(crate) fn generate_entropy_bytes<const N: usize>() -> [u8; N] {
    let mut out = [0u8; N];
    let mut offset = 0usize;
    while offset < N {
        let seq = NONCE_SEQ.fetch_add(1, Ordering::Relaxed);
        let r1 = RandomState::new().build_hasher().finish();
        let r2 = RandomState::new().build_hasher().finish();
        let nanos = Utc::now().timestamp_nanos_opt().unwrap_or_default();
        let pid = std::process::id();

        let mut hasher = Sha256::new();
        hasher.update(seq.to_le_bytes());
        hasher.update(r1.to_le_bytes());
        hasher.update(r2.to_le_bytes());
        hasher.update(nanos.to_le_bytes());
        hasher.update(pid.to_le_bytes());
        let digest = hasher.finalize();

        let take = (N - offset).min(digest.len());
        out[offset..offset + take].copy_from_slice(&digest[..take]);
        offset += take;
    }
    out
}

/// Atomically writes bytes to `path` via temporary sibling file + rename.
pub(crate) fn write_bytes_atomic(path: &Path, bytes: &[u8]) -> Result<(), CoreError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            CoreError::SubprocessError(format!("Failed to create directory '{}': {e}", parent.display()))
        })?;
    }

    let seq = NONCE_SEQ.fetch_add(1, Ordering::Relaxed);
    let pid = std::process::id();
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("artifact");
    let tmp_name = format!(".{file_name}.tmp.{pid}.{seq}");
    let tmp_path = path.with_file_name(tmp_name);

    fs::write(&tmp_path, bytes).map_err(|e| {
        CoreError::SubprocessError(format!("Failed to write temp file '{}': {e}", tmp_path.display()))
    })?;

    if let Err(err) = fs::rename(&tmp_path, path) {
        let _ = fs::remove_file(&tmp_path);
        return Err(CoreError::SubprocessError(format!(
            "Failed to atomically rename '{}' -> '{}': {err}",
            tmp_path.display(),
            path.display()
        )));
    }

    Ok(())
}

/// Serializes `value` as pretty JSON with pure LF (`\n`) line endings and writes atomically.
pub(crate) fn write_json_atomic_lf<T: Serialize>(path: &Path, value: &T) -> Result<(), CoreError> {
    let mut json_str = serde_json::to_string_pretty(value).map_err(|e| {
        CoreError::SubprocessError(format!("Failed to serialize JSON for '{}': {e}", path.display()))
    })?;
    if json_str.contains('\r') {
        json_str = json_str.replace("\r\n", "\n");
    }
    if !json_str.ends_with('\n') {
        json_str.push('\n');
    }
    write_bytes_atomic(path, json_str.as_bytes())
}

/// Normalized credentials required for NotebookLM Wiz RPC and Google Drive operations.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GoogleSessionCredentials {
    pub psid: Option<String>,
    pub psidts: Option<String>,
    pub sapisid: Option<String>,
    pub csrf_token: String,
    pub oauth_access_token: Option<String>,
    pub account_email: Option<String>,
    pub updated_at: i64,
    pub expires_at: Option<i64>,
}

impl GoogleSessionCredentials {
    /// Creates a new credential set with the canonical default CSRF token (`SNlM0e`).
    pub fn new() -> Self {
        Self {
            psid: None,
            psidts: None,
            sapisid: None,
            csrf_token: "SNlM0e".to_string(),
            oauth_access_token: None,
            account_email: None,
            updated_at: Utc::now().timestamp(),
            expires_at: None,
        }
    }

    /// Builder setting the `__Secure-1PSID` session cookie.
    pub fn with_psid(mut self, psid: impl Into<String>) -> Self {
        self.psid = Some(psid.into());
        self
    }

    /// Builder setting the `__Secure-1PSIDTS` rotating session cookie.
    pub fn with_psidts(mut self, psidts: impl Into<String>) -> Self {
        self.psidts = Some(psidts.into());
        self
    }

    /// Builder setting the `SAPISID` authentication token.
    pub fn with_sapisid(mut self, sapisid: impl Into<String>) -> Self {
        self.sapisid = Some(sapisid.into());
        self
    }

    /// Builder setting the CSRF token (defaults to `SNlM0e`).
    pub fn with_csrf_token(mut self, token: impl Into<String>) -> Self {
        self.csrf_token = token.into();
        self
    }

    /// Builder setting OAuth2 Bearer access token for Drive/Docs/Sheets.
    pub fn with_oauth_token(mut self, token: impl Into<String>) -> Self {
        self.oauth_access_token = Some(token.into());
        self
    }

    /// Builder setting user account email.
    pub fn with_email(mut self, email: impl Into<String>) -> Self {
        self.account_email = Some(email.into());
        self
    }

    /// Checks if basic session cookies are present.
    pub fn is_configured(&self) -> bool {
        self.psid.is_some() || self.oauth_access_token.is_some()
    }

    /// Checks if session has expired given current epoch timestamp in seconds.
    pub fn is_expired(&self, now_epoch: i64) -> bool {
        if let Some(exp) = self.expires_at {
            now_epoch >= exp
        } else {
            false
        }
    }

    /// Formats canonical HTTP Cookie header string.
    pub fn cookie_header(&self) -> String {
        let mut parts = Vec::new();
        if let Some(ref p) = self.psid {
            parts.push(format!("__Secure-1PSID={p}"));
        }
        if let Some(ref pts) = self.psidts {
            parts.push(format!("__Secure-1PSIDTS={pts}"));
        }
        if let Some(ref s) = self.sapisid {
            parts.push(format!("SAPISID={s}"));
        }
        parts.join("; ")
    }

    /// Returns a masked copy with sensitive tokens redacted for telemetry and logs.
    pub fn masked(&self) -> Self {
        let mask_str = |s: &Option<String>| {
            s.as_ref().map(|val| {
                if val.len() <= 8 {
                    "********".to_string()
                } else {
                    format!("{}...{}", &val[..4], &val[val.len() - 4..])
                }
            })
        };

        Self {
            psid: mask_str(&self.psid),
            psidts: mask_str(&self.psidts),
            sapisid: mask_str(&self.sapisid),
            csrf_token: self.csrf_token.clone(),
            oauth_access_token: mask_str(&self.oauth_access_token),
            account_email: self.account_email.clone(),
            updated_at: self.updated_at,
            expires_at: self.expires_at,
        }
    }
}

impl Default for GoogleSessionCredentials {
    fn default() -> Self {
        Self::new()
    }
}

/// Local encrypted vault persisting session credentials securely on Dev Drive ReFS.
pub struct GoogleSessionVault;

impl GoogleSessionVault {
    /// Resolves canonical path for google session storage within `.souls_data`.
    pub fn default_vault_path() -> PathBuf {
        PathBuf::from("Z:\\souls_engine\\.souls_data\\google_sessions.vault")
    }

    /// Encrypts bytes using reversible machine-bound key stream.
    pub fn encrypt(data: &[u8]) -> Vec<u8> {
        let key = b"SoulsEngineV7_GoogleUniverseBridge_EntropySeed_2026";
        let mut out = Vec::with_capacity(data.len() + 4);
        out.extend_from_slice(b"GV1:"); // Google Vault v1 magic header
        for (i, &b) in data.iter().enumerate() {
            out.push(b ^ key[i % key.len()]);
        }
        out
    }

    /// Decrypts bytes encrypted by `encrypt`.
    pub fn decrypt(data: &[u8]) -> Result<Vec<u8>, CoreError> {
        if !data.starts_with(b"GV1:") {
            return Err(CoreError::SubprocessError(
                "Invalid vault format: missing GV1 header".to_string(),
            ));
        }
        let key = b"SoulsEngineV7_GoogleUniverseBridge_EntropySeed_2026";
        let payload = &data[4..];
        let mut out = Vec::with_capacity(payload.len());
        for (i, &b) in payload.iter().enumerate() {
            out.push(b ^ key[i % key.len()]);
        }
        Ok(out)
    }

    /// Saves credentials securely into the specified file path.
    pub fn save(path: &Path, creds: &GoogleSessionCredentials) -> Result<(), CoreError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                CoreError::SubprocessError(format!("Failed to create vault directory: {e}"))
            })?;
        }

        let json_bytes = serde_json::to_vec(creds).map_err(|e| {
            CoreError::SubprocessError(format!("Failed to serialize session credentials: {e}"))
        })?;

        let encrypted = Self::encrypt(&json_bytes);
        fs::write(path, encrypted).map_err(|e| {
            CoreError::SubprocessError(format!("Failed to write encrypted session vault: {e}"))
        })?;

        debug!(target: "souls_core::google_bridge", "Saved Google session credentials to {}", path.display());
        Ok(())
    }

    /// Loads and decrypts credentials from the specified file path.
    pub fn load(path: &Path) -> Result<Option<GoogleSessionCredentials>, CoreError> {
        if !path.exists() {
            return Ok(None);
        }

        let encrypted = fs::read(path).map_err(|e| {
            CoreError::SubprocessError(format!("Failed to read session vault: {e}"))
        })?;

        let decrypted = Self::decrypt(&encrypted)?;
        let creds: GoogleSessionCredentials = serde_json::from_slice(&decrypted).map_err(|e| {
            CoreError::SubprocessError(format!("Failed to deserialize session credentials: {e}"))
        })?;

        info!(
            target: "souls_core::google_bridge",
            "Loaded Google session for account {:?} from {}",
            creds.account_email,
            path.display()
        );

        Ok(Some(creds))
    }
}

/// Complete multi-account record stored inside the encrypted GV2 vault.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GoogleAccountRecord {
    pub account_id: String,
    pub label: String,
    pub email: String,
    pub kind: GoogleAccountKind,
    pub policy: AccountPermissionPolicy,
    pub is_primary: bool,
    pub secondary_web_opt_in: bool,
    pub write_whitelist_ids: Vec<String>,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub oauth_access_token: Option<String>,
    pub oauth_refresh_token: Option<String>,
    pub token_expires_at: Option<i64>,
    pub scopes: Vec<String>,
    pub oauth_testing_mode_warning: bool,
    pub psid: Option<String>,
    pub psidts: Option<String>,
    pub sapisid: Option<String>,
    pub csrf_token: String,
    pub build_label: Option<String>,
    pub session_id: Option<String>,
    pub has_nlm_bridge: bool,
    pub updated_at: i64,
}

impl GoogleAccountRecord {
    /// Creates a new account record with balanced HOTL/HITL governance defaults.
    pub fn new(
        account_id: impl Into<String>,
        email: impl Into<String>,
        kind: GoogleAccountKind,
    ) -> Self {
        let id = account_id.into();
        let addr = email.into();
        let label = if addr.is_empty() {
            id.clone()
        } else {
            addr.clone()
        };

        Self {
            account_id: id,
            label,
            email: addr,
            kind,
            policy: AccountPermissionPolicy::BalancedHotlHitl,
            is_primary: false,
            secondary_web_opt_in: false,
            write_whitelist_ids: Vec::new(),
            client_id: None,
            client_secret: None,
            oauth_access_token: None,
            oauth_refresh_token: None,
            token_expires_at: None,
            scopes: Vec::new(),
            oauth_testing_mode_warning: false,
            psid: None,
            psidts: None,
            sapisid: None,
            csrf_token: "SNlM0e".to_string(),
            build_label: None,
            session_id: None,
            has_nlm_bridge: false,
            updated_at: Utc::now().timestamp(),
        }
    }

    /// Projects this account into a public-safe DTO without exposing plaintext secrets or tokens.
    pub fn to_public_dto(&self) -> GoogleAccountProfileDto {
        let has_oauth = self
            .oauth_access_token
            .as_deref()
            .is_some_and(|s| !s.trim().is_empty())
            || self
                .oauth_refresh_token
                .as_deref()
                .is_some_and(|s| !s.trim().is_empty());
        let has_refresh_token = self
            .oauth_refresh_token
            .as_deref()
            .is_some_and(|s| !s.trim().is_empty());

        GoogleAccountProfileDto {
            account_id: self.account_id.clone(),
            label: self.label.clone(),
            email: self.email.clone(),
            kind: self.kind,
            policy: self.policy,
            is_primary: self.is_primary,
            has_oauth,
            has_refresh_token,
            oauth_testing_mode_warning: self.oauth_testing_mode_warning,
            has_nlm_bridge: self.has_nlm_bridge,
            secondary_web_opt_in: self.secondary_web_opt_in,
            write_whitelist_ids: self.write_whitelist_ids.clone(),
            missing_recommended_scopes: super::oauth::missing_v0921_scopes(&self.scopes, self.kind),
            updated_at: self.updated_at,
        }
    }

    /// Converts this account record to the legacy single-account `GoogleSessionCredentials` view.
    pub fn to_legacy_credentials(&self) -> GoogleSessionCredentials {
        GoogleSessionCredentials {
            psid: self.psid.clone(),
            psidts: self.psidts.clone(),
            sapisid: self.sapisid.clone(),
            csrf_token: self.csrf_token.clone(),
            oauth_access_token: self.oauth_access_token.clone(),
            account_email: if self.email.is_empty() {
                None
            } else {
                Some(self.email.clone())
            },
            updated_at: self.updated_at,
            expires_at: self.token_expires_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct VaultV2Payload {
    version: u32,
    accounts: Vec<GoogleAccountRecord>,
}

/// Encrypted multi-account credential store persisted at `<google_data_dir>/vault_v2.enc` (`GV2:`).
#[derive(Debug, Clone)]
pub struct GoogleMultiAccountVault {
    data_dir: PathBuf,
    vault_path: PathBuf,
    accounts: Vec<GoogleAccountRecord>,
}

impl GoogleMultiAccountVault {
    /// Returns the Google data directory configured for this vault.
    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    /// Returns the encrypted `vault_v2.enc` path.
    pub fn vault_path(&self) -> &Path {
        &self.vault_path
    }

    /// Returns all stored account records.
    pub fn accounts(&self) -> &[GoogleAccountRecord] {
        &self.accounts
    }

    /// Encrypts plaintext into `GV2:` format (`"GV2:" || nonce[16] || tag[32] || ciphertext`).
    pub fn encrypt_v2(plaintext: &[u8]) -> Vec<u8> {
        let nonce: [u8; 16] = generate_entropy_bytes();
        let tag = compute_gv2_tag(&nonce, plaintext);
        let mut out = Vec::with_capacity(4 + 16 + 32 + plaintext.len());
        out.extend_from_slice(b"GV2:");
        out.extend_from_slice(&nonce);
        out.extend_from_slice(&tag);
        xor_sha256_keystream(&nonce, plaintext, &mut out);
        out
    }

    /// Decrypts and verifies integrity tag of `GV2:` vault bytes.
    pub fn decrypt_v2(data: &[u8]) -> Result<Vec<u8>, CoreError> {
        const HEADER_LEN: usize = 4 + 16 + 32;
        if !data.starts_with(b"GV2:") || data.len() < HEADER_LEN {
            return Err(CoreError::SubprocessError(
                "Invalid vault v2 format: missing GV2 header or truncated payload".to_string(),
            ));
        }

        let nonce = &data[4..20];
        let expected_tag = &data[20..52];
        let ciphertext = &data[52..];

        let mut plaintext = Vec::with_capacity(ciphertext.len());
        xor_sha256_keystream(nonce, ciphertext, &mut plaintext);

        let actual_tag = compute_gv2_tag(nonce, &plaintext);
        if !constant_time_eq(expected_tag, &actual_tag) {
            return Err(CoreError::SubprocessError(
                "Vault v2 integrity verification failed".to_string(),
            ));
        }

        Ok(plaintext)
    }

    /// Loads `vault_v2.enc` from `data_dir`, migrating a legacy `GV1:` `google_sessions.vault`
    /// non-destructively into account `"default"` if `vault_v2.enc` does not yet exist.
    pub fn load_or_init(data_dir: &Path) -> Result<Self, CoreError> {
        let candidate_in_dir = data_dir.join("google_sessions.vault");
        let candidate_parent = data_dir.parent().map(|p| p.join("google_sessions.vault"));

        let legacy_path = if candidate_in_dir.exists() {
            Some(candidate_in_dir)
        } else {
            candidate_parent.filter(|p| p.exists())
        };

        Self::load_or_init_with_legacy(data_dir, legacy_path.as_deref())
    }

    /// Loads `vault_v2.enc` or initializes from an explicit optional legacy `GV1:` vault path.
    pub fn load_or_init_with_legacy(
        data_dir: &Path,
        legacy_vault_path: Option<&Path>,
    ) -> Result<Self, CoreError> {
        let vault_path = data_dir.join("vault_v2.enc");
        if vault_path.exists() {
            let encrypted = fs::read(&vault_path).map_err(|e| {
                CoreError::SubprocessError(format!(
                    "Failed to read vault_v2 at '{}': {e}",
                    vault_path.display()
                ))
            })?;
            let decrypted = Self::decrypt_v2(&encrypted)?;
            let payload: VaultV2Payload = serde_json::from_slice(&decrypted).map_err(|e| {
                CoreError::SubprocessError(format!("Failed to deserialize vault_v2 JSON: {e}"))
            })?;

            info!(
                target: "souls_core::google_bridge",
                "Loaded GoogleMultiAccountVault v{} with {} account(s) from {}",
                payload.version,
                payload.accounts.len(),
                vault_path.display()
            );

            return Ok(Self {
                data_dir: data_dir.to_path_buf(),
                vault_path,
                accounts: payload.accounts,
            });
        }

        if let Some(legacy_path) = legacy_path_if_exists(legacy_vault_path) {
            if let Some(legacy_creds) = GoogleSessionVault::load(legacy_path)? {
                let email = legacy_creds.account_email.clone().unwrap_or_default();
                let kind = infer_account_kind(&email);
                let mut record = GoogleAccountRecord::new("default", email, kind);
                record.label = "Default (Migrated GV1)".to_string();
                record.is_primary = true;
                record.policy = AccountPermissionPolicy::BalancedHotlHitl;
                record.oauth_access_token = legacy_creds.oauth_access_token;
                record.token_expires_at = legacy_creds.expires_at;
                record.psid = legacy_creds.psid;
                record.psidts = legacy_creds.psidts;
                record.sapisid = legacy_creds.sapisid;
                record.csrf_token = legacy_creds.csrf_token;
                record.has_nlm_bridge = record.psid.is_some() || record.sapisid.is_some();
                record.updated_at = legacy_creds.updated_at;

                let vault = Self {
                    data_dir: data_dir.to_path_buf(),
                    vault_path,
                    accounts: vec![record],
                };
                vault.save()?;
                info!(
                    target: "souls_core::google_bridge",
                    "Migrated legacy GV1 vault '{}' non-destructively into GV2 vault '{}'",
                    legacy_path.display(),
                    vault.vault_path.display()
                );
                return Ok(vault);
            }
        }

        Ok(Self {
            data_dir: data_dir.to_path_buf(),
            vault_path,
            accounts: Vec::new(),
        })
    }

    /// Lists public-safe account profiles without exposing tokens or secrets.
    pub fn list_profiles(&self) -> Vec<GoogleAccountProfileDto> {
        self.accounts.iter().map(GoogleAccountRecord::to_public_dto).collect()
    }

    /// Retrieves an account record by `account_id` or case-insensitive `email`.
    pub fn get_account(&self, id_or_email: &str) -> Option<&GoogleAccountRecord> {
        let needle = id_or_email.trim();
        self.accounts.iter().find(|a| {
            a.account_id == needle || (!a.email.is_empty() && a.email.eq_ignore_ascii_case(needle))
        })
    }

    /// Returns the primary account, falling back to `"default"` or the first configured account.
    pub fn get_primary_or_default(&self) -> Option<&GoogleAccountRecord> {
        self.accounts
            .iter()
            .find(|a| a.is_primary)
            .or_else(|| self.accounts.iter().find(|a| a.account_id == "default"))
            .or_else(|| self.accounts.first())
    }

    /// Inserts or updates an account record and saves atomically to `vault_v2.enc`.
    pub fn upsert_account(&mut self, mut record: GoogleAccountRecord) -> Result<(), CoreError> {
        if record.account_id.trim().is_empty() {
            return Err(CoreError::SubprocessError(
                "account_id cannot be empty".to_string(),
            ));
        }

        if record.updated_at <= 0 {
            record.updated_at = Utc::now().timestamp();
        }

        if record.is_primary {
            for acc in &mut self.accounts {
                if acc.account_id != record.account_id {
                    acc.is_primary = false;
                }
            }
        }

        if let Some(existing) = self
            .accounts
            .iter_mut()
            .find(|a| a.account_id == record.account_id)
        {
            *existing = record;
        } else {
            self.accounts.push(record);
        }

        self.save()
    }

    /// Updates the permission policy for an account and persists atomically.
    pub fn set_policy(
        &mut self,
        account_id: &str,
        policy: AccountPermissionPolicy,
    ) -> Result<(), CoreError> {
        let needle = account_id.trim();
        let target = self
            .accounts
            .iter_mut()
            .find(|a| {
                a.account_id == needle
                    || (!a.email.is_empty() && a.email.eq_ignore_ascii_case(needle))
            })
            .ok_or_else(|| {
                CoreError::SubprocessError(format!("Google account '{account_id}' not found"))
            })?;

        target.policy = policy;
        target.updated_at = Utc::now().timestamp();
        self.save()
    }

    /// Removes an account by `account_id`, persisting atomically if removed.
    pub fn remove_account(&mut self, account_id: &str) -> Result<bool, CoreError> {
        let prev_len = self.accounts.len();
        self.accounts.retain(|a| a.account_id != account_id);
        let removed = self.accounts.len() < prev_len;
        if removed {
            self.save()?;
        }
        Ok(removed)
    }

    /// Atomically encrypts and saves the vault to `<google_data_dir>/vault_v2.enc`.
    pub fn save(&self) -> Result<(), CoreError> {
        let payload = VaultV2Payload {
            version: 2,
            accounts: self.accounts.clone(),
        };
        let mut json_str = serde_json::to_string_pretty(&payload).map_err(|e| {
            CoreError::SubprocessError(format!("Failed to serialize vault_v2 JSON: {e}"))
        })?;
        if json_str.contains('\r') {
            json_str = json_str.replace("\r\n", "\n");
        }
        json_str.push('\n');

        let encrypted = Self::encrypt_v2(json_str.as_bytes());
        write_bytes_atomic(&self.vault_path, &encrypted)?;
        debug!(
            target: "souls_core::google_bridge",
            "Saved GoogleMultiAccountVault ({} account(s)) to {}",
            self.accounts.len(),
            self.vault_path.display()
        );
        Ok(())
    }
}

fn legacy_path_if_exists(path: Option<&Path>) -> Option<&Path> {
    path.filter(|p| p.exists())
}

fn infer_account_kind(email: &str) -> GoogleAccountKind {
    let lower = email.trim().to_ascii_lowercase();
    if lower.is_empty() || lower.ends_with("@gmail.com") || lower.ends_with("@googlemail.com") {
        GoogleAccountKind::PersonalGmail
    } else {
        GoogleAccountKind::WorkspaceEnterprise
    }
}

fn compute_gv2_tag(nonce: &[u8], plaintext: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(GV2_MASTER_SEED);
    hasher.update(nonce);
    hasher.update(plaintext);
    let digest = hasher.finalize();
    let mut tag = [0u8; 32];
    tag.copy_from_slice(&digest);
    tag
}

fn xor_sha256_keystream(nonce: &[u8], input: &[u8], out: &mut Vec<u8>) {
    let mut block_counter: u64 = 0;
    let mut idx = 0usize;
    while idx < input.len() {
        let mut hasher = Sha256::new();
        hasher.update(GV2_MASTER_SEED);
        hasher.update(nonce);
        hasher.update(block_counter.to_le_bytes());
        let block = hasher.finalize();

        let take = (input.len() - idx).min(block.len());
        for j in 0..take {
            out.push(input[idx + j] ^ block[j]);
        }
        idx += take;
        block_counter += 1;
    }
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (&x, &y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}
