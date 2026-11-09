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

        if lower.contains("deepseek") {
            Self::DeepSeek
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

    /// Default prompt caching discount for this provider.
    ///
    /// Returns the fraction saved on cached input tokens:
    /// e.g. 0.74 means cached tokens cost 26% of normal price (a 74% discount, like DeepSeek V3 \$0.07 vs \$0.27).
    #[inline]
    pub fn default_cache_discount_ratio(&self) -> f64 {
        match self {
            Self::DeepSeek => 0.7407,  // $0.07 / $0.27 -> ~74% discount
            Self::Anthropic => 0.9000, // $0.30 / $3.00 -> 90% discount on cache read
            Self::Gemini => 0.5000,    // 50% discount on prompt caching
            Self::OpenAI => 0.5000,    // 50% discount on cached input
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
    /// Groq and Cerebras have sub-unit factors (< 1.0) because of specialized silicon.
    #[inline]
    pub fn latency_scaling_factor(&self) -> f64 {
        match self {
            Self::Groq => 0.35,      // ~3x faster time-to-first-token & generation
            Self::Cerebras => 0.30,  // Wafer scale ultra-fast
            Self::LocalSilicon => 0.80,
            Self::DeepSeek => 1.15,  // High demand concurrency fluctuations
            _ => 1.0,
        }
    }
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
}

