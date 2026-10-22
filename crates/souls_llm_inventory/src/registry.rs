//! Curated SSOT Registry of Elite Hugging Face Models and Quantizations.
//!
//! Provides canonical reference models for the 3 Tiers (Tier 0, Tier 0.5, Tier 1)
//! and 6 Cognitive Profiles (Coding, Reasoning, Agentic, Design, Vision, Fast),
//! dynamically evaluated against the host HardwareDiagnostic with dual SweetSpot/Heavyweight roles.

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
    pub projector_file: Option<String>,
    pub projector_url: Option<String>,
    pub projector_size_mb: Option<u64>,
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
    pub category_role: String, // "SweetSpot" (Eficiência 3B-4B, 45+ tps) ou "Heavyweight" (Capacidade máxima 7B-8B)
    pub why_recommended: String,
    pub highlights: Vec<String>,
    pub benchmark_scores: BenchmarkExpectation,
    pub variants: Vec<QuantizationVariant>,
    pub is_downloaded: bool,
    pub local_file_path: Option<String>,
    pub is_multimodal_vlm: bool,
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
    category_role: &'static str,
    why_recommended: &'static str,
    highlights: &'static [&'static str],
    is_multimodal_vlm: bool,
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
    projector_file: Option<&'static str>,
    projector_size_mb: Option<u64>,
}

const RAW_CATALOG: &[RawModelDef] = &[
    // =========================================================================
    // 1. PERFIL: CODING SPECIALIST (Engenharia & Cirurgia de Código)
    // =========================================================================
    RawModelDef {
        repo_id: "Qwen/Qwen2.5-Coder-3B-Instruct-GGUF",
        model_name: "Qwen 2.5 Coder 3B Instruct",
        author: "Qwen",
        parameter_count: 3_000_000_000,
        context_length: 32768,
        cognitive_profile: "coding",
        assigned_tier: "tier1_coder",
        category_role: "SweetSpot",
        why_recommended: "Campeão de densidade e velocidade (48+ tok/s). Deixa 2.8 GB livres na VRAM para contextos de até 32k tokens sem spillover.",
        highlights: &["48+ tok/s dGPU", "32k Contexto Seguro", "HumanEval 76.5%", "Zero Risco Spillover"],
        is_multimodal_vlm: false,
        coding_score: Some(76.5),
        ifeval_score: Some(74.0),
        mmlu_score: Some(71.0),
        variants: &[
            RawVariantDef {
                quant_tag: "Q5_K_M",
                file_name: "qwen2.5-coder-3b-instruct-q5_k_m.gguf",
                size_mb: 2350,
                sha256: None,
                projector_file: None,
                projector_size_mb: None,
            },
            RawVariantDef {
                quant_tag: "Q4_K_M",
                file_name: "qwen2.5-coder-3b-instruct-q4_k_m.gguf",
                size_mb: 1950,
                sha256: None,
                projector_file: None,
                projector_size_mb: None,
            },
        ],
    },
    RawModelDef {
        repo_id: "Qwen/Qwen2.5-Coder-7B-Instruct-GGUF",
        model_name: "Qwen 2.5 Coder 7B Instruct",
        author: "Qwen",
        parameter_count: 7_000_000_000,
        context_length: 32768,
        cognitive_profile: "coding",
        assigned_tier: "tier1_coder",
        category_role: "Heavyweight",
        why_recommended: "Teto máximo de capacidade local para refatorações pesadas e arquitetura (HumanEval 88.4%). Opera no limite da VRAM (4.3 GB pesos + KV).",
        highlights: &["HumanEval 88.4%", "Lógica SWE Avançada", "28 tok/s dGPU", "VRAM Justa (~4.9GB)"],
        is_multimodal_vlm: false,
        coding_score: Some(88.4),
        ifeval_score: Some(82.5),
        mmlu_score: Some(79.2),
        variants: &[
            RawVariantDef {
                quant_tag: "Q4_K_M",
                file_name: "qwen2.5-coder-7b-instruct-q4_k_m.gguf",
                size_mb: 4350,
                sha256: None,
                projector_file: None,
                projector_size_mb: None,
            },
            RawVariantDef {
                quant_tag: "Q5_K_M",
                file_name: "qwen2.5-coder-7b-instruct-q5_k_m.gguf",
                size_mb: 5150,
                sha256: None,
                projector_file: None,
                projector_size_mb: None,
            },
        ],
    },

    // =========================================================================
    // 2. PERFIL: DEEP REASONING (Raciocínio Socrático & Chain-of-Thought)
    // =========================================================================
    RawModelDef {
        repo_id: "deepseek-ai/DeepSeek-R1-Distill-Qwen-1.5B-GGUF",
        model_name: "DeepSeek R1 Distill Qwen 1.5B",
        author: "DeepSeek",
        parameter_count: 1_500_000_000,
        context_length: 32768,
        cognitive_profile: "reasoning",
        assigned_tier: "tier05_socratico",
        category_role: "SweetSpot",
        why_recommended: "Vanguarda mundial de reasoning formal destilado do DeepSeek-R1. Gera cadeias de pensamento socrático profundas a mais de 65 tok/s.",
        highlights: &["R1 Chain-of-Thought", "65+ tok/s dGPU", "1.1 GB VRAM", "Deliberação Socrática"],
        is_multimodal_vlm: false,
        coding_score: Some(72.0),
        ifeval_score: Some(78.5),
        mmlu_score: Some(74.2),
        variants: &[
            RawVariantDef {
                quant_tag: "Q5_K_M",
                file_name: "deepseek-r1-distill-qwen-1.5b-q5_k_m.gguf",
                size_mb: 1180,
                sha256: None,
                projector_file: None,
                projector_size_mb: None,
            },
            RawVariantDef {
                quant_tag: "Q4_K_M",
                file_name: "deepseek-r1-distill-qwen-1.5b-q4_k_m.gguf",
                size_mb: 990,
                sha256: None,
                projector_file: None,
                projector_size_mb: None,
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
        category_role: "Heavyweight",
        why_recommended: "Matemática rigorosa e resolução de problemas analíticos com arquitetura Phi-4. Pontua 76.4 no MMLU ocupando apenas 2.5 GB.",
        highlights: &["MMLU 76.4", "Matemática Formal", "2.5 GB VRAM", "38 tok/s dGPU"],
        is_multimodal_vlm: false,
        coding_score: Some(78.0),
        ifeval_score: Some(79.5),
        mmlu_score: Some(76.4),
        variants: &[
            RawVariantDef {
                quant_tag: "Q4_K_M",
                file_name: "phi-4-mini-instruct-q4_k_m.gguf",
                size_mb: 2450,
                sha256: None,
                projector_file: None,
                projector_size_mb: None,
            },
            RawVariantDef {
                quant_tag: "Q5_K_M",
                file_name: "phi-4-mini-instruct-q5_k_m.gguf",
                size_mb: 2850,
                sha256: None,
                projector_file: None,
                projector_size_mb: None,
            },
        ],
    },

    // =========================================================================
    // 3. PERFIL: AGENTIC & TOOL CALLING (Orquestração & Function Calling)
    // =========================================================================
    RawModelDef {
        repo_id: "NousResearch/Hermes-3-Llama-3.2-3B-GGUF",
        model_name: "Hermes 3 Llama 3.2 3B",
        author: "NousResearch",
        parameter_count: 3_200_000_000,
        context_length: 32768,
        cognitive_profile: "agentic",
        assigned_tier: "tier1_coder",
        category_role: "SweetSpot",
        why_recommended: "Orquestrador oficial do Hermes Agent. Treinado com os melhores prompts agênticos, auto-reflexão e despacho de subagentes.",
        highlights: &["Simbiose Nativa Hermes", "Excelente Tool Calling", "42 tok/s dGPU", "2.0 GB VRAM"],
        is_multimodal_vlm: false,
        coding_score: Some(72.5),
        ifeval_score: Some(81.0),
        mmlu_score: Some(72.8),
        variants: &[
            RawVariantDef {
                quant_tag: "Q4_K_M",
                file_name: "Hermes-3-Llama-3.2-3B.Q4_K_M.gguf",
                size_mb: 1980,
                sha256: None,
                projector_file: None,
                projector_size_mb: None,
            },
            RawVariantDef {
                quant_tag: "Q5_K_M",
                file_name: "Hermes-3-Llama-3.2-3B.Q5_K_M.gguf",
                size_mb: 2350,
                sha256: None,
                projector_file: None,
                projector_size_mb: None,
            },
        ],
    },
    RawModelDef {
        repo_id: "katanemo/Arch-Function-Chat-3B-GGUF",
        model_name: "Arch Function Chat 3B",
        author: "Katanemo",
        parameter_count: 3_000_000_000,
        context_length: 8192,
        cognitive_profile: "agentic",
        assigned_tier: "tier1_coder",
        category_role: "Heavyweight",
        why_recommended: "Modelo com fine-tuning hiper-especializado em validação de JSON Schema e invocação estrita de chamadas MCP sem desvios sintáticos.",
        highlights: &["JSON Schema Rígido", "Zero Erro de Chamada", "45 tok/s dGPU", "2.2 GB VRAM"],
        is_multimodal_vlm: false,
        coding_score: Some(74.0),
        ifeval_score: Some(84.0),
        mmlu_score: Some(70.5),
        variants: &[
            RawVariantDef {
                quant_tag: "Q5_K_M",
                file_name: "Arch-Function-Chat-3B-Q5_K_M.gguf",
                size_mb: 2200,
                sha256: None,
                projector_file: None,
                projector_size_mb: None,
            },
            RawVariantDef {
                quant_tag: "Q4_K_M",
                file_name: "Arch-Function-Chat-3B-Q4_K_M.gguf",
                size_mb: 1850,
                sha256: None,
                projector_file: None,
                projector_size_mb: None,
            },
        ],
    },

    // =========================================================================
    // 4. PERFIL: UI/UX & FRONTEND DESIGN (CSS, Componentes & Estética Web)
    // =========================================================================
    RawModelDef {
        repo_id: "Qwen/Qwen2.5-Coder-3B-Instruct-GGUF",
        model_name: "Qwen 2.5 Coder 3B (Design Edition)",
        author: "Qwen",
        parameter_count: 3_000_000_000,
        context_length: 32768,
        cognitive_profile: "design",
        assigned_tier: "tier1_coder",
        category_role: "SweetSpot",
        why_recommended: "Calibrado para geração de layouts elegantes, Vanilla CSS moderno, glassmorphism, hierarquia tipográfica e micro-animações interativas.",
        highlights: &["Design Systems", "CSS3 / Vanilla / Flex / Grid", "Micro-animações", "Zero Placeholders"],
        is_multimodal_vlm: false,
        coding_score: Some(78.5),
        ifeval_score: Some(76.0),
        mmlu_score: Some(71.5),
        variants: &[
            RawVariantDef {
                quant_tag: "Q5_K_M",
                file_name: "qwen2.5-coder-3b-instruct-q5_k_m.gguf",
                size_mb: 2350,
                sha256: None,
                projector_file: None,
                projector_size_mb: None,
            },
            RawVariantDef {
                quant_tag: "Q4_K_M",
                file_name: "qwen2.5-coder-3b-instruct-q4_k_m.gguf",
                size_mb: 1950,
                sha256: None,
                projector_file: None,
                projector_size_mb: None,
            },
        ],
    },
    RawModelDef {
        repo_id: "Qwen/Qwen2.5-Coder-7B-Instruct-GGUF",
        model_name: "Qwen 2.5 Coder 7B (Design Heavyweight)",
        author: "Qwen",
        parameter_count: 7_000_000_000,
        context_length: 32768,
        cognitive_profile: "design",
        assigned_tier: "tier1_coder",
        category_role: "Heavyweight",
        why_recommended: "Capacidade máxima de engenharia frontend completa (páginas inteiras, dashboards analíticos complexos e sistemas de design de larga escala).",
        highlights: &["Dashboards Complexos", "SVG / Canvas / UI", "Engenharia Frontend", "Capacidade Máxima"],
        is_multimodal_vlm: false,
        coding_score: Some(89.0),
        ifeval_score: Some(83.0),
        mmlu_score: Some(79.5),
        variants: &[
            RawVariantDef {
                quant_tag: "Q4_K_M",
                file_name: "qwen2.5-coder-7b-instruct-q4_k_m.gguf",
                size_mb: 4350,
                sha256: None,
                projector_file: None,
                projector_size_mb: None,
            },
        ],
    },

    // =========================================================================
    // 5. PERFIL: VISION & MULTIMODAL VLM (Inspeção Visual, OCR & Diagramas)
    // =========================================================================
    RawModelDef {
        repo_id: "Qwen/Qwen2.5-VL-3B-Instruct-GGUF",
        model_name: "Qwen 2.5 VL 3B Instruct (Bundle VLM)",
        author: "Qwen",
        parameter_count: 3_200_000_000,
        context_length: 16384,
        cognitive_profile: "vision",
        assigned_tier: "tier3_multimodal",
        category_role: "SweetSpot",
        why_recommended: "VLM de vanguarda com suporte a leitura de screenshots, diagramas Mermaid/UML e OCR de código. Requer projetor mmproj-F16 baixado em conjunto.",
        highlights: &["Inspeção Visual UI", "OCR de Código & Prints", "Diagramas Mermaid", "GGUF + mmproj F16"],
        is_multimodal_vlm: true,
        coding_score: Some(71.0),
        ifeval_score: Some(75.5),
        mmlu_score: Some(69.0),
        variants: &[
            RawVariantDef {
                quant_tag: "Q4_K_M",
                file_name: "Qwen2.5-VL-3B-Instruct-Q4_K_M.gguf",
                size_mb: 1880,
                sha256: None,
                projector_file: Some("mmproj-F16.gguf"),
                projector_size_mb: Some(1280),
            },
        ],
    },

    // =========================================================================
    // 6. PERFIL: FAST REFLEX & TRIAGE (Classificação Rápida, Guardrails & Entropia)
    // =========================================================================
    RawModelDef {
        repo_id: "HuggingFaceTB/SmolLM2-135M-Instruct-GGUF",
        model_name: "SmolLM2 135M Instruct",
        author: "HuggingFaceTB",
        parameter_count: 135_000_000,
        context_length: 8192,
        cognitive_profile: "fast",
        assigned_tier: "tier0_velocista",
        category_role: "SweetSpot",
        why_recommended: "Velocista ultra-leve para reflexos instantâneos (180+ tok/s). Ocupa apenas 85 MB de VRAM, ideal para pré-filtros e guardrails.",
        highlights: &["180+ tok/s dGPU", "Apenas 85 MB VRAM", "Reflexo Instantâneo", "Zero Latência"],
        is_multimodal_vlm: false,
        coding_score: Some(38.5),
        ifeval_score: Some(62.0),
        mmlu_score: Some(50.1),
        variants: &[
            RawVariantDef {
                quant_tag: "Q4_K_M",
                file_name: "smollm2-135m-instruct-q4_k_m.gguf",
                size_mb: 85,
                sha256: None,
                projector_file: None,
                projector_size_mb: None,
            },
            RawVariantDef {
                quant_tag: "Q8_0",
                file_name: "smollm2-135m-instruct-q8_0.gguf",
                size_mb: 145,
                sha256: None,
                projector_file: None,
                projector_size_mb: None,
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
        category_role: "Heavyweight",
        why_recommended: "Classificação robusta de intenção e verificação de coerência epistêmica a 120 tok/s com menos de 250 MB de peso.",
        highlights: &["120 tok/s dGPU", "225 MB VRAM", "Classificação de Intenção", "Ideal para Tier 0.5"],
        is_multimodal_vlm: false,
        coding_score: Some(44.0),
        ifeval_score: Some(67.5),
        mmlu_score: Some(56.2),
        variants: &[
            RawVariantDef {
                quant_tag: "Q4_K_M",
                file_name: "smollm2-360m-instruct-q4_k_m.gguf",
                size_mb: 225,
                sha256: None,
                projector_file: None,
                projector_size_mb: None,
            },
            RawVariantDef {
                quant_tag: "Q8_0",
                file_name: "smollm2-360m-instruct-q8_0.gguf",
                size_mb: 380,
                sha256: None,
                projector_file: None,
                projector_size_mb: None,
            },
        ],
    },
];

/// Normaliza um nome de arquivo para casamento flexível (sem extensão e case-insensitive)
fn normalize_stem(filename: &str) -> String {
    filename
        .to_ascii_lowercase()
        .trim_end_matches(".gguf")
        .trim_end_matches(".bin")
        .replace(['_', '-', '.'], "")
}

/// Evaluates and returns the curated catalog against the host hardware.
pub fn get_curated_catalog(
    diag: &HardwareDiagnostic,
    installed_filenames: &[String],
) -> Vec<HuggingFaceModelItem> {
    let normalized_installed: Vec<(String, String)> = installed_filenames
        .iter()
        .map(|f| (f.clone(), normalize_stem(f)))
        .collect();

    RAW_CATALOG
        .iter()
        .map(|def| {
            let mut variants = Vec::new();

            for var in def.variants {
                let total_size_mb = var.size_mb + var.projector_size_mb.unwrap_or(0);
                let size_bytes = total_size_mb * 1024 * 1024;
                let eval_context = def.context_length.min(8192);
                let fit = evaluate_model_fit(diag, size_bytes, eval_context, def.parameter_count);

                let download_url = format!(
                    "https://huggingface.co/{}/resolve/main/{}",
                    def.repo_id, var.file_name
                );

                let (proj_file, proj_url) = if let Some(pfile) = var.projector_file {
                    let purl = format!(
                        "https://huggingface.co/{}/resolve/main/{}",
                        def.repo_id, pfile
                    );
                    (Some(pfile.to_string()), Some(purl))
                } else {
                    (None, None)
                };

                variants.push(QuantizationVariant {
                    quant_tag: var.quant_tag.to_string(),
                    file_name: var.file_name.to_string(),
                    download_url,
                    size_bytes: var.size_mb * 1024 * 1024,
                    size_gb: var.size_mb as f64 / 1024.0,
                    sha256: var.sha256.map(String::from),
                    hardware_fit: fit,
                    is_recommended_fit: false,
                    projector_file: proj_file,
                    projector_url: proj_url,
                    projector_size_mb: var.projector_size_mb,
                });
            }

            // Flag optimal recommended variant (Q5_K_M if OptimalVram; else Q4_K_M; else first)
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

            // Mapeamento resiliente de arquivo já instalado
            let mut is_downloaded = false;
            let mut local_file_path = None;

            for v in &variants {
                let v_norm = normalize_stem(&v.file_name);
                for (real_name, real_norm) in &normalized_installed {
                    if real_name.eq_ignore_ascii_case(&v.file_name)
                        || real_norm == &v_norm
                        || real_norm.contains(&v_norm)
                        || v_norm.contains(real_norm)
                    {
                        is_downloaded = true;
                        local_file_path = Some(real_name.clone());
                        break;
                    }
                }
                if is_downloaded {
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
                category_role: def.category_role.to_string(),
                why_recommended: def.why_recommended.to_string(),
                highlights: def.highlights.iter().map(|s| s.to_string()).collect(),
                benchmark_scores: BenchmarkExpectation {
                    coding_score: def.coding_score,
                    ifeval_score: def.ifeval_score,
                    mmlu_score: def.mmlu_score,
                    speed_tokens_sec_estimate: speed_tps,
                },
                variants,
                is_downloaded,
                local_file_path,
                is_multimodal_vlm: def.is_multimodal_vlm,
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
        .find(|m| m.assigned_tier == "tier05_socratico" && m.repo_id.contains("DeepSeek-R1"))
        .cloned()
        .or_else(|| {
            catalog
                .iter()
                .find(|m| m.assigned_tier == "tier05_socratico")
                .cloned()
        })
        .unwrap_or_else(|| catalog[2].clone());

    let tier1 = catalog
        .iter()
        .find(|m| m.assigned_tier == "tier1_coder" && m.category_role == "SweetSpot" && m.cognitive_profile == "coding")
        .cloned()
        .or_else(|| {
            catalog
                .iter()
                .find(|m| m.assigned_tier == "tier1_coder" && m.repo_id.contains("Coder-7B"))
                .cloned()
        })
        .unwrap_or_else(|| catalog[1].clone());

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
        assert_eq!(
            coder_7b.local_file_path.as_deref(),
            Some("qwen2.5-coder-7b-instruct-q4_k_m.gguf")
        );

        // Q4_K_M should be OptimalVram on 5.2GB
        let q4 = coder_7b.variants.iter().find(|v| v.quant_tag == "Q4_K_M").unwrap();
        assert_eq!(q4.hardware_fit, HardwareFit::OptimalVram);
    }

    #[test]
    fn test_curated_catalog_vlm_projector_support() {
        let diag = HardwareDiagnostic::default();
        let catalog = get_curated_catalog(&diag, &[]);

        let vlm = catalog
            .iter()
            .find(|m| m.is_multimodal_vlm)
            .expect("VLM model must be in curated catalog");

        assert_eq!(vlm.cognitive_profile, "vision");
        let v = &vlm.variants[0];
        assert_eq!(v.projector_file.as_deref(), Some("mmproj-F16.gguf"));
        assert!(v.projector_url.is_some());
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
