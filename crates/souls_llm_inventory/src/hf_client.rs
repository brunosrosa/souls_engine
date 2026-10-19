//! Asynchronous client for Hugging Face Hub REST API.
//!
//! Enables dynamic search for GGUF model repositories and live inspection
//! of available quantization files and Git LFS SHA-256 checksums.

use crate::error::InventoryError;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use serde::{Deserialize, Serialize};
use tracing::debug;

const HF_API_BASE: &str = "https://huggingface.co/api/models";

/// Search result item from Hugging Face Hub API.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HfSearchResult {
    pub repo_id: String,
    pub author: Option<String>,
    pub downloads: u64,
    pub likes: u64,
    pub last_modified: Option<String>,
    pub pipeline_tag: Option<String>,
    pub tags: Vec<String>,
}

/// A specific file within a Hugging Face GGUF repository.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HfModelFile {
    pub file_name: String,
    pub size_bytes: u64,
    pub size_gb: f64,
    pub download_url: String,
    pub sha256: Option<String>,
    pub quant_tag: String,
}

/// Client for communicating with the Hugging Face Hub.
#[derive(Clone)]
pub struct HuggingFaceClient {
    client: reqwest::Client,
}

impl Default for HuggingFaceClient {
    fn default() -> Self {
        Self::new()
    }
}

impl HuggingFaceClient {
    /// Constructs a new HuggingFaceClient with default timeout and User-Agent.
    pub fn new() -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static("SoulsEngine/7.0 (Bare-Metal Silicon OS)"),
        );

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(std::time::Duration::from_secs(12))
            .build()
            .unwrap_or_default();

        Self { client }
    }

    /// Searches Hugging Face Hub for GGUF models matching a text query.
    pub async fn search_models(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<HfSearchResult>, InventoryError> {
        let clean_query = query.trim();
        let url = format!(
            "{}?search={}&filter=gguf&sort=downloads&direction=-1&limit={}",
            HF_API_BASE, clean_query, limit.min(50)
        );

        debug!(url = %url, "Querying Hugging Face models API");

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| InventoryError::HfApiError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(InventoryError::HfApiError(format!(
                "API returned HTTP status {}",
                response.status()
            )));
        }

        let raw_items: Vec<serde_json::Value> = response
            .json()
            .await
            .map_err(|e| InventoryError::HfApiError(e.to_string()))?;

        let results = raw_items
            .into_iter()
            .filter_map(|val| {
                let repo_id = val.get("id")?.as_str()?.to_string();
                let author = val.get("author").and_then(|a| a.as_str()).map(String::from);
                let downloads = val.get("downloads").and_then(|d| d.as_u64()).unwrap_or(0);
                let likes = val.get("likes").and_then(|l| l.as_u64()).unwrap_or(0);
                let last_modified = val.get("lastModified").and_then(|m| m.as_str()).map(String::from);
                let pipeline_tag = val.get("pipeline_tag").and_then(|p| p.as_str()).map(String::from);

                let tags = val
                    .get("tags")
                    .and_then(|t| t.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|item| item.as_str().map(String::from))
                            .collect()
                    })
                    .unwrap_or_default();

                Some(HfSearchResult {
                    repo_id,
                    author,
                    downloads,
                    likes,
                    last_modified,
                    pipeline_tag,
                    tags,
                })
            })
            .collect();

        Ok(results)
    }

    /// Fetches all GGUF files and LFS SHA-256 checksums from a specific repo tree.
    pub async fn get_repo_gguf_files(&self, repo_id: &str) -> Result<Vec<HfModelFile>, InventoryError> {
        let clean_repo = repo_id.trim();
        let url = format!("{}/{}/tree/main", HF_API_BASE, clean_repo);

        debug!(url = %url, "Querying Hugging Face repo tree");

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| InventoryError::HfApiError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(InventoryError::HfApiError(format!(
                "Failed to inspect repo '{}': HTTP status {}",
                clean_repo,
                response.status()
            )));
        }

        let raw_tree: Vec<serde_json::Value> = response
            .json()
            .await
            .map_err(|e| InventoryError::HfApiError(e.to_string()))?;

        let mut files = Vec::new();

        for item in raw_tree {
            let path = match item.get("path").and_then(|p| p.as_str()) {
                Some(p) => p,
                None => continue,
            };

            if !path.ends_with(".gguf") {
                continue;
            }

            let size_bytes = item
                .get("size")
                .and_then(|s| s.as_u64())
                .unwrap_or(0);

            // Hugging Face LFS OID represents the SHA-256 hash of the file
            let sha256 = item
                .get("lfs")
                .and_then(|lfs| lfs.get("oid"))
                .and_then(|oid| oid.as_str())
                .map(String::from);

            let download_url = format!(
                "https://huggingface.co/{}/resolve/main/{}",
                clean_repo, path
            );

            let quant_tag = extract_quant_tag(path);

            files.push(HfModelFile {
                file_name: path.to_string(),
                size_bytes,
                size_gb: size_bytes as f64 / (1024.0 * 1024.0 * 1024.0),
                download_url,
                sha256,
                quant_tag,
            });
        }

        Ok(files)
    }
}

/// Extracts a clean quantization tag (e.g. Q4_K_M, Q5_K_M, Q8_0) from a filename.
pub fn extract_quant_tag(file_name: &str) -> String {
    let lower = file_name.to_lowercase();
    for tag in &[
        "q4_k_m", "q4_k_s", "q4_0", "q4_1",
        "q5_k_m", "q5_k_s", "q5_0", "q5_1",
        "q8_0", "q8_1",
        "q3_k_l", "q3_k_m", "q3_k_s",
        "q6_k", "q2_k", "fp16", "bf16",
    ] {
        if lower.contains(tag) {
            return tag.to_uppercase();
        }
    }
    "UNKNOWN".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_quant_tag() {
        assert_eq!(extract_quant_tag("qwen2.5-coder-7b-instruct-q4_k_m.gguf"), "Q4_K_M");
        assert_eq!(extract_quant_tag("smollm2-135m-instruct-q8_0.gguf"), "Q8_0");
        assert_eq!(extract_quant_tag("model-unknown.gguf"), "UNKNOWN");
    }
}
