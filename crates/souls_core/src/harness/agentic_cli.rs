//! Subprocess harness for running headless external agentic CLIs.
//!
//! Provides automated discovery, parameter negotiation, NDJSON streaming event parsing,
//! timeout containment, and transactional git rollback on failure.

use crate::error::CoreError;
use crate::subprocess_guard::SubprocessConfig;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::{Duration, Instant};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tracing::{error, info, warn};

/// Canonical classification of supported external agentic CLIs.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgenticCliKind {
    /// Google Antigravity CLI (`agy`).
    Antigravity,
    /// Anthropic Claude Code CLI (`claude`).
    ClaudeCode,
    /// OpenAI Codex CLI (`codex`).
    Codex,
    /// Custom external CLI executable.
    Custom(String),
}

impl AgenticCliKind {
    /// Canonical binary executable name.
    #[inline]
    pub fn binary_name(&self) -> &str {
        match self {
            Self::Antigravity => "agy",
            Self::ClaudeCode => "claude",
            Self::Codex => "codex",
            Self::Custom(name) => name.as_str(),
        }
    }

    /// Canonical display title.
    #[inline]
    pub fn display_name(&self) -> &str {
        match self {
            Self::Antigravity => "Antigravity CLI (agy)",
            Self::ClaudeCode => "Claude Code (claude)",
            Self::Codex => "Codex CLI (codex)",
            Self::Custom(name) => name.as_str(),
        }
    }

    /// Supported operational execution modes.
    pub fn supported_modes(&self) -> Vec<String> {
        match self {
            Self::Antigravity => vec!["accept-edits".to_string(), "plan".to_string()],
            Self::ClaudeCode => vec!["acceptEdits".to_string(), "default".to_string()],
            Self::Codex => vec!["workspace-write".to_string(), "read-only".to_string()],
            Self::Custom(_) => vec!["default".to_string()],
        }
    }
}

/// Discovered metadata and operational state of an agentic CLI on the host.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgenticCliInfo {
    pub kind: AgenticCliKind,
    pub binary_name: String,
    pub display_name: String,
    pub resolved_path: Option<PathBuf>,
    pub is_installed: bool,
    pub version: Option<String>,
    pub supported_modes: Vec<String>,
}

/// Invocation parameters for dispatching an agentic task to a headless CLI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgenticCliCommand {
    pub cli: AgenticCliKind,
    pub prompt: String,
    pub working_dir: PathBuf,
    pub model: Option<String>,
    pub reasoning_effort: Option<String>,
    pub timeout_secs: u64,
    pub auto_restore_on_failure: bool,
    pub extra_args: Vec<String>,
}

impl AgenticCliCommand {
    /// Creates a new default command for the specified CLI and prompt.
    pub fn new(cli: AgenticCliKind, prompt: impl Into<String>, working_dir: PathBuf) -> Self {
        Self {
            cli,
            prompt: prompt.into(),
            working_dir,
            model: None,
            reasoning_effort: None,
            timeout_secs: 300,
            auto_restore_on_failure: true,
            extra_args: Vec::new(),
        }
    }

    /// Sets target LLM model identifier.
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    /// Sets reasoning effort (e.g. "low", "medium", "high").
    pub fn with_reasoning_effort(mut self, effort: impl Into<String>) -> Self {
        self.reasoning_effort = Some(effort.into());
        self
    }

    /// Sets execution timeout in seconds.
    pub fn with_timeout_secs(mut self, timeout_secs: u64) -> Self {
        self.timeout_secs = timeout_secs;
        self
    }

    /// Configures automatic transactional git restore if the subprocess exits with non-zero.
    pub fn with_auto_restore(mut self, auto_restore: bool) -> Self {
        self.auto_restore_on_failure = auto_restore;
        self
    }

    /// Adds extra arguments passed directly to the binary.
    pub fn with_extra_args(mut self, args: Vec<String>) -> Self {
        self.extra_args = args;
        self
    }
}

/// Parsed NDJSON stream event emitted during CLI execution.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HarnessEvent {
    pub event_type: String,
    pub raw_json: Option<serde_json::Value>,
    pub text_snippet: Option<String>,
    pub timestamp_ms: i64,
}

/// Comprehensive outcome of a headless agentic CLI run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarnessOutput {
    pub cli: AgenticCliKind,
    pub exit_code: i32,
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
    pub events: Vec<HarnessEvent>,
    pub restored_files: Vec<String>,
}

/// Pure helper to resolve CLI binary path including common Windows appdata directories.
pub fn resolve_cli_executable(binary_name: &str) -> Option<PathBuf> {
    // 1. Direct subprocess config probe (handles CWD, .agents/bin, and PATH)
    let cfg = SubprocessConfig::new(binary_name);
    if let Ok(resolved) = cfg.resolve_executable_path() {
        return Some(resolved);
    }

    // 2. Windows specific AppData fallbacks for known CLIs
    #[cfg(windows)]
    {
        if let Ok(local_appdata) = std::env::var("LOCALAPPDATA") {
            let local_base = PathBuf::from(local_appdata);
            let candidates = match binary_name {
                "agy" => vec![
                    local_base.join("agy").join("bin").join("agy.exe"),
                    local_base.join("Programs").join("agy").join("agy.exe"),
                ],
                "claude" => vec![
                    local_base.join("Programs").join("claude").join("claude.exe"),
                    local_base.join("npm").join("claude.cmd"),
                ],
                "codex" => vec![
                    local_base.join("Programs").join("codex").join("codex.exe"),
                    local_base.join("npm").join("codex.cmd"),
                ],
                _ => Vec::new(),
            };

            for cand in candidates {
                if cand.exists() {
                    return Some(cand);
                }
            }
        }

        if let Ok(appdata) = std::env::var("APPDATA") {
            let roaming_base = PathBuf::from(appdata);
            let npm_cand = roaming_base.join("npm").join(format!("{binary_name}.cmd"));
            if npm_cand.exists() {
                return Some(npm_cand);
            }
        }
    }

    None
}

/// Probes an agentic CLI on the host, capturing version if available.
pub async fn probe_agentic_cli(kind: AgenticCliKind) -> AgenticCliInfo {
    let binary = kind.binary_name().to_string();
    let display = kind.display_name().to_string();
    let modes = kind.supported_modes();
    let resolved = resolve_cli_executable(&binary);

    let is_installed = resolved.is_some();
    let mut version = None;

    if let Some(ref path) = resolved {
        // Attempt quick non-blocking version check
        let version_cmd = Command::new(path)
            .arg("--version")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .output();

        if let Ok(Ok(out)) = tokio::time::timeout(Duration::from_secs(3), version_cmd).await {
            if out.status.success() {
                let v = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if !v.is_empty() {
                    version = Some(v);
                }
            }
        }
    }

    AgenticCliInfo {
        kind,
        binary_name: binary,
        display_name: display,
        resolved_path: resolved,
        is_installed,
        version,
        supported_modes: modes,
    }
}

/// Discovers all canonical agentic CLIs installed on the host machine.
pub async fn detect_installed_clis() -> Vec<AgenticCliInfo> {
    let targets = vec![
        AgenticCliKind::Antigravity,
        AgenticCliKind::ClaudeCode,
        AgenticCliKind::Codex,
    ];

    let mut results = Vec::with_capacity(targets.len());
    for target in targets {
        results.push(probe_agentic_cli(target).await);
    }
    results
}

/// Core harness dispatcher executing external agentic CLIs in headless mode.
pub struct AgenticHarnessDispatcher;

impl AgenticHarnessDispatcher {
    /// Builds canonical command-line arguments for a given CLI and parameters.
    pub fn build_arguments(cmd: &AgenticCliCommand) -> Vec<String> {
        let mut args = Vec::new();

        match &cmd.cli {
            AgenticCliKind::Antigravity => {
                args.push("-p".to_string());
                args.push(cmd.prompt.clone());
                args.push("--mode=accept-edits".to_string());
                args.push("--output-format=stream-json".to_string());
                args.push("--dangerously-skip-permissions".to_string());

                if let Some(ref model) = cmd.model {
                    args.push("--model".to_string());
                    args.push(model.clone());
                }
                if let Some(ref effort) = cmd.reasoning_effort {
                    args.push("--effort".to_string());
                    args.push(effort.clone());
                }
            }
            AgenticCliKind::ClaudeCode => {
                args.push("-p".to_string());
                args.push(cmd.prompt.clone());
                args.push("--bare".to_string());
                args.push("--permission-mode".to_string());
                args.push("acceptEdits".to_string());
                args.push("--output-format".to_string());
                args.push("stream-json".to_string());

                if let Some(ref model) = cmd.model {
                    args.push("--model".to_string());
                    args.push(model.clone());
                }
            }
            AgenticCliKind::Codex => {
                args.push("exec".to_string());
                args.push(cmd.prompt.clone());
                args.push("--sandbox".to_string());
                args.push("workspace-write".to_string());
                args.push("--ephemeral".to_string());
                args.push("--json".to_string());

                if let Some(ref model) = cmd.model {
                    args.push("--model".to_string());
                    args.push(model.clone());
                }
            }
            AgenticCliKind::Custom(_) => {
                args.push("-p".to_string());
                args.push(cmd.prompt.clone());
            }
        }

        args.extend(cmd.extra_args.clone());
        args
    }

    /// Dispatches an agentic task to the requested CLI with streaming NDJSON capture and timeout.
    pub async fn dispatch(cmd: &AgenticCliCommand) -> Result<HarnessOutput, CoreError> {
        let binary_name = cmd.cli.binary_name();
        let executable_path = resolve_cli_executable(binary_name).ok_or_else(|| {
            CoreError::SubprocessError(format!(
                "Agentic CLI executable '{}' was not found on the host machine",
                binary_name
            ))
        })?;

        let args = Self::build_arguments(cmd);
        let start_time = Instant::now();

        info!(
            target: "souls_core::harness",
            "Dispatching agentic CLI task: exe={} args={:?} cwd={}",
            executable_path.display(),
            args,
            cmd.working_dir.display()
        );

        let mut process = Command::new(&executable_path);
        process
            .args(&args)
            .current_dir(&cmd.working_dir)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);

        let mut child = process.spawn().map_err(|e| {
            CoreError::SubprocessError(format!(
                "Failed to spawn agentic CLI '{}': {e}",
                executable_path.display()
            ))
        })?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| CoreError::SubprocessError("Failed to capture stdout pipe".to_string()))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| CoreError::SubprocessError("Failed to capture stderr pipe".to_string()))?;

        let mut stdout_reader = BufReader::new(stdout).lines();
        let mut stderr_reader = BufReader::new(stderr).lines();

        let mut stdout_accum = String::new();
        let mut stderr_accum = String::new();
        let mut events = Vec::new();

        let timeout_duration = Duration::from_secs(cmd.timeout_secs);
        let timeout_future = tokio::time::sleep(timeout_duration);
        tokio::pin!(timeout_future);

        let mut child_exited = false;
        let mut exit_status = None;

        loop {
            tokio::select! {
                _ = &mut timeout_future => {
                    let _ = child.start_kill();
                    error!(
                        target: "souls_core::harness",
                        "Agentic CLI execution timed out after {} seconds: killing pid {:?}",
                        cmd.timeout_secs,
                        child.id()
                    );
                    return Err(CoreError::SubprocessError(format!(
                        "Agentic CLI '{}' timed out after {} seconds",
                        binary_name,
                        cmd.timeout_secs
                    )));
                }
                line_res = stdout_reader.next_line() => {
                    match line_res {
                        Ok(Some(line)) => {
                            let now_ms = Utc::now().timestamp_millis();
                            stdout_accum.push_str(&line);
                            stdout_accum.push('\n');

                            // Parse NDJSON event if valid
                            if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(&line) {
                                let event_type = json_val.get("type")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("event")
                                    .to_string();
                                events.push(HarnessEvent {
                                    event_type,
                                    raw_json: Some(json_val),
                                    text_snippet: None,
                                    timestamp_ms: now_ms,
                                });
                            } else {
                                events.push(HarnessEvent {
                                    event_type: "text".to_string(),
                                    raw_json: None,
                                    text_snippet: Some(line),
                                    timestamp_ms: now_ms,
                                });
                            }
                        }
                        Ok(None) => {
                            // Stdout EOF
                            if child_exited {
                                break;
                            }
                        }
                        Err(e) => {
                            warn!(target: "souls_core::harness", "Error reading stdout line: {e}");
                        }
                    }
                }
                err_line_res = stderr_reader.next_line() => {
                    if let Ok(Some(line)) = err_line_res {
                        if stderr_accum.len() < 65_536 {
                            stderr_accum.push_str(&line);
                            stderr_accum.push('\n');
                        }
                    }
                }
                status_res = child.wait(), if !child_exited => {
                    match status_res {
                        Ok(status) => {
                            exit_status = Some(status);
                            child_exited = true;
                        }
                        Err(e) => {
                            return Err(CoreError::SubprocessError(format!("Error waiting for CLI child: {e}")));
                        }
                    }
                }
            }
        }

        let duration_ms = start_time.elapsed().as_millis() as u64;
        let status = exit_status.ok_or_else(|| CoreError::SubprocessError("Child process exited without status".to_string()))?;
        let exit_code = status.code().unwrap_or(-1);
        let success = status.success();

        let mut restored_files = Vec::new();
        if !success && cmd.auto_restore_on_failure {
            warn!(
                target: "souls_core::harness",
                "Agentic CLI exited with error code {}. Triggering transactional git restore in {}",
                exit_code,
                cmd.working_dir.display()
            );
            restored_files = Self::git_restore_dirty_files(&cmd.working_dir).await;
        }

        info!(
            target: "souls_core::harness",
            "Agentic CLI finished: exit_code={} success={} duration_ms={} events={} restored={}",
            exit_code,
            success,
            duration_ms,
            events.len(),
            restored_files.len()
        );

        Ok(HarnessOutput {
            cli: cmd.cli.clone(),
            exit_code,
            success,
            stdout: stdout_accum,
            stderr: stderr_accum,
            duration_ms,
            events,
            restored_files,
        })
    }

    /// Transactionally rolls back any unstaged/staged dirty git files caused by a failed execution.
    pub async fn git_restore_dirty_files(working_dir: &Path) -> Vec<String> {
        let status_output = match Command::new("git")
            .arg("status")
            .arg("--porcelain")
            .current_dir(working_dir)
            .output()
            .await
        {
            Ok(out) if out.status.success() => String::from_utf8_lossy(&out.stdout).to_string(),
            _ => return Vec::new(),
        };

        let mut dirty_files = Vec::new();
        for line in status_output.lines() {
            if line.len() >= 4 {
                // git status --porcelain outputs "XY filename" (2 chars status + 1 char space + filename)
                let file_path = line[3..].trim().to_string();
                if !file_path.is_empty() {
                    dirty_files.push(file_path);
                }
            }
        }

        if dirty_files.is_empty() {
            return Vec::new();
        }

        let _ = Command::new("git")
            .arg("restore")
            .arg("--staged")
            .arg("--worktree")
            .arg(".")
            .current_dir(working_dir)
            .output()
            .await;

        dirty_files
    }
}
