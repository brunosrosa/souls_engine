//! Resilient Asynchronous Download Manager with HTTP Range and SSE Telemetry.
//!
//! Enforces a sequential FIFO queue (1 active download at a time) to prevent
//! network saturation, supports pausing, resuming interrupted downloads via HTTP Range,
//! and verifies SHA-256 integrity before promotion.

use crate::checksum::verify_file_sha256;
use crate::error::InventoryError;
use reqwest::header::{HeaderMap, HeaderValue, RANGE, USER_AGENT};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::{broadcast, watch, Mutex, RwLock};
use tracing::{error, info};

static DOWNLOAD_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Lifecycle state of a download item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DownloadState {
    Queued,
    Connecting,
    Downloading,
    Paused,
    VerifyingChecksum,
    Completed,
    Failed,
    Cancelled,
}

/// DTO representing a download item and its live telemetry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DownloadItem {
    pub id: String,
    pub repo_id: String,
    pub file_name: String,
    pub download_url: String,
    pub destination_path: PathBuf,
    pub temp_part_path: PathBuf,
    pub total_bytes: u64,
    pub downloaded_bytes: u64,
    pub progress_pct: f32,
    pub speed_mb_s: f32,
    pub eta_seconds: u32,
    pub state: DownloadState,
    pub expected_sha256: Option<String>,
    pub error_message: Option<String>,
}

/// Command signal for active download workers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WorkerSignal {
    Continue,
    Pause,
    Cancel,
}

/// Thread-safe Download Manager coordinator.
#[derive(Clone)]
pub struct DownloadManager {
    items: Arc<RwLock<HashMap<String, DownloadItem>>>,
    queue: Arc<Mutex<VecDeque<String>>>,
    active_download_id: Arc<Mutex<Option<String>>>,
    signals: Arc<Mutex<HashMap<String, watch::Sender<WorkerSignal>>>>,
    progress_tx: broadcast::Sender<DownloadItem>,
    client: reqwest::Client,
}

impl Default for DownloadManager {
    fn default() -> Self {
        Self::new()
    }
}

impl DownloadManager {
    /// Constructs a new DownloadManager instance with broadcast channel capacity 64.
    pub fn new() -> Self {
        let (progress_tx, _) = broadcast::channel(64);

        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static("SoulsEngine/7.0 (Bare-Metal Silicon OS)"),
        );

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .connect_timeout(Duration::from_secs(15))
            .build()
            .unwrap_or_default();

        Self {
            items: Arc::new(RwLock::new(HashMap::new())),
            queue: Arc::new(Mutex::new(VecDeque::new())),
            active_download_id: Arc::new(Mutex::new(None)),
            signals: Arc::new(Mutex::new(HashMap::new())),
            progress_tx,
            client,
        }
    }

    /// Generates a monotonic unique download ID.
    fn generate_download_id() -> String {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let count = DOWNLOAD_COUNTER.fetch_add(1, Ordering::Relaxed);
        format!("dl-{nanos:x}-{count:x}")
    }

    /// Subscribes to the live progress broadcast stream (Server-Sent Events).
    pub fn subscribe_progress(&self) -> broadcast::Receiver<DownloadItem> {
        self.progress_tx.subscribe()
    }

    /// Enqueues a new model file for download.
    pub async fn enqueue_download(
        &self,
        repo_id: &str,
        file_name: &str,
        download_url: &str,
        expected_sha256: Option<String>,
        dest_dir: &Path,
    ) -> Result<DownloadItem, InventoryError> {
        let id = Self::generate_download_id();
        let destination_path = dest_dir.join(file_name);
        let temp_part_path = dest_dir.join(format!("{file_name}.part"));

        // Pre-create destination directory if missing
        if let Err(e) = std::fs::create_dir_all(dest_dir) {
            return Err(InventoryError::CatalogIoError(
                dest_dir.display().to_string(),
                e.to_string(),
            ));
        }

        let existing_bytes = if temp_part_path.exists() {
            std::fs::metadata(&temp_part_path)
                .map(|m| m.len())
                .unwrap_or(0)
        } else {
            0
        };

        let item = DownloadItem {
            id: id.clone(),
            repo_id: repo_id.to_string(),
            file_name: file_name.to_string(),
            download_url: download_url.to_string(),
            destination_path,
            temp_part_path,
            total_bytes: 0,
            downloaded_bytes: existing_bytes,
            progress_pct: 0.0,
            speed_mb_s: 0.0,
            eta_seconds: 0,
            state: DownloadState::Queued,
            expected_sha256,
            error_message: None,
        };

        {
            let mut items = self.items.write().await;
            items.insert(id.clone(), item.clone());
        }

        {
            let mut queue = self.queue.lock().await;
            queue.push_back(id.clone());
        }

        info!(id = %id, file = %file_name, "Enqueued download item");
        let _ = self.progress_tx.send(item.clone());

        // Spawn or kick the background queue processor
        self.trigger_worker();

        Ok(item)
    }

    /// Triggers the background worker loop if no download is actively running.
    fn trigger_worker(&self) {
        let manager = self.clone();
        tokio::spawn(async move {
            manager.process_queue().await;
        });
    }

    /// Internal queue processing loop.
    async fn process_queue(&self) {
        let mut active_lock = self.active_download_id.lock().await;
        if active_lock.is_some() {
            // A download is already running in the background
            return;
        }

        let next_id = {
            let mut queue = self.queue.lock().await;
            queue.pop_front()
        };

        let download_id = match next_id {
            Some(id) => id,
            None => return,
        };

        *active_lock = Some(download_id.clone());
        drop(active_lock);

        let manager = self.clone();
        let target_id = download_id.clone();

        // Run the download task
        manager.execute_download(target_id).await;

        // Release active lock and process next queued item
        let mut active_lock = self.active_download_id.lock().await;
        *active_lock = None;
        drop(active_lock);

        // Chain execution
        self.trigger_worker();
    }

    /// Executes the active download task with HTTP Range and cancellation/pause monitoring.
    async fn execute_download(&self, download_id: String) {
        let (mut item, mut rx) = {
            let mut items = self.items.write().await;
            let item = match items.get_mut(&download_id) {
                Some(it) => it,
                None => return,
            };

            if item.state == DownloadState::Paused || item.state == DownloadState::Cancelled {
                return;
            }

            item.state = DownloadState::Connecting;
            let (tx, rx) = watch::channel(WorkerSignal::Continue);
            let mut signals = self.signals.lock().await;
            signals.insert(download_id.clone(), tx);

            (item.clone(), rx)
        };

        let _ = self.progress_tx.send(item.clone());

        let existing_bytes = if item.temp_part_path.exists() {
            std::fs::metadata(&item.temp_part_path)
                .map(|m| m.len())
                .unwrap_or(0)
        } else {
            0
        };

        let mut req = self.client.get(&item.download_url);
        if existing_bytes > 0 {
            info!(id = %download_id, offset = existing_bytes, "Resuming download via HTTP Range");
            req = req.header(RANGE, format!("bytes={existing_bytes}-"));
        }

        let response = match req.send().await {
            Ok(resp) => resp,
            Err(e) => {
                self.mark_failed(&download_id, format!("HTTP request failed: {e}")).await;
                return;
            }
        };

        let status = response.status();
        let is_range_ok = status == reqwest::StatusCode::PARTIAL_CONTENT;
        let is_full_ok = status == reqwest::StatusCode::OK;

        if !is_range_ok && !is_full_ok {
            self.mark_failed(&download_id, format!("Server returned error HTTP {status}")).await;
            return;
        }

        // Determine total file length
        let content_len = response.content_length().unwrap_or(0);
        let total_bytes = if is_range_ok {
            existing_bytes + content_len
        } else {
            content_len
        };

        let file = match OpenOptions::new()
            .create(true)
            .append(is_range_ok)
            .write(true)
            .open(&item.temp_part_path)
        {
            Ok(f) => f,
            Err(e) => {
                self.mark_failed(&download_id, format!("Failed to open .part file: {e}")).await;
                return;
            }
        };

        {
            let mut items = self.items.write().await;
            if let Some(it) = items.get_mut(&download_id) {
                it.total_bytes = total_bytes;
                it.downloaded_bytes = if is_range_ok { existing_bytes } else { 0 };
                it.state = DownloadState::Downloading;
                item = it.clone();
            }
        }
        let _ = self.progress_tx.send(item.clone());

        let mut stream = response;
        let mut writer = std::io::BufWriter::with_capacity(256 * 1024, file);

        let mut downloaded_now = item.downloaded_bytes;
        let start_time = Instant::now();
        let mut last_broadcast = Instant::now();
        let mut last_bytes_count = downloaded_now;

        loop {
            // Check for control signals (Pause / Cancel)
            let signal = if rx.has_changed().unwrap_or(false) {
                Some(*rx.borrow_and_update())
            } else {
                None
            };

            if let Some(sig) = signal {
                match sig {
                    WorkerSignal::Pause => {
                        info!(id = %download_id, "Download paused by user");
                        let _ = writer.flush();
                        self.set_state(&download_id, DownloadState::Paused).await;
                        return;
                    }
                    WorkerSignal::Cancel => {
                        info!(id = %download_id, "Download cancelled by user");
                        let _ = writer.flush();
                        let _ = std::fs::remove_file(&item.temp_part_path);
                        self.set_state(&download_id, DownloadState::Cancelled).await;
                        return;
                    }
                    WorkerSignal::Continue => {}
                }
            }

            match stream.chunk().await {
                Ok(Some(chunk)) => {
                    if let Err(e) = writer.write_all(&chunk) {
                        self.mark_failed(&download_id, format!("Write error: {e}")).await;
                        return;
                    }

                    downloaded_now += chunk.len() as u64;

                    // Periodic throttle broadcast (every 250ms)
                    if last_broadcast.elapsed() >= Duration::from_millis(250) {
                        let elapsed_sec = last_broadcast.elapsed().as_secs_f32().max(0.001);
                        let bytes_delta = downloaded_now.saturating_sub(last_bytes_count);
                        let speed_mb_s = (bytes_delta as f32 / (1024.0 * 1024.0)) / elapsed_sec;

                        let progress_pct = if total_bytes > 0 {
                            (downloaded_now as f32 / total_bytes as f32) * 100.0
                        } else {
                            0.0
                        };

                        let remaining_bytes = total_bytes.saturating_sub(downloaded_now);
                        let eta_seconds = if speed_mb_s > 0.05 {
                            (remaining_bytes as f32 / (speed_mb_s * 1024.0 * 1024.0)).round() as u32
                        } else {
                            0
                        };

                        {
                            let mut items = self.items.write().await;
                            if let Some(it) = items.get_mut(&download_id) {
                                it.downloaded_bytes = downloaded_now;
                                it.progress_pct = progress_pct;
                                it.speed_mb_s = speed_mb_s;
                                it.eta_seconds = eta_seconds;
                                item = it.clone();
                            }
                        }
                        let _ = self.progress_tx.send(item.clone());

                        last_broadcast = Instant::now();
                        last_bytes_count = downloaded_now;
                    }
                }
                Ok(None) => {
                    // Stream exhausted: download complete!
                    break;
                }
                Err(e) => {
                    let _ = writer.flush();
                    self.mark_failed(&download_id, format!("Network stream error: {e}")).await;
                    return;
                }
            }
        }

        let _ = writer.flush();

        // Optional SHA-256 verification
        if let Some(ref expected_hash) = item.expected_sha256 {
            self.set_state(&download_id, DownloadState::VerifyingChecksum).await;
            info!(id = %download_id, "Verifying downloaded SHA-256 integrity");

            match verify_file_sha256(&item.temp_part_path, expected_hash) {
                Ok(true) => {
                    info!(id = %download_id, "SHA-256 verified successfully");
                }
                Ok(false) | Err(_) => {
                    error!(id = %download_id, "Checksum verification failed");
                    self.mark_failed(&download_id, "SHA-256 verification failed".to_string()).await;
                    return;
                }
            }
        }

        // Atomic promotion from .part to final destination .gguf
        if let Err(e) = std::fs::rename(&item.temp_part_path, &item.destination_path) {
            self.mark_failed(&download_id, format!("Failed to promote .part to .gguf: {e}")).await;
            return;
        }

        info!(
            id = %download_id,
            path = %item.destination_path.display(),
            duration_s = start_time.elapsed().as_secs(),
            "Download completed and promoted to catalog"
        );

        {
            let mut items = self.items.write().await;
            if let Some(it) = items.get_mut(&download_id) {
                it.state = DownloadState::Completed;
                it.progress_pct = 100.0;
                it.speed_mb_s = 0.0;
                it.eta_seconds = 0;
                item = it.clone();
            }
        }
        let _ = self.progress_tx.send(item);
    }

    /// Pauses an active or queued download.
    pub async fn pause_download(&self, download_id: &str) -> Result<(), InventoryError> {
        let signals = self.signals.lock().await;
        if let Some(tx) = signals.get(download_id) {
            let _ = tx.send(WorkerSignal::Pause);
            return Ok(());
        }

        // If it was still in queue, remove it from queue and set Paused
        let mut queue = self.queue.lock().await;
        queue.retain(|id| id != download_id);

        self.set_state(download_id, DownloadState::Paused).await;
        Ok(())
    }

    /// Resumes a paused download, re-enqueueing it.
    pub async fn resume_download(&self, download_id: &str) -> Result<(), InventoryError> {
        {
            let mut items = self.items.write().await;
            let item = items
                .get_mut(download_id)
                .ok_or_else(|| InventoryError::DownloadNotFound(download_id.to_string()))?;

            if item.state != DownloadState::Paused && item.state != DownloadState::Failed {
                return Ok(());
            }

            item.state = DownloadState::Queued;
            item.error_message = None;
        }

        {
            let mut queue = self.queue.lock().await;
            queue.push_back(download_id.to_string());
        }

        info!(id = %download_id, "Resumed download item");
        self.trigger_worker();
        Ok(())
    }

    /// Cancels a download, stopping active streaming and deleting temporary files.
    pub async fn cancel_download(&self, download_id: &str) -> Result<(), InventoryError> {
        let signals = self.signals.lock().await;
        if let Some(tx) = signals.get(download_id) {
            let _ = tx.send(WorkerSignal::Cancel);
            return Ok(());
        }

        // If queued, remove from queue
        let mut queue = self.queue.lock().await;
        queue.retain(|id| id != download_id);

        let mut items = self.items.write().await;
        if let Some(item) = items.get_mut(download_id) {
            item.state = DownloadState::Cancelled;
            if item.temp_part_path.exists() {
                let _ = std::fs::remove_file(&item.temp_part_path);
            }
            let _ = self.progress_tx.send(item.clone());
        }

        Ok(())
    }

    /// Lists all known download items.
    pub async fn list_downloads(&self) -> Vec<DownloadItem> {
        let items = self.items.read().await;
        items.values().cloned().collect()
    }

    /// Retrieves an individual download item by ID.
    pub async fn get_download(&self, download_id: &str) -> Option<DownloadItem> {
        let items = self.items.read().await;
        items.get(download_id).cloned()
    }

    async fn set_state(&self, download_id: &str, state: DownloadState) {
        let mut items = self.items.write().await;
        if let Some(item) = items.get_mut(download_id) {
            item.state = state;
            let _ = self.progress_tx.send(item.clone());
        }
    }

    async fn mark_failed(&self, download_id: &str, error_message: String) {
        error!(id = %download_id, err = %error_message, "Download failed");
        let mut items = self.items.write().await;
        if let Some(item) = items.get_mut(download_id) {
            item.state = DownloadState::Failed;
            item.error_message = Some(error_message);
            let _ = self.progress_tx.send(item.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_enqueue_and_queue_lifecycle() {
        let manager = DownloadManager::new();
        let dir = tempdir().unwrap();

        let item = manager
            .enqueue_download(
                "test/repo",
                "test-model.gguf",
                "http://127.0.0.1:9/invalid-url-for-test",
                None,
                dir.path(),
            )
            .await
            .unwrap();

        assert_eq!(item.file_name, "test-model.gguf");
        assert_eq!(item.state, DownloadState::Queued);

        let list = manager.list_downloads().await;
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, item.id);

        // Cancel test
        manager.cancel_download(&item.id).await.unwrap();
        let updated = manager.get_download(&item.id).await.unwrap();
        assert_eq!(updated.state, DownloadState::Cancelled);
    }
}
