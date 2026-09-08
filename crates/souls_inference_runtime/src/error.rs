//! Canonical error types for souls_inference_runtime.
//!
//! Maps errors to Souls Engine domain errors and provides detailed telemetry context.

use thiserror::Error;

/// Canonical error enumeration for the inference runtime.
#[derive(Error, Debug)]
pub enum InferenceError {
    /// Model file was not found at specified path.
    #[error("Model file not found: {0}")]
    ModelNotFound(String),

    /// NVML initialization failed.
    #[error("NVML initialization failed: {0}")]
    NvmlInitError(String),

    /// NVML query failed.
    #[error("NVML query failed: {0}")]
    NvmlQueryError(String),

    /// VRAM or Thermal limit exceeded, triggering hardware throttle.
    #[error("Hardware thermal throttle active: VRAM free={vram_free_mb}MB, GPU temp={gpu_temp_c:.1}°C, barrier={barrier:.2}")]
    VramThermalThrottled {
        vram_free_mb: u32,
        gpu_temp_c: f32,
        barrier: f32,
    },

    /// GPU temperature exceeded critical safety limit.
    #[error("GPU temperature critical: {gpu_temp_c:.1}°C exceeds limit {threshold_c:.1}°C")]
    ThermalLimitExceeded {
        gpu_temp_c: f32,
        threshold_c: f32,
    },

    /// Error parsing GGUF header metadata.
    #[error("GGUF header parse error: {0}")]
    GgufParseError(String),

    /// ONNX Runtime execution error.
    #[error("ONNX Runtime error: {0}")]
    OnnxError(String),

    /// llama.cpp backend execution error.
    #[error("llama.cpp backend error: {0}")]
    LlamaCppError(String),

    /// JSON response healing failed to produce valid JSON.
    #[error("JSON response healing failed: {0}")]
    JsonHealingError(String),

    /// Structured syntax schema violation.
    #[error("Structured syntax schema violation: {0}")]
    SchemaViolation(String),

    /// Underlying standard I/O error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Underlying serde_json error.
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
}

impl From<InferenceError> for souls_protocol::error::McpDomainError {
    fn from(err: InferenceError) -> Self {
        match err {
            InferenceError::VramThermalThrottled { .. }
            | InferenceError::ThermalLimitExceeded { .. } => {
                souls_protocol::error::McpDomainError::VramThermalThrottled
            }
            InferenceError::SchemaViolation(_) => {
                souls_protocol::error::McpDomainError::InvalidInputParameters
            }
            _ => souls_protocol::error::McpDomainError::InternalEnginePanic,
        }
    }
}
