//! Multi-cadence Proactive Pulse store, isolated monitor backoff tracking,
//! Quiet Hours evaluation, Drive <-> NotebookLM sync links, and optional external alert dispatch.

use super::security::{redact_google_secrets, sanitize_for_external_llm};
use super::session::{write_bytes_atomic, write_json_atomic_lf};
use crate::error::CoreError;
use serde::{Deserialize, Serialize};
use serde_json::json;
use souls_protocol::google::{
    DriveNotebookSyncLinkDto, PulseConfigDto, PulseEventDto, PulseMonitorHealthDto, PulseUrgency,
};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, RwLock};
use tracing::warn;

const MAX_PERSISTED_PULSE_EVENTS: usize = 500;
const MAX_FORM_SEEN_IDS: usize = 500;
const DEDUP_WINDOW_SECS: i64 = 3_600;

/// Sensitive external notification credentials stored separately from `PulseConfigDto`
/// in `<data_dir>/pulse_secrets.json`.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct PulseSecrets {
    pub webhook_url: Option<String>,
    pub telegram_bot_token: Option<String>,
    pub telegram_chat_id: Option<String>,
}

/// Persistent cursor, cadence tick counter, cooldowns, and isolated monitor health.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct PulseRuntimeState {
    pub tick_count: u64,
    pub last_run_at: Option<i64>,
    pub gmail_last_seen_ts: Option<i64>,
    pub form_last_seen_ids: HashMap<String, Vec<String>>,
    pub external_cooldowns: HashMap<String, i64>,
    pub monitor_health: HashMap<String, PulseMonitorHealthDto>,
}

/// Thread-safe, atomic LF persistent store for the Google Proactive Pulse engine.
pub struct ProactivePulseStore {
    config_path: PathBuf,
    secrets_path: PathBuf,
    state_path: PathBuf,
    events_path: PathBuf,
    sync_links_path: PathBuf,
    config: RwLock<PulseConfigDto>,
    secrets: RwLock<PulseSecrets>,
    state: RwLock<PulseRuntimeState>,
    events: RwLock<Vec<PulseEventDto>>,
    sync_links: RwLock<Vec<DriveNotebookSyncLinkDto>>,
    events_io_lock: Mutex<()>,
}

impl ProactivePulseStore {
    /// Opens or initializes `ProactivePulseStore` under `data_dir` (`<souls_data>/google/`).
    pub fn open(data_dir: &Path) -> Result<Self, CoreError> {
        fs::create_dir_all(data_dir).map_err(|e| {
            CoreError::SubprocessError(format!(
                "Failed to create pulse data dir '{}': {e}",
                data_dir.display()
            ))
        })?;

        let config_path = data_dir.join("pulse_config.json");
        let secrets_path = data_dir.join("pulse_secrets.json");
        let state_path = data_dir.join("pulse_state.json");
        let events_path = data_dir.join("pulse_events.jsonl");
        let sync_links_path = data_dir.join("sync_links.json");

        let secrets: PulseSecrets = read_json_or_default(&secrets_path)?;
        let mut config: PulseConfigDto = read_json_or_default(&config_path)?;
        normalize_config(&mut config, &secrets);

        let state: PulseRuntimeState = read_json_or_default(&state_path)?;
        let sync_links: Vec<DriveNotebookSyncLinkDto> = read_json_or_default(&sync_links_path)?;
        let events = read_jsonl_events(&events_path);

        Ok(Self {
            config_path,
            secrets_path,
            state_path,
            events_path,
            sync_links_path,
            config: RwLock::new(config),
            secrets: RwLock::new(secrets),
            state: RwLock::new(state),
            events: RwLock::new(events),
            sync_links: RwLock::new(sync_links),
            events_io_lock: Mutex::new(()),
        })
    }

    /// Returns a snapshot of the current `PulseConfigDto` (never contains secrets).
    pub fn config(&self) -> PulseConfigDto {
        self.config
            .read()
            .map(|c| c.clone())
            .unwrap_or_default()
    }

    /// Clamps/normalizes and persists updated `PulseConfigDto`.
    pub fn update_config(&self, mut cfg: PulseConfigDto) -> Result<PulseConfigDto, CoreError> {
        let secrets = self.secrets();
        normalize_config(&mut cfg, &secrets);

        write_json_atomic_lf(&self.config_path, &cfg)?;
        if let Ok(mut guard) = self.config.write() {
            *guard = cfg.clone();
        }
        Ok(cfg)
    }

    /// Updates optional external alert secrets (`Some("")` clears an existing secret; `None` keeps it).
    pub fn update_secrets(
        &self,
        webhook_url: Option<String>,
        telegram_bot_token: Option<String>,
        telegram_chat_id: Option<String>,
    ) -> Result<PulseConfigDto, CoreError> {
        let mut updated_secrets = self.secrets();
        apply_secret_patch(&mut updated_secrets.webhook_url, webhook_url);
        apply_secret_patch(&mut updated_secrets.telegram_bot_token, telegram_bot_token);
        apply_secret_patch(&mut updated_secrets.telegram_chat_id, telegram_chat_id);

        write_json_atomic_lf(&self.secrets_path, &updated_secrets)?;
        if let Ok(mut guard) = self.secrets.write() {
            *guard = updated_secrets.clone();
        }

        let mut cfg = self.config();
        normalize_config(&mut cfg, &updated_secrets);
        write_json_atomic_lf(&self.config_path, &cfg)?;
        if let Ok(mut guard) = self.config.write() {
            *guard = cfg.clone();
        }
        Ok(cfg)
    }

    /// Returns stored `PulseSecrets`.
    pub fn secrets(&self) -> PulseSecrets {
        self.secrets
            .read()
            .map(|s| s.clone())
            .unwrap_or_default()
    }

    /// Returns a snapshot of `PulseRuntimeState`.
    pub fn runtime_state(&self) -> PulseRuntimeState {
        self.state
            .read()
            .map(|s| s.clone())
            .unwrap_or_default()
    }

    /// Persists the Gmail VIP watermark timestamp cursor.
    pub fn set_gmail_cursor(&self, ts: i64) {
        if let Ok(mut guard) = self.state.write() {
            guard.gmail_last_seen_ts = Some(ts);
            if let Err(err) = write_json_atomic_lf(&self.state_path, &*guard) {
                warn!(target: "souls_core::google_bridge", "Failed persisting Gmail cursor: {err}");
            }
        }
    }

    /// Computes unseen response IDs for `form_id`, updates the stored cursor list, and returns new IDs.
    pub fn seen_form_responses_diff(&self, form_id: &str, response_ids: &[String]) -> Vec<String> {
        let Ok(mut guard) = self.state.write() else {
            return Vec::new();
        };

        let seen = guard
            .form_last_seen_ids
            .entry(form_id.to_string())
            .or_default();

        let mut newly_seen = Vec::new();
        for id in response_ids {
            if !id.is_empty() && !seen.contains(id) {
                newly_seen.push(id.clone());
                seen.push(id.clone());
            }
        }

        if seen.len() > MAX_FORM_SEEN_IDS {
            let overflow = seen.len() - MAX_FORM_SEEN_IDS;
            seen.drain(0..overflow);
        }

        if !newly_seen.is_empty() {
            if let Err(err) = write_json_atomic_lf(&self.state_path, &*guard) {
                warn!(target: "souls_core::google_bridge", "Failed persisting form cursor: {err}");
            }
        }

        newly_seen
    }

    /// Returns `true` when the pulse heartbeat is enabled and its configured interval has elapsed.
    pub fn is_due(&self, now_ts: i64) -> bool {
        let cfg = self.config();
        if !cfg.enabled {
            return false;
        }
        let state = self.runtime_state();
        match state.last_run_at {
            None => true,
            Some(prev) => now_ts.saturating_sub(prev) >= (cfg.interval_mins as i64) * 60,
        }
    }

    /// Advances `tick_count`, records `last_run_at = Some(now_ts)`, persists state, and returns the new tick count.
    pub fn begin_tick(&self, now_ts: i64) -> u64 {
        let Ok(mut guard) = self.state.write() else {
            return 1;
        };
        guard.tick_count = guard.tick_count.saturating_add(1);
        guard.last_run_at = Some(now_ts);
        if let Err(err) = write_json_atomic_lf(&self.state_path, &*guard) {
            warn!(target: "souls_core::google_bridge", "Failed persisting pulse tick state: {err}");
        }
        guard.tick_count
    }

    /// Determines whether `monitor` should execute on `tick_count` respecting isolated backoff
    /// and multi-cadence multipliers (`(tick_count - 1) % multiplier == 0` unless `force == true`).
    pub fn should_run_monitor(
        &self,
        monitor: &str,
        tick_count: u64,
        multiplier: u32,
        force: bool,
        now_ts: i64,
    ) -> bool {
        if let Ok(guard) = self.state.read() {
            if let Some(health) = guard.monitor_health.get(monitor) {
                if let Some(until) = health.backoff_until {
                    if now_ts < until {
                        return false;
                    }
                }
            }
        }

        if force {
            return true;
        }

        let mult = multiplier.max(1) as u64;
        tick_count.saturating_sub(1).is_multiple_of(mult)
    }

    /// Clears isolated backoff for `monitor` and updates `last_ok_at`.
    pub fn record_monitor_ok(&self, monitor: &str, cadence_mins: u32, now_ts: i64) {
        if let Ok(mut guard) = self.state.write() {
            let entry = guard
                .monitor_health
                .entry(monitor.to_string())
                .or_insert_with(|| PulseMonitorHealthDto {
                    monitor: monitor.to_string(),
                    cadence_mins,
                    consecutive_errors: 0,
                    backoff_until: None,
                    last_ok_at: None,
                    last_error: None,
                });
            entry.cadence_mins = cadence_mins;
            entry.consecutive_errors = 0;
            entry.backoff_until = None;
            entry.last_ok_at = Some(now_ts);
            entry.last_error = None;

            if let Err(err) = write_json_atomic_lf(&self.state_path, &*guard) {
                warn!(target: "souls_core::google_bridge", "Failed persisting monitor ok state: {err}");
            }
        }
    }

    /// Records a monitor failure, redacting secrets and putting ONLY that monitor into a 2-cycle backoff.
    pub fn record_monitor_error(
        &self,
        monitor: &str,
        cadence_mins: u32,
        err_msg: &str,
        now_ts: i64,
    ) {
        let safe_err = redact_google_secrets(err_msg);
        let backoff_secs = (cadence_mins.max(1) as i64) * 60 * 2;
        if let Ok(mut guard) = self.state.write() {
            let entry = guard
                .monitor_health
                .entry(monitor.to_string())
                .or_insert_with(|| PulseMonitorHealthDto {
                    monitor: monitor.to_string(),
                    cadence_mins,
                    consecutive_errors: 0,
                    backoff_until: None,
                    last_ok_at: None,
                    last_error: None,
                });
            entry.cadence_mins = cadence_mins;
            entry.consecutive_errors = entry.consecutive_errors.saturating_add(1);
            entry.backoff_until = Some(now_ts.saturating_add(backoff_secs));
            entry.last_error = Some(safe_err);

            if let Err(err) = write_json_atomic_lf(&self.state_path, &*guard) {
                warn!(target: "souls_core::google_bridge", "Failed persisting monitor error state: {err}");
            }
        }
    }

    /// Returns sorted per-monitor health and backoff statuses.
    pub fn monitor_health_list(&self) -> Vec<PulseMonitorHealthDto> {
        let mut list: Vec<PulseMonitorHealthDto> = self
            .state
            .read()
            .map(|s| s.monitor_health.values().cloned().collect())
            .unwrap_or_default();
        list.sort_by(|a, b| a.monitor.cmp(&b.monitor));
        list
    }

    /// Appends a pulse event after redacting secrets and deduplicating active `dedup_key` entries.
    /// Returns `Ok(true)` if recorded, `Ok(false)` if suppressed as duplicate.
    pub fn push_event(&self, mut event: PulseEventDto) -> Result<bool, CoreError> {
        event.title = redact_google_secrets(&event.title);
        event.summary = redact_google_secrets(&event.summary);

        let mut guard = self
            .events
            .write()
            .map_err(|_| CoreError::SubprocessError("Pulse events lock poisoned".to_string()))?;

        if !event.dedup_key.trim().is_empty() {
            let is_dup = guard.iter().any(|existing| {
                existing.dedup_key == event.dedup_key
                    && (!existing.acknowledged
                        || event.created_at.saturating_sub(existing.created_at).abs()
                            < DEDUP_WINDOW_SECS)
            });
            if is_dup {
                return Ok(false);
            }
        }

        guard.push(event);
        if guard.len() > MAX_PERSISTED_PULSE_EVENTS {
            let overflow = guard.len() - MAX_PERSISTED_PULSE_EVENTS;
            guard.drain(0..overflow);
        }

        self.write_events_jsonl(&guard)?;
        Ok(true)
    }

    /// Lists up to `limit` most recent pulse events (newest first).
    pub fn list_events(&self, limit: usize) -> Vec<PulseEventDto> {
        if limit == 0 {
            return Vec::new();
        }
        let Ok(guard) = self.events.read() else {
            return Vec::new();
        };
        guard.iter().rev().take(limit).cloned().collect()
    }

    /// Counts unacknowledged pulse events.
    pub fn unacked_count(&self) -> usize {
        self.events
            .read()
            .map(|g| g.iter().filter(|e| !e.acknowledged).count())
            .unwrap_or(0)
    }

    /// Retrieves pending alerts not yet delivered to Hermes, optionally draining (marking `delivered_to_hermes = true`).
    pub fn pending_for_hermes(
        &self,
        limit: usize,
        drain: bool,
    ) -> Result<Vec<PulseEventDto>, CoreError> {
        if limit == 0 {
            return Ok(Vec::new());
        }

        let mut guard = self
            .events
            .write()
            .map_err(|_| CoreError::SubprocessError("Pulse events lock poisoned".to_string()))?;

        let mut candidate_indices: Vec<usize> = guard
            .iter()
            .enumerate()
            .filter(|(_, e)| !e.acknowledged && !e.delivered_to_hermes)
            .map(|(idx, _)| idx)
            .collect();

        candidate_indices.sort_by(|&ia, &ib| {
            let a = &guard[ia];
            let b = &guard[ib];
            urgency_rank(b.urgency)
                .cmp(&urgency_rank(a.urgency))
                .then_with(|| b.created_at.cmp(&a.created_at))
        });

        let selected_indices: Vec<usize> = candidate_indices.into_iter().take(limit).collect();
        let result: Vec<PulseEventDto> = selected_indices
            .iter()
            .map(|&idx| guard[idx].clone())
            .collect();

        if drain && !selected_indices.is_empty() {
            for idx in selected_indices {
                guard[idx].delivered_to_hermes = true;
            }
            self.write_events_jsonl(&guard)?;
        }

        Ok(result)
    }

    /// Acknowledges either specific `event_ids` or all unacknowledged events (`None`).
    pub fn ack_events(&self, event_ids: Option<&[String]>) -> Result<usize, CoreError> {
        let mut guard = self
            .events
            .write()
            .map_err(|_| CoreError::SubprocessError("Pulse events lock poisoned".to_string()))?;

        let mut acked = 0usize;
        for ev in guard.iter_mut() {
            if ev.acknowledged {
                continue;
            }
            let matches = match event_ids {
                None => true,
                Some(ids) => ids.iter().any(|id| id == &ev.id),
            };
            if matches {
                ev.acknowledged = true;
                acked += 1;
            }
        }

        if acked > 0 {
            self.write_events_jsonl(&guard)?;
        }
        Ok(acked)
    }

    /// Checks whether external alert cooldown has elapsed for `category_key`.
    pub fn can_dispatch_external(&self, category_key: &str, now_ts: i64) -> bool {
        let cfg = self.config();
        let cooldown_secs = (cfg.external_cooldown_mins as i64) * 60;
        let last_sent = self
            .state
            .read()
            .ok()
            .and_then(|s| s.external_cooldowns.get(category_key).copied());
        last_sent.is_none_or(|prev| now_ts.saturating_sub(prev) >= cooldown_secs)
    }

    /// Records external delivery cooldown timestamp and marks `delivered_externally = true` on `event_id`.
    pub fn mark_external_dispatched(&self, event_id: &str, category_key: &str, now_ts: i64) {
        if let Ok(mut state_guard) = self.state.write() {
            state_guard
                .external_cooldowns
                .insert(category_key.to_string(), now_ts);
            if let Err(err) = write_json_atomic_lf(&self.state_path, &*state_guard) {
                warn!(target: "souls_core::google_bridge", "Failed persisting external cooldown: {err}");
            }
        }

        if let Ok(mut ev_guard) = self.events.write() {
            let mut changed = false;
            for ev in ev_guard.iter_mut() {
                if ev.id == event_id && !ev.delivered_externally {
                    ev.delivered_externally = true;
                    changed = true;
                }
            }
            if changed {
                if let Err(err) = self.write_events_jsonl(&ev_guard) {
                    warn!(target: "souls_core::google_bridge", "Failed persisting delivered_externally: {err}");
                }
            }
        }
    }

    // ------------------------------------------------------------------------
    // Drive <-> NotebookLM Sync Links
    // ------------------------------------------------------------------------

    /// Lists all configured Drive <-> NotebookLM auto-sync links.
    pub fn list_sync_links(&self) -> Vec<DriveNotebookSyncLinkDto> {
        self.sync_links
            .read()
            .map(|l| l.clone())
            .unwrap_or_default()
    }

    /// Inserts or updates a Drive <-> NotebookLM sync link.
    pub fn upsert_sync_link(
        &self,
        link: DriveNotebookSyncLinkDto,
    ) -> Result<DriveNotebookSyncLinkDto, CoreError> {
        if link.id.trim().is_empty() {
            return Err(CoreError::SubprocessError(
                "Sync link id cannot be empty".to_string(),
            ));
        }

        let mut guard = self.sync_links.write().map_err(|_| {
            CoreError::SubprocessError("Sync links lock poisoned".to_string())
        })?;

        if let Some(existing) = guard.iter_mut().find(|l| l.id == link.id) {
            *existing = link.clone();
        } else {
            guard.push(link.clone());
        }

        write_json_atomic_lf(&self.sync_links_path, &*guard)?;
        Ok(link)
    }

    /// Toggles `auto_sync_enabled` for an existing Drive <-> NotebookLM sync link.
    pub fn toggle_sync_link(
        &self,
        id: &str,
        enabled: bool,
    ) -> Result<DriveNotebookSyncLinkDto, CoreError> {
        let mut guard = self.sync_links.write().map_err(|_| {
            CoreError::SubprocessError("Sync links lock poisoned".to_string())
        })?;

        let target = guard
            .iter_mut()
            .find(|l| l.id == id)
            .ok_or_else(|| CoreError::SubprocessError(format!("Sync link '{id}' not found")))?;
        target.auto_sync_enabled = enabled;
        let updated = target.clone();
        write_json_atomic_lf(&self.sync_links_path, &*guard)?;
        Ok(updated)
    }

    /// Deletes a Drive <-> NotebookLM sync link by `id`.
    pub fn delete_sync_link(&self, id: &str) -> Result<bool, CoreError> {
        let mut guard = self.sync_links.write().map_err(|_| {
            CoreError::SubprocessError("Sync links lock poisoned".to_string())
        })?;

        let prev = guard.len();
        guard.retain(|l| l.id != id);
        let removed = guard.len() < prev;
        if removed {
            write_json_atomic_lf(&self.sync_links_path, &*guard)?;
        }
        Ok(removed)
    }

    /// Lists sync links associated with a specific `drive_file_id`.
    pub fn sync_links_for_drive_file(&self, drive_file_id: &str) -> Vec<DriveNotebookSyncLinkDto> {
        self.sync_links
            .read()
            .map(|l| {
                l.iter()
                    .filter(|item| item.drive_file_id == drive_file_id)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    fn write_events_jsonl(&self, events: &[PulseEventDto]) -> Result<(), CoreError> {
        let _lock = self
            .events_io_lock
            .lock()
            .map_err(|_| CoreError::SubprocessError("Events IO mutex poisoned".to_string()))?;

        let mut buf = String::new();
        for ev in events {
            let mut line = serde_json::to_string(ev).map_err(|e| {
                CoreError::SubprocessError(format!("Failed serializing PulseEventDto: {e}"))
            })?;
            if line.contains('\r') {
                line = line.replace(['\r', '\n'], " ");
            }
            buf.push_str(&line);
            buf.push('\n');
        }
        write_bytes_atomic(&self.events_path, buf.as_bytes())
    }
}

/// Evaluates whether `local_hour` (`0..=23`) falls inside Quiet Hours `[start, end)`.
///
/// Always returns `false` when `enabled == false` or `start == end`. Handles overnight
/// wraparound (`start > end`, e.g. `22..8`) and same-day ranges (`start < end`).
pub fn is_within_quiet_hours(enabled: bool, local_hour: u8, start: u8, end: u8) -> bool {
    if !enabled {
        return false;
    }
    let h = local_hour % 24;
    let s = start.min(23);
    let e = end.min(23);
    if s == e {
        false
    } else if s < e {
        h >= s && h < e
    } else {
        h >= s || h < e
    }
}

/// Pure gate deciding whether an external Webhook/Telegram alert should fire.
/// Only `High` and `Critical` urgencies outside Quiet Hours and cooldown are dispatched.
pub fn should_dispatch_external(
    urgency: PulseUrgency,
    quiet_hours_active: bool,
    cooldown_secs: i64,
    last_sent_ts: Option<i64>,
    now_ts: i64,
) -> bool {
    if !matches!(urgency, PulseUrgency::High | PulseUrgency::Critical) {
        return false;
    }
    if quiet_hours_active {
        return false;
    }
    last_sent_ts.is_none_or(|prev| now_ts.saturating_sub(prev) >= cooldown_secs.max(0))
}

/// Sends a sanitized external alert payload to configured Webhook and/or Telegram Bot channels.
/// Returns the names of channels that succeeded (`"webhook"`, `"telegram"`).
pub async fn dispatch_external_pulse_alert(
    http: &reqwest::Client,
    secrets: &PulseSecrets,
    event: &PulseEventDto,
    telegram_base_override: Option<&str>,
) -> Result<Vec<String>, CoreError> {
    let safe_title = sanitize_for_external_llm(&event.title);
    let safe_summary = sanitize_for_external_llm(&event.summary);
    let mut sent_channels = Vec::new();
    let mut errors = Vec::new();

    if let Some(ref webhook) = secrets.webhook_url {
        let trimmed = webhook.trim();
        if !trimmed.is_empty() {
            let payload = json!({
                "text": format!("[Souls Google Pulse] {}: {}", safe_title, safe_summary),
                "title": safe_title,
                "summary": safe_summary,
                "category": event.category,
                "urgency": event.urgency,
                "source_ref": event.source_ref,
            });
            match http.post(trimmed).json(&payload).send().await {
                Ok(resp) if resp.status().is_success() => {
                    sent_channels.push("webhook".to_string());
                }
                Ok(resp) => {
                    errors.push(format!("webhook HTTP {}", resp.status()));
                }
                Err(err) => {
                    errors.push(format!("webhook transport error: {err}"));
                }
            }
        }
    }

    if let (Some(ref bot_token), Some(ref chat_id)) =
        (&secrets.telegram_bot_token, &secrets.telegram_chat_id)
    {
        let token_trim = bot_token.trim();
        let chat_trim = chat_id.trim();
        if !token_trim.is_empty() && !chat_trim.is_empty() {
            let tg_base = telegram_base_override
                .unwrap_or("https://api.telegram.org")
                .trim_end_matches('/');
            let url = format!("{tg_base}/bot{token_trim}/sendMessage");
            let msg_text = format!(
                "[Souls Google Pulse]\n{}\n\n{}",
                safe_title, safe_summary
            );
            let payload = json!({
                "chat_id": chat_trim,
                "text": msg_text,
            });
            match http.post(&url).json(&payload).send().await {
                Ok(resp) if resp.status().is_success() => {
                    sent_channels.push("telegram".to_string());
                }
                Ok(resp) => {
                    errors.push(format!("telegram HTTP {}", resp.status()));
                }
                Err(err) => {
                    errors.push(format!("telegram transport error: {err}"));
                }
            }
        }
    }

    if sent_channels.is_empty() && !errors.is_empty() {
        return Err(CoreError::SubprocessError(format!(
            "External pulse dispatch failed: {}",
            errors.join("; ")
        )));
    }

    Ok(sent_channels)
}

fn normalize_config(cfg: &mut PulseConfigDto, secrets: &PulseSecrets) {
    cfg.interval_mins = cfg.interval_mins.clamp(5, 60);
    cfg.gmail_tick_multiplier = cfg.gmail_tick_multiplier.max(1);
    cfg.forms_sync_tick_multiplier = cfg.forms_sync_tick_multiplier.max(1);
    cfg.quiet_hours_start = cfg.quiet_hours_start.min(23);
    cfg.quiet_hours_end = cfg.quiet_hours_end.min(23);
    cfg.webhook_configured = secrets
        .webhook_url
        .as_deref()
        .is_some_and(|s| !s.trim().is_empty());
    cfg.telegram_configured = secrets
        .telegram_bot_token
        .as_deref()
        .is_some_and(|s| !s.trim().is_empty())
        && secrets
            .telegram_chat_id
            .as_deref()
            .is_some_and(|s| !s.trim().is_empty());
}

fn apply_secret_patch(slot: &mut Option<String>, patch: Option<String>) {
    if let Some(val) = patch {
        let trimmed = val.trim();
        if trimmed.is_empty() {
            *slot = None;
        } else {
            *slot = Some(trimmed.to_string());
        }
    }
}

fn read_json_or_default<T>(path: &Path) -> Result<T, CoreError>
where
    T: Default + for<'de> Deserialize<'de>,
{
    if !path.exists() {
        return Ok(T::default());
    }
    let raw = fs::read_to_string(path).map_err(|e| {
        CoreError::SubprocessError(format!("Failed reading '{}': {e}", path.display()))
    })?;
    if raw.trim().is_empty() {
        return Ok(T::default());
    }
    serde_json::from_str::<T>(&raw).map_err(|e| {
        CoreError::SubprocessError(format!("Invalid JSON in '{}': {e}", path.display()))
    })
}

fn read_jsonl_events(path: &Path) -> Vec<PulseEventDto> {
    let Ok(raw) = fs::read_to_string(path) else {
        return Vec::new();
    };
    raw.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .filter_map(|line| serde_json::from_str::<PulseEventDto>(line).ok())
        .collect()
}

fn urgency_rank(u: PulseUrgency) -> u8 {
    match u {
        PulseUrgency::Critical => 4,
        PulseUrgency::High => 3,
        PulseUrgency::Medium => 2,
        PulseUrgency::Low => 1,
    }
}
