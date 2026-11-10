//! # Google Universe Bridge & NotebookLM Reverse Engineering (`souls_core::google_bridge`)
//!
//! Provides encrypted local credential vaults, silent session management,
//! Wiz `batchexecute` RPC protocol emulation for Google NotebookLM,
//! and Jules-to-Antigravity (`agy`) fleet migration.

pub mod dual_workspace;
pub mod session;
pub mod wiz_rpc;

pub use dual_workspace::{DualLayerWorkspace, JulesMigrationRouter, LegacyJulesCommand};
pub use session::{GoogleSessionCredentials, GoogleSessionVault};
pub use wiz_rpc::{WizRpcClient, WizRpcId, WizRpcPayload, WizRpcResponse};

