//! File system scanner for discovered local models across standard directories.

use std::path::{Path, PathBuf};
use tracing::{debug, info};
use crate::domain::model::ModelDescriptor;

/// Standard model search directories on the host machine.
pub fn default_model_directories() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    if let Ok(user_profile) = std::env::var("USERPROFILE") {
        // 1. Unified Canonical silicon repository (Priority 1)
        let unified_models = PathBuf::from(&user_profile).join("llm_local_models");
        if unified_models.exists() {
            dirs.push(unified_models);
        }

        // 2. LMStudio primary models cache
        let lm_studio = PathBuf::from(&user_profile).join(".lmstudio").join("models");
        if lm_studio.exists() {
            dirs.push(lm_studio);
        }
    }

    // 3. Dev Drive ReFS data models folder
    let dev_drive_models = PathBuf::from("Z:/souls_engine/.souls_data/models");
    if dev_drive_models.exists() {
        dirs.push(dev_drive_models);
    }

    // Deduplicate directories
    let mut unique_dirs = Vec::new();
    for d in dirs {
        let canonical = d.canonicalize().unwrap_or_else(|_| d.clone());
        if !unique_dirs.iter().any(|existing: &PathBuf| {
            existing.canonicalize().unwrap_or_else(|_| existing.clone()) == canonical
        }) {
            unique_dirs.push(d);
        }
    }

    unique_dirs
}

/// Recursively scans specified directories for `.gguf` and `.onnx` models.
pub fn scan_local_models<P: AsRef<Path>>(search_paths: &[P]) -> Vec<ModelDescriptor> {
    let mut descriptors = Vec::new();

    for p in search_paths {
        let root = p.as_ref();
        if !root.exists() {
            continue;
        }
        info!("Scanning directory for local models: {:?}", root);
        scan_dir_recursive(root, &mut descriptors);
    }

    // Deduplicate by canonical file path
    let mut unique_descriptors = Vec::new();
    let mut seen_paths = std::collections::HashSet::new();

    for desc in descriptors {
        let canonical_str = desc.file_path.canonicalize()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| desc.file_path.to_string_lossy().to_string());

        if seen_paths.insert(canonical_str) {
            unique_descriptors.push(desc);
        }
    }

    debug!("Total models discovered on host: {}", unique_descriptors.len());
    unique_descriptors
}

fn scan_dir_recursive(dir: &Path, acc: &mut Vec<ModelDescriptor>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            scan_dir_recursive(&path, acc);
        } else if path.is_file() {
            let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
            if ext == "gguf" || ext == "onnx" {
                let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
                // Do not add standalone vision projectors as primary LLM models
                if filename.contains("mmproj") || filename.contains("-vision.gguf") {
                    continue;
                }
                let mut desc = ModelDescriptor::from_file(&path);

                // If model_manifest.json exists in the same directory, read authoritative metadata
                let parent = path.parent().unwrap_or(Path::new(""));
                let manifest_path = parent.join("model_manifest.json");
                if manifest_path.exists() {
                    if let Ok(content) = std::fs::read_to_string(&manifest_path) {
                        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                            if let Some(m_id) = val.get("model_id").and_then(|v| v.as_str()) {
                                desc.model_id = m_id.to_string();
                            }
                            if let Some(tier_str) = val.get("tier").and_then(|v| v.as_str()) {
                                if let Some(t) = crate::domain::model::ModelTier::parse(tier_str) {
                                    desc.tier = t;
                                }
                            }
                            if let Some(params) = val.get("parameters").and_then(|v| v.as_str()) {
                                desc.parameters = params.to_string();
                            }
                            if let Some(quant) = val.get("quantization").and_then(|v| v.as_str()) {
                                desc.quant = quant.to_string();
                            }
                            if let Some(has_vis) = val.get("has_mmproj").and_then(|v| v.as_bool()) {
                                desc.has_mmproj = has_vis;
                                if has_vis {
                                    if let Some(mm_name) = val.get("mmproj_filename").and_then(|v| v.as_str()) {
                                        desc.mmproj_path = Some(parent.join(mm_name));
                                    }
                                }
                            }
                        }
                    }
                } else {
                    // Fallback: Check if matching mmproj sidecar exists in the same directory
                    if let Ok(siblings) = std::fs::read_dir(parent) {
                        for sib in siblings.flatten() {
                            let sib_name = sib.file_name().to_string_lossy().to_lowercase();
                            if sib_name.contains("mmproj") && sib.path().is_file() {
                                desc.has_mmproj = true;
                                desc.mmproj_path = Some(sib.path());
                                break;
                            }
                        }
                    }
                }

                acc.push(desc);
            }
        }
    }
}

