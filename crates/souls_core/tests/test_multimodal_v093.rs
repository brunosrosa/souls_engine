//! Integration tests for Stage 3 of Milestone v0.9.3 (`souls_core` multimodal audio & vision).

use souls_core::audio::{
    backchannel_pcm_wav, discover_hermes_audio_tools, evaluate_pcomp_turn_taking,
    generate_pcm_wav_buffer, handoff_chime_wav, normalize_stt_bcp47_language,
    parse_sensevoice_and_filter_hallucinations, probe_audio_engines,
    provision_local_faster_whisper_model, shape_prosodic_text_for_tts, split_prosodic_sentences,
    synthesize_speech_cascade, transcribe_audio_cascade, BARGE_IN_MIN_SPEECH_MS,
    NOISE_IGNORE_MAX_MS, SILERO_VAD_STANDBY_HYSTERESIS_MS, ZERO_AUDIO_VRAM_CONSUMPTION_MB,
};
use souls_core::vision::{
    assess_blind_spot, capture_live_windows_ax_tree, capture_on_demand_snapshot,
    compute_ax_tree_diff, filter_ax_tree_by_intent, generate_som_marks, AXNode, AXNodeState,
    ZERO_VLM_VRAM_CONSUMPTION_MB,
};
use souls_protocol::multimodal::{
    CognitiveVoiceMode, DetectedEmotionTag, TurnTakingVerdict, VisionEscalationReason,
    VoiceSynthesisRequestDto,
};
use std::time::Instant;
use tempfile::TempDir;

#[test]
fn test_silero_vad_hysteresis_timeout_8s() {
    assert_eq!(SILERO_VAD_STANDBY_HYSTERESIS_MS, 8_000);

    // Below 8s with closed sentence -> TakeTurnImmediate (or ContemplativeWait if < recommended_wait_ms)
    let active = evaluate_pcomp_turn_taking("Tudo pronto por aqui.", 7_999, 0.2, false, 0);
    assert_ne!(active.verdict, TurnTakingVerdict::StandbyTimeout);
    assert_eq!(active.verdict, TurnTakingVerdict::TakeTurnImmediate);

    // At or above 8,000 ms silence -> StandbyTimeout
    let standby = evaluate_pcomp_turn_taking("Tudo pronto por aqui.", 8_000, 0.2, false, 0);
    assert_eq!(standby.verdict, TurnTakingVerdict::StandbyTimeout);

    let standby_long = evaluate_pcomp_turn_taking("", 12_500, 0.0, false, 0);
    assert_eq!(standby_long.verdict, TurnTakingVerdict::StandbyTimeout);
}

#[test]
fn test_barge_in_ignores_short_noise_and_aborts_above_300ms() {
    assert_eq!(BARGE_IN_MIN_SPEECH_MS, 300);
    assert_eq!(NOISE_IGNORE_MAX_MS, 200);

    // Brief cough/noise <= 200ms while agent is speaking must NOT interrupt
    let cough = evaluate_pcomp_turn_taking("hm", 50, 0.3, true, 150);
    assert_eq!(cough.verdict, TurnTakingVerdict::ContinueListening);

    let edge_below = evaluate_pcomp_turn_taking("ah", 50, 0.3, true, 299);
    assert_eq!(edge_below.verdict, TurnTakingVerdict::ContinueListening);

    // Sustained user speech >= 300ms triggers immediate BargeInAbort
    let barge_in = evaluate_pcomp_turn_taking("Espera um segundo", 0, 0.6, true, 320);
    assert_eq!(barge_in.verdict, TurnTakingVerdict::BargeInAbort);
    assert_eq!(barge_in.recommended_wait_ms, 0);
}

#[test]
fn test_pcomp_open_connective_prevents_interruption_and_emits_backchannel() {
    // Open connective with brief pause (<300ms) -> ContinueListening (never interrupt mid-thought)
    let brief = evaluate_pcomp_turn_taking("Eu queria refatorar isso porque", 150, 0.2, false, 0);
    assert!(brief.ends_with_open_connective);
    assert!(brief.syntactic_closure_score < 0.5);
    assert_eq!(brief.verdict, TurnTakingVerdict::ContinueListening);
    assert!((200..=600).contains(&brief.recommended_wait_ms));

    // Open connective with 300..=1200ms pause -> EmitBackchannel
    let mid_pause = evaluate_pcomp_turn_taking("Então, o problema é que...", 650, 0.4, false, 0);
    assert!(mid_pause.ends_with_open_connective);
    assert_eq!(mid_pause.verdict, TurnTakingVerdict::EmitBackchannel);
    assert!(mid_pause.suggested_backchannel.is_some());

    // Multi-word connective "e aí"
    let multi = evaluate_pcomp_turn_taking("A gente rodou os testes e aí", 500, 0.1, false, 0);
    assert!(multi.ends_with_open_connective);
    assert_eq!(multi.verdict, TurnTakingVerdict::EmitBackchannel);

    // Complete sentence with high syntactic closure and T_wait formula validation
    let closed = evaluate_pcomp_turn_taking("Pode executar agora.", 700, 0.8, false, 0);
    assert!(!closed.ends_with_open_connective);
    assert!(closed.syntactic_closure_score >= 0.85);
    assert_eq!(closed.verdict, TurnTakingVerdict::TakeTurnImmediate);
    assert!((200..=600).contains(&closed.recommended_wait_ms));
}

#[test]
fn test_sensevoice_tag_extraction_and_hallucination_filter() {
    let (clean, emotion, conf, filtered) =
        parse_sensevoice_and_filter_hallucinations("<HAPPY>Funcionou perfeitamente!</HAPPY>");
    assert_eq!(clean, "Funcionou perfeitamente!");
    assert_eq!(emotion, DetectedEmotionTag::Happy);
    assert!(conf >= 0.85);
    assert!(!filtered);

    let (clean_hes, emo_hes, _, filt_hes) =
        parse_sensevoice_and_filter_hallucinations("<HESITANT>É... acho que talvez</hesitant>");
    assert_eq!(clean_hes, "É... acho que talvez");
    assert_eq!(emo_hes, DetectedEmotionTag::Hesitant);
    assert!(!filt_hes);

    // Heuristic without tags
    let (clean_heur, emo_heur, _, filt_heur) =
        parse_sensevoice_and_filter_hallucinations("Talvez amanhã...");
    assert_eq!(clean_heur, "Talvez amanhã...");
    assert_eq!(emo_heur, DetectedEmotionTag::Hesitant);
    assert!(!filt_heur);

    // Silence hallucinations must be filtered
    for phantom in [
        "",
        "   ",
        "Thank you.",
        "Thanks for watching",
        "ok. ok. ok.",
        "Legendas pela comunidade",
        "...",
    ] {
        let (c, _, _, is_filt) = parse_sensevoice_and_filter_hallucinations(phantom);
        assert!(is_filt, "Expected hallucination filter on '{phantom}'");
        assert!(c.is_empty());
    }
}

#[test]
fn test_sentence_boundary_regex_chunking_sub_1ms() {
    let sample = "Tudo certo! Vamos começar... O sistema v0.9.3 está pronto; falta apenas validar — e seguir em frente.\n\nPróxima etapa: rodar os testes.";
    let started = Instant::now();
    let chunks = split_prosodic_sentences(sample);
    let elapsed = started.elapsed();

    assert!(
        elapsed.as_micros() < 1_000,
        "Chunking took {} us, expected < 1000 us",
        elapsed.as_micros()
    );
    assert!(chunks.len() >= 5);
    assert_eq!(chunks[0], "Tudo certo!");
    assert_eq!(chunks[1], "Vamos começar...");
    assert!(chunks.iter().any(|c| c.contains("0.9.3")));
}

#[test]
fn test_prosody_shaping_strips_markdown_and_adds_cues() {
    let raw_md = "## Resumo\nVeja **isso** em [documentação](https://example.com/docs) e https://souls.local/api\n```rust\nlet x = 1;\n```\n- Passo `cargo test` concluído";

    let (shaped_crisis, chunks_crisis) =
        shape_prosodic_text_for_tts(raw_md, CognitiveVoiceMode::CrisisParalysis);
    assert!(!shaped_crisis.contains("**"));
    assert!(!shaped_crisis.contains("```"));
    assert!(!shaped_crisis.contains("https://"));
    assert!(shaped_crisis.contains("documentação"));
    assert!(shaped_crisis.contains("o link enviado"));
    assert!(shaped_crisis.contains("bloco de código"));
    assert!(shaped_crisis.contains("..."));
    assert!(!chunks_crisis.is_empty());

    let (shaped_sparring, chunks_sparring) = shape_prosodic_text_for_tts(
        "Ponto chave: precisamos cortar latência agora",
        CognitiveVoiceMode::Sparring,
    );
    assert!(shaped_sparring.contains('—'));
    assert!(shaped_sparring.ends_with('.'));
    assert!(!chunks_sparring.is_empty());
}

#[test]
fn test_axtree_page_diff_and_intent_filter() {
    let mut before = AXNode::new("desktop", "Desktop", [0.0, 0.0, 1920.0, 1080.0]);
    let mut win_b = AXNode::new("win", "Window", [0.0, 0.0, 1280.0, 720.0]).with_name("Editor");
    win_b.add_child(
        AXNode::new("btn_save", "Button", [100.0, 100.0, 220.0, 140.0]).with_name("Salvar"),
    );
    win_b.add_child(
        AXNode::new("edit_1", "Edit", [100.0, 180.0, 600.0, 220.0])
            .with_name("Título")
            .with_value("Rascunho"),
    );
    win_b.add_child(
        AXNode::new("link_1", "Link", [100.0, 240.0, 250.0, 270.0]).with_name("Ajuda"),
    );
    win_b.add_child(
        AXNode::new("btn_old", "Button", [300.0, 100.0, 400.0, 140.0]).with_name("Descartar"),
    );
    before.add_child(win_b);

    let mut after = AXNode::new("desktop", "Desktop", [0.0, 0.0, 1920.0, 1080.0]);
    let mut win_a = AXNode::new("win", "Window", [0.0, 0.0, 1280.0, 720.0]).with_name("Editor");
    win_a.add_child(
        AXNode::new("btn_save", "Button", [100.0, 100.0, 220.0, 140.0])
            .with_name("Salvar")
            .with_states(AXNodeState::new().with_focused(true)),
    );
    win_a.add_child(
        AXNode::new("edit_1", "Edit", [100.0, 180.0, 600.0, 220.0])
            .with_name("Título")
            .with_value("Final"),
    );
    win_a.add_child(
        AXNode::new("link_1", "Link", [100.0, 240.0, 250.0, 270.0]).with_name("Ajuda"),
    );
    win_a.add_child(
        AXNode::new("btn_new", "Button", [420.0, 100.0, 540.0, 140.0]).with_name("Publicar"),
    );
    after.add_child(win_a);

    // 1. Diff check
    let diff = compute_ax_tree_diff(&before, &after);
    assert_eq!(diff.added_nodes.len(), 1);
    assert!(diff.added_nodes[0].contains("Button[Publicar]"));
    assert!(diff.added_nodes[0].contains("{click: (480, 120)}"));
    assert_eq!(diff.removed_nodes.len(), 1);
    assert!(diff.removed_nodes[0].contains("Button[Descartar]"));
    assert_eq!(diff.changed_states.len(), 2);
    assert!(diff
        .changed_states
        .iter()
        .any(|s| s.contains("Button[Salvar]") && s.contains("'' -> '(F)'")));
    assert!(diff.compact_lean_diff.contains("+ "));
    assert!(diff.compact_lean_diff.contains("- "));
    assert!(diff.compact_lean_diff.contains("~ "));

    // Identical tree produces [NO_CHANGES]
    let no_diff = compute_ax_tree_diff(&after, &after);
    assert_eq!(no_diff.compact_lean_diff, "[NO_CHANGES]");

    // 2. Intent filtering
    let inputs_only = filter_ax_tree_by_intent(&after, Some("inputs"));
    assert_eq!(inputs_only.total_nodes(), 3); // Desktop -> Window -> Edit

    let buttons_only = filter_ax_tree_by_intent(&after, Some("buttons"));
    assert_eq!(buttons_only.total_nodes(), 5); // Desktop -> Window -> (Salvar, Ajuda, Publicar)

    let nav_only = filter_ax_tree_by_intent(&after, Some("navigation"));
    assert_eq!(nav_only.total_nodes(), 3); // Desktop -> Window -> Link[Ajuda]

    let all_cloned = filter_ax_tree_by_intent(&after, Some("all"));
    assert_eq!(all_cloned.total_nodes(), after.total_nodes());
}

#[test]
fn test_blind_spot_detector_and_som_sha256_dedup() {
    // Normal Hermes Agent tree -> AX sufficient
    let normal_tree = capture_live_windows_ax_tree(Some("Hermes Agent"), None).unwrap();
    let normal_bs = assess_blind_spot(&normal_tree);
    assert!(!normal_bs.requires_pixel_snapshot);
    assert_eq!(normal_bs.reason, VisionEscalationReason::NoneAxSufficient);
    assert!(normal_bs.actionable_node_count >= 4);

    let som = generate_som_marks(&normal_tree);
    assert_eq!(som.len(), normal_bs.actionable_node_count);
    assert_eq!(som[0].index, 1);

    // Canvas / Figma tree -> CanvasOrWebGlDetected
    let figma_tree = capture_live_windows_ax_tree(Some("Figma - Design"), None).unwrap();
    let figma_bs = assess_blind_spot(&figma_tree);
    assert!(figma_bs.requires_pixel_snapshot);
    assert_eq!(
        figma_bs.reason,
        VisionEscalationReason::CanvasOrWebGlDetected
    );
    assert!(figma_bs.canvas_node_count >= 1);

    // Remote Desktop VM -> RemoteDesktopOrVm
    let rdp_root = AXNode::new("rdp_win", "Window", [0.0, 0.0, 1280.0, 720.0])
        .with_name("VMware Workstation - Ubuntu");
    let rdp_bs = assess_blind_spot(&rdp_root);
    assert!(rdp_bs.requires_pixel_snapshot);
    assert_eq!(rdp_bs.reason, VisionEscalationReason::RemoteDesktopOrVm);

    // Zero actionable nodes on large window -> ZeroActionableNodes
    let empty_large = AXNode::new("blank_win", "Window", [0.0, 0.0, 800.0, 600.0])
        .with_name("Custom App");
    let blank_bs = assess_blind_spot(&empty_large);
    assert!(blank_bs.requires_pixel_snapshot);
    assert_eq!(blank_bs.reason, VisionEscalationReason::ZeroActionableNodes);

    // SHA-256 deduplication & PNG signature
    let tmp = TempDir::new().unwrap();
    let snap1 = capture_on_demand_snapshot(
        Some("Hermes Agent"),
        Some([0, 0, 800, 600]),
        Some(&normal_tree),
        None,
        tmp.path(),
        true,
    )
    .unwrap();
    assert_eq!(snap1.vram_consumed_mb, ZERO_VLM_VRAM_CONSUMPTION_MB);
    assert!(!snap1.screen_unchanged);
    assert!(snap1.image_base64_png.is_some());
    assert_eq!(snap1.sha256_hash.len(), 64);

    let saved_bytes = std::fs::read(snap1.image_path.as_ref().unwrap()).unwrap();
    assert!(saved_bytes.starts_with(b"\x89PNG\r\n\x1a\n"));

    // Identical input + matching last_sha256 -> screen_unchanged = true, base64 omitted
    let snap2 = capture_on_demand_snapshot(
        Some("Hermes Agent"),
        Some([0, 0, 800, 600]),
        Some(&normal_tree),
        Some(&snap1.sha256_hash),
        tmp.path(),
        true,
    )
    .unwrap();
    assert!(snap2.screen_unchanged);
    assert_eq!(snap2.sha256_hash, snap1.sha256_hash);
    assert!(snap2.image_base64_png.is_none());

    // Different region -> different SHA-256 & base64 included
    let snap3 = capture_on_demand_snapshot(
        Some("Hermes Agent"),
        Some([10, 10, 400, 300]),
        Some(&normal_tree),
        Some(&snap1.sha256_hash),
        tmp.path(),
        true,
    )
    .unwrap();
    assert!(!snap3.screen_unchanged);
    assert_ne!(snap3.sha256_hash, snap1.sha256_hash);
    assert!(snap3.image_base64_png.is_some());

    // Retention cap check (max 20 files)
    for _ in 0..22 {
        let _ = capture_on_demand_snapshot(
            Some("Hermes Agent"),
            None,
            Some(&normal_tree),
            None,
            tmp.path(),
            false,
        )
        .unwrap();
    }
    let png_count = std::fs::read_dir(tmp.path().join("vision_snapshots"))
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("png"))
        .count();
    assert!(png_count <= 20, "Retention exceeded 20 PNGs: {png_count}");
}

#[test]
fn test_audio_cascade_zero_vram_valid_wav() {
    let tmp = TempDir::new().unwrap();
    let inv = probe_audio_engines(tmp.path());
    assert!(!inv.active_tts_engine.is_empty());
    assert!(!inv.active_stt_engine.is_empty());

    let wav = generate_pcm_wav_buffer(250, 440.0, 0.3);
    assert!(wav.len() > 44);
    assert_eq!(&wav[0..4], b"RIFF");
    assert_eq!(&wav[8..12], b"WAVE");

    let bc = backchannel_pcm_wav("Aham...");
    assert_eq!(&bc[0..4], b"RIFF");

    let chime = handoff_chime_wav();
    assert_eq!(&chime[0..4], b"RIFF");

    let req = VoiceSynthesisRequestDto {
        input: "Olá! Tudo **pronto** para continuar?".to_string(),
        voice: None,
        model: None,
        response_format: Some("wav".to_string()),
        speed: Some(1.0),
        apply_prosody_shaping: Some(true),
        cognitive_mode: Some(CognitiveVoiceMode::Auto),
    };

    let tts = synthesize_speech_cascade(&req, CognitiveVoiceMode::Auto, 250, tmp.path()).unwrap();
    assert_eq!(tts.vram_consumed_mb, ZERO_AUDIO_VRAM_CONSUMPTION_MB);
    assert!(!tts.shaped_text.contains("**"));
    assert!(!tts.audio_base64.is_empty());
    assert!(tts.audio_bytes_len > 44);
    assert_eq!(tts.audio_format, "wav");

    let stt = transcribe_audio_cascade(&wav, "wav", "pt-BR", 400, tmp.path()).unwrap();
    assert_eq!(stt.vram_consumed_mb, ZERO_AUDIO_VRAM_CONSUMPTION_MB);
    assert_eq!(stt.language, "pt-BR");
    assert!(stt.is_silence_hallucination_filtered);
}

#[test]
fn test_bcp47_language_normalization_and_toolchain_probe() {
    assert_eq!(normalize_stt_bcp47_language(""), "pt-BR");
    assert_eq!(normalize_stt_bcp47_language("pt"), "pt-BR");
    assert_eq!(normalize_stt_bcp47_language("pt_br"), "pt-BR");
    assert_eq!(normalize_stt_bcp47_language("PT-BR"), "pt-BR");
    assert_eq!(normalize_stt_bcp47_language("en-US"), "en-US");

    let tmp = TempDir::new().unwrap();
    let probe = discover_hermes_audio_tools(tmp.path());
    assert!(!probe.inventory.active_tts_engine.is_empty());
    assert!(!probe.inventory.active_stt_engine.is_empty());

    // Local model directory detection must identify any installed faster-whisper snapshot without downloading in tests
    let fw_dir = tmp.path().join("models").join("audio").join("faster-whisper-base");
    std::fs::create_dir_all(&fw_dir).unwrap();
    std::fs::write(fw_dir.join("model.bin"), b"stub").unwrap();
    let prov = provision_local_faster_whisper_model(tmp.path()).unwrap();
    assert_eq!(prov.as_deref(), Some(fw_dir.as_path()));
}
