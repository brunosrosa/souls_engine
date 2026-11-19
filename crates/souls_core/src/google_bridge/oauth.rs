//! OAuth2 PKCE (RFC 7636 S256), zero-dependency Base64URL (RFC 4648),
//! `gws` credential discovery, and Google integration doctor diagnostics.

use super::session::{generate_entropy_bytes, GoogleMultiAccountVault};
use crate::error::CoreError;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use souls_protocol::google::{GoogleCliProbeDto, GoogleDoctorReportDto};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const B64URL_ALPHABET: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

/// Encodes arbitrary bytes using RFC 4648 Base64URL without `=` padding.
pub fn base64url_encode(input: &[u8]) -> String {
    if input.is_empty() {
        return String::new();
    }

    let mut out = String::with_capacity((input.len() * 4).div_ceil(3));
    let mut i = 0usize;
    while i + 3 <= input.len() {
        let n = ((input[i] as u32) << 16) | ((input[i + 1] as u32) << 8) | (input[i + 2] as u32);
        out.push(B64URL_ALPHABET[((n >> 18) & 0x3F) as usize] as char);
        out.push(B64URL_ALPHABET[((n >> 12) & 0x3F) as usize] as char);
        out.push(B64URL_ALPHABET[((n >> 6) & 0x3F) as usize] as char);
        out.push(B64URL_ALPHABET[(n & 0x3F) as usize] as char);
        i += 3;
    }

    let rem = input.len() - i;
    if rem == 1 {
        let n = (input[i] as u32) << 16;
        out.push(B64URL_ALPHABET[((n >> 18) & 0x3F) as usize] as char);
        out.push(B64URL_ALPHABET[((n >> 12) & 0x3F) as usize] as char);
    } else if rem == 2 {
        let n = ((input[i] as u32) << 16) | ((input[i + 1] as u32) << 8);
        out.push(B64URL_ALPHABET[((n >> 18) & 0x3F) as usize] as char);
        out.push(B64URL_ALPHABET[((n >> 12) & 0x3F) as usize] as char);
        out.push(B64URL_ALPHABET[((n >> 6) & 0x3F) as usize] as char);
    }

    out
}

/// Decodes RFC 4648 Base64URL (or standard Base64 with/without `=` padding).
pub fn base64url_decode(input: &str) -> Result<Vec<u8>, CoreError> {
    let trimmed = input.trim().trim_end_matches('=');
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }
    if trimmed.len() % 4 == 1 {
        return Err(CoreError::SubprocessError(
            "Invalid Base64URL length".to_string(),
        ));
    }

    let mut out = Vec::with_capacity((trimmed.len() * 3) / 4);
    let bytes = trimmed.as_bytes();
    let mut i = 0usize;

    while i < bytes.len() {
        let remaining = bytes.len() - i;
        let chunk_len = remaining.min(4);
        let mut vals = [0u32; 4];
        for j in 0..chunk_len {
            vals[j] = decode_b64_char(bytes[i + j])? as u32;
        }

        match chunk_len {
            4 => {
                let n = (vals[0] << 18) | (vals[1] << 12) | (vals[2] << 6) | vals[3];
                out.push(((n >> 16) & 0xFF) as u8);
                out.push(((n >> 8) & 0xFF) as u8);
                out.push((n & 0xFF) as u8);
            }
            3 => {
                let n = (vals[0] << 18) | (vals[1] << 12) | (vals[2] << 6);
                out.push(((n >> 16) & 0xFF) as u8);
                out.push(((n >> 8) & 0xFF) as u8);
            }
            2 => {
                let n = (vals[0] << 18) | (vals[1] << 12);
                out.push(((n >> 16) & 0xFF) as u8);
            }
            _ => {
                return Err(CoreError::SubprocessError(
                    "Invalid Base64URL trailing quantum".to_string(),
                ));
            }
        }
        i += chunk_len;
    }

    Ok(out)
}

fn decode_b64_char(b: u8) -> Result<u8, CoreError> {
    match b {
        b'A'..=b'Z' => Ok(b - b'A'),
        b'a'..=b'z' => Ok(b - b'a' + 26),
        b'0'..=b'9' => Ok(b - b'0' + 52),
        b'-' | b'+' => Ok(62),
        b'_' | b'/' => Ok(63),
        _ => Err(CoreError::SubprocessError(format!(
            "Invalid Base64URL byte: {b}"
        ))),
    }
}

/// RFC 3986 percent-encoding helper for OAuth and REST query/form parameters.
pub(crate) fn percent_encode(input: &str) -> String {
    let mut encoded = String::with_capacity(input.len());
    for b in input.bytes() {
        match b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(b as char);
            }
            _ => {
                encoded.push_str(&format!("%{:02X}", b));
            }
        }
    }
    encoded
}

/// RFC 7636 PKCE S256 challenge state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PkceChallenge {
    pub code_verifier: String,
    pub code_challenge: String,
    pub state: String,
}

impl PkceChallenge {
    /// Generates a fresh RFC 7636 PKCE `S256` verifier/challenge pair and CSRF `state`.
    pub fn generate() -> Self {
        let verifier_bytes: [u8; 32] = generate_entropy_bytes();
        let state_bytes: [u8; 18] = generate_entropy_bytes();
        let code_verifier = base64url_encode(&verifier_bytes);
        let code_challenge = Self::compute_s256_challenge(&code_verifier);
        let state = base64url_encode(&state_bytes);
        Self {
            code_verifier,
            code_challenge,
            state,
        }
    }

    /// Computes RFC 7636 `S256` code challenge (`BASE64URL-ENCODE(SHA256(ASCII(verifier)))`).
    pub fn compute_s256_challenge(verifier: &str) -> String {
        let digest = Sha256::digest(verifier.as_bytes());
        base64url_encode(&digest)
    }
}

/// OAuth2 client and endpoint configuration (supports mock endpoint overrides).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OAuth2Config {
    pub client_id: String,
    pub client_secret: Option<String>,
    pub redirect_uri: String,
    pub scopes: Vec<String>,
    pub auth_endpoint: String,
    pub token_endpoint: String,
}

impl OAuth2Config {
    /// Creates an OAuth2 configuration pointing to official Google OAuth2 endpoints.
    pub fn google_default(
        client_id: impl Into<String>,
        client_secret: Option<String>,
        redirect_uri: impl Into<String>,
        scopes: Vec<String>,
    ) -> Self {
        Self {
            client_id: client_id.into(),
            client_secret,
            redirect_uri: redirect_uri.into(),
            scopes,
            auth_endpoint: "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
            token_endpoint: "https://oauth2.googleapis.com/token".to_string(),
        }
    }

    /// Overrides auth and token endpoints (primarily for deterministic local mock tests).
    pub fn with_endpoints(
        mut self,
        auth_endpoint: impl Into<String>,
        token_endpoint: impl Into<String>,
    ) -> Self {
        self.auth_endpoint = auth_endpoint.into();
        self.token_endpoint = token_endpoint.into();
        self
    }
}

/// Parsed OAuth2 token exchange / refresh response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OAuthTokenResponse {
    pub access_token: String,
    #[serde(default)]
    pub refresh_token: Option<String>,
    #[serde(default)]
    pub expires_in: Option<i64>,
    #[serde(default)]
    pub token_type: Option<String>,
    #[serde(default)]
    pub scope: Option<String>,
    #[serde(default)]
    pub id_token: Option<String>,
}

/// Builds a Google OAuth2 authorization URL with PKCE S256 and offline refresh consent.
pub fn build_google_auth_url(
    cfg: &OAuth2Config,
    pkce: &PkceChallenge,
    login_hint: Option<&str>,
) -> String {
    let scope_str = cfg.scopes.join(" ");
    let mut params = vec![
        ("client_id", cfg.client_id.as_str()),
        ("redirect_uri", cfg.redirect_uri.as_str()),
        ("response_type", "code"),
        ("scope", scope_str.as_str()),
        ("access_type", "offline"),
        ("prompt", "consent"),
        ("code_challenge", pkce.code_challenge.as_str()),
        ("code_challenge_method", "S256"),
        ("state", pkce.state.as_str()),
    ];
    if let Some(hint) = login_hint.filter(|h| !h.trim().is_empty()) {
        params.push(("login_hint", hint.trim()));
    }

    let query: String = params
        .into_iter()
        .map(|(k, v)| format!("{}={}", percent_encode(k), percent_encode(v)))
        .collect::<Vec<_>>()
        .join("&");

    let sep = if cfg.auth_endpoint.contains('?') {
        "&"
    } else {
        "?"
    };
    format!("{}{}{}", cfg.auth_endpoint, sep, query)
}

/// Exchanges an authorization code + PKCE `code_verifier` for OAuth2 tokens.
pub async fn exchange_code_for_tokens(
    client: &reqwest::Client,
    cfg: &OAuth2Config,
    code: &str,
    code_verifier: &str,
) -> Result<OAuthTokenResponse, CoreError> {
    let mut pairs = vec![
        ("grant_type", "authorization_code"),
        ("code", code),
        ("redirect_uri", cfg.redirect_uri.as_str()),
        ("client_id", cfg.client_id.as_str()),
        ("code_verifier", code_verifier),
    ];
    if let Some(ref secret) = cfg.client_secret {
        if !secret.is_empty() {
            pairs.push(("client_secret", secret.as_str()));
        }
    }

    post_token_form(client, &cfg.token_endpoint, &pairs).await
}

/// Refreshes an OAuth2 access token using a stored `refresh_token`.
pub async fn refresh_access_token(
    client: &reqwest::Client,
    cfg: &OAuth2Config,
    refresh_token: &str,
) -> Result<OAuthTokenResponse, CoreError> {
    let mut pairs = vec![
        ("grant_type", "refresh_token"),
        ("refresh_token", refresh_token),
        ("client_id", cfg.client_id.as_str()),
    ];
    if let Some(ref secret) = cfg.client_secret {
        if !secret.is_empty() {
            pairs.push(("client_secret", secret.as_str()));
        }
    }

    post_token_form(client, &cfg.token_endpoint, &pairs).await
}

async fn post_token_form(
    client: &reqwest::Client,
    endpoint: &str,
    pairs: &[(&str, &str)],
) -> Result<OAuthTokenResponse, CoreError> {
    let body = pairs
        .iter()
        .map(|(k, v)| format!("{}={}", percent_encode(k), percent_encode(v)))
        .collect::<Vec<_>>()
        .join("&");

    let resp = client
        .post(endpoint)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .map_err(|e| CoreError::SubprocessError(format!("OAuth2 token request failed: {e}")))?;

    let status = resp.status();
    let text = resp
        .text()
        .await
        .map_err(|e| CoreError::SubprocessError(format!("OAuth2 token read failed: {e}")))?;

    if !status.is_success() {
        return Err(CoreError::SubprocessError(format!(
            "OAuth2 token endpoint returned HTTP {status}: {text}"
        )));
    }

    serde_json::from_str::<OAuthTokenResponse>(&text).map_err(|e| {
        CoreError::SubprocessError(format!("Failed to parse OAuth2 token response: {e}"))
    })
}

/// Credentials imported from local `gws` CLI configuration directory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportedGwsConfig {
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub redirect_uris: Vec<String>,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub scopes: Vec<String>,
    pub email: Option<String>,
    pub source_dir: PathBuf,
}

/// Attempts to import existing `gws` CLI configuration from `%USERPROFILE%\.config\gws`
/// (or `$HOME/.config/gws`). Returns `Ok(None)` safely if absent.
pub fn import_gws_credentials() -> Result<Option<ImportedGwsConfig>, CoreError> {
    let home = std::env::var("USERPROFILE")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .or_else(|| std::env::var("HOME").ok().filter(|s| !s.trim().is_empty()));

    let Some(home_dir) = home else {
        return Ok(None);
    };
    let gws_dir = PathBuf::from(home_dir).join(".config").join("gws");
    import_gws_credentials_from_dir(&gws_dir)
}

/// Parsed fields extracted from an official Google Cloud `client_secret.json` file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParsedOAuthClientJson {
    pub client_id: String,
    pub client_secret: Option<String>,
    pub redirect_uris: Vec<String>,
    pub project_id: Option<String>,
}

/// Parses an official Google Cloud OAuth client JSON (`installed`, `web`, or flat object).
pub fn parse_google_client_secret_json(raw: &str) -> Result<ParsedOAuthClientJson, CoreError> {
    let val: serde_json::Value = serde_json::from_str(raw)
        .map_err(|e| CoreError::SubprocessError(format!("Invalid OAuth client JSON: {e}")))?;

    let root = val
        .get("installed")
        .or_else(|| val.get("web"))
        .unwrap_or(&val);

    let client_id = root
        .get("client_id")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| {
            CoreError::SubprocessError(
                "Missing 'client_id' field in OAuth client JSON (expected Google Cloud 'installed' or 'web' format).".to_string(),
            )
        })?
        .to_string();

    let client_secret = root
        .get("client_secret")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToString::to_string);

    let redirect_uris = root
        .get("redirect_uris")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(str::trim).filter(|s| !s.is_empty()).map(ToString::to_string))
                .collect()
        })
        .unwrap_or_default();

    let project_id = root
        .get("project_id")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToString::to_string);

    Ok(ParsedOAuthClientJson {
        client_id,
        client_secret,
        redirect_uris,
        project_id,
    })
}

/// Reads `client_secret.json` and `credentials.json` from an explicit `gws` config directory.
pub fn import_gws_credentials_from_dir(
    gws_dir: &Path,
) -> Result<Option<ImportedGwsConfig>, CoreError> {
    let client_secret_path = gws_dir.join("client_secret.json");
    let credentials_path = gws_dir.join("credentials.json");

    if !client_secret_path.exists() && !credentials_path.exists() {
        return Ok(None);
    }

    let mut config = ImportedGwsConfig {
        client_id: None,
        client_secret: None,
        redirect_uris: Vec::new(),
        access_token: None,
        refresh_token: None,
        scopes: Vec::new(),
        email: None,
        source_dir: gws_dir.to_path_buf(),
    };

    if client_secret_path.exists() {
        let raw = fs::read_to_string(&client_secret_path).map_err(|e| {
            CoreError::SubprocessError(format!(
                "Failed to read '{}': {e}",
                client_secret_path.display()
            ))
        })?;
        let parsed = parse_google_client_secret_json(&raw).map_err(|e| {
            CoreError::SubprocessError(format!(
                "Invalid JSON in '{}': {e}",
                client_secret_path.display()
            ))
        })?;
        config.client_id = Some(parsed.client_id);
        config.client_secret = parsed.client_secret;
        config.redirect_uris = parsed.redirect_uris;
    }

    if credentials_path.exists() {
        let raw = fs::read_to_string(&credentials_path).map_err(|e| {
            CoreError::SubprocessError(format!(
                "Failed to read '{}': {e}",
                credentials_path.display()
            ))
        })?;
        let val: serde_json::Value = serde_json::from_str(&raw).map_err(|e| {
            CoreError::SubprocessError(format!(
                "Invalid JSON in '{}': {e}",
                credentials_path.display()
            ))
        })?;

        if config.client_id.is_none() {
            config.client_id = val
                .get("client_id")
                .and_then(|v| v.as_str())
                .map(ToString::to_string);
        }
        if config.client_secret.is_none() {
            config.client_secret = val
                .get("client_secret")
                .and_then(|v| v.as_str())
                .map(ToString::to_string);
        }
        config.access_token = val
            .get("access_token")
            .or_else(|| val.get("token"))
            .and_then(|v| v.as_str())
            .map(ToString::to_string);
        config.refresh_token = val
            .get("refresh_token")
            .and_then(|v| v.as_str())
            .map(ToString::to_string);
        config.email = val
            .get("email")
            .or_else(|| val.get("account"))
            .and_then(|v| v.as_str())
            .map(ToString::to_string);

        if let Some(arr) = val.get("scopes").and_then(|v| v.as_array()) {
            config.scopes = arr
                .iter()
                .filter_map(|v| v.as_str().map(ToString::to_string))
                .collect();
        } else if let Some(s) = val.get("scope").and_then(|v| v.as_str()) {
            config.scopes = s.split_whitespace().map(ToString::to_string).collect();
        }
    }

    Ok(Some(config))
}

/// Runs diagnostic probes across companion Google CLIs (`gws`, `nlm`, `gemini`, `clasp`),
/// vault accounts, OAuth Testing/Production status, Keep Enterprise limits, and Routes v2 readiness.
pub fn run_google_doctor(
    data_dir: &Path,
    vault: &GoogleMultiAccountVault,
) -> GoogleDoctorReportDto {
    let has_oauth_account = vault.accounts().iter().any(|a| {
        a.oauth_access_token.as_deref().is_some_and(|t| !t.is_empty())
            || a.oauth_refresh_token.as_deref().is_some_and(|t| !t.is_empty())
    });
    let has_nlm_account = vault
        .accounts()
        .iter()
        .any(|a| a.has_nlm_bridge || a.psid.is_some() || a.sapisid.is_some());

    let clis = std::thread::scope(|s| {
        let h_gws = s.spawn(|| probe_cli("gws", has_oauth_account || gws_credentials_exist()));
        let h_nlm = s.spawn(|| probe_cli("nlm", has_nlm_account || nlm_auth_profile_exists()));
        let h_gemini = s.spawn(|| {
            probe_cli(
                "gemini",
                env_non_empty("GEMINI_API_KEY") || has_oauth_account,
            )
        });
        let h_clasp = s.spawn(|| probe_cli("clasp", clasp_auth_exists()));
        vec![
            h_gws.join().unwrap_or_else(|_| probe_cli_fallback("gws")),
            h_nlm.join().unwrap_or_else(|_| probe_cli_fallback("nlm")),
            h_gemini.join().unwrap_or_else(|_| probe_cli_fallback("gemini")),
            h_clasp.join().unwrap_or_else(|_| probe_cli_fallback("clasp")),
        ]
    });

    let accounts_count = vault.accounts().len();
    let mut warnings = Vec::new();

    if accounts_count == 0 {
        warnings.push(format!(
            "No Google accounts configured in vault at '{}'.",
            data_dir.display()
        ));
    }

    for account in vault.accounts() {
        if account.oauth_testing_mode_warning {
            warnings.push(format!(
                "Account '{}' ({}) has oauth_testing_mode_warning=true: OAuth refresh tokens in Testing mode expire after 7 days.",
                account.account_id, account.email
            ));
        }
    }

    let routes_v2_ready =
        env_non_empty("ROUTES_API_KEY") || env_non_empty("GOOGLE_MAPS_API_KEY");
    if !routes_v2_ready {
        warnings.push(
            "Routes API v2 key not configured: set ROUTES_API_KEY or GOOGLE_MAPS_API_KEY for traffic-aware commute alerts."
                .to_string(),
        );
    }

    let overall_status = if accounts_count == 0 {
        "unconfigured".to_string()
    } else if warnings.is_empty() {
        "healthy".to_string()
    } else {
        "degraded".to_string()
    };

    GoogleDoctorReportDto {
        overall_status,
        clis,
        accounts_count,
        oauth_production_guidance: "Google Cloud Console -> APIs & Services -> OAuth consent screen: if Publishing status is 'Testing', refresh tokens expire every 7 days. Click 'Publish App' to switch to 'In Production' (for personal use with <= 100 personal/test accounts, you can operate without completing public Google verification by proceeding through the personal unverified-app consent prompt once)."
            .to_string(),
        keep_enterprise_note: "Google Keep API is restricted exclusively to Google Workspace Enterprise domains with Domain-Wide Delegation. For personal @gmail.com accounts, Souls Engine automatically uses Google Tasks API + Obsidian Quick Capture (raw/inbox) instead of Keep."
            .to_string(),
        routes_v2_ready,
        warnings,
    }
}

fn env_non_empty(key: &str) -> bool {
    std::env::var(key)
        .map(|v| !v.trim().is_empty())
        .unwrap_or(false)
}

fn home_dir_path() -> Option<PathBuf> {
    std::env::var("USERPROFILE")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .or_else(|| std::env::var("HOME").ok().filter(|s| !s.trim().is_empty()))
        .map(PathBuf::from)
}

fn gws_credentials_exist() -> bool {
    home_dir_path()
        .map(|h| h.join(".config").join("gws").join("credentials.json").exists())
        .unwrap_or(false)
}

fn nlm_auth_profile_exists() -> bool {
    home_dir_path()
        .map(|h| h.join(".notebooklm-mcp-cli").exists())
        .unwrap_or(false)
}

fn clasp_auth_exists() -> bool {
    home_dir_path()
        .map(|h| h.join(".clasprc.json").exists())
        .unwrap_or(false)
}

fn probe_cli_fallback(name: &str) -> GoogleCliProbeDto {
    GoogleCliProbeDto {
        name: name.to_string(),
        installed: false,
        path: None,
        version: None,
        authenticated: false,
        notes: format!("Probe thread panicked for '{name}'."),
    }
}

fn probe_cli(name: &str, authenticated_hint: bool) -> GoogleCliProbeDto {
    let resolved = find_binary_on_path(name);
    let Some(bin_path) = resolved else {
        return GoogleCliProbeDto {
            name: name.to_string(),
            installed: false,
            path: None,
            version: None,
            authenticated: false,
            notes: format!("Binary '{name}' not found in PATH or standard user bin directories."),
        };
    };

    let version = run_version_probe(&bin_path, Duration::from_millis(7000));
    let notes = match &version {
        Some(v) => format!("Detected version: {v}"),
        None => "Binary present on disk; version probe timed out or returned non-zero.".to_string(),
    };

    GoogleCliProbeDto {
        name: name.to_string(),
        installed: true,
        path: Some(bin_path.display().to_string()),
        version,
        authenticated: authenticated_hint,
        notes,
    }
}

fn run_version_probe(bin_path: &Path, timeout: Duration) -> Option<String> {
    #[cfg(windows)]
    let mut cmd = {
        let is_batch = bin_path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| e.eq_ignore_ascii_case("cmd") || e.eq_ignore_ascii_case("bat"));
        if is_batch {
            let mut c = Command::new("cmd");
            c.args(["/C", &bin_path.to_string_lossy(), "--version"]);
            c
        } else {
            let mut c = Command::new(bin_path);
            c.arg("--version");
            c
        }
    };
    #[cfg(not(windows))]
    let mut cmd = {
        let mut c = Command::new(bin_path);
        c.arg("--version");
        c
    };

    let mut child = cmd
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .ok()?;

    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_status)) => {
                let output = child.wait_with_output().ok()?;
                let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                let first_line = if !stdout.is_empty() {
                    stdout.lines().next().unwrap_or("").trim().to_string()
                } else {
                    stderr.lines().next().unwrap_or("").trim().to_string()
                };
                return if first_line.is_empty() {
                    None
                } else {
                    Some(first_line)
                };
            }
            Ok(None) => {
                if start.elapsed() >= timeout {
                    let _ = child.kill();
                    let _ = child.wait();
                    return None;
                }
                std::thread::sleep(Duration::from_millis(25));
            }
            Err(_) => {
                let _ = child.kill();
                return None;
            }
        }
    }
}

/// Resolves an executable or Windows batch wrapper in a directory list, prioritizing `.exe`/`.cmd`/`.bat` on Windows.
pub fn find_binary_in_dirs(name: &str, search_dirs: &[PathBuf]) -> Option<PathBuf> {
    for dir in search_dirs {
        #[cfg(windows)]
        {
            for ext in ["exe", "cmd", "bat"] {
                let candidate = dir.join(format!("{name}.{ext}"));
                if candidate.is_file() {
                    return Some(candidate);
                }
            }
        }
        let direct = dir.join(name);
        if direct.is_file() {
            return Some(direct);
        }
    }
    None
}

/// Finds an executable binary on `PATH` and standard user tool directories (`~/.local/bin`, `~/.cargo/bin`, `%APPDATA%\npm`).
pub fn find_binary_on_path(name: &str) -> Option<PathBuf> {
    let mut search_dirs: Vec<PathBuf> = Vec::new();
    if let Ok(path_env) = std::env::var("PATH") {
        search_dirs.extend(std::env::split_paths(&path_env));
    }
    if let Some(home) = home_dir_path() {
        search_dirs.push(home.join(".local").join("bin"));
        search_dirs.push(home.join(".cargo").join("bin"));
    }
    if let Ok(appdata) = std::env::var("APPDATA") {
        if !appdata.trim().is_empty() {
            search_dirs.push(PathBuf::from(appdata).join("npm"));
        }
    }

    find_binary_in_dirs(name, &search_dirs)
}
