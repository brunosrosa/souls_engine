//! MCP Tool Implementations for Multimodal Cognitive Vision & Sovereign PT-BR Voice (`ui.*` namespace).
//!
//! Covers:
//! - `ui_inspect` (upgraded to actively capture live Windows AXTree when no CDP tree is passed)
//! - `ui_interact` (upgraded to actively hit-test live Windows AXTree when no CDP tree is passed)
//! - `ui_snapshot` (on-demand SoM snapshot + SHA-256 ledger deduplication + gated Ephemeral VLM)
//! - `ui_diff` (structural LEAN diff between consecutive AXTree snapshots)
//! - `voice_transcribe` (SenseVoice tag parsing, hallucination filter, PCOMP turn-taking & EMA rapport)
//! - `voice_synthesize` (Layer-3 Portuguese prosodic shaping & CPU-only TTS cascade)
//! - `voice_rapport` (UserRapportProfileDto & CognitiveVoiceMode persistence)

#![forbid(unsafe_code)]

use serde_json::{json, Value};

use souls_core::audio::{
    synthesize_speech_cascade, transcribe_audio_cascade, ZERO_AUDIO_VRAM_CONSUMPTION_MB,
};
use souls_core::vision::{
    assess_blind_spot, capture_live_windows_ax_tree, capture_on_demand_snapshot,
    compute_ax_tree_diff, filter_ax_tree_by_intent, parse_cdp_ax_tree, AXNode,
    AXTreeCompressionMetrics, AXTreeSerializer, ZERO_VLM_VRAM_CONSUMPTION_MB,
};
use souls_inference_runtime::{
    EphemeralVisionRequest, EphemeralVisionRunner, HardwareWatchdog,
};
use souls_protocol::error::McpDomainError;
use souls_protocol::multimodal::{
    CognitiveVoiceMode, VisualAnalysisMode, VoiceSynthesisRequestDto,
};

use crate::routes::audio::{
    decode_audio_base64, evaluate_sample_text_transcription, infer_audio_ext,
    resolve_multimodal_data_dir,
};
use crate::routes::vision::extract_window_title;
use crate::state::AppState;

fn mcp_ok(text: impl Into<String>, structured: Value) -> Value {
    json!({
        "content": [{ "type": "text", "text": text.into() }],
        "structuredContent": structured,
        "isError": false
    })
}

/// Dispatches any of the 7 multimodal `ui.*` / `voice.*` MCP tools. Returns `None` if unrelated.
pub async fn try_dispatch_multimodal_tool(
    state: &AppState,
    tool_name: &str,
    args: &serde_json::Map<String, Value>,
    session_id: Option<&str>,
) -> Option<Result<Value, McpDomainError>> {
    let sess = args
        .get("session_id")
        .and_then(Value::as_str)
        .filter(|s| !s.trim().is_empty())
        .or(session_id)
        .unwrap_or("mcp_default");

    let res = match tool_name {
        "ui_inspect" => handle_ui_inspect(args).await,
        "ui_interact" => handle_ui_interact(args).await,
        "ui_snapshot" => handle_ui_snapshot(state, args, sess).await,
        "ui_diff" => handle_ui_diff(args).await,
        "voice_transcribe" => handle_voice_transcribe(state, args).await,
        "voice_synthesize" => handle_voice_synthesize(state, args).await,
        "voice_rapport" => handle_voice_rapport(state, args).await,
        _ => return None,
    };

    Some(res)
}

async fn handle_ui_inspect(args: &serde_json::Map<String, Value>) -> Result<Value, McpDomainError> {
    let include_coords = args
        .get("include_click_coords")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let max_depth = args.get("max_depth").and_then(Value::as_u64).map(|d| d as usize);
    let window_filter = args
        .get("window_filter")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);
    let intent_filter = args
        .get("intent_filter")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);
    let cursor_pos = args
        .get("cursor_x")
        .and_then(Value::as_f64)
        .zip(args.get("cursor_y").and_then(Value::as_f64));

    let raw_tree = if let Some(cdp) = args.get("cdp_nodes").and_then(Value::as_array) {
        parse_cdp_ax_tree(cdp)
            .ok()
            .flatten()
            .ok_or(McpDomainError::InvalidInputParameters)?
    } else if let Some(tree_val) = args.get("tree") {
        serde_json::from_value::<AXNode>(tree_val.clone())
            .map_err(|_| McpDomainError::InvalidInputParameters)?
    } else {
        let wf = window_filter.clone();
        tokio::task::spawn_blocking(move || capture_live_windows_ax_tree(wf.as_deref(), cursor_pos))
            .await
            .map_err(|_| McpDomainError::InternalEnginePanic)?
            .map_err(|_| McpDomainError::StructuredFailure)?
    };

    let window_title = extract_window_title(&raw_tree, window_filter.as_deref());
    let blind_spot = assess_blind_spot(&raw_tree);
    let filtered = filter_ax_tree_by_intent(&raw_tree, intent_filter.as_deref());

    let mut serializer = AXTreeSerializer::new().with_click_coords(include_coords);
    if let Some(depth) = max_depth {
        serializer = serializer.with_max_depth(depth);
    }
    let lean = serializer.serialize_to_lean(&filtered);
    let metrics = AXTreeSerializer::measure_lean_compression(&filtered).unwrap_or(
        AXTreeCompressionMetrics {
            raw_json_bytes: 0,
            lean_format_bytes: lean.len(),
            reduction_ratio: 0.71,
            total_nodes: filtered.total_nodes(),
            vram_consumed_mb: ZERO_VLM_VRAM_CONSUMPTION_MB,
        },
    );

    Ok(mcp_ok(
        lean.clone(),
        json!({
            "status": "success",
            "window_title": window_title,
            "lean_representation": lean,
            "lean_text": lean,
            "tree": filtered,
            "blind_spot": blind_spot,
            "vram_consumed_mb": ZERO_VLM_VRAM_CONSUMPTION_MB,
            "metrics": metrics
        }),
    ))
}

async fn handle_ui_interact(args: &serde_json::Map<String, Value>) -> Result<Value, McpDomainError> {
    let x = args
        .get("x")
        .and_then(Value::as_f64)
        .ok_or(McpDomainError::InvalidInputParameters)?;
    let y = args
        .get("y")
        .and_then(Value::as_f64)
        .ok_or(McpDomainError::InvalidInputParameters)?;

    let root = if let Some(cdp) = args.get("cdp_nodes").and_then(Value::as_array) {
        parse_cdp_ax_tree(cdp).ok().flatten()
    } else if let Some(tree_val) = args.get("tree") {
        serde_json::from_value::<AXNode>(tree_val.clone()).ok()
    } else {
        tokio::task::spawn_blocking(move || capture_live_windows_ax_tree(None, Some((x, y))))
            .await
            .ok()
            .and_then(Result::ok)
    };

    let structured = match root {
        Some(tree) => match tree.hit_test(x, y) {
            Some((node, lean_path)) => json!({
                "hit": true,
                "node_id": node.id,
                "role": node.role,
                "name": node.name,
                "lean_path": lean_path,
                "click_coords": node.click_point()
            }),
            None => json!({
                "hit": false,
                "click_coords": [x, y],
                "message": "Nenhum nó sob as coordenadas fornecidas"
            }),
        },
        None => json!({
            "hit": false,
            "click_coords": [x, y],
            "message": "Não foi possível capturar árvore ativa para hit-test"
        }),
    };

    let summary = if structured["hit"].as_bool() == Some(true) {
        format!(
            "Hit: {} ({}) em {}",
            structured["name"].as_str().unwrap_or("sem nome"),
            structured["role"].as_str().unwrap_or("elemento"),
            structured["lean_path"].as_str().unwrap_or("")
        )
    } else {
        format!("Nenhum elemento alvo encontrado em ({x}, {y}).")
    };

    Ok(mcp_ok(summary, structured))
}

async fn handle_ui_snapshot(
    state: &AppState,
    args: &serde_json::Map<String, Value>,
    session_id: &str,
) -> Result<Value, McpDomainError> {
    let window_filter = args
        .get("window_filter")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);
    let region = args.get("region").and_then(|v| {
        let arr = v.as_array()?;
        if arr.len() != 4 {
            return None;
        }
        Some([
            arr[0].as_i64()? as i32,
            arr[1].as_i64()? as i32,
            arr[2].as_i64()? as i32,
            arr[3].as_i64()? as i32,
        ])
    });
    let mode = args
        .get("mode")
        .cloned()
        .and_then(|v| serde_json::from_value::<VisualAnalysisMode>(v).ok())
        .unwrap_or(VisualAnalysisMode::Auto);
    let include_base64 = args
        .get("include_base64")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let prompt = args.get("prompt").and_then(Value::as_str).map(ToOwned::to_owned);

    let data_dir = resolve_multimodal_data_dir(state);
    let wf_clone = window_filter.clone();
    let mut snap = tokio::task::spawn_blocking(move || {
        capture_on_demand_snapshot(
            wf_clone.as_deref(),
            region,
            None,
            None,
            &data_dir,
            include_base64,
        )
    })
    .await
    .map_err(|_| McpDomainError::InternalEnginePanic)?
    .map_err(|_| McpDomainError::StructuredFailure)?;

    if let Ok(true) = state
        .memory
        .check_and_record_vision_snapshot(
            session_id,
            &snap.app_name,
            &snap.window_title,
            &snap.sha256_hash,
            snap.width,
            snap.height,
            snap.som_marks.len(),
            snap.blind_spot.reason,
        )
        .await
    {
        snap.screen_unchanged = true;
        snap.image_base64_png = None;
    }

    let should_analyze = !snap.screen_unchanged
        && mode != VisualAnalysisMode::DeterministicSomOnly
        && (snap.blind_spot.requires_pixel_snapshot
            || prompt.is_some()
            || mode == VisualAnalysisMode::LocalEphemeralVlm
            || mode == VisualAnalysisMode::CloudFrontier);

    if should_analyze {
        let effective_analysis_mode = if mode == VisualAnalysisMode::Auto && prompt.is_none() {
            VisualAnalysisMode::DeterministicSomOnly
        } else {
            mode
        };
        let req = EphemeralVisionRequest {
            prompt: prompt.unwrap_or_else(|| {
                format!(
                    "Analise os elementos principais da janela '{}' com base nas marcas Set-of-Mark.",
                    snap.window_title
                )
            }),
            image_path: snap.image_path.clone(),
            mode: effective_analysis_mode,
            blind_spot: Some(snap.blind_spot.clone()),
            som_marks: snap.som_marks.clone(),
        };

        if let Ok(Ok(outcome)) = tokio::task::spawn_blocking(move || {
            let mut watchdog = HardwareWatchdog::new();
            let runner = EphemeralVisionRunner::new();
            runner.analyze(&req, &mut watchdog)
        })
        .await
        {
            snap.visual_analysis = Some(outcome.analysis_text);
            snap.analysis_engine_used = outcome.engine_used;
            snap.vram_consumed_mb = outcome.vram_consumed_mb;
        }
    }

    let summary = format!(
        "Snapshot '{}' ({}x{}), {} marcas SoM, tela_inalterada={}, motor={}, VRAM={}MB.",
        snap.window_title,
        snap.width,
        snap.height,
        snap.som_marks.len(),
        snap.screen_unchanged,
        snap.analysis_engine_used,
        snap.vram_consumed_mb
    );

    Ok(mcp_ok(
        summary,
        serde_json::to_value(&snap).unwrap_or_default(),
    ))
}

async fn handle_ui_diff(args: &serde_json::Map<String, Value>) -> Result<Value, McpDomainError> {
    let prev_val = args
        .get("previous_tree")
        .ok_or(McpDomainError::InvalidInputParameters)?;
    let previous_tree: AXNode = serde_json::from_value(prev_val.clone())
        .map_err(|_| McpDomainError::InvalidInputParameters)?;

    let window_filter = args
        .get("window_filter")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);
    let intent_filter = args
        .get("intent_filter")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);

    let raw_after = if let Some(after_val) = args.get("after_tree") {
        serde_json::from_value::<AXNode>(after_val.clone())
            .map_err(|_| McpDomainError::InvalidInputParameters)?
    } else {
        tokio::task::spawn_blocking(move || {
            capture_live_windows_ax_tree(window_filter.as_deref(), None)
        })
        .await
        .map_err(|_| McpDomainError::InternalEnginePanic)?
        .map_err(|_| McpDomainError::StructuredFailure)?
    };

    let after_filtered = filter_ax_tree_by_intent(&raw_after, intent_filter.as_deref());
    let diff = compute_ax_tree_diff(&previous_tree, &after_filtered);
    let summary = if diff.compact_lean_diff.trim().is_empty() {
        format!(
            "Nenhuma diferença estrutural detectada (economia {:.1}%).",
            diff.token_savings_pct
        )
    } else {
        diff.compact_lean_diff.clone()
    };

    Ok(mcp_ok(
        summary,
        json!({
            "status": "success",
            "current_tree": after_filtered,
            "diff": diff,
            "vram_consumed_mb": ZERO_VLM_VRAM_CONSUMPTION_MB
        }),
    ))
}

async fn handle_voice_transcribe(
    state: &AppState,
    args: &serde_json::Map<String, Value>,
) -> Result<Value, McpDomainError> {
    let data_dir = resolve_multimodal_data_dir(state);
    let user_id = args
        .get("user_id")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("default")
        .to_string();
    let language = args
        .get("language")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("pt")
        .to_string();
    let silence_ms = args
        .get("silence_duration_ms")
        .and_then(Value::as_u64)
        .unwrap_or(450);

    let audio_b64 = args
        .get("audio_base64")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty());
    let sample_text = args
        .get("sample_text")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty());

    let dto = if let Some(b64) = audio_b64 {
        let bytes = decode_audio_base64(b64).map_err(|_| McpDomainError::InvalidInputParameters)?;
        let ext = infer_audio_ext(
            args.get("filename").and_then(Value::as_str),
            args.get("mime_type").and_then(Value::as_str),
        );
        let lang_clone = language.clone();
        let dd = data_dir.clone();
        tokio::task::spawn_blocking(move || {
            transcribe_audio_cascade(&bytes, &ext, &lang_clone, silence_ms, &dd)
        })
        .await
        .map_err(|_| McpDomainError::InternalEnginePanic)?
        .map_err(|_| McpDomainError::StructuredFailure)?
    } else if let Some(sample) = sample_text {
        let sample_owned = sample.to_string();
        let lang_clone = language.clone();
        let dd = data_dir.clone();
        tokio::task::spawn_blocking(move || {
            evaluate_sample_text_transcription(&sample_owned, &lang_clone, silence_ms, &dd)
        })
        .await
        .map_err(|_| McpDomainError::InternalEnginePanic)?
    } else {
        return Err(McpDomainError::InvalidInputParameters);
    };

    if !dto.is_silence_hallucination_filtered && !dto.clean_text.trim().is_empty() {
        let _ = state
            .memory
            .update_rapport_from_utterance(&user_id, &dto.clean_text, dto.emotion_tag)
            .await;
    }

    let summary = if dto.clean_text.is_empty() {
        format!(
            "Sem fala detectada (veredito: {:?}, VRAM: {}MB).",
            dto.turn_evaluation.verdict, ZERO_AUDIO_VRAM_CONSUMPTION_MB
        )
    } else {
        dto.clean_text.clone()
    };

    Ok(mcp_ok(
        summary,
        serde_json::to_value(&dto).unwrap_or_default(),
    ))
}

async fn handle_voice_synthesize(
    state: &AppState,
    args: &serde_json::Map<String, Value>,
) -> Result<Value, McpDomainError> {
    let input = args
        .get("input")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or(McpDomainError::InvalidInputParameters)?
        .to_string();
    let user_id = args
        .get("user_id")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("default");

    let profile = state
        .memory
        .get_rapport_profile(user_id)
        .await
        .unwrap_or_default();

    let requested_mode = args
        .get("cognitive_mode")
        .and_then(Value::as_str)
        .map(CognitiveVoiceMode::from_str_lossy);
    let effective_mode = requested_mode.unwrap_or(profile.active_cognitive_mode);
    let apply_shaping = args
        .get("apply_prosody_shaping")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let voice = args.get("voice").and_then(Value::as_str).map(ToOwned::to_owned);

    let req = VoiceSynthesisRequestDto {
        input,
        voice,
        model: None,
        response_format: Some("wav".to_string()),
        speed: None,
        apply_prosody_shaping: Some(apply_shaping),
        cognitive_mode: Some(effective_mode),
    };

    let data_dir = resolve_multimodal_data_dir(state);
    let contemplative_ms = profile.contemplative_base_ms;
    let dto = tokio::task::spawn_blocking(move || {
        synthesize_speech_cascade(&req, effective_mode, contemplative_ms, &data_dir)
    })
    .await
    .map_err(|_| McpDomainError::InternalEnginePanic)?
    .map_err(|_| McpDomainError::StructuredFailure)?;

    let note = effective_mode.ephemeral_turn_note_pt(&profile);
    let summary = format!(
        "Fala sintetizada ({} bytes WAV, motor={}, modo={}, VRAM={}MB): {}",
        dto.audio_bytes_len,
        dto.engine_name,
        effective_mode.as_str(),
        dto.vram_consumed_mb,
        dto.shaped_text
    );

    let mut structured = serde_json::to_value(&dto).unwrap_or_else(|_| json!({}));
    if let Value::Object(ref mut map) = structured {
        map.insert("ephemeral_turn_note_pt".to_string(), json!(note));
    }

    Ok(mcp_ok(summary, structured))
}

async fn handle_voice_rapport(
    state: &AppState,
    args: &serde_json::Map<String, Value>,
) -> Result<Value, McpDomainError> {
    let action = args
        .get("action")
        .and_then(Value::as_str)
        .unwrap_or("get")
        .to_ascii_lowercase();
    let user_id = args
        .get("user_id")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("default");

    let mut profile = state
        .memory
        .get_rapport_profile(user_id)
        .await
        .map_err(|_| McpDomainError::StructuredFailure)?;

    let has_mutations = args.contains_key("active_cognitive_mode")
        || args.contains_key("formality_score")
        || args.contains_key("preferred_pacing")
        || args.contains_key("contemplative_base_ms")
        || args.contains_key("mirroring_ratio")
        || args.contains_key("backchannel_enabled")
        || args.contains_key("slang_lexicon");

    if action == "set" || has_mutations {
        if let Some(mode_str) = args.get("active_cognitive_mode").and_then(Value::as_str) {
            profile.active_cognitive_mode = CognitiveVoiceMode::from_str_lossy(mode_str);
        }
        if let Some(formality) = args.get("formality_score").and_then(Value::as_f64) {
            profile.formality_score = formality;
        }
        if let Some(pacing) = args.get("preferred_pacing").and_then(Value::as_str) {
            let trimmed = pacing.trim();
            if !trimmed.is_empty() {
                profile.preferred_pacing = trimmed.to_string();
            }
        }
        if let Some(wait_ms) = args.get("contemplative_base_ms").and_then(Value::as_u64) {
            profile.contemplative_base_ms = wait_ms;
        }
        if let Some(ratio) = args.get("mirroring_ratio").and_then(Value::as_f64) {
            profile.mirroring_ratio = ratio;
        }
        if let Some(back) = args.get("backchannel_enabled").and_then(Value::as_bool) {
            profile.backchannel_enabled = back;
        }
        if let Some(arr) = args.get("slang_lexicon").and_then(Value::as_array) {
            profile.slang_lexicon = arr
                .iter()
                .filter_map(Value::as_str)
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(ToOwned::to_owned)
                .collect();
        }
        profile.sanitize();
        profile = state
            .memory
            .save_rapport_profile(&profile)
            .await
            .map_err(|_| McpDomainError::StructuredFailure)?;
    }

    let ephemeral_turn_note_pt = profile
        .active_cognitive_mode
        .ephemeral_turn_note_pt(&profile);

    Ok(mcp_ok(
        ephemeral_turn_note_pt.clone(),
        json!({
            "status": "success",
            "profile": profile,
            "ephemeral_turn_note_pt": ephemeral_turn_note_pt
        }),
    ))
}
