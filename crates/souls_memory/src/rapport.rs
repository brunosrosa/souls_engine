//! User Rapport Profile (EMA-smoothed linguistic mirroring & pacing) and
//! Vision Snapshot Deduplication Ledger (`vision_snapshot_ledger`).
//!
//! Implements Stage 2 persistence for Milestone v0.9.3 / ADR-033 on top of
//! FrankenSQLite STRICT WAL tables `user_rapport_profile` and
//! `vision_snapshot_ledger`.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use sqlx::{Pool, Row, Sqlite};
use tracing::instrument;

use souls_protocol::multimodal::{
    clamp_mirroring_ratio, CognitiveVoiceMode, DetectedEmotionTag, UserRapportProfileDto,
    VisionEscalationReason,
};

use crate::error::MemoryError;

/// Exponential Moving Average smoothing factor for linguistic rapport adaptation.
pub const RAPPORT_EMA_ALPHA: f64 = 0.15;

/// Maximum consecutive identical visual snapshots allowed to be marked unchanged
/// before forcing a fresh capture (`screen_unchanged = false`).
pub const MAX_SNAPSHOT_OMISSION_STREAK: usize = 5;

const MAX_SLANG_LEXICON_LEN: usize = 24;

static SNAPSHOT_SEQ: AtomicU64 = AtomicU64::new(1);

const CASUAL_FORMALITY_TOKENS: &[&str] = &[
    "vc", "tb", "tbm", "pra", "pro", "tô", "to", "ta", "tá", "né", "ne", "massa", "cara", "mano",
    "beleza", "blz", "bugou", "rodar", "fechou", "valeu", "eae", "bora", "show", "tranquilo",
    "firmeza", "top", "suave", "tipo", "kkk", "rsrs", "bro", "dude", "gonna", "wanna", "yeah",
    "yep", "lol",
];

const FORMAL_INDICATORS: &[&str] = &[
    "portanto",
    "todavia",
    "conforme",
    "prezado",
    "solicito",
    "gostaria",
    "poderia",
    "considerando",
    "entretanto",
    "senhor",
    "senhora",
    "atenciosamente",
    "arquitetura",
    "especificação",
    "implementação",
    "therefore",
    "however",
    "regarding",
    "please",
    "kindly",
    "furthermore",
];

const CASUAL_LEXICON_TERMS: &[&str] = &[
    "massa",
    "pode crê",
    "rodar",
    "bugou",
    "fechou",
    "bora",
    "show",
    "tranquilo",
    "firmeza",
    "blz",
    "top",
    "valeu",
    "cara",
    "mano",
    "suave",
    "beleza",
];

fn current_unix_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn normalize_user_id(user_id: &str) -> &str {
    let trimmed = user_id.trim();
    if trimmed.is_empty() {
        "default"
    } else {
        trimmed
    }
}

fn normalize_preferred_pacing(pacing: &str) -> String {
    match pacing.trim() {
        "short_direct" => "short_direct".to_string(),
        "elaborate" => "elaborate".to_string(),
        "bulleted" => "bulleted".to_string(),
        _ => "short_direct".to_string(),
    }
}

fn emotion_to_snake_case(tag: DetectedEmotionTag) -> &'static str {
    match tag {
        DetectedEmotionTag::Neutral => "neutral",
        DetectedEmotionTag::Happy => "happy",
        DetectedEmotionTag::Angry => "angry",
        DetectedEmotionTag::Hesitant => "hesitant",
        DetectedEmotionTag::Laughter => "laughter",
        DetectedEmotionTag::Sad => "sad",
        DetectedEmotionTag::Focused => "focused",
    }
}

fn vision_reason_to_snake_case(reason: VisionEscalationReason) -> &'static str {
    match reason {
        VisionEscalationReason::NoneAxSufficient => "none_ax_sufficient",
        VisionEscalationReason::CanvasOrWebGlDetected => "canvas_or_web_gl_detected",
        VisionEscalationReason::ZeroActionableNodes => "zero_actionable_nodes",
        VisionEscalationReason::RemoteDesktopOrVm => "remote_desktop_or_vm",
        VisionEscalationReason::ExplicitVisualQaRequest => "explicit_visual_qa_request",
    }
}

fn next_snapshot_id(now_secs: i64) -> String {
    let seq = SNAPSHOT_SEQ.fetch_add(1, Ordering::Relaxed);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    format!("vsnap_{}_{}_{}", now_secs, nanos, seq)
}

fn row_to_rapport_profile(row: &sqlx::sqlite::SqliteRow) -> UserRapportProfileDto {
    let user_id: String = row.get("user_id");
    let formality_score: f64 = row.get("formality_score");
    let avg_sentence_length_i64: i64 = row.get("avg_sentence_length");
    let preferred_pacing_raw: String = row.get("preferred_pacing");
    let slang_lexicon_json: String = row.get("slang_lexicon_json");
    let mirroring_ratio: f64 = row.get("mirroring_ratio");
    let mode_raw: String = row.get("active_cognitive_mode");
    let contemplative_i64: i64 = row.get("contemplative_base_ms");
    let backchannel_i64: i64 = row.get("backchannel_enabled");
    let emotion_raw: String = row.get("last_emotion");
    let turn_count_i64: i64 = row.get("turn_count");
    let updated_at: i64 = row.get("updated_at");

    let slang_lexicon: Vec<String> = serde_json::from_str(&slang_lexicon_json).unwrap_or_default();

    let mut dto = UserRapportProfileDto {
        user_id,
        formality_score,
        avg_sentence_length: avg_sentence_length_i64.max(1) as u32,
        preferred_pacing: normalize_preferred_pacing(&preferred_pacing_raw),
        slang_lexicon,
        mirroring_ratio,
        active_cognitive_mode: CognitiveVoiceMode::from_str_lossy(&mode_raw),
        contemplative_base_ms: contemplative_i64.clamp(100, 1000) as u64,
        backchannel_enabled: backchannel_i64 != 0,
        last_emotion: DetectedEmotionTag::from_str_lossy(&emotion_raw),
        turn_count: turn_count_i64.max(0) as u64,
        updated_at,
    };
    dto.sanitize();
    dto
}

async fn fetch_rapport_profile(
    pool: &Pool<Sqlite>,
    user_id: &str,
) -> Result<Option<UserRapportProfileDto>, MemoryError> {
    let row = sqlx::query(
        r#"
        SELECT user_id, formality_score, avg_sentence_length, preferred_pacing,
               slang_lexicon_json, mirroring_ratio, active_cognitive_mode,
               contemplative_base_ms, backchannel_enabled, last_emotion,
               turn_count, updated_at
        FROM user_rapport_profile
        WHERE user_id = ?1
        "#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    Ok(row.as_ref().map(row_to_rapport_profile))
}

/// Retrieves the user's rapport profile or initializes a default profile if missing.
#[instrument(skip(pool))]
pub async fn get_or_init_rapport_profile(
    pool: &Pool<Sqlite>,
    user_id: &str,
) -> Result<UserRapportProfileDto, MemoryError> {
    let uid = normalize_user_id(user_id);

    if let Some(existing) = fetch_rapport_profile(pool, uid).await? {
        return Ok(existing);
    }

    let mut default_profile = UserRapportProfileDto {
        user_id: uid.to_string(),
        updated_at: current_unix_secs(),
        ..UserRapportProfileDto::default()
    };
    default_profile.sanitize();

    let slang_json = serde_json::to_string(&default_profile.slang_lexicon)?;
    let res = sqlx::query(
        r#"
        INSERT INTO user_rapport_profile (
            user_id, formality_score, avg_sentence_length, preferred_pacing,
            slang_lexicon_json, mirroring_ratio, active_cognitive_mode,
            contemplative_base_ms, backchannel_enabled, last_emotion,
            turn_count, updated_at
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
        ON CONFLICT(user_id) DO NOTHING
        "#,
    )
    .bind(&default_profile.user_id)
    .bind(default_profile.formality_score)
    .bind(default_profile.avg_sentence_length as i64)
    .bind(&default_profile.preferred_pacing)
    .bind(&slang_json)
    .bind(default_profile.mirroring_ratio)
    .bind(default_profile.active_cognitive_mode.as_str())
    .bind(default_profile.contemplative_base_ms as i64)
    .bind(if default_profile.backchannel_enabled { 1_i64 } else { 0_i64 })
    .bind(emotion_to_snake_case(default_profile.last_emotion))
    .bind(default_profile.turn_count as i64)
    .bind(default_profile.updated_at)
    .execute(pool)
    .await?;

    if res.rows_affected() == 0 {
        if let Some(existing) = fetch_rapport_profile(pool, uid).await? {
            return Ok(existing);
        }
    }

    Ok(default_profile)
}

/// Sanitizes and upserts a user rapport profile into SQLite STRICT table `user_rapport_profile`.
#[instrument(skip(pool, profile))]
pub async fn upsert_rapport_profile(
    pool: &Pool<Sqlite>,
    profile: &UserRapportProfileDto,
) -> Result<UserRapportProfileDto, MemoryError> {
    let mut sanitized = profile.clone();
    sanitized.sanitize();
    sanitized.user_id = normalize_user_id(&sanitized.user_id).to_string();
    sanitized.preferred_pacing = normalize_preferred_pacing(&sanitized.preferred_pacing);
    if sanitized.slang_lexicon.len() > MAX_SLANG_LEXICON_LEN {
        sanitized.slang_lexicon.truncate(MAX_SLANG_LEXICON_LEN);
    }

    let slang_json = serde_json::to_string(&sanitized.slang_lexicon)?;

    sqlx::query(
        r#"
        INSERT INTO user_rapport_profile (
            user_id, formality_score, avg_sentence_length, preferred_pacing,
            slang_lexicon_json, mirroring_ratio, active_cognitive_mode,
            contemplative_base_ms, backchannel_enabled, last_emotion,
            turn_count, updated_at
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
        ON CONFLICT(user_id) DO UPDATE SET
            formality_score = excluded.formality_score,
            avg_sentence_length = excluded.avg_sentence_length,
            preferred_pacing = excluded.preferred_pacing,
            slang_lexicon_json = excluded.slang_lexicon_json,
            mirroring_ratio = excluded.mirroring_ratio,
            active_cognitive_mode = excluded.active_cognitive_mode,
            contemplative_base_ms = excluded.contemplative_base_ms,
            backchannel_enabled = excluded.backchannel_enabled,
            last_emotion = excluded.last_emotion,
            turn_count = excluded.turn_count,
            updated_at = excluded.updated_at
        "#,
    )
    .bind(&sanitized.user_id)
    .bind(sanitized.formality_score)
    .bind(sanitized.avg_sentence_length as i64)
    .bind(&sanitized.preferred_pacing)
    .bind(&slang_json)
    .bind(sanitized.mirroring_ratio)
    .bind(sanitized.active_cognitive_mode.as_str())
    .bind(sanitized.contemplative_base_ms as i64)
    .bind(if sanitized.backchannel_enabled { 1_i64 } else { 0_i64 })
    .bind(emotion_to_snake_case(sanitized.last_emotion))
    .bind(sanitized.turn_count as i64)
    .bind(sanitized.updated_at)
    .execute(pool)
    .await?;

    Ok(sanitized)
}

fn tokenize_words_lower(text: &str) -> Vec<String> {
    text.split_whitespace()
        .map(|raw| {
            raw.trim_matches(|c: char| !c.is_alphanumeric())
                .to_lowercase()
        })
        .filter(|tok| !tok.is_empty())
        .collect()
}

fn compute_utterance_metrics(transcript: &str) -> Option<(f64, f64, Vec<String>)> {
    let trimmed = transcript.trim();
    if trimmed.is_empty() {
        return None;
    }

    let words = tokenize_words_lower(trimmed);
    if words.is_empty() {
        return None;
    }

    let sentences: Vec<&str> = trimmed
        .split(['.', '!', '?', '\n'])
        .map(str::trim)
        .filter(|s| s.split_whitespace().next().is_some())
        .collect();
    let sentence_count = sentences.len().max(1);
    let obs_sentence_len = (words.len() as f64 / sentence_count as f64).max(1.0);

    let lower_text = trimmed.to_lowercase();

    let mut casual_hits = 0_usize;
    let mut formal_hits = 0_usize;

    for token in &words {
        if CASUAL_FORMALITY_TOKENS.contains(&token.as_str()) {
            casual_hits += 1;
        }
        if FORMAL_INDICATORS.contains(&token.as_str()) {
            formal_hits += 1;
        }
    }

    if lower_text.contains("pode crê") || lower_text.contains("pode cre") {
        casual_hits += 1;
    }

    let mut formality: f64 = 5.0;
    formality -= (casual_hits as f64) * 0.85;
    formality += (formal_hits as f64) * 0.95;

    if trimmed.contains(';') || trimmed.contains(':') {
        formality += 0.6;
    }
    if casual_hits == 0 && obs_sentence_len >= 12.0 && trimmed.contains('.') {
        formality += 0.7;
    }

    let obs_formality = formality.clamp(0.0, 10.0);

    // Extract recurring casual / dev-work vocabulary in deterministic order
    let mut extracted_slang = Vec::new();
    for &term in CASUAL_LEXICON_TERMS {
        let matched = if term == "pode crê" {
            contains_phrase(&lower_text, "pode crê") || contains_phrase(&lower_text, "pode cre")
        } else {
            words.iter().any(|w| w == term)
        };
        if matched {
            extracted_slang.push(term.to_string());
        }
    }

    Some((obs_formality, obs_sentence_len, extracted_slang))
}

fn contains_phrase(haystack_lower: &str, phrase: &str) -> bool {
    let mut start = 0;
    while let Some(idx) = haystack_lower[start..].find(phrase) {
        let abs_start = start + idx;
        let abs_end = abs_start + phrase.len();
        let left_ok = abs_start == 0
            || !haystack_lower[..abs_start]
                .chars()
                .next_back()
                .map(char::is_alphanumeric)
                .unwrap_or(false);
        let right_ok = abs_end == haystack_lower.len()
            || !haystack_lower[abs_end..]
                .chars()
                .next()
                .map(char::is_alphanumeric)
                .unwrap_or(false);
        if left_ok && right_ok {
            return true;
        }
        start = abs_start + phrase.len();
    }
    false
}

/// Updates the user's rapport profile using EMA smoothing (`RAPPORT_EMA_ALPHA = 0.15`)
/// with bounded per-turn variation (`<= 10%`), extracts Brazilian Portuguese casual terms
/// (capped at 24), adjusts `contemplative_base_ms` by emotion tag, and persists the result.
#[instrument(skip(pool, transcript))]
pub async fn update_rapport_ema_from_utterance(
    pool: &Pool<Sqlite>,
    user_id: &str,
    transcript: &str,
    emotion_tag: DetectedEmotionTag,
) -> Result<UserRapportProfileDto, MemoryError> {
    let mut profile = get_or_init_rapport_profile(pool, user_id).await?;

    if let Some((obs_formality, obs_sentence_len, extracted_slang)) =
        compute_utterance_metrics(transcript)
    {
        let prev_formality = profile.formality_score.clamp(0.0, 10.0);
        let ema_formality =
            RAPPORT_EMA_ALPHA * obs_formality + (1.0 - RAPPORT_EMA_ALPHA) * prev_formality;
        let max_formality_delta = 0.10 * prev_formality.max(1.0);
        profile.formality_score = ema_formality
            .clamp(
                prev_formality - max_formality_delta,
                prev_formality + max_formality_delta,
            )
            .clamp(0.0, 10.0);

        let prev_len = profile.avg_sentence_length.max(1) as f64;
        let ema_len = RAPPORT_EMA_ALPHA * obs_sentence_len + (1.0 - RAPPORT_EMA_ALPHA) * prev_len;
        let max_len_delta = ((prev_len * 0.10).floor() as i64).max(1);
        let clamped_len = (ema_len.round() as i64)
            .clamp(prev_len as i64 - max_len_delta, prev_len as i64 + max_len_delta)
            .max(1) as u32;
        profile.avg_sentence_length = clamped_len;

        for term in extracted_slang {
            if profile.slang_lexicon.len() >= MAX_SLANG_LEXICON_LEN {
                break;
            }
            if !profile
                .slang_lexicon
                .iter()
                .any(|existing| existing.eq_ignore_ascii_case(&term))
            {
                profile.slang_lexicon.push(term);
            }
        }
    }

    let target_contemplative_ms: f64 = match emotion_tag {
        DetectedEmotionTag::Hesitant | DetectedEmotionTag::Sad => 450.0,
        DetectedEmotionTag::Angry => 220.0,
        DetectedEmotionTag::Happy | DetectedEmotionTag::Laughter => 275.0,
        DetectedEmotionTag::Neutral | DetectedEmotionTag::Focused => 260.0,
    };
    let ema_contemplative = RAPPORT_EMA_ALPHA * target_contemplative_ms
        + (1.0 - RAPPORT_EMA_ALPHA) * (profile.contemplative_base_ms as f64);
    profile.contemplative_base_ms = (ema_contemplative.round() as u64).clamp(100, 1000);

    profile.mirroring_ratio = clamp_mirroring_ratio(profile.mirroring_ratio);
    profile.turn_count = profile.turn_count.saturating_add(1);
    profile.last_emotion = emotion_tag;
    profile.updated_at = current_unix_secs();

    upsert_rapport_profile(pool, &profile).await
}

/// Records a visual snapshot capture in `vision_snapshot_ledger` and returns `true`
/// if the screen is unchanged AND fewer than `MAX_SNAPSHOT_OMISSION_STREAK` (5)
/// consecutive prior omissions have occurred for `(session_id, app_name, window_title)`.
#[allow(clippy::too_many_arguments)]
#[instrument(skip(pool, sha256_hash))]
pub async fn record_and_check_vision_snapshot(
    pool: &Pool<Sqlite>,
    session_id: &str,
    app_name: &str,
    window_title: &str,
    sha256_hash: &str,
    width: u32,
    height: u32,
    som_count: usize,
    reason: VisionEscalationReason,
) -> Result<bool, MemoryError> {
    let recent_rows = sqlx::query(
        r#"
        SELECT sha256_hash, screen_unchanged
        FROM vision_snapshot_ledger
        WHERE session_id = ?1 AND app_name = ?2 AND window_title = ?3
        ORDER BY created_at DESC, rowid DESC
        LIMIT ?4
        "#,
    )
    .bind(session_id)
    .bind(app_name)
    .bind(window_title)
    .bind(MAX_SNAPSHOT_OMISSION_STREAK as i64)
    .fetch_all(pool)
    .await?;

    let unchanged = if let Some(latest) = recent_rows.first() {
        let latest_hash: String = latest.get("sha256_hash");
        if latest_hash == sha256_hash {
            let prior_omission_streak = recent_rows
                .iter()
                .take_while(|row| {
                    let row_hash: String = row.get("sha256_hash");
                    let row_unchanged: i64 = row.get("screen_unchanged");
                    row_hash == sha256_hash && row_unchanged == 1
                })
                .count();
            prior_omission_streak < MAX_SNAPSHOT_OMISSION_STREAK
        } else {
            false
        }
    } else {
        false
    };

    let now_secs = current_unix_secs();
    let snapshot_id = next_snapshot_id(now_secs);
    let reason_str = vision_reason_to_snake_case(reason);

    sqlx::query(
        r#"
        INSERT INTO vision_snapshot_ledger (
            snapshot_id, session_id, app_name, window_title, sha256_hash,
            width, height, som_marks_count, escalation_reason,
            screen_unchanged, created_at
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
        "#,
    )
    .bind(&snapshot_id)
    .bind(session_id)
    .bind(app_name)
    .bind(window_title)
    .bind(sha256_hash)
    .bind(width as i64)
    .bind(height as i64)
    .bind(som_count as i64)
    .bind(reason_str)
    .bind(if unchanged { 1_i64 } else { 0_i64 })
    .bind(now_secs)
    .execute(pool)
    .await?;

    Ok(unchanged)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sqlite::create_in_memory_sqlite_pool;

    #[test]
    fn test_snake_case_mappings_match_serde() {
        for tag in [
            DetectedEmotionTag::Neutral,
            DetectedEmotionTag::Happy,
            DetectedEmotionTag::Angry,
            DetectedEmotionTag::Hesitant,
            DetectedEmotionTag::Laughter,
            DetectedEmotionTag::Sad,
            DetectedEmotionTag::Focused,
        ] {
            let expected = serde_json::to_string(&tag).unwrap();
            assert_eq!(emotion_to_snake_case(tag), expected.trim_matches('"'));
        }

        for reason in [
            VisionEscalationReason::NoneAxSufficient,
            VisionEscalationReason::CanvasOrWebGlDetected,
            VisionEscalationReason::ZeroActionableNodes,
            VisionEscalationReason::RemoteDesktopOrVm,
            VisionEscalationReason::ExplicitVisualQaRequest,
        ] {
            let expected = serde_json::to_string(&reason).unwrap();
            assert_eq!(
                vision_reason_to_snake_case(reason),
                expected.trim_matches('"')
            );
        }
    }

    #[tokio::test]
    async fn test_rapport_init_and_upsert_sanitization() {
        let pool = create_in_memory_sqlite_pool().await.unwrap();

        // Empty user_id normalizes to "default"
        let profile = get_or_init_rapport_profile(&pool, "   ").await.unwrap();
        assert_eq!(profile.user_id, "default");
        assert!((profile.formality_score - 3.5).abs() < f64::EPSILON);
        assert_eq!(profile.avg_sentence_length, 12);
        assert_eq!(profile.preferred_pacing, "short_direct");
        assert!((profile.mirroring_ratio - 0.25).abs() < f64::EPSILON);
        assert_eq!(profile.active_cognitive_mode, CognitiveVoiceMode::Auto);
        assert_eq!(profile.contemplative_base_ms, 250);
        assert!(profile.backchannel_enabled);
        assert_eq!(profile.last_emotion, DetectedEmotionTag::Neutral);
        assert_eq!(profile.turn_count, 0);

        // Dirty upsert should clamp mirroring_ratio <= 0.35 and validate preferred_pacing
        let dirty = UserRapportProfileDto {
            user_id: "architect_01".to_string(),
            formality_score: 14.2,
            avg_sentence_length: 0,
            preferred_pacing: "invalid_pacing_value".to_string(),
            slang_lexicon: vec!["massa".to_string(), "show".to_string()],
            mirroring_ratio: 0.92,
            active_cognitive_mode: CognitiveVoiceMode::HandsOn,
            contemplative_base_ms: 4200,
            backchannel_enabled: false,
            last_emotion: DetectedEmotionTag::Focused,
            turn_count: 3,
            updated_at: 1_700_000_000,
        };

        let saved = upsert_rapport_profile(&pool, &dirty).await.unwrap();
        assert_eq!(saved.user_id, "architect_01");
        assert!((saved.formality_score - 10.0).abs() < f64::EPSILON);
        assert_eq!(saved.avg_sentence_length, 1);
        assert_eq!(saved.preferred_pacing, "short_direct");
        assert!((saved.mirroring_ratio - 0.35).abs() < f64::EPSILON);
        assert_eq!(saved.contemplative_base_ms, 1000);
        assert!(!saved.backchannel_enabled);

        let reloaded = get_or_init_rapport_profile(&pool, "architect_01")
            .await
            .unwrap();
        assert_eq!(reloaded, saved);
    }

    #[tokio::test]
    async fn test_rapport_profile_ema_smoothing() {
        let pool = create_in_memory_sqlite_pool().await.unwrap();

        let initial = get_or_init_rapport_profile(&pool, "user_br").await.unwrap();
        assert!((initial.formality_score - 3.5).abs() < f64::EPSILON);
        assert_eq!(initial.avg_sentence_length, 12);

        // Turn 1: Very casual Brazilian Portuguese dev utterance with slang
        let after_casual = update_rapport_ema_from_utterance(
            &pool,
            "user_br",
            "eae mano, bugou tudo aqui pra rodar, pode crê, massa demais blz!",
            DetectedEmotionTag::Hesitant,
        )
        .await
        .unwrap();

        let delta_formality_1 = (after_casual.formality_score - initial.formality_score).abs();
        assert!(
            delta_formality_1 <= 0.10 * initial.formality_score.max(1.0) + 1e-9,
            "formality delta {delta_formality_1} exceeded 10% cap"
        );
        assert!(
            after_casual.formality_score < initial.formality_score,
            "casual slang should lower formality_score"
        );
        let delta_len_1 =
            (after_casual.avg_sentence_length as i64 - initial.avg_sentence_length as i64).abs();
        assert!(
            (delta_len_1 as f64) <= (initial.avg_sentence_length as f64 * 0.10).ceil() + 1e-9,
            "sentence length delta {delta_len_1} exceeded 10% cap"
        );
        assert!(after_casual.mirroring_ratio <= 0.35);
        assert_eq!(after_casual.turn_count, 1);
        assert_eq!(after_casual.last_emotion, DetectedEmotionTag::Hesitant);
        assert!(
            after_casual.contemplative_base_ms > initial.contemplative_base_ms,
            "Hesitant emotion should nudge contemplative_base_ms upward"
        );
        for expected_term in ["mano", "bugou", "rodar", "pode crê", "massa", "blz"] {
            assert!(
                after_casual
                    .slang_lexicon
                    .iter()
                    .any(|t| t == expected_term),
                "expected slang term '{expected_term}' to be extracted"
            );
        }

        // Turn 2: Highly formal, very long structured utterance -> must still stay bounded <= 10%
        let formal_utterance = "Gostaria de solicitar, portanto, conforme a especificação arquitetural vigente; \
            considerando todos os parâmetros críticos de latência, integridade transacional e observabilidade \
            do sistema distribuído em produção, uma revisão minuciosa de cada módulo envolvido na execução.";
        let after_formal = update_rapport_ema_from_utterance(
            &pool,
            "user_br",
            formal_utterance,
            DetectedEmotionTag::Angry,
        )
        .await
        .unwrap();

        let delta_formality_2 = (after_formal.formality_score - after_casual.formality_score).abs();
        assert!(
            delta_formality_2 <= 0.10 * after_casual.formality_score.max(1.0) + 1e-9,
            "formal step delta {delta_formality_2} exceeded 10% cap"
        );
        assert!(after_formal.formality_score > after_casual.formality_score);
        let delta_len_2 = (after_formal.avg_sentence_length as i64
            - after_casual.avg_sentence_length as i64)
            .abs();
        assert!(
            (delta_len_2 as f64) <= (after_casual.avg_sentence_length as f64 * 0.10).ceil() + 1e-9,
            "formal sentence length delta {delta_len_2} exceeded 10% cap"
        );
        assert!(
            after_formal.contemplative_base_ms < after_casual.contemplative_base_ms,
            "Angry emotion should keep pacing direct by lowering contemplative_base_ms"
        );
        assert_eq!(after_formal.turn_count, 2);
        assert!(after_formal.mirroring_ratio <= 0.35);
    }

    #[tokio::test]
    async fn test_vision_snapshot_dedup_and_staleness_cap() {
        let pool = create_in_memory_sqlite_pool().await.unwrap();

        let session = "sess_vis_01";
        let app = "Code.exe";
        let title = "souls_engine - Visual Studio Code";
        let hash_a = "aaaa1111bbbb2222cccc3333dddd4444";

        // 1st capture: never seen -> unchanged = false
        let first = record_and_check_vision_snapshot(
            &pool,
            session,
            app,
            title,
            hash_a,
            1920,
            1080,
            14,
            VisionEscalationReason::CanvasOrWebGlDetected,
        )
        .await
        .unwrap();
        assert!(!first, "1st snapshot must report screen_unchanged = false");

        // 2nd..6th identical captures (5 consecutive omissions) -> unchanged = true
        for idx in 1..=MAX_SNAPSHOT_OMISSION_STREAK {
            let omitted = record_and_check_vision_snapshot(
                &pool,
                session,
                app,
                title,
                hash_a,
                1920,
                1080,
                14,
                VisionEscalationReason::CanvasOrWebGlDetected,
            )
            .await
            .unwrap();
            assert!(
                omitted,
                "omission #{idx} within streak limit ({MAX_SNAPSHOT_OMISSION_STREAK}) must return true"
            );
        }

        // 7th identical capture -> exceeds 5-omission staleness cap -> must force false
        let seventh = record_and_check_vision_snapshot(
            &pool,
            session,
            app,
            title,
            hash_a,
            1920,
            1080,
            14,
            VisionEscalationReason::CanvasOrWebGlDetected,
        )
        .await
        .unwrap();
        assert!(
            !seventh,
            "7th identical snapshot must reset screen_unchanged = false after 5 omissions"
        );

        // 8th identical capture -> streak restarted cleanly, returns true again
        let eighth = record_and_check_vision_snapshot(
            &pool,
            session,
            app,
            title,
            hash_a,
            1920,
            1080,
            14,
            VisionEscalationReason::CanvasOrWebGlDetected,
        )
        .await
        .unwrap();
        assert!(eighth, "8th snapshot after reset should be deduplicated again");

        // Different hash immediately returns false
        let hash_b = "eeee5555ffff66667777888899990000";
        let changed = record_and_check_vision_snapshot(
            &pool,
            session,
            app,
            title,
            hash_b,
            1920,
            1080,
            18,
            VisionEscalationReason::ExplicitVisualQaRequest,
        )
        .await
        .unwrap();
        assert!(!changed, "changed sha256_hash must report screen_unchanged = false");
    }
}
