//! Custom Gems catalog store, official Gemini REST dispatch,
//! and strictly gated secondary-sandbox `StreamGenerate` payload builder.

use super::oauth::percent_encode;
use super::session::{write_json_atomic_lf, GoogleAccountRecord};
use super::workspace::parse_json_response;
use crate::error::CoreError;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use souls_protocol::google::GemProfileDto;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

/// Persistent local catalog of Custom Gem personas (`<data_dir>/gems.json`).
pub struct GemsCatalogStore {
    gems_path: PathBuf,
    gems: RwLock<Vec<GemProfileDto>>,
}

impl GemsCatalogStore {
    /// Opens (or initializes) a `GemsCatalogStore` backed by `<data_dir>/gems.json`.
    pub fn open(data_dir: &Path) -> Result<Self, CoreError> {
        fs::create_dir_all(data_dir).map_err(|e| {
            CoreError::SubprocessError(format!(
                "Failed to create gems directory '{}': {e}",
                data_dir.display()
            ))
        })?;

        let gems_path = data_dir.join("gems.json");
        let gems = if gems_path.exists() {
            let raw = fs::read_to_string(&gems_path).map_err(|e| {
                CoreError::SubprocessError(format!(
                    "Failed to read gems catalog '{}': {e}",
                    gems_path.display()
                ))
            })?;
            serde_json::from_str::<Vec<GemProfileDto>>(&raw).map_err(|e| {
                CoreError::SubprocessError(format!(
                    "Failed to parse gems catalog '{}': {e}",
                    gems_path.display()
                ))
            })?
        } else {
            Vec::new()
        };

        Ok(Self {
            gems_path,
            gems: RwLock::new(gems),
        })
    }

    /// Lists all configured Gem profiles.
    pub fn list(&self) -> Vec<GemProfileDto> {
        self.gems
            .read()
            .map(|g| g.clone())
            .unwrap_or_default()
    }

    /// Retrieves a Gem profile by ID.
    pub fn get(&self, id: &str) -> Option<GemProfileDto> {
        self.gems
            .read()
            .ok()
            .and_then(|g| g.iter().find(|item| item.id == id).cloned())
    }

    /// Inserts or updates a Gem profile and persists atomically with LF line endings.
    pub fn upsert(&self, gem: GemProfileDto) -> Result<GemProfileDto, CoreError> {
        if gem.id.trim().is_empty() {
            return Err(CoreError::SubprocessError(
                "Gem id cannot be empty".to_string(),
            ));
        }

        let mut guard = self.gems.write().map_err(|_| {
            CoreError::SubprocessError("Gems catalog lock poisoned".to_string())
        })?;

        if let Some(existing) = guard.iter_mut().find(|item| item.id == gem.id) {
            *existing = gem.clone();
        } else {
            guard.push(gem.clone());
        }

        write_json_atomic_lf(&self.gems_path, &*guard)?;
        Ok(gem)
    }

    /// Removes a Gem profile by ID, persisting changes if removed.
    pub fn remove(&self, id: &str) -> Result<bool, CoreError> {
        let mut guard = self.gems.write().map_err(|_| {
            CoreError::SubprocessError("Gems catalog lock poisoned".to_string())
        })?;

        let prev_len = guard.len();
        guard.retain(|item| item.id != id);
        let removed = guard.len() < prev_len;
        if removed {
            write_json_atomic_lf(&self.gems_path, &*guard)?;
        }
        Ok(removed)
    }
}

/// Dispatches a Gem persona prompt via the official Google AI Studio Gemini REST API
/// (`POST /v1beta/models/{model}:generateContent`).
pub async fn dispatch_gem_official_api(
    client: &reqwest::Client,
    base_url: &str,
    api_key: &str,
    gem: &GemProfileDto,
    user_prompt: &str,
) -> Result<String, CoreError> {
    let base = if base_url.trim().is_empty() {
        "https://generativelanguage.googleapis.com"
    } else {
        base_url.trim_end_matches('/')
    };
    let model = if gem.model.trim().is_empty() {
        "gemini-2.5-flash"
    } else {
        gem.model.trim()
    };

    let url = format!("{base}/v1beta/models/{model}:generateContent");
    let mut body = json!({
        "contents": [
            {
                "role": "user",
                "parts": [{ "text": user_prompt }]
            }
        ]
    });

    if !gem.system_instruction.trim().is_empty() {
        body["systemInstruction"] = json!({
            "parts": [{ "text": gem.system_instruction }]
        });
    }

    let resp = client
        .post(&url)
        .header("x-goog-api-key", api_key)
        .json(&body)
        .send()
        .await
        .map_err(|e| CoreError::SubprocessError(format!("Gemini API dispatch failed: {e}")))?;

    let val = parse_json_response(resp, &url).await?;
    let parts = val
        .pointer("/candidates/0/content/parts")
        .and_then(|v| v.as_array())
        .ok_or_else(|| {
            CoreError::SubprocessError("Gemini API response missing candidates[0].content.parts".to_string())
        })?;

    let text: String = parts
        .iter()
        .filter_map(|p| p.get("text").and_then(|t| t.as_str()))
        .collect::<Vec<_>>()
        .join("");

    Ok(text)
}

/// Positional 69-element `StreamGenerate` payload for explicitly opted-in secondary sandbox accounts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecondaryStreamGeneratePayload {
    pub header_name: String,
    pub header_value: String,
    pub f_req: String,
    pub body_form: String,
}

/// Builds a secondary Gemini Web `StreamGenerate` payload (`inner[19] = gem_id`, length 69)
/// ONLY for non-primary accounts with `secondary_web_opt_in == true`.
///
/// Strictly blocks primary accounts or accounts without explicit opt-in via
/// `CoreError::DangerousCommandBlocked`.
pub fn build_secondary_stream_generate_payload(
    account: &GoogleAccountRecord,
    gem_id: &str,
    prompt: &str,
    model_hash: &str,
) -> Result<SecondaryStreamGeneratePayload, CoreError> {
    if account.is_primary || !account.secondary_web_opt_in {
        return Err(CoreError::DangerousCommandBlocked(format!(
            "Cookie-based StreamGenerate is strictly forbidden on account '{}' (is_primary={}, secondary_web_opt_in={})",
            account.account_id, account.is_primary, account.secondary_web_opt_in
        )));
    }

    let mut inner: Vec<Value> = vec![Value::Null; 69];
    inner[0] = json!([prompt, 0, null, null, null, null, 0]);
    inner[2] = json!(["", "", ""]);
    inner[19] = Value::String(gem_id.to_string());

    let inner_json = serde_json::to_string(&inner).map_err(|e| {
        CoreError::SubprocessError(format!("Failed to serialize StreamGenerate inner array: {e}"))
    })?;
    let f_req = serde_json::to_string(&json!([null, inner_json])).map_err(|e| {
        CoreError::SubprocessError(format!("Failed to serialize StreamGenerate f.req: {e}"))
    })?;

    let header_name = "x-goog-ext-525001261-jspb".to_string();
    let header_value = format!(
        "[1,null,null,null,\"{}\",null,null,0,[4],null,null,2]",
        model_hash.replace('"', "")
    );
    let body_form = format!(
        "f.req={}&at={}",
        percent_encode(&f_req),
        percent_encode(&account.csrf_token)
    );

    Ok(SecondaryStreamGeneratePayload {
        header_name,
        header_value,
        f_req,
        body_form,
    })
}
