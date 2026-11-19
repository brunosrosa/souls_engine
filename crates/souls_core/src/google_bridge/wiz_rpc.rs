//! Google Wiz `batchexecute` RPC protocol emulation for Google NotebookLM,
//! pure-Rust RFC 3174 SHA-1 + `SAPISIDHASH` authentication, rate-limiting,
//! and companion `nlm` CLI JSON bridge.

use super::oauth::{find_binary_on_path, percent_encode};
use super::session::GoogleSessionCredentials;
use crate::error::CoreError;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use souls_protocol::google::{NotebookSourceDto, NotebookSummaryDto, StudioArtifactDto};
use std::fmt;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;
use tokio::process::Command;
use tokio::sync::Mutex;
use tokio::time::Instant;
use tracing::{debug, error};

/// Canonical Google NotebookLM internal `batchexecute` RPC identifiers.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WizRpcId {
    /// List all notebooks in the account (`wXbhsf`).
    ListNotebooks,
    /// Retrieve full notebook details and sources (`rLM1Ne`).
    GetNotebook,
    /// Create a new notebook (`CCqFvf`).
    CreateNotebook,
    /// Rename or update notebook emoji/style (`s0tc2d`).
    RenameOrStyleNotebook,
    /// Delete a notebook (`WWINqb`).
    DeleteNotebook,
    /// Inspect or modify notebook sharing status (`JFMDGd`).
    ShareStatus,
    /// Add/ingest a source document (`izAoDd`).
    AddSource,
    /// Add/ingest source document v2 (`ozz5Z`).
    AddSourceV2,
    /// Retrieve source metadata/content (`hizoJc`).
    GetSource,
    /// Check Google Drive source freshness (`yR9Yof`).
    CheckFreshness,
    /// Sync stale Google Drive source (`FLmJqe`).
    SyncDriveSource,
    /// Rename an attached source (`b7Wfje`).
    RenameSource,
    /// Delete an attached source (`tGMBJ`).
    DeleteSource,
    /// Generate or fetch AI source guide (`tr032e`).
    SourceGuide,
    /// List chat conversations in a notebook (`hPTbtc`).
    ListConversations,
    /// Fetch turns in a notebook chat conversation (`khqZz`).
    GetConversationTurns,
    /// Trigger Fast Research investigation (`Ljjv0c`).
    FastResearch,
    /// Trigger Deep Research background investigation (`QA9ei`).
    DeepResearch,
    /// Poll Fast/Deep Research job status (`e3bVqc`).
    PollResearch,
    /// Import discovered research sources into a notebook (`LBwxtb`).
    ImportResearch,
    /// Create a Studio artifact (`R7cb6c`).
    StudioCreate,
    /// Check Studio artifact generation status (`gArtLc`).
    StudioStatus,
    /// Fetch a completed Studio artifact (`v9rmvd`).
    GetArtifact,
    /// Steer or customize an artifact prompt (`rc3d8d`).
    SteerArtifact,
    /// Revise generated presentation slides (`KmcKPe`).
    ReviseSlides,
    /// Generate a mindmap artifact (`yyryJe`).
    GenerateMindmap,
    /// Save a generated mindmap (`CYK0Xb`).
    SaveMindmap,
    /// List saved mindmaps (`cFji9`).
    ListMindmaps,
    /// Direct RAG evidence-based query against notebook sources (`orqRsd`).
    QueryRAG,
    /// Legacy note creation RPC (`TYPJF`) kept for `souls_server` compatibility until Onda 3.
    CreateNote,
    /// Legacy note read RPC (`cXJTyd`) kept for `souls_server` compatibility until Onda 3.
    ReadNotes,
    /// Legacy alias of `AddSource` (`izAoDd`) kept for `souls_server` compatibility.
    IngestSource,
    /// Custom unmapped Wiz RPC identifier.
    Custom(String),
}

impl WizRpcId {
    /// Returns the internal Google Wiz RPC string identifier.
    #[inline]
    pub fn as_str(&self) -> &str {
        match self {
            Self::ListNotebooks => "wXbhsf",
            Self::GetNotebook => "rLM1Ne",
            Self::CreateNotebook => "CCqFvf",
            Self::RenameOrStyleNotebook => "s0tc2d",
            Self::DeleteNotebook => "WWINqb",
            Self::ShareStatus => "JFMDGd",
            Self::AddSource | Self::IngestSource => "izAoDd",
            Self::AddSourceV2 => "ozz5Z",
            Self::GetSource => "hizoJc",
            Self::CheckFreshness => "yR9Yof",
            Self::SyncDriveSource => "FLmJqe",
            Self::RenameSource => "b7Wfje",
            Self::DeleteSource => "tGMBJ",
            Self::SourceGuide => "tr032e",
            Self::ListConversations => "hPTbtc",
            Self::GetConversationTurns => "khqZz",
            Self::FastResearch => "Ljjv0c",
            Self::DeepResearch => "QA9ei",
            Self::PollResearch => "e3bVqc",
            Self::ImportResearch => "LBwxtb",
            Self::StudioCreate => "R7cb6c",
            Self::StudioStatus => "gArtLc",
            Self::GetArtifact => "v9rmvd",
            Self::SteerArtifact => "rc3d8d",
            Self::ReviseSlides => "KmcKPe",
            Self::GenerateMindmap => "yyryJe",
            Self::SaveMindmap => "CYK0Xb",
            Self::ListMindmaps => "cFji9",
            Self::QueryRAG => "orqRsd",
            Self::CreateNote => "TYPJF",
            Self::ReadNotes => "cXJTyd",
            Self::Custom(s) => s.as_str(),
        }
    }

    /// Descriptive title for the RPC call.
    #[inline]
    pub fn description(&self) -> &'static str {
        match self {
            Self::ListNotebooks => "List Account Notebooks",
            Self::GetNotebook => "Get Notebook Details",
            Self::CreateNotebook => "Create Notebook",
            Self::RenameOrStyleNotebook => "Rename or Style Notebook",
            Self::DeleteNotebook => "Delete Notebook",
            Self::ShareStatus => "Notebook Share Status",
            Self::AddSource | Self::IngestSource => "Add / Ingest Source Document",
            Self::AddSourceV2 => "Add Source Document V2",
            Self::GetSource => "Get Source Metadata",
            Self::CheckFreshness => "Check Drive Source Freshness",
            Self::SyncDriveSource => "Sync Drive Source",
            Self::RenameSource => "Rename Source",
            Self::DeleteSource => "Delete Source",
            Self::SourceGuide => "Fetch Source Guide",
            Self::ListConversations => "List Notebook Conversations",
            Self::GetConversationTurns => "Get Conversation Turns",
            Self::FastResearch => "Trigger Fast Research",
            Self::DeepResearch => "Trigger Deep Research",
            Self::PollResearch => "Poll Research Status",
            Self::ImportResearch => "Import Research Sources",
            Self::StudioCreate => "Studio Create Artifact",
            Self::StudioStatus => "Studio Artifact Status",
            Self::GetArtifact => "Get Studio Artifact",
            Self::SteerArtifact => "Steer Studio Artifact",
            Self::ReviseSlides => "Revise Slides Artifact",
            Self::GenerateMindmap => "Generate Mindmap",
            Self::SaveMindmap => "Save Mindmap",
            Self::ListMindmaps => "List Mindmaps",
            Self::QueryRAG => "Notebook Evidence Query",
            Self::CreateNote => "Create Note",
            Self::ReadNotes => "Read Notes",
            Self::Custom(_) => "Custom Wiz RPC",
        }
    }
}

impl fmt::Display for WizRpcId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Pure-Rust zero-dependency RFC 3174 SHA-1 digest.
pub fn sha1_digest(data: &[u8]) -> [u8; 20] {
    let mut h0: u32 = 0x67452301;
    let mut h1: u32 = 0xEFCDAB89;
    let mut h2: u32 = 0x98BADCFE;
    let mut h3: u32 = 0x10325476;
    let mut h4: u32 = 0xC3D2E1F0;

    let bit_len = (data.len() as u64).wrapping_mul(8);
    let mut padded = Vec::with_capacity(data.len() + 72);
    padded.extend_from_slice(data);
    padded.push(0x80);
    while padded.len() % 64 != 56 {
        padded.push(0x00);
    }
    padded.extend_from_slice(&bit_len.to_be_bytes());

    for block in padded.chunks_exact(64) {
        let mut w = [0u32; 80];
        for (i, chunk) in block.chunks_exact(4).enumerate() {
            w[i] = u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        }
        for t in 16..80 {
            w[t] = (w[t - 3] ^ w[t - 8] ^ w[t - 14] ^ w[t - 16]).rotate_left(1);
        }

        let mut a = h0;
        let mut b = h1;
        let mut c = h2;
        let mut d = h3;
        let mut e = h4;

        for (t, &wt) in w.iter().enumerate() {
            let (f, k) = match t {
                0..=19 => ((b & c) | ((!b) & d), 0x5A827999u32),
                20..=39 => (b ^ c ^ d, 0x6ED9EBA1u32),
                40..=59 => ((b & c) | (b & d) | (c & d), 0x8F1BBCDCu32),
                _ => (b ^ c ^ d, 0xCA62C1D6u32),
            };

            let temp = a
                .rotate_left(5)
                .wrapping_add(f)
                .wrapping_add(e)
                .wrapping_add(k)
                .wrapping_add(wt);
            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = temp;
        }

        h0 = h0.wrapping_add(a);
        h1 = h1.wrapping_add(b);
        h2 = h2.wrapping_add(c);
        h3 = h3.wrapping_add(d);
        h4 = h4.wrapping_add(e);
    }

    let mut out = [0u8; 20];
    out[0..4].copy_from_slice(&h0.to_be_bytes());
    out[4..8].copy_from_slice(&h1.to_be_bytes());
    out[8..12].copy_from_slice(&h2.to_be_bytes());
    out[12..16].copy_from_slice(&h3.to_be_bytes());
    out[16..20].copy_from_slice(&h4.to_be_bytes());
    out
}

/// Computes lowercase hex RFC 3174 SHA-1 string.
pub fn sha1_hex(data: &[u8]) -> String {
    let digest = sha1_digest(data);
    let mut out = String::with_capacity(40);
    for b in digest {
        out.push_str(&format!("{b:02x}"));
    }
    out
}

/// Computes Google `Authorization: SAPISIDHASH <ts>_<sha1_hex>` header value.
pub fn compute_sapisidhash(sapisid: &str, origin: &str, timestamp_secs: i64) -> String {
    let clean_origin = origin.trim_end_matches('/');
    let msg = format!("{timestamp_secs} {sapisid} {clean_origin}");
    let digest = sha1_hex(msg.as_bytes());
    format!("SAPISIDHASH {timestamp_secs}_{digest}")
}

/// Outgoing RPC request payload and arguments.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WizRpcPayload {
    pub rpc_id: WizRpcId,
    pub arguments: Value,
}

impl WizRpcPayload {
    /// Creates a new RPC payload for the given identifier and arguments.
    pub fn new(rpc_id: WizRpcId, arguments: Value) -> Self {
        Self { rpc_id, arguments }
    }

    /// Encodes arguments into the Google Wiz `f.req` envelope:
    /// `[[[rpc_id, serialized_args, null, "generic"]]]`
    pub fn to_f_req_json(&self) -> Result<String, CoreError> {
        let args_str = serde_json::to_string(&self.arguments).map_err(|e| {
            CoreError::SubprocessError(format!("Failed to serialize Wiz arguments: {e}"))
        })?;

        let envelope = serde_json::json!([
            [
                [self.rpc_id.as_str(), args_str, null, "generic"]
            ]
        ]);

        serde_json::to_string(&envelope).map_err(|e| {
            CoreError::SubprocessError(format!("Failed to encode Wiz envelope: {e}"))
        })
    }

    /// Formats the raw application/x-www-form-urlencoded body for `batchexecute`.
    pub fn to_form_body(&self, csrf_token: &str) -> Result<String, CoreError> {
        let f_req = self.to_f_req_json()?;
        let encoded_freq = percent_encode(&f_req);
        let encoded_at = percent_encode(csrf_token);
        Ok(format!("f.req={encoded_freq}&at={encoded_at}"))
    }
}

/// Parsed response from a Google Wiz `batchexecute` call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WizRpcResponse {
    pub rpc_id: String,
    pub success: bool,
    pub payload: Option<Value>,
    pub raw_response: String,
}

impl WizRpcResponse {
    /// Parses a raw Google `batchexecute` response, stripping `)]}'\n` anti-XSS markers
    /// and recursively extracting the matching `["wrb.fr", rpc_id, payload, ...]` envelope.
    pub fn parse(raw: &str, target_rpc_id: &str) -> Result<Self, CoreError> {
        let trimmed = raw.trim();
        let body = if let Some(stripped) = trimmed.strip_prefix(")]}'") {
            stripped.trim()
        } else {
            trimmed
        };

        let mut target_payload = None;
        let mut found_rpc = false;

        for line in body.lines() {
            let l = line.trim();
            if l.is_empty() || l.chars().all(|c| c.is_ascii_digit()) {
                continue;
            }

            if let Ok(val) = serde_json::from_str::<Value>(l) {
                if let Some(raw_inner) = find_wrb_fr(&val, target_rpc_id) {
                    found_rpc = true;
                    if let Some(inner_str) = raw_inner.as_str() {
                        if let Ok(parsed_inner) = serde_json::from_str::<Value>(inner_str) {
                            target_payload = Some(parsed_inner);
                        } else {
                            target_payload = Some(Value::String(inner_str.to_string()));
                        }
                    } else {
                        target_payload = Some(raw_inner);
                    }
                    break;
                }
            }
        }

        Ok(Self {
            rpc_id: target_rpc_id.to_string(),
            success: found_rpc,
            payload: target_payload,
            raw_response: raw.to_string(),
        })
    }
}

/// Recursively traverses arbitrary array nesting to locate the canonical Wiz `wrb.fr` envelope.
fn find_wrb_fr(val: &Value, target_rpc_id: &str) -> Option<Value> {
    match val {
        Value::Array(arr) => {
            if arr.len() >= 3 {
                if let Some("wrb.fr") = arr[0].as_str() {
                    if let Some(id) = arr[1].as_str() {
                        if id == target_rpc_id {
                            return arr.get(2).cloned();
                        }
                    }
                }
            }
            for item in arr {
                if let Some(found) = find_wrb_fr(item, target_rpc_id) {
                    return Some(found);
                }
            }
            None
        }
        _ => None,
    }
}

/// Headless client for executing NotebookLM Wiz RPC requests with `SAPISIDHASH` and rate limiting.
pub struct WizRpcClient {
    credentials: GoogleSessionCredentials,
    client: reqwest::Client,
    base_endpoint: String,
    origin: String,
    hl: String,
    min_interval: Duration,
    last_request_at: Arc<Mutex<Option<Instant>>>,
}

impl WizRpcClient {
    /// Creates a new client with the given session credentials and 500ms default rate limit.
    pub fn new(credentials: GoogleSessionCredentials) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_default();

        Self {
            credentials,
            client,
            base_endpoint: "https://notebooklm.google.com/_/LabsNotebookUi/data/batchexecute"
                .to_string(),
            origin: "https://notebooklm.google.com".to_string(),
            hl: "en".to_string(),
            min_interval: Duration::from_millis(500),
            last_request_at: Arc::new(Mutex::new(None)),
        }
    }

    /// Overrides the base RPC endpoint URL (e.g. for mock testing).
    pub fn with_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.base_endpoint = endpoint.into();
        self
    }

    /// Configures the `hl` locale query parameter (e.g. `"pt-BR"` or `"en"`).
    pub fn with_hl(mut self, hl: impl Into<String>) -> Self {
        self.hl = hl.into();
        self
    }

    /// Configures the `Origin` header used for `SAPISIDHASH` computation.
    pub fn with_origin(mut self, origin: impl Into<String>) -> Self {
        self.origin = origin.into();
        self
    }

    /// Configures the minimum spacing between consecutive RPC calls (`Duration::ZERO` for fast unit mocks).
    pub fn with_min_interval(mut self, interval: Duration) -> Self {
        self.min_interval = interval;
        self
    }

    /// Executes an RPC payload against the NotebookLM batchexecute backend.
    pub async fn execute(&self, payload: &WizRpcPayload) -> Result<WizRpcResponse, CoreError> {
        if !self.min_interval.is_zero() {
            let mut last_guard = self.last_request_at.lock().await;
            if let Some(prev) = *last_guard {
                let elapsed = prev.elapsed();
                if elapsed < self.min_interval {
                    tokio::time::sleep(self.min_interval - elapsed).await;
                }
            }
            *last_guard = Some(Instant::now());
        }

        let url = format!(
            "{}?rpcids={}&source-path=%2F&hl={}&rt=c",
            self.base_endpoint,
            percent_encode(payload.rpc_id.as_str()),
            percent_encode(&self.hl)
        );

        let body = payload.to_form_body(&self.credentials.csrf_token)?;
        let cookie_header = self.credentials.cookie_header();

        debug!(
            target: "souls_core::google_bridge",
            "Executing Wiz RPC {}: endpoint={}",
            payload.rpc_id.as_str(),
            url
        );

        let mut req = self
            .client
            .post(&url)
            .header("X-Same-Domain", "1")
            .header("Origin", &self.origin)
            .header("Referer", format!("{}/", self.origin.trim_end_matches('/')))
            .header(
                "Content-Type",
                "application/x-www-form-urlencoded;charset=UTF-8",
            )
            .header(
                "User-Agent",
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36",
            )
            .body(body);

        if !cookie_header.is_empty() {
            req = req.header("Cookie", cookie_header);
        }

        if let Some(ref sapisid) = self.credentials.sapisid {
            if !sapisid.trim().is_empty() {
                let auth_header =
                    compute_sapisidhash(sapisid, &self.origin, Utc::now().timestamp());
                req = req.header("Authorization", auth_header);
            }
        }

        let resp = req.send().await.map_err(|e| {
            CoreError::SubprocessError(format!("Wiz RPC HTTP transport failure: {e}"))
        })?;

        let status = resp.status();
        let raw_text = resp.text().await.map_err(|e| {
            CoreError::SubprocessError(format!("Failed to read Wiz response text: {e}"))
        })?;

        if !status.is_success() {
            error!(
                target: "souls_core::google_bridge",
                "Wiz RPC returned HTTP error {}: {}",
                status,
                raw_text
            );
            return Err(CoreError::SubprocessError(format!(
                "Wiz RPC failed with HTTP {status}: {raw_text}"
            )));
        }

        WizRpcResponse::parse(&raw_text, payload.rpc_id.as_str())
    }
}

/// Companion bridge for the `nlm` CLI (`notebooklm-mcp-cli`) with offline-testable JSON parsers.
#[derive(Debug, Clone)]
pub struct NlmCliBridge {
    binary_path: Option<PathBuf>,
    timeout_ms: u64,
}

impl Default for NlmCliBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl NlmCliBridge {
    /// Auto-discovers `nlm` on PATH with a 30s default command timeout.
    pub fn new() -> Self {
        Self {
            binary_path: find_binary_on_path("nlm"),
            timeout_ms: 30_000,
        }
    }

    /// Creates a bridge pointing to an explicit `nlm` binary path.
    pub fn with_binary(path: PathBuf) -> Self {
        Self {
            binary_path: Some(path),
            timeout_ms: 30_000,
        }
    }

    /// Configures subprocess execution timeout in milliseconds.
    pub fn with_timeout_ms(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }

    /// Parses JSON output from `nlm notebook list --json`.
    pub fn parse_nlm_notebooks_json(json_str: &str) -> Result<Vec<NotebookSummaryDto>, CoreError> {
        let val: Value = serde_json::from_str(json_str).map_err(|e| {
            CoreError::SubprocessError(format!("Invalid nlm notebooks JSON: {e}"))
        })?;

        let arr = extract_json_array(&val, &["notebooks", "items", "data"]);
        Ok(arr
            .into_iter()
            .filter_map(|item| {
                let notebook_id = first_str(&item, &["notebook_id", "id", "uuid"])?;
                let title =
                    first_str(&item, &["title", "name"]).unwrap_or_else(|| "Untitled".to_string());
                let source_count = item
                    .get("source_count")
                    .or_else(|| item.get("sources_count"))
                    .and_then(|v| v.as_u64())
                    .map(|n| n as usize)
                    .or_else(|| {
                        item.get("sources")
                            .and_then(|v| v.as_array())
                            .map(Vec::len)
                    })
                    .unwrap_or(0);
                let modified_at = first_str(&item, &["modified_at", "updated_at", "modifiedTime"]);
                let has_stale_sources = item
                    .get("has_stale_sources")
                    .or_else(|| item.get("stale"))
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let source_backend = first_str(&item, &["source_backend", "backend"])
                    .unwrap_or_else(|| "nlm_cli".to_string());

                Some(NotebookSummaryDto {
                    notebook_id,
                    title,
                    source_count,
                    modified_at,
                    has_stale_sources,
                    source_backend,
                })
            })
            .collect())
    }

    /// Parses JSON output from `nlm source list <id> --drive --json` or `nlm stale sources <id> --json`.
    pub fn parse_nlm_sources_json(json_str: &str) -> Result<Vec<NotebookSourceDto>, CoreError> {
        let val: Value = serde_json::from_str(json_str).map_err(|e| {
            CoreError::SubprocessError(format!("Invalid nlm sources JSON: {e}"))
        })?;

        let arr = extract_json_array(&val, &["sources", "stale_sources", "items", "data"]);
        Ok(arr
            .into_iter()
            .filter_map(|item| {
                let source_id = first_str(&item, &["source_id", "id", "uuid"])?;
                let title = first_str(&item, &["title", "name", "filename"])
                    .unwrap_or_else(|| source_id.clone());
                let source_type = first_str(&item, &["source_type", "type", "kind"])
                    .unwrap_or_else(|| "unknown".to_string());
                let url = first_str(&item, &["url", "drive_url", "web_view_link"]);
                let is_stale_drive = item
                    .get("is_stale_drive")
                    .or_else(|| item.get("is_stale"))
                    .or_else(|| item.get("stale"))
                    .and_then(|v| v.as_bool());
                let status = first_str(&item, &["status", "state"])
                    .unwrap_or_else(|| {
                        if is_stale_drive == Some(true) {
                            "stale".to_string()
                        } else {
                            "ready".to_string()
                        }
                    });

                Some(NotebookSourceDto {
                    source_id,
                    title,
                    source_type,
                    url,
                    is_stale_drive,
                    status,
                })
            })
            .collect())
    }

    /// Parses JSON output from `nlm studio status <id> --json` or `nlm audio create <id> --confirm --json`.
    pub fn parse_nlm_studio_json(
        notebook_id: &str,
        json_str: &str,
    ) -> Result<Vec<StudioArtifactDto>, CoreError> {
        let val: Value = serde_json::from_str(json_str).map_err(|e| {
            CoreError::SubprocessError(format!("Invalid nlm studio JSON: {e}"))
        })?;

        let arr = extract_json_array(&val, &["artifacts", "studio", "items", "data"]);
        Ok(arr
            .into_iter()
            .filter_map(|item| {
                let artifact_id = first_str(&item, &["artifact_id", "id", "uuid"])?;
                let nb_id = first_str(&item, &["notebook_id"])
                    .unwrap_or_else(|| notebook_id.to_string());
                let artifact_type = first_str(&item, &["artifact_type", "type", "kind"])
                    .unwrap_or_else(|| "audio_overview".to_string());
                let title = first_str(&item, &["title", "name"])
                    .unwrap_or_else(|| "Studio Artifact".to_string());
                let status = first_str(&item, &["status", "state"])
                    .unwrap_or_else(|| "pending".to_string());
                let local_audio_url =
                    first_str(&item, &["local_audio_url", "audio_url", "download_url"]);
                let created_at = first_str(&item, &["created_at", "timestamp"]);

                Some(StudioArtifactDto {
                    artifact_id,
                    notebook_id: nb_id,
                    artifact_type,
                    title,
                    status,
                    local_audio_url,
                    created_at,
                })
            })
            .collect())
    }

    /// Invokes `nlm notebook list --json`.
    pub async fn list_notebooks(&self) -> Result<Vec<NotebookSummaryDto>, CoreError> {
        let raw = self.run_raw(&["notebook", "list", "--json"]).await?;
        Self::parse_nlm_notebooks_json(&raw)
    }

    /// Invokes `nlm notebook get <id> --json`.
    pub async fn get_notebook(&self, notebook_id: &str) -> Result<Value, CoreError> {
        let raw = self
            .run_raw(&["notebook", "get", notebook_id, "--json"])
            .await?;
        parse_json_or_raw(&raw)
    }

    /// Invokes `nlm source list <id> --drive --json`.
    pub async fn list_drive_sources(
        &self,
        notebook_id: &str,
    ) -> Result<Vec<NotebookSourceDto>, CoreError> {
        let raw = self
            .run_raw(&["source", "list", notebook_id, "--drive", "--json"])
            .await?;
        Self::parse_nlm_sources_json(&raw)
    }

    /// Invokes `nlm stale sources <id> --json`.
    pub async fn list_stale_sources(
        &self,
        notebook_id: &str,
    ) -> Result<Vec<NotebookSourceDto>, CoreError> {
        let raw = self
            .run_raw(&["stale", "sources", notebook_id, "--json"])
            .await?;
        Self::parse_nlm_sources_json(&raw)
    }

    /// Invokes `nlm sync sources <id> --confirm`.
    pub async fn sync_sources(&self, notebook_id: &str) -> Result<Value, CoreError> {
        let raw = self
            .run_raw(&["sync", "sources", notebook_id, "--confirm"])
            .await?;
        parse_json_or_raw(&raw)
    }

    /// Invokes `nlm query notebook <id> <query> --json`.
    pub async fn query_notebook(&self, notebook_id: &str, query: &str) -> Result<Value, CoreError> {
        let raw = self
            .run_raw(&["query", "notebook", notebook_id, query, "--json"])
            .await?;
        parse_json_or_raw(&raw)
    }

    /// Invokes `nlm audio create <id> --confirm --json`.
    pub async fn create_audio(&self, notebook_id: &str) -> Result<Value, CoreError> {
        let raw = self
            .run_raw(&["audio", "create", notebook_id, "--confirm", "--json"])
            .await?;
        parse_json_or_raw(&raw)
    }

    /// Invokes `nlm studio status <id> --json`.
    pub async fn studio_status(
        &self,
        notebook_id: &str,
    ) -> Result<Vec<StudioArtifactDto>, CoreError> {
        let raw = self
            .run_raw(&["studio", "status", notebook_id, "--json"])
            .await?;
        Self::parse_nlm_studio_json(notebook_id, &raw)
    }

    /// Invokes `nlm download audio <id> -o <path> --no-progress`.
    pub async fn download_audio(
        &self,
        notebook_id: &str,
        output_path: &Path,
    ) -> Result<PathBuf, CoreError> {
        let out_str = output_path.to_string_lossy().to_string();
        let _ = self
            .run_raw(&[
                "download",
                "audio",
                notebook_id,
                "-o",
                &out_str,
                "--no-progress",
            ])
            .await?;
        Ok(output_path.to_path_buf())
    }

    async fn run_raw(&self, args: &[&str]) -> Result<String, CoreError> {
        let bin = self.binary_path.as_ref().ok_or_else(|| {
            CoreError::SubprocessError("nlm CLI binary is not installed on host".to_string())
        })?;

        let mut cmd = Command::new(bin);
        cmd.args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);

        let child = cmd
            .spawn()
            .map_err(|e| CoreError::SubprocessError(format!("Failed to spawn nlm CLI: {e}")))?;

        let output = tokio::time::timeout(
            Duration::from_millis(self.timeout_ms),
            child.wait_with_output(),
        )
        .await
        .map_err(|_| CoreError::ExecutionTimeout(self.timeout_ms))?
        .map_err(|e| CoreError::SubprocessError(format!("nlm CLI I/O failure: {e}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            return Err(CoreError::SubprocessError(format!(
                "nlm CLI failed (code {:?}): {stderr}",
                output.status.code()
            )));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
}

fn extract_json_array(val: &Value, keys: &[&str]) -> Vec<Value> {
    if let Some(arr) = val.as_array() {
        return arr.clone();
    }
    for key in keys {
        if let Some(arr) = val.get(*key).and_then(|v| v.as_array()) {
            return arr.clone();
        }
    }
    if val.is_object() {
        return vec![val.clone()];
    }
    Vec::new()
}

fn first_str(val: &Value, keys: &[&str]) -> Option<String> {
    for key in keys {
        if let Some(s) = val.get(*key).and_then(|v| v.as_str()) {
            if !s.is_empty() {
                return Some(s.to_string());
            }
        }
    }
    None
}

fn parse_json_or_raw(raw: &str) -> Result<Value, CoreError> {
    if raw.trim().is_empty() {
        return Ok(serde_json::json!({}));
    }
    Ok(serde_json::from_str::<Value>(raw).unwrap_or_else(|_| serde_json::json!({ "raw": raw })))
}
