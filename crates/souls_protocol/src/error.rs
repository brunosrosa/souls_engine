//! McpDomainError, ProtocolError and error envelopes for Souls Engine (v7).
//!
//! Maps domain errors to the non-negotiable JSON-RPC error code range (-32000..=-32099)
//! as specified in ADR-041 and MCP_TOOL_CONTRACTS.md.
//!
//! Re-exports JSON-RPC envelope types from `mcp` for complete backwards compatibility.

use serde::{Deserialize, Serialize};
use thiserror::Error;

// Re-export MCP envelope and result structures for backwards compatibility
pub use crate::mcp::{JsonRpcError, JsonRpcRequest, JsonRpcResponse, McpToolResult, ToolCallResult, ToolContentBlock};

/// Unified domain error matrix for the Souls MCP bus.
#[derive(Error, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum McpDomainError {
    /// Pânico recuperado na thread Tokio via catch_unwind (Código -32000).
    #[error("InternalEnginePanic: Pânico recuperado na thread Tokio")]
    InternalEnginePanic,

    /// O identificador fornecido não foi achado na AST (Código -32001).
    #[error("SymbolNotFound: O identificador não foi achado na AST")]
    SymbolNotFound,

    /// Watchdog NVML disparou barreira de VRAM ou temperatura (Código -32002).
    #[error("VramThermalThrottled: Watchdog NVML disparou barreira de VRAM ou térmica")]
    VramThermalThrottled,

    /// Arquivo físico excedeu o limite máximo de parsing de AST (Código -32003).
    #[error("FileTooLargeForAST: O arquivo excede o limite físico para análise AST")]
    FileTooLargeForAST,

    /// Operação negada por intervenção humana / Human-In-The-Loop (Código -32004).
    #[error("HitlDenied: Operação cancelada ou negada por intervenção humana (HITL)")]
    HitlDenied,

    /// Falha estruturada no isolamento de runtime Wasmtime (Código -32005).
    #[error("StructuredFailure: Falha estruturada enjaulada no runtime Wasmtime")]
    StructuredFailure,

    /// Violação de barreira ontológica no LadybugDB (Código -32006).
    #[error("OntologicalBarrierViolation: LadybugDB detectou violação de integridade ontológica")]
    OntologicalBarrierViolation,

    /// Parâmetros de entrada inválidos ou faltantes no JSON Schema (Código -32007).
    #[error("InvalidInputParameters: Parâmetros de entrada inválidos ou campos obrigatórios ausentes")]
    InvalidInputParameters,

    /// Tentativa de escape da partição ReFS Z:\ (Código -32008).
    #[error("RefsPathLeakViolation: Caminho informado viola os limites da partição ReFS Z:")]
    RefsPathLeakViolation,

    /// Falha ao inspecionar repositório Git bare-metal via gix (Código -32009).
    #[error("GitoxideRepositoryError: Falha ao inspecionar o repositório Git via gix")]
    GitoxideRepositoryError,

    /// Violação de integridade STRICT em query do SQLite (Código -32010).
    #[error("SqlStrictViolation: Violação de tipo de dado ou instrução de escrita em sqlite_query")]
    SqlStrictViolation,

    /// Ferramenta solicitada não existe ou não pertence ao namespace (Código -32011).
    #[error("ToolNotFoundInNamespace: Ferramenta solicitada não encontrada no namespace ativo")]
    ToolNotFoundInNamespace,
}

impl McpDomainError {
    /// Returns the standardized JSON-RPC error code in the -32000..=-32099 range.
    #[must_use]
    pub const fn jsonrpc_code(&self) -> i32 {
        match self {
            Self::InternalEnginePanic => -32000,
            Self::SymbolNotFound => -32001,
            Self::VramThermalThrottled => -32002,
            Self::FileTooLargeForAST => -32003,
            Self::HitlDenied => -32004,
            Self::StructuredFailure => -32005,
            Self::OntologicalBarrierViolation => -32006,
            Self::InvalidInputParameters => -32007,
            Self::RefsPathLeakViolation => -32008,
            Self::GitoxideRepositoryError => -32009,
            Self::SqlStrictViolation => -32010,
            Self::ToolNotFoundInNamespace => -32011,
        }
    }

    /// Reconstructs a known `McpDomainError` from its JSON-RPC integer code.
    #[must_use]
    pub const fn from_code(code: i32) -> Option<Self> {
        match code {
            -32000 => Some(Self::InternalEnginePanic),
            -32001 => Some(Self::SymbolNotFound),
            -32002 => Some(Self::VramThermalThrottled),
            -32003 => Some(Self::FileTooLargeForAST),
            -32004 => Some(Self::HitlDenied),
            -32005 => Some(Self::StructuredFailure),
            -32006 => Some(Self::OntologicalBarrierViolation),
            -32007 => Some(Self::InvalidInputParameters),
            -32008 => Some(Self::RefsPathLeakViolation),
            -32009 => Some(Self::GitoxideRepositoryError),
            -32010 => Some(Self::SqlStrictViolation),
            -32011 => Some(Self::ToolNotFoundInNamespace),
            _ => None,
        }
    }

    /// Converts this domain error to a standard `JsonRpcError` with no additional data.
    #[must_use]
    pub fn to_jsonrpc_error(&self) -> JsonRpcError {
        JsonRpcError::new(self.jsonrpc_code(), self.to_string())
    }

    /// Converts this domain error to a `JsonRpcError` including structured metadata.
    #[must_use]
    pub fn to_jsonrpc_error_with_data(&self, data: serde_json::Value) -> JsonRpcError {
        JsonRpcError::with_data(self.jsonrpc_code(), self.to_string(), data)
    }
}

impl From<McpDomainError> for JsonRpcError {
    fn from(err: McpDomainError) -> Self {
        err.to_jsonrpc_error()
    }
}

impl From<&McpDomainError> for JsonRpcError {
    fn from(err: &McpDomainError) -> Self {
        err.to_jsonrpc_error()
    }
}

/// Generic protocol-level errors occurring during serialization, parsing or envelope transport.
#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("JSON serialization or deserialization failure: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Invalid JSON-RPC 2.0 version header: expected '2.0', found '{0}'")]
    InvalidVersion(String),

    #[error("Missing mandatory field in protocol envelope: '{0}'")]
    MissingField(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_domain_error_conversion() {
        let err = McpDomainError::RefsPathLeakViolation;
        assert_eq!(err.jsonrpc_code(), -32008);
        assert_eq!(McpDomainError::from_code(-32008), Some(McpDomainError::RefsPathLeakViolation));

        let rpc_err = err.to_jsonrpc_error();
        assert_eq!(rpc_err.code, -32008);
        assert!(rpc_err.message.contains("RefsPathLeakViolation"));
    }

    #[test]
    fn test_error_with_data() {
        let err = McpDomainError::InvalidInputParameters;
        let rpc_err = err.to_jsonrpc_error_with_data(serde_json::json!({"field": "path"}));
        assert_eq!(rpc_err.code, -32007);
        assert!(rpc_err.data.is_some());
    }
}
