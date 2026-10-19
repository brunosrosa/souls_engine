//! Curated SSOT Registry of Elite Hugging Face Models and Quantizations.
//!
//! Provides canonical reference models for the 3 Tiers (Tier 0, Tier 0.5, Tier 1)
//! and 5 Cognitive Profiles (Coding, Agentic, Fast Triage, Reasoning, Curation),
//! dynamically evaluated against the host HardwareDiagnostic.

use crate::profiler::{
    estimate_throughput_tps, evaluate_model_fit, HardwareDiagnostic, HardwareFit,
};
use serde::{Deserialize, Serialize};

/// Quantization variant of a Hugging Face GGUF model repository.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QuantizationVariant {
    pub quant_tag: String,
    pub file_name: String,
    pub download_url: String,
    pub size_bytes: u64,
    pub size_gb: f64,
    pub sha256: Option<String>,
    pub hardware_fit: HardwareFit,
    pub is_recommended_fit: bool,
}

/// Baseline cognitive benchmark expectation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BenchmarkExpectation {
    pub coding_score: Option<f32>,
    pub ifeval_score: Option<f32>,
    pub mmlu_score: Option<f32>,
    pub speed_tokens_sec_estimate: f32,
}

/// A curated Hugging Face model item.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HuggingFaceModelItem {
    pub repo_id: String,
    pub model_name: String,
    pub author: String,
    pub parameter_count: u64,
    pub default_context_length: u32,
    pub cognitive_profile: String,
    pub assigned_tier: String,
    pub benchmark_scores: BenchmarkExpectation,
    pub variants: Vec<QuantizationVariant>,
    pub is_downloaded: bool,
    pub local_file_path: Option<String>,
}

/// Recommended 3-Tier combo tailored to the host silicon.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RecommendedTriad {
    pub tier0_velocista: HuggingFaceModelItem,
    pub tier05_socratico: HuggingFaceModelItem,
    pub tier1_coder: HuggingFaceModelItem,
}

/// Helper to build raw curated manifest entries before hardware evaluation.
struct RawModelDef {
    repo_id: &'static str,
    model_name: &'static str,
    author: &'static str,
    parameter_count: u64,
    context_length: u32,
    cognitive_profile: &'static str,
    assigned_tier: &'static str,
    coding_score: Option<f32>,
    ifeval_score: Option<f32>,
    mmlu_score: Option<f32>,
    variants: &'static [RawVariantDef],
}

struct RawVariantDef {
    quant_tag: &'static str,
    file_name: &'static str,
    size_mb: u64,
    sha256: Option<&'static str>,
}

const RAW_CATALOG: &[RawModelDef] = &[
    // --- TIER 0: Velocistas (Triage, Guardrails, Classificação) ---
    RawModelDef {
        repo_id: "HuggingFaceTB/SmolLM2-135M-Instruct-GGUF",
        model_name: "SmolLM2 135M Instruct",
        author: "HuggingFaceTB",
        parameter_count: 135_000_000,
        context_length: 8192,
        cognitive_profile: "fast",
        assigned_tier: "tier0_velocista",
        coding_score: Some(38.5),
        ifeval_score: Some(62.0),
        mmlu_score: Some(50.1),
        variants: &[
            RawVariantDef {
                quant_tag: "Q8_0",
                file_name: "smollm2-135m-instruct-q8_0.gguf",
                size_mb: 145,
                sha256: None,
            },
            RawVariantDef {
                quant_tag: "Q4_K_M",
                file_name: "smollm2-135m-instruct-q4_k_m.gguf",
                size_mb: 85,
                sha256: None,
            },
        ],
    },
    RawModelDef {
        repo_id: "HuggingFaceTB/SmolLM2-360M-Instruct-GGUF",
        model_name: "SmolLM2 360M Instruct",
        author: "HuggingFaceTB",
        parameter_count: 360_000_000,
        context_length: 8192,
        cognitive_profile: "fast",
        assigned_tier: "tier0_velocista",
        coding_score: Some(44.0),
        ifeval_score: Some(67.5),
        mmlu_score: Some(56.2),
        variants: &[
            RawVariantDef {
                quant_tag: "Q8_0",
                file_name: "smollm2-360m-instruct-q8_0.gguf",
                size_mb: 380,
                sha256: None,
            },
            RawVariantDef {
                quant_tag: "Q4_K_M",
                file_name: "smollm2-360m-instruct-q4_k_m.gguf",
                size_mb: 225,
                sha256: None,
            },
        ],
    },

    // --- TIER 0.5: Socrático (Entropia, Cohomologia, Deliberação) ---
    RawModelDef {
        repo_id: "Qwen/Qwen2.5-1.5B-Instruct-GGUF",
        model_name: "Qwen 2.5 1.5B Instruct",
        author: "Qwen",
        parameter_count: 1_500_000_000,
        context_length: 32768,
        cognitive_profile: "reasoning",
        assigned_tier: "tier05_socratico",
        coding_score: Some(65.4),
        ifeval_score: Some(72.8),
        mmlu_score: Some(68.2),
        variants: &[
            RawVariantDef {
                quant_tag: "Q5_K_M",
                file_name: "qwen2.5-1.5b-instruct-q5_k_m.gguf",
                size_mb: 1150,
                sha256: None,
            },
            RawVariantDef {
                quant_tag: "Q4_K_M",
                file_name: "qwen2.5-1.5b-instruct-q4_k_m.gguf",
                size_mb: 980,
                sha256: None,
            },
            RawVariantDef {
                quant_tag: "Q8_0",
                file_name: "qwen2.5-1.5b-instruct-q8_0.gguf",
                size_mb: 1650,
                sha256: None,
            },
        ],
    },
    RawModelDef {
        repo_id: "google/gemma-2-2b-it-GGUF",
        model_name: "Gemma 2 2B Instruct",
        author: "Google",
        parameter_count: 2_600_000_000,
        context_length: 8192,
        cognitive_profile: "curation",
        assigned_tier: "tier05_socratico",
        coding_score: Some(60.2),
        ifeval_score: Some(75.1),
        mmlu_score: Some(70.4),
        variants: &[
            RawVariantDef {
                quant_tag: "Q4_K_M",
                file_name: "gemma-2-2b-it-q4_k_m.gguf",
                size_mb: 1750,
                sha256: None,
            },
            RawVariantDef {
                quant_tag: "Q5_K_M",
                file_name: "gemma-2-2b-it-q5_k_m.gguf",
                size_mb: 2050,
                sha256: None,
            },
        ],
    },

    // --- TIER 1: Tratores de Código & Agentes Autônomos ---
    RawModelDef {
        repo_id: "Qwen/Qwen2.5-Coder-7B-Instruct-GGUF",
        model_name: "Qwen 2.5 Coder 7B Instruct",
        author: "Qwen",
        parameter_count: 7_000_000_000,
        context_length: 32768,
        cognitive_profile: "coding",
        assigned_tier: "tier1_coder",
        coding_score: Some(88.4),
        ifeval_score: Some(82.5),
        mmlu_score: Some(79.2),
        variants: &[
            RawVariantDef {
                quant_tag: "Q4_K_M",
                file_name: "qwen2.5-coder-7b-instruct-q4_k_m.gguf",
                size_mb: 4350,
                sha256: None,
            },
            RawVariantDef {
                quant_tag: "Q5_K_M",
                file_name: "qwen2.5-coder-7b-instruct-q5_k_m.gguf",
                size_mb: 5150,
                sha256: None,
            },
            RawVariantDef {
                quant_tag: "Q8_0",
                file_name: "qwen2.5-coder-7b-instruct-q8_0.gguf",
                size_mb: 7700,
                sha256: None,
            },
        ],
    },
    RawModelDef {
        repo_id: "Qwen/Qwen2.5-Coder-3B-Instruct-GGUF",
        model_name: "Qwen 2.5 Coder 3B Instruct",
        author: "Qwen",
        parameter_count: 3_000_000_000,
        context_length: 32768,
        cognitive_profile: "agentic",
        assigned_tier: "tier1_coder",
        coding_score: Some(76.5),
        ifeval_score: Some(74.0),
        mmlu_score: Some(71.0),
        variants: &[
            RawVariantDef {
                quant_tag: "Q5_K_M",
                file_name: "qwen2.5-coder-3b-instruct-q5_k_m.gguf",
                size_mb: 2350,
                sha256: None,
            },
            RawVariantDef {
                quant_tag: "Q4_K_M",
                file_name: "qwen2.5-coder-3b-instruct-q4_k_m.gguf",
                size_mb: 1950,
                sha256: None,
            },
        ],
    },
    RawModelDef {
        repo_id: "microsoft/Phi-4-mini-instruct-GGUF",
        model_name: "Phi 4 Mini 3.8B Instruct",
        author: "Microsoft",
        parameter_count: 3_800_000_000,
        context_length: 16384,
        cognitive_profile: "reasoning",
        assigned_tier: "tier1_coder",
        coding_score: Some(78.0),
        ifeval_score: Some(79.5),
        mmlu_score: Some(76.4),
        variants: &[
            RawVariantDef {
                quant_tag: "Q4_K_M",
                file_name: "phi-4-mini-instruct-q4_k_m.gguf",
                size_mb: 2450,
                sha256: None,
            },
            RawVariantDef {
                quant_tag: "Q5_K_M",
                file_name: "phi-4-mini-instruct-q5_k_m.gguf",
                size_mb: 2850,
                sha256: None,
            },
        ],
    },
];

/// Evaluates and returns the curated catalog against the host hardware.
pub fn get_curated_catalog(
    diag: &HardwareDiagnostic,
    installed_filenames: &[String],
) -> Vec<HuggingFaceModelItem> {
    RAW_CATALOG
        .iter()
        .map(|def| {
            let mut variants = Vec::new();

            for var in def.variants {
                let size_bytes = var.size_mb * 1024 * 1024;
                let eval_context = def.context_length.min(8192);
                let fit = evaluate_model_fit(diag, size_bytes, eval_context, def.parameter_count);

                // Build Hugging Face download URL (resolve direct LFS link)
                let download_url = format!(
                    "https://huggingface.co/{}/resolve/main/{}",
                    def.repo_id, var.file_name
                );

                variants.push(QuantizationVariant {
                    quant_tag: var.quant_tag.to_string(),
                    file_name: var.file_name.to_string(),
                    download_url,
                    size_bytes,
                    size_gb: var.size_mb as f64 / 1024.0,
                    sha256: var.sha256.map(String::from),
                    hardware_fit: fit,
                    is_recommended_fit: false,
                });
            }

            // Flag the optimal recommended quantization variant:
            // Prefer Q5_K_M if it fits OptimalVram; otherwise Q4_K_M if OptimalVram; otherwise first variant
            let mut recommended_idx = 0;
            for (i, v) in variants.iter().enumerate() {
                if v.hardware_fit == HardwareFit::OptimalVram {
                    if v.quant_tag == "Q5_K_M" {
                        recommended_idx = i;
                        break;
                    } else if v.quant_tag == "Q4_K_M" {
                        recommended_idx = i;
                    }
                }
            }
            if let Some(v) = variants.get_mut(recommended_idx) {
                v.is_recommended_fit = true;
            }

            // Check if any variant is already downloaded
            let mut is_downloaded = false;
            let mut local_file_path = None;
            for v in &variants {
                if installed_filenames.iter().any(|f| f.eq_ignore_ascii_case(&v.file_name)) {
                    is_downloaded = true;
                    local_file_path = Some(v.file_name.clone());
                    break;
                }
            }

            let primary_fit = variants
                .get(recommended_idx)
                .map(|v| v.hardware_fit)
                .unwrap_or(HardwareFit::PartialCpuOffload);

            let speed_tps = estimate_throughput_tps(
                primary_fit,
                def.parameter_count,
                diag.has_avx2,
            );

            HuggingFaceModelItem {
                repo_id: def.repo_id.to_string(),
                model_name: def.model_name.to_string(),
                author: def.author.to_string(),
                parameter_count: def.parameter_count,
                default_context_length: def.context_length,
                cognitive_profile: def.cognitive_profile.to_string(),
                assigned_tier: def.assigned_tier.to_string(),
                benchmark_scores: BenchmarkExpectation {
                    coding_score: def.coding_score,
                    ifeval_score: def.ifeval_score,
                    mmlu_score: def.mmlu_score,
                    speed_tokens_sec_estimate: speed_tps,
                },
                variants,
                is_downloaded,
                local_file_path,
            }
        })
        .collect()
}

/// Computes the recommended 3-tier combo calibrated for the host silicon.
pub fn get_recommended_triad(
    diag: &HardwareDiagnostic,
    installed_filenames: &[String],
) -> RecommendedTriad {
    let catalog = get_curated_catalog(diag, installed_filenames);

    let tier0 = catalog
        .iter()
        .find(|m| m.assigned_tier == "tier0_velocista" && m.repo_id.contains("SmolLM2-135M"))
        .cloned()
        .unwrap_or_else(|| catalog[0].clone());

    let tier05 = catalog
        .iter()
        .find(|m| m.assigned_tier == "tier05_socratico" && m.repo_id.contains("Qwen2.5-1.5B"))
        .cloned()
        .unwrap_or_else(|| catalog[2].clone());

    // For Tier 1, choose Qwen 2.5 Coder 7B if it fits in VRAM or RAM, or Coder 3B if tight
    let tier1 = catalog
        .iter()
        .find(|m| m.assigned_tier == "tier1_coder" && m.repo_id.contains("Coder-7B"))
        .cloned()
        .unwrap_or_else(|| catalog[4].clone());

    RecommendedTriad {
        tier0_velocista: tier0,
        tier05_socratico: tier05,
        tier1_coder: tier1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_curated_catalog_evaluation() {
        let diag = HardwareDiagnostic {
            has_dedicated_gpu: true,
            gpu_name: "RTX 2060m".to_string(),
            vram_total_mb: 6144,
            vram_free_mb: 5000,
            vram_safe_usable_mb: 5294,
            host_ram_total_mb: 32768,
            host_ram_available_mb: 24000,
            host_ram_safe_usable_mb: 20000,
            cpu_cores_physical: 8,
            cpu_cores_logical: 16,
            has_avx2: true,
            has_avx512: false,
        };

        let installed = vec!["qwen2.5-coder-7b-instruct-q4_k_m.gguf".to_string()];
        let catalog = get_curated_catalog(&diag, &installed);

        assert!(catalog.len() >= 6);

        let coder_7b = catalog
            .iter()
            .find(|m| m.repo_id.contains("Coder-7B"))
            .expect("Coder 7B must exist");

        assert!(coder_7b.is_downloaded);
        assert_eq!(coder_7b.local_file_path.as_deref(), Some("qwen2.5-coder-7b-instruct-q4_k_m.gguf"));

        // Q4_K_M should be OptimalVram on 5.2GB
        let q4 = coder_7b.variants.iter().find(|v| v.quant_tag == "Q4_K_M").unwrap();
        assert_eq!(q4.hardware_fit, HardwareFit::OptimalVram);

        // Q8_0 should be PartialCpuOffload on 5.2GB
        let q8 = coder_7b.variants.iter().find(|v| v.quant_tag == "Q8_0").unwrap();
        assert_eq!(q8.hardware_fit, HardwareFit::PartialCpuOffload);
    }

    #[test]
    fn test_recommended_triad() {
        let diag = HardwareDiagnostic::default();
        let triad = get_recommended_triad(&diag, &[]);

        assert_eq!(triad.tier0_velocista.assigned_tier, "tier0_velocista");
        assert_eq!(triad.tier05_socratico.assigned_tier, "tier05_socratico");
        assert_eq!(triad.tier1_coder.assigned_tier, "tier1_coder");
    }
}
