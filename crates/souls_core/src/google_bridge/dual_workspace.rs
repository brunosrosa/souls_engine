//! Dual-Layer Workspace coordinator and Jules-to-Antigravity (`agy`) fleet migration.
//!
//! Replaces legacy Google Jules (decommissioned on 29/10/2026) by transparently routing
//! code generation, refactoring, and file edit tasks to the native Antigravity CLI (`agy`).

use crate::harness::{AgenticCliCommand, AgenticCliKind};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tracing::{info, warn};

/// Dual-layer workspace state binding a local Dev Drive repository to Google Drive assets.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DualLayerWorkspace {
    /// Local Dev Drive absolute root path (e.g. `Z:\souls_engine`).
    pub local_root: PathBuf,
    /// Associated Google Drive folder identifier.
    pub google_drive_folder_id: Option<String>,
    /// Associated Google Docs / Sheets resource identifiers for sync.
    pub linked_document_ids: Vec<String>,
    /// Whether local ReFS block cloning is active.
    pub refs_block_cloning_enabled: bool,
}

impl DualLayerWorkspace {
    /// Initializes a new dual-layer workspace on the Dev Drive.
    pub fn new(local_root: impl Into<PathBuf>) -> Self {
        Self {
            local_root: local_root.into(),
            google_drive_folder_id: None,
            linked_document_ids: Vec::new(),
            refs_block_cloning_enabled: true,
        }
    }

    /// Associates a Google Drive folder ID for source synchronization.
    pub fn with_drive_folder(mut self, folder_id: impl Into<String>) -> Self {
        self.google_drive_folder_id = Some(folder_id.into());
        self
    }

    /// Links a Google Doc or Sheet document ID.
    pub fn with_linked_document(mut self, doc_id: impl Into<String>) -> Self {
        self.linked_document_ids.push(doc_id.into());
        self
    }
}

/// Representation of a legacy Google Jules task submission.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LegacyJulesCommand {
    pub prompt: String,
    pub working_dir: Option<PathBuf>,
    pub auto_approve: bool,
    pub model: Option<String>,
}

/// Migration router transparently converting legacy Google Jules commands into Antigravity (`agy`) executions.
pub struct JulesMigrationRouter;

impl JulesMigrationRouter {
    /// Decommission notice date of Google Jules.
    pub const JULES_DECOMMISSION_DATE: &'static str = "2026-10-29";

    /// Checks if a command line string refers to the legacy Jules binary.
    pub fn is_legacy_jules_command(command_line: &str) -> bool {
        let trimmed = command_line.trim_start();
        trimmed.starts_with("jules ") || trimmed.starts_with("jules.exe ") || trimmed == "jules"
    }

    /// Converts a legacy Jules task request into a canonical `AgenticCliCommand` for Antigravity (`agy`).
    pub fn migrate_to_antigravity(
        jules_req: LegacyJulesCommand,
        default_working_dir: &Path,
    ) -> AgenticCliCommand {
        warn!(
            target: "souls_core::google_bridge",
            "Google Jules was decommissioned on {}. Transparently routing task to Antigravity CLI (agy).",
            Self::JULES_DECOMMISSION_DATE
        );

        let cwd = jules_req
            .working_dir
            .unwrap_or_else(|| default_working_dir.to_path_buf());

        let mut cmd = AgenticCliCommand::new(AgenticCliKind::Antigravity, jules_req.prompt, cwd)
            .with_timeout_secs(300)
            .with_auto_restore(true);

        if let Some(m) = jules_req.model {
            cmd = cmd.with_model(m);
        }

        if jules_req.auto_approve {
            cmd = cmd.with_extra_args(vec!["--dangerously-skip-permissions".to_string()]);
        }

        info!(
            target: "souls_core::google_bridge",
            "Transformed Jules request -> Antigravity CLI (agy) command successfully."
        );

        cmd
    }
}

