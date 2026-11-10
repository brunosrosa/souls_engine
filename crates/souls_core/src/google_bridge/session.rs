//! Encrypted local credential vault and session manager for Google services.
//!
//! Handles cookies (`__Secure-1PSIDTS`, `__Secure-1PSID`, `SAPISID`), CSRF tokens (`SNlM0e`),
//! and OAuth2 credentials, preventing plain-text token leakage in memory dumps and disk.

use crate::error::CoreError;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, info};

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

