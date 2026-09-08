//! McpDomainError and JSON-RPC 2.0 transport envelopes for Souls Engine (v7).
//!
//! Maps domain errors to the non-negotiable JSON-RPC error code range (-32000..=-32099)
//! as specified in ADR-041 and MCP_TOOL_CONTRACTS.md.

use serde::{Deserialize, Serialize};
use thiserror::Error;

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

/// Standardized JSON-RPC 2.0 Error Object.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl JsonRpcError {
    #[must_use]
    pub fn new(code: i32, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            data: None,
        }
    }

    #[must_use]
    pub fn with_data(code: i32, message: impl Into<String>, data: serde_json::Value) -> Self {
        Self {
            code,
            message: message.into(),
            data: Some(data),
        }
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

/// Canonical JSON-RPC 2.0 Request envelope.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JsonRpcRequest<T = serde_json::Value> {
    pub jsonrpc: String,
    pub id: serde_json::Value,
    pub method: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub params: Option<T>,
}

impl<T> JsonRpcRequest<T> {
    pub fn new(id: serde_json::Value, method: impl Into<String>, params: Option<T>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            method: method.into(),
            params,
        }
    }
}

/// Canonical JSON-RPC 2.0 Response envelope.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JsonRpcResponse<T = serde_json::Value> {
    pub jsonrpc: String,
    pub id: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

impl<T> JsonRpcResponse<T> {
    #[must_use]
    pub fn success(id: serde_json::Value, result: T) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    #[must_use]
    pub fn error(id: serde_json::Value, error: JsonRpcError) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(error),
        }
    }
}

/// Envelope for MCP tool invocation content output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolCallResult {
    pub content: Vec<ToolContentBlock>,
    #[serde(rename = "isError", default)]
    pub is_error: bool,
}

/// Content block types supported by MCP tool responses.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ToolContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_response_constructors() {
        let resp_ok = JsonRpcResponse::success(serde_json::json!(1), "hello".to_string());
        assert_eq!(resp_ok.result, Some("hello".to_string()));
        assert!(resp_ok.error.is_none());

        let err = JsonRpcError::new(-32000, "Internal error");
        let resp_err = JsonRpcResponse::<()>::error(serde_json::json!(2), err);
        assert!(resp_err.result.is_none());
        assert_eq!(resp_err.error.as_ref().map(|e| e.code), Some(-32000));
    }

    #[test]
    fn test_tool_call_result_serde() {
        let result = ToolCallResult {
            content: vec![ToolContentBlock::Text {
                text: "fn main() {}".to_string(),
            }],
            is_error: false,
        };

        let json = serde_json::to_string(&result).expect("serialization failed");
        assert!(json.contains(r#""type":"text""#));
        let de: ToolCallResult = serde_json::from_str(&json).expect("deserialization failed");
        assert_eq!(result, de);
    }
}
