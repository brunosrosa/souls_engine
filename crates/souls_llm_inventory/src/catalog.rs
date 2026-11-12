//! Zero-copy local model catalog scanning and physical asset management.
//!
//! Scans `%USERPROFILE%\llm_local_models`, extracts GGUF headers via zero-copy mmap,
//! evaluates hardware fit against the host diagnostic, and manages model deletion.

use crate::error::InventoryError;
use crate::hf_client::extract_quant_tag;
use crate::profiler::{evaluate_model_fit, HardwareDiagnostic, HardwareFit};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::UNIX_EPOCH;
use tracing::info;

static RE_MOE_A: OnceLock<Regex> = OnceLock::new();
static RE_MOE_X: OnceLock<Regex> = OnceLock::new();
static RE_MILLIONS: OnceLock<Regex> = OnceLock::new();
static RE_BILLIONS: OnceLock<Regex> = OnceLock::new();

fn get_re_moe_a() -> &'static Regex {
    RE_MOE_A.get_or_init(|| {
        Regex::new(r"(?i)(\d+(?:\.\d+)?)\s*b(?:[-_ ](?:instruct|chat))?[-_ ]a(\d+(?:\.\d+)?)\s*b")
            .expect("valid regex for MoE A")
    })
}

fn get_re_moe_x() -> &'static Regex {
    RE_MOE_X.get_or_init(|| {
        Regex::new(r"(?i)(\d+)\s*x\s*(\d+(?:\.\d+)?)\s*b")
            .expect("valid regex for MoE X")
    })
}

fn get_re_millions() -> &'static Regex {
    RE_MILLIONS.get_or_init(|| {
        Regex::new(r"(?i)(?:^|[^0-9a-z])(\d+)\s*m\b")
            .expect("valid regex for Millions")
    })
}

fn get_re_billions() -> &'static Regex {
    RE_BILLIONS.get_or_init(|| {
        Regex::new(r"(?i)(?:^|[^0-9a-z])(\d+(?:\.\d+)?)\s*b\b")
            .expect("valid regex for Billions")
    })
}

/// Parses total parameters, active parameters (if MoE), and MoE flag from a model name.
///
/// If physical `size_bytes` exceeds 8 GB while parsed parameters indicate <= 4B,
/// an adjustment is applied proportionally to physical weights: `(size_bytes as f64 / 0.58) as u64`.
pub fn parse_model_params_from_name(name: &str, size_bytes: Option<u64>) -> (u64, Option<u64>, bool) {
    let apply_sanity_check = |total: u64| -> u64 {
        if let Some(bytes) = size_bytes {
            if bytes > 8_000_000_000 && total <= 4_000_000_000 {
                return (bytes as f64 / 0.58) as u64;
            }
        }
        total
    };

    let name_clean = name.replace('_', "-");

    // 1. MoE patterns:
    if let Some(caps) = get_re_moe_a().captures(&name_clean) {
        if let (Some(total_m), Some(active_m)) = (caps.get(1), caps.get(2)) {
            if let (Ok(total_val), Ok(active_val)) = (
                total_m.as_str().parse::<f64>(),
                active_m.as_str().parse::<f64>(),
            ) {
                let total = (total_val * 1_000_000_000.0) as u64;
                let active = (active_val * 1_000_000_000.0) as u64;
                return (apply_sanity_check(total), Some(active), true);
            }
        }
    }

    if let Some(caps) = get_re_moe_x().captures(&name_clean) {
        if let (Some(exp_m), Some(sz_m)) = (caps.get(1), caps.get(2)) {
            if let (Ok(experts), Ok(sz_val)) = (
                exp_m.as_str().parse::<u64>(),
                sz_m.as_str().parse::<f64>(),
            ) {
                let active = (sz_val * 1_000_000_000.0) as u64;
                let total = experts.saturating_mul(active);
                return (apply_sanity_check(total), Some(active), true);
            }
        }
    }

    // 2. Dense billions:
    let mut b_candidates: Vec<f64> = Vec::new();
    let bytes = name_clean.as_bytes();

    for caps in get_re_billions().captures_iter(&name_clean) {
        if let Some(m) = caps.get(1) {
            let val_str = m.as_str();
            let val_start = m.start();

            // Check if this number is preceded by a hyphenated sub-2B fraction (e.g. "0-5b", "1-5b")
            let mut fraction_val = None;
            if val_start >= 2 && bytes[val_start - 1] == b'-' {
                let lead_char = bytes[val_start - 2];
                if lead_char == b'0' || lead_char == b'1' {
                    // Check if lead_char is preceded by a dot or digit (e.g. "3.1-8b", "1.1-7b", "0.1-7b")
                    let preceded_by_dot_or_digit = if val_start >= 3 {
                        let prev = bytes[val_start - 3];
                        prev == b'.' || prev.is_ascii_digit()
                    } else {
                        false
                    };

                    if !preceded_by_dot_or_digit {
                        if lead_char == b'0' {
                            // Sub-1B model: e.g. 0-5b -> 0.5, 0-7b -> 0.7
                            if let Ok(frac) = val_str.parse::<f64>() {
                                let divisor = 10.0_f64.powi(val_str.len() as i32);
                                fraction_val = Some(frac / divisor);
                            }
                        } else if lead_char == b'1' && val_str == "5" {
                            // Sub-2B model: 1-5b -> 1.5
                            fraction_val = Some(1.5);
                        }
                    }
                }
            }

            if let Some(f_val) = fraction_val {
                b_candidates.push(f_val);
            } else if let Ok(v) = val_str.parse::<f64>() {
                b_candidates.push(v);
            }
        }
    }

    if let Some(&max_b) = b_candidates.iter().max_by(|a, b| a.total_cmp(b)) {
        let total = (max_b * 1_000_000_000.0) as u64;
        return (apply_sanity_check(total), None, false);
    }

    // 3. Millions:
    let mut m_candidates: Vec<u64> = Vec::new();
    for caps in get_re_millions().captures_iter(&name_clean) {
        if let Some(m) = caps.get(1) {
            let s = m.as_str();
            let val = s.parse::<u64>().unwrap_or(u64::MAX);
            m_candidates.push(val);
        }
    }

    if let Some(&max_m) = m_candidates.iter().max() {
        let total = max_m.saturating_mul(1_000_000);
        return (apply_sanity_check(total), None, false);
    }

    // 4. Semantic model aliases (Phi-4-mini / Phi-3-mini) when no explicit parameter count is parsed
    let name_lower = name.to_ascii_lowercase();
    if name_lower.contains("phi-4-mini") || name_lower.contains("phi-3-mini") {
        return (apply_sanity_check(3_800_000_000), None, false);
    }

    // 5. Fallback if no parameter count identified from name
    let fallback_total = if let Some(bytes) = size_bytes {
        if bytes > 0 {
            (bytes as f64 / 0.58) as u64
        } else {
            3_000_000_000
        }
    } else {
        3_000_000_000
    };

    (fallback_total, None, false)
}

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

    let fallback = PathBuf::from(r".\llm_local_models");
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

    let (parsed_params, _active_params, _is_moe) =
        parse_model_params_from_name(&file_name, Some(size_bytes));

    // Try zero-copy mmap header inspection from souls_inference_runtime
    let (architecture, context_length, parameter_count) =
        match souls_inference_runtime::inspect_gguf_metadata_o1(path) {
            Ok(info) => {
                let arch = info.architecture;
                let ctx = info.context_length as u32;
                (arch, ctx, Some(parsed_params))
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
                (arch, ctx, Some(parsed_params))
            }
        };

    let quantization = extract_quant_tag(&file_name);
    let hardware_fit = evaluate_model_fit(diag, size_bytes, context_length, parsed_params);

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

    #[test]
    fn test_parse_model_params_moe() {
        // 30B-A3B pattern
        let (total, active, is_moe) = parse_model_params_from_name("Qwen/Qwen3-30B-A3B-GGUF", None);
        assert_eq!(total, 30_000_000_000);
        assert_eq!(active, Some(3_000_000_000));
        assert!(is_moe);

        let (total, active, is_moe) = parse_model_params_from_name("unsloth/Qwen3-30B-A3B-Instruct-GGUF", None);
        assert_eq!(total, 30_000_000_000);
        assert_eq!(active, Some(3_000_000_000));
        assert!(is_moe);

        // 8x7B pattern
        let (total, active, is_moe) = parse_model_params_from_name("8x7B", None);
        assert_eq!(total, 56_000_000_000);
        assert_eq!(active, Some(7_000_000_000));
        assert!(is_moe);

        let (total, active, is_moe) = parse_model_params_from_name("Mixtral-8x7B-Instruct-v0.1.Q4_K_M.gguf", None);
        assert_eq!(total, 56_000_000_000);
        assert_eq!(active, Some(7_000_000_000));
        assert!(is_moe);
    }

    #[test]
    fn test_parse_model_params_millions() {
        assert_eq!(
            parse_model_params_from_name("smollm2-135m-instruct-q8_0.gguf", None),
            (135_000_000, None, false)
        );
        assert_eq!(
            parse_model_params_from_name("SmolLM-360M", None),
            (360_000_000, None, false)
        );
        assert_eq!(
            parse_model_params_from_name("model-500M.gguf", None),
            (500_000_000, None, false)
        );
    }

    #[test]
    fn test_parse_model_params_dense_billions() {
        // Sub-1B hyphenated and dotted
        assert_eq!(
            parse_model_params_from_name("qwen2-5-coder-0-5b-instruct-q4-k-m.gguf", None),
            (500_000_000, None, false)
        );
        assert_eq!(
            parse_model_params_from_name("qwen2.5-coder-0.5b-instruct.gguf", None),
            (500_000_000, None, false)
        );

        // 1.5B
        assert_eq!(
            parse_model_params_from_name("qwen2-5-coder-1-5b.gguf", None),
            (1_500_000_000, None, false)
        );
        assert_eq!(
            parse_model_params_from_name("qwen2.5-coder-1.5b.gguf", None),
            (1_500_000_000, None, false)
        );

        // 4B with Qwen 3.5 family prefix
        assert_eq!(
            parse_model_params_from_name("qwen3-5-4b-agentic-coder-v4.gguf", None),
            (4_000_000_000, None, false)
        );

        // 7B
        assert_eq!(
            parse_model_params_from_name("qwen2-5-coder-7b-instruct-q4_k_m.gguf", None),
            (7_000_000_000, None, false)
        );
        assert_eq!(
            parse_model_params_from_name("qwen2.5-coder-7b-instruct-q4_k_m.gguf", None),
            (7_000_000_000, None, false)
        );

        // Multiple B tokens: maximum wins
        assert_eq!(
            parse_model_params_from_name("deepseek-coder-33b-instruct-3b-bench.gguf", None),
            (33_000_000_000, None, false)
        );
    }

    #[test]
    fn test_parse_model_params_physical_sanity_check() {
        // 15 GB file mislabeled as 3B -> adjusted to ~25.8B
        let (total_adj, _, _) = parse_model_params_from_name("test-model-3b.gguf", Some(15_000_000_000));
        assert_eq!(total_adj, (15_000_000_000.0 / 0.58) as u64);

        // 2 GB file labeled as 3B -> not adjusted
        let (total_normal, _, _) = parse_model_params_from_name("test-model-3b.gguf", Some(2_000_000_000));
        assert_eq!(total_normal, 3_000_000_000);
    }

    #[test]
    fn test_parse_model_params_regression_subphase_2_1() {
        assert_eq!(
            parse_model_params_from_name("Llama-3.1-8B-Instruct.gguf", None),
            (8_000_000_000, None, false)
        );
        assert_eq!(
            parse_model_params_from_name("Llama-3.1-70B-Instruct.gguf", None),
            (70_000_000_000, None, false)
        );
        assert_eq!(
            parse_model_params_from_name("Llama-3.1-405B-Instruct.gguf", None),
            (405_000_000_000, None, false)
        );
        assert_eq!(
            parse_model_params_from_name("google/gemma-1.1-7b-it.gguf", None),
            (7_000_000_000, None, false)
        );
        assert_eq!(
            parse_model_params_from_name("google/gemma-1.1-2b-it.gguf", None),
            (2_000_000_000, None, false)
        );
        assert_eq!(
            parse_model_params_from_name("mistral-v0.1-7b.gguf", None),
            (7_000_000_000, None, false)
        );
        assert_eq!(
            parse_model_params_from_name("llama-1-7b.gguf", None),
            (7_000_000_000, None, false)
        );
        assert_eq!(
            parse_model_params_from_name("llama-1-13b.gguf", None),
            (13_000_000_000, None, false)
        );
        assert_eq!(
            parse_model_params_from_name("llama-1-65b.gguf", None),
            (65_000_000_000, None, false)
        );
        assert_eq!(
            parse_model_params_from_name("Meta-Llama-3-8B_Q4_K_M.gguf", None),
            (8_000_000_000, None, false)
        );
        assert_eq!(
            parse_model_params_from_name("Llama-3-70B_Q4_K_M.gguf", None),
            (70_000_000_000, None, false)
        );
        assert_eq!(
            parse_model_params_from_name("smollm-135m_q4_k_m.gguf", None),
            (135_000_000, None, false)
        );
        assert_eq!(
            parse_model_params_from_name("qwen2.5-coder-7b_q4_k_m.gguf", None),
            (7_000_000_000, None, false)
        );
        assert_eq!(
            parse_model_params_from_name("microsoft/Phi-4-mini-instruct-GGUF", None),
            (3_800_000_000, None, false)
        );
        assert_eq!(
            parse_model_params_from_name("microsoft/Phi-3-mini-4k-instruct", None),
            (3_800_000_000, None, false)
        );
    }
}
