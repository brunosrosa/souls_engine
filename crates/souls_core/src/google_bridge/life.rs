//! Life Hub personal productivity integrations:
//! - Google Tasks API + Obsidian Quick Capture (replacing Keep for personal `@gmail.com`)
//! - Google People/Contacts search + deterministic upcoming birthday detector
//! - Google Maps Routes API v2 (`TRAFFIC_AWARE` + `X-Goog-FieldMask`) + commute departure alerts
//! - YouTube Innertube transcript extraction + pure-Rust XML/SRV3 parser + disk cache

use super::oauth::percent_encode;
use super::security::sanitize_for_external_llm;
use super::session::{generate_entropy_bytes, write_bytes_atomic, write_json_atomic_lf};
use super::workspace::parse_json_response;
use crate::error::CoreError;
use chrono::{Datelike, DateTime, Duration as ChronoDuration, NaiveDate, Utc};
use serde_json::{json, Value};
use souls_protocol::google::{
    CommuteRouteAlertDto, ContactCardDto, GoogleAccountKind, GoogleTaskDto,
    YouTubeVideoSummaryDto,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

// ============================================================================
// 1. Google Tasks + Obsidian Quick Capture + Keep Enterprise Eligibility
// ============================================================================

/// REST client for Google Tasks API v1.
#[derive(Debug, Clone)]
pub struct GoogleTasksClient {
    http: reqwest::Client,
    base_url: String,
}

impl Default for GoogleTasksClient {
    fn default() -> Self {
        Self::new()
    }
}

impl GoogleTasksClient {
    /// Creates a Google Tasks client targeting production endpoints.
    pub fn new() -> Self {
        Self::with_base_url("https://tasks.googleapis.com/tasks/v1")
    }

    /// Creates a Google Tasks client with a custom base URL for mock testing.
    pub fn with_base_url(base_url: impl Into<String>) -> Self {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_default();
        Self {
            http,
            base_url: base_url.into().trim_end_matches('/').to_string(),
        }
    }

    /// Lists tasks in `tasklist_id` (defaults to `@default` when empty).
    pub async fn list_tasks(
        &self,
        access_token: &str,
        tasklist_id: &str,
        show_completed: bool,
    ) -> Result<Vec<GoogleTaskDto>, CoreError> {
        let list_id = if tasklist_id.trim().is_empty() {
            "@default"
        } else {
            tasklist_id.trim()
        };
        let url = format!(
            "{}/lists/{}/tasks?showCompleted={}",
            self.base_url,
            percent_encode(list_id),
            show_completed
        );

        let resp = self
            .http
            .get(&url)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| CoreError::SubprocessError(format!("Tasks list failed: {e}")))?;
        let val = parse_json_response(resp, &url).await?;
        let items = val
            .get("items")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        Ok(items
            .iter()
            .map(|item| parse_task_dto(list_id, item))
            .collect())
    }

    /// Creates a new task in `tasklist_id` (`Tier1SafeReversible`).
    pub async fn create_task(
        &self,
        access_token: &str,
        tasklist_id: &str,
        title: &str,
        notes: Option<&str>,
        due_rfc3339: Option<&str>,
    ) -> Result<GoogleTaskDto, CoreError> {
        let list_id = if tasklist_id.trim().is_empty() {
            "@default"
        } else {
            tasklist_id.trim()
        };
        let url = format!("{}/lists/{}/tasks", self.base_url, percent_encode(list_id));

        let mut payload = json!({ "title": title });
        if let Some(n) = notes.filter(|s| !s.trim().is_empty()) {
            payload["notes"] = json!(n);
        }
        if let Some(d) = due_rfc3339.filter(|s| !s.trim().is_empty()) {
            payload["due"] = json!(d);
        }

        let resp = self
            .http
            .post(&url)
            .bearer_auth(access_token)
            .json(&payload)
            .send()
            .await
            .map_err(|e| CoreError::SubprocessError(format!("Tasks create failed: {e}")))?;
        let val = parse_json_response(resp, &url).await?;
        Ok(parse_task_dto(list_id, &val))
    }

    /// Marks an existing task as `"completed"` (`Tier1SafeReversible`).
    pub async fn complete_task(
        &self,
        access_token: &str,
        tasklist_id: &str,
        task_id: &str,
    ) -> Result<GoogleTaskDto, CoreError> {
        let list_id = if tasklist_id.trim().is_empty() {
            "@default"
        } else {
            tasklist_id.trim()
        };
        let url = format!(
            "{}/lists/{}/tasks/{}",
            self.base_url,
            percent_encode(list_id),
            percent_encode(task_id)
        );

        let resp = self
            .http
            .patch(&url)
            .bearer_auth(access_token)
            .json(&json!({ "status": "completed" }))
            .send()
            .await
            .map_err(|e| CoreError::SubprocessError(format!("Tasks complete failed: {e}")))?;
        let val = parse_json_response(resp, &url).await?;
        Ok(parse_task_dto(list_id, &val))
    }
}

/// Writes a PII-redacted, pure-LF Markdown quick-capture note with YAML frontmatter
/// into `raw_inbox_dir` (Obsidian vault `raw/inbox/` fallback for Google Keep).
pub fn quick_capture_to_obsidian(
    raw_inbox_dir: &Path,
    title: &str,
    body: &str,
    tags: &[String],
) -> Result<PathBuf, CoreError> {
    let clean_title = sanitize_for_external_llm(title).replace("\r\n", "\n");
    let clean_body = sanitize_for_external_llm(body).replace("\r\n", "\n");

    let slug = sanitize_slug(&clean_title);
    let now = Utc::now();
    let suffix_bytes: [u8; 3] = generate_entropy_bytes();
    let suffix = format!("{:02x}{:02x}{:02x}", suffix_bytes[0], suffix_bytes[1], suffix_bytes[2]);
    let filename = format!("{}_{}_{}.md", now.format("%Y%m%d_%H%M%S"), slug, suffix);

    fs::create_dir_all(raw_inbox_dir).map_err(|e| {
        CoreError::SubprocessError(format!(
            "Failed to create Obsidian quick capture directory '{}': {e}",
            raw_inbox_dir.display()
        ))
    })?;

    let target_path = raw_inbox_dir.join(&filename);
    let mut clean_tags: Vec<String> = tags
        .iter()
        .map(|t| t.trim().trim_start_matches('#').to_string())
        .filter(|t| !t.is_empty())
        .collect();
    if !clean_tags.iter().any(|t| t == "quick-capture") {
        clean_tags.push("quick-capture".to_string());
    }

    let yaml_title = clean_title.replace('"', "'").replace('\n', " ");
    let yaml_tags = clean_tags
        .iter()
        .map(|t| format!("  - {}", t.replace(['\r', '\n'], " ")))
        .collect::<Vec<_>>()
        .join("\n");

    let markdown = format!(
        "---\ntitle: \"{yaml_title}\"\ncreated_at: \"{}\"\nsource: \"google_quick_capture\"\ntags:\n{yaml_tags}\n---\n\n# {yaml_title}\n\n{clean_body}\n",
        now.to_rfc3339()
    );

    write_bytes_atomic(&target_path, markdown.as_bytes())?;
    Ok(target_path)
}

/// Validates whether a Google account kind can call the enterprise Google Keep API.
/// Returns a clear actionable error for `@gmail.com` personal accounts directing to
/// Google Tasks + Obsidian Quick Capture.
pub fn check_keep_eligibility(kind: GoogleAccountKind) -> Result<(), CoreError> {
    match kind {
        GoogleAccountKind::WorkspaceEnterprise => Ok(()),
        GoogleAccountKind::PersonalGmail | GoogleAccountKind::SecondarySandbox => {
            Err(CoreError::SubprocessError(
                "Google Keep API is restricted to Google Workspace Enterprise accounts with Domain-Wide Delegation. For personal @gmail.com or sandbox accounts, use Google Tasks (`google_tasks`) and Obsidian Quick Capture (`quick_capture_to_obsidian`) instead."
                    .to_string(),
            ))
        }
    }
}

fn sanitize_slug(raw: &str) -> String {
    let mut out = String::new();
    let mut prev_dash = false;
    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            prev_dash = false;
        } else if !prev_dash && !out.is_empty() {
            out.push('-');
            prev_dash = true;
        }
        if out.len() >= 48 {
            break;
        }
    }
    let trimmed = out.trim_matches('-').to_string();
    if trimmed.is_empty() {
        "capture-note".to_string()
    } else {
        trimmed
    }
}

fn parse_task_dto(list_id: &str, val: &Value) -> GoogleTaskDto {
    GoogleTaskDto {
        id: val
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string(),
        tasklist_id: list_id.to_string(),
        title: val
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string(),
        notes: val
            .get("notes")
            .and_then(|v| v.as_str())
            .map(ToString::to_string),
        due_rfc3339: val
            .get("due")
            .and_then(|v| v.as_str())
            .map(ToString::to_string),
        status: val
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("needsAction")
            .to_string(),
        updated_at: val
            .get("updated")
            .and_then(|v| v.as_str())
            .map(ToString::to_string),
    }
}

// ============================================================================
// 2. Google Contacts (People API)
// ============================================================================

/// REST client for Google People (Contacts) API v1.
#[derive(Debug, Clone)]
pub struct PeopleClient {
    http: reqwest::Client,
    base_url: String,
}

impl Default for PeopleClient {
    fn default() -> Self {
        Self::new()
    }
}

impl PeopleClient {
    /// Creates a People client targeting production endpoints.
    pub fn new() -> Self {
        Self::with_base_url("https://people.googleapis.com/v1")
    }

    /// Creates a People client with a custom base URL for mock testing.
    pub fn with_base_url(base_url: impl Into<String>) -> Self {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_default();
        Self {
            http,
            base_url: base_url.into().trim_end_matches('/').to_string(),
        }
    }

    /// Searches contacts matching `query` via `people:searchContacts`.
    pub async fn search_contacts(
        &self,
        access_token: &str,
        query: &str,
        page_size: usize,
    ) -> Result<Vec<ContactCardDto>, CoreError> {
        let limit = page_size.clamp(1, 30);
        let url = format!(
            "{}/people:searchContacts?query={}&pageSize={}&readMask=names,emailAddresses,phoneNumbers,organizations,birthdays",
            self.base_url,
            percent_encode(query),
            limit
        );

        let resp = self
            .http
            .get(&url)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| CoreError::SubprocessError(format!("People searchContacts failed: {e}")))?;
        let val = parse_json_response(resp, &url).await?;

        let results = val
            .get("results")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        Ok(results
            .iter()
            .filter_map(|item| item.get("person").map(parse_contact_card))
            .collect())
    }

    /// Filters contacts whose birthday falls within `0..=within_days` from `(today_month, today_day)`.
    pub fn upcoming_birthdays(
        contacts: &[ContactCardDto],
        today_month: u32,
        today_day: u32,
        within_days: u32,
    ) -> Vec<ContactCardDto> {
        upcoming_birthdays(contacts, today_month, today_day, within_days)
    }
}

/// Filters contacts whose birthday falls within `0..=within_days` from `(today_month, today_day)`.
pub fn upcoming_birthdays(
    contacts: &[ContactCardDto],
    today_month: u32,
    today_day: u32,
    within_days: u32,
) -> Vec<ContactCardDto> {
    let Some(today) = NaiveDate::from_ymd_opt(2026, today_month, today_day)
        .or_else(|| NaiveDate::from_ymd_opt(2024, today_month, today_day))
    else {
        return Vec::new();
    };

    let mut matched: Vec<(i64, ContactCardDto)> = Vec::new();
    for contact in contacts {
        let Some(ref bday) = contact.birthday else {
            continue;
        };
        let Some((bm, bd)) = parse_birthday_month_day(bday) else {
            continue;
        };

        let same_year = NaiveDate::from_ymd_opt(today.year(), bm, bd).or_else(|| {
            if bm == 2 && bd == 29 {
                NaiveDate::from_ymd_opt(today.year(), 2, 28)
            } else {
                None
            }
        });
        let Some(mut target) = same_year else {
            continue;
        };
        if target < today {
            let next_yr = today.year() + 1;
            target = NaiveDate::from_ymd_opt(next_yr, bm, bd).unwrap_or_else(|| {
                NaiveDate::from_ymd_opt(next_yr, 2, 28).unwrap_or(target)
            });
        }

        let delta = (target - today).num_days();
        if delta >= 0 && delta <= within_days as i64 {
            matched.push((delta, contact.clone()));
        }
    }

    matched.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.display_name.cmp(&b.1.display_name)));
    matched.into_iter().map(|(_, c)| c).collect()
}

fn parse_birthday_month_day(raw: &str) -> Option<(u32, u32)> {
    let clean = raw.trim().trim_start_matches("--");
    let parts: Vec<&str> = clean.split('-').collect();
    match parts.as_slice() {
        [_year, m, d] => Some((m.parse().ok()?, d.parse().ok()?)),
        [m, d] => Some((m.parse().ok()?, d.parse().ok()?)),
        _ => None,
    }
}

fn parse_contact_card(person: &Value) -> ContactCardDto {
    let resource_name = person
        .get("resourceName")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();

    let display_name = person
        .pointer("/names/0/displayName")
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown Contact")
        .to_string();

    let emails = person
        .get("emailAddresses")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|e| e.get("value").and_then(|v| v.as_str()).map(ToString::to_string))
                .collect()
        })
        .unwrap_or_default();

    let phones = person
        .get("phoneNumbers")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|p| p.get("value").and_then(|v| v.as_str()).map(ToString::to_string))
                .collect()
        })
        .unwrap_or_default();

    let organization = person
        .pointer("/organizations/0/name")
        .and_then(|v| v.as_str())
        .map(ToString::to_string);

    let birthday = person.pointer("/birthdays/0/date").and_then(|d| {
        let m = d.get("month").and_then(|v| v.as_u64())?;
        let day = d.get("day").and_then(|v| v.as_u64())?;
        if let Some(y) = d.get("year").and_then(|v| v.as_u64()) {
            Some(format!("{y:04}-{m:02}-{day:02}"))
        } else {
            Some(format!("--{m:02}-{day:02}"))
        }
    });

    ContactCardDto {
        resource_name,
        display_name,
        emails,
        phones,
        organization,
        birthday,
    }
}

// ============================================================================
// 3. Google Maps Routes API v2
// ============================================================================

/// Result of a Maps Routes API v2 `computeRoutes` calculation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteComputation {
    pub duration_seconds: u64,
    pub static_duration_seconds: u64,
    pub distance_meters: u64,
    pub localized_duration: Option<String>,
    pub localized_distance: Option<String>,
}

/// REST client for Google Maps Routes API v2 (`TRAFFIC_AWARE`).
#[derive(Debug, Clone)]
pub struct RoutesV2Client {
    http: reqwest::Client,
    base_url: String,
}

impl Default for RoutesV2Client {
    fn default() -> Self {
        Self::new()
    }
}

impl RoutesV2Client {
    /// Creates a Routes v2 client targeting `https://routes.googleapis.com`.
    pub fn new() -> Self {
        Self::with_base_url("https://routes.googleapis.com")
    }

    /// Creates a Routes v2 client with an overridden base URL for mock testing.
    pub fn with_base_url(base_url: impl Into<String>) -> Self {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_default();
        Self {
            http,
            base_url: base_url.into().trim_end_matches('/').to_string(),
        }
    }

    /// Computes a traffic-aware driving route using Google Routes API v2 and `X-Goog-FieldMask`.
    pub async fn compute_route(
        &self,
        api_key: &str,
        origin: &str,
        destination: &str,
    ) -> Result<RouteComputation, CoreError> {
        let url = format!("{}/directions/v2:computeRoutes", self.base_url);
        let body = json!({
            "origin": { "address": origin },
            "destination": { "address": destination },
            "travelMode": "DRIVE",
            "routingPreference": "TRAFFIC_AWARE"
        });

        let resp = self
            .http
            .post(&url)
            .header("X-Goog-Api-Key", api_key)
            .header(
                "X-Goog-FieldMask",
                "routes.duration,routes.staticDuration,routes.distanceMeters,routes.localizedValues",
            )
            .json(&body)
            .send()
            .await
            .map_err(|e| CoreError::SubprocessError(format!("Routes v2 request failed: {e}")))?;

        let val = parse_json_response(resp, &url).await?;
        let route = val
            .pointer("/routes/0")
            .ok_or_else(|| CoreError::SubprocessError("Routes v2 returned no routes".to_string()))?;

        let duration_seconds = route
            .get("duration")
            .and_then(|v| v.as_str())
            .and_then(parse_google_duration_secs)
            .ok_or_else(|| {
                CoreError::SubprocessError("Routes v2 missing valid routes[0].duration".to_string())
            })?;

        let static_duration_seconds = route
            .get("staticDuration")
            .and_then(|v| v.as_str())
            .and_then(parse_google_duration_secs)
            .unwrap_or(duration_seconds);

        let distance_meters = route
            .get("distanceMeters")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let localized_duration = route
            .pointer("/localizedValues/duration/text")
            .and_then(|v| v.as_str())
            .map(ToString::to_string);
        let localized_distance = route
            .pointer("/localizedValues/distance/text")
            .and_then(|v| v.as_str())
            .map(ToString::to_string);

        Ok(RouteComputation {
            duration_seconds,
            static_duration_seconds,
            distance_meters,
            localized_duration,
            localized_distance,
        })
    }

    /// Computes a proactive departure alert for an upcoming calendar event.
    #[allow(clippy::too_many_arguments)]
    pub fn compute_commute_alert(
        event_id: &str,
        event_title: &str,
        event_start_rfc3339: &str,
        destination: &str,
        duration_secs: u64,
        static_duration_secs: u64,
        distance_meters: u64,
        buffer_mins: u32,
    ) -> Result<CommuteRouteAlertDto, CoreError> {
        compute_commute_alert(
            event_id,
            event_title,
            event_start_rfc3339,
            destination,
            duration_secs,
            static_duration_secs,
            distance_meters,
            buffer_mins,
        )
    }
}

/// Computes a proactive departure alert (`CommuteRouteAlertDto`) for a geolocated event.
#[allow(clippy::too_many_arguments)]
pub fn compute_commute_alert(
    event_id: &str,
    event_title: &str,
    event_start_rfc3339: &str,
    destination: &str,
    duration_secs: u64,
    static_duration_secs: u64,
    distance_meters: u64,
    buffer_mins: u32,
) -> Result<CommuteRouteAlertDto, CoreError> {
    let start_dt = DateTime::parse_from_rfc3339(event_start_rfc3339).map_err(|e| {
        CoreError::SubprocessError(format!(
            "Invalid event_start_rfc3339 '{event_start_rfc3339}': {e}"
        ))
    })?;

    let lead_secs = duration_secs.saturating_add((buffer_mins as u64).saturating_mul(60));
    let suggested_departure = start_dt - ChronoDuration::seconds(lead_secs as i64);
    let traffic_delay_seconds = duration_secs as i64 - static_duration_secs as i64;
    let eta_mins = duration_secs.div_ceil(60);
    let delay_mins = (traffic_delay_seconds.max(0) as u64).div_ceil(60);

    let advisory = if traffic_delay_seconds > 180 {
        format!(
            "Traffic delay +{delay_mins}m (ETA {eta_mins}m + {buffer_mins}m buffer): leave by {}.",
            suggested_departure.to_rfc3339()
        )
    } else {
        format!(
            "Normal traffic (ETA {eta_mins}m + {buffer_mins}m buffer): leave by {}.",
            suggested_departure.to_rfc3339()
        )
    };

    Ok(CommuteRouteAlertDto {
        event_id: event_id.to_string(),
        event_title: event_title.to_string(),
        event_start_rfc3339: event_start_rfc3339.to_string(),
        destination: destination.to_string(),
        duration_seconds: duration_secs,
        static_duration_seconds: static_duration_secs,
        traffic_delay_seconds,
        distance_meters,
        suggested_departure_rfc3339: suggested_departure.to_rfc3339(),
        advisory,
    })
}

fn parse_google_duration_secs(raw: &str) -> Option<u64> {
    let trimmed = raw.trim().trim_end_matches('s');
    trimmed
        .parse::<f64>()
        .ok()
        .filter(|v| v.is_finite() && *v >= 0.0)
        .map(|v| v.round() as u64)
}

// ============================================================================
// 4. YouTube Transcript Extractor + Cache
// ============================================================================

/// Zero-dependency YouTube Innertube transcript fetcher with local disk cache.
#[derive(Debug, Clone)]
pub struct YouTubeClient {
    http: reqwest::Client,
    base_url: String,
}

impl Default for YouTubeClient {
    fn default() -> Self {
        Self::new()
    }
}

impl YouTubeClient {
    /// Creates a YouTube client targeting `https://www.youtube.com`.
    pub fn new() -> Self {
        Self::with_base_url("https://www.youtube.com")
    }

    /// Creates a YouTube client with an overridden base URL for mock testing.
    pub fn with_base_url(base_url: impl Into<String>) -> Self {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_default();
        Self {
            http,
            base_url: base_url.into().trim_end_matches('/').to_string(),
        }
    }

    /// Extracts and validates an 11-character YouTube video ID from a URL or raw ID.
    pub fn extract_video_id(url_or_id: &str) -> Result<String, CoreError> {
        extract_video_id(url_or_id)
    }

    /// Parses YouTube timedtext XML or SRV3 captions into plain text and segment count.
    pub fn parse_captions_xml(xml_or_srv3: &str) -> Result<(String, usize), CoreError> {
        parse_captions_xml(xml_or_srv3)
    }

    /// Fetches a YouTube transcript via Innertube (`ANDROID` client) and caches it under
    /// `<data_dir>/youtube_cache/{video_id}.json`.
    pub async fn fetch_transcript_with_cache(
        &self,
        data_dir: &Path,
        url_or_id: &str,
        preferred_lang: &str,
    ) -> Result<YouTubeVideoSummaryDto, CoreError> {
        let video_id = extract_video_id(url_or_id)?;
        let cache_dir = data_dir.join("youtube_cache");
        let cache_path = cache_dir.join(format!("{video_id}.json"));

        if cache_path.exists() {
            if let Ok(raw) = fs::read_to_string(&cache_path) {
                if let Ok(mut cached_dto) = serde_json::from_str::<YouTubeVideoSummaryDto>(&raw) {
                    cached_dto.cached = true;
                    return Ok(cached_dto);
                }
            }
        }

        let player_url = format!("{}/youtubei/v1/player?prettyPrint=false", self.base_url);
        let player_req = json!({
            "context": {
                "client": {
                    "clientName": "ANDROID",
                    "clientVersion": "20.10.38"
                }
            },
            "videoId": video_id
        });

        let resp = self
            .http
            .post(&player_url)
            .header("Content-Type", "application/json")
            .json(&player_req)
            .send()
            .await
            .map_err(|e| CoreError::WebExtractionError(format!("Innertube player failed: {e}")))?;

        let player_val = parse_json_response(resp, &player_url).await?;
        let title = player_val
            .pointer("/videoDetails/title")
            .and_then(|v| v.as_str())
            .map(ToString::to_string);
        let channel = player_val
            .pointer("/videoDetails/author")
            .and_then(|v| v.as_str())
            .map(ToString::to_string);

        let tracks = player_val
            .pointer("/captions/playerCaptionsTracklistRenderer/captionTracks")
            .and_then(|v| v.as_array())
            .ok_or_else(|| {
                CoreError::WebExtractionError(format!(
                    "No captionTracks returned by Innertube for video '{video_id}'"
                ))
            })?;

        let selected_track = select_caption_track(tracks, preferred_lang).ok_or_else(|| {
            CoreError::WebExtractionError(format!(
                "Empty captionTracks list for video '{video_id}'"
            ))
        })?;

        let track_lang = selected_track
            .get("languageCode")
            .and_then(|v| v.as_str())
            .unwrap_or(preferred_lang)
            .to_string();

        let raw_base_url = selected_track
            .get("baseUrl")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                CoreError::WebExtractionError("Caption track missing baseUrl".to_string())
            })?;

        let captions_url = if raw_base_url.starts_with('/') {
            format!("{}{}", self.base_url, raw_base_url)
        } else {
            raw_base_url.to_string()
        };

        let xml_resp = self
            .http
            .get(&captions_url)
            .send()
            .await
            .map_err(|e| {
                CoreError::WebExtractionError(format!("Failed fetching caption track: {e}"))
            })?;
        let xml_status = xml_resp.status();
        let xml_body = xml_resp.text().await.map_err(|e| {
            CoreError::WebExtractionError(format!("Failed reading caption XML: {e}"))
        })?;
        if !xml_status.is_success() {
            return Err(CoreError::WebExtractionError(format!(
                "Caption endpoint returned HTTP {xml_status}: {xml_body}"
            )));
        }

        let (transcript_text, segment_count) = parse_captions_xml(&xml_body)?;
        let summary = YouTubeVideoSummaryDto {
            video_id: video_id.clone(),
            url: format!("https://www.youtube.com/watch?v={video_id}"),
            title,
            channel,
            language: track_lang,
            transcript_text,
            segment_count,
            cached: false,
        };

        fs::create_dir_all(&cache_dir).map_err(CoreError::Io)?;
        write_json_atomic_lf(&cache_path, &summary)?;
        Ok(summary)
    }
}

/// Extracts and validates an 11-character YouTube video ID from a URL or raw ID.
pub fn extract_video_id(url_or_id: &str) -> Result<String, CoreError> {
    let input = url_or_id.trim();
    if is_valid_yt_id(input) {
        return Ok(input.to_string());
    }

    if let Some(idx) = input.find("v=") {
        let rest = &input[idx + 2..];
        let candidate: String = rest
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
            .collect();
        if is_valid_yt_id(&candidate) {
            return Ok(candidate);
        }
    }

    for marker in ["youtu.be/", "/shorts/", "/embed/", "/live/", "/v/"] {
        if let Some(idx) = input.find(marker) {
            let rest = &input[idx + marker.len()..];
            let candidate: String = rest
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
                .collect();
            if is_valid_yt_id(&candidate) {
                return Ok(candidate);
            }
        }
    }

    Err(CoreError::WebExtractionError(format!(
        "Could not extract a valid 11-char YouTube video ID from '{url_or_id}'"
    )))
}

fn is_valid_yt_id(s: &str) -> bool {
    s.len() == 11
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}

fn select_caption_track<'a>(tracks: &'a [Value], preferred_lang: &str) -> Option<&'a Value> {
    let pref = preferred_lang.trim().to_ascii_lowercase();
    if !pref.is_empty() {
        if let Some(exact) = tracks.iter().find(|t| {
            t.get("languageCode")
                .and_then(|v| v.as_str())
                .is_some_and(|l| l.eq_ignore_ascii_case(&pref))
        }) {
            return Some(exact);
        }
        let base_pref = pref.split('-').next().unwrap_or(&pref);
        if let Some(prefix) = tracks.iter().find(|t| {
            t.get("languageCode")
                .and_then(|v| v.as_str())
                .is_some_and(|l| l.to_ascii_lowercase().starts_with(base_pref))
        }) {
            return Some(prefix);
        }
    }

    tracks
        .iter()
        .find(|t| {
            t.get("languageCode")
                .and_then(|v| v.as_str())
                .is_some_and(|l| l.to_ascii_lowercase().starts_with("en"))
        })
        .or_else(|| tracks.first())
}

/// Pure-Rust zero-dependency parser for YouTube timedtext XML (`<text>`) and SRV3 (`<p>` / `<s>`).
pub fn parse_captions_xml(xml_or_srv3: &str) -> Result<(String, usize), CoreError> {
    let mut segments = Vec::new();

    // Try SRV3 <p ...>...</p> first if present, otherwise <text ...>...</text>
    let tag_pairs = if xml_or_srv3.contains("<p ") || xml_or_srv3.contains("<p>") {
        extract_xml_tag_bodies(xml_or_srv3, "p")
    } else {
        extract_xml_tag_bodies(xml_or_srv3, "text")
    };

    for raw_inner in tag_pairs {
        let stripped = strip_inner_xml_tags(&raw_inner);
        let decoded = decode_xml_entities(&stripped);
        let collapsed: String = decoded.split_whitespace().collect::<Vec<_>>().join(" ");
        if !collapsed.is_empty() {
            segments.push(collapsed);
        }
    }

    if segments.is_empty() {
        return Err(CoreError::WebExtractionError(
            "No caption segments found in YouTube XML/SRV3 payload".to_string(),
        ));
    }

    let count = segments.len();
    Ok((segments.join("\n"), count))
}

fn extract_xml_tag_bodies(xml: &str, tag: &str) -> Vec<String> {
    let mut bodies = Vec::new();
    let open_prefix = format!("<{tag}");
    let close_tag = format!("</{tag}>");
    let mut cursor = 0usize;

    while let Some(rel_start) = xml[cursor..].find(&open_prefix) {
        let abs_start = cursor + rel_start;
        let after_prefix = abs_start + open_prefix.len();
        let Some(next_byte) = xml.as_bytes().get(after_prefix).copied() else {
            break;
        };
        if next_byte != b'>' && !next_byte.is_ascii_whitespace() {
            cursor = after_prefix;
            continue;
        }

        let Some(rel_gt) = xml[after_prefix..].find('>') else {
            break;
        };
        let gt_pos = after_prefix + rel_gt;
        if gt_pos > 0 && xml.as_bytes()[gt_pos - 1] == b'/' {
            cursor = gt_pos + 1;
            continue;
        }

        let body_start = gt_pos + 1;
        let Some(rel_end) = xml[body_start..].find(&close_tag) else {
            break;
        };
        let body_end = body_start + rel_end;
        bodies.push(xml[body_start..body_end].to_string());
        cursor = body_end + close_tag.len();
    }

    bodies
}

fn strip_inner_xml_tags(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut in_tag = false;
    for ch in raw.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    out
}

/// Decodes standard XML entities (`&amp;`, `&lt;`, `&gt;`, `&quot;`, `&#39;`, `&apos;`,
/// plus decimal `&#NNN;` and hex `&#xHHH;` character references).
pub fn decode_xml_entities(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut i = 0usize;
    let bytes = input.as_bytes();

    while i < bytes.len() {
        if bytes[i] == b'&' {
            if let Some(rel_semi) = input[i..].find(';') {
                if rel_semi <= 12 {
                    let entity = &input[i + 1..i + rel_semi];
                    let decoded = match entity {
                        "amp" => Some('&'),
                        "lt" => Some('<'),
                        "gt" => Some('>'),
                        "quot" => Some('"'),
                        "apos" => Some('\''),
                        _ if entity.starts_with("#x") || entity.starts_with("#X") => {
                            u32::from_str_radix(&entity[2..], 16)
                                .ok()
                                .and_then(char::from_u32)
                        }
                        _ if entity.starts_with('#') => entity[1..]
                            .parse::<u32>()
                            .ok()
                            .and_then(char::from_u32),
                        _ => None,
                    };
                    if let Some(ch) = decoded {
                        out.push(ch);
                        i += rel_semi + 1;
                        continue;
                    }
                }
            }
        }
        let ch_len = input[i..].chars().next().map_or(1, char::len_utf8);
        out.push_str(&input[i..i + ch_len]);
        i += ch_len;
    }

    out
}
