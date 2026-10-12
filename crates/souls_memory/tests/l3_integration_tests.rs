//! Comprehensive Integration Tests for Souls Engine L3 Brain (`souls_memory`).
//!
//! Validates:
//! - Sub-agent lifecycle, status transitions, operational diaries, and turn handoff payloads.
//! - Socratic thinking DAG engine, revisions, overthinking breaker, and SQLite roundtrip.
//! - Ultra-safe read-only SQL queries with AST-level mutation blocking.
//! - Epistemic memory storage and BM25 FTS5 lexical synchronization.
//! - Non-blocking background MPSC persistence bridge.

use std::time::Duration;
use souls_memory::{
    EpistemicMemoryRecord, EpistemicPartition, MemoryBridgeOp, SocraticThoughtInput,
    SoulsMemoryStore, SubAgentStatus, ThinkingContext, ThinkingParadigm,
};

#[tokio::test]
async fn test_full_l3_subagent_lifecycle_and_handoff() {
    let store = SoulsMemoryStore::new_in_memory().await.unwrap();
    let sub_mgr = store.sub_agent_manager();

    // 1. Spawn a sub-agent
    let sub = sub_mgr
        .spawn_subagent("master_session_42", "Refactor memory module", "Tier1")
        .await
        .unwrap();
    assert_eq!(sub.status, SubAgentStatus::Spawned);
    assert_eq!(sub.parent_session_id, "master_session_42");

    // 2. Transition to RUNNING
    let running = sub_mgr
        .update_status(&sub.agent_id, SubAgentStatus::Running)
        .await
        .unwrap();
    assert_eq!(running.status, SubAgentStatus::Running);
    assert!(running.completed_at.is_none());

    // 3. Record operational diary
    let diary = sub_mgr
        .record_diary(&sub.agent_id, "master_session_42", "Extracted 3 structs from donor")
        .await
        .unwrap();
    assert_eq!(diary.agent_id, sub.agent_id);

    let diaries = sub_mgr.list_diaries(&sub.agent_id).await.unwrap();
    assert_eq!(diaries.len(), 1);
    assert_eq!(diaries[0].content, "Extracted 3 structs from donor");

    // 4. Create handoff payload targeting this sub-agent
    let handoff = sub_mgr
        .create_handoff(
            "master_session_42",
            &sub.agent_id,
            "Handoff AST outline state to subagent",
            serde_json::json!({ "focus": "crates/souls_memory/src/thinking.rs" }),
        )
        .await
        .unwrap();
    assert_eq!(handoff.target_agent_id, sub.agent_id);

    let fetched_hdf = sub_mgr.get_handoff(&sub.agent_id).await.unwrap();
    assert!(fetched_hdf.is_some());
    assert_eq!(
        fetched_hdf.unwrap().context_summary,
        "Handoff AST outline state to subagent"
    );

    // 5. Complete sub-agent
    let completed = sub_mgr
        .update_status(&sub.agent_id, SubAgentStatus::Completed)
        .await
        .unwrap();
    assert_eq!(completed.status, SubAgentStatus::Completed);
    assert!(completed.completed_at.is_some());

    // 6. List by parent session
    let list = sub_mgr
        .list_subagents_by_parent("master_session_42")
        .await
        .unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].agent_id, sub.agent_id);
}

#[tokio::test]
async fn test_full_l3_socratic_thinking_dag_and_revisions() {
    let store = SoulsMemoryStore::new_in_memory().await.unwrap();

    // 1. Instantiate thinking engine
    let engine_lock = store
        .get_or_create_thinking_engine("sess_socratic_integ")
        .await;
    {
        let mut engine = engine_lock.write().await;

        // Context heuristic evaluation
        let ctx = ThinkingContext {
            file_path: None,
            impact_score: 4.2,
            consecutive_failures: 0,
            hitl_authorized: false,
        };
        let chosen_paradigm = souls_memory::ThinkingEngine::determine_paradigm(&ctx);
        assert_eq!(chosen_paradigm, ThinkingParadigm::CouncilMAD);
        engine.set_paradigm(chosen_paradigm);

        // Step 1: Initial hypothesis
        engine
            .add_thought(SocraticThoughtInput {
                thought: "Hypothesis 1: Memory leaks in MPSC bridge".to_string(),
                thought_number: 1,
                total_thoughts: 3,
                next_thought_needed: true,
                is_revision: None,
                revises_thought: None,
                branch_from_thought: None,
                branch_id: None,
            })
            .unwrap();

        // Step 2: Alternative branch
        engine
            .add_thought(SocraticThoughtInput {
                thought: "Hypothesis 2: Bounded buffer eliminates leaks".to_string(),
                thought_number: 2,
                total_thoughts: 3,
                next_thought_needed: true,
                is_revision: None,
                revises_thought: None,
                branch_from_thought: Some(1),
                branch_id: Some("branch_bounded".to_string()),
            })
            .unwrap();

        // Step 3: Revision of Step 1
        engine
            .add_thought(SocraticThoughtInput {
                thought: "Hypothesis 1 revised: Leak is bounded by Tokio channel".to_string(),
                thought_number: 3,
                total_thoughts: 3,
                next_thought_needed: false,
                is_revision: Some(true),
                revises_thought: Some(1),
                branch_from_thought: None,
                branch_id: None,
            })
            .unwrap();

        // Coherence analysis
        let analysis = engine.analyze_coherence();
        assert_eq!(analysis["total_thoughts"], 3);
        assert_eq!(analysis["revision_count"], 1);

        // Persist to database
        engine.persist_to_db(&store.pool()).await.unwrap();

        // Markdown export
        let md = engine.export_markdown();
        assert!(md.contains("Socratic Session Tree"));
        assert!(md.contains("[REVISION]"));
        assert!(md.contains("[BRANCH FORK]"));
    }

    // Rehydrate from store
    let reloaded_lock = store
        .get_or_create_thinking_engine("sess_socratic_integ")
        .await;
    let reloaded = reloaded_lock.read().await;
    assert_eq!(reloaded.thoughts().len(), 3);
}

#[tokio::test]
async fn test_full_l3_epistemic_memories_and_readonly_query() {
    let store = SoulsMemoryStore::new_in_memory().await.unwrap();

    // 1. Store epistemic memory
    let mem = EpistemicMemoryRecord {
        id: "mem_integ_001".to_string(),
        session_id: "sess_integ".to_string(),
        category: "architecture".to_string(),
        content: "Souls Engine L3 Hippocampus operates under FrankenSQLite STRICT WAL".to_string(),
        partition: EpistemicPartition::Stable,
        salience: 1.0,
        access_count: 5,
        source_ref: Some("docs/storage/SCHEMAS_DDL_MANIFEST.md".to_string()),
        created_at: 1000,
        last_accessed_at: 1000,
        updated_at: 1000,
    };
    store.upsert_epistemic_memory(&mem).await.unwrap();

    // 2. Retrieve epistemic memory
    let retrieved = store.get_epistemic_memory("mem_integ_001").await.unwrap();
    assert!(retrieved.is_some());
    let r = retrieved.unwrap();
    assert_eq!(r.category, "architecture");
    assert_eq!(r.partition, EpistemicPartition::Stable);

    // 3. Search via FTS5 BM25
    let fts_matches = store
        .search_epistemic_fts5("Hippocampus", 5)
        .await
        .unwrap();
    assert_eq!(fts_matches.len(), 1);
    assert_eq!(fts_matches[0].0, "mem_integ_001");

    // 4. Safe read-only query
    let rows = store
        .execute_read_only_query("SELECT id, category, salience FROM epistemic_memories WHERE id = 'mem_integ_001'")
        .await
        .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["id"], "mem_integ_001");

    // 5. Block mutation
    let bad_res = store
        .execute_read_only_query("DELETE FROM epistemic_memories")
        .await;
    assert!(bad_res.is_err());
}

#[tokio::test]
async fn test_full_l3_mpsc_bridge_background_drain() {
    let store = SoulsMemoryStore::new_in_memory().await.unwrap();
    let bridge = store.bridge();

    // Send async operation
    let op = MemoryBridgeOp::SaveCheckpoint {
        id: "ckpt_integ_1".to_string(),
        session_id: "sess_bridge_test".to_string(),
        raw_blob: r#"[{"role":"system","content":"test"}]"#.to_string(),
        message_count: 2,
        pre_compress_tokens: 40,
        hash_sha256: "sha256_mock_abc".to_string(),
        created_at: 2000,
    };
    bridge.send_async(op).await.unwrap();

    // Let background worker drain
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Verify checkpoint via read-only query
    let rows = store
        .execute_read_only_query("SELECT id, message_count FROM session_checkpoints WHERE id = 'ckpt_integ_1'")
        .await
        .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["id"], "ckpt_integ_1");
}
