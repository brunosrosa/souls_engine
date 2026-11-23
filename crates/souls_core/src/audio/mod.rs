//! # Zero-VRAM Multimodal Voice, Prosody & Pcomp Turn-Taking (`souls_core::audio`)
//!
//! CPU-native voice pipeline keeping 0 MB VRAM pressure on the host GPU:
//! - Pcomp prosodic turn-taking evaluator (open connectives, backchannels, barge-in, 8s hysteresis).
//! - SenseVoice paralinguistic emotion tag extractor & Whisper silence hallucination filter.
//! - Layer-3 markdown-free Portuguese prosodic text shaper + sub-1ms sentence chunker.
//! - Pure-Rust 24 kHz 16-bit mono PCM RIFF/WAVE generator & bounded local TTS/STT cascade.

use crate::error::CoreError;
#[cfg(windows)]
use crate::vision::ax_tree::encode_powershell_utf16le_base64;
use crate::vision::ax_tree::{is_test_mode, run_bounded_subprocess, standard_base64_encode};
use serde::{Deserialize, Serialize};
use souls_protocol::multimodal::{
    AudioEngineTier, CognitiveVoiceMode, DetectedEmotionTag, PcompTurnEvaluationDto,
    TurnTakingVerdict, VoiceSynthesisRequestDto, VoiceSynthesisResultDto,
    VoiceTranscriptionResultDto,
};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

/// Standby hysteresis timeout (8 seconds) before returning Silero VAD to low-power standby.
pub const SILERO_VAD_STANDBY_HYSTERESIS_MS: u64 = 8_000;
/// Minimum sustained user speech duration (300 ms) required to trigger a barge-in abort.
pub const BARGE_IN_MIN_SPEECH_MS: u64 = 300;
/// Maximum transient noise burst duration (200 ms) ignored during agent speech.
pub const NOISE_IGNORE_MAX_MS: u64 = 200;
/// Zero GPU VRAM consumption guarantee for local CPU audio engines.
pub const ZERO_AUDIO_VRAM_CONSUMPTION_MB: usize = 0;

static AUDIO_SEQ: AtomicU64 = AtomicU64::new(1);

// ============================================================================
// 1. Pcomp Turn-Taking Evaluator
// ============================================================================

const SINGLE_OPEN_CONNECTIVES: &[&str] = &[
    "porque", "mas", "então", "entao", "porém", "porem", "como", "para", "quando", "se", "que",
    "e", "ou", "because", "but", "and", "so", "however",
];

const MULTI_OPEN_CONNECTIVES: &[&str] = &["e aí", "e ai", "ou seja"];

fn ends_with_open_connective_tail(trimmed: &str) -> bool {
    if trimmed.is_empty() {
        return false;
    }

    if trimmed.ends_with("...") || trimmed.ends_with(',') {
        return true;
    }

    // Strip trailing punctuation ONLY for `...` and `,`
    let stripped = trimmed
        .trim_end_matches("...")
        .trim_end_matches(',')
        .trim();
    if stripped.is_empty() {
        return false;
    }

    let lower = stripped.to_lowercase();
    let words: Vec<&str> = lower.split_whitespace().collect();
    if words.is_empty() {
        return false;
    }

    if words.len() >= 2 {
        let last_two = format!("{} {}", words[words.len() - 2], words[words.len() - 1]);
        if MULTI_OPEN_CONNECTIVES.contains(&last_two.as_str()) {
            return true;
        }
    }

    let last = words[words.len() - 1];
    SINGLE_OPEN_CONNECTIVES.contains(&last)
}

fn tail_excerpt(text: &str, max_chars: usize) -> String {
    let trimmed = text.trim();
    let count = trimmed.chars().count();
    if count <= max_chars {
        return trimmed.to_string();
    }
    trimmed.chars().skip(count - max_chars).collect()
}

/// Evaluates whether the user has finished their conversational turn using prosodic,
/// syntactic, and acoustic signals.
pub fn evaluate_pcomp_turn_taking(
    transcript: &str,
    silence_ms: u64,
    acoustic_stress: f32,
    is_agent_speaking: bool,
    user_voice_active_ms: u64,
) -> PcompTurnEvaluationDto {
    let trimmed = transcript.trim();
    let stress = if acoustic_stress.is_nan() {
        0.0
    } else {
        acoustic_stress.clamp(0.0, 1.0)
    };

    let ends_with_open_connective = ends_with_open_connective_tail(trimmed);
    let has_terminal_punct =
        !ends_with_open_connective && (trimmed.ends_with('.') || trimmed.ends_with('!') || trimmed.ends_with('?'));

    let syntactic_closure_score: f32 = if trimmed.is_empty() {
        0.0
    } else if ends_with_open_connective {
        0.2
    } else if has_terminal_punct {
        0.9
    } else {
        0.55
    };

    let syntactic_complexity = (1.0 - syntactic_closure_score).clamp(0.0, 1.0);
    let mut recommended_wait_ms =
        (200 + (150.0 * syntactic_complexity + 250.0 * stress).round() as u64).clamp(200, 600);

    let mut suggested_backchannel = None;

    let verdict = if is_agent_speaking {
        if user_voice_active_ms >= BARGE_IN_MIN_SPEECH_MS {
            recommended_wait_ms = 0;
            TurnTakingVerdict::BargeInAbort
        } else {
            TurnTakingVerdict::ContinueListening
        }
    } else if silence_ms >= SILERO_VAD_STANDBY_HYSTERESIS_MS {
        TurnTakingVerdict::StandbyTimeout
    } else if ends_with_open_connective {
        if (300..=1200).contains(&silence_ms) {
            let cue = match trimmed.len() % 3 {
                0 => "Aham...",
                1 => "Entendo...",
                _ => "Certo...",
            };
            suggested_backchannel = Some(cue.to_string());
            TurnTakingVerdict::EmitBackchannel
        } else if silence_ms < 2000 {
            TurnTakingVerdict::ContinueListening
        } else {
            TurnTakingVerdict::ContemplativeWait
        }
    } else if silence_ms >= recommended_wait_ms {
        TurnTakingVerdict::TakeTurnImmediate
    } else {
        TurnTakingVerdict::ContemplativeWait
    };

    PcompTurnEvaluationDto {
        transcript_tail: tail_excerpt(trimmed, 80),
        silence_duration_ms: silence_ms,
        ends_with_open_connective,
        syntactic_closure_score,
        acoustic_stress_score: stress,
        recommended_wait_ms,
        verdict,
        suggested_backchannel,
    }
}

// ============================================================================
// 2. SenseVoice Tag Extraction & Hallucination Filter
// ============================================================================

const KNOWN_EMOTION_TAGS: &[(&str, DetectedEmotionTag)] = &[
    ("HAPPY", DetectedEmotionTag::Happy),
    ("ANGRY", DetectedEmotionTag::Angry),
    ("SAD", DetectedEmotionTag::Sad),
    ("LAUGHTER", DetectedEmotionTag::Laughter),
    ("HESITANT", DetectedEmotionTag::Hesitant),
    ("FOCUSED", DetectedEmotionTag::Focused),
    ("NEUTRAL", DetectedEmotionTag::Neutral),
];

fn normalize_spaces(input: &str) -> String {
    input.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn is_hallucination(clean: &str) -> bool {
    let trimmed = clean.trim();
    if trimmed.is_empty() {
        return true;
    }

    if trimmed.chars().all(|c| !c.is_alphanumeric()) {
        return true;
    }

    let normalized = trimmed
        .to_lowercase()
        .trim_end_matches(['.', '!', '?', ',', '…'])
        .trim()
        .to_string();

    if matches!(
        normalized.as_str(),
        "" | "thank you"
            | "thanks"
            | "thanks for watching"
            | "thank you for watching"
            | "bye"
            | "you"
            | "legendas pela comunidade"
            | "inscreva-se"
            | "obrigado por assistir"
            | "subtitles by the community"
    ) {
        return true;
    }

    let ok_tokens: Vec<String> = trimmed
        .split_whitespace()
        .map(|tok| {
            tok.trim_matches(|c: char| !c.is_alphanumeric())
                .to_lowercase()
        })
        .filter(|s| !s.is_empty())
        .collect();

    if ok_tokens.len() >= 2 && ok_tokens.iter().all(|t| t == "ok") {
        return true;
    }

    false
}

/// Extracts SenseVoice emotion tags, cleans markup, and filters Whisper silence hallucinations.
pub fn parse_sensevoice_and_filter_hallucinations(
    raw_transcript: &str,
) -> (String, DetectedEmotionTag, f32, bool) {
    let mut detected_tag: Option<DetectedEmotionTag> = None;
    let mut cleaned = raw_transcript.to_string();

    // Case-insensitive tag stripping for known emotion tags (both <TAG> and </TAG>)
    loop {
        let upper = cleaned.to_ascii_uppercase();
        let mut removed_any = false;
        for &(tag_name, tag_variant) in KNOWN_EMOTION_TAGS {
            let open_pat = format!("<{tag_name}>");
            let close_pat = format!("</{tag_name}>");

            if let Some(byte_idx) = upper.find(&open_pat) {
                if detected_tag.is_none() {
                    detected_tag = Some(tag_variant);
                }
                cleaned.replace_range(byte_idx..byte_idx + open_pat.len(), " ");
                removed_any = true;
                break;
            }
            if let Some(byte_idx) = upper.find(&close_pat) {
                if detected_tag.is_none() {
                    detected_tag = Some(tag_variant);
                }
                cleaned.replace_range(byte_idx..byte_idx + close_pat.len(), " ");
                removed_any = true;
                break;
            }
        }
        if !removed_any {
            break;
        }
    }

    let clean_text = normalize_spaces(&cleaned);

    if is_hallucination(&clean_text) {
        return (String::new(), DetectedEmotionTag::Neutral, 0.0, true);
    }

    let (emotion, confidence) = if let Some(explicit) = detected_tag {
        (explicit, 0.90)
    } else {
        let lower = clean_text.to_lowercase();
        if lower.contains("...") || lower.contains("é...") || lower.contains("talvez") || lower.contains("hmm") {
            (DetectedEmotionTag::Hesitant, 0.65)
        } else if lower.contains("haha") || lower.contains("kkkk") || lower.contains("rsrs") {
            (DetectedEmotionTag::Laughter, 0.72)
        } else if lower.contains('!') || lower.contains("massa") || lower.contains("funcionou") || lower.contains("ótimo") {
            (DetectedEmotionTag::Happy, 0.68)
        } else {
            (DetectedEmotionTag::Neutral, 0.50)
        }
    };

    (clean_text, emotion, confidence, false)
}

// ============================================================================
// 3. Layer-3 Prosodic Text Shaping & Sub-1ms Sentence Chunking
// ============================================================================

fn strip_markdown_for_speech(raw: &str) -> String {
    // 1. Replace fenced code blocks ```...``` with spoken summary
    let mut no_fences = String::with_capacity(raw.len());
    let mut rest = raw;
    while let Some(start) = rest.find("```") {
        no_fences.push_str(&rest[..start]);
        let after_open = &rest[start + 3..];
        if let Some(end) = after_open.find("```") {
            no_fences.push_str(" bloco de código ");
            rest = &after_open[end + 3..];
        } else {
            no_fences.push_str(" bloco de código ");
            rest = "";
            break;
        }
    }
    no_fences.push_str(rest);

    // 2. Replace markdown links [label](url) -> label
    let mut no_links = String::with_capacity(no_fences.len());
    let mut cur = no_fences.as_str();
    while let Some(open_br) = cur.find('[') {
        let after_open = &cur[open_br + 1..];
        if let Some(close_br) = after_open.find("](") {
            let label = &after_open[..close_br];
            let after_mid = &after_open[close_br + 2..];
            if let Some(close_paren) = after_mid.find(')') {
                no_links.push_str(&cur[..open_br]);
                no_links.push_str(label);
                cur = &after_mid[close_paren + 1..];
                continue;
            }
        }
        no_links.push_str(&cur[..open_br + 1]);
        cur = after_open;
    }
    no_links.push_str(cur);

    // 3. Shorten bare URLs https://... or http://... -> "o link enviado"
    let mut no_urls = String::with_capacity(no_links.len());
    for token in no_links.split_inclusive(char::is_whitespace) {
        let trimmed = token.trim();
        if trimmed.starts_with("https://") || trimmed.starts_with("http://") {
            let trailing_ws: String = token
                .chars()
                .rev()
                .take_while(|c| c.is_whitespace())
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect();
            no_urls.push_str("o link enviado");
            no_urls.push_str(&trailing_ws);
        } else {
            no_urls.push_str(token);
        }
    }

    // 4. Strip heading markers, list bullets, bold/italic, inline backticks per line
    let mut lines_out = Vec::new();
    for line in no_urls.lines() {
        let mut l = line.trim();
        while l.starts_with('#') {
            l = l[1..].trim_start();
        }
        if l.starts_with("> ") {
            l = l[2..].trim_start();
        } else if l.starts_with("- ") || l.starts_with("* ") {
            l = l[2..].trim_start();
        }

        let cleaned_line = l
            .replace("**", "")
            .replace("__", "")
            .replace('`', "")
            .replace('*', "");

        let cleaned_line = strip_underscore_emphasis(&cleaned_line);
        let normalized = normalize_spaces(&cleaned_line);
        if !normalized.is_empty() {
            lines_out.push(normalized);
        }
    }

    lines_out.join("\n\n")
}

fn strip_underscore_emphasis(text: &str) -> String {
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut out = Vec::with_capacity(words.len());
    for w in words {
        let stripped = w.trim_matches('_');
        if !stripped.is_empty() {
            out.push(stripped.to_string());
        }
    }
    out.join(" ")
}

fn insert_breathing_commas(clause: &str) -> String {
    let words: Vec<&str> = clause.split_whitespace().collect();
    if words.len() <= 10 || clause.contains(',') || clause.contains("...") || clause.contains('—') {
        return clause.to_string();
    }
    let mid = words.len() / 2;
    let mut rebuilt = String::with_capacity(clause.len() + 2);
    for (i, w) in words.iter().enumerate() {
        if i > 0 {
            rebuilt.push(' ');
        }
        rebuilt.push_str(w);
        if i == mid && !w.ends_with([',', '.', '!', '?']) {
            rebuilt.push(',');
        }
    }
    rebuilt
}

/// Shapes raw assistant text into natural, markdown-free Portuguese speech with
/// cognitive-mode prosody cues (`...`, `—`, `,`) and pre-chunked sentences.
pub fn shape_prosodic_text_for_tts(raw_text: &str, mode: CognitiveVoiceMode) -> (String, Vec<String>) {
    let stripped = strip_markdown_for_speech(raw_text);
    let base = stripped.trim();
    if base.is_empty() {
        return (String::new(), Vec::new());
    }

    let shaped = match mode {
        CognitiveVoiceMode::Decompression | CognitiveVoiceMode::CrisisParalysis => {
            let mut text = insert_breathing_commas(base);
            if !text.contains("...") {
                if let Some(pos) = text.find(", ") {
                    text.replace_range(pos..pos + 2, "... ");
                } else if let Some(pos) = text.find(". ") {
                    text.replace_range(pos..pos + 2, "... ");
                } else if !text.ends_with(['.', '!', '?']) {
                    text.push_str("...");
                } else {
                    let prefix = "Respire... ";
                    text = format!("{prefix}{text}");
                }
            }
            text
        }
        CognitiveVoiceMode::Sparring | CognitiveVoiceMode::HandsOn => {
            let mut text = base.replace(" - ", " — ").replace(": ", " — ");
            if !text.ends_with(['.', '!', '?']) {
                text.push('.');
            }
            text
        }
        CognitiveVoiceMode::Philosophical => {
            let mut text = base.replace(" - ", " — ").replace(", mas ", " — mas ");
            if !text.contains("...") && !text.contains('—') {
                if let Some(pos) = text.find(". ") {
                    text.replace_range(pos..pos + 2, "... ");
                } else {
                    text.push_str("...");
                }
            }
            text
        }
        CognitiveVoiceMode::Auto => {
            let mut text = base.to_string();
            if !text.ends_with(['.', '!', '?', '…']) {
                text.push('.');
            }
            text
        }
    };

    let chunks = split_prosodic_sentences(&shaped);
    (shaped, chunks)
}

/// Linear sub-1ms sentence boundary splitter preserving prosodic punctuation cleanly.
pub fn split_prosodic_sentences(text: &str) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut current = String::new();
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        let ch = chars[i];

        // Double newline paragraph break
        if ch == '\n' && i + 1 < len && chars[i + 1] == '\n' {
            let trimmed = current.trim();
            if !trimmed.is_empty() {
                chunks.push(trimmed.to_string());
            }
            current.clear();
            while i < len && chars[i] == '\n' {
                i += 1;
            }
            continue;
        }

        // Ellipsis `...`
        if ch == '.' && i + 2 < len && chars[i + 1] == '.' && chars[i + 2] == '.' {
            current.push_str("...");
            let trimmed = current.trim();
            if !trimmed.is_empty() {
                chunks.push(trimmed.to_string());
            }
            current.clear();
            i += 3;
            continue;
        }

        // Avoid splitting inside decimal numbers like 3.14
        if ch == '.'
            && i > 0
            && i + 1 < len
            && chars[i - 1].is_ascii_digit()
            && chars[i + 1].is_ascii_digit()
        {
            current.push(ch);
            i += 1;
            continue;
        }

        current.push(ch);

        if matches!(ch, '.' | '!' | '?' | ';' | '—') {
            let trimmed = current.trim();
            if !trimmed.is_empty() {
                chunks.push(trimmed.to_string());
            }
            current.clear();
        }

        i += 1;
    }

    let tail = current.trim();
    if !tail.is_empty() {
        chunks.push(tail.to_string());
    }

    chunks
}

// ============================================================================
// 4. Pure-Rust 24kHz 16-bit Mono PCM RIFF/WAVE Generator
// ============================================================================

/// Generates a valid 24 kHz, 16-bit mono RIFF/WAVE buffer in pure Rust.
pub fn generate_pcm_wav_buffer(duration_ms: u32, freq_hz: f32, amplitude: f32) -> Vec<u8> {
    const SAMPLE_RATE: u32 = 24_000;
    const CHANNELS: u16 = 1;
    const BITS_PER_SAMPLE: u16 = 16;

    let clamped_ms = duration_ms.clamp(20, 30_000);
    let num_samples = ((SAMPLE_RATE as u64 * clamped_ms as u64) / 1000) as usize;
    let data_len = (num_samples * 2) as u32;
    let byte_rate = SAMPLE_RATE * (CHANNELS as u32) * (BITS_PER_SAMPLE as u32 / 8);
    let block_align = CHANNELS * (BITS_PER_SAMPLE / 8);

    let mut wav = Vec::with_capacity(44 + data_len as usize);
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&(36 + data_len).to_le_bytes());
    wav.extend_from_slice(b"WAVE");
    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16u32.to_le_bytes()); // PCM fmt chunk size
    wav.extend_from_slice(&1u16.to_le_bytes()); // AudioFormat = 1 (PCM)
    wav.extend_from_slice(&CHANNELS.to_le_bytes());
    wav.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
    wav.extend_from_slice(&byte_rate.to_le_bytes());
    wav.extend_from_slice(&block_align.to_le_bytes());
    wav.extend_from_slice(&BITS_PER_SAMPLE.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_len.to_le_bytes());

    let amp = amplitude.clamp(0.0, 1.0);
    let fade_samples = (SAMPLE_RATE as usize / 100).max(1); // 10ms fade envelope

    for idx in 0..num_samples {
        let t = idx as f32 / SAMPLE_RATE as f32;
        let env = if idx < fade_samples {
            idx as f32 / fade_samples as f32
        } else if idx + fade_samples > num_samples {
            (num_samples - idx) as f32 / fade_samples as f32
        } else {
            1.0
        };
        let sample = (2.0 * std::f32::consts::PI * freq_hz * t).sin() * amp * env;
        let pcm_i16 = (sample * i16::MAX as f32) as i16;
        wav.extend_from_slice(&pcm_i16.to_le_bytes());
    }

    wav
}

/// Generates a short soft backchannel acknowledgment WAV buffer (`"Aham..."`, `"Certo..."`).
pub fn backchannel_pcm_wav(phrase: &str) -> Vec<u8> {
    let base_freq = match phrase.len() % 3 {
        0 => 320.0,
        1 => 350.0,
        _ => 380.0,
    };
    generate_pcm_wav_buffer(180, base_freq, 0.22)
}

/// Generates a warm 24 kHz handoff chime WAV buffer.
pub fn handoff_chime_wav() -> Vec<u8> {
    generate_pcm_wav_buffer(240, 528.0, 0.28)
}

// ============================================================================
// 5. Audio Engine Inventory & Zero-VRAM Synthesis / Transcription Cascade
// ============================================================================

/// Probe result describing available local CPU audio engines.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AudioEngineInventory {
    pub onnx_kokoro_ready: bool,
    pub onnx_sensevoice_ready: bool,
    pub silero_vad_ready: bool,
    pub piper_ready: bool,
    pub whisper_ready: bool,
    pub windows_speech_ready: bool,
    pub active_tts_engine: String,
    pub active_stt_engine: String,
}

/// Detailed probe of host/Hermes local CPU audio tools and models (`0 MB VRAM`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HermesAudioToolchainProbe {
    pub inventory: AudioEngineInventory,
    pub edge_tts_bin: Option<String>,
    pub sherpa_onnx_cli_bin: Option<String>,
    pub faster_whisper_python: Option<String>,
    pub faster_whisper_model_path: Option<String>,
    pub ffmpeg_bin: Option<String>,
    pub piper_bin: Option<String>,
    pub whisper_cli_bin: Option<String>,
    pub hermes_venv_scripts: Vec<String>,
}

fn dir_contains_matching(dir: &Path, needles: &[&str], ext: &str) -> bool {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return false;
    };
    for entry in entries.flatten() {
        let p = entry.path();
        if !p.is_file() {
            continue;
        }
        let name_lower = p
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if !ext.is_empty() && !name_lower.ends_with(ext) {
            continue;
        }
        if needles.iter().any(|n| name_lower.contains(n)) {
            return true;
        }
    }
    false
}

fn find_first_matching_file(dir: &Path, needles: &[&str], ext: &str) -> Option<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return None;
    };
    for entry in entries.flatten() {
        let p = entry.path();
        if !p.is_file() {
            continue;
        }
        let name_lower = p
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if !ext.is_empty() && !name_lower.ends_with(ext) {
            continue;
        }
        if needles.iter().any(|n| name_lower.contains(n)) {
            return Some(p);
        }
    }
    None
}

fn find_binary_on_path_or_dirs(names: &[&str], extra_dirs: &[PathBuf]) -> Option<PathBuf> {
    for dir in extra_dirs {
        for name in names {
            let p = dir.join(name);
            if p.is_file() {
                return Some(p);
            }
        }
    }
    if let Ok(path_var) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path_var) {
            for name in names {
                let p = dir.join(name);
                if p.is_file() {
                    return Some(p);
                }
            }
        }
    }
    None
}

fn piper_search_dirs(data_dir: &Path) -> Vec<PathBuf> {
    let mut dirs = vec![data_dir.join("models").join("audio")];
    if let Ok(local_app) = std::env::var("LOCALAPPDATA") {
        let base = PathBuf::from(local_app)
            .join("hermes")
            .join("cache")
            .join("piper-voices");
        dirs.push(base);
    }
    dirs
}

fn find_hermes_venv_scripts_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    let Ok(local_app) = std::env::var("LOCALAPPDATA") else {
        return dirs;
    };
    let installs_dir = PathBuf::from(local_app).join("hermes").join("installs");
    let Ok(installs) = std::fs::read_dir(installs_dir) else {
        return dirs;
    };

    for inst in installs.flatten() {
        let inst_path = inst.path();
        if !inst_path.is_dir() {
            continue;
        }
        let envs_dir = inst_path.join("environments");
        if let Ok(envs) = std::fs::read_dir(envs_dir) {
            for env_entry in envs.flatten() {
                let scripts = env_entry.path().join("venv").join("Scripts");
                if scripts.is_dir() {
                    dirs.push(scripts);
                }
            }
        }
        let pm_gens = inst_path.join("pm-runtime").join("generations");
        if let Ok(gens) = std::fs::read_dir(pm_gens) {
            for gen_entry in gens.flatten() {
                let scripts = gen_entry.path().join("Scripts");
                if scripts.is_dir() {
                    dirs.push(scripts);
                }
            }
        }
    }

    dirs.sort();
    dirs.dedup();
    dirs
}

fn find_hermes_ffmpeg_dirs(data_dir: &Path) -> Vec<PathBuf> {
    let mut dirs = vec![data_dir.join("bin"), data_dir.join("models").join("audio")];
    if let Ok(local_app) = std::env::var("LOCALAPPDATA") {
        let hermes_dir = PathBuf::from(local_app).join("hermes");
        dirs.push(hermes_dir.join("bin"));
        let tools_dir = hermes_dir.join("tools");
        if let Ok(entries) = std::fs::read_dir(tools_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_ascii_lowercase();
                if name.starts_with("ffmpeg") {
                    let p = entry.path();
                    dirs.push(p.join("bin"));
                    dirs.push(p);
                }
            }
        }
    }
    dirs.extend(find_hermes_venv_scripts_dirs());
    dirs
}

fn detect_faster_whisper_python(scripts_dirs: &[PathBuf]) -> Option<PathBuf> {
    for scripts_dir in scripts_dirs {
        let py = scripts_dir.join("python.exe");
        if !py.is_file() {
            continue;
        }
        let venv_root = scripts_dir.parent();
        let pkg_dir = venv_root.map(|r| r.join("Lib").join("site-packages").join("faster_whisper"));
        let has_pkg = pkg_dir.is_some_and(|p| p.is_dir());
        let has_ct2 = scripts_dir.join("ct2-transformers-converter.exe").is_file();
        if has_pkg || has_ct2 {
            return Some(py);
        }
    }
    None
}

fn resolve_local_faster_whisper_model_dir(data_dir: &Path) -> Option<PathBuf> {
    let audio_models_dir = data_dir.join("models").join("audio");
    if let Ok(entries) = std::fs::read_dir(&audio_models_dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() && p.join("model.bin").is_file() {
                return Some(p);
            }
        }
    }

    let profile_dir = std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)?;
    let hub_dir = profile_dir.join(".cache").join("huggingface").join("hub");
    let Ok(hub_entries) = std::fs::read_dir(hub_dir) else {
        return None;
    };
    for entry in hub_entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_ascii_lowercase();
        if !name.contains("faster-whisper") {
            continue;
        }
        let snaps_dir = entry.path().join("snapshots");
        if let Ok(snaps) = std::fs::read_dir(snaps_dir) {
            for snap in snaps.flatten() {
                let snap_path = snap.path();
                if snap_path.join("model.bin").is_file() {
                    return Some(snap_path);
                }
            }
        }
    }
    None
}

/// Normalizes short/neutral STT language tags such as `"pt"` or `"pt_br"` into canonical BCP-47 (`"pt-BR"`).
pub fn normalize_stt_bcp47_language(language: &str) -> String {
    let trimmed = language.trim();
    if trimmed.is_empty() {
        return "pt-BR".to_string();
    }
    let folded = trimmed.to_ascii_lowercase().replace('_', "-");
    if folded == "pt" || folded == "pt-br" || folded == "por" {
        "pt-BR".to_string()
    } else {
        trimmed.to_string()
    }
}

/// Inspects local audio model directories, Hermes Desktop venv scripts, and system PATH (0 MB VRAM).
pub fn discover_hermes_audio_tools(data_dir: &Path) -> HermesAudioToolchainProbe {
    let audio_models_dir = data_dir.join("models").join("audio");
    let piper_dirs = piper_search_dirs(data_dir);
    let hermes_scripts = find_hermes_venv_scripts_dirs();

    let mut search_dirs = vec![audio_models_dir.clone()];
    search_dirs.extend(hermes_scripts.iter().cloned());

    let onnx_kokoro_ready = dir_contains_matching(&audio_models_dir, &["kokoro"], ".onnx");
    let onnx_sensevoice_ready = dir_contains_matching(&audio_models_dir, &["sensevoice"], ".onnx");
    let silero_vad_ready = dir_contains_matching(&audio_models_dir, &["silero"], ".onnx");

    let piper_bin_path = find_binary_on_path_or_dirs(&["piper.exe", "piper"], &search_dirs);
    let piper_voice = piper_dirs
        .iter()
        .any(|d| dir_contains_matching(d, &["pt_br", "pt-br", "piper"], ".onnx"));
    let piper_ready = piper_bin_path.is_some() && piper_voice;

    let whisper_cli_path = find_binary_on_path_or_dirs(
        &["whisper-cli.exe", "whisper.exe", "whisper-cli", "whisper"],
        &search_dirs,
    );
    let faster_whisper_py = detect_faster_whisper_python(&hermes_scripts);
    let faster_whisper_model = resolve_local_faster_whisper_model_dir(data_dir);
    let faster_whisper_ready = faster_whisper_py.is_some() && faster_whisper_model.is_some();
    let whisper_ready = whisper_cli_path.is_some() || faster_whisper_ready;

    let edge_tts_path = find_binary_on_path_or_dirs(&["edge-tts.exe", "edge-tts"], &search_dirs);
    let sherpa_cli_path = find_binary_on_path_or_dirs(
        &[
            "sherpa-onnx-offline-tts.exe",
            "sherpa-onnx-offline-tts",
            "sherpa-onnx-cli.exe",
            "sherpa-onnx-cli",
        ],
        &search_dirs,
    );
    let ffmpeg_dirs = find_hermes_ffmpeg_dirs(data_dir);
    let ffmpeg_path = find_binary_on_path_or_dirs(&["ffmpeg.exe", "ffmpeg"], &ffmpeg_dirs);

    let windows_speech_ready = cfg!(windows);

    let active_tts_engine = if onnx_kokoro_ready {
        "kokoro_onnx_cpu".to_string()
    } else if piper_ready {
        "piper_pt_br_cpu".to_string()
    } else if edge_tts_path.is_some() && ffmpeg_path.is_some() {
        "edge_tts_neural_pt_br".to_string()
    } else if windows_speech_ready {
        "windows_sapi_pt_br".to_string()
    } else {
        "pure_rust_pcm_fallback".to_string()
    };

    let active_stt_engine = if onnx_sensevoice_ready {
        "sensevoice_onnx_cpu".to_string()
    } else if whisper_cli_path.is_some() {
        "whisper_cpp_cpu".to_string()
    } else if faster_whisper_ready {
        "faster_whisper_cpu".to_string()
    } else if faster_whisper_py.is_some() {
        "faster_whisper_needs_download".to_string()
    } else if windows_speech_ready {
        "windows_speech_recognition".to_string()
    } else {
        "offline_fallback".to_string()
    };

    let inventory = AudioEngineInventory {
        onnx_kokoro_ready,
        onnx_sensevoice_ready,
        silero_vad_ready,
        piper_ready,
        whisper_ready,
        windows_speech_ready,
        active_tts_engine,
        active_stt_engine,
    };

    HermesAudioToolchainProbe {
        inventory,
        edge_tts_bin: edge_tts_path.map(|p| p.to_string_lossy().to_string()),
        sherpa_onnx_cli_bin: sherpa_cli_path.map(|p| p.to_string_lossy().to_string()),
        faster_whisper_python: faster_whisper_py.map(|p| p.to_string_lossy().to_string()),
        faster_whisper_model_path: faster_whisper_model.map(|p| p.to_string_lossy().to_string()),
        ffmpeg_bin: ffmpeg_path.map(|p| p.to_string_lossy().to_string()),
        piper_bin: piper_bin_path.map(|p| p.to_string_lossy().to_string()),
        whisper_cli_bin: whisper_cli_path.map(|p| p.to_string_lossy().to_string()),
        hermes_venv_scripts: hermes_scripts
            .into_iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect(),
    }
}

/// Ensures local CTranslate2 `Systran/faster-whisper-base` CPU weights exist under `data_dir/models/audio`.
///
/// Uses the Hermes Desktop Python venv (`huggingface_hub::snapshot_download`) with a bounded 180s
/// subprocess timeout (`0 MB VRAM`). In test mode, avoids network downloads and returns any local hit.
pub fn provision_local_faster_whisper_model(data_dir: &Path) -> Result<Option<PathBuf>, CoreError> {
    if let Some(existing) = resolve_local_faster_whisper_model_dir(data_dir) {
        return Ok(Some(existing));
    }
    if is_test_mode() {
        return Ok(None);
    }

    let hermes_scripts = find_hermes_venv_scripts_dirs();
    let Some(py_bin) = detect_faster_whisper_python(&hermes_scripts) else {
        return Ok(None);
    };

    let target_dir = data_dir
        .join("models")
        .join("audio")
        .join("faster-whisper-base");
    std::fs::create_dir_all(&target_dir)?;

    let py_code = r#"import sys
from huggingface_hub import snapshot_download
snapshot_download(repo_id="Systran/faster-whisper-base", local_dir=sys.argv[1])
"#;
    let target_s = target_dir.to_string_lossy().to_string();
    let py_s = py_bin.to_string_lossy().to_string();
    let out = run_bounded_subprocess(&py_s, &["-c", py_code, &target_s], 180_000)?;
    if out.exit_code == Some(0) && target_dir.join("model.bin").is_file() {
        return Ok(Some(target_dir));
    }

    Err(CoreError::SubprocessError(format!(
        "Falha ao baixar Systran/faster-whisper-base (exit_code={:?})",
        out.exit_code
    )))
}

/// Probes local CPU audio models and binaries under `data_dir`, Hermes venvs, and Windows environment.
pub fn probe_audio_engines(data_dir: &Path) -> AudioEngineInventory {
    discover_hermes_audio_tools(data_dir).inventory
}

#[cfg(windows)]
fn synthesize_windows_sapi_wav(text: &str, out_wav: &Path) -> Option<Vec<u8>> {
    let escaped_out = out_wav.to_string_lossy().replace('\'', "''");
    let escaped_text = text.replace('\'', "''");
    let ps_script = format!(
        r#"Add-Type -AssemblyName System.Speech
$synth = New-Object System.Speech.Synthesis.SpeechSynthesizer
$ptVoice = $synth.GetInstalledVoices() | Where-Object {{ $_.VoiceInfo.Culture.Name -like 'pt-BR*' }} | Select-Object -First 1
if ($null -ne $ptVoice) {{
    $synth.SelectVoice($ptVoice.VoiceInfo.Name)
}}
$synth.SetOutputToWaveFile('{escaped_out}')
$synth.Speak('{escaped_text}')
$synth.Dispose()
"#
    );

    let encoded = encode_powershell_utf16le_base64(&ps_script);
    let out = run_bounded_subprocess(
        "powershell.exe",
        &["-STA", "-NoProfile", "-NonInteractive", "-EncodedCommand", &encoded],
        8000,
    )
    .ok()?;

    if out.exit_code == Some(0) && out_wav.is_file() {
        let bytes = std::fs::read(out_wav).ok()?;
        if bytes.len() >= 44 && bytes.starts_with(b"RIFF") {
            return Some(bytes);
        }
    }
    None
}

fn select_edge_tts_voice(requested_voice: Option<&str>) -> &'static str {
    let lower = requested_voice
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase();
    if lower.contains("antonio") || lower.contains("male") || lower.contains("masc") {
        "pt-BR-AntonioNeural"
    } else {
        "pt-BR-FranciscaNeural"
    }
}

fn synthesize_edge_tts_wav(
    edge_bin: &str,
    ffmpeg_bin: &str,
    text: &str,
    requested_voice: Option<&str>,
    out_wav: &Path,
) -> Option<Vec<u8>> {
    let mp3_path = out_wav.with_extension("mp3");
    let voice = select_edge_tts_voice(requested_voice);
    let mp3_str = mp3_path.to_string_lossy().to_string();
    let wav_str = out_wav.to_string_lossy().to_string();

    let synth = run_bounded_subprocess(
        edge_bin,
        &["--voice", voice, "--text", text, "--write-media", &mp3_str],
        7000,
    )
    .ok()?;
    if synth.exit_code != Some(0) || !mp3_path.is_file() {
        let _ = std::fs::remove_file(&mp3_path);
        return None;
    }

    let conv = run_bounded_subprocess(
        ffmpeg_bin,
        &["-y", "-i", &mp3_str, "-ar", "24000", "-ac", "1", &wav_str],
        5000,
    );
    let _ = std::fs::remove_file(&mp3_path);

    if matches!(conv, Ok(res) if res.exit_code == Some(0)) && out_wav.is_file() {
        let bytes = std::fs::read(out_wav).ok()?;
        if bytes.len() >= 44 && bytes.starts_with(b"RIFF") {
            return Some(bytes);
        }
    }
    None
}

/// Synthesizes Portuguese speech using CPU-only cascade
/// (Kokoro -> Piper -> Edge Neural PT-BR -> Windows SAPI PT-BR -> PCM fallback)
/// with 0 MB GPU VRAM footprint.
pub fn synthesize_speech_cascade(
    req: &VoiceSynthesisRequestDto,
    mode: CognitiveVoiceMode,
    contemplative_delay_ms: u64,
    data_dir: &Path,
) -> Result<VoiceSynthesisResultDto, CoreError> {
    let started = Instant::now();
    let effective_mode = req.cognitive_mode.unwrap_or(mode);
    let apply_shaping = req.apply_prosody_shaping.unwrap_or(true);

    let (shaped_text, sentences_chunks) = if apply_shaping {
        shape_prosodic_text_for_tts(&req.input, effective_mode)
    } else {
        let clean = req.input.trim().to_string();
        let chunks = split_prosodic_sentences(&clean);
        (clean, chunks)
    };

    let probe = discover_hermes_audio_tools(data_dir);
    let inventory = probe.inventory.clone();
    let cache_dir = data_dir.join("audio_cache");
    std::fs::create_dir_all(&cache_dir)?;

    let seq = AUDIO_SEQ.fetch_add(1, Ordering::Relaxed);
    let ts = chrono::Utc::now().timestamp_millis();
    let out_wav_path = cache_dir.join(format!("tts_{ts}_{seq}.wav"));

    let duration_ms = ((shaped_text.chars().count() as u32) * 35).clamp(180, 6000);

    let (wav_bytes, engine_tier, engine_name) = if is_test_mode() || shaped_text.is_empty() {
        let bytes = generate_pcm_wav_buffer(duration_ms, 440.0, 0.25);
        std::fs::write(&out_wav_path, &bytes)?;
        let tier = if inventory.onnx_kokoro_ready {
            AudioEngineTier::OnnxPrimary
        } else if inventory.piper_ready {
            AudioEngineTier::LocalSecondary
        } else {
            AudioEngineTier::WindowsNativeFallback
        };
        (bytes, tier, inventory.active_tts_engine)
    } else {
        let mut produced: Option<(Vec<u8>, AudioEngineTier, String)> = None;
        let models_dir = data_dir.join("models").join("audio");
        let mut extra_dirs = vec![models_dir.clone()];
        extra_dirs.extend(find_hermes_venv_scripts_dirs());

        // Tier 1: Kokoro via sherpa-onnx-offline-tts if present
        if inventory.onnx_kokoro_ready {
            if let (Some(bin), Some(model)) = (
                find_binary_on_path_or_dirs(
                    &["sherpa-onnx-offline-tts.exe", "sherpa-onnx-offline-tts"],
                    &extra_dirs,
                ),
                find_first_matching_file(&models_dir, &["kokoro"], ".onnx"),
            ) {
                let model_arg = format!("--vits-model={}", model.to_string_lossy());
                let out_arg = format!("--output-filename={}", out_wav_path.to_string_lossy());
                if let Ok(res) = run_bounded_subprocess(
                    &bin.to_string_lossy(),
                    &[&model_arg, &out_arg, &shaped_text],
                    8000,
                ) {
                    if res.exit_code == Some(0) && out_wav_path.is_file() {
                        if let Ok(bytes) = std::fs::read(&out_wav_path) {
                            if bytes.starts_with(b"RIFF") {
                                produced = Some((
                                    bytes,
                                    AudioEngineTier::OnnxPrimary,
                                    "kokoro_onnx_cpu".to_string(),
                                ));
                            }
                        }
                    }
                }
            }
        }

        // Tier 2: Piper CLI + pt_BR .onnx voice
        if produced.is_none() && inventory.piper_ready {
            let piper_dirs = piper_search_dirs(data_dir);
            let voice_opt = piper_dirs
                .iter()
                .find_map(|d| find_first_matching_file(d, &["pt_br", "pt-br", "piper"], ".onnx"));
            if let (Some(bin), Some(voice)) = (
                probe.piper_bin.as_deref(),
                voice_opt,
            ) {
                let voice_s = voice.to_string_lossy().to_string();
                let out_s = out_wav_path.to_string_lossy().to_string();
                if let Ok(res) = run_bounded_subprocess(
                    bin,
                    &["--model", &voice_s, "--output_file", &out_s, &shaped_text],
                    8000,
                ) {
                    if res.exit_code == Some(0) && out_wav_path.is_file() {
                        if let Ok(bytes) = std::fs::read(&out_wav_path) {
                            if bytes.starts_with(b"RIFF") {
                                produced = Some((
                                    bytes,
                                    AudioEngineTier::LocalSecondary,
                                    "piper_pt_br_cpu".to_string(),
                                ));
                            }
                        }
                    }
                }
            }
        }

        // Tier 3a: Edge Neural PT-BR (Hermes venv edge-tts.exe + ffmpeg.exe)
        if produced.is_none() {
            if let (Some(edge_bin), Some(ff_bin)) = (
                probe.edge_tts_bin.as_deref(),
                probe.ffmpeg_bin.as_deref(),
            ) {
                if let Some(bytes) = synthesize_edge_tts_wav(
                    edge_bin,
                    ff_bin,
                    &shaped_text,
                    req.voice.as_deref(),
                    &out_wav_path,
                ) {
                    produced = Some((
                        bytes,
                        AudioEngineTier::LocalSecondary,
                        "edge_tts_neural_pt_br".to_string(),
                    ));
                }
            }
        }

        // Tier 3b: Windows 11 SAPI offline fallback (Microsoft Maria Desktop pt-BR)
        #[cfg(windows)]
        if produced.is_none() {
            if let Some(bytes) = synthesize_windows_sapi_wav(&shaped_text, &out_wav_path) {
                produced = Some((
                    bytes,
                    AudioEngineTier::WindowsNativeFallback,
                    "windows_sapi_pt_br".to_string(),
                ));
            }
        }

        match produced {
            Some(v) => v,
            None => {
                let bytes = generate_pcm_wav_buffer(duration_ms, 440.0, 0.25);
                std::fs::write(&out_wav_path, &bytes)?;
                (
                    bytes,
                    AudioEngineTier::WindowsNativeFallback,
                    "pure_rust_pcm_fallback".to_string(),
                )
            }
        }
    };

    let audio_bytes_len = wav_bytes.len();
    let audio_base64 = standard_base64_encode(&wav_bytes);
    let latency_ms = started.elapsed().as_secs_f64() * 1000.0;

    Ok(VoiceSynthesisResultDto {
        shaped_text,
        sentences_chunks,
        audio_format: "wav".to_string(),
        audio_bytes_len,
        audio_base64,
        output_path: Some(out_wav_path.to_string_lossy().to_string()),
        contemplative_delay_ms,
        engine_tier,
        engine_name,
        vram_consumed_mb: ZERO_AUDIO_VRAM_CONSUMPTION_MB,
        latency_ms,
    })
}

fn transcribe_faster_whisper_python(
    py_bin: &str,
    wav_path: &Path,
    language: &str,
    model_path: Option<&str>,
) -> Option<String> {
    let short_lang = normalize_stt_bcp47_language(language)
        .split('-')
        .next()
        .unwrap_or("pt")
        .to_ascii_lowercase();
    let wav_str = wav_path.to_string_lossy().to_string();
    let model_arg = model_path.unwrap_or("base");

    let py_code = r#"import sys
from faster_whisper import WhisperModel
wav_path, lang, model_ref = sys.argv[1], sys.argv[2], sys.argv[3]
model = WhisperModel(model_ref, device="cpu", compute_type="int8", local_files_only=True)
segs, _ = model.transcribe(wav_path, language=lang, vad_filter=True)
txt = " ".join(s.text.strip() for s in segs if s.text).strip()
print(txt)
"#;

    let out = run_bounded_subprocess(
        py_bin,
        &["-c", py_code, &wav_str, &short_lang, model_arg],
        12000,
    )
    .ok()?;

    if out.exit_code == Some(0) {
        let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
        return Some(text);
    }
    None
}

#[cfg(windows)]
fn transcribe_windows_speech_wav(wav_path: &Path, language: &str) -> Option<String> {
    let escaped_wav = wav_path.to_string_lossy().replace('\'', "''");
    let bcp47 = normalize_stt_bcp47_language(language);
    let escaped_lang = bcp47.replace('\'', "''");
    let short_prefix = bcp47
        .split('-')
        .next()
        .unwrap_or("pt")
        .replace('\'', "''");

    let ps_script = format!(
        r#"[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
Add-Type -AssemblyName System.Speech
$installed = [System.Speech.Recognition.SpeechRecognitionEngine]::InstalledRecognizers()
if ($null -eq $installed -or $installed.Count -eq 0) {{ exit 0 }}
$chosen = $installed | Where-Object {{ $_.Culture.Name -ieq '{escaped_lang}' }} | Select-Object -First 1
if ($null -eq $chosen) {{
    $chosen = $installed | Where-Object {{ $_.Culture.Name -like '{short_prefix}*' }} | Select-Object -First 1
}}
if ($null -eq $chosen) {{
    exit 0
}}
try {{
    $rec = New-Object System.Speech.Recognition.SpeechRecognitionEngine($chosen)
    $rec.LoadGrammar((New-Object System.Speech.Recognition.DictationGrammar))
    $rec.SetInputToWaveFile('{escaped_wav}')
    $res = $rec.Recognize()
    $rec.Dispose()
    if ($null -ne $res -and -not [string]::IsNullOrWhiteSpace($res.Text)) {{
        Write-Output $res.Text
    }}
}} catch {{
}}
"#
    );

    let encoded = encode_powershell_utf16le_base64(&ps_script);
    let out = run_bounded_subprocess(
        "powershell.exe",
        &["-STA", "-NoProfile", "-NonInteractive", "-EncodedCommand", &encoded],
        8000,
    )
    .ok()?;

    if out.exit_code == Some(0) {
        let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
        return Some(text);
    }
    None
}

/// Transcribes audio bytes using CPU-only cascade
/// (SenseVoice -> whisper.cpp -> Hermes faster-whisper CPU -> Windows Speech)
/// with emotion tag parsing, silence hallucination filtering, Pcomp evaluation, and 0 MB GPU VRAM.
pub fn transcribe_audio_cascade(
    audio_bytes: &[u8],
    file_ext: &str,
    language: &str,
    silence_ms: u64,
    data_dir: &Path,
) -> Result<VoiceTranscriptionResultDto, CoreError> {
    let started = Instant::now();
    let probe = discover_hermes_audio_tools(data_dir);
    let inventory = probe.inventory.clone();
    let eff_lang = normalize_stt_bcp47_language(language);

    let (raw_text, engine_tier, engine_name) = if audio_bytes.is_empty() || is_test_mode() {
        let tier = if inventory.onnx_sensevoice_ready {
            AudioEngineTier::OnnxPrimary
        } else if inventory.whisper_ready {
            AudioEngineTier::LocalSecondary
        } else {
            AudioEngineTier::WindowsNativeFallback
        };
        (String::new(), tier, inventory.active_stt_engine)
    } else {
        let cache_dir = data_dir.join("audio_cache");
        std::fs::create_dir_all(&cache_dir)?;

        let seq = AUDIO_SEQ.fetch_add(1, Ordering::Relaxed);
        let ts = chrono::Utc::now().timestamp_millis();
        let clean_ext = file_ext.trim().trim_start_matches('.');
        let ext = if clean_ext.is_empty() { "wav" } else { clean_ext };

        let input_path = cache_dir.join(format!("stt_in_{ts}_{seq}.{ext}"));
        std::fs::write(&input_path, audio_bytes)?;

        let converted = cache_dir.join(format!("stt_conv_{ts}_{seq}.wav"));
        let wav_path = if let Some(ffmpeg) = probe.ffmpeg_bin.as_deref() {
            let in_s = input_path.to_string_lossy().to_string();
            let out_s = converted.to_string_lossy().to_string();
            let _ = run_bounded_subprocess(
                ffmpeg,
                &[
                    "-y",
                    "-i",
                    &in_s,
                    "-ar",
                    "16000",
                    "-ac",
                    "1",
                    "-sample_fmt",
                    "s16",
                    &out_s,
                ],
                8000,
            );
            if converted.is_file() {
                converted
            } else {
                input_path.clone()
            }
        } else {
            input_path.clone()
        };

        let mut transcribed: Option<(String, AudioEngineTier, String)> = None;
        let models_dir = data_dir.join("models").join("audio");
        let mut extra_dirs = vec![models_dir.clone()];
        extra_dirs.extend(find_hermes_venv_scripts_dirs());

        // Tier 1: sherpa-onnx SenseVoice
        if inventory.onnx_sensevoice_ready {
            if let (Some(bin), Some(model)) = (
                find_binary_on_path_or_dirs(
                    &["sherpa-onnx-offline.exe", "sherpa-onnx-offline"],
                    &extra_dirs,
                ),
                find_first_matching_file(&models_dir, &["sensevoice"], ".onnx"),
            ) {
                let m_arg = format!("--sense-voice-model={}", model.to_string_lossy());
                let w_s = wav_path.to_string_lossy().to_string();
                if let Ok(out) = run_bounded_subprocess(
                    &bin.to_string_lossy(),
                    &[&m_arg, &w_s],
                    8000,
                ) {
                    if out.exit_code == Some(0) {
                        let txt = String::from_utf8_lossy(&out.stdout).trim().to_string();
                        transcribed = Some((
                            txt,
                            AudioEngineTier::OnnxPrimary,
                            "sensevoice_onnx_cpu".to_string(),
                        ));
                    }
                }
            }
        }

        // Tier 2a: whisper.cpp CLI
        if transcribed.is_none() {
            if let Some(bin) = probe.whisper_cli_bin.as_deref() {
                let w_s = wav_path.to_string_lossy().to_string();
                if let Ok(out) = run_bounded_subprocess(
                    bin,
                    &["-f", &w_s, "-l", &eff_lang, "--no-timestamps"],
                    8000,
                ) {
                    if out.exit_code == Some(0) {
                        let txt = String::from_utf8_lossy(&out.stdout).trim().to_string();
                        transcribed = Some((
                            txt,
                            AudioEngineTier::LocalSecondary,
                            "whisper_cpp_cpu".to_string(),
                        ));
                    }
                }
            }
        }

        // Tier 2b: Hermes faster-whisper CPU (int8) using local cached weights
        if transcribed.is_none() {
            if let Some(py_bin) = probe.faster_whisper_python.as_deref() {
                if let Some(txt) = transcribe_faster_whisper_python(
                    py_bin,
                    &wav_path,
                    &eff_lang,
                    probe.faster_whisper_model_path.as_deref(),
                ) {
                    if !txt.is_empty() {
                        transcribed = Some((
                            txt,
                            AudioEngineTier::LocalSecondary,
                            "faster_whisper_cpu".to_string(),
                        ));
                    }
                }
            }
        }

        // Tier 3: Windows 11 SpeechRecognitionEngine (InstalledRecognizers-safe)
        #[cfg(windows)]
        if transcribed.is_none() {
            if let Some(txt) = transcribe_windows_speech_wav(&wav_path, &eff_lang) {
                transcribed = Some((
                    txt,
                    AudioEngineTier::WindowsNativeFallback,
                    "windows_speech_recognition".to_string(),
                ));
            }
        }

        let _ = std::fs::remove_file(&input_path);
        if wav_path != input_path {
            let _ = std::fs::remove_file(&wav_path);
        }

        transcribed.unwrap_or_else(|| {
            (
                String::new(),
                AudioEngineTier::WindowsNativeFallback,
                "offline_fallback".to_string(),
            )
        })
    };

    let (clean_text, emotion_tag, emotion_confidence, is_silence_hallucination_filtered) =
        parse_sensevoice_and_filter_hallucinations(&raw_text);

    let turn_evaluation = evaluate_pcomp_turn_taking(&clean_text, silence_ms, 0.25, false, 0);
    let latency_ms = started.elapsed().as_secs_f64() * 1000.0;

    Ok(VoiceTranscriptionResultDto {
        text: raw_text,
        clean_text,
        emotion_tag,
        emotion_confidence,
        is_silence_hallucination_filtered,
        turn_evaluation,
        engine_tier,
        engine_name,
        language: eff_lang,
        vram_consumed_mb: ZERO_AUDIO_VRAM_CONSUMPTION_MB,
        latency_ms,
    })
}
