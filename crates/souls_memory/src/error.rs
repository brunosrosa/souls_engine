//! Canonical error types for the `souls_memory` crate.

use thiserror::Error;

/// Central error enumeration for all memory operations, including SQLite STRICT,
/// LanceDB columnar vector storage, RRF fusion, LadybugDB graph, and Chyros Langevin metabolism.
#[derive(Debug, Error)]
pub enum MemoryError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Core error: {0}")]
    Core(#[from] souls_core::error::CoreError),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("LanceDB vector storage error: {0}")]
    LanceDb(String),

    #[error("Vector dimension mismatch: expected {expected}, received {received}")]
    VectorDimensionMismatch { expected: usize, received: usize },

    #[error("Ontology violation in LadybugDB: {reason} (violated node: {violated_node})")]
    OntologyViolation {
        reason: String,
        violated_node: String,
    },

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Win32 API error: code {code}, {message}")]
    Win32Error { code: u32, message: String },

    #[error("Node '{0}' not found in LadybugDB graph")]
    NodeNotFound(String),

    #[error("Invalid partition value: '{0}'. Expected 'STABLE' or 'EVOLVING'")]
    InvalidPartition(String),

    #[error("Read-only violation: mutation query rejected '{0}'")]
    ReadOnlyViolation(String),

    #[error("Overthinking threshold breached: {current} thoughts exceed limit {limit} without HITL authorization")]
    OverthinkingThresholdBreached { current: u32, limit: u32 },

    #[error("Invalid revision targeting thought #{target}: thought does not exist or invalid sequence")]
    InvalidThoughtRevision { target: u32 },

    #[error("Session '{0}' not found")]
    SessionNotFound(String),

    #[error("Sub-agent '{0}' not found")]
    SubAgentNotFound(String),

    #[error("Handoff payload '{0}' not found")]
    HandoffNotFound(String),
}
