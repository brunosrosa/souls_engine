//! # souls_llm_local_arena (Souls Engine v7)
//!
//! Bare-metal silicon profiling, cognitive evaluation, and benchmark colosseum
//! governed strictly by ADR-007 and ARENA_BENCHMARK_SPECIFICATION.md.
//!
//! Provides orthogonal decoupling across:
//! - `ModelTier` (Physical hardware scale: Tier 0 to Tier 4)
//! - `InferenceEngine` (Interchangeable backends: LlamaCpp CUDA/CPU, ONNX, Mock)
//! - `BenchmarkType` (The 5 Canonical Evaluation Batteries: Types 1 to 5)

#![forbid(unsafe_code)]

pub mod batteries;
pub mod discovery;
pub mod domain;
pub mod engines;
pub mod error;
pub mod harness;
pub mod reporters;

pub use batteries::{
    run_context_pressure, run_cognitive_eval, run_hardware_profile, run_multimodal_vision,
    run_speculative_draft, BenchmarkType,
};
pub use discovery::{
    default_model_directories, enrich_with_gguf_metadata, inspect_model_file, resolve_vault_db_path,
    scan_local_models, VaultManager,
};
pub use domain::{
    ArenaInferEngine, CognitiveEvalResult, ContextPressureResult, EngineType, HardwareProfileResult,
    InferenceRunParams, ModelDescriptor, ModelTier, MultimodalVisionResult, RawInferenceOutput,
    SpeculativeResult,
};
pub use engines::{MockInferEngine, SiliconRuntimeEngine};
pub use error::ArenaError;
pub use harness::{
    check_hardware_safeguards, dispatch_safe_worker, get_current_process_ram_mb,
    SAFE_WDDM_VRAM_CEILING_MB, THERMAL_ABORT_THRESHOLD_C,
};
pub use reporters::{
    export_markdown_dossier, print_cognitive_eval_table, print_hardware_profile_table,
    print_model_list_table, print_pressure_table, print_speculative_table, print_vision_table,
};

use std::sync::Arc;
use souls_inference_runtime::{HardwareWatchdog, InferenceRuntime};

/// High-level unified coordinator facade for the Arena.
pub struct ArenaRunner {
    runtime: Arc<InferenceRuntime>,
    watchdog: HardwareWatchdog,
}

impl ArenaRunner {
    /// Creates a new ArenaRunner with default silicon runtime.
    pub fn new() -> Self {
        Self {
            runtime: Arc::new(InferenceRuntime::new()),
            watchdog: HardwareWatchdog::new(),
        }
    }

    /// Discovers all local models across default directories.
    pub fn discover_models(&self) -> Vec<ModelDescriptor> {
        let dirs = default_model_directories();
        scan_local_models(&dirs)
    }

    /// Connects to the heuristic vault database.
    pub async fn connect_vault(&self) -> Result<VaultManager, ArenaError> {
        let db_path = resolve_vault_db_path();
        VaultManager::connect(&db_path).await
    }

    /// Synchronizes discovered models with the SQLite database.
    pub async fn sync_models_to_vault(&self) -> Result<usize, ArenaError> {
        let models = self.discover_models();
        let vault = self.connect_vault().await?;
        vault.sync_models(&models).await
    }

    /// Executes Type 1 HardwareProfile on a target model descriptor.
    pub async fn run_hardware_profile(
        &mut self,
        descriptor: &ModelDescriptor,
        engine_type: EngineType,
    ) -> Result<HardwareProfileResult, ArenaError> {
        let engine = SiliconRuntimeEngine::new(Arc::clone(&self.runtime), engine_type);
        run_hardware_profile(descriptor, &engine, &mut self.watchdog).await
    }

    /// Executes Type 2 CognitiveEval on a target model descriptor.
    pub async fn run_cognitive_eval(
        &self,
        descriptor: &ModelDescriptor,
        engine_type: EngineType,
        track: &str,
    ) -> Result<CognitiveEvalResult, ArenaError> {
        let engine = SiliconRuntimeEngine::new(Arc::clone(&self.runtime), engine_type);
        run_cognitive_eval(descriptor, &engine, track).await
    }
}

impl Default for ArenaRunner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_model_tier_classification_rules() {
        let tier0 = ModelTier::classify(Path::new("gliclass_model.onnx"), 250.0, false);
        assert_eq!(tier0, ModelTier::Tier0Bootstrap);

        let tier05 = ModelTier::classify(Path::new("SmolLM-360M.gguf"), 720.0, false);
        assert_eq!(tier05, ModelTier::Tier05Epistemic);

        let tier1 = ModelTier::classify(Path::new("Qwen3.5-4B-Coder.gguf"), 2600.0, false);
        assert_eq!(tier1, ModelTier::Tier1Master);

        let tier2 = ModelTier::classify(Path::new("Bonsai-27B.gguf"), 16000.0, false);
        assert_eq!(tier2, ModelTier::Tier2Background);

        let tier3 = ModelTier::classify(Path::new("Qwen-VL-7B.gguf"), 4200.0, true);
        assert_eq!(tier3, ModelTier::Tier3Vision);

        let tier4 = ModelTier::classify(Path::new("Qwen-DFlash-Draft.gguf"), 400.0, false);
        assert_eq!(tier4, ModelTier::Tier4Drafter);
    }

    #[test]
    fn test_benchmark_type_parsing() {
        assert_eq!(BenchmarkType::parse("1"), Some(BenchmarkType::Type1Hardware));
        assert_eq!(BenchmarkType::parse("profile"), Some(BenchmarkType::Type1Hardware));
        assert_eq!(BenchmarkType::parse("cognitive"), Some(BenchmarkType::Type2Cognitive));
        assert_eq!(BenchmarkType::parse("3"), Some(BenchmarkType::Type3Vision));
        assert_eq!(BenchmarkType::parse("speculative"), Some(BenchmarkType::Type4Speculative));
        assert_eq!(BenchmarkType::parse("pressure"), Some(BenchmarkType::Type5Pressure));
        assert_eq!(BenchmarkType::parse("all"), Some(BenchmarkType::All));
    }

    #[test]
    fn test_quant_and_param_extraction() {
        let q = domain::model::extract_quant("qwen3.5-4b-agentic-coder-v4.i1-Q4_K_S.gguf");
        assert_eq!(q, "I1-Q4_K_S");

        let p = domain::model::extract_parameters("unsloth_Mamba-Codestral-7B-Instruct.gguf");
        assert_eq!(p, "7B");
    }

    #[tokio::test]
    async fn test_mock_engine_execution() {
        let desc = ModelDescriptor::from_file("test_model_4b.gguf");
        let engine = MockInferEngine;
        let params = InferenceRunParams::default();

        let out = engine.execute(&desc, &params).await.unwrap();
        assert!(out.tps > 0.0);
        assert!(out.ttft_ms > 0.0);
    }

    #[tokio::test]
    async fn test_type1_mock_hardware_profile() {
        let desc = ModelDescriptor::from_file("Qwen3.5-4B.gguf");
        let engine = MockInferEngine;
        let mut watchdog = HardwareWatchdog::new();

        let res = run_hardware_profile(&desc, &engine, &mut watchdog).await.unwrap();
        assert_eq!(res.model_id, "Qwen3.5-4B");
        assert!(res.tps > 0.0);
        assert_eq!(res.status, "NOMINAL");
    }

    #[tokio::test]
    async fn test_type2_mock_cognitive_eval() {
        let desc = ModelDescriptor::from_file("Mamba-Codestral-7B.gguf");
        let engine = MockInferEngine;

        let ast_res = run_cognitive_eval(&desc, &engine, "ast").await.unwrap();
        assert!(ast_res.ast_valid);
        assert_eq!(ast_res.status, "QUALIFIED");

        let json_res = run_cognitive_eval(&desc, &engine, "json").await.unwrap();
        assert!(json_res.json_strict_valid);
    }
}
