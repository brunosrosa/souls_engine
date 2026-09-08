//! Structured asynchronous logging and ANSI sanitization pipeline.
//!
//! Provides non-blocking tracing setup and terminal buffer purification
//! for the Souls Engine bare-metal infrastructure.

pub use crate::ansi_filter::{ansi_density, strip_ansi, strip_ansi_escapes};
use crate::error::CoreError;
use tracing::Level;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::EnvFilter;

/// Initializes structured logging using tracing-subscriber.
///
/// Configures filtering by log_level (e.g. "info", "debug", "trace")
/// and ensures stdout/stderr logs are cleanly formatted.
pub fn init_tracing(log_level: &str) -> Result<(), CoreError> {
    let level = match log_level.to_ascii_lowercase().as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    };

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(format!("{level},souls_core={level}")));

    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_span_events(FmtSpan::CLOSE)
        .with_target(true)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .map_err(|e| CoreError::TracingInitError(e.to_string()))?;

    Ok(())
}

/// Sanitizes a log message or terminal input buffer by removing all ANSI VT100 escapes.
#[inline]
pub fn sanitize_terminal_buffer(raw: &str) -> String {
    strip_ansi_escapes(raw)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_terminal_buffer_powershell_dense() {
        let dense_ansi = "\x1b[38;2;255;100;0m[WARN]\x1b[0m \x1b[1mNVMe high temperature detected:\x1b[0m \x1b[4m68C\x1b[0m\x1b[2K\r\n";
        let sanitized = sanitize_terminal_buffer(dense_ansi);
        assert_eq!(sanitized, "[WARN] NVMe high temperature detected: 68C\r\n");
        assert!(!sanitized.contains('\x1b'));
    }

    #[test]
    fn test_strip_ansi_escapes_reexport() {
        let input = "\x1b[32mSUCCESS\x1b[0m: Operation completed";
        assert_eq!(strip_ansi_escapes(input), "SUCCESS: Operation completed");
    }
}
