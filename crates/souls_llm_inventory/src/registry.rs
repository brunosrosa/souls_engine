//! Curated SSOT Registry of Elite Hugging Face Models and Quantizations.
//!
//! Provides canonical reference models for the 3 Tiers (Tier 0, Tier 0.5, Tier 1)
//! and 6 Cognitive Profiles (Coding, Reasoning, Agentic, Design, Vision, Fast),
//! dynamically evaluated against the host HardwareDiagnostic with dual SweetSpot/Heavyweight roles.

use crate::catalog::{parse_model_params_from_name, LocalModelMeta};
use crate::hf_client::{extract_quant_tag, HfModelFile, HfSearchResult};
use crate::profiler::{
    estimate_throughput_tps, estimate_variant_throughput_tps, evaluate_model_fit,
    HardwareDiagnostic, HardwareFit,
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
    #[serde(default)]
    pub is_installed: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub installed_file_name: Option<String>,
    #[serde(default)]
    pub estimated_tps: f32,
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

/// A curated or dynamically discovered Hugging Face model item.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HuggingFaceModelItem {
    pub repo_id: String,
    pub model_name: String,
    pub author: String,
    pub parameter_count: u64,
    #[serde(default)]
    pub active_parameter_count: Option<u64>,
    #[serde(default)]
    pub is_moe: bool,
    pub default_context_length: u32,
    pub cognitive_profile: String,
    pub assigned_tier: String,
    pub category_role: String, // "Featherweight" (<2B), "SweetSpot" (2B-4.5B), ou "Heavyweight" (>=6.5B)
    pub why_recommended: String,
    pub highlights: Vec<String>,
    pub benchmark_scores: BenchmarkExpectation,
    pub variants: Vec<QuantizationVariant>,
    #[serde(default)]
    pub is_downloaded: bool,
    #[serde(default)]
    pub is_installed: bool,
    pub local_file_path: Option<String>,
    pub is_multimodal_vlm: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub upgrade_recommendation: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub upgrade_candidate: Option<Box<HuggingFaceModelItem>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub downloads: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub likes: Option<u64>,
    #[serde(default)]
    pub is_trending: bool,
}

/// Recommended 3-Tier combo tailored to the host silicon.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RecommendedTriad {
    pub tier0_velocista: HuggingFaceModelItem,
    pub tier05_socratico: HuggingFaceModelItem,
    pub tier1_coder: HuggingFaceModelItem,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub upgrade_recommendations: Vec<HuggingFaceModelItem>,
}

/// Helper to build raw curated manifest entries before hardware evaluation.
struct RawModelDef {
    repo_id: &'static str,
    model_name: &'static str,
    author: &'static str,
    parameter_count: u64,
    active_parameter_count: Option<u64>,
    is_moe: bool,
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
        repo_id: "Qwen/Qwen3.5-4B-Agentic-Coder-GGUF",
        model_name: "Qwen 3.5 4B Agentic Coder v4",
        author: "Qwen",
        parameter_count: 4_000_000_000,
        active_parameter_count: None,
        is_moe: false,
        context_length: 32768,
        cognitive_profile: "coding",
        assigned_tier: "tier1_coder",
        category_role: "SweetSpot",
        why_recommended: "Geração Qwen 3.5 com Native MTP (Multi-Token Prediction) e fine-tuning agentic para cirurgia de código e refatoração de alta precisão.",
        highlights: &["Geração Qwen 3.5", "Native MTP Integrado", "42+ tok/s dGPU", "2.6 GB VRAM"],
        is_multimodal_vlm: false,
        coding_score: Some(82.0),
        ifeval_score: Some(80.5),
        mmlu_score: Some(76.8),
        variants: &[
            RawVariantDef {
                quant_tag: "Q4_K_M",
                file_name: "qwen3-5-4b-agentic-coder-v4.gguf",
                size_mb: 2650,
                sha256: None,
                projector_file: None,
                projector_size_mb: None,
            },
            RawVariantDef {
                quant_tag: "Q5_K_M",
                file_name: "qwen3-5-4b-agentic-coder-v4-q5_k_m.gguf",
                size_mb: 3100,
                sha256: None,
                projector_file: None,
                projector_size_mb: None,
            },
        ],
    },
    RawModelDef {
        repo_id: "Qwen/Qwen3-30B-A3B-GGUF",
        model_name: "Qwen 3 30B A3B Instruct (MoE)",
        author: "Qwen",
        parameter_count: 30_000_000_000,
        active_parameter_count: Some(3_000_000_000),
        is_moe: true,
        context_length: 32768,
        cognitive_profile: "coding",
        assigned_tier: "tier1_coder",
        category_role: "Heavyweight",
        why_recommended: "Arquitetura MoE revolucionária: 30B total com apenas 3B parâmetros ativos por token. Velocidade de inferência de 3B com capacidade de raciocínio de 30B.",
        highlights: &["MoE 30B (3B Ativos)", "Velocidade de 3B", "HumanEval 90.2%", "Alta Eficiência"],
        is_multimodal_vlm: false,
        coding_score: Some(90.2),
        ifeval_score: Some(86.5),
        mmlu_score: Some(82.4),
        variants: &[
            RawVariantDef {
                quant_tag: "Q4_K_M",
                file_name: "qwen3-30b-a3b-instruct-q4_k_m.gguf",
                size_mb: 18500,
                sha256: None,
                projector_file: None,
                projector_size_mb: None,
            },
            RawVariantDef {
                quant_tag: "Q3_K_M",
                file_name: "qwen3-30b-a3b-instruct-q3_k_m.gguf",
                size_mb: 14200,
                sha256: None,
                projector_file: None,
                projector_size_mb: None,
            },
        ],
    },
    RawModelDef {
        repo_id: "Qwen/Qwen3-8B-GGUF",
        model_name: "Qwen 3 8B Instruct",
        author: "Qwen",
        parameter_count: 8_000_000_000,
        active_parameter_count: None,
        is_moe: false,
        context_length: 40960,
        cognitive_profile: "coding",
        assigned_tier: "tier1_coder",
        category_role: "Heavyweight",
        why_recommended: "Modelo denso Qwen 3 de 8B com raciocínio híbrido (/think) e alta precisão em engenharia de software e refatoração estrutural no teto da VRAM.",
        highlights: &["Geração Qwen 3", "Hybrid Thinking", "~22 tok/s dGPU", "4.8 GB VRAM (Q4)"],
        is_multimodal_vlm: false,
        coding_score: Some(89.6),
        ifeval_score: Some(85.8),
        mmlu_score: Some(81.5),
        variants: &[
            RawVariantDef {
                quant_tag: "Q4_K_M",
                file_name: "Qwen3-8B-Q4_K_M.gguf",
                size_mb: 4920,
                sha256: None,
                projector_file: None,
                projector_size_mb: None,
            },
            RawVariantDef {
                quant_tag: "Q5_K_M",
                file_name: "Qwen3-8B-Q5_K_M.gguf",
                size_mb: 5850,
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
        active_parameter_count: None,
        is_moe: false,
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
    RawModelDef {
        repo_id: "Qwen/Qwen2.5-Coder-3B-Instruct-GGUF",
        model_name: "Qwen 2.5 Coder 3B Instruct",
        author: "Qwen",
        parameter_count: 3_000_000_000,
        active_parameter_count: None,
        is_moe: false,
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

    // =========================================================================
    // 2. PERFIL: DEEP REASONING (Raciocínio Socrático & Chain-of-Thought)
    // =========================================================================
    RawModelDef {
        repo_id: "Qwen/Qwen3-4B-GGUF",
        model_name: "Qwen 3 4B Instruct (Hybrid Thinking)",
        author: "Qwen",
        parameter_count: 4_000_000_000,
        active_parameter_count: None,
        is_moe: false,
        context_length: 40960,
        cognitive_profile: "reasoning",
        assigned_tier: "tier05_socratico",
        category_role: "SweetSpot",
        why_recommended: "Geração Qwen 3 com alternância dinâmica entre resposta instantânea e cadeia de pensamento (/think), entregando precisão analítica superior em ~2.5 GB.",
        highlights: &["Qwen 3 Dual-Mode", "Chain-of-Thought", "~37 tok/s dGPU", "2.5 GB VRAM"],
        is_multimodal_vlm: false,
        coding_score: Some(81.5),
        ifeval_score: Some(83.2),
        mmlu_score: Some(77.8),
        variants: &[
            RawVariantDef {
                quant_tag: "Q4_K_M",
                file_name: "Qwen3-4B-Q4_K_M.gguf",
                size_mb: 2500,
                sha256: None,
                projector_file: None,
                projector_size_mb: None,
            },
            RawVariantDef {
                quant_tag: "Q5_K_M",
                file_name: "Qwen3-4B-Q5_K_M.gguf",
                size_mb: 2890,
                sha256: None,
                projector_file: None,
                projector_size_mb: None,
            },
        ],
    },
    RawModelDef {
        repo_id: "deepseek-ai/DeepSeek-R1-Distill-Qwen-1.5B-GGUF",
        model_name: "DeepSeek R1 Distill Qwen 1.5B",
        author: "DeepSeek",
        parameter_count: 1_500_000_000,
        active_parameter_count: None,
        is_moe: false,
        context_length: 32768,
        cognitive_profile: "reasoning",
        assigned_tier: "tier05_socratico",
        category_role: "Featherweight",
        why_recommended: "Vanguarda mundial de reasoning formal destilado do DeepSeek-R1. Gera cadeias de pensamento socrático profundas a mais de 65 tok/s.",
        highlights: &["R1 Chain-of-Thought", "~78 tok/s dGPU", "1.1 GB VRAM", "Deliberação Socrática"],
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
        active_parameter_count: None,
        is_moe: false,
        context_length: 131072,
        cognitive_profile: "reasoning",
        assigned_tier: "tier05_socratico",
        category_role: "SweetSpot",
        why_recommended: "Matemática rigorosa e resolução de problemas analíticos com arquitetura Phi-4 e janela nativa de 128k. Pontua 76.4 no MMLU ocupando apenas 2.5 GB.",
        highlights: &["MMLU 76.4", "Matemática Formal", "2.5 GB VRAM", "~37 tok/s dGPU"],
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
    RawModelDef {
        repo_id: "meta-llama/Llama-3.3-70B-Instruct-GGUF",
        model_name: "Llama 3.3 70B Instruct",
        author: "Meta",
        parameter_count: 70_550_000_000,
        active_parameter_count: None,
        is_moe: false,
        context_length: 131072,
        cognitive_profile: "reasoning",
        assigned_tier: "tier1_coder",
        category_role: "Heavyweight",
        why_recommended: "Topo de linha open weights mundial. Desempenho equivalente a modelos de fronteira em benchmarks analíticos, ideal para offload em CPU com 32GB+ RAM ou GPUs combinadas.",
        highlights: &["Nível Frontier LLM", "128k Contexto", "MMLU 88.6%", "Offload RAM/VRAM"],
        is_multimodal_vlm: false,
        coding_score: Some(89.5),
        ifeval_score: Some(91.2),
        mmlu_score: Some(88.6),
        variants: &[
            RawVariantDef {
                quant_tag: "Q4_K_M",
                file_name: "Llama-3.3-70B-Instruct-Q4_K_M.gguf",
                size_mb: 43500,
                sha256: None,
                projector_file: None,
                projector_size_mb: None,
            },
            RawVariantDef {
                quant_tag: "Q3_K_M",
                file_name: "Llama-3.3-70B-Instruct-Q3_K_M.gguf",
                size_mb: 34500,
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
        active_parameter_count: None,
        is_moe: false,
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
        repo_id: "meta-llama/Llama-3.2-3B-Instruct-GGUF",
        model_name: "Llama 3.2 3B Instruct",
        author: "Meta",
        parameter_count: 3_210_000_000,
        active_parameter_count: None,
        is_moe: false,
        context_length: 131072,
        cognitive_profile: "agentic",
        assigned_tier: "tier1_coder",
        category_role: "SweetSpot",
        why_recommended: "Modelo oficial da Meta com janela nativa de 128k tokens, excelente densidade analítica e alta aderência a tool calling.",
        highlights: &["128k Contexto Nativo", "45 tok/s dGPU", "2.0 GB VRAM", "Meta Llama 3.2"],
        is_multimodal_vlm: false,
        coding_score: Some(70.5),
        ifeval_score: Some(80.0),
        mmlu_score: Some(72.4),
        variants: &[
            RawVariantDef {
                quant_tag: "Q4_K_M",
                file_name: "Llama-3.2-3B-Instruct-Q4_K_M.gguf",
                size_mb: 2020,
                sha256: None,
                projector_file: None,
                projector_size_mb: None,
            },
            RawVariantDef {
                quant_tag: "Q5_K_M",
                file_name: "Llama-3.2-3B-Instruct-Q5_K_M.gguf",
                size_mb: 2320,
                sha256: None,
                projector_file: None,
                projector_size_mb: None,
            },
        ],
    },
    RawModelDef {
        repo_id: "Salesforce/xLAM-7b-fc-GGUF",
        model_name: "Salesforce xLAM-7B Function Calling",
        author: "Salesforce",
        parameter_count: 7_000_000_000,
        active_parameter_count: None,
        is_moe: false,
        context_length: 16384,
        cognitive_profile: "agentic",
        assigned_tier: "tier1_coder",
        category_role: "Heavyweight",
        why_recommended: "Líder mundial em Function Calling no Berkeley Function Calling Leaderboard (BFCL). Especializado em agentes autônomos e execução de chamadas MCP sem desvios sintáticos.",
        highlights: &["Líder BFCL Leaderboard", "JSON Schema Rigoroso", "Zero Alucinação MCP", "Ações Autônomas"],
        is_multimodal_vlm: false,
        coding_score: Some(78.0),
        ifeval_score: Some(89.5),
        mmlu_score: Some(77.2),
        variants: &[
            RawVariantDef {
                quant_tag: "Q4_K_M",
                file_name: "xlam-7b-fc-r-q4_k_m.gguf",
                size_mb: 4350,
                sha256: None,
                projector_file: None,
                projector_size_mb: None,
            },
            RawVariantDef {
                quant_tag: "Q5_K_M",
                file_name: "xlam-7b-fc-r-q5_k_m.gguf",
                size_mb: 5150,
                sha256: None,
                projector_file: None,
                projector_size_mb: None,
            },
        ],
    },
    RawModelDef {
        repo_id: "nvidia/Nemotron-3-Nano-4B-GGUF",
        model_name: "NVIDIA Nemotron 3 Nano 4B",
        author: "NVIDIA",
        parameter_count: 4_000_000_000,
        active_parameter_count: None,
        is_moe: false,
        context_length: 16384,
        cognitive_profile: "agentic",
        assigned_tier: "tier05_socratico",
        category_role: "SweetSpot",
        why_recommended: "Desenvolvido pela NVIDIA para inferência de alta precisão com otimização máxima para Tensor Cores. Especializado em RAG agêntico, guardrails e raciocínio socrático.",
        highlights: &["NVIDIA On-Device AI", "Otimizado para Tensor Cores", "Agentic RAG & Guardrails", "2.5 GB VRAM"],
        is_multimodal_vlm: false,
        coding_score: Some(73.5),
        ifeval_score: Some(82.0),
        mmlu_score: Some(75.8),
        variants: &[
            RawVariantDef {
                quant_tag: "Q4_K_M",
                file_name: "NVIDIA-Nemotron-3-Nano-4B-Q4_K_M.gguf",
                size_mb: 2480,
                sha256: None,
                projector_file: None,
                projector_size_mb: None,
            },
            RawVariantDef {
                quant_tag: "Q5_K_M",
                file_name: "NVIDIA-Nemotron-3-Nano-4B-Q5_K_M.gguf",
                size_mb: 2900,
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
        repo_id: "google/gemma-3-4b-it-GGUF",
        model_name: "Gemma 3 4B IT (Design & UI Edition)",
        author: "Google",
        parameter_count: 4_000_000_000,
        active_parameter_count: None,
        is_moe: false,
        context_length: 131072,
        cognitive_profile: "design",
        assigned_tier: "tier1_coder",
        category_role: "SweetSpot",
        why_recommended: "Modelo de 4B da Google de última geração calibrado para geração de layouts responsivos, Vanilla CSS moderno, hierarquia tipográfica e micro-animações interativas.",
        highlights: &["Google Gemma 3", "Design Systems Web", "128k Contexto", "45 tok/s dGPU"],
        is_multimodal_vlm: false,
        coding_score: Some(76.0),
        ifeval_score: Some(81.5),
        mmlu_score: Some(74.5),
        variants: &[
            RawVariantDef {
                quant_tag: "Q4_K_M",
                file_name: "gemma-3-4b-it-q4_k_m.gguf",
                size_mb: 2600,
                sha256: None,
                projector_file: None,
                projector_size_mb: None,
            },
            RawVariantDef {
                quant_tag: "Q5_K_M",
                file_name: "gemma-3-4b-it-q5_k_m.gguf",
                size_mb: 3050,
                sha256: None,
                projector_file: None,
                projector_size_mb: None,
            },
        ],
    },
    RawModelDef {
        repo_id: "google/gemma-3-12b-it-GGUF",
        model_name: "Gemma 3 12B IT (Design Heavyweight)",
        author: "Google",
        parameter_count: 12_000_000_000,
        active_parameter_count: None,
        is_moe: false,
        context_length: 131072,
        cognitive_profile: "design",
        assigned_tier: "tier1_coder",
        category_role: "Heavyweight",
        why_recommended: "Capacidade máxima de engenharia frontend completa (páginas inteiras, dashboards analíticos complexos e sistemas de design corporativo).",
        highlights: &["Google Gemma 3 12B", "Dashboards Complexos", "128k Contexto", "Alta Precisão"],
        is_multimodal_vlm: false,
        coding_score: Some(81.0),
        ifeval_score: Some(85.0),
        mmlu_score: Some(80.2),
        variants: &[
            RawVariantDef {
                quant_tag: "Q4_K_M",
                file_name: "gemma-3-12b-it-q4_k_m.gguf",
                size_mb: 7500,
                sha256: None,
                projector_file: None,
                projector_size_mb: None,
            },
        ],
    },
    RawModelDef {
        repo_id: "google/gemma-3-27b-it-GGUF",
        model_name: "Gemma 3 27B IT (Frontier Open)",
        author: "Google",
        parameter_count: 27_000_000_000,
        active_parameter_count: None,
        is_moe: false,
        context_length: 131072,
        cognitive_profile: "design",
        assigned_tier: "tier1_coder",
        category_role: "Heavyweight",
        why_recommended: "Pico de capacidade aberta da Google em modelo denso. Excelente para refatorações monumentais e arquitetura de frontends massivos.",
        highlights: &["Google Gemma 3 27B", "Pico de Capacidade", "128k Contexto", "Offload RAM/VRAM"],
        is_multimodal_vlm: false,
        coding_score: Some(85.5),
        ifeval_score: Some(88.2),
        mmlu_score: Some(84.8),
        variants: &[
            RawVariantDef {
                quant_tag: "Q4_K_M",
                file_name: "gemma-3-27b-it-q4_k_m.gguf",
                size_mb: 16800,
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
        active_parameter_count: None,
        is_moe: false,
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
        repo_id: "Qwen/Qwen3-1.7B-GGUF",
        model_name: "Qwen 3 1.7B Instruct",
        author: "Qwen",
        parameter_count: 1_700_000_000,
        active_parameter_count: None,
        is_moe: false,
        context_length: 32768,
        cognitive_profile: "fast",
        assigned_tier: "tier0_velocista",
        category_role: "Featherweight",
        why_recommended: "Ultra-compacto de nova geração Qwen 3 (1.7B). Combina velocidade acima de 70 tok/s na dGPU com inteligência superior para triagem, roteamento e guardrails.",
        highlights: &["Geração Qwen 3", "~72 tok/s dGPU", "1.1 GB VRAM", "Triagem de Elite"],
        is_multimodal_vlm: false,
        coding_score: Some(64.5),
        ifeval_score: Some(77.0),
        mmlu_score: Some(68.2),
        variants: &[
            RawVariantDef {
                quant_tag: "Q4_K_M",
                file_name: "Qwen3-1.7B-Q4_K_M.gguf",
                size_mb: 1120,
                sha256: None,
                projector_file: None,
                projector_size_mb: None,
            },
            RawVariantDef {
                quant_tag: "Q8_0",
                file_name: "Qwen3-1.7B-Q8_0.gguf",
                size_mb: 1820,
                sha256: None,
                projector_file: None,
                projector_size_mb: None,
            },
        ],
    },
    RawModelDef {
        repo_id: "HuggingFaceTB/SmolLM2-135M-Instruct-GGUF",
        model_name: "SmolLM2 135M Instruct",
        author: "HuggingFaceTB",
        parameter_count: 135_000_000,
        active_parameter_count: None,
        is_moe: false,
        context_length: 8192,
        cognitive_profile: "fast",
        assigned_tier: "tier0_velocista",
        category_role: "Featherweight",
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
        active_parameter_count: None,
        is_moe: false,
        context_length: 8192,
        cognitive_profile: "fast",
        assigned_tier: "tier0_velocista",
        category_role: "Featherweight",
        why_recommended: "Classificação robusta de intenção e verificação de coerência epistêmica com menos de 250 MB de peso.",
        highlights: &["~180 tok/s dGPU", "225 MB VRAM", "Classificação de Intenção", "Ideal para Tier 0"],
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
    RawModelDef {
        repo_id: "HuggingFaceTB/SmolLM2-1.7B-Instruct-GGUF",
        model_name: "SmolLM2 1.7B Instruct",
        author: "HuggingFaceTB",
        parameter_count: 1_700_000_000,
        active_parameter_count: None,
        is_moe: false,
        context_length: 8192,
        cognitive_profile: "fast",
        assigned_tier: "tier05_socratico",
        category_role: "Featherweight",
        why_recommended: "Modelo de 1.7B de vanguarda ultra-leve da HuggingFace. Excelente compromisso entre velocidade (~75 tok/s) e capacidade de deliberação para o Tier 0.5 ocupando cerca de 1.1 GB de VRAM.",
        highlights: &["~75 tok/s dGPU", "1.1 GB VRAM", "Tier 0.5 Eficiente", "HuggingFace SmolLM2"],
        is_multimodal_vlm: false,
        coding_score: Some(58.0),
        ifeval_score: Some(74.0),
        mmlu_score: Some(65.5),
        variants: &[
            RawVariantDef {
                quant_tag: "Q4_K_M",
                file_name: "smollm2-1.7b-instruct-q4_k_m.gguf",
                size_mb: 1050,
                sha256: None,
                projector_file: None,
                projector_size_mb: None,
            },
            RawVariantDef {
                quant_tag: "Q5_K_M",
                file_name: "smollm2-1.7b-instruct-q5_k_m.gguf",
                size_mb: 1250,
                sha256: None,
                projector_file: None,
                projector_size_mb: None,
            },
        ],
    },
    RawModelDef {
        repo_id: "meta-llama/Llama-3.2-1B-Instruct-GGUF",
        model_name: "Llama 3.2 1B Instruct",
        author: "Meta",
        parameter_count: 1_230_000_000,
        active_parameter_count: None,
        is_moe: false,
        context_length: 131072,
        cognitive_profile: "fast",
        assigned_tier: "tier0_velocista",
        category_role: "Featherweight",
        why_recommended: "Velocista oficial da Meta com janela nativa de 128k tokens para classificação rápida e guardrails ocupando apenas 780 MB.",
        highlights: &["128k Contexto Nativo", "~92 tok/s dGPU", "Apenas 780 MB VRAM", "Meta Llama 3.2"],
        is_multimodal_vlm: false,
        coding_score: Some(54.0),
        ifeval_score: Some(72.0),
        mmlu_score: Some(63.4),
        variants: &[
            RawVariantDef {
                quant_tag: "Q4_K_M",
                file_name: "Llama-3.2-1B-Instruct-Q4_K_M.gguf",
                size_mb: 780,
                sha256: None,
                projector_file: None,
                projector_size_mb: None,
            },
            RawVariantDef {
                quant_tag: "Q8_0",
                file_name: "Llama-3.2-1B-Instruct-Q8_0.gguf",
                size_mb: 1320,
                sha256: None,
                projector_file: None,
                projector_size_mb: None,
            },
        ],
    },
    RawModelDef {
        repo_id: "google/gemma-3-1b-it-GGUF",
        model_name: "Gemma 3 1B IT",
        author: "Google",
        parameter_count: 1_000_000_000,
        active_parameter_count: None,
        is_moe: false,
        context_length: 32768,
        cognitive_profile: "fast",
        assigned_tier: "tier0_velocista",
        category_role: "Featherweight",
        why_recommended: "Modelo ultra-compacto da Google com arquitetura Gemma 3. Rápido (~90 tok/s), ideal para triagem instantânea e baixo consumo de memória.",
        highlights: &["Google Gemma 3", "~90 tok/s dGPU", "Apenas 800 MB", "Baixo Consumo"],
        is_multimodal_vlm: false,
        coding_score: Some(52.0),
        ifeval_score: Some(73.5),
        mmlu_score: Some(62.8),
        variants: &[
            RawVariantDef {
                quant_tag: "Q4_K_M",
                file_name: "gemma-3-1b-it-q4_k_m.gguf",
                size_mb: 800,
                sha256: None,
                projector_file: None,
                projector_size_mb: None,
            },
            RawVariantDef {
                quant_tag: "Q8_0",
                file_name: "gemma-3-1b-it-q8_0.gguf",
                size_mb: 1150,
                sha256: None,
                projector_file: None,
                projector_size_mb: None,
            },
        ],
    },
];

/// Normaliza um nome de arquivo para casamento flexível (sem extensão e case-insensitive).
fn normalize_stem(filename: &str) -> String {
    filename
        .to_ascii_lowercase()
        .trim_end_matches(".gguf")
        .trim_end_matches(".bin")
        .replace(['_', '-', '.'], "")
}

/// Strip quantization/imatrix suffixes so model identity can be compared independently of quant tag.
fn strip_quant_from_stem(filename: &str) -> String {
    let mut s = filename
        .to_ascii_lowercase()
        .trim_end_matches(".gguf")
        .trim_end_matches(".bin")
        .replace(['-', '.'], "_");
    for marker in &[
        "_iq4_xs", "_iq4_nl", "_q4_k_l", "_q4_k_m", "_q4_k_s", "_q4_0", "_q4_1",
        "_q5_k_l", "_q5_k_m", "_q5_k_s", "_q5_0", "_q5_1",
        "_q6_k_l", "_q6_k",
        "_q8_0", "_q8_1",
        "_iq3_m", "_iq3_xs", "_q3_k_l", "_q3_k_m", "_q3_k_s",
        "_q2_k", "_fp16", "_bf16", "_f16",
        "_imatrix", "_i1",
    ] {
        if let Some(idx) = s.find(marker) {
            s = format!("{}{}", &s[..idx], &s[idx + marker.len()..]);
        }
    }
    s.replace('_', "")
}

/// Checks if two model base stems and parameter counts refer to the same underlying model family+size.
fn base_stem_and_params_match(candidate_name: &str, installed_name: &str) -> bool {
    let c_base = strip_quant_from_stem(candidate_name);
    let i_base = strip_quant_from_stem(installed_name);
    if c_base.len() < 5 || i_base.len() < 5 {
        return false;
    }
    let stem_matches = i_base == c_base
        || (c_base.len() >= 6 && i_base.contains(&c_base))
        || (i_base.len() >= 6 && c_base.contains(&i_base));
    if !stem_matches {
        return false;
    }

    let (c_params, _, _) = parse_model_params_from_name(candidate_name, None);
    let (i_params, _, _) = parse_model_params_from_name(installed_name, None);
    if c_params > 0 && i_params > 0 {
        let ratio = (c_params as f64) / (i_params as f64);
        if !(0.8..=1.25).contains(&ratio) {
            return false;
        }
    }
    true
}

/// Strict quantization-aware match: checks exact filename/stem or stem + matching quant tag.
fn variant_matches_installed(
    variant_file: &str,
    variant_quant: &str,
    installed_real_name: &str,
    installed_norm: &str,
    installed_quant: &str,
) -> bool {
    if installed_real_name.eq_ignore_ascii_case(variant_file) {
        return true;
    }
    let v_norm = normalize_stem(variant_file);
    if installed_norm == v_norm {
        return true;
    }
    if !base_stem_and_params_match(variant_file, installed_real_name) {
        return false;
    }
    installed_quant.eq_ignore_ascii_case(variant_quant)
        || (installed_quant == "UNKNOWN" && variant_quant.eq_ignore_ascii_case("Q4_K_M"))
}

/// Picks the optimal default index when inspecting a set of quantization variants:
/// Priority 1: Already-installed variant on disk.
/// Priority 2: OptimalVram `Q5_K_M` when <= 3.2 GB, else `Q4_K_M`, else first OptimalVram, else 0.
fn pick_recommended_variant_index(variants: &[QuantizationVariant]) -> usize {
    if let Some((idx, _)) = variants.iter().enumerate().find(|(_, v)| v.is_installed) {
        return idx;
    }
    let mut fallback_opt: Option<usize> = None;
    for (i, v) in variants.iter().enumerate() {
        if v.hardware_fit == HardwareFit::OptimalVram {
            if fallback_opt.is_none() {
                fallback_opt = Some(i);
            }
            let total_mb = (v.size_bytes / (1024 * 1024)) + v.projector_size_mb.unwrap_or(0);
            if v.quant_tag == "Q5_K_M" && total_mb <= 3200 {
                return i;
            } else if v.quant_tag == "Q4_K_M" {
                fallback_opt = Some(i);
            }
        }
    }
    fallback_opt.unwrap_or(0)
}

/// Builds coherent highlights so any `tok/s` tag matches the computed variant speed.
fn sync_highlights_with_speed(
    raw_highlights: &[String],
    speed_tps: f32,
    fit: HardwareFit,
) -> Vec<String> {
    let silicon = match fit {
        HardwareFit::OptimalVram => "dGPU",
        HardwareFit::PartialCpuOffload => "CPU Offload",
        HardwareFit::IncompatibleSpillover => "Spillover",
    };
    raw_highlights
        .iter()
        .map(|h| {
            let low = h.to_ascii_lowercase();
            if low.contains("tok/s") || low.contains("tps") {
                format!("~{} tok/s {}", speed_tps.round() as u32, silicon)
            } else {
                h.clone()
            }
        })
        .collect()
}

/// Evaluates and returns the curated catalog against host hardware using rich local metadata.
pub fn get_curated_catalog_from_models(
    diag: &HardwareDiagnostic,
    installed_models: &[LocalModelMeta],
) -> Vec<HuggingFaceModelItem> {
    struct InstalledEntry {
        real_name: String,
        norm: String,
        quant_tag: String,
        size_bytes: u64,
    }

    let installed_entries: Vec<InstalledEntry> = installed_models
        .iter()
        .map(|m| {
            let q = if !m.quantization.is_empty() && m.quantization != "UNKNOWN" {
                m.quantization.to_ascii_uppercase()
            } else {
                extract_quant_tag(&m.file_name)
            };
            InstalledEntry {
                real_name: m.file_name.clone(),
                norm: normalize_stem(&m.file_name),
                quant_tag: q,
                size_bytes: m.size_bytes,
            }
        })
        .collect();

    RAW_CATALOG
        .iter()
        .map(|def| {
            let eval_params = def.active_parameter_count.unwrap_or(def.parameter_count);
            let eval_context = def.context_length.min(8192);
            let mut variants = Vec::new();

            for var in def.variants {
                let total_size_mb = var.size_mb + var.projector_size_mb.unwrap_or(0);
                let fit_size_bytes = total_size_mb * 1024 * 1024;
                let file_size_bytes = var.size_mb * 1024 * 1024;
                let fit = evaluate_model_fit(diag, fit_size_bytes, eval_context, def.parameter_count);
                let est_tps = estimate_variant_throughput_tps(diag, fit, file_size_bytes, eval_params);

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

                let matched_installed = installed_entries.iter().find(|ins| {
                    variant_matches_installed(
                        var.file_name,
                        var.quant_tag,
                        &ins.real_name,
                        &ins.norm,
                        &ins.quant_tag,
                    )
                });

                variants.push(QuantizationVariant {
                    quant_tag: var.quant_tag.to_string(),
                    file_name: var.file_name.to_string(),
                    download_url,
                    size_bytes: file_size_bytes,
                    size_gb: var.size_mb as f64 / 1024.0,
                    sha256: var.sha256.map(String::from),
                    hardware_fit: fit,
                    is_recommended_fit: false,
                    is_installed: matched_installed.is_some(),
                    installed_file_name: matched_installed.map(|ins| ins.real_name.clone()),
                    estimated_tps: est_tps,
                    projector_file: proj_file,
                    projector_url: proj_url,
                    projector_size_mb: var.projector_size_mb,
                });
            }

            // Detect disk files belonging to the same model stem but quantized differently (e.g. i1-Q4_K_S)
            if let Some(ref_var) = variants.first().cloned() {
                for ins in &installed_entries {
                    let already_tracked = variants
                        .iter()
                        .any(|v| v.installed_file_name.as_deref() == Some(ins.real_name.as_str()));
                    if !already_tracked && base_stem_and_params_match(&ref_var.file_name, &ins.real_name) {
                        let actual_bytes = if ins.size_bytes > 0 {
                            ins.size_bytes
                        } else {
                            ref_var.size_bytes
                        };
                        let fit = evaluate_model_fit(diag, actual_bytes, eval_context, def.parameter_count);
                        let est_tps = estimate_variant_throughput_tps(diag, fit, actual_bytes, eval_params);
                        let qtag = if ins.quant_tag == "UNKNOWN" {
                            "LOCAL".to_string()
                        } else {
                            ins.quant_tag.clone()
                        };
                        variants.push(QuantizationVariant {
                            quant_tag: qtag,
                            file_name: ins.real_name.clone(),
                            download_url: String::new(),
                            size_bytes: actual_bytes,
                            size_gb: actual_bytes as f64 / (1024.0 * 1024.0 * 1024.0),
                            sha256: None,
                            hardware_fit: fit,
                            is_recommended_fit: false,
                            is_installed: true,
                            installed_file_name: Some(ins.real_name.clone()),
                            estimated_tps: est_tps,
                            projector_file: ref_var.projector_file.clone(),
                            projector_url: ref_var.projector_url.clone(),
                            projector_size_mb: ref_var.projector_size_mb,
                        });
                    }
                }
            }

            let recommended_idx = pick_recommended_variant_index(&variants);
            if let Some(v) = variants.get_mut(recommended_idx) {
                v.is_recommended_fit = true;
            }

            let installed_variant = variants.iter().find(|v| v.is_installed);
            let is_downloaded = installed_variant.is_some();
            let local_file_path = variants
                .get(recommended_idx)
                .and_then(|v| v.installed_file_name.clone())
                .or_else(|| installed_variant.and_then(|v| v.installed_file_name.clone()));

            let primary_fit = variants
                .get(recommended_idx)
                .map(|v| v.hardware_fit)
                .unwrap_or(HardwareFit::PartialCpuOffload);

            let speed_tps = variants
                .get(recommended_idx)
                .map(|v| v.estimated_tps)
                .unwrap_or_else(|| estimate_throughput_tps(primary_fit, eval_params, diag.has_avx2));

            let raw_highlights: Vec<String> = def.highlights.iter().map(|s| s.to_string()).collect();
            let highlights = sync_highlights_with_speed(&raw_highlights, speed_tps, primary_fit);

            HuggingFaceModelItem {
                repo_id: def.repo_id.to_string(),
                model_name: def.model_name.to_string(),
                author: def.author.to_string(),
                parameter_count: def.parameter_count,
                active_parameter_count: def.active_parameter_count,
                is_moe: def.is_moe,
                default_context_length: def.context_length,
                cognitive_profile: def.cognitive_profile.to_string(),
                assigned_tier: def.assigned_tier.to_string(),
                category_role: def.category_role.to_string(),
                why_recommended: def.why_recommended.to_string(),
                highlights,
                benchmark_scores: BenchmarkExpectation {
                    coding_score: def.coding_score,
                    ifeval_score: def.ifeval_score,
                    mmlu_score: def.mmlu_score,
                    speed_tokens_sec_estimate: speed_tps,
                },
                variants,
                is_downloaded,
                is_installed: is_downloaded,
                local_file_path,
                is_multimodal_vlm: def.is_multimodal_vlm,
                upgrade_recommendation: None,
                upgrade_candidate: None,
                downloads: None,
                likes: None,
                is_trending: false,
            }
        })
        .collect()
}

/// Evaluates and returns the curated catalog against the host hardware.
pub fn get_curated_catalog(
    diag: &HardwareDiagnostic,
    installed_filenames: &[String],
) -> Vec<HuggingFaceModelItem> {
    let installed_models: Vec<LocalModelMeta> = installed_filenames
        .iter()
        .map(|name| {
            let (params, _, _) = parse_model_params_from_name(name, None);
            let fit = evaluate_model_fit(diag, 0, 8192, params);
            LocalModelMeta {
                file_name: name.clone(),
                absolute_path: std::path::PathBuf::from(name),
                format: crate::catalog::ModelFormat::Gguf,
                size_bytes: 0,
                size_gb: 0.0,
                architecture: "unknown".to_string(),
                parameter_count: Some(params),
                quantization: extract_quant_tag(name),
                context_length_max: 8192,
                hardware_fit: fit,
                assigned_tier: None,
                last_modified_unix: 0,
            }
        })
        .collect();

    get_curated_catalog_from_models(diag, &installed_models)
}

/// Selects the strongest non-installed OptimalVram alternative/upgrade candidate for a tier.
fn select_evolutionary_alternative(
    tier_item: &HuggingFaceModelItem,
    target_tier: &str,
    catalog: &[HuggingFaceModelItem],
    diag: &HardwareDiagnostic,
) -> Option<HuggingFaceModelItem> {
    let score_candidate = |c: &HuggingFaceModelItem| -> f32 {
        let coding = c.benchmark_scores.coding_score.unwrap_or(52.0);
        let mmlu = c.benchmark_scores.mmlu_score.unwrap_or(54.0);
        let ifeval = c.benchmark_scores.ifeval_score.unwrap_or(62.0);
        let mut base = coding * 0.45 + mmlu * 0.35 + ifeval * 0.20;

        if c.repo_id.contains("Qwen3") {
            base += 6.0;
        } else if c.repo_id.contains("Phi-4") || c.repo_id.contains("DeepSeek-R1") || c.repo_id.contains("gemma-3") {
            base += 4.5;
        }
        if target_tier == "tier1_coder" && diag.vram_safe_usable_mb >= 4800 && c.category_role == "Heavyweight" {
            base += 8.0;
        }
        if c.parameter_count > tier_item.parameter_count {
            base += 3.0;
        }
        base
    };

    let mut eligible: Vec<&HuggingFaceModelItem> = catalog
        .iter()
        .filter(|c| {
            c.assigned_tier == target_tier
                && c.repo_id != tier_item.repo_id
                && !c.is_downloaded
                && c.variants.iter().any(|v| v.hardware_fit == HardwareFit::OptimalVram)
        })
        .collect();

    if eligible.is_empty() {
        eligible = catalog
            .iter()
            .filter(|c| {
                c.repo_id != tier_item.repo_id
                    && !c.is_downloaded
                    && !c.is_multimodal_vlm
                    && c.variants.iter().any(|v| v.hardware_fit == HardwareFit::OptimalVram)
            })
            .collect();
    }

    eligible.sort_by(|a, b| score_candidate(b).total_cmp(&score_candidate(a)));
    eligible.first().map(|&item| {
        let mut clean = item.clone();
        clean.upgrade_candidate = None;
        clean.upgrade_recommendation = None;
        clean
    })
}

/// Computes the recommended 3-tier combo calibrated for the host silicon,
/// pairing installed champions with evolutionary OptimalVram alternatives/upgrades.
pub fn get_recommended_triad_from_models(
    diag: &HardwareDiagnostic,
    installed_models: &[LocalModelMeta],
) -> RecommendedTriad {
    let mut catalog = get_curated_catalog_from_models(diag, installed_models);

    for item in &mut catalog {
        if item.is_downloaded {
            item.is_installed = true;
        }
    }

    let select_tier = |target_tier: &str, priority_stems: &[&str]| -> HuggingFaceModelItem {
        for stem in priority_stems {
            if let Some(m) = installed_models
                .iter()
                .find(|im| im.file_name.to_ascii_lowercase().contains(stem))
            {
                if let Some(mut cat_item) = catalog
                    .iter()
                    .find(|ci| {
                        ci.assigned_tier == target_tier
                            && (ci.local_file_path.as_deref() == Some(&m.file_name)
                                || ci.repo_id.to_ascii_lowercase().contains(stem))
                            && ci.variants.iter().any(|v| v.hardware_fit == HardwareFit::OptimalVram)
                    })
                    .cloned()
                {
                    cat_item.is_downloaded = true;
                    cat_item.is_installed = true;
                    cat_item.local_file_path = Some(m.file_name.clone());
                    return cat_item;
                }
            }
        }

        for stem in priority_stems {
            if let Some(cat_item) = catalog.iter().find(|ci| {
                ci.assigned_tier == target_tier
                    && ci.is_downloaded
                    && ci.repo_id.to_ascii_lowercase().contains(stem)
                    && ci.variants.iter().any(|v| v.hardware_fit == HardwareFit::OptimalVram)
            }) {
                return cat_item.clone();
            }
        }

        if let Some(cat_item) = catalog
            .iter()
            .find(|ci| ci.assigned_tier == target_tier && ci.is_downloaded)
        {
            return cat_item.clone();
        }

        catalog
            .iter()
            .find(|ci| {
                ci.assigned_tier == target_tier
                    && ci.variants.iter().any(|v| v.hardware_fit == HardwareFit::OptimalVram)
            })
            .cloned()
            .unwrap_or_else(|| {
                catalog
                    .iter()
                    .find(|ci| ci.assigned_tier == target_tier)
                    .cloned()
                    .unwrap_or_else(|| catalog[0].clone())
            })
    };

    let mut tier0 = select_tier(
        "tier0_velocista",
        &["smollm2-360m", "qwen3-1.7b", "smollm2-135m", "smollm-135m", "llama-3.2-1b", "gemma-3-1b", "qwen2.5-coder-0.5b"],
    );
    let mut tier05 = select_tier(
        "tier05_socratico",
        &["qwen3-4b", "deepseek-r1", "nemotron", "phi-4-mini", "smollm2-1.7b", "temper"],
    );
    let mut tier1 = select_tier(
        "tier1_coder",
        &["qwen3.5-4b-agentic", "qwen3.5", "qwen3-5", "qwen3-8b", "qwen2.5-coder-3b", "qwen2.5-coder-7b", "gemma-3-4b", "hermes-3", "xlam"],
    );

    let mut upgrade_recs = Vec::new();

    for (tier_item, tier_key, reason_suffix) in [
        (&mut tier0, "tier0_velocista", "triagem e roteamento evoluído na VRAM"),
        (&mut tier05, "tier05_socratico", "deliberação socrática e raciocínio aprimorado"),
        (&mut tier1, "tier1_coder", "alternativa de alta capacidade para código e arquitetura"),
    ] {
        if let Some(alt) = select_evolutionary_alternative(tier_item, tier_key, &catalog, diag) {
            let rec_q = alt
                .variants
                .iter()
                .find(|v| v.is_recommended_fit)
                .or_else(|| alt.variants.first())
                .map(|v| v.quant_tag.clone())
                .unwrap_or_else(|| "Q4_K_M".to_string());
            tier_item.upgrade_recommendation = Some(format!(
                "{} ({}) — {}",
                alt.model_name, rec_q, reason_suffix
            ));
            upgrade_recs.push(alt.clone());
            tier_item.upgrade_candidate = Some(Box::new(alt));
        }
    }

    RecommendedTriad {
        tier0_velocista: tier0,
        tier05_socratico: tier05,
        tier1_coder: tier1,
        upgrade_recommendations: upgrade_recs,
    }
}

/// Computes the recommended 3-tier combo calibrated for the host silicon.
pub fn get_recommended_triad(
    diag: &HardwareDiagnostic,
    installed_filenames: &[String],
) -> RecommendedTriad {
    let installed_models: Vec<LocalModelMeta> = installed_filenames
        .iter()
        .map(|name| {
            let (params, _, _) = parse_model_params_from_name(name, None);
            let fit = evaluate_model_fit(diag, 0, 8192, params);
            LocalModelMeta {
                file_name: name.clone(),
                absolute_path: std::path::PathBuf::from(name),
                format: crate::catalog::ModelFormat::Gguf,
                size_bytes: 0,
                size_gb: 0.0,
                architecture: "unknown".to_string(),
                parameter_count: Some(params),
                quantization: extract_quant_tag(name),
                context_length_max: 8192,
                hardware_fit: fit,
                assigned_tier: None,
                last_modified_unix: 0,
            }
        })
        .collect();

    get_recommended_triad_from_models(diag, &installed_models)
}

/// Builds a rich dynamic catalog item from a live Hugging Face repository metadata entry.
///
/// When `real_files` is provided, variants use exact GGUF sizes, SHA-256 checksums, and download URLs;
/// otherwise realistic quantization estimates are computed from parameter counts.
pub fn build_dynamic_hf_catalog_item(
    diag: &HardwareDiagnostic,
    hf: &HfSearchResult,
    installed_filenames: &[String],
    is_trending: bool,
    real_files: Option<&[HfModelFile]>,
) -> Option<HuggingFaceModelItem> {
    let repo_lower = hf.repo_id.to_ascii_lowercase();
    let tags_lower: Vec<String> = hf.tags.iter().map(|t| t.to_ascii_lowercase()).collect();
    let pipeline = hf.pipeline_tag.as_deref().unwrap_or("").to_ascii_lowercase();

    // Filter out non-conversational/embedding/audio utility repos
    if matches!(
        pipeline.as_str(),
        "feature-extraction"
            | "sentence-similarity"
            | "automatic-speech-recognition"
            | "text-to-speech"
            | "text-to-image"
            | "image-classification"
    ) {
        return None;
    }

    let is_multimodal_vlm = pipeline == "image-text-to-text"
        || tags_lower.iter().any(|t| t == "multimodal" || t == "vision" || t == "image-text-to-text")
        || repo_lower.contains("-vl-")
        || repo_lower.contains("-vl")
        || repo_lower.contains("vlm")
        || repo_lower.contains("vision")
        || repo_lower.contains("llava")
        || repo_lower.contains("pixtral")
        || repo_lower.contains("minicpm-v")
        || repo_lower.contains("internvl");

    if !is_multimodal_vlm
        && ["embed", "bge-", "rerank", "whisper", "kokoro", "piper", "minilm", "sdxl", "flux"]
            .iter()
            .any(|kw| repo_lower.contains(kw))
    {
        return None;
    }

    // Parse parameter count from repo_id (falling back to tags or family heuristics)
    let (mut parameter_count, mut active_parameter_count, mut is_moe) =
        parse_model_params_from_name(&hf.repo_id, None);
    let has_explicit_size = repo_lower.contains('b') && repo_lower.chars().any(|c| c.is_ascii_digit())
        && parameter_count != 3_000_000_000
        || repo_lower.contains("-3b")
        || repo_lower.contains("3b-");

    if !has_explicit_size {
        for tag in &hf.tags {
            let (tp, ta, tmoe) = parse_model_params_from_name(tag, None);
            if tp != 3_000_000_000 || tag.to_ascii_lowercase().contains("3b") {
                parameter_count = tp;
                active_parameter_count = ta;
                is_moe = tmoe;
                break;
            }
        }
        if repo_lower.contains("phi-4") && !repo_lower.contains("mini") {
            parameter_count = 14_000_000_000;
        } else if repo_lower.contains("nemo") && !repo_lower.contains("nemotron") {
            parameter_count = 12_000_000_000;
        }
    }

    let eval_params = active_parameter_count.unwrap_or(parameter_count);
    let total_b = parameter_count as f64 / 1_000_000_000.0;
    let eval_b = eval_params as f64 / 1_000_000_000.0;

    let has_tag_kw = |kw: &str| tags_lower.iter().any(|t| t.contains(kw));
    let cognitive_profile = if is_multimodal_vlm {
        "vision"
    } else if repo_lower.contains("coder")
        || repo_lower.contains("code")
        || repo_lower.contains("codestral")
        || repo_lower.contains("devstral")
        || repo_lower.contains("starcoder")
        || has_tag_kw("code")
    {
        "coding"
    } else if repo_lower.contains("r1")
        || repo_lower.contains("reason")
        || repo_lower.contains("think")
        || repo_lower.contains("qwq")
        || repo_lower.contains("cot")
        || repo_lower.contains("math")
        || has_tag_kw("reasoning")
    {
        "reasoning"
    } else if repo_lower.contains("design")
        || repo_lower.contains("frontend")
        || repo_lower.contains("ui")
        || (repo_lower.contains("gemma-3") && (3.5..=14.0).contains(&total_b))
    {
        "design"
    } else if eval_b <= 1.8 {
        "fast"
    } else {
        "agentic"
    };

    let category_role = if total_b < 2.0 {
        "Featherweight"
    } else if total_b <= 4.5 {
        "SweetSpot"
    } else {
        "Heavyweight"
    };

    let assigned_tier = if is_multimodal_vlm {
        "tier3_multimodal"
    } else if is_moe || total_b >= 20.0 {
        "tier2_moe"
    } else if total_b <= 1.8 {
        "tier0_velocista"
    } else if total_b <= 4.5 && (cognitive_profile == "reasoning" || cognitive_profile == "fast") {
        "tier05_socratico"
    } else {
        "tier1_coder"
    };

    let installed_entries: Vec<(String, String, String)> = installed_filenames
        .iter()
        .map(|f| (f.clone(), normalize_stem(f), extract_quant_tag(f)))
        .collect();

    let repo_short = hf.repo_id.split('/').last().unwrap_or(&hf.repo_id).to_string();
    let author = hf
        .author
        .clone()
        .filter(|a| !a.is_empty())
        .or_else(|| hf.repo_id.split('/').next().map(String::from))
        .unwrap_or_else(|| "HuggingFace".to_string());
    let default_context_length = if repo_lower.contains("qwen3") {
        40960
    } else if repo_lower.contains("llama-3.2") || repo_lower.contains("gemma-3") || repo_lower.contains("phi-4") {
        131072
    } else {
        32768
    };
    let eval_context = default_context_length.min(8192);

    let mut variants = Vec::new();
    if let Some(files) = real_files.filter(|f| !f.is_empty()) {
        let proj = files
            .iter()
            .find(|f| f.file_name.to_ascii_lowercase().contains("mmproj"));

        let mut preferred: Vec<&HfModelFile> = files
            .iter()
            .filter(|f| !f.file_name.to_ascii_lowercase().contains("mmproj"))
            .filter(|f| {
                matches!(
                    f.quant_tag.as_str(),
                    "Q4_K_M" | "Q4_K_S" | "IQ4_XS" | "Q5_K_M" | "Q5_K_S" | "Q6_K" | "Q8_0" | "Q3_K_M"
                )
            })
            .collect();

        if preferred.is_empty() {
            preferred = files
                .iter()
                .filter(|f| !f.file_name.to_ascii_lowercase().contains("mmproj"))
                .take(4)
                .collect();
        }

        // Sort common practical order: Q4_K_M, Q5_K_M, IQ4_XS, Q4_K_S, Q6_K, Q8_0, Q3_K_M
        let rank_q = |tag: &str| -> u8 {
            match tag {
                "Q4_K_M" => 0,
                "Q5_K_M" => 1,
                "IQ4_XS" => 2,
                "Q4_K_S" => 3,
                "Q5_K_S" => 4,
                "Q6_K" => 5,
                "Q8_0" => 6,
                "Q3_K_M" => 7,
                _ => 8,
            }
        };
        preferred.sort_by_key(|f| rank_q(&f.quant_tag));

        let mut seen_tags = std::collections::HashSet::new();
        for f in preferred {
            if !seen_tags.insert(f.quant_tag.clone()) && variants.len() >= 4 {
                continue;
            }
            let proj_mb = proj.map(|p| p.size_bytes / (1024 * 1024));
            let fit_bytes = f.size_bytes + proj.map(|p| p.size_bytes).unwrap_or(0);
            let fit = evaluate_model_fit(diag, fit_bytes, eval_context, parameter_count);
            let est_tps = estimate_variant_throughput_tps(diag, fit, f.size_bytes, eval_params);

            let matched = installed_entries.iter().find(|(rn, norm, iq)| {
                variant_matches_installed(&f.file_name, &f.quant_tag, rn, norm, iq)
            });

            variants.push(QuantizationVariant {
                quant_tag: f.quant_tag.clone(),
                file_name: f.file_name.clone(),
                download_url: f.download_url.clone(),
                size_bytes: f.size_bytes,
                size_gb: f.size_gb,
                sha256: f.sha256.clone(),
                hardware_fit: fit,
                is_recommended_fit: false,
                is_installed: matched.is_some(),
                installed_file_name: matched.map(|m| m.0.clone()),
                estimated_tps: est_tps,
                projector_file: proj.map(|p| p.file_name.clone()),
                projector_url: proj.map(|p| p.download_url.clone()),
                projector_size_mb: proj_mb,
            });
            if variants.len() >= 6 {
                break;
            }
        }
    }

    // Synthetic quantization fallback if live repo tree was not queried yet
    if variants.is_empty() {
        let base_stem = repo_short.trim_end_matches("-GGUF").trim_end_matches("-gguf");
        let synth_specs: &[(&str, f64)] = if total_b > 20.0 {
            &[("Q3_K_M", 490.0), ("Q4_K_M", 610.0), ("Q5_K_M", 715.0)]
        } else {
            &[("Q4_K_M", 610.0), ("Q5_K_M", 715.0), ("Q8_0", 1040.0)]
        };

        for (qtag, mb_per_b) in synth_specs {
            let size_mb = ((total_b * mb_per_b).round() as u64).max(80);
            let size_bytes = size_mb * 1024 * 1024;
            let file_name = format!("{}-{}.gguf", base_stem, qtag);
            let fit = evaluate_model_fit(diag, size_bytes, eval_context, parameter_count);
            let est_tps = estimate_variant_throughput_tps(diag, fit, size_bytes, eval_params);

            let matched = installed_entries.iter().find(|(rn, norm, iq)| {
                variant_matches_installed(&file_name, qtag, rn, norm, iq)
            });

            variants.push(QuantizationVariant {
                quant_tag: (*qtag).to_string(),
                file_name,
                download_url: String::new(), // Frontend resolves real tree via /repo-files before downloading
                size_bytes,
                size_gb: size_mb as f64 / 1024.0,
                sha256: None,
                hardware_fit: fit,
                is_recommended_fit: false,
                is_installed: matched.is_some(),
                installed_file_name: matched.map(|m| m.0.clone()),
                estimated_tps: est_tps,
                projector_file: None,
                projector_url: None,
                projector_size_mb: None,
            });
        }
    }

    let recommended_idx = pick_recommended_variant_index(&variants);
    if let Some(v) = variants.get_mut(recommended_idx) {
        v.is_recommended_fit = true;
    }

    let installed_var = variants.iter().find(|v| v.is_installed);
    let is_downloaded = installed_var.is_some();
    let local_file_path = variants
        .get(recommended_idx)
        .and_then(|v| v.installed_file_name.clone())
        .or_else(|| installed_var.and_then(|v| v.installed_file_name.clone()));

    let primary_fit = variants
        .get(recommended_idx)
        .map(|v| v.hardware_fit)
        .unwrap_or(HardwareFit::PartialCpuOffload);
    let speed_tps = variants
        .get(recommended_idx)
        .map(|v| v.estimated_tps)
        .unwrap_or_else(|| estimate_throughput_tps(primary_fit, eval_params, diag.has_avx2));

    // Heuristic baseline benchmark projections calibrated by architecture generation + parameter scale
    let gen_bonus: f32 = if repo_lower.contains("qwen3")
        || repo_lower.contains("deepseek-r1")
        || repo_lower.contains("phi-4")
        || repo_lower.contains("gemma-3")
    {
        5.5
    } else if repo_lower.contains("qwen2.5")
        || repo_lower.contains("llama-3.2")
        || repo_lower.contains("llama-3.3")
        || repo_lower.contains("hermes-3")
        || repo_lower.contains("nemotron-3")
    {
        3.0
    } else {
        0.5
    };

    let log_b = (total_b + 0.5).ln() as f32;
    let mmlu_score = Some((54.5 + 7.2 * log_b + gen_bonus).clamp(44.0, 92.0));
    let coding_bias = match cognitive_profile {
        "coding" => 7.5,
        "reasoning" => 2.5,
        "fast" => -8.0,
        _ => 0.0,
    };
    let coding_score = Some((52.0 + 7.4 * log_b + gen_bonus + coding_bias).clamp(36.0, 94.0));
    let ifeval_bias = match cognitive_profile {
        "agentic" | "reasoning" => 3.5,
        _ => 0.0,
    };
    let ifeval_score = Some((59.5 + 6.5 * log_b + gen_bonus + ifeval_bias).clamp(52.0, 93.0));

    let param_label = if total_b < 1.0 {
        format!("{:.0}M", total_b * 1000.0)
    } else {
        format!("{:.1}B", total_b).replace(".0B", "B")
    };

    let size_label = variants
        .get(recommended_idx)
        .map(|v| format!("{:.2} GB {}", v.size_gb, v.quant_tag))
        .unwrap_or_else(|| "GGUF".to_string());

    let silicon_label = match primary_fit {
        HardwareFit::OptimalVram => "dGPU",
        HardwareFit::PartialCpuOffload => "CPU Offload",
        HardwareFit::IncompatibleSpillover => "Spillover",
    };

    let has_verified_files = real_files.map(|f| !f.is_empty()).unwrap_or(false);
    let highlights = vec![
        format!("Porte {}", param_label),
        format!("~{} tok/s {}", speed_tps.round() as u32, silicon_label),
        size_label,
        if has_verified_files {
            "SHA-256 Verificado HF".to_string()
        } else if is_trending {
            "Em Alta Hugging Face".to_string()
        } else {
            "Descoberta HF Live".to_string()
        },
    ];

    let display_name = repo_short
        .trim_end_matches("-GGUF")
        .trim_end_matches("-gguf")
        .replace('-', " ");

    let why_recommended = format!(
        "{} no Hugging Face ({} downloads, {} curtidas). Score e velocidade projetados dinamicamente para {}.",
        if is_trending { "Tendência em alta" } else { "Repositório validado" },
        hf.downloads,
        hf.likes,
        param_label,
    );

    Some(HuggingFaceModelItem {
        repo_id: hf.repo_id.clone(),
        model_name: display_name,
        author,
        parameter_count,
        active_parameter_count,
        is_moe,
        default_context_length,
        cognitive_profile: cognitive_profile.to_string(),
        assigned_tier: assigned_tier.to_string(),
        category_role: category_role.to_string(),
        why_recommended,
        highlights,
        benchmark_scores: BenchmarkExpectation {
            coding_score,
            ifeval_score,
            mmlu_score,
            speed_tokens_sec_estimate: speed_tps,
        },
        variants,
        is_downloaded,
        is_installed: is_downloaded,
        local_file_path,
        is_multimodal_vlm,
        upgrade_recommendation: None,
        upgrade_candidate: None,
        downloads: Some(hf.downloads),
        likes: Some(hf.likes),
        is_trending,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_curated_catalog_evaluation() {
        let diag = HardwareDiagnostic {
            has_dedicated_gpu: true,
            gpu_name: "Generic Test GPU".to_string(),
            cpu_name: "Generic Test CPU".to_string(),
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
        assert!(coder_7b.is_installed);
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

    #[test]
    fn test_recommended_triad_prioritizes_installed_optimal_vram() {
        let diag = HardwareDiagnostic {
            has_dedicated_gpu: true,
            gpu_name: "Generic Test GPU".to_string(),
            cpu_name: "Generic Test CPU".to_string(),
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

        let installed_models = vec![
            LocalModelMeta {
                file_name: "smollm2-360m-instruct-q8_0.gguf".to_string(),
                absolute_path: std::path::PathBuf::from("smollm2-360m-instruct-q8_0.gguf"),
                format: crate::catalog::ModelFormat::Gguf,
                size_bytes: 380 * 1024 * 1024,
                size_gb: 0.38,
                architecture: "llama".to_string(),
                parameter_count: Some(360_000_000),
                quantization: "Q8_0".to_string(),
                context_length_max: 8192,
                hardware_fit: HardwareFit::OptimalVram,
                assigned_tier: Some("tier0_velocista".to_string()),
                last_modified_unix: 1700000000,
            },
            LocalModelMeta {
                file_name: "NVIDIA-Nemotron-3-Nano-4B-Q4_K_M.gguf".to_string(),
                absolute_path: std::path::PathBuf::from("NVIDIA-Nemotron-3-Nano-4B-Q4_K_M.gguf"),
                format: crate::catalog::ModelFormat::Gguf,
                size_bytes: 2480 * 1024 * 1024,
                size_gb: 2.48,
                architecture: "nemotron".to_string(),
                parameter_count: Some(4_000_000_000),
                quantization: "Q4_K_M".to_string(),
                context_length_max: 16384,
                hardware_fit: HardwareFit::OptimalVram,
                assigned_tier: Some("tier05_socratico".to_string()),
                last_modified_unix: 1700000000,
            },
            LocalModelMeta {
                file_name: "qwen3-5-4b-agentic-coder-v4.gguf".to_string(),
                absolute_path: std::path::PathBuf::from("qwen3-5-4b-agentic-coder-v4.gguf"),
                format: crate::catalog::ModelFormat::Gguf,
                size_bytes: 2650 * 1024 * 1024,
                size_gb: 2.65,
                architecture: "qwen2".to_string(),
                parameter_count: Some(4_000_000_000),
                quantization: "Q4_K_M".to_string(),
                context_length_max: 32768,
                hardware_fit: HardwareFit::OptimalVram,
                assigned_tier: Some("tier1_coder".to_string()),
                last_modified_unix: 1700000000,
            },
        ];

        let triad = get_recommended_triad_from_models(&diag, &installed_models);

        assert!(triad.tier0_velocista.is_installed);
        assert!(triad.tier0_velocista.repo_id.contains("SmolLM2-360M"));

        assert!(triad.tier05_socratico.is_installed);
        assert!(triad.tier05_socratico.repo_id.contains("Nemotron-3-Nano"));

        assert!(triad.tier1_coder.is_installed);
        assert!(triad.tier1_coder.repo_id.contains("Qwen3.5-4B-Agentic-Coder"));
        assert!(triad.tier1_coder.upgrade_recommendation.is_some());
    }

    #[test]
    fn test_curated_catalog_moe_parameters() {
        let diag = HardwareDiagnostic::default();
        let catalog = get_curated_catalog(&diag, &[]);

        let moe = catalog
            .iter()
            .find(|m| m.is_moe)
            .expect("MoE model must exist in catalog");

        assert_eq!(moe.repo_id, "Qwen/Qwen3-30B-A3B-GGUF");
        assert_eq!(moe.parameter_count, 30_000_000_000);
        assert_eq!(moe.active_parameter_count, Some(3_000_000_000));
    }

    #[test]
    fn test_recommended_triad_no_cross_tier_misattribution() {
        let diag = HardwareDiagnostic {
            has_dedicated_gpu: true,
            gpu_name: "Generic Test GPU".to_string(),
            cpu_name: "Generic Test CPU".to_string(),
            vram_total_mb: 6144,
            vram_free_mb: 5000,
            vram_safe_usable_mb: 4800,
            host_ram_total_mb: 32768,
            host_ram_available_mb: 24000,
            host_ram_safe_usable_mb: 20000,
            cpu_cores_physical: 8,
            cpu_cores_logical: 16,
            has_avx2: true,
            has_avx512: false,
        };

        // ONLY smollm2-1.7b is installed
        let installed_models = vec![
            LocalModelMeta {
                file_name: "smollm2-1.7b-instruct-q4_k_m.gguf".to_string(),
                absolute_path: std::path::PathBuf::from("smollm2-1.7b-instruct-q4_k_m.gguf"),
                format: crate::catalog::ModelFormat::Gguf,
                size_bytes: 1050 * 1024 * 1024,
                size_gb: 1.05,
                architecture: "llama".to_string(),
                parameter_count: Some(1_700_000_000),
                quantization: "Q4_K_M".to_string(),
                context_length_max: 8192,
                hardware_fit: HardwareFit::OptimalVram,
                assigned_tier: Some("tier05_socratico".to_string()),
                last_modified_unix: 1700000000,
            },
        ];

        let triad = get_recommended_triad_from_models(&diag, &installed_models);

        // Tier 0 (135M / 360M) must NOT be marked installed using 1.7B file
        assert!(!triad.tier0_velocista.is_installed, "Tier 0 should not be marked installed when only 1.7B is present");
        assert_ne!(triad.tier0_velocista.local_file_path.as_deref(), Some("smollm2-1.7b-instruct-q4_k_m.gguf"));

        // Tier 0.5 (1.7B) SHOULD be marked installed
        assert!(triad.tier05_socratico.is_installed);
        assert_eq!(triad.tier05_socratico.local_file_path.as_deref(), Some("smollm2-1.7b-instruct-q4_k_m.gguf"));

        // Tier 1 must NOT be marked installed
        assert!(!triad.tier1_coder.is_installed);
    }

    #[test]
    fn test_strict_variant_quantization_matching_and_roles() {
        let diag = HardwareDiagnostic {
            has_dedicated_gpu: true,
            gpu_name: "NVIDIA GeForce RTX 2060".to_string(),
            cpu_name: "Intel Core i9".to_string(),
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

        let installed = vec![
            "Phi-4-mini-instruct-Q4_K_M.gguf".to_string(),
            "smollm2-360m-instruct-q8_0.gguf".to_string(),
            "qwen3.5-4b-agentic-coder-v4.i1-Q4_K_S.gguf".to_string(),
        ];
        let catalog = get_curated_catalog(&diag, &installed);

        // Role sanity checks: no small model tagged Heavyweight
        let smol_360 = catalog.iter().find(|m| m.repo_id.contains("SmolLM2-360M")).unwrap();
        assert_eq!(smol_360.category_role, "Featherweight");
        assert!(smol_360.variants.iter().find(|v| v.quant_tag == "Q8_0").unwrap().is_installed);
        assert!(!smol_360.variants.iter().find(|v| v.quant_tag == "Q4_K_M").unwrap().is_installed);
        assert!(smol_360.variants.iter().find(|v| v.quant_tag == "Q8_0").unwrap().is_recommended_fit);

        let phi4 = catalog.iter().find(|m| m.repo_id.contains("Phi-4-mini")).unwrap();
        assert_eq!(phi4.category_role, "SweetSpot");
        assert!(phi4.variants.iter().find(|v| v.quant_tag == "Q4_K_M").unwrap().is_installed);
        assert!(!phi4.variants.iter().find(|v| v.quant_tag == "Q5_K_M").unwrap().is_installed);
        assert!(phi4.variants.iter().find(|v| v.quant_tag == "Q4_K_M").unwrap().is_recommended_fit);

        // Unlisted imatrix quant Q4_K_S matches model stem cleanly while keeping Q5_K_M not installed
        let qwen35 = catalog.iter().find(|m| m.repo_id.contains("Qwen3.5-4B")).unwrap();
        assert!(qwen35.is_installed);
        assert!(!qwen35.variants.iter().find(|v| v.quant_tag == "Q5_K_M").unwrap().is_installed);
        let q4s = qwen35.variants.iter().find(|v| v.quant_tag == "Q4_K_S").expect("Q4_K_S local variant added");
        assert!(q4s.is_installed);
        assert!(q4s.is_recommended_fit);
    }

    #[test]
    fn test_evolutionary_triad_populates_all_three_upgrade_candidates() {
        let diag = HardwareDiagnostic {
            has_dedicated_gpu: true,
            gpu_name: "NVIDIA GeForce RTX 2060".to_string(),
            cpu_name: "Intel Core i9".to_string(),
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

        let installed = vec![
            "smollm2-360m-instruct-q8_0.gguf".to_string(),
            "NVIDIA-Nemotron-3-Nano-4B-Q4_K_M.gguf".to_string(),
            "qwen3.5-4b-agentic-coder-v4.i1-Q4_K_S.gguf".to_string(),
        ];
        let triad = get_recommended_triad(&diag, &installed);

        for tier_item in [&triad.tier0_velocista, &triad.tier05_socratico, &triad.tier1_coder] {
            let alt = tier_item
                .upgrade_candidate
                .as_ref()
                .expect("Each installed tier should expose an evolutionary non-installed alternative");
            assert!(!alt.is_downloaded);
            assert_ne!(alt.repo_id, tier_item.repo_id);
            assert!(alt.variants.iter().any(|v| v.hardware_fit == HardwareFit::OptimalVram));
        }
    }

    #[test]
    fn test_build_dynamic_hf_catalog_item() {
        let diag = HardwareDiagnostic::default();
        let hf_llm = HfSearchResult {
            repo_id: "unsloth/Qwen3-4B-GGUF".to_string(),
            author: Some("unsloth".to_string()),
            downloads: 42000,
            likes: 180,
            last_modified: Some("2026-10-01T00:00:00Z".to_string()),
            pipeline_tag: Some("text-generation".to_string()),
            tags: vec!["gguf".to_string(), "reasoning".to_string()],
        };
        let item = build_dynamic_hf_catalog_item(&diag, &hf_llm, &[], true, None)
            .expect("LLM GGUF repo must convert into dynamic catalog item");
        assert_eq!(item.category_role, "SweetSpot");
        assert_eq!(item.cognitive_profile, "reasoning");
        assert!(item.is_trending);
        assert_eq!(item.variants.len(), 3);
        assert!(item.benchmark_scores.mmlu_score.is_some());

        let hf_embed = HfSearchResult {
            repo_id: "BAAI/bge-m3-GGUF".to_string(),
            author: Some("BAAI".to_string()),
            downloads: 90000,
            likes: 200,
            last_modified: None,
            pipeline_tag: Some("feature-extraction".to_string()),
            tags: vec!["gguf".to_string()],
        };
        assert!(build_dynamic_hf_catalog_item(&diag, &hf_embed, &[], true, None).is_none());
    }
}
