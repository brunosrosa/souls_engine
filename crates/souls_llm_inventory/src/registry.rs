//! Curated SSOT Registry of Elite Hugging Face Models and Quantizations.
//!
//! Provides canonical reference models for the 3 Tiers (Tier 0, Tier 0.5, Tier 1)
//! and 6 Cognitive Profiles (Coding, Reasoning, Agentic, Design, Vision, Fast),
//! dynamically evaluated against the host HardwareDiagnostic with dual SweetSpot/Heavyweight roles.

use crate::catalog::LocalModelMeta;
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
    #[serde(default)]
    pub active_parameter_count: Option<u64>,
    #[serde(default)]
    pub is_moe: bool,
    pub default_context_length: u32,
    pub cognitive_profile: String,
    pub assigned_tier: String,
    pub category_role: String, // "SweetSpot" (Eficiência 3B-4B, 45+ tps) ou "Heavyweight" (Capacidade máxima 7B-8B)
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
        repo_id: "deepseek-ai/DeepSeek-R1-Distill-Qwen-1.5B-GGUF",
        model_name: "DeepSeek R1 Distill Qwen 1.5B",
        author: "DeepSeek",
        parameter_count: 1_500_000_000,
        active_parameter_count: None,
        is_moe: false,
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
        active_parameter_count: None,
        is_moe: false,
        context_length: 131072,
        cognitive_profile: "reasoning",
        assigned_tier: "tier05_socratico",
        category_role: "Heavyweight",
        why_recommended: "Matemática rigorosa e resolução de problemas analíticos com arquitetura Phi-4 e janela nativa de 128k. Pontua 76.4 no MMLU ocupando apenas 2.5 GB.",
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
                quant_tag: "Q4_K_M",
                file_name: "nemotron-mini-4b-instruct-q4_k_m.gguf",
                size_mb: 2480,
                sha256: None,
                projector_file: None,
                projector_size_mb: None,
            },
            RawVariantDef {
                quant_tag: "Q5_K_M",
                file_name: "nemotron-mini-4b-instruct-q5_k_m.gguf",
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
        repo_id: "HuggingFaceTB/SmolLM2-135M-Instruct-GGUF",
        model_name: "SmolLM2 135M Instruct",
        author: "HuggingFaceTB",
        parameter_count: 135_000_000,
        active_parameter_count: None,
        is_moe: false,
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
        active_parameter_count: None,
        is_moe: false,
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
        category_role: "SweetSpot",
        why_recommended: "Modelo de 1.7B de vanguarda ultra-leve da HuggingFace. Excelente compromisso entre velocidade (85 tok/s) e capacidade de deliberação para o Tier 0.5 sem ocupar mais de 1.1 GB de VRAM.",
        highlights: &["85+ tok/s dGPU", "1.1 GB VRAM", "Tier 0.5 Eficiente", "HuggingFace SmolLM2"],
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
        category_role: "SweetSpot",
        why_recommended: "Velocista oficial da Meta com janela nativa de 128k tokens. 140+ tok/s para classificação rápida e guardrails ocupando apenas 780 MB.",
        highlights: &["128k Contexto Nativo", "140+ tok/s dGPU", "Apenas 780 MB VRAM", "Meta Llama 3.2"],
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
        category_role: "SweetSpot",
        why_recommended: "Modelo ultra-compacto da Google com arquitetura Gemma 3. Rápido (120+ tok/s), ideal para triagem instantânea e baixo consumo de memória.",
        highlights: &["Google Gemma 3", "120+ tok/s dGPU", "Apenas 800 MB", "Baixo Consumo"],
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

            let eval_params = def.active_parameter_count.unwrap_or(def.parameter_count);
            let speed_tps = estimate_throughput_tps(
                primary_fit,
                eval_params,
                diag.has_avx2,
            );

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
                highlights: def.highlights.iter().map(|s| s.to_string()).collect(),
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
            }
        })
        .collect()
}

/// Computes the recommended 3-tier combo calibrated for the host silicon,
/// prioritizing installed models that fit comfortably in VRAM.
pub fn get_recommended_triad_from_models(
    diag: &HardwareDiagnostic,
    installed_models: &[LocalModelMeta],
) -> RecommendedTriad {
    let installed_filenames: Vec<String> = installed_models
        .iter()
        .map(|m| m.file_name.clone())
        .collect();

    let mut catalog = get_curated_catalog(diag, &installed_filenames);

    // Synchronize is_installed with is_downloaded
    for item in &mut catalog {
        if item.is_downloaded {
            item.is_installed = true;
        }
    }

    // Helper: Select best model for a tier
    let select_tier = |target_tier: &str, priority_stems: &[&str]| -> HuggingFaceModelItem {
        // A. First check if any installed model matches this tier and fits OptimalVram or TightVram
        for stem in priority_stems {
            if let Some(m) = installed_models.iter().find(|im| {
                im.file_name.to_ascii_lowercase().contains(stem)
            }) {
                if let Some(mut cat_item) = catalog.iter().find(|ci| {
                    ci.assigned_tier == target_tier
                        && (ci.local_file_path.as_deref() == Some(&m.file_name)
                            || ci.repo_id.to_ascii_lowercase().contains(stem))
                        && ci.variants.iter().any(|v| v.hardware_fit == HardwareFit::OptimalVram)
                }).cloned() {
                    cat_item.is_downloaded = true;
                    cat_item.is_installed = true;
                    cat_item.local_file_path = Some(m.file_name.clone());
                    return cat_item;
                }
            }
        }

        // B. Next check catalog items that are already downloaded / installed and fit safely
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

        // C. Any downloaded catalog item for this tier
        if let Some(cat_item) = catalog.iter().find(|ci| {
            ci.assigned_tier == target_tier
                && ci.is_downloaded
        }) {
            return cat_item.clone();
        }

        // D. Fallback to catalog recommendation fitting OptimalVram
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

    let mut tier0 = select_tier("tier0_velocista", &["smollm2-360m", "smollm2-135m", "smollm-135m", "llama-3.2-1b", "gemma-3-1b", "qwen2.5-coder-0.5b"]);
    let mut tier05 = select_tier("tier05_socratico", &["deepseek-r1", "nemotron", "phi-4-mini", "smollm2-1.7b", "temper"]);
    let mut tier1 = select_tier("tier1_coder", &["qwen3.5-4b-agentic", "qwen3.5", "qwen3-5", "qwen2.5-coder-3b", "qwen2.5-coder-7b", "gemma-3-4b", "hermes-3", "xlam"]);

    // Attach upgrade recommendations
    let mut upgrade_recs = Vec::new();

    // Tier 0 upgrade
    if tier0.parameter_count <= 200_000_000 {
        tier0.upgrade_recommendation = Some("SmolLM2 360M Instruct (compreensão aprimorada a 160+ tok/s)".to_string());
    }

    // Tier 0.5 upgrade
    if tier05.parameter_count <= 2_000_000_000 && diag.vram_safe_usable_mb >= 3500 {
        tier05.upgrade_recommendation = Some("Microsoft Phi-4 Mini 3.8B (raciocínio deliberativo avançado em 128k)".to_string());
    }

    // Tier 1 upgrade: Heavyweight if host has >= 5GB usable VRAM
    if tier1.category_role == "SweetSpot" && diag.vram_safe_usable_mb >= 5000 {
        if let Some(heavy) = catalog.iter().find(|m| {
            m.assigned_tier == "tier1_coder"
                && m.category_role == "Heavyweight"
                && m.variants.iter().any(|v| v.hardware_fit == HardwareFit::OptimalVram)
        }) {
            tier1.upgrade_recommendation = Some(format!("{} (Q4_K_M) — capacidade máxima para arquitetura pesada", heavy.model_name));
            upgrade_recs.push(heavy.clone());
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
            let (params, _, _) = crate::catalog::parse_model_params_from_name(name, None);
            let fit = evaluate_model_fit(diag, 0, 8192, params);
            LocalModelMeta {
                file_name: name.clone(),
                absolute_path: std::path::PathBuf::from(name),
                format: crate::catalog::ModelFormat::Gguf,
                size_bytes: 0,
                size_gb: 0.0,
                architecture: "unknown".to_string(),
                parameter_count: Some(params),
                quantization: crate::hf_client::extract_quant_tag(name),
                context_length_max: 8192,
                hardware_fit: fit,
                assigned_tier: None,
                last_modified_unix: 0,
            }
        })
        .collect();

    get_recommended_triad_from_models(diag, &installed_models)
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
}
