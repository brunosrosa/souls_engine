//! Canonical error types for the `souls_memory` crate.

use thiserror::Error;

/// Central error enumeration for all memory operations, including SQLite STRICT,
/// RRF fusion, LadybugDB graph, and Chyros Langevin metabolism.
#[derive(Debug, Error)]
pub enum MemoryError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Core error: {0}")]
    Core(#[from] souls_core::error::CoreError),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

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
}
