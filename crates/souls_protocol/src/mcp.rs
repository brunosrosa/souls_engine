//! Model Context Protocol (MCP) data structures and JSON-RPC 2.0 envelopes.
//!
//! Provides agnostic definitions for RPC requests, responses, tool schemas,
//! and invocation results for the Souls Engine v7.
//!
//! Pure types, zero I/O side effects, zero unsafe code.

use serde::{Deserialize, Serialize};

/// Standardized JSON-RPC 2.0 Request envelope.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JsonRpcRequest<T = serde_json::Value> {
    pub jsonrpc: String,
    pub id: serde_json::Value,
    pub method: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub params: Option<T>,
}

impl<T> JsonRpcRequest<T> {
    #[must_use]
    pub fn new(id: serde_json::Value, method: impl Into<String>, params: Option<T>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            method: method.into(),
            params,
        }
    }
}

/// Standardized JSON-RPC 2.0 Response envelope.
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

/// MCP Tool schema definition exposed via `tools/list`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct McpToolDefinition {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: serde_json::Value,
}

impl McpToolDefinition {
    #[must_use]
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        input_schema: serde_json::Value,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            input_schema,
        }
    }
}

/// Inbound MCP tool invocation request from the client (`tools/call`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct McpToolCall {
    pub name: String,
    #[serde(default)]
    pub arguments: serde_json::Value,
}

/// Content block types supported by MCP tool responses.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ToolContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
}

/// Envelope for MCP tool invocation content output (`tools/call` response).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolCallResult {
    pub content: Vec<ToolContentBlock>,
    #[serde(rename = "isError", default)]
    pub is_error: bool,
}

impl ToolCallResult {
    #[must_use]
    pub fn ok(text: impl Into<String>) -> Self {
        Self {
            content: vec![ToolContentBlock::Text { text: text.into() }],
            is_error: false,
        }
    }

    #[must_use]
    pub fn error(text: impl Into<String>) -> Self {
        Self {
            content: vec![ToolContentBlock::Text { text: text.into() }],
            is_error: true,
        }
    }
}

/// Alias for MCP tool result envelope conforming to CRATES_SPECIFICATION.md.
pub type McpToolResult = ToolCallResult;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jsonrpc_success_response() {
        let resp = JsonRpcResponse::success(serde_json::json!(1), "ok".to_string());
        assert_eq!(resp.jsonrpc, "2.0");
        assert_eq!(resp.result, Some("ok".to_string()));
        assert!(resp.error.is_none());
    }

    #[test]
    fn test_mcp_tool_definition_serde() {
        let tool = McpToolDefinition {
            name: "souls_ast_outline".to_string(),
            description: "Extract syntactical skeleton".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" }
                },
                "required": ["path"]
            }),
        };

        let json = serde_json::to_string(&tool).expect("serialize tool definition");
        assert!(json.contains("souls_ast_outline"));
        assert!(json.contains("inputSchema"));

        let de: McpToolDefinition = serde_json::from_str(&json).expect("deserialize tool definition");
        assert_eq!(tool.name, de.name);
    }

    #[test]
    fn test_tool_call_result_constructors() {
        let ok_res = ToolCallResult::ok("success");
        assert!(!ok_res.is_error);
        assert_eq!(ok_res.content.len(), 1);

        let err_res = ToolCallResult::error("failed");
        assert!(err_res.is_error);
    }
}
