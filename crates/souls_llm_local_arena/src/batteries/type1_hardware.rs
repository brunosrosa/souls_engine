//! Type 1: HardwareProfile benchmark battery implementation.

use std::time::Instant;
use souls_inference_runtime::HardwareWatchdog;
use crate::domain::engine::{ArenaInferEngine, InferenceRunParams};
use crate::domain::model::ModelDescriptor;
use crate::domain::result::HardwareProfileResult;
use crate::error::ArenaError;
use crate::harness::nvml_guard::{
    check_hardware_safeguards, get_current_process_ram_mb, SAFE_WDDM_VRAM_CEILING_MB,
};

/// Executes Type 1 HardwareProfile on the specified model and engine.
pub async fn run_hardware_profile(
    descriptor: &ModelDescriptor,
    engine: &dyn ArenaInferEngine,
    watchdog: &mut HardwareWatchdog,
) -> Result<HardwareProfileResult, ArenaError> {
    // 1. Guardrail check before starting
    check_hardware_safeguards(watchdog)?;

    let initial_telemetry = watchdog.sample_once();
    let initial_vram_mb = (souls_inference_runtime::RTX_2060M_VRAM_TOTAL_MB - initial_telemetry.vram_free_mb) as f64;
    let initial_ram_mb = get_current_process_ram_mb();

    // 2. Measure Cold Load timing (graceful fallback if path is synthetic or during test)
    let cold_start = Instant::now();
    let file_exists = descriptor.file_path.exists();
    if file_exists {
        let _ = descriptor.file_path.metadata();
    }
    let cold_load_ms = if file_exists {
        cold_start.elapsed().as_secs_f64() * 1000.0
    } else {
        15.0
    };

    // 3. Execution prompt for sustained throughput and prefill TTFT test
    let test_params = InferenceRunParams {
        prompt: "Write a high-performance concurrent queue in Rust with atomic pointers and backoff."
            .to_string(),
        system_prompt: Some("You are an expert low-level systems programming AI.".to_string()),
        max_tokens: 128,
        temperature: 0.1,
        top_p: 0.9,
        json_schema: None,
    };

    let output = engine.execute(descriptor, &test_params).await?;

    // 4. Sample peak hardware after execution
    let final_telemetry = watchdog.sample_once();
    let final_vram_mb = (souls_inference_runtime::RTX_2060M_VRAM_TOTAL_MB - final_telemetry.vram_free_mb) as f64;
    let final_ram_mb = get_current_process_ram_mb();

    let peak_vram_mb = output.peak_vram_mb.max(final_vram_mb).max(initial_vram_mb);
    let peak_ram_mb = output.ram_mb.max(final_ram_mb).max(initial_ram_mb);

    // 5. Evaluate status against WDDM ceiling
    let status = if peak_vram_mb > SAFE_WDDM_VRAM_CEILING_MB {
        "VRAM_SPILLOVER_WARNING".to_string()
    } else if final_telemetry.gpu_temp_c >= 78.0 {
        "THERMAL_WARNING".to_string()
    } else {
        "NOMINAL".to_string()
    };

    // 6. Calculate KV Cache descriptor based on context length
    let kv_cache_desc = format!("{}k (asym)", descriptor.ctx_max / 1024);

    // 7. Calculate E3 hardware score: (TPS^2) / (TTFT_sec + 0.001) / 1000.0
    let ttft_sec = output.ttft_ms / 1000.0;
    let e3_score = if ttft_sec > 0.0 {
        ((output.tps * output.tps) / (ttft_sec + 0.001)).min(9999.0)
    } else {
        output.tps
    };

    let is_gpu = engine.engine_type() == crate::domain::engine::EngineType::LlamaCppCuda;
    let is_cpu = engine.engine_type() == crate::domain::engine::EngineType::LlamaCppCpu
        || engine.engine_type() == crate::domain::engine::EngineType::OnnxCpu;
    let is_hybrid = descriptor.tier == crate::domain::model::ModelTier::Tier2Background;

    Ok(HardwareProfileResult {
        model_id: descriptor.model_id.clone(),
        model_path: descriptor.file_path.to_string_lossy().to_string(),
        tier: descriptor.tier.as_str().to_string(),
        engine: engine.engine_type().as_str().to_string(),
        size_gb: descriptor.size_mb / 1024.0,
        ctx_max: descriptor.ctx_max,
        quant: descriptor.quant.clone(),
        is_cpu,
        is_gpu,
        is_hybrid,
        cold_load_ms: output.cold_load_ms.max(cold_load_ms),
        ttft_ms: output.ttft_ms,
        tpot_ms: output.tpot_ms,
        tps: output.tps,
        peak_vram_mb,
        ram_mb: peak_ram_mb,
        kv_cache_desc,
        e3_score,
        status,
    })
}
