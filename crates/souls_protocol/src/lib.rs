//! # Souls Protocol (`souls_protocol`)
//!
//! Universal Data Transfer Objects (DTOs), FinOps metrics, hardware telemetry,
//! and MCP JSON-RPC domain error definitions for the Souls Engine v7.
//!
//! Bare-metal foundation crate: pure types, zero I/O side effects, zero unsafe code.

#![forbid(unsafe_code)]

pub mod dto;
pub mod error;

pub use dto::*;
pub use error::*;
