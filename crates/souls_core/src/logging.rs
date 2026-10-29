//! Structured asynchronous logging and ANSI sanitization pipeline.
//!
//! Provides non-blocking tracing setup, terminal buffer purification,
//! and high-throughput MPSC batch log writing with periodic 5-second or 100-event
//! flushes for the Souls Engine bare-metal infrastructure.

pub use crate::ansi_filter::{ansi_density, strip_ansi, strip_ansi_escapes};
use crate::error::CoreError;
use chrono::Utc;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, oneshot};
use tracing::Level;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer};

/// Raw log event captured from tracing or system instrumentation.
#[derive(Debug, Clone)]
pub struct RawLogEvent {
    pub timestamp: chrono::DateTime<Utc>,
    pub level: String,
    pub target: String,
    pub message: String,
}

impl RawLogEvent {
    /// Constructs a new raw log event with cleaned, ANSI-stripped content.
    pub fn new(level: impl Into<String>, target: impl Into<String>, message: impl AsRef<str>) -> Self {
        Self {
            timestamp: Utc::now(),
            level: level.into(),
            target: target.into(),
            message: strip_ansi_escapes(message.as_ref()),
        }
    }

    /// Formats the log entry into canonical line format.
    pub fn format_line(&self) -> String {
        format!(
            "[{}] [{}] [{}] {}\n",
            self.timestamp.to_rfc3339(),
            self.level.to_uppercase(),
            self.target,
            self.message
        )
    }
}

/// Commands delivered through the asynchronous MPSC logging pipeline.
#[derive(Debug)]
pub enum LogCommand {
    Append(RawLogEvent),
    Flush(oneshot::Sender<()>),
    Shutdown(oneshot::Sender<()>),
}

/// Configuration governing asynchronous MPSC batch logging.
#[derive(Debug, Clone)]
pub struct MpscBatchLogConfig {
    pub log_file_path: PathBuf,
    pub batch_flush_interval_secs: u64,
    pub batch_size_threshold: usize,
    pub channel_capacity: usize,
}

impl Default for MpscBatchLogConfig {
    fn default() -> Self {
        Self {
            log_file_path: PathBuf::from(r"Z:\souls_engine\.souls_data\logs\souls_raw.log"),
            batch_flush_interval_secs: 5,
            batch_size_threshold: 100,
            channel_capacity: 4096,
        }
    }
}

/// Asynchronous, non-blocking MPSC batch log writer.
///
/// Decouples file I/O operations from the Tokio execution threads, preventing
/// Windows 11 NTFS/ReFS locking bottlenecks. Flushes batches either when 100 entries
/// accumulate or every 5 seconds.
#[derive(Debug, Clone)]
pub struct MpscBatchLogWriter {
    tx: mpsc::Sender<LogCommand>,
    dropped_count: Arc<AtomicUsize>,
}

impl MpscBatchLogWriter {
    /// Spawns the background worker processing log batches and flushing to disk.
    pub fn spawn(config: MpscBatchLogConfig) -> (Self, tokio::task::JoinHandle<()>) {
        let (tx, mut rx) = mpsc::channel::<LogCommand>(config.channel_capacity);
        let dropped_count = Arc::new(AtomicUsize::new(0));
        let dropped_clone = dropped_count.clone();

        let handle = tokio::spawn(async move {
            let mut buffer: Vec<RawLogEvent> = Vec::with_capacity(config.batch_size_threshold * 2);
            let mut interval = tokio::time::interval(Duration::from_secs(config.batch_flush_interval_secs.max(1)));
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            let log_path = config.log_file_path.clone();

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        if !buffer.is_empty() {
                            Self::flush_buffer_to_disk(&mut buffer, &log_path).await;
                        }
                    }
                    cmd = rx.recv() => {
                        match cmd {
                            Some(LogCommand::Append(event)) => {
                                buffer.push(event);
                                if buffer.len() >= config.batch_size_threshold {
                                    Self::flush_buffer_to_disk(&mut buffer, &log_path).await;
                                }
                            }
                            Some(LogCommand::Flush(ack)) => {
                                if !buffer.is_empty() {
                                    Self::flush_buffer_to_disk(&mut buffer, &log_path).await;
                                }
                                let _ = ack.send(());
                            }
                            Some(LogCommand::Shutdown(ack)) => {
                                if !buffer.is_empty() {
                                    Self::flush_buffer_to_disk(&mut buffer, &log_path).await;
                                }
                                let _ = ack.send(());
                                break;
                            }
                            None => {
                                // Channel disconnected, drain remaining
                                if !buffer.is_empty() {
                                    Self::flush_buffer_to_disk(&mut buffer, &log_path).await;
                                }
                                break;
                            }
                        }
                    }
                }
            }
        });

        (
            Self {
                tx,
                dropped_count: dropped_clone,
            },
            handle,
        )
    }

    /// Appends a raw log event non-blockingly.
    ///
    /// If the MPSC buffer is saturated, increments `dropped_count` and returns an error
    /// without blocking the caller Tokio worker thread.
    pub fn log(&self, event: RawLogEvent) -> Result<(), CoreError> {
        match self.tx.try_send(LogCommand::Append(event)) {
            Ok(_) => Ok(()),
            Err(mpsc::error::TrySendError::Full(_)) => {
                self.dropped_count.fetch_add(1, Ordering::Relaxed);
                Err(CoreError::BatchLogError("MPSC batch log queue full; event dropped".to_string()))
            }
            Err(mpsc::error::TrySendError::Closed(_)) => {
                Err(CoreError::BatchLogError("MPSC batch log queue closed".to_string()))
            }
        }
    }

    /// Asynchronously flushes all accumulated log events to disk and awaits confirmation.
    pub async fn flush(&self) -> Result<(), CoreError> {
        let (ack_tx, ack_rx) = oneshot::channel();
        self.tx
            .send(LogCommand::Flush(ack_tx))
            .await
            .map_err(|e| CoreError::BatchLogError(format!("Failed to send flush command: {e}")))?;
        ack_rx
            .await
            .map_err(|e| CoreError::BatchLogError(format!("Failed to receive flush ACK: {e}")))?;
        Ok(())
    }

    /// Gracefully flushes remaining logs and signals worker termination.
    pub async fn shutdown(&self) -> Result<(), CoreError> {
        let (ack_tx, ack_rx) = oneshot::channel();
        self.tx
            .send(LogCommand::Shutdown(ack_tx))
            .await
            .map_err(|e| CoreError::BatchLogError(format!("Failed to send shutdown command: {e}")))?;
        ack_rx
            .await
            .map_err(|e| CoreError::BatchLogError(format!("Failed to receive shutdown ACK: {e}")))?;
        Ok(())
    }

    /// Returns the number of events dropped due to buffer saturation.
    pub fn dropped_events_count(&self) -> usize {
        self.dropped_count.load(Ordering::Relaxed)
    }

    /// Writes accumulated log buffer to disk in a worker-isolated manner.
    async fn flush_buffer_to_disk(buffer: &mut Vec<RawLogEvent>, path: &Path) {
        if buffer.is_empty() {
            return;
        }

        let mut lines = String::with_capacity(buffer.len() * 128);
        for entry in buffer.drain(..) {
            lines.push_str(&entry.format_line());
        }

        let target_path = path.to_path_buf();
        // Offload physical I/O to spawn_blocking to eliminate ReFS/NTFS locks on Tokio reactor
        let _ = tokio::task::spawn_blocking(move || {
            if let Some(parent) = target_path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            if let Ok(mut file) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&target_path)
            {
                use std::io::Write;
                let _ = file.write_all(lines.as_bytes());
                let _ = file.flush();
            }
        })
        .await;
    }
}

/// Tracing layer capturing events and piping them non-blockingly to `MpscBatchLogWriter`.
pub struct MpscTracingLayer {
    writer: MpscBatchLogWriter,
}

impl MpscTracingLayer {
    pub fn new(writer: MpscBatchLogWriter) -> Self {
        Self { writer }
    }
}

impl<S> Layer<S> for MpscTracingLayer
where
    S: tracing::Subscriber,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let metadata = event.metadata();
        let mut visitor = StringVisitor::default();
        event.record(&mut visitor);

        let raw_event = RawLogEvent::new(
            metadata.level().as_str(),
            metadata.target(),
            visitor.0,
        );

        let _ = self.writer.log(raw_event);
    }
}

#[derive(Default)]
struct StringVisitor(String);

impl tracing::field::Visit for StringVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if !self.0.is_empty() {
            self.0.push(' ');
        }
        if field.name() == "message" {
            self.0.push_str(&format!("{value:?}").trim_matches('"'));
        } else {
            self.0.push_str(&format!("{}={:?}", field.name(), value));
        }
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if !self.0.is_empty() {
            self.0.push(' ');
        }
        if field.name() == "message" {
            self.0.push_str(value);
        } else {
            self.0.push_str(&format!("{}={}", field.name(), value));
        }
    }
}

/// Initializes structured logging using tracing-subscriber with standard terminal output.
pub fn init_tracing(log_level: &str) -> Result<(), CoreError> {
    let level = parse_level(log_level);

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

/// Initializes structured logging combining clean stdout formatting with the
/// non-blocking MPSC batch log pipeline.
pub fn init_tracing_with_mpsc(
    log_level: &str,
    writer: MpscBatchLogWriter,
) -> Result<(), CoreError> {
    let level = parse_level(log_level);

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(format!("{level},souls_core={level}")));

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_span_events(FmtSpan::CLOSE)
        .with_target(true);

    let mpsc_layer = MpscTracingLayer::new(writer);

    let subscriber = tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .with(mpsc_layer);

    subscriber
        .try_init()
        .map_err(|e| CoreError::TracingInitError(e.to_string()))?;

    Ok(())
}

fn parse_level(log_level: &str) -> Level {
    match log_level.to_ascii_lowercase().as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    }
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

    #[tokio::test]
    async fn test_mpsc_batch_log_flushes_on_size_threshold() {
        let temp_dir = tempfile::tempdir().expect("tempdir");
        let log_file = temp_dir.path().join("test_batch_100.log");

        let config = MpscBatchLogConfig {
            log_file_path: log_file.clone(),
            batch_flush_interval_secs: 60, // Long interval so flush only triggers by size
            batch_size_threshold: 100,
            channel_capacity: 512,
        };

        let (writer, _handle) = MpscBatchLogWriter::spawn(config);

        for i in 0..100 {
            writer
                .log(RawLogEvent::new("INFO", "test_worker", format!("Batch event #{i}")))
                .expect("log event");
        }

        // Allow spawn_blocking to write file
        tokio::time::sleep(Duration::from_millis(50)).await;

        let content = std::fs::read_to_string(&log_file).expect("read log file");
        let line_count = content.lines().count();
        assert_eq!(line_count, 100, "Expected exactly 100 lines flushed on batch size threshold");
        assert!(content.contains("Batch event #0"));
        assert!(content.contains("Batch event #99"));

        writer.shutdown().await.expect("shutdown");
    }

    #[tokio::test]
    async fn test_mpsc_batch_log_flushes_on_time_interval() {
        let temp_dir = tempfile::tempdir().expect("tempdir");
        let log_file = temp_dir.path().join("test_time_interval.log");

        let config = MpscBatchLogConfig {
            log_file_path: log_file.clone(),
            batch_flush_interval_secs: 1, // 1s interval for quick test
            batch_size_threshold: 100,
            channel_capacity: 512,
        };

        let (writer, _handle) = MpscBatchLogWriter::spawn(config);

        // Send only 5 events (well below the 100 threshold)
        for i in 0..5 {
            writer
                .log(RawLogEvent::new("DEBUG", "test_timer", format!("Time-based event #{i}")))
                .expect("log event");
        }

        // Wait 1.3s for timer tick
        tokio::time::sleep(Duration::from_millis(1300)).await;

        let content = std::fs::read_to_string(&log_file).expect("read log file");
        let line_count = content.lines().count();
        assert_eq!(line_count, 5, "Expected 5 lines flushed by periodic timer tick");

        writer.shutdown().await.expect("shutdown");
    }

    #[tokio::test]
    async fn test_mpsc_batch_log_explicit_flush() {
        let temp_dir = tempfile::tempdir().expect("tempdir");
        let log_file = temp_dir.path().join("test_explicit_flush.log");

        let config = MpscBatchLogConfig {
            log_file_path: log_file.clone(),
            batch_flush_interval_secs: 300,
            batch_size_threshold: 500,
            channel_capacity: 512,
        };

        let (writer, _handle) = MpscBatchLogWriter::spawn(config);

        writer
            .log(RawLogEvent::new("WARN", "kernel", "Thermal zone elevated"))
            .expect("log event");

        // Explicit flush
        writer.flush().await.expect("explicit flush");

        let content = std::fs::read_to_string(&log_file).expect("read log file");
        assert!(content.contains("Thermal zone elevated"));
        assert!(content.contains("[WARN]"));

        writer.shutdown().await.expect("shutdown");
    }

    #[test]
    fn test_raw_log_event_strips_ansi() {
        let event = RawLogEvent::new("ERROR", "test", "\x1b[31mCritical failure\x1b[0m");
        assert_eq!(event.message, "Critical failure");
        assert!(!event.format_line().contains("\x1b"));
    }
}
