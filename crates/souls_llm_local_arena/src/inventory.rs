//! RFC-007: Bare-metal Local Model Inventory & Dynamic Path Resolution.
//!
//! Provides zero-copy catalog scanning, manifest ingestion, and physical path
//! resolution across `%USERPROFILE%\llm_local_models` governed by ADR-004 and ADR-007.

use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

/// Safe WDDM VRAM ceiling for NVIDIA RTX 2060m.
pub const SAFE_WDDM_CEILING_MB: u32 = 5294;

/// Runtime parameters recommended in model manifests.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RuntimeFlags {
    pub engine: String,
    pub context_window: u32,
    pub flags: Vec<String>,
    pub rationale: String,
}

/// Catalog item representing a physical local model on host storage.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LocalModelItem {
    pub model_id: String,
    pub name: String,
    pub tier: String,
    pub file_path: PathBuf,
    pub size_mb: f64,
    pub quantization: String,
    pub context_window: u32,
    pub tags: Vec<String>,
    pub recommended_runtime: Option<RuntimeFlags>,
    pub is_gpu_accelerated: bool,
    pub wddm_safe: bool,
}

/// In-memory catalog of physical local models discovered on host storage.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LocalModelInventory {
    pub models: Vec<LocalModelItem>,
    pub root_dir: PathBuf,
}

impl LocalModelInventory {
    /// Discovers and catalogs all physical local models in the canonical directory.
    pub fn scan() -> Self {
        let root = Self::canonical_models_dir();
        let mut inventory = Self {
            models: Vec::new(),
            root_dir: root.clone(),
        };

        if !root.exists() {
            debug!("Local models root does not exist: {:?}", root);
            return inventory;
        }

        info!("Scanning physical local models directory: {:?}", root);
        inventory.scan_directory(&root);
        inventory
    }

    /// Resolves canonical base directory for local models (%USERPROFILE%\llm_local_models).
    pub fn canonical_models_dir() -> PathBuf {
        if let Ok(override_dir) = std::env::var("SOULS_LOCAL_MODELS_DIR") {
            let p = PathBuf::from(override_dir);
            if p.exists() {
                return p;
            }
        }

        if let Ok(user_profile) = std::env::var("USERPROFILE") {
            let p = PathBuf::from(user_profile).join("llm_local_models");
            if p.exists() {
                return p;
            }
        }

        PathBuf::from(r"C:\Users\rosas\llm_local_models")
    }

    /// Scans a directory recursively for `.gguf` weights and manifest metadata.
    fn scan_directory(&mut self, root: &Path) {
        let entries = match std::fs::read_dir(root) {
            Ok(e) => e,
            Err(_) => return,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // Check if this directory contains GGUF files directly
                let ggufs = Self::find_ggufs_in_dir(&path);
                if !ggufs.is_empty() {
                    self.ingest_model_dir(&path, &ggufs);
                } else {
                    // Recurse into tier subdirectories (e.g. tier_1_master_dgpu)
                    self.scan_directory(&path);
                }
            }
        }

        // Sort models by size ascending (ultralightweight tiers first)
        self.models.sort_by(|a, b| a.size_mb.partial_cmp(&b.size_mb).unwrap_or(std::cmp::Ordering::Equal));
    }

    fn find_ggufs_in_dir(dir: &Path) -> Vec<PathBuf> {
        let mut ggufs = Vec::new();
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_file() {
                    if let Some(ext) = p.extension() {
                        if ext.to_string_lossy().eq_ignore_ascii_case("gguf") {
                            ggufs.push(p);
                        }
                    }
                }
            }
        }
        ggufs
    }

    fn ingest_model_dir(&mut self, dir: &Path, ggufs: &[PathBuf]) {
        // Read manifest if available
        let manifest_path = dir.join("model_manifest.json");
        let alt_manifest_path = dir.join("manifest.json");
        let manifest_val: Option<serde_json::Value> = if manifest_path.exists() {
            std::fs::read_to_string(&manifest_path)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
        } else if alt_manifest_path.exists() {
            std::fs::read_to_string(&alt_manifest_path)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
        } else {
            None
        };

        // Pick primary model weights (ignoring sidecar mmproj for primary identity)
        let primary_gguf = ggufs
            .iter()
            .find(|p| !p.file_name().unwrap_or_default().to_string_lossy().to_lowercase().contains("mmproj"))
            .or_else(|| ggufs.first());

        let primary_gguf = match primary_gguf {
            Some(p) => p,
            None => return,
        };

        let size_bytes = std::fs::metadata(primary_gguf).map(|m| m.len()).unwrap_or(0);
        let size_mb = (size_bytes as f64) / (1024.0 * 1024.0);

        let folder_name = dir.file_name().unwrap_or_default().to_string_lossy().to_string();
        let model_id = manifest_val
            .as_ref()
            .and_then(|m| m.get("model_id").and_then(|v| v.as_str()))
            .unwrap_or(&folder_name)
            .to_string();

        let tier = manifest_val
            .as_ref()
            .and_then(|m| m.get("tier").and_then(|v| v.as_str()))
            .unwrap_or_else(|| Self::classify_tier_by_dir(dir, size_mb))
            .to_string();

        let quantization = manifest_val
            .as_ref()
            .and_then(|m| m.get("quantization").and_then(|v| v.as_str()))
            .unwrap_or_else(|| Self::extract_quant_from_filename(primary_gguf))
            .to_string();

        let context_window = manifest_val
            .as_ref()
            .and_then(|m| m.get("recommended_runtime").and_then(|r| r.get("context_window")).and_then(|v| v.as_u64()))
            .map(|v| v as u32)
            .unwrap_or(8192);

        let tags = Self::assign_specialty_tags(&model_id, &tier);

        let recommended_runtime = manifest_val
            .as_ref()
            .and_then(|m| m.get("recommended_runtime"))
            .and_then(|r| {
                let engine = r.get("engine")?.as_str()?.to_string();
                let context_window = r.get("context_window")?.as_u64()? as u32;
                let flags = r.get("flags")?
                    .as_array()?
                    .iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect();
                let rationale = r.get("rationale").and_then(|v| v.as_str()).unwrap_or("").to_string();

                Some(RuntimeFlags {
                    engine,
                    context_window,
                    flags,
                    rationale,
                })
            });

        let is_gpu = tier.contains("Tier 1") || tier.contains("Tier 3") || tier.contains("Tier 4");
        let wddm_safe = size_mb < (SAFE_WDDM_CEILING_MB as f64);

        self.models.push(LocalModelItem {
            model_id: model_id.clone(),
            name: folder_name,
            tier,
            file_path: primary_gguf.clone(),
            size_mb: (size_mb * 100.0).round() / 100.0,
            quantization,
            context_window,
            tags,
            recommended_runtime,
            is_gpu_accelerated: is_gpu,
            wddm_safe,
        });
    }

    fn classify_tier_by_dir(dir: &Path, size_mb: f64) -> &'static str {
        let parent = dir.parent().and_then(|p| p.file_name()).unwrap_or_default().to_string_lossy();
        if parent.contains("0_bootstrap") || size_mb < 300.0 {
            "Tier 0"
        } else if parent.contains("05_epistemic") || size_mb < 1200.0 {
            "Tier 0.5"
        } else if parent.contains("3_multimodal") {
            "Tier 3"
        } else if parent.contains("4_speculative") {
            "Tier 4"
        } else if parent.contains("2_background") {
            "Tier 2"
        } else {
            "Tier 1"
        }
    }

    fn extract_quant_from_filename(path: &Path) -> &'static str {
        let name = path.file_name().unwrap_or_default().to_string_lossy().to_uppercase();
        if name.contains("Q4_K_M") {
            "Q4_K_M"
        } else if name.contains("Q4_K_S") || name.contains("I1-Q4_K_S") {
            "I1-Q4_K_S"
        } else if name.contains("Q5_K_M") {
            "Q5_K_M"
        } else if name.contains("Q8_0") {
            "Q8_0"
        } else if name.contains("F16") {
            "F16"
        } else {
            "Q4_K_M"
        }
    }

    fn assign_specialty_tags(model_id: &str, tier: &str) -> Vec<String> {
        let lower = model_id.to_lowercase();
        let mut tags = Vec::new();

        if lower.contains("coder") || lower.contains("nemotron") || lower.contains("temper") {
            tags.push("code".to_string());
            tags.push("rust_ast".to_string());
        }
        if lower.contains("function") || lower.contains("xlam") || lower.contains("agentic") {
            tags.push("mcp_tools".to_string());
            tags.push("json_schema".to_string());
        }
        if lower.contains("phi-4") || lower.contains("hy-mt2") || lower.contains("reasoning") {
            tags.push("deep_reasoning".to_string());
            tags.push("cot".to_string());
        }
        if lower.contains("smollm2") || lower.contains("gliclass") || tier.contains("Tier 0") {
            tags.push("fast_triage".to_string());
            tags.push("cpu_avx2".to_string());
        }
        if lower.contains("vl") || tier.contains("Tier 3") {
            tags.push("vision_vlm".to_string());
        }
        if tags.is_empty() {
            tags.push("general".to_string());
        }
        tags
    }

    /// Resolves physical model file path by exact or fuzzy model identifier.
    pub fn resolve_model_path(&self, query: &str) -> Option<PathBuf> {
        let q = query.trim().to_lowercase();
        if q.is_empty() {
            return None;
        }

        // 1. Exact model_id match
        if let Some(item) = self.models.iter().find(|m| m.model_id.eq_ignore_ascii_case(&q)) {
            return Some(item.file_path.clone());
        }

        // 2. Exact folder name match
        if let Some(item) = self.models.iter().find(|m| m.name.eq_ignore_ascii_case(&q)) {
            return Some(item.file_path.clone());
        }

        // 3. Substring match
        if let Some(item) = self.models.iter().find(|m| m.model_id.to_lowercase().contains(&q) || m.name.to_lowercase().contains(&q)) {
            return Some(item.file_path.clone());
        }

        None
    }

    /// Gets catalog item by query.
    pub fn get_model(&self, query: &str) -> Option<&LocalModelItem> {
        let q = query.trim().to_lowercase();
        self.models.iter().find(|m| m.model_id.eq_ignore_ascii_case(&q) || m.model_id.to_lowercase().contains(&q))
    }

    /// Returns all discovered models.
    pub fn list_models(&self) -> &[LocalModelItem] {
        &self.models
    }

    /// Total model count in inventory.
    pub fn len(&self) -> usize {
        self.models.len()
    }

    /// Returns true if catalog is empty.
    pub fn is_empty(&self) -> bool {
        self.models.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inventory_canonical_dir_resolution() {
        let dir = LocalModelInventory::canonical_models_dir();
        assert!(dir.to_string_lossy().contains("llm_local_models"));
    }

    #[test]
    fn test_inventory_scan_and_resolve() {
        let inventory = LocalModelInventory::scan();
        // If models exist on the host, verify non-empty resolution
        if !inventory.is_empty() {
            assert!(inventory.len() >= 10);
            let coder = inventory.resolve_model_path("qwen3-5");
            assert!(coder.is_some(), "Should resolve qwen3-5 model path");
        }
    }
}
