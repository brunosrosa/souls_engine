//! Structured result models for the 5 benchmark types.

use serde::{Deserialize, Serialize};

/// Type 1: HardwareProfile benchmark outcome report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareProfileResult {
    pub model_id: String,
    pub model_path: String,
    pub tier: String,
    pub engine: String,
    pub size_gb: f64,
    pub ctx_max: u32,
    pub quant: String,
    pub is_cpu: bool,
    pub is_gpu: bool,
    pub is_hybrid: bool,
    pub cold_load_ms: f64,
    pub ttft_ms: f64,
    pub tpot_ms: f64,
    pub tps: f64,
    pub peak_vram_mb: f64,
    pub ram_mb: f64,
    pub kv_cache_desc: String,
    pub e3_score: f64,
    pub status: String,
}

/// Type 2: CognitiveEval benchmark outcome report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveEvalResult {
    pub model_id: String,
    pub model_path: String,
    pub track: String,
    pub accuracy_pct: f64,
    pub ast_valid: bool,
    pub json_strict_valid: bool,
    pub think_tokens_avg: u32,
    pub cot_efficiency_ratio: f64,
    pub avg_ttft_ms: f64,
    pub avg_tps: f64,
    pub e3_score: f64,
    pub status: String,
}

/// Type 3: MultimodalVision benchmark outcome report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultimodalVisionResult {
    pub model_id: String,
    pub sidecar_path: String,
    pub projection_latency_ms: f64,
    pub multimodal_ttft_ms: f64,
    pub status: String,
}

/// Type 4: SpeculativeDraft benchmark outcome report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeculativeResult {
    pub model_id: String,
    pub drafter_id: String,
    pub draft_type: String,
    pub acceptance_rate_alpha: f64,
    pub base_tps: f64,
    pub speculative_tps: f64,
    pub speedup_ratio: f64,
    pub is_beneficial: bool,
    pub verdict: String,
}

/// Type 5: ContextPressure benchmark outcome report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextPressureResult {
    pub model_id: String,
    pub max_tested_context: u32,
    pub max_effective_context: u32,
    pub needle_at_4k: bool,
    pub needle_at_8k: bool,
    pub needle_at_16k: bool,
    pub needle_at_32k: bool,
    pub degradation_detected: bool,
    pub status: String,
}
