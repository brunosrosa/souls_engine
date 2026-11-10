//! # Cloud Provider Taxonomy & FinOps Cache Engine
//!
//! Provides first-class classification for modern cloud inference providers (DeepSeek, Groq,
//! Cerebras, Bedrock, Together, Azure, etc.) and models caching economics (prompt caching discounts).

use serde::{Deserialize, Serialize};

/// Canonical cloud and local provider classification.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CloudProvider {
    /// DeepSeek API (v3 / r1) with aggressive prompt caching discounts.
    DeepSeek,
    /// Alibaba Cloud DashScope (Qwen 2.5 / 3.5 / Max).
    AlibabaDashScope,
    /// Moonshot AI (Kimi 128k / Kimi Latest).
    MoonshotKimi,
    /// Zhipu AI (GLM-4-Plus / GLM-4-Air).
    ZhipuGLM,
    /// MiniMax (ABAB 6.5s Chat).
    MiniMax,
    /// SiliconFlow multi-model accelerator endpoint.
    SiliconFlow,
    /// Groq LPU low-latency inference.
    Groq,
    /// Cerebras Wafer-Scale cluster high-throughput inference.
    Cerebras,
    /// Together AI multi-model endpoint.
    Together,
    /// Amazon Bedrock (Claude 3.5 Sonnet, Nova, etc.).
    Bedrock,
    /// Microsoft Azure OpenAI.
    AzureOpenAI,
    /// OpenAI official API.
    OpenAI,
    /// Anthropic official API.
    Anthropic,
    /// Google Gemini official API.
    Gemini,
    /// OpenRouter aggregator.
    OpenRouter,
    /// Host silicon (RTX 2060m or CPU AVX2).
    LocalSilicon,
    /// Custom or self-hosted endpoint.
    Custom(String),
}

impl CloudProvider {
    /// Infers the provider from model identifier or provider string.
    pub fn from_model_id_and_type(model_id: &str, provider_type: &str) -> Self {
        if provider_type.eq_ignore_ascii_case("LOCAL") {
            return Self::LocalSilicon;
        }

        let lower = model_id.to_ascii_lowercase();

        // 1. Explicit provider prefix/namespace checking
        if lower.starts_with("siliconflow/") {
            Self::SiliconFlow
        } else if lower.starts_with("dashscope/") || lower.starts_with("alibaba/") {
            Self::AlibabaDashScope
        } else if lower.starts_with("moonshot/") || lower.starts_with("kimi/") {
            Self::MoonshotKimi
        } else if lower.starts_with("zhipu/") || lower.starts_with("glm/") {
            Self::ZhipuGLM
        } else if lower.starts_with("minimax/") {
            Self::MiniMax
        } else if lower.starts_with("deepseek/") {
            Self::DeepSeek
        } else if lower.starts_with("groq/") {
            Self::Groq
        } else if lower.starts_with("cerebras/") {
            Self::Cerebras
        } else if lower.starts_with("together/") {
            Self::Together
        } else if lower.starts_with("bedrock/") {
            Self::Bedrock
        } else if lower.starts_with("azure/") {
            Self::AzureOpenAI
        } else if lower.starts_with("openai/") {
            Self::OpenAI
        } else if lower.starts_with("anthropic/") {
            Self::Anthropic
        } else if lower.starts_with("google/") || lower.starts_with("gemini/") {
            Self::Gemini
        } else if lower.starts_with("openrouter/") {
            Self::OpenRouter
        // 2. Keyword heuristic matching
        } else if lower.contains("deepseek") {
            Self::DeepSeek
        } else if lower.contains("dashscope") || lower.contains("qwen") || lower.contains("alibaba") {
            Self::AlibabaDashScope
        } else if lower.contains("moonshot") || lower.contains("kimi") {
            Self::MoonshotKimi
        } else if lower.contains("zhipu") || lower.contains("glm") {
            Self::ZhipuGLM
        } else if lower.contains("minimax") || lower.contains("abab") {
            Self::MiniMax
        } else if lower.contains("siliconflow") {
            Self::SiliconFlow
        } else if lower.contains("groq") {
            Self::Groq
        } else if lower.contains("cerebras") {
            Self::Cerebras
        } else if lower.contains("together") {
            Self::Together
        } else if lower.contains("bedrock") {
            Self::Bedrock
        } else if lower.contains("azure") {
            Self::AzureOpenAI
        } else if lower.contains("openai") || lower.starts_with("gpt-") || lower.starts_with("o1") || lower.starts_with("o3") {
            Self::OpenAI
        } else if lower.contains("anthropic") || lower.contains("claude") {
            Self::Anthropic
        } else if lower.contains("gemini") || lower.contains("google") {
            Self::Gemini
        } else if lower.contains("openrouter") || lower.contains('/') {
            Self::OpenRouter
        } else {
            Self::Custom(model_id.to_string())
        }
    }

    /// Infers provider classification from provider name or slug.
    pub fn from_provider_name(name: &str) -> Option<Self> {
        let lower = name.trim().to_ascii_lowercase();
        match lower.as_str() {
            "deepseek" => Some(Self::DeepSeek),
            "dashscope" | "alibaba" | "aliyun" | "qwen" => Some(Self::AlibabaDashScope),
            "moonshot" | "kimi" => Some(Self::MoonshotKimi),
            "zhipu" | "zhipuai" | "glm" => Some(Self::ZhipuGLM),
            "minimax" => Some(Self::MiniMax),
            "siliconflow" => Some(Self::SiliconFlow),
            "groq" => Some(Self::Groq),
            "cerebras" => Some(Self::Cerebras),
            "together" | "togetherai" => Some(Self::Together),
            "bedrock" | "aws" => Some(Self::Bedrock),
            "azure" | "azure_openai" => Some(Self::AzureOpenAI),
            "openai" => Some(Self::OpenAI),
            "anthropic" => Some(Self::Anthropic),
            "gemini" | "google" => Some(Self::Gemini),
            "openrouter" => Some(Self::OpenRouter),
            "local" | "localsilicon" | "host" => Some(Self::LocalSilicon),
            _ => None,
        }
    }

    /// Canonical short slug identifier.
    #[inline]
    pub fn canonical_slug(&self) -> &'static str {
        match self {
            Self::DeepSeek => "deepseek",
            Self::AlibabaDashScope => "dashscope",
            Self::MoonshotKimi => "moonshot",
            Self::ZhipuGLM => "zhipu",
            Self::MiniMax => "minimax",
            Self::SiliconFlow => "siliconflow",
            Self::Groq => "groq",
            Self::Cerebras => "cerebras",
            Self::Together => "together",
            Self::Bedrock => "bedrock",
            Self::AzureOpenAI => "azure_openai",
            Self::OpenAI => "openai",
            Self::Anthropic => "anthropic",
            Self::Gemini => "gemini",
            Self::OpenRouter => "openrouter",
            Self::LocalSilicon => "local",
            Self::Custom(_) => "custom",
        }
    }

    /// Human-readable display name.
    #[inline]
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::DeepSeek => "DeepSeek",
            Self::AlibabaDashScope => "Alibaba Cloud DashScope",
            Self::MoonshotKimi => "Moonshot AI (Kimi)",
            Self::ZhipuGLM => "Zhipu AI (GLM)",
            Self::MiniMax => "MiniMax",
            Self::SiliconFlow => "SiliconFlow",
            Self::Groq => "Groq LPU",
            Self::Cerebras => "Cerebras Wafer-Scale",
            Self::Together => "Together AI",
            Self::Bedrock => "Amazon Bedrock",
            Self::AzureOpenAI => "Microsoft Azure OpenAI",
            Self::OpenAI => "OpenAI",
            Self::Anthropic => "Anthropic",
            Self::Gemini => "Google Gemini",
            Self::OpenRouter => "OpenRouter",
            Self::LocalSilicon => "Host Local Silicon (RTX 2060m / AVX2)",
            Self::Custom(_) => "Custom Endpoint",
        }
    }

    /// Canonical base URL for API completions.
    #[inline]
    pub fn default_base_url(&self) -> &'static str {
        match self {
            Self::DeepSeek => "https://api.deepseek.com/v1",
            Self::AlibabaDashScope => "https://dashscope.aliyuncs.com/compatible-mode/v1",
            Self::MoonshotKimi => "https://api.moonshot.cn/v1",
            Self::ZhipuGLM => "https://open.bigmodel.cn/api/paas/v4",
            Self::MiniMax => "https://api.minimax.chat/v1",
            Self::SiliconFlow => "https://api.siliconflow.cn/v1",
            Self::Groq => "https://api.groq.com/openai/v1",
            Self::Cerebras => "https://api.cerebras.ai/v1",
            Self::Together => "https://api.together.xyz/v1",
            Self::Bedrock => "https://bedrock-runtime.us-east-1.amazonaws.com",
            Self::AzureOpenAI => "https://models.inference.ai.azure.com",
            Self::OpenAI => "https://api.openai.com/v1",
            Self::Anthropic => "https://api.anthropic.com/v1",
            Self::Gemini => "https://generativelanguage.googleapis.com/v1beta",
            Self::OpenRouter => "https://openrouter.ai/api/v1",
            Self::LocalSilicon => "http://127.0.0.1:9123/v1",
            Self::Custom(_) => "",
        }
    }

    /// Canonical environment variable name for the API authentication token.
    #[inline]
    pub fn auth_header_env(&self) -> &'static str {
        match self {
            Self::DeepSeek => "DEEPSEEK_API_KEY",
            Self::AlibabaDashScope => "DASHSCOPE_API_KEY",
            Self::MoonshotKimi => "MOONSHOT_API_KEY",
            Self::ZhipuGLM => "ZHIPUAI_API_KEY",
            Self::MiniMax => "MINIMAX_API_KEY",
            Self::SiliconFlow => "SILICONFLOW_API_KEY",
            Self::Groq => "GROQ_API_KEY",
            Self::Cerebras => "CEREBRAS_API_KEY",
            Self::Together => "TOGETHER_API_KEY",
            Self::Bedrock => "AWS_BEARER_TOKEN_BEDROCK",
            Self::AzureOpenAI => "AZURE_OPENAI_API_KEY",
            Self::OpenAI => "OPENAI_API_KEY",
            Self::Anthropic => "ANTHROPIC_API_KEY",
            Self::Gemini => "GEMINI_API_KEY",
            Self::OpenRouter => "OPENROUTER_API_KEY",
            Self::LocalSilicon => "",
            Self::Custom(_) => "CUSTOM_API_KEY",
        }
    }

    /// Default prompt caching discount for this provider.
    ///
    /// Returns the fraction saved on cached input tokens:
    /// e.g. 0.74 means cached tokens cost 26% of normal price (a 74% discount, like DeepSeek V3 \$0.07 vs \$0.27).
    #[inline]
    pub fn default_cache_discount_ratio(&self) -> f64 {
        match self {
            Self::DeepSeek => 0.7407,      // $0.07 / $0.27 -> ~74% discount
            Self::AlibabaDashScope => 0.8000, // DashScope context caching up to 80% discount ($0.04 vs $0.20)
            Self::MoonshotKimi => 0.7500,  // Kimi prompt caching 75% discount
            Self::ZhipuGLM => 0.5000,      // GLM prompt caching 50% discount
            Self::MiniMax => 0.5000,       // MiniMax context cache 50% discount
            Self::SiliconFlow => 0.5000,   // SiliconFlow prompt cache ~50% discount
            Self::Anthropic => 0.9000,     // $0.30 / $3.00 -> 90% discount on cache read
            Self::Gemini => 0.5000,        // 50% discount on prompt caching
            Self::OpenAI => 0.5000,        // 50% discount on cached input
            Self::AzureOpenAI => 0.5000,
            _ => 0.0,
        }
    }

    /// Computes the effective FinOps cost in USD taking prompt cache hits into account.
    ///
    /// # Arguments
    /// * `input_tokens` - Total input prompt tokens.
    /// * `cached_input_tokens` - Tokens hitting the prompt cache.
    /// * `output_tokens` - Generated completion tokens.
    /// * `cost_input_per_m` - List price per 1M input tokens.
    /// * `cost_output_per_m` - List price per 1M output tokens.
    pub fn calculate_effective_cost(
        &self,
        input_tokens: u32,
        cached_input_tokens: u32,
        output_tokens: u32,
        cost_input_per_m: f64,
        cost_output_per_m: f64,
    ) -> f64 {
        if *self == Self::LocalSilicon {
            return 0.0;
        }

        let cached_count = cached_input_tokens.min(input_tokens);
        let uncached_count = input_tokens.saturating_sub(cached_count);

        let discount_ratio = self.default_cache_discount_ratio();
        let cached_rate = cost_input_per_m * (1.0 - discount_ratio);

        let cost_uncached = (uncached_count as f64 / 1_000_000.0) * cost_input_per_m;
        let cost_cached = (cached_count as f64 / 1_000_000.0) * cached_rate;
        let cost_output = (output_tokens as f64 / 1_000_000.0) * cost_output_per_m;

        cost_uncached + cost_cached + cost_output
    }

    /// Latency factor relative to standard cloud baselines.
    ///
    /// Specialized architectures (Cerebras, Groq, SiliconFlow) have sub-unit factors (< 1.0).
    #[inline]
    pub fn latency_scaling_factor(&self) -> f64 {
        match self {
            Self::Cerebras => 0.30,  // Wafer scale ultra-fast
            Self::Groq => 0.35,      // ~3x faster time-to-first-token & generation
            Self::LocalSilicon => 0.80,
            Self::SiliconFlow => 0.90, // Multi-tenant fast accelerator cluster
            Self::AlibabaDashScope => 1.00,
            Self::ZhipuGLM => 1.05,
            Self::MiniMax => 1.05,
            Self::MoonshotKimi => 1.10,
            Self::DeepSeek => 1.15,  // High demand concurrency fluctuations
            _ => 1.0,
        }
    }
}

/// Normalized model specification for global and Chinese frontier models.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FrontierModelSpec {
    pub model_id: String,
    pub display_name: String,
    pub context_window: u32,
    pub cost_input_per_m: f64,
    pub cost_output_per_m: f64,
    pub supports_cache: bool,
    pub supports_tools: bool,
    pub recommended_task: String,
    pub score_lmarena: u32,
}

impl FrontierModelSpec {
    /// Converts frontier model specification into canonical `ModelRecord` for `model_registry`.
    pub fn to_model_record(&self, provider: &CloudProvider) -> crate::priors::ModelRecord {
        crate::priors::ModelRecord {
            model_id: self.model_id.clone(),
            provider_type: "CLOUD".to_string(),
            vram_base_mb: 0,
            kv_cache_cost_per_k: 0,
            max_context_window: self.context_window,
            specialty_tags: format!("[\"{}\"]", self.recommended_task),
            score_lmarena: self.score_lmarena,
            cost_input_per_m: self.cost_input_per_m,
            cost_output_per_m: self.cost_output_per_m,
            is_active: true,
            ema_latency_ms: (provider.latency_scaling_factor() * 400.0).round() as u32,
            success_rate_ema: 1.0,
        }
    }
}

/// Frontier provider specification including endpoint, authentication env var and default models.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FrontierProviderSpec {
    pub provider: CloudProvider,
    pub name: String,
    pub base_url: String,
    pub auth_header_env: String,
    pub default_cache_discount: f64,
    pub latency_factor: f64,
    pub models: Vec<FrontierModelSpec>,
}

/// Returns the canonical catalog of global and Chinese frontier providers and models.
pub fn get_canonical_frontier_providers() -> Vec<FrontierProviderSpec> {
    vec![
        // 1. Alibaba Cloud DashScope
        FrontierProviderSpec {
            provider: CloudProvider::AlibabaDashScope,
            name: CloudProvider::AlibabaDashScope.display_name().to_string(),
            base_url: CloudProvider::AlibabaDashScope.default_base_url().to_string(),
            auth_header_env: CloudProvider::AlibabaDashScope.auth_header_env().to_string(),
            default_cache_discount: CloudProvider::AlibabaDashScope.default_cache_discount_ratio(),
            latency_factor: CloudProvider::AlibabaDashScope.latency_scaling_factor(),
            models: vec![
                FrontierModelSpec {
                    model_id: "qwen2.5-coder-32b-instruct".to_string(),
                    display_name: "Qwen 2.5 Coder 32B Instruct".to_string(),
                    context_window: 131_072,
                    cost_input_per_m: 0.20,
                    cost_output_per_m: 0.60,
                    supports_cache: true,
                    supports_tools: true,
                    recommended_task: "code".to_string(),
                    score_lmarena: 1290,
                },
                FrontierModelSpec {
                    model_id: "qwen-max".to_string(),
                    display_name: "Qwen Max".to_string(),
                    context_window: 32_768,
                    cost_input_per_m: 2.80,
                    cost_output_per_m: 8.40,
                    supports_cache: true,
                    supports_tools: true,
                    recommended_task: "agentic".to_string(),
                    score_lmarena: 1330,
                },
                FrontierModelSpec {
                    model_id: "qwen-plus".to_string(),
                    display_name: "Qwen Plus".to_string(),
                    context_window: 131_072,
                    cost_input_per_m: 0.40,
                    cost_output_per_m: 1.20,
                    supports_cache: true,
                    supports_tools: true,
                    recommended_task: "general".to_string(),
                    score_lmarena: 1260,
                },
            ],
        },
        // 2. Moonshot AI (Kimi)
        FrontierProviderSpec {
            provider: CloudProvider::MoonshotKimi,
            name: CloudProvider::MoonshotKimi.display_name().to_string(),
            base_url: CloudProvider::MoonshotKimi.default_base_url().to_string(),
            auth_header_env: CloudProvider::MoonshotKimi.auth_header_env().to_string(),
            default_cache_discount: CloudProvider::MoonshotKimi.default_cache_discount_ratio(),
            latency_factor: CloudProvider::MoonshotKimi.latency_scaling_factor(),
            models: vec![
                FrontierModelSpec {
                    model_id: "moonshot-v1-128k".to_string(),
                    display_name: "Moonshot Kimi v1 128K".to_string(),
                    context_window: 128_000,
                    cost_input_per_m: 1.50,
                    cost_output_per_m: 1.50,
                    supports_cache: true,
                    supports_tools: true,
                    recommended_task: "research".to_string(),
                    score_lmarena: 1240,
                },
                FrontierModelSpec {
                    model_id: "kimi-latest".to_string(),
                    display_name: "Kimi Latest".to_string(),
                    context_window: 128_000,
                    cost_input_per_m: 1.00,
                    cost_output_per_m: 2.00,
                    supports_cache: true,
                    supports_tools: true,
                    recommended_task: "reasoning".to_string(),
                    score_lmarena: 1300,
                },
            ],
        },
        // 3. Zhipu AI (GLM)
        FrontierProviderSpec {
            provider: CloudProvider::ZhipuGLM,
            name: CloudProvider::ZhipuGLM.display_name().to_string(),
            base_url: CloudProvider::ZhipuGLM.default_base_url().to_string(),
            auth_header_env: CloudProvider::ZhipuGLM.auth_header_env().to_string(),
            default_cache_discount: CloudProvider::ZhipuGLM.default_cache_discount_ratio(),
            latency_factor: CloudProvider::ZhipuGLM.latency_scaling_factor(),
            models: vec![
                FrontierModelSpec {
                    model_id: "glm-4-plus".to_string(),
                    display_name: "GLM-4 Plus".to_string(),
                    context_window: 128_000,
                    cost_input_per_m: 1.40,
                    cost_output_per_m: 1.40,
                    supports_cache: true,
                    supports_tools: true,
                    recommended_task: "agentic".to_string(),
                    score_lmarena: 1280,
                },
                FrontierModelSpec {
                    model_id: "glm-4-air".to_string(),
                    display_name: "GLM-4 Air".to_string(),
                    context_window: 128_000,
                    cost_input_per_m: 0.14,
                    cost_output_per_m: 0.14,
                    supports_cache: true,
                    supports_tools: true,
                    recommended_task: "fast".to_string(),
                    score_lmarena: 1210,
                },
            ],
        },
        // 4. MiniMax
        FrontierProviderSpec {
            provider: CloudProvider::MiniMax,
            name: CloudProvider::MiniMax.display_name().to_string(),
            base_url: CloudProvider::MiniMax.default_base_url().to_string(),
            auth_header_env: CloudProvider::MiniMax.auth_header_env().to_string(),
            default_cache_discount: CloudProvider::MiniMax.default_cache_discount_ratio(),
            latency_factor: CloudProvider::MiniMax.latency_scaling_factor(),
            models: vec![
                FrontierModelSpec {
                    model_id: "abab6.5s-chat".to_string(),
                    display_name: "MiniMax ABAB 6.5s Chat".to_string(),
                    context_window: 245_760,
                    cost_input_per_m: 0.14,
                    cost_output_per_m: 0.14,
                    supports_cache: true,
                    supports_tools: true,
                    recommended_task: "fast".to_string(),
                    score_lmarena: 1220,
                },
            ],
        },
        // 5. SiliconFlow
        FrontierProviderSpec {
            provider: CloudProvider::SiliconFlow,
            name: CloudProvider::SiliconFlow.display_name().to_string(),
            base_url: CloudProvider::SiliconFlow.default_base_url().to_string(),
            auth_header_env: CloudProvider::SiliconFlow.auth_header_env().to_string(),
            default_cache_discount: CloudProvider::SiliconFlow.default_cache_discount_ratio(),
            latency_factor: CloudProvider::SiliconFlow.latency_scaling_factor(),
            models: vec![
                FrontierModelSpec {
                    model_id: "deepseek-ai/DeepSeek-V3".to_string(),
                    display_name: "SiliconFlow DeepSeek-V3".to_string(),
                    context_window: 64_000,
                    cost_input_per_m: 0.27,
                    cost_output_per_m: 1.10,
                    supports_cache: true,
                    supports_tools: true,
                    recommended_task: "code".to_string(),
                    score_lmarena: 1320,
                },
                FrontierModelSpec {
                    model_id: "Qwen/Qwen2.5-Coder-32B-Instruct".to_string(),
                    display_name: "SiliconFlow Qwen 2.5 Coder 32B".to_string(),
                    context_window: 32_768,
                    cost_input_per_m: 0.20,
                    cost_output_per_m: 0.60,
                    supports_cache: true,
                    supports_tools: true,
                    recommended_task: "code".to_string(),
                    score_lmarena: 1290,
                },
            ],
        },
        // 6. DeepSeek Official
        FrontierProviderSpec {
            provider: CloudProvider::DeepSeek,
            name: CloudProvider::DeepSeek.display_name().to_string(),
            base_url: CloudProvider::DeepSeek.default_base_url().to_string(),
            auth_header_env: CloudProvider::DeepSeek.auth_header_env().to_string(),
            default_cache_discount: CloudProvider::DeepSeek.default_cache_discount_ratio(),
            latency_factor: CloudProvider::DeepSeek.latency_scaling_factor(),
            models: vec![
                FrontierModelSpec {
                    model_id: "deepseek-chat".to_string(),
                    display_name: "DeepSeek Chat (V3)".to_string(),
                    context_window: 64_000,
                    cost_input_per_m: 0.27,
                    cost_output_per_m: 1.10,
                    supports_cache: true,
                    supports_tools: true,
                    recommended_task: "code".to_string(),
                    score_lmarena: 1320,
                },
                FrontierModelSpec {
                    model_id: "deepseek-reasoner".to_string(),
                    display_name: "DeepSeek Reasoner (R1)".to_string(),
                    context_window: 64_000,
                    cost_input_per_m: 0.55,
                    cost_output_per_m: 2.19,
                    supports_cache: true,
                    supports_tools: false,
                    recommended_task: "reasoning".to_string(),
                    score_lmarena: 1360,
                },
            ],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_inference_from_model_id() {
        assert_eq!(
            CloudProvider::from_model_id_and_type("deepseek/deepseek-chat", "CLOUD"),
            CloudProvider::DeepSeek
        );
        assert_eq!(
            CloudProvider::from_model_id_and_type("groq/llama-3.3-70b-versatile", "CLOUD"),
            CloudProvider::Groq
        );
        assert_eq!(
            CloudProvider::from_model_id_and_type("cerebras/llama3.1-70b", "CLOUD"),
            CloudProvider::Cerebras
        );
        assert_eq!(
            CloudProvider::from_model_id_and_type("bedrock/anthropic.claude-3-5-sonnet", "CLOUD"),
            CloudProvider::Bedrock
        );
        assert_eq!(
            CloudProvider::from_model_id_and_type("tier1_qwen_coder_gpu", "LOCAL"),
            CloudProvider::LocalSilicon
        );
        // Chinese frontier providers
        assert_eq!(
            CloudProvider::from_model_id_and_type("dashscope/qwen2.5-coder-32b-instruct", "CLOUD"),
            CloudProvider::AlibabaDashScope
        );
        assert_eq!(
            CloudProvider::from_model_id_and_type("moonshot/moonshot-v1-128k", "CLOUD"),
            CloudProvider::MoonshotKimi
        );
        assert_eq!(
            CloudProvider::from_model_id_and_type("zhipu/glm-4-plus", "CLOUD"),
            CloudProvider::ZhipuGLM
        );
        assert_eq!(
            CloudProvider::from_model_id_and_type("minimax/abab6.5s-chat", "CLOUD"),
            CloudProvider::MiniMax
        );
        assert_eq!(
            CloudProvider::from_model_id_and_type("siliconflow/deepseek-ai/DeepSeek-V3", "CLOUD"),
            CloudProvider::SiliconFlow
        );
    }

    #[test]
    fn test_provider_inference_from_name() {
        assert_eq!(CloudProvider::from_provider_name("dashscope"), Some(CloudProvider::AlibabaDashScope));
        assert_eq!(CloudProvider::from_provider_name("kimi"), Some(CloudProvider::MoonshotKimi));
        assert_eq!(CloudProvider::from_provider_name("zhipu"), Some(CloudProvider::ZhipuGLM));
        assert_eq!(CloudProvider::from_provider_name("minimax"), Some(CloudProvider::MiniMax));
        assert_eq!(CloudProvider::from_provider_name("siliconflow"), Some(CloudProvider::SiliconFlow));
        assert_eq!(CloudProvider::from_provider_name("deepseek"), Some(CloudProvider::DeepSeek));
    }

    #[test]
    fn test_effective_cost_with_dashscope_context_caching() {
        let provider = CloudProvider::AlibabaDashScope;
        let total_in = 1_000_000;
        let cached_in = 800_000; // 80% cache hit
        let total_out = 100_000;

        let cost_in_per_m = 0.20;
        let cost_out_per_m = 0.60;

        let cost = provider.calculate_effective_cost(
            total_in,
            cached_in,
            total_out,
            cost_in_per_m,
            cost_out_per_m,
        );

        // Uncached: 200k * 0.20 / 1M = $0.040
        // Cached: 800k * (0.20 * (1 - 0.80)) / 1M = 800k * 0.04 / 1M = $0.032
        // Output: 100k * 0.60 / 1M = $0.060
        // Total expected = 0.040 + 0.032 + 0.060 = $0.132
        assert!((cost - 0.132).abs() < 1e-4, "Expected cost 0.132, got {}", cost);
    }

    #[test]
    fn test_effective_cost_with_deepseek_prompt_caching() {
        let provider = CloudProvider::DeepSeek;
        let total_in = 1_000_000;
        let cached_in = 800_000; // 80% cache hit
        let total_out = 100_000;

        let cost_in_per_m = 0.27;
        let cost_out_per_m = 1.10;

        let cost = provider.calculate_effective_cost(
            total_in,
            cached_in,
            total_out,
            cost_in_per_m,
            cost_out_per_m,
        );

        // Uncached: 200k * 0.27 / 1M = $0.054
        // Cached: 800k * (0.27 * (1 - 0.7407)) / 1M ≈ 800k * 0.07 / 1M = $0.056
        // Output: 100k * 1.10 / 1M = $0.110
        // Total expected ≈ $0.220
        assert!(cost < 0.25 && cost > 0.20, "Expected cost ~0.22, got {}", cost);

        // Zero-cache comparison
        let zero_cache_cost = provider.calculate_effective_cost(
            total_in,
            0,
            total_out,
            cost_in_per_m,
            cost_out_per_m,
        );
        // 1M * 0.27 + 100k * 1.10 = 0.27 + 0.11 = $0.38
        assert!((zero_cache_cost - 0.38).abs() < 1e-4);
        assert!(cost < zero_cache_cost);
    }

    #[test]
    fn test_canonical_frontier_providers_specs() {
        let providers = get_canonical_frontier_providers();
        assert_eq!(providers.len(), 6);

        let dashscope = providers.iter().find(|p| p.provider == CloudProvider::AlibabaDashScope).unwrap();
        assert_eq!(dashscope.auth_header_env, "DASHSCOPE_API_KEY");
        assert_eq!(dashscope.base_url, "https://dashscope.aliyuncs.com/compatible-mode/v1");
        assert_eq!(dashscope.models.len(), 3);

        let qwen_spec = &dashscope.models[0];
        let record = qwen_spec.to_model_record(&dashscope.provider);
        assert_eq!(record.model_id, "qwen2.5-coder-32b-instruct");
        assert_eq!(record.cost_input_per_m, 0.20);
        assert_eq!(record.provider_type, "CLOUD");
        assert!(record.is_active);
    }
}

