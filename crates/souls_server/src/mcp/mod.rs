//! # Model Context Protocol (MCP) Subsystem (`souls_server::mcp`)
//!
//! Bare-metal headless daemon MCP implementation.
//! Provides Dual-Mount HTTP/SSE transport, Progressive Tool Disclosure,
//! alias normalization, ReFS path containment, and lock-free session governance.

pub mod dispatcher;
pub mod handlers;
pub mod registry;
pub mod session;
pub mod transport;

pub use session::McpSessionManager;
