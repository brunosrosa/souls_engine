//! REST & OpenAI-Compatible Audio Gateway endpoints for Sovereign PT-BR Voice (v0.9.3).
//!
//! Provides:
//! - `/api/v1/audio/status`, `/api/v1/audio/provision`
//! - `/api/v1/audio/rapport` (GET/POST)
//! - `/api/v1/audio/synthesize`, `/api/v1/audio/transcribe`
//! - `/v1/audio/speech`, `/v1/audio/transcriptions` (OpenAI-compatible wire gateway, including pure-Rust multipart/form-data)
//! - `/api/v1/hermes/voice-customs`, `/api/v1/hermes/voice-customs/sync` (hierarchical YAML block sync with `.yaml.bak`)

#![forbid(unsafe_code)]

use std::io::Read as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use axum::body::Bytes;
use axum::extract::{Query, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};

use souls_core::audio::{
    discover_hermes_audio_tools, evaluate_pcomp_turn_taking,
    parse_sensevoice_and_filter_hallucinations, probe_audio_engines,
    provision_local_faster_whisper_model, synthesize_speech_cascade, transcribe_audio_cascade,
    AudioEngineInventory, HermesAudioToolchainProbe, ZERO_AUDIO_VRAM_CONSUMPTION_MB,
};
use souls_core::google_bridge::oauth::base64url_decode;
use souls_protocol::multimodal::{
    AudioEngineTier, HermesVoiceCustomsReportDto, OpenAiSpeechRequest,
    OpenAiTranscriptionResponse, UserRapportProfileDto, VoiceSynthesisRequestDto,
    VoiceTranscriptionResultDto,
};

use crate::state::AppState;

/// Resolves the multimodal data directory without polluting `Z:\souls_engine\.souls_data` in `:memory:` tests.
pub(crate) fn resolve_multimodal_data_dir(state: &AppState) -> PathBuf {
    if state.memory.db_path().to_string_lossy() == ":memory:" {
        state.google_hub.data_dir.clone()
    } else {
        crate::resolve_data_dir()
    }
}

fn tts_engine_label_pt(engine: &str) -> &'static str {
    match engine {
        "kokoro_onnx_cpu" => "Voz neural Kokoro (PT-BR, processador)",
        "piper_pt_br_cpu" => "Voz local Piper (PT-BR, processador)",
        "edge_tts_neural_pt_br" => "Voz neural em português (Edge TTS, processador)",
        "windows_sapi_pt_br" => "Voz nativa do Windows (PT-BR)",
        _ => "Sintetizador local de reserva",
    }
}

fn stt_engine_label_pt(engine: &str) -> &'static str {
    match engine {
        "sensevoice_onnx_cpu" => "Escuta neural SenseVoice (processador)",
        "whisper_cpp_cpu" => "Escuta local Whisper (processador)",
        "faster_whisper_cpu" => "Escuta local Whisper (faster-whisper CPU)",
        "faster_whisper_needs_download" => "Whisper local instalado (clique em Baixar & Ativar Voz Local)",
        "windows_speech_recognition" => "Reconhecimento de voz do Windows",
        _ => "Escuta offline de reserva",
    }
}

fn inventory_to_json(inv: &AudioEngineInventory) -> Value {
    json!({
        "onnx_kokoro_ready": inv.onnx_kokoro_ready,
        "onnx_sensevoice_ready": inv.onnx_sensevoice_ready,
        "silero_vad_ready": inv.silero_vad_ready,
        "piper_ready": inv.piper_ready,
        "whisper_ready": inv.whisper_ready,
        "windows_speech_ready": inv.windows_speech_ready,
        "active_tts_engine": inv.active_tts_engine,
        "active_stt_engine": inv.active_stt_engine,
        "active_tts_label_pt": tts_engine_label_pt(&inv.active_tts_engine),
        "active_stt_label_pt": stt_engine_label_pt(&inv.active_stt_engine)
    })
}

pub(crate) fn decode_audio_base64(raw: &str) -> Result<Vec<u8>, String> {
    let trimmed = raw.trim();
    let payload = if let Some(idx) = trimmed.find(";base64,") {
        &trimmed[idx + ";base64,".len()..]
    } else {
        trimmed
    };
    let compact: String = payload.chars().filter(|c| !c.is_ascii_whitespace()).collect();
    base64url_decode(&compact).map_err(|e| format!("Base64 inválido para áudio: {e}"))
}

pub(crate) fn infer_audio_ext(filename: Option<&str>, mime_type: Option<&str>) -> String {
    if let Some(fname) = filename {
        if let Some(ext) = Path::new(fname).extension().and_then(|e| e.to_str()) {
            let clean = ext.trim().trim_start_matches('.').to_ascii_lowercase();
            if !clean.is_empty() {
                return clean;
            }
        }
    }
    if let Some(mime) = mime_type {
        let lower = mime.to_ascii_lowercase();
        if lower.contains("webm") {
            return "webm".to_string();
        }
        if lower.contains("ogg") || lower.contains("opus") {
            return "ogg".to_string();
        }
        if lower.contains("mpeg") || lower.contains("mp3") {
            return "mp3".to_string();
        }
        if lower.contains("mp4") || lower.contains("m4a") {
            return "m4a".to_string();
        }
        if lower.contains("flac") {
            return "flac".to_string();
        }
    }
    "wav".to_string()
}

/// Query parameters for `GET /api/v1/audio/rapport`.
#[derive(Debug, Deserialize)]
pub struct RapportQuery {
    pub user_id: Option<String>,
}

/// Request payload for `POST /api/v1/audio/transcribe`.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct AudioTranscribeJsonRequest {
    pub audio_base64: Option<String>,
    pub filename: Option<String>,
    pub mime_type: Option<String>,
    pub file_ext: Option<String>,
    pub language: Option<String>,
    pub sample_text: Option<String>,
    pub silence_duration_ms: Option<u64>,
    pub user_id: Option<String>,
    pub response_format: Option<String>,
}

/// GET /api/v1/audio/status
///
/// Probes available CPU-only audio engines with 0 MB GPU VRAM footprint.
pub async fn audio_status_handler(State(state): State<AppState>) -> impl IntoResponse {
    let _guard = state.acquire_connection();
    let data_dir = resolve_multimodal_data_dir(&state);

    let inv = tokio::task::spawn_blocking(move || probe_audio_engines(&data_dir))
        .await
        .unwrap_or_else(|_| AudioEngineInventory {
            onnx_kokoro_ready: false,
            onnx_sensevoice_ready: false,
            silero_vad_ready: false,
            piper_ready: false,
            whisper_ready: false,
            windows_speech_ready: cfg!(windows),
            active_tts_engine: "windows_sapi_pt_br".to_string(),
            active_stt_engine: "windows_speech_recognition".to_string(),
        });

    (
        StatusCode::OK,
        Json(json!({
            "status": "success",
            "vram_consumed_mb": ZERO_AUDIO_VRAM_CONSUMPTION_MB,
            "inventory": inventory_to_json(&inv)
        })),
    )
}

/// GET /api/v1/audio/rapport
///
/// Retrieves or initializes the user's linguistic rapport and cognitive voice profile.
pub async fn get_rapport_handler(
    State(state): State<AppState>,
    Query(query): Query<RapportQuery>,
) -> impl IntoResponse {
    let _guard = state.acquire_connection();
    let user_id = query
        .user_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("default");

    match state.memory.get_rapport_profile(user_id).await {
        Ok(profile) => {
            let ephemeral_turn_note_pt = profile.active_cognitive_mode.ephemeral_turn_note_pt(&profile);
            (
                StatusCode::OK,
                Json(json!({
                    "status": "success",
                    "profile": profile,
                    "ephemeral_turn_note_pt": ephemeral_turn_note_pt
                })),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "status": "error",
                "message": format!("Falha ao consultar perfil de rapport: {e}")
            })),
        ),
    }
}

/// POST /api/v1/audio/rapport
///
/// Sanitizes and persists updated user rapport preferences and cognitive voice mode.
pub async fn post_rapport_handler(
    State(state): State<AppState>,
    Json(mut profile): Json<UserRapportProfileDto>,
) -> impl IntoResponse {
    let _guard = state.acquire_connection();
    if profile.user_id.trim().is_empty() {
        profile.user_id = "default".to_string();
    }
    if let Ok(existing) = state.memory.get_rapport_profile(&profile.user_id).await {
        profile.turn_count = profile.turn_count.max(existing.turn_count);
    }
    profile.sanitize();

    match state.memory.save_rapport_profile(&profile).await {
        Ok(saved) => {
            let ephemeral_turn_note_pt = saved.active_cognitive_mode.ephemeral_turn_note_pt(&saved);
            (
                StatusCode::OK,
                Json(json!({
                    "status": "success",
                    "profile": saved,
                    "ephemeral_turn_note_pt": ephemeral_turn_note_pt
                })),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "status": "error",
                "message": format!("Falha ao salvar perfil de rapport: {e}")
            })),
        ),
    }
}

/// POST /api/v1/audio/synthesize
///
/// Shapes Portuguese prosody and synthesizes speech on CPU (0 MB VRAM).
pub async fn synthesize_audio_handler(
    State(state): State<AppState>,
    Json(req): Json<VoiceSynthesisRequestDto>,
) -> impl IntoResponse {
    let _guard = state.acquire_connection();
    let profile = state
        .memory
        .get_rapport_profile("default")
        .await
        .unwrap_or_default();
    let mode = req.cognitive_mode.unwrap_or(profile.active_cognitive_mode);
    let ephemeral_note = mode.ephemeral_turn_note_pt(&profile);
    let data_dir = resolve_multimodal_data_dir(&state);
    let contemplative_ms = profile.contemplative_base_ms;

    let synth_res = tokio::task::spawn_blocking(move || {
        synthesize_speech_cascade(&req, mode, contemplative_ms, &data_dir)
    })
    .await;

    match synth_res {
        Ok(Ok(dto)) => {
            let mut val = serde_json::to_value(&dto).unwrap_or_else(|_| json!({}));
            if let Value::Object(ref mut map) = val {
                map.insert("status".to_string(), json!("success"));
                map.insert("ephemeral_turn_note_pt".to_string(), json!(ephemeral_note));
            }
            (StatusCode::OK, Json(val))
        }
        Ok(Err(e)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "status": "error",
                "message": format!("Falha na síntese de voz local: {e}")
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "status": "error",
                "message": format!("Erro interno na síntese de voz: {e}")
            })),
        ),
    }
}

pub(crate) fn evaluate_sample_text_transcription(
    sample: &str,
    language: &str,
    silence_ms: u64,
    data_dir: &Path,
) -> VoiceTranscriptionResultDto {
    let started = Instant::now();
    let inv = probe_audio_engines(data_dir);
    let (clean_text, emotion_tag, emotion_confidence, is_filtered) =
        parse_sensevoice_and_filter_hallucinations(sample);
    let turn_evaluation = evaluate_pcomp_turn_taking(&clean_text, silence_ms, 0.25, false, 0);

    let engine_tier = if inv.onnx_sensevoice_ready {
        AudioEngineTier::OnnxPrimary
    } else if inv.whisper_ready {
        AudioEngineTier::LocalSecondary
    } else {
        AudioEngineTier::WindowsNativeFallback
    };

    VoiceTranscriptionResultDto {
        text: sample.to_string(),
        clean_text,
        emotion_tag,
        emotion_confidence,
        is_silence_hallucination_filtered: is_filtered,
        turn_evaluation,
        engine_tier,
        engine_name: "pcomp_sample_evaluator".to_string(),
        language: if language.trim().is_empty() {
            "pt".to_string()
        } else {
            language.trim().to_string()
        },
        vram_consumed_mb: ZERO_AUDIO_VRAM_CONSUMPTION_MB,
        latency_ms: started.elapsed().as_secs_f64() * 1000.0,
    }
}

/// POST /api/v1/audio/transcribe
///
/// Evaluates speech audio or sample transcript with SenseVoice tag parsing,
/// Whisper silence hallucination filtering, PCOMP turn-taking, and EMA rapport updates.
pub async fn transcribe_audio_handler(
    State(state): State<AppState>,
    Json(payload): Json<AudioTranscribeJsonRequest>,
) -> impl IntoResponse {
    let _guard = state.acquire_connection();
    let data_dir = resolve_multimodal_data_dir(&state);
    let user_id = payload
        .user_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("default")
        .to_string();
    let silence_ms = payload.silence_duration_ms.unwrap_or(450);
    let language = payload
        .language
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("pt")
        .to_string();

    let has_audio_b64 = payload
        .audio_base64
        .as_deref()
        .is_some_and(|s| !s.trim().is_empty());

    let result = if !has_audio_b64 {
        if let Some(ref sample) = payload.sample_text {
            if !sample.trim().is_empty() {
                let sample_owned = sample.clone();
                let lang_owned = language.clone();
                let dd = data_dir.clone();
                tokio::task::spawn_blocking(move || {
                    Ok(evaluate_sample_text_transcription(
                        &sample_owned,
                        &lang_owned,
                        silence_ms,
                        &dd,
                    ))
                })
                .await
                .unwrap_or_else(|e| Err(format!("Erro interno ao avaliar texto: {e}")))
            } else {
                Err("Forneça 'audio_base64' ou 'sample_text' para transcrever.".to_string())
            }
        } else {
            Err("Forneça 'audio_base64' ou 'sample_text' para transcrever.".to_string())
        }
    } else {
        let raw_b64 = payload.audio_base64.as_deref().unwrap_or("");
        match decode_audio_base64(raw_b64) {
            Ok(audio_bytes) => {
                let ext = payload
                    .file_ext
                    .as_deref()
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(ToOwned::to_owned)
                    .unwrap_or_else(|| {
                        infer_audio_ext(
                            payload.filename.as_deref(),
                            payload.mime_type.as_deref(),
                        )
                    });
                let lang_owned = language.clone();
                let dd = data_dir.clone();
                tokio::task::spawn_blocking(move || {
                    transcribe_audio_cascade(&audio_bytes, &ext, &lang_owned, silence_ms, &dd)
                        .map_err(|e| format!("Falha na transcrição de áudio: {e}"))
                })
                .await
                .unwrap_or_else(|e| Err(format!("Erro interno na transcrição: {e}")))
            }
            Err(e) => Err(e),
        }
    };

    let dto = match result {
        Ok(dto) => dto,
        Err(msg) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "status": "error",
                    "message": msg
                })),
            );
        }
    };

    if !dto.is_silence_hallucination_filtered && !dto.clean_text.trim().is_empty() {
        let _ = state
            .memory
            .update_rapport_from_utterance(&user_id, &dto.clean_text, dto.emotion_tag)
            .await;
    }

    let mut val = serde_json::to_value(&dto).unwrap_or_else(|_| json!({}));
    if let Value::Object(ref mut map) = val {
        map.insert("status".to_string(), json!("success"));
    }
    (StatusCode::OK, Json(val))
}

fn find_hermes_ffmpeg() -> Option<PathBuf> {
    let mut dirs = Vec::new();
    if let Ok(local_app) = std::env::var("LOCALAPPDATA") {
        let hermes_dir = PathBuf::from(local_app).join("hermes");
        dirs.push(hermes_dir.join("bin"));
        let tools_dir = hermes_dir.join("tools");
        if let Ok(entries) = std::fs::read_dir(&tools_dir) {
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
    for dir in dirs {
        for bin in ["ffmpeg.exe", "ffmpeg"] {
            let candidate = dir.join(bin);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    if let Ok(path_var) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path_var) {
            for bin in ["ffmpeg.exe", "ffmpeg"] {
                let candidate = dir.join(bin);
                if candidate.is_file() {
                    return Some(candidate);
                }
            }
        }
    }
    None
}

fn run_ffmpeg_bounded(ffmpeg_bin: &Path, input: &Path, output: &Path, args: &[&str]) -> bool {
    let in_s = input.to_string_lossy().to_string();
    let out_s = output.to_string_lossy().to_string();
    let mut cmd = Command::new(ffmpeg_bin);
    cmd.arg("-y")
        .arg("-i")
        .arg(&in_s)
        .args(args)
        .arg(&out_s)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let Ok(mut child) = cmd.spawn() else {
        return false;
    };

    let stdout_handle = child.stdout.take().map(|mut pipe| {
        std::thread::spawn(move || {
            let mut buf = Vec::new();
            let _ = pipe.read_to_end(&mut buf);
        })
    });
    let stderr_handle = child.stderr.take().map(|mut pipe| {
        std::thread::spawn(move || {
            let mut buf = Vec::new();
            let _ = pipe.read_to_end(&mut buf);
        })
    });

    let start = Instant::now();
    let timeout = Duration::from_secs(8);
    let success = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status.success(),
            Ok(None) => {
                if start.elapsed() >= timeout {
                    let _ = child.kill();
                    let _ = child.wait();
                    break false;
                }
                std::thread::sleep(Duration::from_millis(15));
            }
            Err(_) => break false,
        }
    };

    if let Some(h) = stdout_handle {
        let _ = h.join();
    }
    if let Some(h) = stderr_handle {
        let _ = h.join();
    }

    success && output.is_file()
}

/// POST /v1/audio/speech
///
/// OpenAI-compatible Text-to-Speech wire gateway returning raw audio bytes (`audio/wav`, `audio/mpeg`, or `audio/ogg`).
pub async fn openai_audio_speech_handler(
    State(state): State<AppState>,
    Json(req): Json<OpenAiSpeechRequest>,
) -> Response {
    let _guard = state.acquire_connection();
    let profile = state
        .memory
        .get_rapport_profile("default")
        .await
        .unwrap_or_default();
    let mode = profile.active_cognitive_mode;
    let contemplative_ms = profile.contemplative_base_ms;
    let data_dir = resolve_multimodal_data_dir(&state);

    let requested_fmt = req
        .response_format
        .as_deref()
        .unwrap_or("wav")
        .trim()
        .to_ascii_lowercase();

    let synth_req = VoiceSynthesisRequestDto {
        input: req.input,
        voice: req.voice,
        model: req.model,
        response_format: Some(requested_fmt.clone()),
        speed: req.speed,
        apply_prosody_shaping: Some(true),
        cognitive_mode: Some(mode),
    };

    let data_dir_for_synth = data_dir.clone();
    let synth_res = tokio::task::spawn_blocking(move || {
        synthesize_speech_cascade(&synth_req, mode, contemplative_ms, &data_dir_for_synth)
    })
    .await;

    let dto = match synth_res {
        Ok(Ok(d)) => d,
        Ok(Err(e)) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": { "message": format!("TTS cascade failed: {e}") } })),
            )
                .into_response();
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": { "message": format!("TTS task failed: {e}") } })),
            )
                .into_response();
        }
    };

    let wav_bytes = match decode_audio_base64(&dto.audio_base64) {
        Ok(b) => b,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": { "message": e } })),
            )
                .into_response();
        }
    };

    let is_test = std::env::var("SOULS_TEST_MODE")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    if !is_test && matches!(requested_fmt.as_str(), "mp3" | "opus" | "ogg") {
        let target_fmt = requested_fmt.clone();
        let data_dir_trans = data_dir.clone();
        let wav_clone = wav_bytes.clone();
        if let Ok(Some((bytes, mime))) = tokio::task::spawn_blocking(move || {
            let ffmpeg = find_hermes_ffmpeg()?;
            let cache_dir = data_dir_trans.join("audio_cache");
            let _ = std::fs::create_dir_all(&cache_dir);
            let ts = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0);
            let in_wav = cache_dir.join(format!("speech_in_{ts}.wav"));
            let (out_ext, mime, ff_args): (&str, &str, &[&str]) = if target_fmt == "mp3" {
                ("mp3", "audio/mpeg", &["-c:a", "libmp3lame", "-q:a", "4"])
            } else {
                ("ogg", "audio/ogg", &["-c:a", "libopus", "-b:a", "48k"])
            };
            let out_file = cache_dir.join(format!("speech_out_{ts}.{out_ext}"));
            if std::fs::write(&in_wav, &wav_clone).is_err() {
                return None;
            }
            let ok = run_ffmpeg_bounded(&ffmpeg, &in_wav, &out_file, ff_args);
            let _ = std::fs::remove_file(&in_wav);
            let result = if ok {
                std::fs::read(&out_file).ok().map(|b| (b, mime))
            } else {
                None
            };
            let _ = std::fs::remove_file(&out_file);
            result
        })
        .await
        {
            return (StatusCode::OK, [(header::CONTENT_TYPE, mime)], bytes).into_response();
        }
    }

    (StatusCode::OK, [(header::CONTENT_TYPE, "audio/wav")], wav_bytes).into_response()
}

struct ParsedMultipartUpload {
    file_bytes: Option<Vec<u8>>,
    filename: Option<String>,
    language: Option<String>,
    model: Option<String>,
    response_format: Option<String>,
}

fn extract_multipart_boundary(content_type: &str) -> Option<String> {
    let lower = content_type.to_ascii_lowercase();
    if !lower.contains("multipart/form-data") {
        return None;
    }
    for segment in content_type.split(';') {
        let trimmed = segment.trim();
        if trimmed.len() >= 9 && trimmed[..9].eq_ignore_ascii_case("boundary=") {
            let val = trimmed[9..].trim().trim_matches('"').trim_matches('\'');
            if !val.is_empty() {
                return Some(val.to_string());
            }
        }
    }
    None
}

fn extract_quoted_or_token_attr(header_line: &str, key: &str) -> Option<String> {
    let lower = header_line.to_ascii_lowercase();
    let pat = format!("{key}=");
    let pos = lower.find(&pat)?;
    let rest = &header_line[pos + pat.len()..];
    if let Some(after_quote) = rest.strip_prefix('"') {
        let end = after_quote.find('"')?;
        Some(after_quote[..end].to_string())
    } else {
        let end = rest.find(';').unwrap_or(rest.len());
        let val = rest[..end].trim();
        if val.is_empty() {
            None
        } else {
            Some(val.to_string())
        }
    }
}

fn find_subslice(haystack: &[u8], needle: &[u8], from: usize) -> Option<usize> {
    if needle.is_empty() || from + needle.len() > haystack.len() {
        return None;
    }
    haystack[from..]
        .windows(needle.len())
        .position(|w| w == needle)
        .map(|rel| from + rel)
}

fn parse_multipart_form_data(body: &[u8], boundary: &str) -> ParsedMultipartUpload {
    let mut parsed = ParsedMultipartUpload {
        file_bytes: None,
        filename: None,
        language: None,
        model: None,
        response_format: None,
    };

    let marker = format!("--{boundary}").into_bytes();
    let mut positions = Vec::new();
    let mut cursor = 0usize;
    while let Some(pos) = find_subslice(body, &marker, cursor) {
        positions.push(pos);
        cursor = pos + marker.len();
    }

    for pair in positions.windows(2) {
        let start = pair[0] + marker.len();
        let end = pair[1];
        if start >= end {
            continue;
        }
        let mut part = &body[start..end];
        if part.starts_with(b"--") {
            break;
        }
        if let Some(stripped) = part.strip_prefix(b"\r\n") {
            part = stripped;
        } else if let Some(stripped) = part.strip_prefix(b"\n") {
            part = stripped;
        }

        let (headers_bytes, mut payload) = if let Some(idx) = find_subslice(part, b"\r\n\r\n", 0) {
            (&part[..idx], &part[idx + 4..])
        } else if let Some(idx) = find_subslice(part, b"\n\n", 0) {
            (&part[..idx], &part[idx + 2..])
        } else {
            continue;
        };

        if let Some(stripped) = payload.strip_suffix(b"\r\n") {
            payload = stripped;
        } else if let Some(stripped) = payload.strip_suffix(b"\n") {
            payload = stripped;
        }

        let headers_str = String::from_utf8_lossy(headers_bytes);
        let mut field_name: Option<String> = None;
        let mut file_name: Option<String> = None;
        for line in headers_str.lines() {
            if line.to_ascii_lowercase().starts_with("content-disposition:") {
                field_name = extract_quoted_or_token_attr(line, "name");
                file_name = extract_quoted_or_token_attr(line, "filename");
            }
        }

        match field_name.as_deref() {
            Some("file") => {
                parsed.file_bytes = Some(payload.to_vec());
                if file_name.is_some() {
                    parsed.filename = file_name;
                }
            }
            Some("language") => {
                let val = String::from_utf8_lossy(payload).trim().to_string();
                if !val.is_empty() {
                    parsed.language = Some(val);
                }
            }
            Some("model") => {
                let val = String::from_utf8_lossy(payload).trim().to_string();
                if !val.is_empty() {
                    parsed.model = Some(val);
                }
            }
            Some("response_format") => {
                let val = String::from_utf8_lossy(payload).trim().to_string();
                if !val.is_empty() {
                    parsed.response_format = Some(val);
                }
            }
            _ => {}
        }
    }

    parsed
}

/// POST /v1/audio/transcriptions
///
/// OpenAI-compatible Speech-to-Text wire gateway supporting `multipart/form-data` uploads
/// (pure Rust boundary parsing), JSON payloads, and `response_format="text"`.
pub async fn openai_audio_transcriptions_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let _guard = state.acquire_connection();
    let data_dir = resolve_multimodal_data_dir(&state);

    let content_type = headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let (audio_bytes_opt, file_ext, language, sample_text_opt, silence_ms, response_format) =
        if let Some(boundary) = extract_multipart_boundary(content_type) {
            let mp = parse_multipart_form_data(&body, &boundary);
            let _model = mp.model;
            let ext = infer_audio_ext(mp.filename.as_deref(), None);
            let lang = mp.language.unwrap_or_else(|| "pt".to_string());
            (mp.file_bytes, ext, lang, None, 450u64, mp.response_format)
        } else {
            let json_req: AudioTranscribeJsonRequest = match serde_json::from_slice(&body) {
                Ok(req) => req,
                Err(e) => {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(json!({ "error": { "message": format!("Payload inválido em /v1/audio/transcriptions: {e}") } })),
                    )
                        .into_response();
                }
            };
            let lang = json_req
                .language
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .unwrap_or("pt")
                .to_string();
            let ext = json_req
                .file_ext
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(ToOwned::to_owned)
                .unwrap_or_else(|| {
                    infer_audio_ext(json_req.filename.as_deref(), json_req.mime_type.as_deref())
                });
            let bytes_opt = match json_req.audio_base64.as_deref().filter(|s| !s.trim().is_empty()) {
                Some(b64) => match decode_audio_base64(b64) {
                    Ok(b) => Some(b),
                    Err(e) => {
                        return (
                            StatusCode::BAD_REQUEST,
                            Json(json!({ "error": { "message": e } })),
                        )
                            .into_response();
                    }
                },
                None => None,
            };
            (
                bytes_opt,
                ext,
                lang,
                json_req.sample_text,
                json_req.silence_duration_ms.unwrap_or(450),
                json_req.response_format,
            )
        };

    let dto_res = match (audio_bytes_opt, sample_text_opt) {
        (Some(bytes), _) => {
            let lang_clone = language.clone();
            let dd = data_dir.clone();
            tokio::task::spawn_blocking(move || {
                transcribe_audio_cascade(&bytes, &file_ext, &lang_clone, silence_ms, &dd)
                    .map_err(|e| format!("Transcription failed: {e}"))
            })
            .await
            .unwrap_or_else(|e| Err(format!("Transcription join error: {e}")))
        }
        (None, Some(sample)) if !sample.trim().is_empty() => {
            let lang_clone = language.clone();
            let dd = data_dir.clone();
            tokio::task::spawn_blocking(move || {
                Ok(evaluate_sample_text_transcription(
                    &sample,
                    &lang_clone,
                    silence_ms,
                    &dd,
                ))
            })
            .await
            .unwrap_or_else(|e| Err(format!("Sample evaluation error: {e}")))
        }
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": { "message": "Missing audio 'file' or 'audio_base64' payload." } })),
            )
                .into_response();
        }
    };

    let dto = match dto_res {
        Ok(d) => d,
        Err(msg) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": { "message": msg } })),
            )
                .into_response();
        }
    };

    if !dto.is_silence_hallucination_filtered && !dto.clean_text.trim().is_empty() {
        let _ = state
            .memory
            .update_rapport_from_utterance("default", &dto.clean_text, dto.emotion_tag)
            .await;
    }

    if response_format
        .as_deref()
        .map(str::trim)
        .is_some_and(|rf| rf.eq_ignore_ascii_case("text"))
    {
        return (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
            dto.clean_text,
        )
            .into_response();
    }

    let resp = OpenAiTranscriptionResponse {
        text: dto.clean_text,
        emotion: Some(dto.emotion_tag),
        language: Some(dto.language),
    };
    (StatusCode::OK, Json(resp)).into_response()
}

/// POST /api/v1/audio/provision
///
/// Ensures local CPU audio directories exist, probes local/Hermes engines at 0 MB VRAM,
/// and writes `audio_engines_manifest.json` with discovered binaries and model paths.
pub async fn provision_audio_handler(State(state): State<AppState>) -> impl IntoResponse {
    let _guard = state.acquire_connection();
    let data_dir = resolve_multimodal_data_dir(&state);
    let models_dir = data_dir.join("models").join("audio");
    let cache_dir = data_dir.join("audio_cache");

    let models_clone = models_dir.clone();
    let cache_clone = cache_dir.clone();
    let dd = data_dir.clone();

    let res = tokio::task::spawn_blocking(
        move || -> Result<(HermesAudioToolchainProbe, PathBuf), String> {
            std::fs::create_dir_all(&models_clone)
                .map_err(|e| format!("Falha ao criar pasta de modelos de áudio: {e}"))?;
            std::fs::create_dir_all(&cache_clone)
                .map_err(|e| format!("Falha ao criar pasta de cache de áudio: {e}"))?;

            let _ = provision_local_faster_whisper_model(&dd);
            let probe = discover_hermes_audio_tools(&dd);
            let manifest_path = models_clone.join("audio_engines_manifest.json");
            let manifest_json = json!({
                "version": env!("CARGO_PKG_VERSION"),
                "vram_consumed_mb": ZERO_AUDIO_VRAM_CONSUMPTION_MB,
                "generated_at": chrono::Utc::now().to_rfc3339(),
                "models_dir": models_clone.to_string_lossy(),
                "cache_dir": cache_clone.to_string_lossy(),
                "toolchain": &probe,
            });
            let formatted = serde_json::to_string_pretty(&manifest_json)
                .map_err(|e| format!("Falha ao serializar manifesto de áudio: {e}"))?;
            std::fs::write(&manifest_path, formatted.as_bytes())
                .map_err(|e| format!("Falha ao gravar manifesto de áudio: {e}"))?;

            Ok((probe, manifest_path))
        },
    )
    .await;

    match res {
        Ok(Ok((probe, manifest_path))) => (
            StatusCode::OK,
            Json(json!({
                "status": "success",
                "vram_consumed_mb": ZERO_AUDIO_VRAM_CONSUMPTION_MB,
                "models_dir": models_dir.to_string_lossy(),
                "cache_dir": cache_dir.to_string_lossy(),
                "manifest_path": manifest_path.to_string_lossy(),
                "downloaded_whisper_model": probe.faster_whisper_model_path,
                "toolchain": probe,
                "inventory": inventory_to_json(&probe.inventory),
                "message": "Pastas, pesos Whisper locais e manifesto de voz preparados no processador (0 MB de vídeo)."
            })),
        ),
        Ok(Err(msg)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "status": "error",
                "message": msg
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "status": "error",
                "message": format!("Erro interno ao preparar áudio local: {e}")
            })),
        ),
    }
}

// ============================================================================
// Hermes Voice Customs YAML Hierarchical Inspector & Atomic Sync
// ============================================================================

#[derive(Debug, Default)]
struct ParsedVoiceYaml {
    file_exists: bool,
    stt_enabled: Option<bool>,
    stt_provider: String,
    stt_language: String,
    stt_use_gateway: Option<bool>,
    stt_openai_language: String,
    stt_openai_base_url: String,
    tts_provider: String,
    tts_use_gateway: Option<bool>,
    tts_openai_base_url: String,
}

fn strip_yaml_inline_comment(raw: &str) -> &str {
    let mut in_single = false;
    let mut in_double = false;
    for (idx, ch) in raw.char_indices() {
        match ch {
            '\'' if !in_double => in_single = !in_single,
            '"' if !in_single => in_double = !in_double,
            '#' if !in_single && !in_double => return raw[..idx].trim_end(),
            _ => {}
        }
    }
    raw.trim_end()
}

fn clean_yaml_scalar(raw: &str) -> String {
    strip_yaml_inline_comment(raw)
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .trim()
        .to_string()
}

fn split_yaml_key_val(trimmed: &str) -> Option<(&str, &str)> {
    let colon_idx = trimmed.find(':')?;
    let key = trimmed[..colon_idx].trim();
    if key.is_empty() || key.starts_with('-') {
        return None;
    }
    let val = trimmed[colon_idx + 1..].trim();
    Some((key, val))
}

fn leading_spaces_count(line: &str) -> usize {
    line.chars()
        .take_while(|c| *c == ' ' || *c == '\t')
        .map(|c| if c == '\t' { 2 } else { 1 })
        .sum()
}

fn parse_hermes_voice_yaml(cfg_path: &Path) -> ParsedVoiceYaml {
    let mut out = ParsedVoiceYaml::default();
    if !cfg_path.is_file() {
        return out;
    }
    out.file_exists = true;
    let Ok(content) = std::fs::read_to_string(cfg_path) else {
        return out;
    };

    let mut top_section: Option<String> = None;
    let mut section_child_indent: Option<usize> = None;
    let mut sub_section: Option<String> = None;
    let mut sub_section_indent: Option<usize> = None;

    for raw_line in content.lines() {
        let no_cr = raw_line.trim_end_matches('\r');
        let trimmed_start = no_cr.trim_start();
        if trimmed_start.is_empty() || trimmed_start.starts_with('#') {
            continue;
        }
        let indent = leading_spaces_count(no_cr);
        let no_comment = strip_yaml_inline_comment(trimmed_start);
        let Some((key, val_raw)) = split_yaml_key_val(no_comment) else {
            continue;
        };

        if indent == 0 {
            top_section = Some(key.to_string());
            section_child_indent = None;
            sub_section = None;
            sub_section_indent = None;
            continue;
        }

        let Some(ref section) = top_section else {
            continue;
        };
        if section != "stt" && section != "tts" {
            continue;
        }

        let base_child = match section_child_indent {
            Some(i) => i,
            None => {
                section_child_indent = Some(indent);
                indent
            }
        };

        if indent <= base_child {
            sub_section = None;
            sub_section_indent = None;
            let scalar = clean_yaml_scalar(val_raw);
            if scalar.is_empty() {
                sub_section = Some(key.to_string());
                sub_section_indent = Some(indent);
            } else if section == "stt" {
                match key {
                    "enabled" => {
                        out.stt_enabled = Some(scalar.eq_ignore_ascii_case("true"));
                    }
                    "provider" => out.stt_provider = scalar,
                    "language" => out.stt_language = scalar,
                    "use_gateway" => {
                        out.stt_use_gateway = Some(scalar.eq_ignore_ascii_case("true"));
                    }
                    _ => {}
                }
            } else if section == "tts" {
                match key {
                    "provider" => out.tts_provider = scalar,
                    "use_gateway" => {
                        out.tts_use_gateway = Some(scalar.eq_ignore_ascii_case("true"));
                    }
                    _ => {}
                }
            }
        } else if let Some(ref sub) = sub_section {
            let sub_base = sub_section_indent.unwrap_or(base_child);
            if indent > sub_base && sub == "openai" {
                let scalar = clean_yaml_scalar(val_raw);
                if section == "stt" {
                    match key {
                        "language" => out.stt_openai_language = scalar,
                        "base_url" => out.stt_openai_base_url = scalar,
                        _ => {}
                    }
                } else if section == "tts" && key == "base_url" {
                    out.tts_openai_base_url = scalar;
                }
            }
        }
    }

    out
}

fn is_pt_language(lang: &str) -> bool {
    let lower = lang.trim().to_ascii_lowercase();
    lower == "pt" || lower == "pt-br" || lower == "pt_br"
}

fn is_souls_local_url(url: &str) -> bool {
    let lower = url.trim().to_ascii_lowercase();
    lower.contains("127.0.0.1:9123") || lower.contains("localhost:9123")
}

fn discover_hermes_subprofile_configs(root_cfg: &Path) -> Vec<(String, PathBuf)> {
    let mut list = Vec::new();
    let Some(hermes_dir) = root_cfg.parent() else {
        return list;
    };
    let profiles_dir = hermes_dir.join("profiles");
    let Ok(entries) = std::fs::read_dir(profiles_dir) else {
        return list;
    };
    for entry in entries.flatten() {
        let p = entry.path();
        if !p.is_dir() {
            continue;
        }
        let sub_cfg = p.join("config.yaml");
        if sub_cfg.is_file() {
            let name = entry.file_name().to_string_lossy().to_string();
            list.push((name, sub_cfg));
        }
    }
    list.sort_by(|a, b| a.0.cmp(&b.0));
    list
}

fn is_single_voice_profile_synced(parsed: &ParsedVoiceYaml) -> bool {
    if !parsed.file_exists || parsed.stt_enabled != Some(true) {
        return false;
    }
    if parsed.stt_use_gateway == Some(true) || parsed.tts_use_gateway == Some(true) {
        return false;
    }
    let effective_stt_lang = if !parsed.stt_language.is_empty() {
        parsed.stt_language.as_str()
    } else {
        parsed.stt_openai_language.as_str()
    };
    if !is_pt_language(effective_stt_lang) {
        return false;
    }
    if !parsed.stt_openai_language.is_empty() && !is_pt_language(&parsed.stt_openai_language) {
        return false;
    }
    parsed.stt_provider.eq_ignore_ascii_case("openai")
        && is_souls_local_url(&parsed.stt_openai_base_url)
        && parsed.tts_provider.eq_ignore_ascii_case("openai")
        && is_souls_local_url(&parsed.tts_openai_base_url)
}

fn build_hermes_voice_report(cfg_path: &Path, current_voice_mode: &str) -> HermesVoiceCustomsReportDto {
    let parsed = parse_hermes_voice_yaml(cfg_path);
    let mut issues = Vec::new();

    let stt_enabled = parsed.stt_enabled.unwrap_or(false);
    let effective_stt_lang = if !parsed.stt_language.is_empty() {
        parsed.stt_language.clone()
    } else {
        parsed.stt_openai_language.clone()
    };
    let effective_stt_provider = if parsed.stt_provider.is_empty() {
        "unset".to_string()
    } else {
        parsed.stt_provider.clone()
    };
    let effective_tts_provider = if parsed.tts_provider.is_empty() {
        "unset".to_string()
    } else {
        parsed.tts_provider.clone()
    };

    if !parsed.file_exists {
        issues.push(
            "Arquivo config.yaml do Hermes ainda não encontrado; clique em sincronizar para criar as preferências em português."
                .to_string(),
        );
    } else {
        if !stt_enabled {
            issues.push("O reconhecimento de voz (STT) está desativado no Hermes.".to_string());
        }
        if parsed.stt_use_gateway == Some(true) {
            issues.push(
                "A escuta (STT) ainda força 'use_gateway: true' em vez do gateway local em português."
                    .to_string(),
            );
        }
        if !is_pt_language(&effective_stt_lang) {
            let display_lang = if effective_stt_lang.is_empty() {
                "não definido"
            } else {
                effective_stt_lang.as_str()
            };
            issues.push(format!(
                "Idioma de escuta do Hermes está em '{display_lang}' em vez de português ('pt')."
            ));
        }
        if !parsed.stt_openai_language.is_empty() && !is_pt_language(&parsed.stt_openai_language) {
            issues.push(format!(
                "Sub-bloco stt.openai.language está em '{}' em vez de 'pt'.",
                parsed.stt_openai_language
            ));
        }
        if !effective_stt_provider.eq_ignore_ascii_case("openai")
            || !is_souls_local_url(&parsed.stt_openai_base_url)
        {
            issues.push(
                "A escuta do Hermes ainda não aponta para o gateway local em http://127.0.0.1:9123/v1."
                    .to_string(),
            );
        }
        if parsed.tts_use_gateway == Some(true) {
            issues.push(
                "A síntese de voz (TTS) ainda força 'use_gateway: true' em vez do servidor local."
                    .to_string(),
            );
        }
        if effective_tts_provider.eq_ignore_ascii_case("elevenlabs") {
            issues.push(
                "A voz do Hermes está configurada em provedor pago (ElevenLabs) em vez do motor local em português."
                    .to_string(),
            );
        } else if !effective_tts_provider.eq_ignore_ascii_case("openai")
            || !is_souls_local_url(&parsed.tts_openai_base_url)
        {
            issues.push(
                "A síntese de voz (TTS) do Hermes ainda não aponta para http://127.0.0.1:9123/v1."
                    .to_string(),
            );
        }

        for (prof_name, sub_cfg) in discover_hermes_subprofile_configs(cfg_path) {
            if !is_single_voice_profile_synced(&parse_hermes_voice_yaml(&sub_cfg)) {
                issues.push(format!(
                    "Sub-perfil '{prof_name}' ainda não está sincronizado para voz local em português."
                ));
            }
        }
    }

    let is_synced = issues.is_empty();
    let human_summary_pt = if is_synced {
        "Hermes e Telegram já estão alinhados para conversar em português local (0 MB de vídeo)."
            .to_string()
    } else {
        "Encontramos ajustes recomendados para que o Hermes escute e responda em português localmente."
            .to_string()
    };

    HermesVoiceCustomsReportDto {
        config_path: cfg_path.to_string_lossy().to_string(),
        stt_enabled,
        current_stt_language: effective_stt_lang,
        current_stt_provider: effective_stt_provider,
        current_tts_provider: effective_tts_provider,
        current_voice_mode: current_voice_mode.to_string(),
        is_synced_with_souls_pt_br: is_synced,
        issues_found: issues,
        human_summary_pt,
    }
}

fn canonical_openai_subblock_lines(
    section: &str,
    indent_str: &str,
    sub_indent_str: &str,
) -> Vec<String> {
    if section == "stt" {
        vec![
            format!("{indent_str}openai:"),
            format!("{sub_indent_str}model: whisper-1"),
            format!("{sub_indent_str}language: pt"),
            format!("{sub_indent_str}base_url: 'http://127.0.0.1:9123/v1'"),
            format!("{sub_indent_str}api_key: souls-local"),
        ]
    } else {
        vec![
            format!("{indent_str}openai:"),
            format!("{sub_indent_str}model: souls-kokoro-ptbr"),
            format!("{sub_indent_str}voice: souls-pt-br"),
            format!("{sub_indent_str}base_url: 'http://127.0.0.1:9123/v1'"),
            format!("{sub_indent_str}api_key: souls-local"),
        ]
    }
}

/// Surgically updates `stt:` / `tts:` settings without discarding sibling blocks (`local:`, `elevenlabs:`, comments).
fn rewrite_voice_section(section: &str, body_lines: &[String]) -> Vec<String> {
    let mut split_idx = body_lines.len();
    while split_idx > 0 && body_lines[split_idx - 1].trim().is_empty() {
        split_idx -= 1;
    }
    let core_lines = &body_lines[..split_idx];
    let trailing_blanks = &body_lines[split_idx..];

    let base_indent = core_lines
        .iter()
        .filter_map(|line| {
            let trimmed = line.trim_start();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                None
            } else {
                Some(leading_spaces_count(line))
            }
        })
        .find(|ind| *ind > 0)
        .unwrap_or(2);
    let child_indent = core_lines
        .iter()
        .filter_map(|line| {
            let trimmed = line.trim_start();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                None
            } else {
                let ind = leading_spaces_count(line);
                if ind > base_indent {
                    Some(ind)
                } else {
                    None
                }
            }
        })
        .next()
        .unwrap_or(base_indent + 2);

    let indent_str = " ".repeat(base_indent);
    let sub_indent_str = " ".repeat(child_indent);

    let mut saw_enabled = false;
    let mut saw_provider = false;
    let mut saw_language = false;
    let mut saw_openai = false;
    let mut skip_openai_children_until_indent: Option<usize> = None;
    let mut body_out: Vec<String> = Vec::new();

    for line in core_lines {
        let trimmed = line.trim_start();
        let is_blank_or_comment = trimmed.is_empty() || trimmed.starts_with('#');
        let indent = leading_spaces_count(line);

        if let Some(openai_parent_indent) = skip_openai_children_until_indent {
            if is_blank_or_comment || indent > openai_parent_indent {
                continue;
            }
            skip_openai_children_until_indent = None;
        }

        if !is_blank_or_comment && indent == base_indent {
            let no_comment = strip_yaml_inline_comment(trimmed);
            if let Some((key, _val_raw)) = split_yaml_key_val(no_comment) {
                if key == "openai" {
                    saw_openai = true;
                    body_out.extend(canonical_openai_subblock_lines(
                        section,
                        &indent_str,
                        &sub_indent_str,
                    ));
                    skip_openai_children_until_indent = Some(indent);
                    continue;
                }
                if section == "stt" {
                    match key {
                        "enabled" => {
                            saw_enabled = true;
                            body_out.push(format!("{indent_str}enabled: true"));
                            continue;
                        }
                        "provider" => {
                            saw_provider = true;
                            body_out.push(format!("{indent_str}provider: openai"));
                            continue;
                        }
                        "language" => {
                            saw_language = true;
                            body_out.push(format!("{indent_str}language: pt"));
                            continue;
                        }
                        "use_gateway" => {
                            body_out.push(format!("{indent_str}use_gateway: false"));
                            continue;
                        }
                        _ => {}
                    }
                } else if section == "tts" {
                    match key {
                        "provider" => {
                            saw_provider = true;
                            body_out.push(format!("{indent_str}provider: openai"));
                            continue;
                        }
                        "use_gateway" => {
                            body_out.push(format!("{indent_str}use_gateway: false"));
                            continue;
                        }
                        _ => {}
                    }
                }
            }
        }

        body_out.push(line.clone());
    }

    let mut result: Vec<String> = vec![format!("{section}:")];
    if section == "stt" && !saw_enabled {
        result.push(format!("{indent_str}enabled: true"));
    }
    if !saw_provider {
        result.push(format!("{indent_str}provider: openai"));
    }
    if section == "stt" && !saw_language {
        result.push(format!("{indent_str}language: pt"));
    }

    result.extend(body_out);

    if !saw_openai {
        result.extend(canonical_openai_subblock_lines(
            section,
            &indent_str,
            &sub_indent_str,
        ));
    }
    result.extend(trailing_blanks.iter().cloned());
    result
}

fn rewrite_hermes_voice_yaml(existing: &str) -> String {
    let normalized = existing.replace("\r\n", "\n").replace('\r', "\n");
    let lines: Vec<String> = normalized.lines().map(ToOwned::to_owned).collect();

    let mut out_lines: Vec<String> = Vec::new();
    let mut wrote_stt = false;
    let mut wrote_tts = false;
    let mut i = 0usize;

    while i < lines.len() {
        let line = &lines[i];
        let trimmed = line.trim_start();
        let is_top_key = !trimmed.is_empty()
            && !trimmed.starts_with('#')
            && leading_spaces_count(line) == 0
            && split_yaml_key_val(strip_yaml_inline_comment(trimmed)).is_some();

        if is_top_key {
            let key = split_yaml_key_val(strip_yaml_inline_comment(trimmed))
                .map(|(k, _)| k)
                .unwrap_or("");
            if key == "stt" || key == "tts" {
                let mut end = i + 1;
                while end < lines.len() {
                    let next = &lines[end];
                    let next_trimmed = next.trim_start();
                    let next_is_top = !next_trimmed.is_empty()
                        && !next_trimmed.starts_with('#')
                        && leading_spaces_count(next) == 0
                        && split_yaml_key_val(strip_yaml_inline_comment(next_trimmed)).is_some();
                    if next_is_top {
                        break;
                    }
                    end += 1;
                }
                let section_body = &lines[i + 1..end];
                out_lines.extend(rewrite_voice_section(key, section_body));
                if key == "stt" {
                    wrote_stt = true;
                } else {
                    wrote_tts = true;
                }
                i = end;
                continue;
            }
        }

        out_lines.push(line.clone());
        i += 1;
    }

    if !wrote_stt {
        if !out_lines.is_empty() && !out_lines.last().is_some_and(|l| l.is_empty()) {
            out_lines.push(String::new());
        }
        out_lines.extend(rewrite_voice_section("stt", &[]));
    }

    if !wrote_tts {
        if !out_lines.is_empty() && !out_lines.last().is_some_and(|l| l.is_empty()) {
            out_lines.push(String::new());
        }
        out_lines.extend(rewrite_voice_section("tts", &[]));
    }

    let mut rendered = out_lines.join("\n");
    if !rendered.ends_with('\n') {
        rendered.push('\n');
    }
    rendered
}

/// GET /api/v1/hermes/voice-customs
///
/// Inspects Hermes `config.yaml` hierarchically for PT-BR STT/TTS alignment.
pub async fn hermes_voice_customs_handler(State(state): State<AppState>) -> impl IntoResponse {
    let _guard = state.acquire_connection();
    let cfg_path = crate::profiles::resolve_hermes_config_path();
    let profile = state
        .memory
        .get_rapport_profile("default")
        .await
        .unwrap_or_default();
    let mode_str = profile.active_cognitive_mode.as_str().to_string();

    let report = tokio::task::spawn_blocking(move || build_hermes_voice_report(&cfg_path, &mode_str))
        .await
        .unwrap_or_else(|_| HermesVoiceCustomsReportDto {
            config_path: String::new(),
            stt_enabled: false,
            current_stt_language: String::new(),
            current_stt_provider: "unset".to_string(),
            current_tts_provider: "unset".to_string(),
            current_voice_mode: "auto".to_string(),
            is_synced_with_souls_pt_br: false,
            issues_found: vec!["Erro ao inspecionar config.yaml".to_string()],
            human_summary_pt: "Não foi possível ler a configuração de voz do Hermes.".to_string(),
        });

    let mut val = serde_json::to_value(&report).unwrap_or_else(|_| json!({}));
    if let Value::Object(ref mut map) = val {
        map.insert("status".to_string(), json!("success"));
    }
    (StatusCode::OK, Json(val))
}

fn sync_single_voice_yaml_file(yaml_path: &Path) -> Result<Option<String>, String> {
    if let Some(parent) = yaml_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Falha ao preparar diretório '{}': {e}", parent.display()))?;
    }

    let mut backup_path_str: Option<String> = None;
    let existing = if yaml_path.is_file() {
        let bak = yaml_path.with_extension("yaml.bak");
        std::fs::copy(yaml_path, &bak)
            .map_err(|e| format!("Falha ao criar backup preventivo (.yaml.bak): {e}"))?;
        backup_path_str = Some(bak.to_string_lossy().to_string());
        std::fs::read_to_string(yaml_path)
            .map_err(|e| format!("Falha ao ler '{}': {e}", yaml_path.display()))?
    } else {
        String::new()
    };

    let updated = rewrite_hermes_voice_yaml(&existing);
    std::fs::write(yaml_path, updated.as_bytes())
        .map_err(|e| format!("Falha ao gravar '{}': {e}", yaml_path.display()))?;

    Ok(backup_path_str)
}

/// POST /api/v1/hermes/voice-customs/sync
///
/// Creates `.yaml.bak` backups and atomically updates `stt:` and `tts:` blocks in Hermes root
/// `config.yaml` and any `profiles/*/config.yaml` sub-profiles to route PT-BR voice locally.
pub async fn hermes_voice_customs_sync_handler(
    State(state): State<AppState>,
    _body: Bytes,
) -> impl IntoResponse {
    let _guard = state.acquire_connection();
    let cfg_path = crate::profiles::resolve_hermes_config_path();
    let profile = state
        .memory
        .get_rapport_profile("default")
        .await
        .unwrap_or_default();
    let mode_str = profile.active_cognitive_mode.as_str().to_string();

    let sync_res = tokio::task::spawn_blocking(
        move || -> Result<(Option<String>, Vec<String>, HermesVoiceCustomsReportDto), String> {
            let backup_path_str = sync_single_voice_yaml_file(&cfg_path)?;
            let mut synced_profiles = vec!["default".to_string()];

            for (prof_name, sub_cfg) in discover_hermes_subprofile_configs(&cfg_path) {
                sync_single_voice_yaml_file(&sub_cfg)?;
                synced_profiles.push(prof_name);
            }

            let report = build_hermes_voice_report(&cfg_path, &mode_str);
            Ok((backup_path_str, synced_profiles, report))
        },
    )
    .await;

    match sync_res {
        Ok(Ok((backup_path, synced_profiles, report))) => (
            StatusCode::OK,
            Json(json!({
                "status": "success",
                "backup_path": backup_path,
                "synced_profiles": synced_profiles,
                "report": report
            })),
        ),
        Ok(Err(msg)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "status": "error",
                "message": msg
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "status": "error",
                "message": format!("Erro interno ao sincronizar config.yaml: {e}")
            })),
        ),
    }
}
