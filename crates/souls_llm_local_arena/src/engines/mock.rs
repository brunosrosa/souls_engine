//! Synthetic Mock Engine for testing and hermetic test runs.

use async_trait::async_trait;
use crate::domain::engine::{ArenaInferEngine, EngineType, InferenceRunParams, RawInferenceOutput};
use crate::domain::model::ModelDescriptor;
use crate::error::ArenaError;

pub struct MockInferEngine;

#[async_trait]
impl ArenaInferEngine for MockInferEngine {
    fn engine_type(&self) -> EngineType {
        EngineType::Mock
    }

    async fn execute(
        &self,
        descriptor: &ModelDescriptor,
        params: &InferenceRunParams,
    ) -> Result<RawInferenceOutput, ArenaError> {
        let simulated_tps = if descriptor.tier == crate::domain::model::ModelTier::Tier0Bootstrap {
            110.0
        } else if descriptor.tier == crate::domain::model::ModelTier::Tier05Epistemic {
            68.0
        } else if descriptor.tier == crate::domain::model::ModelTier::Tier1Master {
            42.5
        } else {
            18.0
        };

        let response_text = if params.json_schema.is_some() {
            r#"{"action": "refactor", "target": "crates/souls_ast", "confidence": 0.98}"#.to_string()
        } else if params.prompt.contains("Rust") {
            "pub enum State { Idle, Running }\npub struct Task<'a> { name: &'a str }\nimpl<'a> std::fmt::Display for Task<'a> { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, \"{}\", self.name) } }".to_string()
        } else {
            "<think>Analyzing physical WDDM memory constraints on Windows 11.</think>WDDM reserves up to 850 MB for the Desktop Window Manager compositor swapchains.".to_string()
        };

        Ok(RawInferenceOutput {
            text: response_text,
            prompt_tokens: 32,
            completion_tokens: 64,
            cold_load_ms: 25.0,
            ttft_ms: 45.0,
            tpot_ms: 1000.0 / simulated_tps,
            tps: simulated_tps,
            peak_vram_mb: if descriptor.tier == crate::domain::model::ModelTier::Tier1Master { 3120.0 } else { 0.0 },
            ram_mb: 280.0,
        })
    }
}
