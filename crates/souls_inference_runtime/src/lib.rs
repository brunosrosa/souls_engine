//! # souls_inference_runtime (Souls Engine v7)
//!
//! Bare-metal silicon execution and inference governance layer:
//! - Tier 0: ONNX Runtime on CPU AVX2 (GLiClass Multilang Ultra / ModernBERT) with 0 MB VRAM.
//! - Tier 0.5: Logit Probing probability engine (Gemma-4 / Phi-4-mini) on CPU Host with 0 MB VRAM.
//! - Tier 1: Upstream official llama.cpp engine on dGPU RTX 2060m with Asymmetric KV Cache (Key: FP16, Value: Q4_0).
//! - Hardware Watchdog: Active thermodynamic monitoring via NVML and EWMA thermal prediction (ADR-004).
//! - GGUF Parser: Zero-copy O(1) header inspection via `memmap2`.
//! - Response Healing: Deterministic stack-based JSON repair pipeline.

pub mod cohomology;
pub mod error;
pub mod gguf_mmap;
pub mod healing;
pub mod inference;
pub mod llamacpp;
pub mod nvml;
pub mod onnx;

// Re-export canonical types and functions
pub use cohomology::{
    boost_memory_conflict, compute_cohomology, CohomologyResult, FactEdge, RelationKind,
    RestrictionMatrix, SocraticFactGraph,
};
pub use error::InferenceError;
pub use gguf_mmap::{
    inspect_gguf_metadata_o1, parse_gguf_slice, BoundedGgufPool, GgufMappedReader,
    GgufMetadataInfo, MAX_MAPPED_HANDLES,
};
pub use healing::heal_json_response;
pub use inference::{
    get_or_compile_grammar, gpu_execution_lock, grammar_cache, resolve_llama_bench_binary,
    resolve_llama_cli_binary, resolve_llama_completion_binary, CompiledGrammar,
    GenOutput, GenParams, HardwareProfileMetrics, KvCacheConfig, KvCacheType,
    Tier1GenerativeEngine,
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
pub use souls_protocol::{
    DynamicReasoningRecommendation, ProviderReasoningPayloads, ReasoningEffortLevel,
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

    /// Combined Tier 0 + Tier 0.5 Intent evaluation and Epistemic entropy probing (0 MB VRAM).
    pub async fn evaluate_intent(
        &self,
        prompt: &str,
    ) -> Result<serde_json::Value, InferenceError> {
        self.evaluate_intent_with_retry(prompt, false).await
    }

    /// Combined Tier 0 + Tier 0.5 Intent evaluation with Fallback Escalation support.
    pub async fn evaluate_intent_with_retry(
        &self,
        prompt: &str,
        is_retry: bool,
    ) -> Result<serde_json::Value, InferenceError> {
        let classify = self.infer_tier0_classify(prompt).await?;
        let probe = self.infer_tier05_probe(prompt).await?;
        let reasoning = calculate_reasoning_effort(probe.entropy, &classify.top_label, is_retry);

        Ok(serde_json::json!({
            "intent": classify.top_label,
            "confidence": classify.confidence,
            "entropy": probe.entropy,
            "binary_entropy": probe.binary_entropy,
            "circuit_breaker_triggered": probe.circuit_breaker_triggered,
            "latency_ms": probe.latency_ms + classify.latency_ms,
            "vram_mb": 0,
            "source": probe.source,
            "scores": classify.scores,
            "reasoning_recommendation": reasoning,
        }))
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

    /// Physical token generation against a designated model path and prompt.
    pub async fn infer_model_generate(
        &self,
        model_path: &Path,
        prompt: &str,
        params: &GenParams,
    ) -> Result<GenOutput, InferenceError> {
        let mut wd = self.watchdog.lock().map_err(|_| {
            InferenceError::OnnxError("Watchdog lock poisoned".to_string())
        })?;

        self.tier1.generate_with_model(model_path, prompt, params, &mut wd)
    }

    /// Zero-copy O(1) GGUF metadata inspection backed by the bounded handle pool.
    pub fn inspect_gguf(&self, path: &Path) -> Result<GgufMetadataInfo, InferenceError> {
        self.gguf_pool.inspect_metadata(path)
    }

    /// In-flight JSON healing pipeline.
    pub fn heal_json(&self, raw: &str) -> Result<serde_json::Value, InferenceError> {
        heal_json_response(raw)
    }

    /// Measures physical hardware profiling metrics (TTFT, TPOT, TPS) on bare-metal silicion.
    pub fn profile_hardware(
        &self,
        model_path: &Path,
        n_prompt: u32,
        n_gen: u32,
        force_cpu: bool,
    ) -> Result<HardwareProfileMetrics, InferenceError> {
        self.tier1.profile_model_hardware(model_path, n_prompt, n_gen, force_cpu)
    }
}

impl Default for InferenceRuntime {
    fn default() -> Self {
        Self::new()
    }
}

/// Calculates dynamic reasoning effort and token thinking budget based on Tier 0.5 epistemic entropy,
/// intent categorization, and progressive escalation for retries (FinOps ADR).
pub fn calculate_reasoning_effort(
    entropy: f32,
    intent: &str,
    is_retry: bool,
) -> DynamicReasoningRecommendation {
    let base_level = if entropy < 0.35 || intent == "json_extraction" || intent == "formatting" {
        ReasoningEffortLevel::Low
    } else if entropy > 0.70
        || intent == "architecture_design"
        || intent == "security"
        || intent == "mathematics"
    {
        ReasoningEffortLevel::High
    } else {
        ReasoningEffortLevel::Medium
    };

    let (final_level, escalated) = if is_retry {
        match base_level {
            ReasoningEffortLevel::None => (ReasoningEffortLevel::Low, true),
            ReasoningEffortLevel::Low => (ReasoningEffortLevel::Medium, true),
            ReasoningEffortLevel::Medium => (ReasoningEffortLevel::High, true),
            ReasoningEffortLevel::High => (ReasoningEffortLevel::High, true),
        }
    } else {
        (base_level, false)
    };

    let budget = match final_level {
        ReasoningEffortLevel::None => 0,
        ReasoningEffortLevel::Low => 1024,
        ReasoningEffortLevel::Medium => 4096,
        ReasoningEffortLevel::High => 16384,
    };

    let rationale = if escalated {
        format!(
            "Escalated reasoning effort due to retry on {} (entropy: {:.2})",
            intent, entropy
        )
    } else {
        format!(
            "Calculated reasoning effort based on intent {} and entropy {:.2}",
            intent, entropy
        )
    };

    DynamicReasoningRecommendation::new(
        final_level,
        budget,
        entropy as f64,
        escalated,
        rationale,
    )
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

    #[tokio::test]
    async fn test_unified_inference_runtime_evaluate_intent() {
        let runtime = InferenceRuntime::new();
        let res = runtime
            .evaluate_intent("Write a rust parser with nom")
            .await
            .expect("evaluate_intent failed");

        assert_eq!(res["intent"], "code");
        assert_eq!(res["vram_mb"], 0);
        assert!(res["latency_ms"].as_f64().unwrap() > 0.0);
        assert!(res["reasoning_recommendation"]["reasoning_effort"].is_string());
        assert!(res["reasoning_recommendation"]["thinking_budget_tokens"].as_u64().unwrap() > 0);
    }

    #[test]
    fn test_calculate_reasoning_effort_stages_and_fallback() {
        // Estágio 1: Baixa entropia (H < 0.35)
        let low = calculate_reasoning_effort(0.20, "json_extraction", false);
        assert_eq!(low.reasoning_effort, ReasoningEffortLevel::Low);
        assert_eq!(low.thinking_budget_tokens, 1024);
        assert!(!low.is_escalated_retry);

        // Estágio 2: Média entropia (0.35 <= H <= 0.70)
        let med = calculate_reasoning_effort(0.48, "code_refactoring", false);
        assert_eq!(med.reasoning_effort, ReasoningEffortLevel::Medium);
        assert_eq!(med.thinking_budget_tokens, 4096);

        // Estágio 3: Alta entropia (H > 0.70)
        let high = calculate_reasoning_effort(0.82, "architecture_design", false);
        assert_eq!(high.reasoning_effort, ReasoningEffortLevel::High);
        assert_eq!(high.thinking_budget_tokens, 16384);

        // Teste de Fallback Escalation em retry: Low -> Medium
        let escalated_low = calculate_reasoning_effort(0.20, "json_extraction", true);
        assert_eq!(escalated_low.reasoning_effort, ReasoningEffortLevel::Medium);
        assert_eq!(escalated_low.thinking_budget_tokens, 4096);
        assert!(escalated_low.is_escalated_retry);

        // Teste de Fallback Escalation em retry: Medium -> High
        let escalated_med = calculate_reasoning_effort(0.48, "code_refactoring", true);
        assert_eq!(escalated_med.reasoning_effort, ReasoningEffortLevel::High);
        assert_eq!(escalated_med.thinking_budget_tokens, 16384);
        assert!(escalated_med.is_escalated_retry);
    }
}
