//! Google Forms API v1 client + deterministic response analyzer,
//! and Google Chat API v1 client (Workspace Enterprise gated) + incoming webhook dispatcher.

use super::oauth::percent_encode;
use super::workspace::parse_json_response;
use crate::error::CoreError;
use serde_json::{json, Value};
use souls_protocol::google::{
    GoogleAccountKind, GoogleChatMessageDto, GoogleChatSpaceDto, GoogleFormOptionCountDto,
    GoogleFormQuestionDto, GoogleFormQuestionStatsDto, GoogleFormResponseDto, GoogleFormStatsDto,
    GoogleFormSummaryDto,
};
use std::collections::{BTreeMap, HashMap};
use std::time::Duration;

// ============================================================================
// 1. Google Forms API v1
// ============================================================================

/// REST client for Google Forms API v1 (`forms.googleapis.com/v1`).
#[derive(Debug, Clone)]
pub struct GoogleFormsClient {
    http: reqwest::Client,
    base_url: String,
}

impl Default for GoogleFormsClient {
    fn default() -> Self {
        Self::new()
    }
}

impl GoogleFormsClient {
    /// Creates a Forms client targeting production endpoints.
    pub fn new() -> Self {
        Self::with_base_url("https://forms.googleapis.com/v1")
    }

    /// Creates a Forms client with a custom base URL for offline `TcpListener` mocks.
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

    /// Fetches metadata and question schema for a Google Form (`GET /forms/{formId}`).
    pub async fn get_form(
        &self,
        token: &str,
        form_id: &str,
    ) -> Result<GoogleFormSummaryDto, CoreError> {
        let url = format!("{}/forms/{}", self.base_url, percent_encode(form_id));
        let val = self.get_json(&url, token).await?;
        Ok(parse_form_summary(&val))
    }

    /// Creates a new Google Form (`POST /forms`).
    pub async fn create_form(
        &self,
        token: &str,
        title: &str,
        document_title: Option<&str>,
    ) -> Result<GoogleFormSummaryDto, CoreError> {
        let url = format!("{}/forms", self.base_url);
        let mut info = json!({ "title": title });
        if let Some(doc_title) = document_title.filter(|s| !s.trim().is_empty()) {
            info["documentTitle"] = json!(doc_title.trim());
        }
        let val = self.post_json(&url, token, &json!({ "info": info })).await?;
        Ok(parse_form_summary(&val))
    }

    /// Applies schema/question updates to a Google Form (`POST /forms/{formId}:batchUpdate`).
    pub async fn batch_update_form(
        &self,
        token: &str,
        form_id: &str,
        requests_json: Value,
    ) -> Result<Value, CoreError> {
        let url = format!("{}/forms/{}:batchUpdate", self.base_url, percent_encode(form_id));
        let body = if requests_json.get("requests").is_some() {
            requests_json
        } else {
            json!({ "requests": requests_json })
        };
        self.post_json(&url, token, &body).await
    }

    /// Lists submitted responses for a Google Form (`GET /forms/{formId}/responses`).
    pub async fn list_responses(
        &self,
        token: &str,
        form_id: &str,
    ) -> Result<Vec<GoogleFormResponseDto>, CoreError> {
        let url = format!("{}/forms/{}/responses", self.base_url, percent_encode(form_id));
        let val = self.get_json(&url, token).await?;
        let arr = val
            .get("responses")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        Ok(arr.iter().map(parse_form_response).collect())
    }

    async fn get_json(&self, url: &str, token: &str) -> Result<Value, CoreError> {
        let resp = self
            .http
            .get(url)
            .bearer_auth(token)
            .send()
            .await
            .map_err(|e| CoreError::SubprocessError(format!("Forms GET '{url}': {e}")))?;
        parse_json_response(resp, url).await
    }

    async fn post_json(&self, url: &str, token: &str, body: &Value) -> Result<Value, CoreError> {
        let resp = self
            .http
            .post(url)
            .bearer_auth(token)
            .json(body)
            .send()
            .await
            .map_err(|e| CoreError::SubprocessError(format!("Forms POST '{url}': {e}")))?;
        parse_json_response(resp, url).await
    }
}

/// Computes deterministic per-question counts, choice percentages, up to 5 sample text answers,
/// and a pure-LF Markdown summary ready for Obsidian or NotebookLM export.
pub fn compute_form_stats(
    form: &GoogleFormSummaryDto,
    responses: &[GoogleFormResponseDto],
) -> GoogleFormStatsDto {
    let latest_submission_at = responses
        .iter()
        .map(|r| {
            if r.last_submitted_time.is_empty() {
                r.create_time.as_str()
            } else {
                r.last_submitted_time.as_str()
            }
        })
        .filter(|s| !s.is_empty())
        .max()
        .map(ToString::to_string);

    let mut question_stats = Vec::with_capacity(form.questions.len());

    for q in &form.questions {
        let mut collected_values: Vec<String> = Vec::new();
        for resp in responses {
            for (k, v) in &resp.answers {
                if k == &q.item_id || k.eq_ignore_ascii_case(&q.title) {
                    let trimmed = v.trim();
                    if !trimmed.is_empty() {
                        collected_values.push(trimmed.to_string());
                    }
                }
            }
        }

        let total_answers = collected_values.len();
        let mut counts: BTreeMap<String, usize> = BTreeMap::new();
        for opt in &q.options {
            counts.entry(opt.clone()).or_insert(0);
        }

        let is_choice = !q.options.is_empty()
            || q.question_type.contains("CHOICE")
            || q.question_type.contains("RADIO")
            || q.question_type.contains("CHECKBOX")
            || q.question_type.contains("DROP_DOWN")
            || q.question_type.contains("SCALE");

        let mut sample_text_answers = Vec::new();
        for val in &collected_values {
            if is_choice {
                *counts.entry(val.clone()).or_insert(0) += 1;
            }
            if sample_text_answers.len() < 5 && !sample_text_answers.contains(val) {
                sample_text_answers.push(val.clone());
            }
        }

        let mut option_counts: Vec<GoogleFormOptionCountDto> = if is_choice {
            counts
                .into_iter()
                .map(|(option, count)| {
                    let percentage = if total_answers > 0 {
                        ((count as f32 / total_answers as f32) * 1000.0).round() / 10.0
                    } else {
                        0.0
                    };
                    GoogleFormOptionCountDto {
                        option,
                        count,
                        percentage,
                    }
                })
                .collect()
        } else {
            Vec::new()
        };

        option_counts.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.option.cmp(&b.option)));

        question_stats.push(GoogleFormQuestionStatsDto {
            question_title: q.title.clone(),
            total_answers,
            option_counts,
            sample_text_answers,
        });
    }

    let mut md = String::new();
    md.push_str(&format!("# {} (Google Forms)\n\n", form.title));
    md.push_str(&format!("- **Form ID:** `{}`\n", form.form_id));
    md.push_str(&format!("- **Total de respostas:** {}\n", responses.len()));
    if let Some(ref latest) = latest_submission_at {
        md.push_str(&format!("- **Última resposta:** {}\n", latest));
    }
    if let Some(ref uri) = form.responder_uri {
        md.push_str(&format!("- **Link público:** {}\n", uri));
    }
    md.push('\n');

    for qs in &question_stats {
        md.push_str(&format!("## {}\n", qs.question_title));
        md.push_str(&format!("- Respostas válidas: {}\n", qs.total_answers));
        if !qs.option_counts.is_empty() {
            for oc in &qs.option_counts {
                md.push_str(&format!(
                    "  - **{}**: {} ({:.1}%)\n",
                    oc.option, oc.count, oc.percentage
                ));
            }
        } else if !qs.sample_text_answers.is_empty() {
            md.push_str("  - Amostras de texto:\n");
            for sample in &qs.sample_text_answers {
                md.push_str(&format!("    - \"{}\"\n", sample.replace('\n', " ")));
            }
        }
        md.push('\n');
    }

    GoogleFormStatsDto {
        form_id: form.form_id.clone(),
        form_title: form.title.clone(),
        total_responses: responses.len(),
        latest_submission_at,
        question_stats,
        markdown_summary: md.replace("\r\n", "\n"),
    }
}

// ============================================================================
// 2. Google Chat API v1 (Workspace Enterprise Only)
// ============================================================================

/// Enforces Workspace Enterprise eligibility before calling the Google Chat API.
pub fn check_chat_eligibility(kind: GoogleAccountKind) -> Result<(), CoreError> {
    match kind {
        GoogleAccountKind::WorkspaceEnterprise => Ok(()),
        GoogleAccountKind::PersonalGmail | GoogleAccountKind::SecondarySandbox => {
            Err(CoreError::SubprocessError(
                "Google Chat API and Spaces are restricted exclusively to Google Workspace Enterprise accounts. Personal @gmail.com and sandbox accounts do not have Google Chat API access."
                    .to_string(),
            ))
        }
    }
}

/// REST client for Google Chat API v1 (`chat.googleapis.com/v1`).
#[derive(Debug, Clone)]
pub struct GoogleChatClient {
    http: reqwest::Client,
    base_url: String,
}

impl Default for GoogleChatClient {
    fn default() -> Self {
        Self::new()
    }
}

impl GoogleChatClient {
    /// Creates a Google Chat client targeting production endpoints.
    pub fn new() -> Self {
        Self::with_base_url("https://chat.googleapis.com/v1")
    }

    /// Creates a Google Chat client with a custom base URL for offline mocks.
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

    /// Lists Google Chat spaces accessible to the authenticated Workspace account (`GET /spaces`).
    pub async fn list_spaces(&self, token: &str) -> Result<Vec<GoogleChatSpaceDto>, CoreError> {
        let url = format!("{}/spaces", self.base_url);
        let val = self.get_json(&url, token).await?;
        let arr = val
            .get("spaces")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        Ok(arr.iter().map(parse_chat_space).collect())
    }

    /// Lists recent messages in a Google Chat space (`GET /{space_name}/messages`).
    pub async fn list_messages(
        &self,
        token: &str,
        space_name: &str,
        page_size: usize,
    ) -> Result<Vec<GoogleChatMessageDto>, CoreError> {
        let space = normalize_space_name(space_name)?;
        let limit = page_size.clamp(1, 100);
        let url = format!("{}/{}/messages?pageSize={}", self.base_url, space, limit);
        let val = self.get_json(&url, token).await?;
        let arr = val
            .get("messages")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        Ok(arr.iter().map(parse_chat_message).collect())
    }

    /// Posts a message into a Google Chat space (`POST /{space_name}/messages`).
    pub async fn create_message(
        &self,
        token: &str,
        space_name: &str,
        text: &str,
    ) -> Result<GoogleChatMessageDto, CoreError> {
        let space = normalize_space_name(space_name)?;
        let url = format!("{}/{}/messages", self.base_url, space);
        let val = self.post_json(&url, token, &json!({ "text": text })).await?;
        Ok(parse_chat_message(&val))
    }

    /// Sends a message to a verified Google Chat incoming webhook URL
    /// (`https://chat.googleapis.com/v1/spaces/...`).
    pub async fn send_incoming_webhook(
        webhook_url: &str,
        text: &str,
    ) -> Result<Value, CoreError> {
        Self::send_incoming_webhook_with_allow_local(webhook_url, text, false).await
    }

    /// Sends a message to an incoming webhook URL, optionally permitting `http://127.0.0.1`
    /// or `http://localhost` during offline `TcpListener` tests.
    pub async fn send_incoming_webhook_with_allow_local(
        webhook_url: &str,
        text: &str,
        allow_test_http: bool,
    ) -> Result<Value, CoreError> {
        let trimmed = webhook_url.trim();
        let is_google_chat = trimmed.starts_with("https://chat.googleapis.com/v1/spaces/");
        let is_local_test = allow_test_http
            && (trimmed.starts_with("http://127.0.0.1:")
                || trimmed.starts_with("http://localhost:"));

        if !is_google_chat && !is_local_test {
            return Err(CoreError::DangerousCommandBlocked(format!(
                "Invalid Google Chat webhook URL '{trimmed}': must start with 'https://chat.googleapis.com/v1/spaces/'"
            )));
        }

        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .unwrap_or_default();

        let resp = http
            .post(trimmed)
            .json(&json!({ "text": text }))
            .send()
            .await
            .map_err(|e| CoreError::SubprocessError(format!("Chat webhook failed: {e}")))?;

        parse_json_response(resp, trimmed).await
    }

    async fn get_json(&self, url: &str, token: &str) -> Result<Value, CoreError> {
        let resp = self
            .http
            .get(url)
            .bearer_auth(token)
            .send()
            .await
            .map_err(|e| CoreError::SubprocessError(format!("Chat GET '{url}': {e}")))?;
        parse_json_response(resp, url).await
    }

    async fn post_json(&self, url: &str, token: &str, body: &Value) -> Result<Value, CoreError> {
        let resp = self
            .http
            .post(url)
            .bearer_auth(token)
            .json(body)
            .send()
            .await
            .map_err(|e| CoreError::SubprocessError(format!("Chat POST '{url}': {e}")))?;
        parse_json_response(resp, url).await
    }
}

fn normalize_space_name(raw: &str) -> Result<String, CoreError> {
    let trimmed = raw.trim().trim_matches('/');
    if trimmed.is_empty() || trimmed.contains("..") || trimmed.contains('\\') {
        return Err(CoreError::RefsPathLeakViolation(format!(
            "Invalid Google Chat space identifier '{raw}'"
        )));
    }
    if trimmed.starts_with("spaces/") {
        Ok(trimmed.to_string())
    } else {
        Ok(format!("spaces/{trimmed}"))
    }
}

fn parse_form_summary(val: &Value) -> GoogleFormSummaryDto {
    let form_id = val
        .get("formId")
        .or_else(|| val.get("form_id"))
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    let title = val
        .pointer("/info/title")
        .or_else(|| val.get("title"))
        .and_then(|v| v.as_str())
        .unwrap_or("Untitled Form")
        .to_string();
    let description = val
        .pointer("/info/description")
        .or_else(|| val.get("description"))
        .and_then(|v| v.as_str())
        .map(ToString::to_string);
    let responder_uri = val
        .get("responderUri")
        .or_else(|| val.get("responder_uri"))
        .and_then(|v| v.as_str())
        .map(ToString::to_string);
    let linked_sheet_id = val
        .get("linkedSheetId")
        .or_else(|| val.get("linked_sheet_id"))
        .and_then(|v| v.as_str())
        .map(ToString::to_string);

    let mut questions = Vec::new();
    if let Some(items) = val.get("items").and_then(|v| v.as_array()) {
        for item in items {
            if let Some(q_item) = item.get("questionItem") {
                let q = q_item.get("question").unwrap_or(q_item);
                let item_id = q
                    .get("questionId")
                    .or_else(|| item.get("itemId"))
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string();
                let q_title = item
                    .get("title")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Untitled Question")
                    .to_string();
                let required = q.get("required").and_then(|v| v.as_bool()).unwrap_or(false);

                let (question_type, options) = if let Some(cq) = q.get("choiceQuestion") {
                    let qtype = cq
                        .get("type")
                        .and_then(|v| v.as_str())
                        .unwrap_or("CHOICE")
                        .to_string();
                    let opts = cq
                        .get("options")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|o| o.get("value").and_then(|v| v.as_str()).map(ToString::to_string))
                                .collect()
                        })
                        .unwrap_or_default();
                    (qtype, opts)
                } else if q.get("scaleQuestion").is_some() {
                    ("SCALE".to_string(), Vec::new())
                } else if q.get("dateQuestion").is_some() {
                    ("DATE".to_string(), Vec::new())
                } else if q.get("timeQuestion").is_some() {
                    ("TIME".to_string(), Vec::new())
                } else {
                    ("TEXT".to_string(), Vec::new())
                };

                questions.push(GoogleFormQuestionDto {
                    item_id,
                    title: q_title,
                    question_type,
                    required,
                    options,
                });
            }
        }
    } else if let Some(arr) = val.get("questions").and_then(|v| v.as_array()) {
        for q in arr {
            if let Ok(dto) = serde_json::from_value::<GoogleFormQuestionDto>(q.clone()) {
                questions.push(dto);
            }
        }
    }

    GoogleFormSummaryDto {
        form_id,
        title,
        description,
        responder_uri,
        linked_sheet_id,
        questions,
    }
}

fn parse_form_response(val: &Value) -> GoogleFormResponseDto {
    let response_id = val
        .get("responseId")
        .or_else(|| val.get("response_id"))
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    let create_time = val
        .get("createTime")
        .or_else(|| val.get("create_time"))
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    let last_submitted_time = val
        .get("lastSubmittedTime")
        .or_else(|| val.get("last_submitted_time"))
        .and_then(|v| v.as_str())
        .unwrap_or(&create_time)
        .to_string();
    let respondent_email = val
        .get("respondentEmail")
        .or_else(|| val.get("respondent_email"))
        .and_then(|v| v.as_str())
        .map(ToString::to_string);

    let mut answers = Vec::new();
    if let Some(ans_map) = val.get("answers").and_then(|v| v.as_object()) {
        // Stable key ordering via BTreeMap
        let ordered: HashMap<String, &Value> = ans_map
            .iter()
            .map(|(k, v)| (k.clone(), v))
            .collect();
        let mut keys: Vec<String> = ordered.keys().cloned().collect();
        keys.sort();
        for key in keys {
            let entry = ordered[&key];
            let qid = entry
                .get("questionId")
                .and_then(|v| v.as_str())
                .unwrap_or(&key)
                .to_string();
            let text_vals: Vec<String> = entry
                .pointer("/textAnswers/answers")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|a| a.get("value").and_then(|v| v.as_str()).map(ToString::to_string))
                        .collect()
                })
                .unwrap_or_default();
            if !text_vals.is_empty() {
                answers.push((qid, text_vals.join(", ")));
            } else if let Some(s) = entry.as_str() {
                answers.push((qid, s.to_string()));
            }
        }
    }

    GoogleFormResponseDto {
        response_id,
        create_time,
        last_submitted_time,
        respondent_email,
        answers,
    }
}

fn parse_chat_space(val: &Value) -> GoogleChatSpaceDto {
    let space_type = val
        .get("spaceType")
        .or_else(|| val.get("type"))
        .and_then(|v| v.as_str())
        .unwrap_or("ROOM")
        .to_string();
    let threaded = val
        .get("spaceThreadingState")
        .and_then(|v| v.as_str())
        .is_some_and(|s| s.contains("THREADED"))
        || val.get("threaded").and_then(|v| v.as_bool()).unwrap_or(false);

    GoogleChatSpaceDto {
        name: val
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string(),
        display_name: val
            .get("displayName")
            .or_else(|| val.get("display_name"))
            .and_then(|v| v.as_str())
            .unwrap_or("Unnamed Space")
            .to_string(),
        space_type,
        threaded,
    }
}

fn parse_chat_message(val: &Value) -> GoogleChatMessageDto {
    GoogleChatMessageDto {
        name: val
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string(),
        sender_name: val
            .pointer("/sender/displayName")
            .or_else(|| val.pointer("/sender/name"))
            .or_else(|| val.get("sender_name"))
            .and_then(|v| v.as_str())
            .map(ToString::to_string),
        text: val
            .get("text")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string(),
        create_time: val
            .get("createTime")
            .or_else(|| val.get("create_time"))
            .and_then(|v| v.as_str())
            .map(ToString::to_string),
        thread_name: val
            .pointer("/thread/name")
            .or_else(|| val.get("thread_name"))
            .and_then(|v| v.as_str())
            .map(ToString::to_string),
    }
}
