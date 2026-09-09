//! Error types for souls_model_router.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum RouterError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Memory store error: {0}")]
    Memory(#[from] souls_memory::MemoryError),

    #[error("Inference runtime error: {0}")]
    Inference(#[from] souls_inference_runtime::InferenceError),

    #[error("No active model candidates available for routing: {0}")]
    NoActiveCandidates(String),

    #[error("Invalid prior parameters: {0}")]
    InvalidPriorParameters(String),

    #[error("Context window exceeded: {0}")]
    ContextWindowExceeded(String),

    #[error("Invalid task outcome feedback: {0}")]
    InvalidOutcome(String),

    #[error("Watchdog lock poisoned: {0}")]
    LockPoisoned(String),
}
