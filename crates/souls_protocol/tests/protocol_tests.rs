use souls_protocol::dto::{
    AllowedRoute, ContextLedgerEntry, E3EfficiencyScore, EpistemicMemoryRecord,
    EpistemicPartition, EwmaMetrics, FinOpsCostMetrics, HardwareTelemetry, ModelTier,
    SubAgentDiaryEntry, SwarmHandoffPayload, TelemetryBatchRecord,
};
use souls_protocol::error::{JsonRpcError, JsonRpcRequest, JsonRpcResponse, McpDomainError};

#[test]
fn test_finops_e3_cost_metrics_serde_roundtrip() {
    let metrics = FinOpsCostMetrics {
        prefill_tokens: 1500,
        generation_tokens: 250,
        total_tokens: 1750,
        latency_ms: 124.5,
        cost_usd: 0.0035,
        quality_score: Some(0.95),
        e3_score: 0.88,
        deviation_ratio: Some(0.02),
    };

    let serialized = serde_json::to_string(&metrics).expect("Serialization failed");
    let deserialized: FinOpsCostMetrics =
        serde_json::from_str(&serialized).expect("Deserialization failed");
    assert_eq!(metrics, deserialized);

    let e3 = E3EfficiencyScore {
        efficiency: 0.92,
        economics: 0.85,
        execution: 0.98,
        composite_score: 0.916,
    };
    let e3_json = serde_json::to_string(&e3).expect("Serialization failed");
    let e3_de: E3EfficiencyScore = serde_json::from_str(&e3_json).expect("Deserialization failed");
    assert_eq!(e3, e3_de);

    let route = AllowedRoute::OriginalRoute(ModelTier::PremiumCloud);
    let route_json = serde_json::to_string(&route).expect("Serialization failed");
    let route_de: AllowedRoute = serde_json::from_str(&route_json).expect("Deserialization failed");
    assert_eq!(route, route_de);

    let telemetry_record = TelemetryBatchRecord {
        id: "evt_1234".to_string(),
        session_id: "sess_5678".to_string(),
        event_type: "iron_cost_approved".to_string(),
        execution_tier: "Tier1".to_string(),
        payload_json: "{\"tokens\": 1750}".to_string(),
        latency_ms: 45.2,
        tokens_input: 1500,
        tokens_output: 250,
        direct_cost_usd: 0.0012,
        e3_score: 0.94,
        created_at: 1725750000000,
    };
    let tel_json = serde_json::to_string(&telemetry_record).expect("Serialization failed");
    let tel_de: TelemetryBatchRecord =
        serde_json::from_str(&tel_json).expect("Deserialization failed");
    assert_eq!(telemetry_record, tel_de);

    let epistemic = EpistemicMemoryRecord {
        id: "mem_abc".to_string(),
        session_id: "sess_5678".to_string(),
        category: "finops".to_string(),
        content: "Rule for budget pacing".to_string(),
        partition: EpistemicPartition::Stable,
        salience: 1.0,
        access_count: 5,
        source_ref: Some("ADR-046".to_string()),
        created_at: 1725750000000,
        last_accessed_at: 1725750500000,
        updated_at: 1725750500000,
    };
    let ep_json = serde_json::to_string(&epistemic).expect("Serialization failed");
    let ep_de: EpistemicMemoryRecord =
        serde_json::from_str(&ep_json).expect("Deserialization failed");
    assert_eq!(epistemic, ep_de);
}

#[test]
fn test_hardware_telemetry_and_ewma_serde_roundtrip() {
    let hw = HardwareTelemetry {
        vram_used_mb: 4200,
        vram_total_mb: 6144,
        ram_used_mb: 18432,
        ram_total_mb: 32768,
        cpu_temp_celsius: 62.5,
        gpu_temp_celsius: 74.0,
        pcie_bandwidth_gbps: 34.5,
        thermal_throttle: false,
        timestamp_epoch_ms: 1725750001000,
    };

    let serialized = serde_json::to_string(&hw).expect("Serialization failed");
    let deserialized: HardwareTelemetry =
        serde_json::from_str(&serialized).expect("Deserialization failed");
    assert_eq!(hw, deserialized);

    let ewma = EwmaMetrics {
        alpha: 0.3,
        ewma_latency_ms: 142.6,
        peak_latency_ms: 250.0,
        sample_count: 64,
    };
    let ewma_json = serde_json::to_string(&ewma).expect("Serialization failed");
    let ewma_de: EwmaMetrics = serde_json::from_str(&ewma_json).expect("Deserialization failed");
    assert_eq!(ewma, ewma_de);
}

#[test]
fn test_swarm_and_context_ledger_serde_roundtrip() {
    let diary = SubAgentDiaryEntry {
        id: Some(1),
        agent_id: "agent_sub_01".to_string(),
        session_id: "sess_main".to_string(),
        content: "Extracted symbols and validated Myers patch.".to_string(),
        created_at: 1725750002000,
        tags: vec!["code_surgery".to_string(), "ast".to_string()],
        parent_agent_id: Some("agent_orchestrator".to_string()),
        turn_index: Some(2),
    };

    let diary_json = serde_json::to_string(&diary).expect("Serialization failed");
    let diary_de: SubAgentDiaryEntry =
        serde_json::from_str(&diary_json).expect("Deserialization failed");
    assert_eq!(diary, diary_de);

    let ledger = ContextLedgerEntry {
        ledger_id: "cld_999".to_string(),
        session_id: "sess_main".to_string(),
        sender_agent_id: "agent_sub_01".to_string(),
        recipient_agent_id: Some("agent_orchestrator".to_string()),
        payload_encrypted: vec![0xDE, 0xAD, 0xBE, 0xEF, 0x01, 0x02],
        nonce: vec![0x12, 0x34, 0x56, 0x78],
        signature: Some("ed25519_sig_mock".to_string()),
        checksum_sha256: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
            .to_string(),
        timestamp_epoch_ms: 1725750003000,
    };

    let ledger_json = serde_json::to_string(&ledger).expect("Serialization failed");
    let ledger_de: ContextLedgerEntry =
        serde_json::from_str(&ledger_json).expect("Deserialization failed");
    assert_eq!(ledger, ledger_de);

    let handoff = SwarmHandoffPayload {
        handoff_id: "hnd_456".to_string(),
        source_agent: "agent_sub_01".to_string(),
        target_agent: "agent_sub_02".to_string(),
        task_description: "Run verification on ast parsing".to_string(),
        context_summary: "Myers diff validated with 0 conflicts".to_string(),
        variables: serde_json::json!({
            "target_crate": "souls_protocol",
            "tests_passed": true
        }),
        created_at: 1725750004000,
    };

    let handoff_json = serde_json::to_string(&handoff).expect("Serialization failed");
    let handoff_de: SwarmHandoffPayload =
        serde_json::from_str(&handoff_json).expect("Deserialization failed");
    assert_eq!(handoff, handoff_de);
}

#[test]
fn test_mcp_domain_error_codes_mapping() {
    assert_eq!(McpDomainError::InternalEnginePanic.jsonrpc_code(), -32000);
    assert_eq!(McpDomainError::SymbolNotFound.jsonrpc_code(), -32001);
    assert_eq!(McpDomainError::VramThermalThrottled.jsonrpc_code(), -32002);
    assert_eq!(McpDomainError::FileTooLargeForAST.jsonrpc_code(), -32003);
    assert_eq!(McpDomainError::HitlDenied.jsonrpc_code(), -32004);
    assert_eq!(McpDomainError::StructuredFailure.jsonrpc_code(), -32005);

    // Test reverse lookup
    assert_eq!(
        McpDomainError::from_code(-32000),
        Some(McpDomainError::InternalEnginePanic)
    );
    assert_eq!(
        McpDomainError::from_code(-32001),
        Some(McpDomainError::SymbolNotFound)
    );
    assert_eq!(
        McpDomainError::from_code(-32002),
        Some(McpDomainError::VramThermalThrottled)
    );
    assert_eq!(
        McpDomainError::from_code(-32003),
        Some(McpDomainError::FileTooLargeForAST)
    );
    assert_eq!(
        McpDomainError::from_code(-32004),
        Some(McpDomainError::HitlDenied)
    );
    assert_eq!(
        McpDomainError::from_code(-32005),
        Some(McpDomainError::StructuredFailure)
    );
    assert_eq!(McpDomainError::from_code(-99999), None);
}

#[test]
fn test_mcp_error_jsonrpc_conversion_and_serialization() {
    let err = McpDomainError::SymbolNotFound;
    let jsonrpc_err: JsonRpcError = err.clone().into();
    assert_eq!(jsonrpc_err.code, -32001);
    assert!(jsonrpc_err.message.contains("SymbolNotFound"));

    let err_with_data = err.to_jsonrpc_error_with_data(serde_json::json!({
        "symbol_query": "UnknownStruct",
        "path": "crates/souls_protocol/src/lib.rs"
    }));
    assert_eq!(err_with_data.code, -32001);
    assert!(err_with_data.data.is_some());

    let response = JsonRpcResponse::<String> {
        jsonrpc: "2.0".to_string(),
        id: serde_json::json!("req_001"),
        result: None,
        error: Some(err_with_data),
    };

    let serialized = serde_json::to_string(&response).expect("Serialization failed");
    assert!(serialized.contains("-32001"));
    assert!(serialized.contains("UnknownStruct"));

    let request = JsonRpcRequest::<serde_json::Value> {
        jsonrpc: "2.0".to_string(),
        id: serde_json::json!("req_002"),
        method: "tools/call".to_string(),
        params: Some(serde_json::json!({
            "name": "outline",
            "arguments": { "path": "crates/souls_protocol/src/dto.rs" }
        })),
    };

    let req_serialized = serde_json::to_string(&request).expect("Serialization failed");
    let req_de: JsonRpcRequest<serde_json::Value> =
        serde_json::from_str(&req_serialized).expect("Deserialization failed");
    assert_eq!(request, req_de);
}

#[test]
fn test_multimodal_enums_snake_case_and_defaults() {
    use souls_protocol::multimodal::{
        AudioEngineTier, CognitiveVoiceMode, DetectedEmotionTag, TurnTakingVerdict,
        VisionEscalationReason, VisualAnalysisMode,
    };

    assert_eq!(CognitiveVoiceMode::default(), CognitiveVoiceMode::Auto);
    assert_eq!(DetectedEmotionTag::default(), DetectedEmotionTag::Neutral);
    assert_eq!(
        TurnTakingVerdict::default(),
        TurnTakingVerdict::TakeTurnImmediate
    );
    assert_eq!(
        AudioEngineTier::default(),
        AudioEngineTier::WindowsNativeFallback
    );
    assert_eq!(
        VisionEscalationReason::default(),
        VisionEscalationReason::NoneAxSufficient
    );
    assert_eq!(VisualAnalysisMode::default(), VisualAnalysisMode::Auto);

    let enum_cases = [
        (
            serde_json::to_string(&CognitiveVoiceMode::CrisisParalysis).unwrap(),
            "\"crisis_paralysis\"",
        ),
        (
            serde_json::to_string(&CognitiveVoiceMode::HandsOn).unwrap(),
            "\"hands_on\"",
        ),
        (
            serde_json::to_string(&DetectedEmotionTag::Hesitant).unwrap(),
            "\"hesitant\"",
        ),
        (
            serde_json::to_string(&TurnTakingVerdict::ContemplativeWait).unwrap(),
            "\"contemplative_wait\"",
        ),
        (
            serde_json::to_string(&AudioEngineTier::OnnxPrimary).unwrap(),
            "\"onnx_primary\"",
        ),
        (
            serde_json::to_string(&VisionEscalationReason::CanvasOrWebGlDetected).unwrap(),
            "\"canvas_or_web_gl_detected\"",
        ),
        (
            serde_json::to_string(&VisualAnalysisMode::DeterministicSomOnly).unwrap(),
            "\"deterministic_som_only\"",
        ),
    ];

    for (actual, expected) in enum_cases {
        assert_eq!(actual, expected);
    }

    assert_eq!(
        DetectedEmotionTag::from_str_lossy("risada"),
        DetectedEmotionTag::Laughter
    );
    assert_eq!(
        DetectedEmotionTag::from_str_lossy("unknown_tag"),
        DetectedEmotionTag::Neutral
    );
}

#[test]
fn test_multimodal_dtos_serde_roundtrip() {
    use souls_protocol::multimodal::{
        AXTreeDiffSummary, AudioEngineTier, BlindSpotAssessment, CognitiveVoiceMode,
        DetectedEmotionTag, HermesVoiceCustomsReportDto, OpenAiSpeechRequest,
        OpenAiTranscriptionResponse, PcompTurnEvaluationDto, SomOverlayMark, TurnTakingVerdict,
        UserRapportProfileDto, VisionEscalationReason, VisualSnapshotResultDto,
        VoiceSynthesisRequestDto, VoiceSynthesisResultDto, VoiceTranscriptionResultDto,
    };

    let som_mark = SomOverlayMark {
        index: 1,
        node_id: "btn_save".to_string(),
        role: "button".to_string(),
        label: "Salvar".to_string(),
        bounds: [10.0, 20.0, 120.0, 48.0],
        click_point: (65.0, 34.0),
    };
    let som_de: SomOverlayMark =
        serde_json::from_str(&serde_json::to_string(&som_mark).unwrap()).unwrap();
    assert_eq!(som_mark, som_de);

    let blind_spot = BlindSpotAssessment {
        requires_pixel_snapshot: true,
        reason: VisionEscalationReason::CanvasOrWebGlDetected,
        actionable_node_count: 2,
        canvas_node_count: 1,
        coverage_ratio: 0.18,
        recommendation_pt: "Canvas detectado; captura visual recomendada.".to_string(),
    };
    let blind_de: BlindSpotAssessment =
        serde_json::from_str(&serde_json::to_string(&blind_spot).unwrap()).unwrap();
    assert_eq!(blind_spot, blind_de);

    let ax_diff = AXTreeDiffSummary {
        added_nodes: vec!["btn_confirm".to_string()],
        removed_nodes: vec!["spinner_loading".to_string()],
        changed_states: vec!["input_email:focused".to_string()],
        compact_lean_diff: "+btn_confirm -spinner_loading ~input_email".to_string(),
        full_lean_bytes: 2048,
        diff_lean_bytes: 128,
        token_savings_pct: 93.75,
    };
    let ax_de: AXTreeDiffSummary =
        serde_json::from_str(&serde_json::to_string(&ax_diff).unwrap()).unwrap();
    assert_eq!(ax_diff, ax_de);

    let visual_snapshot = VisualSnapshotResultDto {
        snapshot_id: "snap_001".to_string(),
        window_title: "Editor".to_string(),
        app_name: "code.exe".to_string(),
        width: 1920,
        height: 1080,
        sha256_hash: "abc123".to_string(),
        screen_unchanged: false,
        som_marks: vec![som_mark],
        blind_spot,
        image_path: Some("Z:\\souls_engine\\.souls_data\\snap.png".to_string()),
        image_base64_png: None,
        visual_analysis: Some("Botao Salvar visivel no topo.".to_string()),
        analysis_engine_used: "deterministic_som".to_string(),
        vram_consumed_mb: 0,
        latency_ms: 14.2,
    };
    let visual_de: VisualSnapshotResultDto =
        serde_json::from_str(&serde_json::to_string(&visual_snapshot).unwrap()).unwrap();
    assert_eq!(visual_snapshot, visual_de);

    let turn_eval = PcompTurnEvaluationDto {
        transcript_tail: "entao eu acho que...".to_string(),
        silence_duration_ms: 340,
        ends_with_open_connective: true,
        syntactic_closure_score: 0.28,
        acoustic_stress_score: 0.41,
        recommended_wait_ms: 520,
        verdict: TurnTakingVerdict::ContemplativeWait,
        suggested_backchannel: Some("uhum".to_string()),
    };
    let turn_de: PcompTurnEvaluationDto =
        serde_json::from_str(&serde_json::to_string(&turn_eval).unwrap()).unwrap();
    assert_eq!(turn_eval, turn_de);

    let rapport = UserRapportProfileDto {
        user_id: "rafael".to_string(),
        formality_score: 2.5,
        avg_sentence_length: 10,
        preferred_pacing: "short_direct".to_string(),
        slang_lexicon: vec!["bora".to_string(), "massa".to_string()],
        mirroring_ratio: 0.3,
        active_cognitive_mode: CognitiveVoiceMode::HandsOn,
        contemplative_base_ms: 280,
        backchannel_enabled: true,
        last_emotion: DetectedEmotionTag::Focused,
        turn_count: 14,
        updated_at: 1763856481,
    };
    let rapport_de: UserRapportProfileDto =
        serde_json::from_str(&serde_json::to_string(&rapport).unwrap()).unwrap();
    assert_eq!(rapport, rapport_de);

    let stt = VoiceTranscriptionResultDto {
        text: "Entao bora testar isso".to_string(),
        clean_text: "Entao bora testar isso".to_string(),
        emotion_tag: DetectedEmotionTag::Focused,
        emotion_confidence: 0.87,
        is_silence_hallucination_filtered: false,
        turn_evaluation: turn_eval,
        engine_tier: AudioEngineTier::OnnxPrimary,
        engine_name: "sherpa-onnx-whisper".to_string(),
        language: "pt-BR".to_string(),
        vram_consumed_mb: 0,
        latency_ms: 68.5,
    };
    let stt_de: VoiceTranscriptionResultDto =
        serde_json::from_str(&serde_json::to_string(&stt).unwrap()).unwrap();
    assert_eq!(stt, stt_de);

    let tts_req = VoiceSynthesisRequestDto {
        input: "Fechado — vamos direto para o proximo passo.".to_string(),
        voice: Some("pt_BR-faber-medium".to_string()),
        model: Some("piper-pt-br".to_string()),
        response_format: Some("wav".to_string()),
        speed: Some(1.05),
        apply_prosody_shaping: Some(true),
        cognitive_mode: Some(CognitiveVoiceMode::HandsOn),
    };
    let tts_req_de: VoiceSynthesisRequestDto =
        serde_json::from_str(&serde_json::to_string(&tts_req).unwrap()).unwrap();
    assert_eq!(tts_req, tts_req_de);

    let tts_res = VoiceSynthesisResultDto {
        shaped_text: "Fechado — vamos direto para o proximo passo.".to_string(),
        sentences_chunks: vec!["Fechado — vamos direto para o proximo passo.".to_string()],
        audio_format: "wav".to_string(),
        audio_bytes_len: 1024,
        audio_base64: "UklGRg==".to_string(),
        output_path: None,
        contemplative_delay_ms: 250,
        engine_tier: AudioEngineTier::LocalSecondary,
        engine_name: "piper-tts".to_string(),
        vram_consumed_mb: 0,
        latency_ms: 42.0,
    };
    let tts_res_de: VoiceSynthesisResultDto =
        serde_json::from_str(&serde_json::to_string(&tts_res).unwrap()).unwrap();
    assert_eq!(tts_res, tts_res_de);

    let report = HermesVoiceCustomsReportDto {
        config_path: "C:\\Users\\user\\.hermes\\config.yaml".to_string(),
        stt_enabled: true,
        current_stt_language: "pt".to_string(),
        current_stt_provider: "local".to_string(),
        current_tts_provider: "piper".to_string(),
        current_voice_mode: "hands_on".to_string(),
        is_synced_with_souls_pt_br: true,
        issues_found: vec![],
        human_summary_pt: "Voz configurada em portugues local.".to_string(),
    };
    let report_de: HermesVoiceCustomsReportDto =
        serde_json::from_str(&serde_json::to_string(&report).unwrap()).unwrap();
    assert_eq!(report, report_de);

    let speech_req = OpenAiSpeechRequest {
        model: Some("tts-1".to_string()),
        input: "Ola mundo".to_string(),
        voice: Some("alloy".to_string()),
        response_format: Some("mp3".to_string()),
        speed: Some(1.0),
    };
    let speech_req_de: OpenAiSpeechRequest =
        serde_json::from_str(&serde_json::to_string(&speech_req).unwrap()).unwrap();
    assert_eq!(speech_req, speech_req_de);

    let trans_res = OpenAiTranscriptionResponse {
        text: "Tudo certo".to_string(),
        emotion: Some(DetectedEmotionTag::Happy),
        language: Some("pt".to_string()),
    };
    let trans_res_de: OpenAiTranscriptionResponse =
        serde_json::from_str(&serde_json::to_string(&trans_res).unwrap()).unwrap();
    assert_eq!(trans_res, trans_res_de);
}

#[test]
fn test_cognitive_voice_mode_helpers_and_ephemeral_notes() {
    use souls_protocol::multimodal::{CognitiveVoiceMode, UserRapportProfileDto};

    assert_eq!(
        CognitiveVoiceMode::from_str_lossy("hands_on"),
        CognitiveVoiceMode::HandsOn
    );
    assert_eq!(
        CognitiveVoiceMode::from_str_lossy("hands-on"),
        CognitiveVoiceMode::HandsOn
    );
    assert_eq!(
        CognitiveVoiceMode::from_str_lossy("mão na massa"),
        CognitiveVoiceMode::HandsOn
    );
    assert_eq!(
        CognitiveVoiceMode::from_str_lossy("sparring"),
        CognitiveVoiceMode::Sparring
    );
    assert_eq!(
        CognitiveVoiceMode::from_str_lossy("descompressao"),
        CognitiveVoiceMode::Decompression
    );
    assert_eq!(
        CognitiveVoiceMode::from_str_lossy("crise"),
        CognitiveVoiceMode::CrisisParalysis
    );
    assert_eq!(
        CognitiveVoiceMode::from_str_lossy("filosofico"),
        CognitiveVoiceMode::Philosophical
    );
    assert_eq!(
        CognitiveVoiceMode::from_str_lossy("desconhecido"),
        CognitiveVoiceMode::Auto
    );

    let modes = [
        CognitiveVoiceMode::Auto,
        CognitiveVoiceMode::HandsOn,
        CognitiveVoiceMode::Sparring,
        CognitiveVoiceMode::Decompression,
        CognitiveVoiceMode::CrisisParalysis,
        CognitiveVoiceMode::Philosophical,
    ];

    let casual_profile = UserRapportProfileDto {
        formality_score: 2.0,
        mirroring_ratio: 0.90, // exceeds 0.35, must be clamped inside ephemeral_turn_note_pt
        slang_lexicon: vec!["show".to_string(), "massa".to_string()],
        ..UserRapportProfileDto::default()
    };

    for mode in modes {
        assert!(!mode.as_str().is_empty());
        assert!(!mode.human_label_pt().is_empty());
        assert!(!mode.human_description_pt().is_empty());

        let note = mode.ephemeral_turn_note_pt(&casual_profile);
        assert!(note.contains("35%"));
        assert!(note.contains("show, massa"));
        assert!(note.contains("..."));
        assert!(note.contains("—"));
        assert!(note.contains(','));
    }

    let formal_profile = UserRapportProfileDto {
        formality_score: 8.5,
        mirroring_ratio: 0.20,
        slang_lexicon: vec!["mano".to_string()],
        ..UserRapportProfileDto::default()
    };
    let formal_note = CognitiveVoiceMode::Sparring.ephemeral_turn_note_pt(&formal_profile);
    assert!(formal_note.contains("evite gírias"));
    assert!(!formal_note.contains("mano"));
}

#[test]
fn test_user_rapport_profile_default_and_sanitize() {
    use souls_protocol::multimodal::{
        clamp_mirroring_ratio, CognitiveVoiceMode, DetectedEmotionTag, UserRapportProfileDto,
    };

    let default_profile = UserRapportProfileDto::default();
    assert_eq!(default_profile.user_id, "default");
    assert_eq!(default_profile.formality_score, 3.5);
    assert_eq!(default_profile.avg_sentence_length, 12);
    assert_eq!(default_profile.preferred_pacing, "short_direct");
    assert_eq!(default_profile.mirroring_ratio, 0.25);
    assert_eq!(
        default_profile.active_cognitive_mode,
        CognitiveVoiceMode::Auto
    );
    assert_eq!(default_profile.contemplative_base_ms, 250);
    assert!(default_profile.backchannel_enabled);
    assert_eq!(default_profile.last_emotion, DetectedEmotionTag::Neutral);

    assert_eq!(clamp_mirroring_ratio(-0.5), 0.0);
    assert_eq!(clamp_mirroring_ratio(0.22), 0.22);
    assert_eq!(clamp_mirroring_ratio(0.85), 0.35);

    let mut dirty = UserRapportProfileDto {
        formality_score: 25.0,
        avg_sentence_length: 0,
        mirroring_ratio: 0.99,
        contemplative_base_ms: 40,
        ..UserRapportProfileDto::default()
    };
    dirty.sanitize();
    assert_eq!(dirty.formality_score, 10.0);
    assert_eq!(dirty.avg_sentence_length, 1);
    assert_eq!(dirty.mirroring_ratio, 0.35);
    assert_eq!(dirty.contemplative_base_ms, 100);

    dirty.formality_score = -3.0;
    dirty.mirroring_ratio = -1.0;
    dirty.contemplative_base_ms = 5000;
    dirty.sanitize();
    assert_eq!(dirty.formality_score, 0.0);
    assert_eq!(dirty.mirroring_ratio, 0.0);
    assert_eq!(dirty.contemplative_base_ms, 1000);
}

#[test]
fn test_v094_harness_evolve_dtos_and_slots_roundtrip() {
    use souls_protocol::harness_evolve::{
        CatnapConfigDto, CatnapSliceRecordDto, CatnapStateDto, ConstitutionalPinStatusDto,
        CordisFiberDto, CordisRollbackReportDto, DomainExperienceEntryDto, HarnessComponentSlot,
        MemoryTombstoneDto, RalphSkillCandidateDto, WeEvolveRatingDto,
    };

    assert_eq!(HarnessComponentSlot::all().len(), 8);
    assert!(HarnessComponentSlot::C1SystemPrompt.is_constitutionally_pinned());
    assert!(!HarnessComponentSlot::C4Skills.is_constitutionally_pinned());
    assert_eq!(
        HarnessComponentSlot::from_str_lossy("c5"),
        Some(HarnessComponentSlot::C5ToolGuidance)
    );
    assert_eq!(HarnessComponentSlot::C8PluginsHooks.as_code(), "c8");

    let cfg = CatnapConfigDto::default();
    assert!(cfg.enabled);
    assert_eq!(cfg.min_idle_secs, 180);
    assert_eq!(cfg.max_gpu_temp_c, 62.0);
    assert_eq!(cfg.max_daily_budget_usd, 0.0);

    let fiber = CordisFiberDto {
        fiber_id: "fib_01".to_string(),
        slot: HarnessComponentSlot::C5ToolGuidance,
        title_pt: "Preferir esqueleto antes de editar".to_string(),
        benefit_summary_pt: "Economiza 75% de tokens".to_string(),
        ephemeral_cue_text: "Use outline antes de read_file".to_string(),
        target_task_type: Some("coding".to_string()),
        coeffect_dependencies: vec!["local_llamacpp".to_string()],
        active: true,
        paused_by_coeffect: false,
        undo_operations_count: 2,
        elo_rating: 1216.0,
        ema_score: 0.56,
        drift_streak: 0,
        created_at: 1795440000,
    };
    let json_fiber = serde_json::to_string(&fiber).unwrap();
    let de_fiber: CordisFiberDto = serde_json::from_str(&json_fiber).unwrap();
    assert_eq!(fiber, de_fiber);

    let pin = ConstitutionalPinStatusDto {
        file_path_posix: "AGENTS.md".to_string(),
        expected_sha256: "abc".to_string(),
        actual_sha256: "abc".to_string(),
        verified: true,
        fail_closed_triggered: false,
        checked_at: 1795440000,
    };
    let rollback = CordisRollbackReportDto {
        fiber_id: "fib_01".to_string(),
        reverted_operations: vec!["op2".to_string(), "op1".to_string()],
        elapsed_us: 42,
        restored_cleanly: true,
    };
    let state = CatnapStateDto {
        state: "standby".to_string(),
        idle_elapsed_secs: 10,
        last_preemption_latency_ms: 0.2,
        slices_completed_today: 1,
        spent_today_usd: 0.0,
        last_cursor_checkpoint: "cur_1".to_string(),
        pending_failed_trajectories: 0,
        config: cfg,
    };
    let slice = CatnapSliceRecordDto {
        slice_id: "sl_1".to_string(),
        slice_kind: "shadow_probe_ppc".to_string(),
        runmark_fingerprint: "fp_1".to_string(),
        cursor_checkpoint: "cur_1".to_string(),
        status: "completed".to_string(),
        duration_ms: 12.5,
        cost_usd: 0.0,
        summary_pt: "Sondagem sem regressões".to_string(),
        created_at: 1795440000,
    };
    let tomb = MemoryTombstoneDto {
        tombstone_id: "tmb_1".to_string(),
        superseded_memory_id: "m_old".to_string(),
        superseded_by_memory_id: "m_new".to_string(),
        category: "architecture".to_string(),
        superseded_content_preview: "Usar X".to_string(),
        new_content_preview: "Substituir X por Y".to_string(),
        reason_pt: "Decisão substituída em tempo real".to_string(),
        status: "superseded".to_string(),
        created_at: 1795440000,
        restored_at: None,
    };
    let rating = WeEvolveRatingDto {
        target_id: "fib_01".to_string(),
        target_kind: "cordis_fiber".to_string(),
        elo_rating: 1216.0,
        ema_score: 0.56,
        total_matches: 1,
        last_signal: "test_success".to_string(),
        updated_at: 1795440000,
    };
    let ralph = RalphSkillCandidateDto {
        candidate_id: "ralph_1".to_string(),
        slug: "fix-lock-win32".to_string(),
        title_pt: "Correção de Lock Win32".to_string(),
        source_session_id: "sess_1".to_string(),
        problem_pattern_pt: "Erro 32 em SQLite".to_string(),
        solution_summary_pt: "Aplicar WAL + busy_timeout".to_string(),
        e3_score: 0.88,
        elo_rating: 1220.0,
        quarantine_path: ".souls_data/skills_hub/quarantine/fix-lock-win32/SKILL.md".to_string(),
        diff_preview: "+ # Fix Lock".to_string(),
        status: "quarantined".to_string(),
        created_at: 1795440000,
    };
    let exp = DomainExperienceEntryDto {
        episode_id: "ep_1".to_string(),
        domain_tag: "rust-bare-metal".to_string(),
        cue_pattern: "win32 file lock".to_string(),
        always_on_chars: 420,
        skills_count: 1,
        subagents_count: 1,
        scripts_count: 0,
        hooks_count: 1,
        runmark_fingerprint: "fp_1".to_string(),
        experience_summary_pt: "Solução determinística validada".to_string(),
        created_at: 1795440000,
    };
    assert!(pin.verified && rollback.restored_cleanly);
    assert_eq!(state.state, "standby");
    assert_eq!(slice.cost_usd, 0.0);
    assert_eq!(tomb.status, "superseded");
    assert_eq!(rating.elo_rating, 1216.0);
    assert_eq!(ralph.status, "quarantined");
    assert_eq!(exp.always_on_chars, 420);
}
