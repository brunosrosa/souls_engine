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
