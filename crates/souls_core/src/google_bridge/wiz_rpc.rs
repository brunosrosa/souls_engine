//! Google Wiz `batchexecute` RPC protocol emulation for Google NotebookLM.
//!
//! Reconstructs internal RPC envelopes with CSRF verification (`SNlM0e`),
//! strips anti-XSS prefixes (`)]}'`), and parses multi-chunk payload envelopes.

use super::session::GoogleSessionCredentials;
use crate::error::CoreError;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::Duration;
use tracing::{debug, error};

/// Canonical Google NotebookLM internal `batchexecute` RPC identifiers.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WizRpcId {
    /// Create or edit a notebook note (`wXbhsf`).
    CreateNote,
    /// List, inspect, or query notes in a notebook (`rLM1Ne`).
    ReadNotes,
    /// Ingest a source document (Google Drive doc, local PDF, web URL) (`izAoDd`).
    IngestSource,
    /// Trigger an autonomous Deep Research background investigation (`QA9ei`).
    DeepResearch,
    /// Create a Studio artifact (Audio Overview, Mindmap, Slides) (`R7cb6c`).
    StudioCreate,
    /// Check Studio artifact generation progress (`yyryJe`).
    StudioStatus,
    /// Direct RAG evidence-based query against notebook sources (`orqRsd`).
    QueryRAG,
    /// List all notebooks in the account (`s07Lfd`).
    ListNotebooks,
    /// Custom unmapped Wiz RPC identifier.
    Custom(String),
}

impl WizRpcId {
    /// Returns the internal Google Wiz RPC string identifier.
    #[inline]
    pub fn as_str(&self) -> &str {
        match self {
            Self::CreateNote => "wXbhsf",
            Self::ReadNotes => "rLM1Ne",
            Self::IngestSource => "izAoDd",
            Self::DeepResearch => "QA9ei",
            Self::StudioCreate => "R7cb6c",
            Self::StudioStatus => "yyryJe",
            Self::QueryRAG => "orqRsd",
            Self::ListNotebooks => "s07Lfd",
            Self::Custom(s) => s.as_str(),
        }
    }

    /// Descriptive title for the RPC call.
    #[inline]
    pub fn description(&self) -> &'static str {
        match self {
            Self::CreateNote => "Create / Edit Note",
            Self::ReadNotes => "Read / List Notes",
            Self::IngestSource => "Ingest Source Document",
            Self::DeepResearch => "Trigger Deep Research",
            Self::StudioCreate => "Studio Create Artifact",
            Self::StudioStatus => "Studio Artifact Status",
            Self::QueryRAG => "Notebook Evidence Query",
            Self::ListNotebooks => "List Account Notebooks",
            Self::Custom(_) => "Custom Wiz RPC",
        }
    }
}

impl fmt::Display for WizRpcId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Outgoing RPC request payload and arguments.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WizRpcPayload {
    pub rpc_id: WizRpcId,
    pub arguments: serde_json::Value,
}

impl WizRpcPayload {
    /// Creates a new RPC payload for the given identifier and arguments.
    pub fn new(rpc_id: WizRpcId, arguments: serde_json::Value) -> Self {
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

/// Standard RFC 3986 percent-encoding for form body parameters without external crate dependencies.
fn percent_encode(input: &str) -> String {
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

/// Parsed response from a Google Wiz `batchexecute` call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WizRpcResponse {
    pub rpc_id: String,
    pub success: bool,
    pub payload: Option<serde_json::Value>,
    pub raw_response: String,
}

impl WizRpcResponse {
    /// Parses a raw Google `batchexecute` response, stripping `)]}'\n` anti-XSS markers
    /// and recursively extracting the matching `["wrb.fr", rpc_id, payload, ...]` envelope.
    pub fn parse(raw: &str, target_rpc_id: &str) -> Result<Self, CoreError> {
        // 1. Strip anti-XSS envelope prefix
        let trimmed = raw.trim();
        let body = if let Some(stripped) = trimmed.strip_prefix(")]}'") {
            stripped.trim()
        } else {
            trimmed
        };

        let mut target_payload = None;
        let mut found_rpc = false;

        // 2. Scan lines for JSON arrays containing ["wrb.fr", target_rpc_id, ...]
        for line in body.lines() {
            let l = line.trim();
            if l.is_empty() || l.chars().all(|c| c.is_ascii_digit()) {
                continue;
            }

            if let Ok(val) = serde_json::from_str::<serde_json::Value>(l) {
                if let Some(raw_inner) = find_wrb_fr(&val, target_rpc_id) {
                    found_rpc = true;
                    if let Some(inner_str) = raw_inner.as_str() {
                        if let Ok(parsed_inner) = serde_json::from_str::<serde_json::Value>(inner_str) {
                            target_payload = Some(parsed_inner);
                        } else {
                            target_payload = Some(serde_json::Value::String(inner_str.to_string()));
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
fn find_wrb_fr(val: &serde_json::Value, target_rpc_id: &str) -> Option<serde_json::Value> {
    match val {
        serde_json::Value::Array(arr) => {
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

/// Headless client for executing NotebookLM Wiz RPC requests.
pub struct WizRpcClient {
    credentials: GoogleSessionCredentials,
    client: reqwest::Client,
    base_endpoint: String,
}

impl WizRpcClient {
    /// Creates a new client with the given session credentials.
    pub fn new(credentials: GoogleSessionCredentials) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_default();

        Self {
            credentials,
            client,
            base_endpoint: "https://notebooklm.google.com/_/LabsNotebookUi/data/batchexecute".to_string(),
        }
    }

    /// Overrides the base RPC endpoint URL (e.g. for mock testing).
    pub fn with_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.base_endpoint = endpoint.into();
        self
    }

    /// Executes an RPC payload against the NotebookLM batchexecute backend.
    pub async fn execute(&self, payload: &WizRpcPayload) -> Result<WizRpcResponse, CoreError> {
        let url = format!(
            "{}?rpcids={}&source-path=%2F&hl=en&rt=c",
            self.base_endpoint,
            payload.rpc_id.as_str()
        );

        let body = payload.to_form_body(&self.credentials.csrf_token)?;
        let cookie_header = self.credentials.cookie_header();

        debug!(
            target: "souls_core::google_bridge",
            "Executing Wiz RPC {}: endpoint={}",
            payload.rpc_id.as_str(),
            url
        );

        let mut req = self.client.post(&url)
            .header("X-Same-Domain", "1")
            .header("Content-Type", "application/x-www-form-urlencoded;charset=UTF-8")
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36")
            .body(body);

        if !cookie_header.is_empty() {
            req = req.header("Cookie", cookie_header);
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
