//! Canonical error types for souls_core.

use thiserror::Error;

/// Core engine errors for bare-metal I/O, ReFS operations, and token budgeting.
#[derive(Error, Debug)]
pub enum CoreError {
    /// I/O error wrapper.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// ReFS Block Cloning failure via Win32 FSCTL_DUPLICATE_EXTENTS_TO_FILE.
    #[error("ReFS block cloning failed: {0}")]
    RefsBlockCloningFailed(String),

    /// Security boundary violation: path leaks outside canonical Dev Drive (Z:\).
    #[error("ReFS path leak violation: {0}")]
    RefsPathLeakViolation(String),

    /// Token budget exceeded during auto-shrink / smart-read.
    #[error("Token budget exceeded: {tokens} tokens exceeds budget of {budget}")]
    TokenBudgetExceeded { tokens: usize, budget: usize },

    /// Async task spawn_blocking join error.
    #[error("Task join error: {0}")]
    TaskJoinError(String),

    /// Logging / Tracing initialization failure.
    #[error("Tracing initialization error: {0}")]
    TracingInitError(String),
}
