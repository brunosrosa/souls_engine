use souls_memory::{
    calculate_bradley_terry_elo, list_catnap_slices, list_memory_tombstones, list_weevolve_ratings,
    record_catnap_slice, record_weevolve_signal, restore_tombstoned_memory, update_ema,
    DomainExperienceLibrary, EpistemicMemoryRecord, EpistemicPartition, SoulsMemoryStore,
};
use souls_protocol::{CatnapSliceRecordDto, DomainExperienceEntryDto};

#[tokio::test]
async fn test_v094_jit_tombstone_preserves_stable_creates_ladybug_edge_and_restores() {
    let store = SoulsMemoryStore::new_in_memory().await.unwrap();

    // 1. Store STABLE architectural rule
    let old_mem = EpistemicMemoryRecord {
        id: "mem_lib_old".to_string(),
        session_id: "sess_1".to_string(),
        category: "architecture".to_string(),
        content: "Usar biblioteca X para serialização rápida".to_string(),
        partition: EpistemicPartition::Stable,
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

    // 2. Store contradicting decision -> triggers JIT Tombstone in 0ms
    let new_mem = EpistemicMemoryRecord {
        id: "mem_lib_new".to_string(),
        session_id: "sess_1".to_string(),
        category: "architecture".to_string(),
        content: "Substituímos X por Y, nunca use X novamente".to_string(),
        partition: EpistemicPartition::Stable,
        salience: 1.0,
        access_count: 1,
        source_ref: Some("decision".to_string()),
        created_at: 2000,
        last_accessed_at: 2000,
        updated_at: 2000,
    };
    let tombs = store.upsert_epistemic_memory_jit(&new_mem).await.unwrap();
    assert_eq!(tombs.len(), 1);
    assert_eq!(tombs[0].superseded_memory_id, "mem_lib_old");
    assert_eq!(tombs[0].superseded_by_memory_id, "mem_lib_new");
    assert_eq!(tombs[0].status, "superseded");

    // 3. Old STABLE memory MUST NOT be deleted physically (ADR-006 compliance!)
    let physical_old = store.get_epistemic_memory("mem_lib_old").await.unwrap();
    assert!(
        physical_old.is_some(),
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
    assert_eq!(all_tombs.len(), 1);

    // 7. Restoring tombstone must also clear the conflicts_with edge in SQLite ladybug_edges
    let edge_count_after_restore: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM ladybug_edges WHERE relationship = 'conflicts_with' AND target_id = 'mem_node_mem_lib_old'",
    )
    .fetch_one(&*pool)
    .await
    .unwrap();
    assert_eq!(edge_count_after_restore, 0);

    // 8. Precision guard: generic single-word overlap ("sistema") + "agora usamos" MUST NOT falsely tombstone!
    let stable_rust = EpistemicMemoryRecord {
        id: "mem_rust_sys".to_string(),
        session_id: "sess_2".to_string(),
        category: "architecture".to_string(),
        content: "Usamos Rust no backend do sistema".to_string(),
        partition: EpistemicPartition::Stable,
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
        partition: EpistemicPartition::Stable,
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

    // >= 2 shared specific subject tokens ("validador", "json") + replacement marker MUST tombstone
    let json_old = EpistemicMemoryRecord {
        id: "mem_json_old".to_string(),
        session_id: "sess_2".to_string(),
        category: "architecture".to_string(),
        content: "Usamos validador json legado nos contratos".to_string(),
        partition: EpistemicPartition::Stable,
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
        partition: EpistemicPartition::Stable,
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
