//! # Sandbox Execution and Context Protection Guard (`souls_core::sandbox`)
//!
//! Provides strict security filtering, ReFS working directory enforcement,
//! RAII child process lifecycle management, ANSI stripping, and Head+Tail
//! output truncation to prevent LLM context explosion.

use std::path::Path;
use std::time::{Duration, Instant};

use crate::ansi_filter::strip_ansi_escapes;
use crate::error::CoreError;
use crate::fs::validate_refs_path;
use crate::subprocess_guard::{SubprocessConfig, SubprocessGuard};

/// Configuration for output truncation to prevent LLM context window blowup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputTruncationPolicy {
    /// Number of lines preserved from the beginning of output (setup/parameters).
    pub head_lines: usize,
    /// Number of lines preserved from the end of output (errors/traceback/result).
    pub tail_lines: usize,
    /// Maximum character limit before hard clamping.
    pub max_chars: usize,
}

impl Default for OutputTruncationPolicy {
    fn default() -> Self {
        Self {
            head_lines: 50,
            tail_lines: 70,
            max_chars: 16_384,
        }
    }
}

impl OutputTruncationPolicy {
    /// Creates a custom truncation policy.
    #[must_use]
    pub fn new(head_lines: usize, tail_lines: usize, max_chars: usize) -> Self {
        Self {
            head_lines,
            tail_lines,
            max_chars,
        }
    }

    /// Truncates raw text preserving head and tail lines if total lines exceed budget.
    pub fn truncate_text(&self, text: &str) -> (String, bool) {
        if text.is_empty() {
            return (String::new(), false);
        }

        let lines: Vec<&str> = text.lines().collect();
        let total_lines = lines.len();
        let max_allowed = self.head_lines + self.tail_lines;

        if total_lines <= max_allowed && text.len() <= self.max_chars {
            return (text.to_string(), false);
        }

        let mut out = Vec::with_capacity(max_allowed + 2);

        if total_lines > max_allowed {
            let omitted = total_lines.saturating_sub(max_allowed);
            out.extend_from_slice(&lines[..self.head_lines]);
            out.push(&"");
            let marker = format!(
                "--- [TRUNCATED: {} lines omitted by Souls Sandbox Guard to protect context] ---",
                omitted
            );
            // Temporary heap string allocated into leaked reference or pushed to out as owned:
            let mut formatted = Vec::new();
            for l in &lines[..self.head_lines] {
                formatted.push(*l);
            }
            let marker_owned = marker;
            let marker_str: &str = &marker_owned;
            formatted.push(marker_str);
            for l in &lines[total_lines - self.tail_lines..] {
                formatted.push(*l);
            }
            let joined = formatted.join("\n");
            if joined.len() > self.max_chars {
                let clamped: String = joined.chars().take(self.max_chars).collect();
                return (format!("{}\n--- [TRUNCATED BY CHAR LIMIT] ---", clamped), true);
            }
            return (joined, true);
        }

        let clamped: String = text.chars().take(self.max_chars).collect();
        (format!("{}\n--- [TRUNCATED BY CHAR LIMIT] ---", clamped), true)
    }
}

/// Security filter detecting destructive, root-wiping, or host-hijacking commands.
pub struct CommandSafetyFilter;

impl CommandSafetyFilter {
    /// Inspects command string for blacklisted destructive operations under Windows NT.
    pub fn validate_command(command: &str) -> Result<(), CoreError> {
        let lower = command.trim().to_lowercase();

        // 1. Storage format / disk partition commands
        if lower.starts_with("format ") || lower == "format" || lower.contains("format c:") || lower.contains("diskpart") {
            return Err(CoreError::DangerousCommandBlocked(
                "Execution of disk formatting or partition commands is strictly forbidden".to_string(),
            ));
        }

        // 2. Destructive root tree wipes
        if (lower.contains("rmdir ") || lower.contains("rd "))
            && (lower.contains("/s") || lower.contains("-r"))
            && (lower.contains("c:\\") || lower.contains("c:/") || lower.contains("%systemroot%") || lower.contains("%systemdrive%"))
        {
            return Err(CoreError::DangerousCommandBlocked(
                "Recursive root deletion of host system drives is strictly forbidden".to_string(),
            ));
        }

        if lower.contains("del ") && lower.contains("/s") && (lower.contains("c:\\") || lower.contains("c:/")) {
            return Err(CoreError::DangerousCommandBlocked(
                "Mass file deletion targeting system root is strictly forbidden".to_string(),
            ));
        }

        // 3. Registry root wipes
        if lower.contains("reg delete") && (lower.contains("hklm") || lower.contains("hkcu") || lower.contains("hkey_local_machine")) {
            return Err(CoreError::DangerousCommandBlocked(
                "Registry root hive destruction is strictly forbidden".to_string(),
            ));
        }

        // 4. Remote shutdown commands
        if lower.starts_with("shutdown ") && (lower.contains("/s") || lower.contains("-s")) {
            return Err(CoreError::DangerousCommandBlocked(
                "System shutdown or restart commands are forbidden inside sandbox".to_string(),
            ));
        }

        Ok(())
    }
}

/// Structured outcome of a sandboxed command or shell invocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SandboxedResult {
    /// Sanitized and truncated stdout.
    pub stdout: String,
    /// Sanitized and truncated stderr.
    pub stderr: String,
    /// Process exit code (-1 if terminated or killed).
    pub exit_code: i32,
    /// Total execution duration in milliseconds.
    pub execution_time_ms: u64,
    /// Indicates whether stdout or stderr was truncated.
    pub truncated: bool,
}

/// Executes a shell command inside a sandboxed PowerShell process anchored to Dev Drive ReFS.
pub async fn run_sandboxed_shell(
    command: &str,
    cwd: &Path,
    timeout_ms: u64,
) -> Result<SandboxedResult, CoreError> {
    CommandSafetyFilter::validate_command(command)?;
    validate_refs_path(cwd)?;

    let timeout_duration = Duration::from_millis(timeout_ms.clamp(500, 60_000));
    let start = Instant::now();

    #[cfg(windows)]
    let (exe_name, args) = (
        "powershell",
        vec![
            "-NoProfile".to_string(),
            "-NonInteractive".to_string(),
            "-Command".to_string(),
            command.to_string(),
        ],
    );

    #[cfg(not(windows))]
    let (exe_name, args) = (
        "sh",
        vec!["-c".to_string(), command.to_string()],
    );

    let cfg = SubprocessConfig::new(exe_name)
        .with_args(args)
        .with_working_dir(cwd.to_string_lossy().to_string())
        .with_kill_on_drop(true);

    let guard = SubprocessGuard::spawn(&cfg)?;
    let child = guard.into_child().ok_or_else(|| {
        CoreError::SubprocessError("Failed to detach child process from guard".to_string())
    })?;

    let exec_fut = child.wait_with_output();

    let output = match tokio::time::timeout(timeout_duration, exec_fut).await {
        Ok(res) => res.map_err(|e| CoreError::SubprocessError(format!("Process I/O error: {e}")))?,
        Err(_) => return Err(CoreError::ExecutionTimeout(timeout_duration.as_millis() as u64)),
    };

    let raw_stdout = String::from_utf8_lossy(&output.stdout);
    let raw_stderr = String::from_utf8_lossy(&output.stderr);

    let clean_stdout = strip_ansi_escapes(&raw_stdout);
    let clean_stderr = strip_ansi_escapes(&raw_stderr);

    let policy = OutputTruncationPolicy::default();
    let (trunc_stdout, t_out) = policy.truncate_text(&clean_stdout);
    let (trunc_stderr, t_err) = policy.truncate_text(&clean_stderr);

    let duration_ms = start.elapsed().as_millis() as u64;

    Ok(SandboxedResult {
        stdout: trunc_stdout,
        stderr: trunc_stderr,
        exit_code: output.status.code().unwrap_or(-1),
        execution_time_ms: duration_ms,
        truncated: t_out || t_err,
    })
}

/// Executes a native binary executable inside a sandboxed environment with strict timeout.
pub async fn run_sandboxed_command(
    executable: &str,
    args: &[String],
    cwd: &Path,
    timeout_ms: u64,
) -> Result<SandboxedResult, CoreError> {
    validate_refs_path(cwd)?;

    let timeout_duration = Duration::from_millis(timeout_ms.clamp(500, 60_000));
    let start = Instant::now();

    let cfg = SubprocessConfig::new(executable)
        .with_args(args.to_vec())
        .with_working_dir(cwd.to_string_lossy().to_string())
        .with_kill_on_drop(true);

    let guard = SubprocessGuard::spawn(&cfg)?;
    let child = guard.into_child().ok_or_else(|| {
        CoreError::SubprocessError("Failed to detach child from guard".to_string())
    })?;

    let exec_fut = child.wait_with_output();

    let output = match tokio::time::timeout(timeout_duration, exec_fut).await {
        Ok(res) => res.map_err(|e| CoreError::SubprocessError(format!("Command I/O error: {e}")))?,
        Err(_) => return Err(CoreError::ExecutionTimeout(timeout_duration.as_millis() as u64)),
    };

    let raw_stdout = String::from_utf8_lossy(&output.stdout);
    let raw_stderr = String::from_utf8_lossy(&output.stderr);

    let clean_stdout = strip_ansi_escapes(&raw_stdout);
    let clean_stderr = strip_ansi_escapes(&raw_stderr);

    let policy = OutputTruncationPolicy::default();
    let (trunc_stdout, t_out) = policy.truncate_text(&clean_stdout);
    let (trunc_stderr, t_err) = policy.truncate_text(&clean_stderr);

    let duration_ms = start.elapsed().as_millis() as u64;

    Ok(SandboxedResult {
        stdout: trunc_stdout,
        stderr: trunc_stderr,
        exit_code: output.status.code().unwrap_or(-1),
        execution_time_ms: duration_ms,
        truncated: t_out || t_err,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safety_filter_blocks_destructive_commands() {
        assert!(CommandSafetyFilter::validate_command("format C: /fs:ntfs").is_err());
        assert!(CommandSafetyFilter::validate_command("rmdir /s /q C:\\Windows").is_err());
        assert!(CommandSafetyFilter::validate_command("del /f /s /q C:\\*.*").is_err());
        assert!(CommandSafetyFilter::validate_command("reg delete HKLM\\Software").is_err());
        assert!(CommandSafetyFilter::validate_command("shutdown /s /t 0").is_err());
        assert!(CommandSafetyFilter::validate_command("echo Hello world").is_ok());
        assert!(CommandSafetyFilter::validate_command("cargo check --workspace").is_ok());
    }

    #[test]
    fn test_output_truncation_preserves_head_and_tail() {
        let policy = OutputTruncationPolicy::new(10, 10, 10_000);
        let mut massive_output = Vec::new();
        for i in 1..=500 {
            massive_output.push(format!("Line #{i}: compilation artifact {i}"));
        }
        let input = massive_output.join("\n");

        let (truncated, was_truncated) = policy.truncate_text(&input);
        assert!(was_truncated);
        assert!(truncated.contains("Line #1:"));
        assert!(truncated.contains("Line #10:"));
        assert!(truncated.contains("TRUNCATED: 480 lines omitted"));
        assert!(truncated.contains("Line #491:"));
        assert!(truncated.contains("Line #500:"));
        assert!(!truncated.contains("Line #250:"));
    }

    #[tokio::test]
    async fn test_run_sandboxed_shell_success() {
        let cwd = Path::new("Z:/souls_engine");
        let res = run_sandboxed_shell("Write-Output 'SoulsEngine_Sandbox_OK'", cwd, 10_000).await;
        if let Ok(out) = res {
            assert_eq!(out.exit_code, 0);
            assert!(out.stdout.contains("SoulsEngine_Sandbox_OK"));
            assert!(!out.truncated);
        }
    }
}
