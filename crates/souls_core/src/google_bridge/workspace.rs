//! Google Workspace Life Hub REST clients (Gmail, Calendar, Drive, Docs, Sheets, Slides),
//! incremental Docs Scratchpad store, deterministic triage/focus/financial analyzers,
//! and optional `gws` CLI bridge.

use super::oauth::{base64url_decode, base64url_encode, find_binary_on_path, percent_encode};
use super::security::wrap_untrusted_content;
use super::session::{write_bytes_atomic, write_json_atomic_lf};
use crate::error::CoreError;
use crate::web::html_to_clean_markdown;
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use serde_json::{json, Value};
use souls_protocol::google::{
    CalendarEventDto, DocsScratchpadDto, DriveFileDto, FocusSlotDto, GmailMessageSummaryDto,
    GmailTriageResultDto, GoogleCliProbeDto, SheetRangeAnalysisDto, SlideDeckPlanDto,
    SlideItemDto,
};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command;

/// REST client for Google Workspace APIs with configurable base URLs for offline mock testing.
#[derive(Debug, Clone)]
pub struct GoogleWorkspaceClient {
    http: reqwest::Client,
    gmail_base: String,
    calendar_base: String,
    drive_base: String,
    docs_base: String,
    sheets_base: String,
    slides_base: String,
}

impl Default for GoogleWorkspaceClient {
    fn default() -> Self {
        Self::new()
    }
}

impl GoogleWorkspaceClient {
    /// Creates a client configured for official Google Workspace production endpoints.
    pub fn new() -> Self {
        Self::with_base_urls(
            "https://gmail.googleapis.com/gmail/v1",
            "https://www.googleapis.com/calendar/v3",
            "https://www.googleapis.com/drive/v3",
            "https://docs.googleapis.com/v1",
            "https://sheets.googleapis.com/v4",
            "https://slides.googleapis.com/v1",
        )
    }

    /// Creates a client with overridden base URLs (e.g. pointing to local `TcpListener` mocks).
    pub fn with_base_urls(
        gmail_base: impl Into<String>,
        calendar_base: impl Into<String>,
        drive_base: impl Into<String>,
        docs_base: impl Into<String>,
        sheets_base: impl Into<String>,
        slides_base: impl Into<String>,
    ) -> Self {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_default();

        Self {
            http,
            gmail_base: trim_trailing_slash(gmail_base.into()),
            calendar_base: trim_trailing_slash(calendar_base.into()),
            drive_base: trim_trailing_slash(drive_base.into()),
            docs_base: trim_trailing_slash(docs_base.into()),
            sheets_base: trim_trailing_slash(sheets_base.into()),
            slides_base: trim_trailing_slash(slides_base.into()),
        }
    }

    // ------------------------------------------------------------------------
    // Gmail
    // ------------------------------------------------------------------------

    /// Lists Gmail messages matching `query`, resolving metadata headers and deterministic priority scores.
    pub async fn list_messages(
        &self,
        access_token: &str,
        query: &str,
        max_results: usize,
    ) -> Result<Vec<GmailMessageSummaryDto>, CoreError> {
        let limit = max_results.max(1);
        let url = format!(
            "{}/users/me/messages?q={}&maxResults={}",
            self.gmail_base,
            percent_encode(query),
            limit
        );
        let list_val = self.get_json(&url, access_token).await?;

        let items = list_val
            .get("messages")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let mut summaries = Vec::new();
        for item in items.into_iter().take(limit) {
            let id = item
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            if id.is_empty() {
                continue;
            }

            let msg_val = if item.get("payload").is_some() || item.get("subject").is_some() {
                item
            } else {
                let meta_url = format!(
                    "{}/users/me/messages/{}?format=metadata&metadataHeaders=From&metadataHeaders=Subject&metadataHeaders=Date",
                    self.gmail_base,
                    percent_encode(&id)
                );
                self.get_json(&meta_url, access_token).await?
            };

            summaries.push(parse_gmail_summary(&id, &msg_val));
        }

        Ok(summaries)
    }

    /// Fetches a full Gmail message and wraps its external content in a randomized untrusted delimiter.
    pub async fn get_message_framed(
        &self,
        access_token: &str,
        msg_id: &str,
    ) -> Result<String, CoreError> {
        let url = format!(
            "{}/users/me/messages/{}?format=full",
            self.gmail_base,
            percent_encode(msg_id)
        );
        let msg_val = self.get_json(&url, access_token).await?;

        let summary = parse_gmail_summary(msg_id, &msg_val);
        let body_text = msg_val
            .get("payload")
            .and_then(extract_gmail_body)
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| summary.snippet.clone());

        let combined = format!(
            "From: {}\nSubject: {}\nDate: {}\n\n{}",
            summary.from, summary.subject, summary.date, body_text
        );

        Ok(wrap_untrusted_content(
            &format!("gmail:message:{msg_id}"),
            &combined,
        ))
    }

    /// Fetches all messages in a Gmail thread and wraps the combined transcript
    /// in a randomized untrusted delimiter.
    pub async fn get_gmail_thread_text(
        &self,
        access_token: &str,
        thread_id: &str,
    ) -> Result<String, CoreError> {
        let url = format!(
            "{}/users/me/threads/{}?format=full",
            self.gmail_base,
            percent_encode(thread_id)
        );
        let thread_val = self.get_json(&url, access_token).await?;
        let messages = thread_val
            .get("messages")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let mut parts = Vec::with_capacity(messages.len());
        for msg in &messages {
            let msg_id = msg
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or(thread_id);
            let summary = parse_gmail_summary(msg_id, msg);
            let body_text = msg
                .get("payload")
                .and_then(extract_gmail_body)
                .filter(|s| !s.trim().is_empty())
                .unwrap_or_else(|| summary.snippet.clone());
            parts.push(format!(
                "From: {}\nSubject: {}\nDate: {}\n\n{}",
                summary.from, summary.subject, summary.date, body_text
            ));
        }

        let combined = parts.join("\n\n---\n\n");
        Ok(wrap_untrusted_content(
            &format!("gmail:thread:{thread_id}"),
            &combined,
        ))
    }

    /// Deterministically scores and classifies Gmail messages, separating newsletters
    /// and framing untrusted snippets.
    pub fn triage_inbox_deterministically(
        messages: &[GmailMessageSummaryDto],
    ) -> GmailTriageResultDto {
        let mut newsletter_count = 0usize;
        let mut priority_messages = Vec::new();

        for msg in messages {
            let mut scored = msg.clone();
            let (score, category, requires_reply, is_newsletter) = classify_gmail_message(&scored);
            scored.priority_score = score;
            scored.category = category;
            scored.requires_reply = requires_reply;

            if is_newsletter {
                newsletter_count += 1;
            } else {
                if !scored.snippet.trim().is_empty() {
                    scored.snippet = wrap_untrusted_content(
                        &format!("gmail:snippet:{}", scored.id),
                        &scored.snippet,
                    );
                }
                priority_messages.push(scored);
            }
        }

        priority_messages.sort_by(|a, b| {
            b.priority_score
                .partial_cmp(&a.priority_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        GmailTriageResultDto {
            account_email: String::new(),
            total_scanned: messages.len(),
            priority_messages,
            newsletter_count,
            untrusted_framed: true,
        }
    }

    /// Creates a Gmail draft (`Tier1SafeReversible`).
    pub async fn create_draft(
        &self,
        access_token: &str,
        to: &str,
        subject: &str,
        body: &str,
    ) -> Result<Value, CoreError> {
        let raw_rfc2822 = encode_rfc2822_base64url(to, subject, body);
        let url = format!("{}/users/me/drafts", self.gmail_base);
        self.post_json(
            &url,
            access_token,
            &json!({
                "message": { "raw": raw_rfc2822 }
            }),
        )
        .await
    }

    /// Applies bulk label modifications across Gmail message IDs.
    pub async fn bulk_modify_labels(
        &self,
        access_token: &str,
        ids: &[String],
        add_labels: &[String],
        remove_labels: &[String],
    ) -> Result<Value, CoreError> {
        let url = format!("{}/users/me/messages/batchModify", self.gmail_base);
        self.post_json(
            &url,
            access_token,
            &json!({
                "ids": ids,
                "addLabelIds": add_labels,
                "removeLabelIds": remove_labels,
            }),
        )
        .await
    }

    /// Sends an email message (`Tier2ExternalDestructive` — caller must gate via BMAD Inbox).
    pub async fn send_message(
        &self,
        access_token: &str,
        to: &str,
        subject: &str,
        body: &str,
    ) -> Result<Value, CoreError> {
        let raw_rfc2822 = encode_rfc2822_base64url(to, subject, body);
        let url = format!("{}/users/me/messages/send", self.gmail_base);
        self.post_json(
            &url,
            access_token,
            &json!({
                "raw": raw_rfc2822
            }),
        )
        .await
    }

    // ------------------------------------------------------------------------
    // Calendar
    // ------------------------------------------------------------------------

    /// Lists primary Google Calendar events between `time_min` and `time_max` (RFC 3339).
    pub async fn list_events(
        &self,
        access_token: &str,
        time_min: &str,
        time_max: &str,
    ) -> Result<Vec<CalendarEventDto>, CoreError> {
        let url = format!(
            "{}/calendars/primary/events?timeMin={}&timeMax={}&singleEvents=true&orderBy=startTime",
            self.calendar_base,
            percent_encode(time_min),
            percent_encode(time_max)
        );
        let val = self.get_json(&url, access_token).await?;
        let items = val
            .get("items")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        Ok(items.iter().map(parse_calendar_event).collect())
    }

    /// Computes uninterrupted deep-focus slots within `[work_start_utc_iso, work_end_utc_iso]`
    /// of at least `min_duration_mins`.
    pub fn find_deep_focus_slots(
        events: &[CalendarEventDto],
        work_start_utc_iso: &str,
        work_end_utc_iso: &str,
        min_duration_mins: u32,
    ) -> Vec<FocusSlotDto> {
        let Ok(win_start) = DateTime::parse_from_rfc3339(work_start_utc_iso) else {
            return Vec::new();
        };
        let Ok(win_end) = DateTime::parse_from_rfc3339(work_end_utc_iso) else {
            return Vec::new();
        };

        let start_utc = win_start.with_timezone(&Utc);
        let end_utc = win_end.with_timezone(&Utc);
        if end_utc <= start_utc {
            return Vec::new();
        }

        let mut busy: Vec<(DateTime<Utc>, DateTime<Utc>)> = Vec::new();
        for ev in events {
            if ev.status.eq_ignore_ascii_case("cancelled") {
                continue;
            }
            let Ok(ev_start) = DateTime::parse_from_rfc3339(&ev.start_rfc3339) else {
                continue;
            };
            let Ok(ev_end) = DateTime::parse_from_rfc3339(&ev.end_rfc3339) else {
                continue;
            };
            let s = ev_start.with_timezone(&Utc).max(start_utc);
            let e = ev_end.with_timezone(&Utc).min(end_utc);
            if e > s {
                busy.push((s, e));
            }
        }

        busy.sort_by_key(|(s, _)| *s);

        let mut merged: Vec<(DateTime<Utc>, DateTime<Utc>)> = Vec::new();
        for (s, e) in busy {
            if let Some(last) = merged.last_mut() {
                if s <= last.1 {
                    last.1 = last.1.max(e);
                    continue;
                }
            }
            merged.push((s, e));
        }

        let min_dur = ChronoDuration::minutes(min_duration_mins as i64);
        let mut slots = Vec::new();
        let mut cursor = start_utc;

        for (busy_s, busy_e) in merged {
            if busy_s > cursor && busy_s - cursor >= min_dur {
                let mins = (busy_s - cursor).num_minutes() as u32;
                slots.push(FocusSlotDto {
                    start_rfc3339: cursor.to_rfc3339(),
                    end_rfc3339: busy_s.to_rfc3339(),
                    duration_minutes: mins,
                    label: focus_slot_label(cursor, mins),
                });
            }
            cursor = cursor.max(busy_e);
        }

        if end_utc > cursor && end_utc - cursor >= min_dur {
            let mins = (end_utc - cursor).num_minutes() as u32;
            slots.push(FocusSlotDto {
                start_rfc3339: cursor.to_rfc3339(),
                end_rfc3339: end_utc.to_rfc3339(),
                duration_minutes: mins,
                label: focus_slot_label(cursor, mins),
            });
        }

        slots
    }

    /// Creates a primary Calendar event (`send_updates` defaults to `"none"` to suppress external emails).
    #[allow(clippy::too_many_arguments)]
    pub async fn create_event(
        &self,
        access_token: &str,
        summary: &str,
        start_rfc3339: &str,
        end_rfc3339: &str,
        location: Option<&str>,
        attendees: &[String],
        send_updates: Option<&str>,
    ) -> Result<CalendarEventDto, CoreError> {
        let updates = send_updates.unwrap_or("none");
        let url = format!(
            "{}/calendars/primary/events?sendUpdates={}",
            self.calendar_base,
            percent_encode(updates)
        );

        let mut payload = json!({
            "summary": summary,
            "start": { "dateTime": start_rfc3339 },
            "end": { "dateTime": end_rfc3339 },
        });

        if let Some(loc) = location.filter(|l| !l.trim().is_empty()) {
            payload["location"] = json!(loc);
        }
        if !attendees.is_empty() {
            let att_objs: Vec<Value> = attendees.iter().map(|e| json!({ "email": e })).collect();
            payload["attendees"] = Value::Array(att_objs);
        }

        let created = self.post_json(&url, access_token, &payload).await?;
        Ok(parse_calendar_event(&created))
    }

    /// Deletes a Calendar event (`Tier2ExternalDestructive`).
    pub async fn delete_event(&self, access_token: &str, event_id: &str) -> Result<(), CoreError> {
        let url = format!(
            "{}/calendars/primary/events/{}",
            self.calendar_base,
            percent_encode(event_id)
        );
        self.delete_ok(&url, access_token).await
    }

    // ------------------------------------------------------------------------
    // Drive
    // ------------------------------------------------------------------------

    /// Searches Google Drive files matching a Drive query string.
    pub async fn search_files(
        &self,
        access_token: &str,
        query: &str,
        page_size: usize,
    ) -> Result<Vec<DriveFileDto>, CoreError> {
        let limit = page_size.clamp(1, 100);
        let url = format!(
            "{}/files?q={}&pageSize={}&fields=files(id,name,mimeType,modifiedTime,webViewLink,size)",
            self.drive_base,
            percent_encode(query),
            limit
        );
        let val = self.get_json(&url, access_token).await?;
        let files = val
            .get("files")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        Ok(files.iter().map(parse_drive_file).collect())
    }

    /// Retrieves metadata for a specific Google Drive file.
    pub async fn get_file_metadata(
        &self,
        access_token: &str,
        file_id: &str,
    ) -> Result<DriveFileDto, CoreError> {
        let url = format!(
            "{}/files/{}?fields=id,name,mimeType,modifiedTime,webViewLink,size",
            self.drive_base,
            percent_encode(file_id)
        );
        let val = self.get_json(&url, access_token).await?;
        Ok(parse_drive_file(&val))
    }

    /// Downloads a Drive file strictly into `sandbox_exchange_dir`, blocking any path traversal.
    pub async fn download_to_sandbox(
        &self,
        access_token: &str,
        file_id: &str,
        filename: &str,
        sandbox_exchange_dir: &Path,
    ) -> Result<PathBuf, CoreError> {
        validate_safe_filename(filename)?;

        fs::create_dir_all(sandbox_exchange_dir).map_err(|e| {
            CoreError::SubprocessError(format!(
                "Failed to create sandbox dir '{}': {e}",
                sandbox_exchange_dir.display()
            ))
        })?;

        let target_path = sandbox_exchange_dir.join(filename);
        let url = format!(
            "{}/files/{}?alt=media",
            self.drive_base,
            percent_encode(file_id)
        );

        let resp = self
            .http
            .get(&url)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| CoreError::SubprocessError(format!("Drive download failed: {e}")))?;

        let status = resp.status();
        if !status.is_success() {
            let err_text = resp.text().await.unwrap_or_default();
            return Err(CoreError::SubprocessError(format!(
                "Drive download HTTP {status}: {err_text}"
            )));
        }

        let bytes = resp
            .bytes()
            .await
            .map_err(|e| CoreError::SubprocessError(format!("Drive read bytes failed: {e}")))?;

        write_bytes_atomic(&target_path, &bytes)?;
        Ok(target_path)
    }

    /// Exports a native Google Workspace document (Docs/Sheets/Slides) as plain text or CSV/HTML.
    pub async fn export_drive_file_text(
        &self,
        access_token: &str,
        file_id: &str,
        mime_type: &str,
    ) -> Result<String, CoreError> {
        let url = format!(
            "{}/files/{}/export?mimeType={}",
            self.drive_base,
            percent_encode(file_id),
            percent_encode(mime_type)
        );
        let resp = self
            .http
            .get(&url)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| CoreError::SubprocessError(format!("Drive export failed: {e}")))?;
        let status = resp.status();
        let text = resp
            .text()
            .await
            .map_err(|e| CoreError::SubprocessError(format!("Drive export read failed: {e}")))?;
        if !status.is_success() {
            return Err(CoreError::SubprocessError(format!(
                "Drive export HTTP {status}: {text}"
            )));
        }
        Ok(text)
    }

    /// Downloads raw Drive file bytes into memory while enforcing a hard `max_bytes` cap.
    pub async fn download_drive_file_bytes(
        &self,
        access_token: &str,
        file_id: &str,
        max_bytes: usize,
    ) -> Result<Vec<u8>, CoreError> {
        let url = format!(
            "{}/files/{}?alt=media",
            self.drive_base,
            percent_encode(file_id)
        );
        let mut resp = self
            .http
            .get(&url)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| CoreError::SubprocessError(format!("Drive download failed: {e}")))?;
        let status = resp.status();
        if !status.is_success() {
            let err_text = resp.text().await.unwrap_or_default();
            return Err(CoreError::SubprocessError(format!(
                "Drive download HTTP {status}: {err_text}"
            )));
        }

        if let Some(cl) = resp.content_length() {
            if cl as usize > max_bytes {
                return Err(CoreError::SubprocessError(format!(
                    "Drive file size ({cl} bytes) exceeds max_bytes limit ({max_bytes})"
                )));
            }
        }

        let mut buf = Vec::new();
        while let Some(chunk) = resp
            .chunk()
            .await
            .map_err(|e| CoreError::SubprocessError(format!("Drive stream read failed: {e}")))?
        {
            if buf.len().saturating_add(chunk.len()) > max_bytes {
                return Err(CoreError::SubprocessError(format!(
                    "Drive file download exceeded max_bytes limit ({max_bytes})"
                )));
            }
            buf.extend_from_slice(&chunk);
        }
        Ok(buf)
    }

    /// Updates sharing permissions on a Drive file (`Tier2ExternalDestructive`).
    pub async fn update_permissions(
        &self,
        access_token: &str,
        file_id: &str,
        role: &str,
        perm_type: &str,
        email_address: Option<&str>,
    ) -> Result<Value, CoreError> {
        let url = format!(
            "{}/files/{}/permissions",
            self.drive_base,
            percent_encode(file_id)
        );
        let mut body = json!({
            "role": role,
            "type": perm_type,
        });
        if let Some(email) = email_address.filter(|s| !s.trim().is_empty()) {
            body["emailAddress"] = json!(email);
        }
        self.post_json(&url, access_token, &body).await
    }

    /// Deletes a Drive file (`Tier2ExternalDestructive`).
    pub async fn delete_file(&self, access_token: &str, file_id: &str) -> Result<(), CoreError> {
        let url = format!("{}/files/{}", self.drive_base, percent_encode(file_id));
        self.delete_ok(&url, access_token).await
    }

    // ------------------------------------------------------------------------
    // Docs
    // ------------------------------------------------------------------------

    /// Pushes rendered text content to a Google Doc via `documents:batchUpdate`.
    pub async fn push_docs_content(
        &self,
        access_token: &str,
        doc_id: &str,
        content: &str,
    ) -> Result<Value, CoreError> {
        let doc_url = format!("{}/documents/{}", self.docs_base, percent_encode(doc_id));
        let doc_val = self.get_json(&doc_url, access_token).await?;

        let end_index = doc_val
            .pointer("/body/content")
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.last())
            .and_then(|elem| elem.get("endIndex"))
            .and_then(|v| v.as_i64())
            .unwrap_or(1);

        let mut requests = Vec::new();
        if end_index > 2 {
            requests.push(json!({
                "deleteContentRange": {
                    "range": {
                        "startIndex": 1,
                        "endIndex": end_index - 1
                    }
                }
            }));
        }
        if !content.is_empty() {
            requests.push(json!({
                "insertText": {
                    "location": { "index": 1 },
                    "text": content
                }
            }));
        }

        let batch_url = format!(
            "{}/documents/{}:batchUpdate",
            self.docs_base,
            percent_encode(doc_id)
        );
        self.post_json(&batch_url, access_token, &json!({ "requests": requests }))
            .await
    }

    // ------------------------------------------------------------------------
    // Sheets
    // ------------------------------------------------------------------------

    /// Reads cell range values from a Google Sheet as strings.
    pub async fn read_range(
        &self,
        access_token: &str,
        spreadsheet_id: &str,
        range: &str,
    ) -> Result<Vec<Vec<String>>, CoreError> {
        let url = format!(
            "{}/spreadsheets/{}/values/{}",
            self.sheets_base,
            percent_encode(spreadsheet_id),
            percent_encode(range)
        );
        let val = self.get_json(&url, access_token).await?;
        let rows = val
            .get("values")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        Ok(rows
            .into_iter()
            .map(|row| {
                row.as_array()
                    .map(|cells| cells.iter().map(json_cell_to_string).collect())
                    .unwrap_or_default()
            })
            .collect())
    }

    /// Deterministically analyzes financial/numeric rows, computing column totals,
    /// category breakdowns, >2x-mean anomalies, and a 30-day projection.
    pub fn analyze_financial_rows(
        spreadsheet_id: &str,
        range: &str,
        rows: &[Vec<String>],
    ) -> SheetRangeAnalysisDto {
        if rows.is_empty() {
            return SheetRangeAnalysisDto {
                spreadsheet_id: spreadsheet_id.to_string(),
                range: range.to_string(),
                row_count: 0,
                numeric_columns: Vec::new(),
                totals_by_column: Vec::new(),
                category_breakdown: Vec::new(),
                anomalies: Vec::new(),
                projected_monthly_total: None,
            };
        }

        let first_row_is_header = rows.len() > 1
            && rows[0]
                .iter()
                .any(|cell| !cell.trim().is_empty() && parse_numeric_cell(cell).is_none());

        let max_cols = rows.iter().map(Vec::len).max().unwrap_or(0);
        let headers: Vec<String> = if first_row_is_header {
            (0..max_cols)
                .map(|c| {
                    rows[0]
                        .get(c)
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .unwrap_or_else(|| format!("Col_{}", c + 1))
                })
                .collect()
        } else {
            (0..max_cols).map(|c| format!("Col_{}", c + 1)).collect()
        };

        let data_rows: &[Vec<String>] = if first_row_is_header {
            &rows[1..]
        } else {
            rows
        };

        let mut numeric_columns = Vec::new();
        let mut numeric_col_indices = Vec::new();
        let mut totals_by_column = Vec::new();

        for (col_idx, header) in headers.iter().enumerate() {
            let mut non_empty = 0usize;
            let mut parsed_vals = Vec::new();

            for row in data_rows {
                if let Some(cell) = row.get(col_idx) {
                    let trimmed = cell.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    non_empty += 1;
                    if let Some(v) = parse_numeric_cell(trimmed) {
                        parsed_vals.push(v);
                    }
                }
            }

            if non_empty > 0 && parsed_vals.len() * 2 >= non_empty {
                let sum: f64 = parsed_vals.iter().sum();
                numeric_columns.push(header.clone());
                numeric_col_indices.push(col_idx);
                totals_by_column.push((header.clone(), round2(sum)));
            }
        }

        let first_text_col = (0..max_cols).find(|c| !numeric_col_indices.contains(c));
        let primary_num_col = select_primary_numeric_col(&headers, &numeric_col_indices);

        let mut cat_totals: HashMap<String, f64> = HashMap::new();
        let mut anomalies = Vec::new();

        if let Some(num_col) = primary_num_col {
            let mut row_values: Vec<(usize, String, f64)> = Vec::new();
            for (r_idx, row) in data_rows.iter().enumerate() {
                if let Some(val) = row.get(num_col).and_then(|c| parse_numeric_cell(c)) {
                    let cat = first_text_col
                        .and_then(|tc| row.get(tc))
                        .map(|s| s.trim())
                        .filter(|s| !s.is_empty())
                        .unwrap_or("Uncategorized")
                        .to_string();
                    *cat_totals.entry(cat.clone()).or_insert(0.0) += val;
                    row_values.push((r_idx + if first_row_is_header { 2 } else { 1 }, cat, val));
                }
            }

            if row_values.len() >= 2 {
                let sum: f64 = row_values.iter().map(|(_, _, v)| *v).sum();
                let mean = sum / row_values.len() as f64;
                if mean > 0.0 {
                    for (sheet_row, cat, val) in &row_values {
                        if *val > 2.0 * mean {
                            anomalies.push(format!(
                                "Row {} ({}): {:.2} exceeds 2x mean ({:.2})",
                                sheet_row, cat, val, mean
                            ));
                        }
                    }
                }
            }
        }

        let mut category_breakdown: Vec<(String, f64)> = cat_totals
            .into_iter()
            .map(|(k, v)| (k, round2(v)))
            .collect();
        category_breakdown.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.0.cmp(&b.0))
        });

        let projected_monthly_total = if !data_rows.is_empty() {
            primary_num_col.and_then(|col| {
                headers
                    .get(col)
                    .and_then(|h| totals_by_column.iter().find(|(name, _)| name == h))
                    .map(|(_, total)| round2((*total / data_rows.len() as f64) * 30.0))
            })
        } else {
            None
        };

        SheetRangeAnalysisDto {
            spreadsheet_id: spreadsheet_id.to_string(),
            range: range.to_string(),
            row_count: data_rows.len(),
            numeric_columns,
            totals_by_column,
            category_breakdown,
            anomalies,
            projected_monthly_total,
        }
    }

    /// Writes values into a Google Sheet range (`USER_ENTERED`).
    pub async fn write_range(
        &self,
        access_token: &str,
        spreadsheet_id: &str,
        range: &str,
        values: Vec<Vec<Value>>,
    ) -> Result<Value, CoreError> {
        let url = format!(
            "{}/spreadsheets/{}/values/{}?valueInputOption=USER_ENTERED",
            self.sheets_base,
            percent_encode(spreadsheet_id),
            percent_encode(range)
        );
        self.put_json(
            &url,
            access_token,
            &json!({
                "range": range,
                "majorDimension": "ROWS",
                "values": values,
            }),
        )
        .await
    }

    // ------------------------------------------------------------------------
    // Slides
    // ------------------------------------------------------------------------

    /// Builds a structured `SlideDeckPlanDto` clamped to `5..=15` slides.
    pub fn build_slide_deck_plan(
        topic: &str,
        briefing: &str,
        target_slides: usize,
    ) -> SlideDeckPlanDto {
        let count = target_slides.clamp(5, 15);
        let chunks: Vec<String> = briefing
            .lines()
            .flat_map(|l| l.split('.'))
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(ToString::to_string)
            .collect();

        let mut slides = Vec::with_capacity(count);
        slides.push(SlideItemDto {
            index: 0,
            title: topic.to_string(),
            bullets: vec![
                "Executive Overview".to_string(),
                chunks
                    .first()
                    .cloned()
                    .unwrap_or_else(|| format!("Key context for {topic}")),
            ],
            speaker_notes: Some(format!("Introduce '{topic}' and summarize objectives.")),
        });

        for i in 1..(count - 1) {
            let point = chunks
                .get((i - 1) % chunks.len().max(1))
                .cloned()
                .unwrap_or_else(|| format!("Analysis pillar {i} for {topic}"));
            slides.push(SlideItemDto {
                index: i,
                title: format!("Section {}: Key Insights", i),
                bullets: vec![point, format!("Supporting evidence & metrics (#{i})")],
                speaker_notes: Some(format!("Elaborate on section {i} insights.")),
            });
        }

        slides.push(SlideItemDto {
            index: count - 1,
            title: "Next Steps & Action Items".to_string(),
            bullets: vec![
                "Confirm owners and milestones".to_string(),
                "Review open risks and follow-up schedule".to_string(),
            ],
            speaker_notes: Some("Close with concrete commitments and timeline.".to_string()),
        });

        SlideDeckPlanDto {
            title: topic.to_string(),
            topic: topic.to_string(),
            slides,
            presentation_id: None,
        }
    }

    /// Creates a Google Slides presentation and populates slide placeholders via `batchUpdate`.
    pub async fn create_presentation(
        &self,
        access_token: &str,
        plan: &SlideDeckPlanDto,
    ) -> Result<SlideDeckPlanDto, CoreError> {
        let create_url = format!("{}/presentations", self.slides_base);
        let created = self
            .post_json(&create_url, access_token, &json!({ "title": plan.title }))
            .await?;

        let pres_id = created
            .get("presentationId")
            .and_then(|v| v.as_str())
            .unwrap_or("pres_created")
            .to_string();

        let mut requests = Vec::new();
        for slide in &plan.slides {
            let slide_id = format!("slide_{}", slide.index);
            requests.push(json!({
                "createSlide": {
                    "objectId": slide_id,
                    "insertionIndex": slide.index,
                    "slideLayoutReference": { "predefinedLayout": "TITLE_AND_BODY" }
                }
            }));
        }

        if !requests.is_empty() {
            let batch_url = format!(
                "{}/presentations/{}:batchUpdate",
                self.slides_base,
                percent_encode(&pres_id)
            );
            let _ = self
                .post_json(&batch_url, access_token, &json!({ "requests": requests }))
                .await?;
        }

        let mut out = plan.clone();
        out.presentation_id = Some(pres_id);
        Ok(out)
    }

    // ------------------------------------------------------------------------
    // Shared HTTP helpers
    // ------------------------------------------------------------------------

    async fn get_json(&self, url: &str, access_token: &str) -> Result<Value, CoreError> {
        let resp = self
            .http
            .get(url)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| CoreError::SubprocessError(format!("Workspace GET '{url}' failed: {e}")))?;
        parse_json_response(resp, url).await
    }

    async fn post_json(
        &self,
        url: &str,
        access_token: &str,
        body: &Value,
    ) -> Result<Value, CoreError> {
        let resp = self
            .http
            .post(url)
            .bearer_auth(access_token)
            .json(body)
            .send()
            .await
            .map_err(|e| {
                CoreError::SubprocessError(format!("Workspace POST '{url}' failed: {e}"))
            })?;
        parse_json_response(resp, url).await
    }

    async fn put_json(
        &self,
        url: &str,
        access_token: &str,
        body: &Value,
    ) -> Result<Value, CoreError> {
        let resp = self
            .http
            .put(url)
            .bearer_auth(access_token)
            .json(body)
            .send()
            .await
            .map_err(|e| CoreError::SubprocessError(format!("Workspace PUT '{url}' failed: {e}")))?;
        parse_json_response(resp, url).await
    }

    async fn delete_ok(&self, url: &str, access_token: &str) -> Result<(), CoreError> {
        let resp = self
            .http
            .delete(url)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| {
                CoreError::SubprocessError(format!("Workspace DELETE '{url}' failed: {e}"))
            })?;
        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(CoreError::SubprocessError(format!(
                "Workspace DELETE '{url}' failed with HTTP {status}: {text}"
            )));
        }
        Ok(())
    }
}

pub(crate) async fn parse_json_response(
    resp: reqwest::Response,
    url: &str,
) -> Result<Value, CoreError> {
    let status = resp.status();
    let text = resp.text().await.map_err(|e| {
        CoreError::SubprocessError(format!("Failed reading HTTP body from '{url}': {e}"))
    })?;
    if !status.is_success() {
        return Err(CoreError::SubprocessError(format!(
            "HTTP {status} from '{url}': {text}"
        )));
    }
    if text.trim().is_empty() {
        return Ok(json!({}));
    }
    serde_json::from_str::<Value>(&text).map_err(|e| {
        CoreError::SubprocessError(format!("Invalid JSON response from '{url}': {e}"))
    })
}

fn trim_trailing_slash(s: String) -> String {
    s.trim_end_matches('/').to_string()
}

fn validate_safe_filename(filename: &str) -> Result<(), CoreError> {
    let trimmed = filename.trim();
    if trimmed.is_empty()
        || trimmed == "."
        || trimmed == ".."
        || trimmed.contains("..")
        || trimmed.contains('/')
        || trimmed.contains('\\')
        || trimmed.contains(':')
        || trimmed.contains('\0')
    {
        return Err(CoreError::RefsPathLeakViolation(format!(
            "Unsafe sandbox filename rejected: '{filename}'"
        )));
    }
    Ok(())
}

/// Incremental line-addressed local scratchpad store for Google Docs (`<data_dir>/scratchpads/{doc_id}.json`).
pub struct DocsScratchpadStore {
    scratchpads_dir: PathBuf,
}

impl DocsScratchpadStore {
    /// Opens (or initializes) a `DocsScratchpadStore` rooted under `data_dir`.
    pub fn open(data_dir: &Path) -> Result<Self, CoreError> {
        let scratchpads_dir = data_dir.join("scratchpads");
        fs::create_dir_all(&scratchpads_dir).map_err(|e| {
            CoreError::SubprocessError(format!(
                "Failed to create scratchpads directory '{}': {e}",
                scratchpads_dir.display()
            ))
        })?;
        Ok(Self { scratchpads_dir })
    }

    /// Loads an existing scratchpad or creates and persists a new empty one.
    pub fn load_or_create(&self, doc_id: &str, title: &str) -> Result<DocsScratchpadDto, CoreError> {
        let path = self.doc_path(doc_id)?;
        if path.exists() {
            return self.load(doc_id);
        }

        let dto = DocsScratchpadDto {
            doc_id: doc_id.to_string(),
            title: title.to_string(),
            lines: Vec::new(),
            updated_at: Utc::now().timestamp(),
            revision: 1,
        };
        write_json_atomic_lf(&path, &dto)?;
        Ok(dto)
    }

    /// Replaces inclusive 1-based line range `[start_1based, end_1based]` with `new_lines`.
    pub fn replace_lines(
        &self,
        doc_id: &str,
        start_1based: usize,
        end_1based: usize,
        new_lines: Vec<String>,
    ) -> Result<DocsScratchpadDto, CoreError> {
        let mut doc = self.load(doc_id)?;
        if start_1based == 0 || start_1based > end_1based || end_1based > doc.lines.len() {
            return Err(CoreError::SubprocessError(format!(
                "Invalid replace_lines range [{start_1based}..={end_1based}] for {} lines",
                doc.lines.len()
            )));
        }

        doc.lines.splice((start_1based - 1)..end_1based, new_lines);
        doc.revision = doc.revision.saturating_add(1);
        doc.updated_at = Utc::now().timestamp();
        let path = self.doc_path(doc_id)?;
        write_json_atomic_lf(&path, &doc)?;
        Ok(doc)
    }

    /// Inserts `lines` before 1-based index `at_1based` (`len + 1` appends at end).
    pub fn insert_lines(
        &self,
        doc_id: &str,
        at_1based: usize,
        lines: Vec<String>,
    ) -> Result<DocsScratchpadDto, CoreError> {
        let mut doc = self.load(doc_id)?;
        if at_1based == 0 || at_1based > doc.lines.len() + 1 {
            return Err(CoreError::SubprocessError(format!(
                "Invalid insert_lines index {at_1based} for {} lines",
                doc.lines.len()
            )));
        }

        let idx = at_1based - 1;
        doc.lines.splice(idx..idx, lines);
        doc.revision = doc.revision.saturating_add(1);
        doc.updated_at = Utc::now().timestamp();
        let path = self.doc_path(doc_id)?;
        write_json_atomic_lf(&path, &doc)?;
        Ok(doc)
    }

    /// Deletes inclusive 1-based line range `[start_1based, end_1based]`.
    pub fn delete_lines(
        &self,
        doc_id: &str,
        start_1based: usize,
        end_1based: usize,
    ) -> Result<DocsScratchpadDto, CoreError> {
        let mut doc = self.load(doc_id)?;
        if start_1based == 0 || start_1based > end_1based || end_1based > doc.lines.len() {
            return Err(CoreError::SubprocessError(format!(
                "Invalid delete_lines range [{start_1based}..={end_1based}] for {} lines",
                doc.lines.len()
            )));
        }

        doc.lines.drain((start_1based - 1)..end_1based);
        doc.revision = doc.revision.saturating_add(1);
        doc.updated_at = Utc::now().timestamp();
        let path = self.doc_path(doc_id)?;
        write_json_atomic_lf(&path, &doc)?;
        Ok(doc)
    }

    /// Renders the scratchpad buffer into pure-LF text.
    pub fn render_text(&self, doc_id: &str) -> Result<String, CoreError> {
        let doc = self.load(doc_id)?;
        Ok(doc.lines.join("\n"))
    }

    fn load(&self, doc_id: &str) -> Result<DocsScratchpadDto, CoreError> {
        let path = self.doc_path(doc_id)?;
        let raw = fs::read_to_string(&path).map_err(|e| {
            CoreError::SubprocessError(format!(
                "Scratchpad '{doc_id}' not found at '{}': {e}",
                path.display()
            ))
        })?;
        serde_json::from_str::<DocsScratchpadDto>(&raw).map_err(|e| {
            CoreError::SubprocessError(format!("Failed to parse scratchpad '{doc_id}': {e}"))
        })
    }

    fn doc_path(&self, doc_id: &str) -> Result<PathBuf, CoreError> {
        validate_safe_filename(doc_id)?;
        Ok(self.scratchpads_dir.join(format!("{doc_id}.json")))
    }
}

/// Optional companion bridge for invoking the `gws` CLI when installed.
#[derive(Debug, Clone)]
pub struct GwsCliBridge {
    binary_path: Option<PathBuf>,
}

impl Default for GwsCliBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl GwsCliBridge {
    /// Auto-discovers `gws` binary on PATH.
    pub fn new() -> Self {
        Self {
            binary_path: find_binary_on_path("gws"),
        }
    }

    /// Constructs a bridge targeting an explicit binary path.
    pub fn with_binary(path: PathBuf) -> Self {
        Self {
            binary_path: Some(path),
        }
    }

    /// Reports availability of the `gws` binary.
    pub fn detect(&self) -> GoogleCliProbeDto {
        match &self.binary_path {
            Some(path) if path.exists() => GoogleCliProbeDto {
                name: "gws".to_string(),
                installed: true,
                path: Some(path.display().to_string()),
                version: None,
                authenticated: false,
                notes: "gws CLI binary located.".to_string(),
            },
            _ => GoogleCliProbeDto {
                name: "gws".to_string(),
                installed: false,
                path: None,
                version: None,
                authenticated: false,
                notes: "gws CLI binary not found.".to_string(),
            },
        }
    }

    /// Executes `gws` with arguments under a strict timeout, parsing JSON output if present.
    pub async fn invoke(&self, args: &[&str], timeout_ms: u64) -> Result<Value, CoreError> {
        let bin = self.binary_path.as_ref().ok_or_else(|| {
            CoreError::SubprocessError("gws CLI is not installed on host".to_string())
        })?;

        let mut cmd = Command::new(bin);
        cmd.args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);

        let child = cmd
            .spawn()
            .map_err(|e| CoreError::SubprocessError(format!("Failed to spawn gws CLI: {e}")))?;

        let out = tokio::time::timeout(Duration::from_millis(timeout_ms), child.wait_with_output())
            .await
            .map_err(|_| CoreError::ExecutionTimeout(timeout_ms))?
            .map_err(|e| CoreError::SubprocessError(format!("gws CLI I/O error: {e}")))?;

        if !out.status.success() {
            let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
            return Err(CoreError::SubprocessError(format!(
                "gws CLI failed (code {:?}): {stderr}",
                out.status.code()
            )));
        }

        let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if stdout.is_empty() {
            return Ok(json!({}));
        }
        Ok(serde_json::from_str::<Value>(&stdout).unwrap_or_else(|_| json!({ "raw": stdout })))
    }
}

fn encode_rfc2822_base64url(to: &str, subject: &str, body: &str) -> String {
    let raw = format!(
        "To: {to}\r\nSubject: {subject}\r\nContent-Type: text/plain; charset=\"UTF-8\"\r\n\r\n{body}"
    );
    base64url_encode(raw.as_bytes())
}

fn parse_gmail_summary(id: &str, val: &Value) -> GmailMessageSummaryDto {
    let thread_id = val
        .get("threadId")
        .or_else(|| val.get("thread_id"))
        .and_then(|v| v.as_str())
        .unwrap_or(id)
        .to_string();

    let headers = val
        .pointer("/payload/headers")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let find_header = |name: &str| -> Option<String> {
        headers.iter().find_map(|h| {
            let h_name = h.get("name").and_then(|v| v.as_str())?;
            if h_name.eq_ignore_ascii_case(name) {
                h.get("value")
                    .and_then(|v| v.as_str())
                    .map(ToString::to_string)
            } else {
                None
            }
        })
    };

    let from = find_header("From")
        .or_else(|| val.get("from").and_then(|v| v.as_str()).map(ToString::to_string))
        .unwrap_or_default();
    let subject = find_header("Subject")
        .or_else(|| {
            val.get("subject")
                .and_then(|v| v.as_str())
                .map(ToString::to_string)
        })
        .unwrap_or_else(|| "(No Subject)".to_string());
    let date = find_header("Date")
        .or_else(|| val.get("date").and_then(|v| v.as_str()).map(ToString::to_string))
        .unwrap_or_default();
    let snippet = val
        .get("snippet")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    let labels = val
        .get("labelIds")
        .or_else(|| val.get("labels"))
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(ToString::to_string))
                .collect()
        })
        .unwrap_or_default();

    let mut summary = GmailMessageSummaryDto {
        id: id.to_string(),
        thread_id,
        from,
        subject,
        snippet,
        date,
        labels,
        priority_score: 0.5,
        category: "general".to_string(),
        requires_reply: false,
    };
    let (score, cat, req_reply, _) = classify_gmail_message(&summary);
    summary.priority_score = score;
    summary.category = cat;
    summary.requires_reply = req_reply;
    summary
}

fn extract_gmail_body(payload: &Value) -> Option<String> {
    let mime = payload
        .get("mimeType")
        .and_then(|v| v.as_str())
        .unwrap_or_default();

    if let Some(data) = payload.pointer("/body/data").and_then(|v| v.as_str()) {
        if !data.trim().is_empty() {
            if let Ok(decoded) = base64url_decode(data) {
                let text = String::from_utf8_lossy(&decoded).into_owned();
                if mime.eq_ignore_ascii_case("text/html") {
                    return Some(html_to_clean_markdown(&text));
                }
                return Some(text);
            }
        }
    }

    if let Some(parts) = payload.get("parts").and_then(|v| v.as_array()) {
        for part in parts {
            if part
                .get("mimeType")
                .and_then(|v| v.as_str())
                .is_some_and(|m| m.eq_ignore_ascii_case("text/plain"))
            {
                if let Some(found) = extract_gmail_body(part) {
                    return Some(found);
                }
            }
        }
        for part in parts {
            if let Some(found) = extract_gmail_body(part) {
                return Some(found);
            }
        }
    }

    None
}

fn classify_gmail_message(msg: &GmailMessageSummaryDto) -> (f32, String, bool, bool) {
    let from_lower = msg.from.to_ascii_lowercase();
    let subj_lower = msg.subject.to_ascii_lowercase();
    let snip_lower = msg.snippet.to_ascii_lowercase();

    let has_promo_label = msg.labels.iter().any(|l| {
        matches!(
            l.as_str(),
            "CATEGORY_PROMOTIONS" | "CATEGORY_SOCIAL" | "CATEGORY_UPDATES" | "SPAM"
        )
    });
    let is_newsletter = has_promo_label
        || from_lower.contains("noreply")
        || from_lower.contains("no-reply")
        || from_lower.contains("newsletter")
        || from_lower.contains("digest")
        || snip_lower.contains("unsubscribe")
        || subj_lower.contains("newsletter");

    if is_newsletter {
        return (0.15, "newsletter".to_string(), false, true);
    }

    let mut score: f32 = 0.45;
    let mut category = "general".to_string();
    let mut requires_reply = false;

    if msg
        .labels
        .iter()
        .any(|l| l == "IMPORTANT" || l == "STARRED")
    {
        score += 0.25;
    }

    if subj_lower.contains("urgent")
        || subj_lower.contains("asap")
        || subj_lower.contains("action required")
        || subj_lower.contains("deadline")
    {
        score += 0.30;
        category = "urgent".to_string();
        requires_reply = true;
    } else if subj_lower.contains("security")
        || subj_lower.contains("alert")
        || subj_lower.contains("verification")
    {
        score += 0.25;
        category = "security".to_string();
    } else if subj_lower.contains("invoice")
        || subj_lower.contains("payment")
        || subj_lower.contains("budget")
        || subj_lower.contains("fatura")
    {
        score += 0.20;
        category = "finance".to_string();
    }

    if subj_lower.contains('?')
        || snip_lower.contains("please review")
        || snip_lower.contains("let me know")
        || snip_lower.contains("can you")
        || subj_lower.starts_with("re:")
    {
        score += 0.15;
        if category == "general" {
            category = "work".to_string();
        }
        requires_reply = true;
    }

    (score.clamp(0.0, 1.0), category, requires_reply, false)
}

fn parse_calendar_event(val: &Value) -> CalendarEventDto {
    let id = val
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    let summary = val
        .get("summary")
        .and_then(|v| v.as_str())
        .unwrap_or("(Untitled Event)")
        .to_string();
    let start_rfc3339 = val
        .pointer("/start/dateTime")
        .or_else(|| val.pointer("/start/date"))
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    let end_rfc3339 = val
        .pointer("/end/dateTime")
        .or_else(|| val.pointer("/end/date"))
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    let location = val
        .get("location")
        .and_then(|v| v.as_str())
        .map(ToString::to_string);
    let attendees_count = val
        .get("attendees")
        .and_then(|v| v.as_array())
        .map_or(0, Vec::len);
    let status = val
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("confirmed")
        .to_string();

    CalendarEventDto {
        id,
        summary,
        start_rfc3339,
        end_rfc3339,
        location,
        attendees_count,
        status,
    }
}

fn focus_slot_label(start_utc: DateTime<Utc>, mins: u32) -> String {
    let hour = start_utc.format("%H").to_string().parse::<u32>().unwrap_or(12);
    let period = if hour < 12 {
        "Morning"
    } else if hour < 18 {
        "Afternoon"
    } else {
        "Evening"
    };
    format!("{period} Deep Focus ({mins}m)")
}

fn parse_drive_file(val: &Value) -> DriveFileDto {
    let id = val
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    let name = val
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    let mime_type = val
        .get("mimeType")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    let modified_time = val
        .get("modifiedTime")
        .and_then(|v| v.as_str())
        .map(ToString::to_string);
    let web_view_link = val
        .get("webViewLink")
        .and_then(|v| v.as_str())
        .map(ToString::to_string);
    let size_bytes = val.get("size").and_then(|v| {
        v.as_u64()
            .or_else(|| v.as_str().and_then(|s| s.parse::<u64>().ok()))
    });

    DriveFileDto {
        id,
        name,
        mime_type,
        modified_time,
        web_view_link,
        size_bytes,
    }
}

fn json_cell_to_string(v: &Value) -> String {
    match v {
        Value::Null => String::new(),
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        other => other.to_string(),
    }
}

fn parse_numeric_cell(raw: &str) -> Option<f64> {
    let cleaned = raw
        .trim()
        .replace("R$", "")
        .replace(['$', '€', '£', ' '], "");
    if cleaned.is_empty() {
        return None;
    }

    let normalized = match (cleaned.rfind(','), cleaned.rfind('.')) {
        (Some(c), Some(d)) => {
            if c > d {
                cleaned.replace('.', "").replace(',', ".")
            } else {
                cleaned.replace(',', "")
            }
        }
        (Some(c), None) => {
            let after = &cleaned[c + 1..];
            if after.len() <= 2 && after.chars().all(|ch| ch.is_ascii_digit()) {
                cleaned.replace(',', ".")
            } else {
                cleaned.replace(',', "")
            }
        }
        _ => cleaned,
    };

    normalized.parse::<f64>().ok().filter(|v| v.is_finite())
}

fn select_primary_numeric_col(headers: &[String], numeric_cols: &[usize]) -> Option<usize> {
    for &idx in numeric_cols {
        if let Some(h) = headers.get(idx) {
            let lower = h.to_ascii_lowercase();
            if lower.contains("amount")
                || lower.contains("value")
                || lower.contains("total")
                || lower.contains("valor")
                || lower.contains("cost")
                || lower.contains("price")
            {
                return Some(idx);
            }
        }
    }
    numeric_cols.first().copied()
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}
