//! RAII Subprocess Guard and lifecycle owner for Souls Engine (v7).
//!
//! Encapsulates `tokio::process::Child` in a RAII guard that sends atomic termination
//! on `Drop`, eradicating zombie processes in host memory.
//!
//! Conforms to CRATES_SPECIFICATION.md Section 2.2 and AGENTS.md Line 3.

use crate::error::CoreError;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::{Child, Command};

/// Canonical configuration for spawning a protected subprocess.
#[derive(Debug, Clone)]
pub struct SubprocessConfig {
    pub executable_path: String,
    pub args: Vec<String>,
    pub working_dir: String,
    pub kill_on_drop: bool,
}

impl SubprocessConfig {
    /// Creates a new configuration for an executable with kill-on-drop enabled by default.
    #[must_use]
    pub fn new(executable_path: impl Into<String>) -> Self {
        Self {
            executable_path: executable_path.into(),
            args: Vec::new(),
            working_dir: String::new(),
            kill_on_drop: true,
        }
    }

    /// Sets CLI arguments for the process.
    #[must_use]
    pub fn with_args(mut self, args: Vec<String>) -> Self {
        self.args = args;
        self
    }

    /// Sets the working directory for the process.
    #[must_use]
    pub fn with_working_dir(mut self, working_dir: impl Into<String>) -> Self {
        self.working_dir = working_dir.into();
        self
    }

    /// Controls whether the child process is killed when the guard is dropped.
    #[must_use]
    pub fn with_kill_on_drop(mut self, kill_on_drop: bool) -> Self {
        self.kill_on_drop = kill_on_drop;
        self
    }

    /// Resolves the actual executable path, searching in:
    /// 1. Direct path / absolute path (+ `.exe` on Windows).
    /// 2. Current Working Directory (CWD).
    /// 3. `.agents/bin/<name>`.
    /// 4. Every directory in `$PATH`.
    pub fn resolve_executable_path(&self) -> Result<PathBuf, CoreError> {
        let exec = self.executable_path.trim();
        if exec.is_empty() {
            return Err(CoreError::SubprocessError(
                "executable_path cannot be empty".to_string(),
            ));
        }

        let cwd = std::env::current_dir()
            .map_err(|e| CoreError::SubprocessError(format!("Failed to retrieve CWD: {e}")))?;

        resolve_executable(exec, &cwd).ok_or_else(|| {
            CoreError::SubprocessError(format!(
                "Executable '{}' was not found in CWD, .agents/bin, or $PATH",
                exec
            ))
        })
    }

    /// Validates that the executable exists.
    pub fn validate(&self) -> Result<(), CoreError> {
        self.resolve_executable_path().map(|_| ())
    }
}

/// Pure resolution function locating an executable across path candidates.
fn resolve_executable(exec: &str, cwd: &Path) -> Option<PathBuf> {
    let exec_clean = if exec.starts_with("${") && exec.ends_with('}') {
        "souls_server"
    } else {
        exec
    };

    let has_separator = exec_clean.contains('/') || exec_clean.contains('\\');
    let is_absolute = Path::new(exec_clean).is_absolute();

    if is_absolute {
        return try_with_exe_extension(Path::new(exec_clean));
    }

    if has_separator {
        if let Some(found) = try_with_exe_extension(&cwd.join(exec_clean)) {
            return Some(found);
        }
        return try_with_exe_extension(Path::new(exec_clean));
    }

    // Direct CWD probe
    if let Some(found) = try_with_exe_extension(&cwd.join(exec_clean)) {
        return Some(found);
    }

    // Probe in .agents/bin/<name>
    if let Some(found) = try_with_exe_extension(&cwd.join(".agents").join("bin").join(exec_clean)) {
        return Some(found);
    }

    // Search PATH entries
    if let Ok(path_env) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path_env) {
            if let Some(found) = try_with_exe_extension(&dir.join(exec_clean)) {
                return Some(found);
            }
        }
    }

    None
}

/// Checks if path exists, appending `.exe` on Windows if missing.
fn try_with_exe_extension(p: &Path) -> Option<PathBuf> {
    if p.exists() {
        return Some(p.to_path_buf());
    }
    #[cfg(windows)]
    {
        if let Some(s) = p.to_str() {
            if !s.to_lowercase().ends_with(".exe") {
                let with_exe = format!("{s}.exe");
                let p2 = Path::new(&with_exe);
                if p2.exists() {
                    return Some(p2.to_path_buf());
                }
            }
        }
    }
    None
}

/// RAII lifecycle guard managing a child process.
///
/// Ensures atomic termination on `Drop` (if `kill_on_drop` is true),
/// preventing zombie processes when tasks are cancelled or interrupted.
pub struct SubprocessGuard {
    child: Option<Child>,
    pid: Option<u32>,
    kill_on_drop: bool,
    label: String,
}

impl SubprocessGuard {
    /// Spawns a subprocess with piped stdio.
    pub fn spawn(cfg: &SubprocessConfig) -> Result<Self, CoreError> {
        let resolved_path = cfg.resolve_executable_path()?;

        let mut cmd = Command::new(&resolved_path);
        cmd.args(&cfg.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(cfg.kill_on_drop);

        if !cfg.working_dir.trim().is_empty() {
            cmd.current_dir(&cfg.working_dir);
        }

        let child = cmd.spawn().map_err(|e| {
            CoreError::SubprocessError(format!(
                "Failed to spawn subprocess '{}' (resolved: {}): {e}",
                cfg.executable_path,
                resolved_path.display()
            ))
        })?;

        let pid = child.id();
        let label = format!("Proc:{}", resolved_path.display());
        tracing::info!(
            target: "souls_core::subprocess",
            "Subprocess spawned: pid={:?} exe={} args={:?}",
            pid,
            resolved_path.display(),
            cfg.args
        );

        Ok(Self {
            child: Some(child),
            pid,
            kill_on_drop: cfg.kill_on_drop,
            label,
        })
    }

    /// Returns the OS process ID (PID).
    #[must_use]
    pub fn pid(&self) -> Option<u32> {
        self.pid
    }

    /// Mutable reference to the inner `Child` for I/O piping.
    pub fn child_mut(&mut self) -> Option<&mut Child> {
        self.child.as_mut()
    }

    /// Immutable reference to the inner `Child`.
    pub fn child_ref(&self) -> Option<&Child> {
        self.child.as_ref()
    }

    /// Releases ownership of the `Child`, disabling automatic kill on Drop.
    pub fn into_child(mut self) -> Option<Child> {
        self.kill_on_drop = false;
        self.child.take()
    }

    /// Explicit asynchronous termination. Awaits process exit.
    pub async fn kill(mut self) -> io::Result<()> {
        self.kill_on_drop = false;
        if let Some(mut child) = self.child.take() {
            let _ = child.start_kill();
            let _ = child.wait().await;
            tracing::info!(
                target: "souls_core::subprocess",
                "Subprocess killed and reaped: pid={:?} label={}",
                self.pid,
                self.label
            );
        }
        Ok(())
    }

    /// Probes whether the process is still running without consuming the guard.
    pub fn probe_state(&mut self) -> SubprocessState {
        if let Some(child) = self.child.as_mut() {
            match child.try_wait() {
                Ok(Some(status)) => SubprocessState::Exited(status),
                Ok(None) => SubprocessState::Alive,
                Err(e) => SubprocessState::Unknown(e.to_string()),
            }
        } else {
            SubprocessState::Reaped
        }
    }
}

impl Drop for SubprocessGuard {
    fn drop(&mut self) {
        if !self.kill_on_drop {
            return;
        }
        if let Some(mut child) = self.child.take() {
            match child.start_kill() {
                Ok(()) => {
                    tracing::info!(
                        target: "souls_core::subprocess",
                        "SubprocessGuard::drop: atomic kill sent to pid={:?} ({})",
                        self.pid,
                        self.label
                    );
                }
                Err(e) => {
                    tracing::error!(
                        target: "souls_core::subprocess",
                        "SubprocessGuard::drop: failed to kill pid={:?}: {e}",
                        self.pid
                    );
                }
            }
        }
    }
}

/// Observed state of a subprocess via non-blocking probe.
#[derive(Debug)]
pub enum SubprocessState {
    /// Process is currently executing.
    Alive,
    /// Process has terminated with exit status.
    Exited(std::process::ExitStatus),
    /// Error inspecting process status.
    Unknown(String),
    /// Guard has already yielded or reaped the child process.
    Reaped,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subprocess_config_builder() {
        let cfg = SubprocessConfig::new("cmd")
            .with_args(vec!["/c".to_string(), "echo ok".to_string()])
            .with_kill_on_drop(true);

        assert_eq!(cfg.executable_path, "cmd");
        assert_eq!(cfg.args.len(), 2);
        assert!(cfg.kill_on_drop);
    }

    #[test]
    fn test_resolve_system_executable() {
        #[cfg(windows)]
        let binary_name = "cmd.exe";
        #[cfg(not(windows))]
        let binary_name = "sh";

        let cfg = SubprocessConfig::new(binary_name);
        let resolved = cfg.resolve_executable_path();
        assert!(resolved.is_ok(), "Expected to find system shell in PATH");
    }

    #[tokio::test]
    async fn test_spawn_and_kill_subprocess() {
        #[cfg(windows)]
        let cfg = SubprocessConfig::new("cmd").with_args(vec!["/c".to_string(), "ping 127.0.0.1 -n 3".to_string()]);
        #[cfg(not(windows))]
        let cfg = SubprocessConfig::new("sleep").with_args(vec!["3".to_string()]);

        if let Ok(guard) = SubprocessGuard::spawn(&cfg) {
            let pid = guard.pid();
            assert!(pid.is_some());
            let kill_res = guard.kill().await;
            assert!(kill_res.is_ok());
        }
    }
}
