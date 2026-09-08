//! Error domain for souls_ast.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AstError {
    #[error("diff computation error: {0}")]
    DiffError(String),

    #[error("git repository inspection error: {0}")]
    GitError(String),

    #[error("wasm runtime trap: {0}")]
    WasmTrap(String),

    #[error("wasm compilation/instantiation error: {0}")]
    WasmError(String),

    #[error("ast parsing failure for {language}: {reason}")]
    ParseFailure {
        language: String,
        reason: String,
    },

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
