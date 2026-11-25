use souls_memory::{
    apply_memory_tombstone, calculate_bradley_terry_elo, cleanup_residue_ladybug_nodes,
    detect_and_apply_jit_tombstones, get_catnap_utc_day_totals, list_catnap_slices,
    list_memory_tombstones, list_weevolve_ratings, preview_jit_tombstones, record_catnap_slice,
    record_weevolve_signal, restore_tombstoned_memory, update_ema, DomainExperienceLibrary,
    EpistemicMemoryRecord, EpistemicPartition, SoulsMemoryStore,
};
use souls_protocol::{CatnapSliceRecordDto, DomainExperienceEntryDto};

#[tokio::test]
async fn test_v094_jit_tombstone_preserves_stable_creates_ladybug_edge_and_restores() {
    let store = SoulsMemoryStore::new_in_memory().await.unwrap();

    // 1. Store EVOLVING architectural rule for automatic scan + preview check
    let old_mem = EpistemicMemoryRecord {
        id: "mem_lib_old".to_string(),
        session_id: "sess_1".to_string(),
        category: "architecture".to_string(),
        content: "Usar biblioteca X para serialização rápida".to_string(),
        partition: EpistemicPartition::Evolving,
        salience: 1.0,
        access_count: 3,
        source_ref: Some("decision".to_string()),
        created_at: 1000,
        last_accessed_at: 1000,
        updated_at: 1000,
    };
    let t0 = store.upsert_epistemic_memory_jit(&old_mem).await.unwrap();
    assert!(t0.is_empty());

    let before_recall = store.hybrid_recall("biblioteca X", None, 5).await.unwrap();
    assert!(before_recall.iter().any(|m| m.id == "mem_lib_old"));

    // 2. Dry-run preview must NOT write any tombstones, then upsert triggers JIT Tombstone in 0ms
    let new_mem = EpistemicMemoryRecord {
        id: "mem_lib_new".to_string(),
        session_id: "sess_1".to_string(),
        category: "architecture".to_string(),
        content: "Substituímos X por Y, nunca use X novamente".to_string(),
        partition: EpistemicPartition::Evolving,
        salience: 1.0,
        access_count: 1,
        source_ref: Some("decision".to_string()),
        created_at: 2000,
        last_accessed_at: 2000,
        updated_at: 2000,
    };
    let preview = preview_jit_tombstones(&store.pool(), &new_mem, 2000)
        .await
        .unwrap();
    assert_eq!(preview.len(), 1);
    assert_eq!(preview[0].status, "preview");
    assert!(list_memory_tombstones(&store.pool(), 10).await.unwrap().is_empty());

    let tombs = store.upsert_epistemic_memory_jit(&new_mem).await.unwrap();
    assert_eq!(tombs.len(), 1);
    assert_eq!(tombs[0].superseded_memory_id, "mem_lib_old");
    assert_eq!(tombs[0].superseded_by_memory_id, "mem_lib_new");
    assert_eq!(tombs[0].status, "superseded");

    // Second scan on the same pair MUST be a no-op (idempotent)
    let second_scan = detect_and_apply_jit_tombstones(&store.pool(), store.ladybug(), &new_mem, 2001)
        .await
        .unwrap();
    assert!(second_scan.is_empty());

    // Reverse scan with the older memory MUST NOT cause circular mutual suicide
    let reverse_scan = detect_and_apply_jit_tombstones(&store.pool(), store.ladybug(), &old_mem, 2002)
        .await
        .unwrap();
    assert!(reverse_scan.is_empty());

    // 3. Explicit STABLE tombstone must NOT physically delete STABLE rows (ADR-006 compliance!)
    let stable_keep = EpistemicMemoryRecord {
        id: "mem_stable_rule".to_string(),
        session_id: "sess_1".to_string(),
        category: "architecture".to_string(),
        content: "Regra permanente A".to_string(),
        partition: EpistemicPartition::Stable,
        salience: 1.0,
        access_count: 2,
        source_ref: None,
        created_at: 1500,
        last_accessed_at: 1500,
        updated_at: 1500,
    };
    assert!(store.upsert_epistemic_memory_jit(&stable_keep).await.unwrap().is_empty());
    let stable_override = EpistemicMemoryRecord {
        id: "mem_stable_override".to_string(),
        content: "Regra permanente B em vez de A".to_string(),
        updated_at: 2500,
        ..stable_keep.clone()
    };
    assert!(
        store.upsert_epistemic_memory_jit(&stable_override).await.unwrap().is_empty(),
        "Automatic JIT scan must never mutate STABLE records"
    );
    let _explicit_stable_tomb = apply_memory_tombstone(
        &store.pool(),
        store.ladybug(),
        &stable_keep,
        &stable_override,
        "Substituição explícita em STABLE.",
        2600,
    )
    .await
    .unwrap();
    let physical_stable = store.get_epistemic_memory("mem_stable_rule").await.unwrap();
    assert!(
        physical_stable.is_some(),
        "STABLE memory must stay physically in SQLite (ADR-006)"
    );

    // 4. Old memory MUST be hidden immediately from hybrid_recall & FTS5 search
    let after_recall = store.hybrid_recall("biblioteca X", None, 5).await.unwrap();
    assert!(!after_recall.iter().any(|m| m.id == "mem_lib_old"));
    let fts_after = store.search_epistemic_fts5("biblioteca", 5).await.unwrap();
    assert!(!fts_after.iter().any(|(id, _, _)| id == "mem_lib_old"));

    // 5. LadybugDB MUST contain conflicts_with edge without NodeNotFound
    let neighbors = store.ladybug().get_neighbors("mem_node_mem_lib_new").await;
    assert!(neighbors
        .iter()
        .any(|e| e.target_id == "mem_node_mem_lib_old" && e.relationship == "conflicts_with"));

    // 6. 1-click restore brings old memory back to recall
    let pool = store.pool();
    let restored = restore_tombstoned_memory(&pool, &tombs[0].tombstone_id, 3000)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(restored.status, "restored");

    let restored_recall = store.hybrid_recall("biblioteca X", None, 5).await.unwrap();
    assert!(restored_recall.iter().any(|m| m.id == "mem_lib_old"));

    let all_tombs = list_memory_tombstones(&pool, 10).await.unwrap();
    assert_eq!(all_tombs.len(), 2);

    // 7. Restoring tombstone must also clear the conflicts_with edge in SQLite ladybug_edges
    let edge_count_after_restore: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM ladybug_edges WHERE relationship = 'conflicts_with' AND target_id = 'mem_node_mem_lib_old'",
    )
    .fetch_one(&*pool)
    .await
    .unwrap();
    assert_eq!(edge_count_after_restore, 0);

    // 8. URL ('http://') & bio ('2e:') colons must NEVER trigger false key:value tombstones!
    let prose_a = EpistemicMemoryRecord {
        id: "mem_prose_url_a".to_string(),
        session_id: "sess_prose".to_string(),
        category: "hermes_memory".to_string(),
        content: "Souls Engine (v0.9.2+3 / MCP v7.2.0 at http://127.0.0.1:9123/mcp) exposes 85 tools.".to_string(),
        partition: EpistemicPartition::Evolving,
        salience: 1.0,
        access_count: 1,
        source_ref: None,
        created_at: 3500,
        last_accessed_at: 3500,
        updated_at: 3500,
    };
    let prose_b = EpistemicMemoryRecord {
        id: "mem_prose_url_b".to_string(),
        content: "Souls Engine (v0.9.2+3 / MCP v7.2.0 at http://127.0.0.1:9123/mcp) exposes 96 tools.".to_string(),
        updated_at: 3600,
        ..prose_a.clone()
    };
    let bio_a = EpistemicMemoryRecord {
        id: "mem_bio_a".to_string(),
        category: "hermes_user".to_string(),
        content: "Bruno (born 1985-02-08, 2e: ADHD + AH/SD, father of Lucca) works with Rust.".to_string(),
        updated_at: 3500,
        ..prose_a.clone()
    };
    let bio_b = EpistemicMemoryRecord {
        id: "mem_bio_b".to_string(),
        category: "hermes_user".to_string(),
        content: "Bruno (born 1985-02-08, 2e: ADHD + AH/SD, father of Lucca) uses Windows 11.".to_string(),
        updated_at: 3600,
        ..prose_a.clone()
    };
    assert!(store.upsert_epistemic_memory_jit(&prose_a).await.unwrap().is_empty());
    assert!(store.upsert_epistemic_memory_jit(&prose_b).await.unwrap().is_empty(), "http:// must never trigger KV prefix");
    assert!(store.upsert_epistemic_memory_jit(&bio_a).await.unwrap().is_empty());
    assert!(store.upsert_epistemic_memory_jit(&bio_b).await.unwrap().is_empty(), "2e: in prose must never trigger KV prefix");

    // 9. Precision guard: generic single-word overlap ("sistema") + "agora usamos" MUST NOT falsely tombstone!
    let stable_rust = EpistemicMemoryRecord {
        id: "mem_rust_sys".to_string(),
        session_id: "sess_2".to_string(),
        category: "architecture".to_string(),
        content: "Usamos Rust no backend do sistema".to_string(),
        partition: EpistemicPartition::Evolving,
        salience: 1.0,
        access_count: 1,
        source_ref: None,
        created_at: 4000,
        last_accessed_at: 4000,
        updated_at: 4000,
    };
    let stable_tts = EpistemicMemoryRecord {
        id: "mem_tts_sys".to_string(),
        session_id: "sess_2".to_string(),
        category: "architecture".to_string(),
        content: "Agora usamos Edge Neural TTS no sistema".to_string(),
        partition: EpistemicPartition::Evolving,
        salience: 1.0,
        access_count: 1,
        source_ref: None,
        created_at: 4100,
        last_accessed_at: 4100,
        updated_at: 4100,
    };
    assert!(store.upsert_epistemic_memory_jit(&stable_rust).await.unwrap().is_empty());
    assert!(
        store.upsert_epistemic_memory_jit(&stable_tts).await.unwrap().is_empty(),
        "Generic shared word 'sistema' must not trigger false JIT tombstone"
    );

    // >= 2 shared specific subject tokens ("validador", "json") + replacement marker on EVOLVING MUST tombstone
    let json_old = EpistemicMemoryRecord {
        id: "mem_json_old".to_string(),
        session_id: "sess_2".to_string(),
        category: "architecture".to_string(),
        content: "Usamos validador json legado nos contratos".to_string(),
        partition: EpistemicPartition::Evolving,
        salience: 1.0,
        access_count: 1,
        source_ref: None,
        created_at: 4200,
        last_accessed_at: 4200,
        updated_at: 4200,
    };
    let json_new = EpistemicMemoryRecord {
        id: "mem_json_new".to_string(),
        session_id: "sess_2".to_string(),
        category: "architecture".to_string(),
        content: "Agora usamos validador json estrito em vez do legado".to_string(),
        partition: EpistemicPartition::Evolving,
        salience: 1.0,
        access_count: 1,
        source_ref: None,
        created_at: 4300,
        last_accessed_at: 4300,
        updated_at: 4300,
    };
    assert!(store.upsert_epistemic_memory_jit(&json_old).await.unwrap().is_empty());
    let json_tombs = store.upsert_epistemic_memory_jit(&json_new).await.unwrap();
    assert_eq!(json_tombs.len(), 1);
    assert_eq!(json_tombs[0].superseded_memory_id, "mem_json_old");
}

#[tokio::test]
async fn test_v094_weevolve_elo_ema_catnap_ledger_and_domain_library() {
    let store = SoulsMemoryStore::new_in_memory().await.unwrap();
    let pool = store.pool();

    // Pure math sanity check
    let (elo_up, s_pos) = calculate_bradley_terry_elo(1200.0, 1200.0, 32.0, 1.2);
    assert!(elo_up > 1200.0);
    let ema_up = update_ema(0.5, s_pos, 0.15);
    assert!(ema_up > 0.5);

    // Database signal persistence
    let r1 = record_weevolve_signal(&pool, "fib_coding_01", "cordis_fiber", "test_success", None, 1000)
        .await
        .unwrap();
    assert!(r1.elo_rating > 1200.0);
    assert_eq!(r1.total_matches, 1);

    let r2 = record_weevolve_signal(&pool, "fib_coding_01", "cordis_fiber", "git_rollback", None, 1010)
        .await
        .unwrap();
    assert!(r2.elo_rating < r1.elo_rating);
    assert_eq!(r2.total_matches, 2);

    let ratings = list_weevolve_ratings(&pool, 10).await.unwrap();
    assert_eq!(ratings.len(), 1);

    // STRICT mode validation on new tables
    let bad_rating = sqlx::query(
        "INSERT INTO weevolve_ratings (target_id, target_kind, elo_rating, ema_score, total_matches, last_signal, updated_at) VALUES ('bad', 'invalid_kind', 1200.0, 0.5, 0, 'x', 1)",
    )
    .execute(&*pool)
    .await;
    assert!(bad_rating.is_err());

    // Catnap ledger persistence
    let slice = CatnapSliceRecordDto {
        slice_id: "slice_001".to_string(),
        slice_kind: "shadow_probe_ppc".to_string(),
        runmark_fingerprint: "fp_abc123".to_string(),
        cursor_checkpoint: "cur_1".to_string(),
        status: "completed".to_string(),
        duration_ms: 14.2,
        cost_usd: 0.0,
        summary_pt: "Sondagem sem regressão a custo $0.00".to_string(),
        trigger_mode: "auto".to_string(),
        actor: "catnap_scheduler".to_string(),
        authorization_provenance: None,
        created_at: 2000,
    };
    record_catnap_slice(&pool, &slice).await.unwrap();
    let slices = list_catnap_slices(&pool, 10).await.unwrap();
    assert_eq!(slices.len(), 1);
    assert_eq!(slices[0].slice_id, "slice_001");

    // Domain Experience Library (EXPERIENCE.md + O(M) INDEX.md)
    let tmp = tempfile::tempdir().unwrap();
    let lib = DomainExperienceLibrary::new(tmp.path());
    let ep = DomainExperienceEntryDto {
        episode_id: "ep_rust_01".to_string(),
        domain_tag: "rust_ast".to_string(),
        cue_pattern: "cargo test lock".to_string(),
        always_on_chars: 380,
        skills_count: 1,
        subagents_count: 1,
        scripts_count: 0,
        hooks_count: 1,
        runmark_fingerprint: "fp_abc123".to_string(),
        experience_summary_pt: "Usar outline antes de editar reduziu tokens em 78%.".to_string(),
        created_at: 2005,
    };
    let index_path = lib.record_episode(&ep).unwrap();
    assert!(index_path.is_file());
    let index_txt = std::fs::read_to_string(&index_path).unwrap();
    assert!(!index_txt.contains("\r\n"), "INDEX.md must use LF");
    assert!(index_txt.contains("ep_rust_01"));
    assert!(index_txt.contains("380"));
    assert_eq!(lib.list_episodes().unwrap().len(), 1);

    // Deduplicate episodes sharing the same runmark_fingerprint instead of multiplying folders
    let ep_dup = DomainExperienceEntryDto {
        episode_id: "ep_rust_duplicate_should_reuse".to_string(),
        experience_summary_pt: "Atualizado sem duplicar diretório.".to_string(),
        created_at: 2020,
        ..ep
    };
    let _ = lib.record_episode(&ep_dup).unwrap();
    let eps = lib.list_episodes().unwrap();
    assert_eq!(eps.len(), 1, "Same runmark_fingerprint must reuse episode folder");
    assert_eq!(eps[0].episode_id, "ep_rust_01");
}

#[tokio::test]
async fn test_v0943_catnap_utc_totals_force_manual_audit_and_legacy_migration() {
    let store = SoulsMemoryStore::new_in_memory().await.unwrap();
    let pool = store.pool();

    let prov = souls_protocol::AuthorizationProvenanceDto {
        channel: "antigravity_ide".to_string(),
        timestamp: 1795577000,
        scope: "manual_catnap_smoke".to_string(),
        actor: Some("bruno".to_string()),
    };

    let day_start = 1795564800;
    let old_slice = CatnapSliceRecordDto {
        slice_id: "slice_prev_day".to_string(),
        slice_kind: "shadow_probe_ppc".to_string(),
        runmark_fingerprint: "fp_old".to_string(),
        cursor_checkpoint: "cur_old".to_string(),
        status: "completed".to_string(),
        duration_ms: 2.0,
        cost_usd: 0.15,
        summary_pt: "Ontem".to_string(),
        trigger_mode: "auto".to_string(),
        actor: "catnap_scheduler".to_string(),
        authorization_provenance: None,
        created_at: day_start - 10,
    };
    let today_manual = CatnapSliceRecordDto {
        slice_id: "slice_today_manual".to_string(),
        slice_kind: "shadow_probe_ppc".to_string(),
        runmark_fingerprint: "fp_today".to_string(),
        cursor_checkpoint: "cur_today".to_string(),
        status: "completed".to_string(),
        duration_ms: 3.5,
        cost_usd: 0.05,
        summary_pt: "Hoje manual".to_string(),
        trigger_mode: "manual_forced".to_string(),
        actor: "bruno_ide".to_string(),
        authorization_provenance: Some(prov.clone()),
        created_at: day_start + 50,
    };
    let today_skipped = CatnapSliceRecordDto {
        slice_id: "slice_today_skip".to_string(),
        slice_kind: "ralph_crystallize".to_string(),
        runmark_fingerprint: "fp_skip".to_string(),
        cursor_checkpoint: "cur_skip".to_string(),
        status: "skipped".to_string(),
        duration_ms: 1.0,
        cost_usd: 0.0,
        summary_pt: "Ignorada".to_string(),
        trigger_mode: "auto".to_string(),
        actor: "catnap_scheduler".to_string(),
        authorization_provenance: None,
        created_at: day_start + 60,
    };

    record_catnap_slice(&pool, &old_slice).await.unwrap();
    record_catnap_slice(&pool, &today_manual).await.unwrap();
    record_catnap_slice(&pool, &today_skipped).await.unwrap();

    let (count_today, spent_today) = get_catnap_utc_day_totals(&pool, day_start).await.unwrap();
    assert_eq!(count_today, 1);
    assert!((spent_today - 0.05).abs() < 1e-6);

    let listed = list_catnap_slices(&pool, 10).await.unwrap();
    let found = listed
        .iter()
        .find(|s| s.slice_id == "slice_today_manual")
        .unwrap();
    assert_eq!(found.trigger_mode, "manual_forced");
    assert_eq!(found.actor, "bruno_ide");
    assert_eq!(found.authorization_provenance.as_ref(), Some(&prov));

    // Legacy table without the 3 new columns must migrate idempotently
    sqlx::raw_sql(
        r#"
        DROP TABLE catnap_consolidation_ledger;
        CREATE TABLE catnap_consolidation_ledger (
            slice_id TEXT PRIMARY KEY NOT NULL,
            slice_kind TEXT NOT NULL,
            runmark_fingerprint TEXT NOT NULL,
            cursor_checkpoint TEXT NOT NULL,
            status TEXT NOT NULL,
            duration_ms REAL NOT NULL DEFAULT 0.0,
            cost_usd REAL NOT NULL DEFAULT 0.0,
            summary_pt TEXT NOT NULL,
            created_at INTEGER NOT NULL
        ) STRICT;
        "#,
    )
    .execute(&*pool)
    .await
    .unwrap();
    souls_memory::sqlite::ensure_catnap_audit_columns(&pool)
        .await
        .unwrap();
    souls_memory::sqlite::ensure_catnap_audit_columns(&pool)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_v0943_ladybug_residue_cleanup_is_idempotent() {
    let store = SoulsMemoryStore::new_in_memory().await.unwrap();
    let pool = store.pool();
    let lb = store.ladybug();

    let clean_node = souls_memory::OntologicalNode {
        id: "node_souls_engine".to_string(),
        node_type: "entity".to_string(),
        canonical_name: "souls_engine".to_string(),
        metadata_json: "{}".to_string(),
        banned_patterns: vec![],
    };
    let residue_node = souls_memory::OntologicalNode {
        id: "mem_node_hermes:memory:ff233f".to_string(),
        node_type: "epistemic_memory".to_string(),
        canonical_name: "Souls Engine (v0.9.2+3 / MCP v7.2.0 at http://127.0.0.1:9123/...".to_string(),
        metadata_json: "{}".to_string(),
        banned_patterns: vec![],
    };
    lb.insert_node(&pool, clean_node, 1000).await.unwrap();
    lb.insert_node(&pool, residue_node, 1000).await.unwrap();

    let first_removed = cleanup_residue_ladybug_nodes(&pool, lb).await.unwrap();
    assert_eq!(first_removed, vec!["mem_node_hermes:memory:ff233f".to_string()]);

    let second_removed = cleanup_residue_ladybug_nodes(&pool, lb).await.unwrap();
    assert!(second_removed.is_empty());

    let remaining: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM ladybug_nodes")
        .fetch_one(&*pool)
        .await
        .unwrap();
    assert_eq!(remaining, 1);
}
