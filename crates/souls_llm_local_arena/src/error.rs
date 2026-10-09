//! Unified error types for the Arena benchmark engine.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ArenaError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Database error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("Inference runtime error: {0}")]
    Inference(#[from] souls_inference_runtime::InferenceError),

    #[error("Model not found on disk or vault: {0}")]
    ModelNotFound(String),

    #[error("Thermal throttled by NVML watchdog: {0}")]
    ThermalThrottled(String),

    #[error("VRAM allocation ceiling exceeded: requested {requested_mb:.1} MB, ceiling is {ceiling_mb:.1} MB")]
    VramCeilingExceeded {
        requested_mb: f64,
        ceiling_mb: f64,
    },

    #[error("Worker thread panic: {0}")]
    WorkerPanic(String),

    #[error("Benchmark execution timeout: {0}")]
    Timeout(String),

    #[error("Unsupported engine for model topology: {0}")]
    UnsupportedEngine(String),

    #[error("Invalid arguments: {0}")]
    InvalidArguments(String),
}
