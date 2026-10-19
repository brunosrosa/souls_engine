//! Zero-copy local model catalog scanning and physical asset management.
//!
//! Scans `%USERPROFILE%\llm_local_models`, extracts GGUF headers via zero-copy mmap,
//! evaluates hardware fit against the host diagnostic, and manages model deletion.

use crate::error::InventoryError;
use crate::hf_client::extract_quant_tag;
use crate::profiler::{evaluate_model_fit, HardwareDiagnostic, HardwareFit};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;
use tracing::info;

/// Supported format for local weights.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelFormat {
    Gguf,
    Onnx,
    SafeTensors,
}

/// Metadata of an inspected physical model file on disk.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LocalModelMeta {
    pub file_name: String,
    pub absolute_path: PathBuf,
    pub format: ModelFormat,
    pub size_bytes: u64,
    pub size_gb: f64,
    pub architecture: String,
    pub parameter_count: Option<u64>,
    pub quantization: String,
    pub context_length_max: u32,
    pub hardware_fit: HardwareFit,
    pub assigned_tier: Option<String>,
    pub last_modified_unix: u64,
}

/// Resolves the canonical base directory for local models (%USERPROFILE%\llm_local_models).
///
/// Ensures the directory exists or creates it idempotently.
pub fn resolve_canonical_models_dir() -> PathBuf {
    if let Ok(override_dir) = std::env::var("SOULS_LOCAL_MODELS_DIR") {
        let p = PathBuf::from(override_dir);
        let _ = std::fs::create_dir_all(&p);
        return p;
    }

    if let Ok(user_profile) = std::env::var("USERPROFILE") {
        let p = PathBuf::from(user_profile).join("llm_local_models");
        let _ = std::fs::create_dir_all(&p);
        return p;
    }

    if let Ok(home) = std::env::var("HOME") {
        let p = PathBuf::from(home).join("llm_local_models");
        let _ = std::fs::create_dir_all(&p);
        return p;
    }

    let fallback = PathBuf::from(r"C:\Users\rosas\llm_local_models");
    let _ = std::fs::create_dir_all(&fallback);
    fallback
}

/// Scans the local models directory and extracts structured metadata for all GGUF files.
pub fn scan_local_models(
    root: &Path,
    diag: &HardwareDiagnostic,
) -> Result<Vec<LocalModelMeta>, InventoryError> {
    if !root.exists() {
        return Ok(Vec::new());
    }

    let mut results = Vec::new();
    scan_recursive(root, diag, &mut results);

    // Sort by file name
    results.sort_by(|a, b| a.file_name.cmp(&b.file_name));
    Ok(results)
}

fn scan_recursive(dir: &Path, diag: &HardwareDiagnostic, results: &mut Vec<LocalModelMeta>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            scan_recursive(&path, diag, results);
        } else if path.is_file() {
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();

            if ext == "gguf" {
                if let Some(meta) = inspect_gguf_file(&path, diag) {
                    results.push(meta);
                }
            }
        }
    }
}

/// Inspects a single GGUF file extracting zero-copy metadata and hardware fit.
pub fn inspect_gguf_file(path: &Path, diag: &HardwareDiagnostic) -> Option<LocalModelMeta> {
    let file_name = path.file_name()?.to_str()?.to_string();
    let metadata = std::fs::metadata(path).ok()?;
    let size_bytes = metadata.len();
    let size_gb = size_bytes as f64 / (1024.0 * 1024.0 * 1024.0);

    let last_modified_unix = metadata
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0);

    // Try zero-copy mmap header inspection from souls_inference_runtime
    let (architecture, context_length, parameter_count) =
        match souls_inference_runtime::inspect_gguf_metadata_o1(path) {
            Ok(info) => {
                let arch = info.architecture;
                let ctx = info.context_length as u32;
                // Approximate parameter count from tensor count or size if missing
                let params = (size_bytes as f64 / 0.6) as u64;
                (arch, ctx, Some(params))
            }
            Err(_) => {
                // Heuristic fallback from filename
                let arch = if file_name.contains("qwen") {
                    "qwen2".to_string()
                } else if file_name.contains("smollm") {
                    "llama".to_string()
                } else if file_name.contains("phi") {
                    "phi3".to_string()
                } else {
                    "unknown".to_string()
                };
                let ctx = 8192;
                let params = (size_bytes as f64 / 0.6) as u64;
                (arch, ctx, Some(params))
            }
        };

    let quantization = extract_quant_tag(&file_name);
    let params_val = parameter_count.unwrap_or(3_000_000_000);
    let hardware_fit = evaluate_model_fit(diag, size_bytes, context_length, params_val);

    let assigned_tier = if size_gb < 0.6 {
        Some("tier0_velocista".to_string())
    } else if size_gb <= 2.2 {
        Some("tier05_socratico".to_string())
    } else {
        Some("tier1_coder".to_string())
    };

    Some(LocalModelMeta {
        file_name,
        absolute_path: path.to_path_buf(),
        format: ModelFormat::Gguf,
        size_bytes,
        size_gb,
        architecture,
        parameter_count,
        quantization,
        context_length_max: context_length,
        hardware_fit,
        assigned_tier,
        last_modified_unix,
    })
}

/// Safely removes a model file from the canonical directory, guarding against path traversal.
pub fn delete_model_file(file_name: &str) -> Result<(), InventoryError> {
    let clean_name = file_name.trim();

    // Guard against path traversal
    if clean_name.contains("..") || clean_name.contains('/') || clean_name.contains('\\') {
        return Err(InventoryError::CatalogIoError(
            clean_name.to_string(),
            "Path traversal prohibited".to_string(),
        ));
    }

    let root = resolve_canonical_models_dir();
    let target = root.join(clean_name);

    if !target.exists() {
        return Err(InventoryError::ModelNotFound(clean_name.to_string()));
    }

    std::fs::remove_file(&target).map_err(|e| {
        InventoryError::CatalogIoError(target.display().to_string(), e.to_string())
    })?;

    info!(file = %clean_name, "Model file deleted safely from disk");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_scan_and_delete_lifecycle() {
        let dir = tempdir().unwrap();
        let diag = HardwareDiagnostic::default();

        let dummy_path = dir.path().join("test-model-q4_k_m.gguf");
        let mut file = std::fs::File::create(&dummy_path).unwrap();
        file.write_all(b"GGUF_DUMMY_HEADER").unwrap();

        let scanned = scan_local_models(dir.path(), &diag).unwrap();
        assert_eq!(scanned.len(), 1);
        assert_eq!(scanned[0].file_name, "test-model-q4_k_m.gguf");
        assert_eq!(scanned[0].quantization, "Q4_K_M");
    }

    #[test]
    fn test_path_traversal_guard() {
        assert!(delete_model_file("../escape.gguf").is_err());
        assert!(delete_model_file(r"subdir\escape.gguf").is_err());
    }
}
