//! # souls_inference_runtime (Souls Engine v7)
//!
//! Bare-metal silicon execution and inference governance layer:
//! - Tier 0: ONNX Runtime on CPU AVX2 (GLiClass Multilang Ultra / ModernBERT) with 0 MB VRAM.
//! - Tier 0.5: Logit Probing probability engine (Gemma-4 / Phi-4-mini) on CPU Host with 0 MB VRAM.
//! - Tier 1: Upstream official llama.cpp engine on dGPU RTX 2060m with Asymmetric KV Cache (Key: FP16, Value: Q4_0).
//! - Hardware Watchdog: Active thermodynamic monitoring via NVML and EWMA thermal prediction (ADR-004).
//! - GGUF Parser: Zero-copy O(1) header inspection via `memmap2`.
//! - Response Healing: Deterministic stack-based JSON repair pipeline.

pub mod error;
pub mod gguf_mmap;
pub mod healing;
pub mod inference;
pub mod llamacpp;
pub mod nvml;
pub mod onnx;

// Re-export canonical types and functions
pub use error::InferenceError;
pub use gguf_mmap::{
    inspect_gguf_metadata_o1, parse_gguf_slice, BoundedGgufPool, GgufMappedReader,
    GgufMetadataInfo, MAX_MAPPED_HANDLES,
};
pub use healing::heal_json_response;
pub use inference::{
    get_or_compile_grammar, grammar_cache, CompiledGrammar, GenOutput, GenParams,
    KvCacheConfig, KvCacheType, Tier1GenerativeEngine,
};
pub use llamacpp::{
    LlamaBatchLayout, LlamaLogitProber, LogitProbingOutput,
    CANONICAL_VOCAB_PROJECTION_SIZE, ENTROPY_CIRCUIT_BREAKER_THRESHOLD,
};
pub use nvml::{
    compute_thermodynamic_barrier, evaluate_operational_zone, GpuTelemetrySnapshot,
    HardwareWatchdog, OperationalZone, RTX_2060M_VRAM_TOTAL_MB,
};
pub use onnx::{
    ClassificationLabel, ClassificationOutput, OrtClassifierEngine, OrtSessionBuilder,
    OrtSessionConfig, MAX_TRIAGE_CHARS,
};

use std::path::Path;
use std::sync::{Arc, Mutex};

/// Unified Facade for the Souls Inference Runtime.
pub struct InferenceRuntime {
    watchdog: Arc<Mutex<HardwareWatchdog>>,
    classifier: &'static OrtClassifierEngine,
    prober: LlamaLogitProber,
    tier1: Tier1GenerativeEngine,
    gguf_pool: Arc<BoundedGgufPool>,
}

impl InferenceRuntime {
    /// Creates a unified inference runtime with default configuration.
    pub fn new() -> Self {
        Self {
            watchdog: Arc::new(Mutex::new(HardwareWatchdog::new())),
            classifier: OrtClassifierEngine::global(),
            prober: LlamaLogitProber::new(),
            tier1: Tier1GenerativeEngine::new(),
            gguf_pool: Arc::new(BoundedGgufPool::new()),
        }
    }

    /// Obtains a clone of the hardware watchdog mutex.
    pub fn watchdog(&self) -> Arc<Mutex<HardwareWatchdog>> {
        Arc::clone(&self.watchdog)
    }

    /// Obtains a clone of the bounded GGUF handle pool.
    pub fn gguf_pool(&self) -> Arc<BoundedGgufPool> {
        Arc::clone(&self.gguf_pool)
    }

    /// Tier 0 Classification (CPU Host AVX2, 0 MB VRAM).
    pub async fn infer_tier0_classify(
        &self,
        input: &str,
    ) -> Result<ClassificationOutput, InferenceError> {
        let candidates = ["code", "chat", "security", "memory", "general"];
        self.classifier.classify(input, &candidates)
    }

    /// Tier 0.5 Logit Probing (CPU Host AVX2, 0 MB VRAM).
    pub async fn infer_tier05_probe(
        &self,
        prompt: &str,
    ) -> Result<LogitProbingOutput, InferenceError> {
        self.prober.probe_logits(prompt)
    }

    /// Tier 1 Generation (dGPU RTX 2060m with Asymmetric KV Cache).
    pub async fn infer_tier1_generate(
        &self,
        prompt: &str,
        params: &GenParams,
    ) -> Result<String, InferenceError> {
        let mut wd = self.watchdog.lock().map_err(|_| {
            InferenceError::OnnxError("Watchdog lock poisoned".to_string())
        })?;

        let res = self.tier1.generate(prompt, params, &mut wd)?;
        Ok(res.text)
    }

    /// Zero-copy O(1) GGUF metadata inspection backed by the bounded handle pool.
    pub fn inspect_gguf(&self, path: &Path) -> Result<GgufMetadataInfo, InferenceError> {
        self.gguf_pool.inspect_metadata(path)
    }

    /// In-flight JSON healing pipeline.
    pub fn heal_json(&self, raw: &str) -> Result<serde_json::Value, InferenceError> {
        heal_json_response(raw)
    }
}

impl Default for InferenceRuntime {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_unified_inference_runtime_tier0() {
        let runtime = InferenceRuntime::new();
        let res = runtime
            .infer_tier0_classify("fn compute_entropy() -> f32")
            .await
            .expect("Tier 0 classification failed");

        assert_eq!(res.top_label, "code");
        assert_eq!(res.vram_mb, 0);
    }

    #[tokio::test]
    async fn test_unified_inference_runtime_tier05() {
        let runtime = InferenceRuntime::new();
        let res = runtime
            .infer_tier05_probe("Evaluate concurrency lock contention")
            .await
            .expect("Tier 0.5 probing failed");

        assert_eq!(res.vram_mb, 0);
        assert!(res.latency_ms < 150.0);
    }

    #[tokio::test]
    async fn test_unified_inference_runtime_tier1() {
        let runtime = InferenceRuntime::new();
        let params = GenParams::default();
        let res = runtime
            .infer_tier1_generate("Generate a short answer", &params)
            .await
            .expect("Tier 1 generation failed");

        assert!(!res.is_empty());
    }
}
