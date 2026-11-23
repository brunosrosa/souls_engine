use std::collections::BTreeMap;

use souls_core::{
    compute_runmark_fingerprint, fold_aci_observation, is_non_idempotent_tool,
    is_within_quiet_hours, normalize_posix_rel_path, scrub_candidate_environment, CatnapMonitor,
    ConstitutionalPinVerifier, CordisMicrokernel, EffectUndoOp,
};
use souls_protocol::{CatnapConfigDto, HarnessComponentSlot};

#[test]
fn test_v094_surface_posix_scrub_and_constitutional_pin_fail_closed() {
    assert_eq!(
        normalize_posix_rel_path(r".\scripts\summarize.py").unwrap(),
        "scripts/summarize.py"
    );
    assert_eq!(
        normalize_posix_rel_path(r"skills\souls-core\SKILL.md").unwrap(),
        "skills/souls-core/SKILL.md"
    );
    assert!(normalize_posix_rel_path(r"..\outside.txt").is_err());
    assert!(normalize_posix_rel_path(r"C:\Windows\System32").is_err());

    let mut env = BTreeMap::new();
    env.insert("GITHUB_TOKEN".to_string(), "ghp_secret".to_string());
    env.insert("OPENAI_API_KEY".to_string(), "sk-secret".to_string());
    env.insert("SOULS_CUSTOM_SECRET".to_string(), "123".to_string());
    env.insert("PATH".to_string(), "/usr/bin".to_string());
    let scrubbed = scrub_candidate_environment(&mut env);
    assert_eq!(scrubbed, 3);
    assert_eq!(env.get("PATH").map(String::as_str), Some("/usr/bin"));
    assert!(!env.contains_key("GITHUB_TOKEN"));

    assert!(ConstitutionalPinVerifier::reject_if_c1_mutated(HarnessComponentSlot::C1SystemPrompt).is_err());
    assert!(ConstitutionalPinVerifier::reject_if_c1_mutated(HarnessComponentSlot::C5ToolGuidance).is_ok());

    let temp_dir = tempfile::tempdir().unwrap();
    let soul_path = temp_dir.path().join("SOUL.md");
    std::fs::write(&soul_path, "# Soul Core\r\nBe helpful and sovereign.\r\n").unwrap();

    let verifier = ConstitutionalPinVerifier::new();
    let pin_status = verifier.pin_file_from_disk("SOUL.md", &soul_path).unwrap();
    assert!(pin_status.verified);
    assert!(!pin_status.fail_closed_triggered);

    // Same semantic text with LF instead of CRLF must still match pin!
    assert!(verifier
        .assert_candidate_matches_pin("SOUL.md", b"# Soul Core\nBe helpful and sovereign.\n")
        .is_ok());

    // Tampering triggers fail-closed mismatch
    std::fs::write(&soul_path, "# Soul Core\nGeneric corporate bot.\n").unwrap();
    let check = verifier.verify_or_fail_closed();
    assert_eq!(check.len(), 1);
    assert!(!check[0].verified);
    assert!(check[0].fail_closed_triggered);
    assert_eq!(check[0].pin_mode, "fail_closed");

    // Watch mode tracks legitimate Hermes memory writes (verified=false) without fail-closed freeze
    let memory_path = temp_dir.path().join("MEMORY.md");
    std::fs::write(&memory_path, "- Prefer local zero-cost inference.\n").unwrap();
    let watch_pin = verifier
        .pin_file_from_disk_with_mode("hermes/memories/MEMORY.md", &memory_path, "watch")
        .unwrap();
    assert_eq!(watch_pin.pin_mode, "watch");
    assert!(watch_pin.verified);

    std::fs::write(&memory_path, "- Prefer local zero-cost inference.\n- New fact saved.\n").unwrap();
    let check_mixed = verifier.verify_or_fail_closed();
    let mem_check = check_mixed
        .iter()
        .find(|p| p.file_path_posix == "hermes/memories/MEMORY.md")
        .unwrap();
    assert!(!mem_check.verified);
    assert!(!mem_check.fail_closed_triggered);

    // 1-byte tamper in SOUL.md still triggers fail_closed_triggered=true until human re-sign
    let soul_check = check_mixed
        .iter()
        .find(|p| p.file_path_posix == "SOUL.md")
        .unwrap();
    assert!(soul_check.fail_closed_triggered);

    let resigned = verifier.resign_all_disk_pins();
    assert!(resigned.iter().all(|p| p.verified && !p.fail_closed_triggered));

    // ACI folding and timeout_retry=false idempotency guard
    let long_output = (0..40)
        .map(|i| {
            if i == 15 {
                "ERROR: missing closing bracket at line 15".to_string()
            } else {
                format!("line {i}: compilation progress diagnostic output payload")
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    let folded = fold_aci_observation("cargo_check", &long_output, 350);
    assert!(folded.contains("ERROR: missing closing bracket"));
    assert!(folded.chars().count() <= 350);

    assert!(is_non_idempotent_tool("google_gmail", Some("send")));
    assert!(!is_non_idempotent_tool("google_gmail", Some("triage")));
    assert!(is_non_idempotent_tool("inbox_resolve", None));
}

#[test]
fn test_v094_cordis_lifo_coeffects_and_hemispheric_drift_rollback() {
    let cordis = CordisMicrokernel::new();

    // Fail-closed for c1 and Quarantine enforcement for c4/c6
    assert!(cordis
        .mount_fiber(
            None,
            HarnessComponentSlot::C1SystemPrompt,
            "Bad",
            "Should fail",
            "cue",
            None,
            vec![],
            vec![],
            1000,
        )
        .is_err());
    assert!(cordis
        .mount_fiber(
            None,
            HarnessComponentSlot::C4Skills,
            "Skill bypass",
            "Should require quarantine",
            "cue",
            None,
            vec![],
            vec![],
            1000,
        )
        .is_err());

    // Mount tactical fiber with 2 chained undo operations
    let fib = cordis
        .mount_fiber(
            Some("fib_test_lifo".to_string()),
            HarnessComponentSlot::C5ToolGuidance,
            "Preferir outline antes de read_file",
            "Poupa 75% de tokens",
            "Use outline antes de editar codigo Rust.",
            Some("coding".to_string()),
            vec!["local_llamacpp".to_string()],
            vec![
                EffectUndoOp {
                    op_id: "op_1".to_string(),
                    description_pt: "Step 1".to_string(),
                    state_key: "cfg.mode".to_string(),
                    new_value: "lean_v1".to_string(),
                    previous_value: None,
                },
                EffectUndoOp {
                    op_id: "op_2".to_string(),
                    description_pt: "Step 2".to_string(),
                    state_key: "cfg.mode".to_string(),
                    new_value: "lean_v2".to_string(),
                    previous_value: None,
                },
            ],
            1000,
        )
        .unwrap();

    assert!(fib.active);
    assert!(!fib.paused_by_coeffect);
    assert_eq!(
        cordis.get_runtime_value("cfg.mode").as_deref(),
        Some("lean_v2")
    );
    assert_eq!(cordis.active_cues_for_task_type("coding").len(), 1);
    assert_eq!(cordis.active_cues_for_task_type("research").len(), 0);

    // Reactive coeffect drop pauses fiber gracefully without losing state
    let paused = cordis.update_coeffect("local_llamacpp", false);
    assert_eq!(paused, 1);
    assert!(cordis.active_prefetch_cues().is_empty());

    let resumed = cordis.update_coeffect("local_llamacpp", true);
    assert_eq!(resumed, 1);
    assert_eq!(cordis.active_prefetch_cues().len(), 1);

    // LIFO dispose pops op_2 then op_1, leaving cfg.mode cleared cleanly
    let report = cordis.dispose_fiber("fib_test_lifo").unwrap();
    assert!(report.restored_cleanly);
    assert!(report.elapsed_us < 10_000);
    assert_eq!(report.reverted_operations.len(), 2);
    assert!(report.reverted_operations[0].starts_with("op_2"));
    assert!(report.reverted_operations[1].starts_with("op_1"));
    assert_eq!(cordis.get_runtime_value("cfg.mode"), None);

    // Hemispheric 3-turn drift triggers auto-rollback
    let fib2 = cordis
        .mount_fiber(
            Some("fib_drift".to_string()),
            HarnessComponentSlot::C2Guardrails,
            "Anti-loop guardrail",
            "Reduz repeticoes",
            "Evite repetir chamadas identicas",
            None,
            vec![],
            vec![],
            1001,
        )
        .unwrap();
    assert!(fib2.active);
    assert!(cordis
        .record_hemispheric_signal("fib_drift", false, 1185.0, 0.48)
        .is_none());
    assert!(cordis
        .record_hemispheric_signal("fib_drift", false, 1170.0, 0.45)
        .is_none());
    let auto_rev = cordis.record_hemispheric_signal("fib_drift", false, 1155.0, 0.42);
    assert!(auto_rev.is_some());
    assert!(!cordis
        .list_fibers()
        .into_iter()
        .find(|f| f.fiber_id == "fib_drift")
        .unwrap()
        .active);

    // Reactivate disposed fiber & test query-aware cue filtering
    let reactivated = cordis.reactivate_fiber("fib_test_lifo").unwrap();
    assert!(reactivated.active);
    assert!(cordis.select_query_aware_cues("Qual minha agenda de amanha?").is_empty());
    assert_eq!(cordis.select_query_aware_cues("Corrigir bug no arquivo .rs via cargo test").len(), 1);
}

#[test]
fn test_v094_catnap_preemption_gates_backoff_and_runmark() {
    let monitor = CatnapMonitor::new(CatnapConfigDto::default());
    let (allowed_now, _) = monitor.evaluate_gate(0, 54.0, 5.0, 14);
    assert!(!allowed_now, "Should not catnap before min_idle_secs=180");

    monitor.set_idle_elapsed_secs_for_test(240);
    // 55C is realistic warm idle on gaming laptop heatpipe -> now allowed under 62C default!
    let (allowed_idle, _) = monitor.evaluate_gate(0, 55.0, 6.0, 14);
    assert!(allowed_idle);

    // 65C blocks general/compute slice under 62C default, but allows 0 MB VRAM lightweight SQLite slice!
    let (blocked_hot, _) = monitor.evaluate_gate(0, 65.0, 6.0, 14);
    assert!(!blocked_hot);
    let (allowed_light, _) = monitor.evaluate_gate_for_slice(0, 65.0, 6.0, 14, Some("ladybug_reconcile"));
    assert!(allowed_light);

    // Thermal hysteresis +5C (62 + 5 = 67C) does not abort at 65C once started, aborts above 67C
    assert!(!monitor.check_thermal_abort_in_flight(65.0));
    assert!(monitor.check_thermal_abort_in_flight(68.5));

    // Round-robin cycling across all 5 canonical slice kinds
    let rr1 = monitor.next_round_robin_slice_kind();
    let rr2 = monitor.next_round_robin_slice_kind();
    assert_ne!(rr1, rr2);

    let (blocked_active, _) = monitor.evaluate_gate(1, 54.0, 6.0, 14);
    assert!(!blocked_active);

    monitor.begin_slice("cursor_1");
    assert!(!monitor.check_preempted());
    monitor.notify_activity();
    assert!(monitor.check_preempted());
    let snap = monitor.snapshot_state();
    assert_eq!(snap.state, "preempted");
    assert!(snap.last_preemption_latency_ms < 1.0);

    // Isolated backoff per slice_kind
    monitor.record_slice_failure("shadow_probe_ppc");
    assert!(monitor.is_slice_in_backoff("shadow_probe_ppc"));
    assert!(!monitor.is_slice_in_backoff("ladybug_reconcile"));
    monitor.reset_slice_backoff("shadow_probe_ppc");
    assert!(!monitor.is_slice_in_backoff("shadow_probe_ppc"));

    // Quiet hours check
    let mut cfg = CatnapConfigDto::default();
    assert!(is_within_quiet_hours(&cfg, 23));
    assert!(is_within_quiet_hours(&cfg, 7));
    assert!(!is_within_quiet_hours(&cfg, 14));
    cfg.quiet_hours_enabled = false;
    assert!(!is_within_quiet_hours(&cfg, 23));

    // Deterministic runmark fingerprint invalidation
    let fp1 = compute_runmark_fingerprint("task_a", "hash_1", HarnessComponentSlot::C5ToolGuidance);
    let fp2 = compute_runmark_fingerprint("task_a", "hash_2", HarnessComponentSlot::C5ToolGuidance);
    assert_ne!(fp1, fp2);
}
