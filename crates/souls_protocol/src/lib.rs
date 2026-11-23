//! # Souls Protocol (`souls_protocol`)
//!
//! Universal Data Transfer Objects (DTOs), LEAN compression types,
//! Model Context Protocol (MCP) envelopes, and JSON-RPC domain error definitions
//! for the Souls Engine v7.
//!
//! Bare-metal foundation crate: pure types, zero I/O side effects, zero unsafe code.

#![forbid(unsafe_code)]

pub mod dto;
pub mod error;
pub mod google;
pub mod lean;
pub mod mcp;
pub mod multimodal;

pub use dto::*;
pub use error::*;
pub use google::*;
pub use lean::*;
pub use mcp::*;
pub use multimodal::*;
