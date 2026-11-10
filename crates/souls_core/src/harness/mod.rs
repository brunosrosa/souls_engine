//! # Headless Agentic CLI Harness (`souls_core::harness`)
//!
//! Subprocess dispatch, streaming NDJSON telemetry, and transactional git restoration
//! for external agentic CLIs (`agy`, `claude`, `codex`).

pub mod agentic_cli;

pub use agentic_cli::{
    detect_installed_clis, AgenticCliCommand, AgenticCliInfo, AgenticCliKind,
    AgenticHarnessDispatcher, HarnessEvent, HarnessOutput,
};
