//! # seed_aliases.rs — Catálogo Inicial de Modelos Canônicos Contemporâneos e Aliases
//!
//! Modelos líderes da Fronteira de 2026 e seus aliases canônicos no Nous Portal e OpenRouter.

use crate::domain::CanonicalModel;

pub struct ModelSeed {
    pub model: CanonicalModel,
    pub aliases: Vec<(&'static str, &'static str)>, // (alias, source)
}

pub fn get_initial_seed_models() -> Vec<ModelSeed> {
    vec![
        ModelSeed {
            model: CanonicalModel {
                id: "anthropic/claude-opus-5.5".to_string(),
                display_name: "Claude Opus 5.5".to_string(),
                family: "claude".to_string(),
                developer: "anthropic".to_string(),
                context_window: 1_000_000,
                supports_tools: true,
                supports_structured_outputs: true,
                supports_caching: true,
                is_multimodal: true,
                created_at: "2026-10-14T00:00:00Z".to_string(),
            },
            aliases: vec![
                ("claude-opus-5.5", "nous_portal"),
                ("~anthropic/claude-opus-5.5", "nous_router"),
            ],
        },
        ModelSeed {
            model: CanonicalModel {
                id: "anthropic/claude-sonnet-5.5".to_string(),
                display_name: "Claude Sonnet 5.5".to_string(),
                family: "claude".to_string(),
                developer: "anthropic".to_string(),
                context_window: 1_000_000,
                supports_tools: true,
                supports_structured_outputs: true,
                supports_caching: true,
                is_multimodal: true,
                created_at: "2026-10-14T00:00:00Z".to_string(),
            },
            aliases: vec![
                ("claude-sonnet-5.5", "nous_portal"),
                ("~anthropic/claude-sonnet-5.5", "nous_router"),
            ],
        },
        ModelSeed {
            model: CanonicalModel {
                id: "google/gemini-4-argon".to_string(),
                display_name: "Gemini 4 Argon".to_string(),
                family: "gemini".to_string(),
                developer: "google".to_string(),
                context_window: 1_048_576,
                supports_tools: true,
                supports_structured_outputs: true,
                supports_caching: true,
                is_multimodal: true,
                created_at: "2026-10-14T00:00:00Z".to_string(),
            },
            aliases: vec![
                ("gemini-4-argon", "nous_portal"),
                ("gemini-4-argon-high", "lmsys"),
            ],
        },
        ModelSeed {
            model: CanonicalModel {
                id: "openai/gpt-6-astra".to_string(),
                display_name: "GPT-6 Astra".to_string(),
                family: "gpt".to_string(),
                developer: "openai".to_string(),
                context_window: 1_050_000,
                supports_tools: true,
                supports_structured_outputs: true,
                supports_caching: true,
                is_multimodal: true,
                created_at: "2026-10-14T00:00:00Z".to_string(),
            },
            aliases: vec![
                ("gpt-6-astra", "nous_portal"),
                ("~openai/gpt-astra-latest", "nous_router"),
            ],
        },
        ModelSeed {
            model: CanonicalModel {
                id: "openai/gpt-6.1-sol-pro".to_string(),
                display_name: "GPT-6.1 Sol Pro".to_string(),
                family: "gpt".to_string(),
                developer: "openai".to_string(),
                context_window: 1_050_000,
                supports_tools: true,
                supports_structured_outputs: true,
                supports_caching: true,
                is_multimodal: true,
                created_at: "2026-10-14T00:00:00Z".to_string(),
            },
            aliases: vec![
                ("gpt-6.1-sol-pro", "nous_portal"),
                ("~openai/gpt-sol-latest", "nous_router"),
            ],
        },
        ModelSeed {
            model: CanonicalModel {
                id: "z-ai/glm-5.3-prime".to_string(),
                display_name: "GLM 5.3 Prime".to_string(),
                family: "glm".to_string(),
                developer: "z-ai".to_string(),
                context_window: 1_000_000,
                supports_tools: true,
                supports_structured_outputs: true,
                supports_caching: true,
                is_multimodal: true,
                created_at: "2026-10-14T00:00:00Z".to_string(),
            },
            aliases: vec![
                ("glm-5.3-prime", "nous_portal"),
                ("glm-5.3-max", "lmsys"),
            ],
        },
        ModelSeed {
            model: CanonicalModel {
                id: "qwen/qwen3.8-max-prime".to_string(),
                display_name: "Qwen 3.8 Max Prime".to_string(),
                family: "qwen".to_string(),
                developer: "qwen".to_string(),
                context_window: 1_000_000,
                supports_tools: true,
                supports_structured_outputs: true,
                supports_caching: true,
                is_multimodal: true,
                created_at: "2026-10-14T00:00:00Z".to_string(),
            },
            aliases: vec![
                ("qwen3.8-max-prime", "nous_portal"),
                ("qwen3.8-max", "lmsys"),
            ],
        },
        ModelSeed {
            model: CanonicalModel {
                id: "deepseek/deepseek-v4.1-flash".to_string(),
                display_name: "DeepSeek V4.1 Flash".to_string(),
                family: "deepseek".to_string(),
                developer: "deepseek".to_string(),
                context_window: 1_048_576,
                supports_tools: true,
                supports_structured_outputs: true,
                supports_caching: true,
                is_multimodal: false,
                created_at: "2026-10-14T00:00:00Z".to_string(),
            },
            aliases: vec![
                ("deepseek-v4.1-flash", "nous_portal"),
                ("~deepseek/deepseek-flash-latest", "nous_router"),
            ],
        },
        ModelSeed {
            model: CanonicalModel {
                id: "xiaomi/mimo-v2.6-pro".to_string(),
                display_name: "MiMo-V2.6-Pro".to_string(),
                family: "mimo".to_string(),
                developer: "xiaomi".to_string(),
                context_window: 1_050_000,
                supports_tools: true,
                supports_structured_outputs: true,
                supports_caching: true,
                is_multimodal: true,
                created_at: "2026-10-14T00:00:00Z".to_string(),
            },
            aliases: vec![
                ("mimo-v2.6-pro", "nous_portal"),
            ],
        },
        ModelSeed {
            model: CanonicalModel {
                id: "xiaomi/mimo-v2.6-flash".to_string(),
                display_name: "MiMo-V2.6-Flash".to_string(),
                family: "mimo".to_string(),
                developer: "xiaomi".to_string(),
                context_window: 1_050_000,
                supports_tools: true,
                supports_structured_outputs: true,
                supports_caching: true,
                is_multimodal: true,
                created_at: "2026-10-14T00:00:00Z".to_string(),
            },
            aliases: vec![
                ("mimo-v2.6-flash", "nous_portal"),
            ],
        },
    ]
}
