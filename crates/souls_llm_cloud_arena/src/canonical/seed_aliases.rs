//! # seed_aliases.rs — Catálogo Inicial de Modelos Canônicos e Aliases
//!
//! Seed dos 16+ modelos líderes de mercado e seus aliases heterogêneos.

use crate::domain::CanonicalModel;

pub struct ModelSeed {
    pub model: CanonicalModel,
    pub aliases: Vec<(&'static str, &'static str)>, // (alias, source)
}

pub fn get_initial_seed_models() -> Vec<ModelSeed> {
    vec![
        ModelSeed {
            model: CanonicalModel {
                id: "claude-3-7-sonnet".to_string(),
                display_name: "Claude 3.7 Sonnet".to_string(),
                family: "Claude".to_string(),
                developer: "Anthropic".to_string(),
                context_window: 200_000,
                supports_tools: true,
                supports_structured_outputs: true,
                supports_caching: true,
                is_multimodal: true,
                created_at: "2026-10-12T00:00:00Z".to_string(),
            },
            aliases: vec![
                ("anthropic/claude-3-7-sonnet", "openrouter"),
                ("anthropic/claude-3-7-sonnet:beta", "openrouter"),
                ("anthropic/claude-3.7-sonnet", "openrouter"),
                ("claude-3-7-sonnet", "nous_portal"),
            ],
        },
        ModelSeed {
            model: CanonicalModel {
                id: "claude-3-5-sonnet".to_string(),
                display_name: "Claude 3.5 Sonnet".to_string(),
                family: "Claude".to_string(),
                developer: "Anthropic".to_string(),
                context_window: 200_000,
                supports_tools: true,
                supports_structured_outputs: true,
                supports_caching: true,
                is_multimodal: true,
                created_at: "2026-10-12T00:00:00Z".to_string(),
            },
            aliases: vec![
                ("anthropic/claude-3-5-sonnet", "openrouter"),
                ("anthropic/claude-3.5-sonnet", "openrouter"),
                ("anthropic/claude-3-5-sonnet:beta", "openrouter"),
            ],
        },
        ModelSeed {
            model: CanonicalModel {
                id: "deepseek-v3".to_string(),
                display_name: "DeepSeek V3".to_string(),
                family: "DeepSeek".to_string(),
                developer: "DeepSeek".to_string(),
                context_window: 64_000,
                supports_tools: true,
                supports_structured_outputs: true,
                supports_caching: true,
                is_multimodal: false,
                created_at: "2026-10-12T00:00:00Z".to_string(),
            },
            aliases: vec![
                ("deepseek/deepseek-chat", "openrouter"),
                ("deepseek/deepseek-v3", "openrouter"),
                ("deepseek-v3", "nous_portal"),
                ("deepseek/deepseek-v3", "nous_portal"),
            ],
        },
        ModelSeed {
            model: CanonicalModel {
                id: "deepseek-r1".to_string(),
                display_name: "DeepSeek R1".to_string(),
                family: "DeepSeek".to_string(),
                developer: "DeepSeek".to_string(),
                context_window: 64_000,
                supports_tools: true,
                supports_structured_outputs: true,
                supports_caching: true,
                is_multimodal: false,
                created_at: "2026-10-12T00:00:00Z".to_string(),
            },
            aliases: vec![
                ("deepseek/deepseek-r1", "openrouter"),
                ("deepseek/deepseek-reasoner", "openrouter"),
                ("deepseek-r1", "nous_portal"),
                ("deepseek/deepseek-r1", "nous_portal"),
            ],
        },
        ModelSeed {
            model: CanonicalModel {
                id: "qwen-2.5-coder-32b".to_string(),
                display_name: "Qwen 2.5 Coder 32B".to_string(),
                family: "Qwen".to_string(),
                developer: "Alibaba".to_string(),
                context_window: 131_072,
                supports_tools: true,
                supports_structured_outputs: true,
                supports_caching: true,
                is_multimodal: false,
                created_at: "2026-10-12T00:00:00Z".to_string(),
            },
            aliases: vec![
                ("qwen/qwen-2.5-coder-32b-instruct", "openrouter"),
                ("qwen/qwen-2.5-coder-32b", "openrouter"),
                ("qwen-2.5-coder-32b", "nous_portal"),
            ],
        },
        ModelSeed {
            model: CanonicalModel {
                id: "gpt-4o".to_string(),
                display_name: "GPT-4o".to_string(),
                family: "GPT".to_string(),
                developer: "OpenAI".to_string(),
                context_window: 128_000,
                supports_tools: true,
                supports_structured_outputs: true,
                supports_caching: true,
                is_multimodal: true,
                created_at: "2026-10-12T00:00:00Z".to_string(),
            },
            aliases: vec![
                ("openai/gpt-4o", "openrouter"),
                ("gpt-4o", "openrouter"),
            ],
        },
        ModelSeed {
            model: CanonicalModel {
                id: "gpt-4o-mini".to_string(),
                display_name: "GPT-4o Mini".to_string(),
                family: "GPT".to_string(),
                developer: "OpenAI".to_string(),
                context_window: 128_000,
                supports_tools: true,
                supports_structured_outputs: true,
                supports_caching: true,
                is_multimodal: true,
                created_at: "2026-10-12T00:00:00Z".to_string(),
            },
            aliases: vec![
                ("openai/gpt-4o-mini", "openrouter"),
                ("gpt-4o-mini", "openrouter"),
            ],
        },
        ModelSeed {
            model: CanonicalModel {
                id: "o3-mini".to_string(),
                display_name: "OpenAI o3-mini".to_string(),
                family: "o-series".to_string(),
                developer: "OpenAI".to_string(),
                context_window: 200_000,
                supports_tools: true,
                supports_structured_outputs: true,
                supports_caching: true,
                is_multimodal: false,
                created_at: "2026-10-12T00:00:00Z".to_string(),
            },
            aliases: vec![
                ("openai/o3-mini", "openrouter"),
                ("o3-mini", "openrouter"),
            ],
        },
        ModelSeed {
            model: CanonicalModel {
                id: "llama-3.3-70b".to_string(),
                display_name: "Llama 3.3 70B Instruct".to_string(),
                family: "Llama".to_string(),
                developer: "Meta".to_string(),
                context_window: 128_000,
                supports_tools: true,
                supports_structured_outputs: true,
                supports_caching: false,
                is_multimodal: false,
                created_at: "2026-10-12T00:00:00Z".to_string(),
            },
            aliases: vec![
                ("meta-llama/llama-3.3-70b-instruct", "openrouter"),
                ("meta-llama/llama-3.3-70b", "openrouter"),
            ],
        },
        ModelSeed {
            model: CanonicalModel {
                id: "hermes-3-llama-3.1-405b".to_string(),
                display_name: "Hermes 3 (Llama 3.1 405B)".to_string(),
                family: "Hermes".to_string(),
                developer: "Nous Research".to_string(),
                context_window: 128_000,
                supports_tools: true,
                supports_structured_outputs: true,
                supports_caching: true,
                is_multimodal: false,
                created_at: "2026-10-12T00:00:00Z".to_string(),
            },
            aliases: vec![
                ("nousresearch/hermes-3-llama-3.1-405b", "openrouter"),
                ("nousresearch/hermes-3-llama-3.1-405b", "nous_portal"),
                ("hermes-3-llama-3.1-405b", "nous_portal"),
            ],
        },
        ModelSeed {
            model: CanonicalModel {
                id: "hermes-3-llama-3.1-70b".to_string(),
                display_name: "Hermes 3 (Llama 3.1 70B)".to_string(),
                family: "Hermes".to_string(),
                developer: "Nous Research".to_string(),
                context_window: 128_000,
                supports_tools: true,
                supports_structured_outputs: true,
                supports_caching: true,
                is_multimodal: false,
                created_at: "2026-10-12T00:00:00Z".to_string(),
            },
            aliases: vec![
                ("nousresearch/hermes-3-llama-3.1-70b", "openrouter"),
                ("nousresearch/hermes-3-llama-3.1-70b", "nous_portal"),
                ("hermes-3-llama-3.1-70b", "nous_portal"),
            ],
        },
        ModelSeed {
            model: CanonicalModel {
                id: "gemini-2.0-flash".to_string(),
                display_name: "Gemini 2.0 Flash".to_string(),
                family: "Gemini".to_string(),
                developer: "Google".to_string(),
                context_window: 1_000_000,
                supports_tools: true,
                supports_structured_outputs: true,
                supports_caching: true,
                is_multimodal: true,
                created_at: "2026-10-12T00:00:00Z".to_string(),
            },
            aliases: vec![
                ("google/gemini-2.0-flash-001", "openrouter"),
                ("google/gemini-2.0-flash", "openrouter"),
            ],
        },
    ]
}
