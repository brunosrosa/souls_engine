//! Abstractions for multi-engine execution in the Arena.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::domain::model::ModelDescriptor;
use crate::error::ArenaError;

/// Multi-Engine execution backend selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EngineType {
    /// Upstream official llama.cpp engine with CUDA acceleration on RTX 2060m.
    LlamaCppCuda,
    /// Upstream official llama.cpp engine running purely on CPU AVX2 (0 MB VRAM).
    LlamaCppCpu,
    /// ONNX Runtime engine on CPU AVX2.
    OnnxCpu,
    /// Mock engine for isolated regression tests and CI verification.
    Mock,
}

impl EngineType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EngineType::LlamaCppCuda => "llama-cuda",
            EngineType::LlamaCppCpu => "llama-cpu",
            EngineType::OnnxCpu => "onnx-cpu",
            EngineType::Mock => "mock",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "llama-cuda" | "cuda" | "gpu" => Some(EngineType::LlamaCppCuda),
            "llama-cpu" | "cpu" => Some(EngineType::LlamaCppCpu),
            "onnx-cpu" | "onnx" => Some(EngineType::OnnxCpu),
            "mock" | "test" => Some(EngineType::Mock),
            _ => None,
        }
    }
}

/// Parameters passed to the engine for benchmark evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceRunParams {
    pub prompt: String,
    pub system_prompt: Option<String>,
    pub max_tokens: u32,
    pub temperature: f32,
    pub top_p: f32,
    pub json_schema: Option<serde_json::Value>,
}

impl Default for InferenceRunParams {
    fn default() -> Self {
        Self {
            prompt: String::new(),
            system_prompt: None,
            max_tokens: 256,
            temperature: 0.1,
            top_p: 0.95,
            json_schema: None,
        }
    }
}

/// Raw execution output telemetry produced by the engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawInferenceOutput {
    pub text: String,
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub cold_load_ms: f64,
    pub ttft_ms: f64,
    pub tpot_ms: f64,
    pub tps: f64,
    pub peak_vram_mb: f64,
    pub ram_mb: f64,
}

/// Core trait implemented by all inference engines in the Arena.
#[async_trait]
pub trait ArenaInferEngine: Send + Sync {
    fn engine_type(&self) -> EngineType;

    async fn execute(
        &self,
        descriptor: &ModelDescriptor,
        params: &InferenceRunParams,
    ) -> Result<RawInferenceOutput, ArenaError>;
}
