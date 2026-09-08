//! FrankenSQLite STRICT WAL connection pool and canonical DDL migration engine.
//!
//! Enforces Windows NT-safe PRAGMAs, strictly typed tables (`STRICT`),
//! transactional FTS5 triggers with rowid synchronization, and Dev Drive ReFS anchoring.

use std::path::Path;
use std::time::Duration;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};
use sqlx::{Pool, Sqlite};
use tracing::{info, instrument};

use crate::error::MemoryError;
use souls_core::fs::{ensure_refs_directory, validate_refs_path};

/// Canonical DDL statements defining all persistent relational tables in STRICT mode.
pub const CANONICAL_DDL_MANIFEST: &str = r#"
-- 1. Epistemic Memories (Relational L3 Hippocampus)
CREATE TABLE IF NOT EXISTS epistemic_memories (
    id TEXT PRIMARY KEY NOT NULL,
    session_id TEXT NOT NULL,
    category TEXT NOT NULL,
    content TEXT NOT NULL,
    partition TEXT NOT NULL CHECK(partition IN ('STABLE', 'EVOLVING')),
    salience REAL NOT NULL CHECK(salience >= 0.0 AND salience <= 1.0),
    access_count INTEGER NOT NULL DEFAULT 0,
    source_ref TEXT,
    created_at INTEGER NOT NULL,
    last_accessed_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
) STRICT;

CREATE INDEX IF NOT EXISTS idx_epistemic_partition_salience 
    ON epistemic_memories (partition, salience DESC);

CREATE INDEX IF NOT EXISTS idx_epistemic_session 
    ON epistemic_memories (session_id);

CREATE INDEX IF NOT EXISTS idx_epistemic_category 
    ON epistemic_memories (category);

-- 2. Epistemic Memories FTS5 Virtual Table (Lexical Search)
CREATE VIRTUAL TABLE IF NOT EXISTS epistemic_memories_fts USING fts5 (
    id UNINDEXED,
    content,
    category,
    tokenize = 'unicode61 remove_diacritics 2'
);

-- FTS5 Synchronization Triggers utilizing rowid
CREATE TRIGGER IF NOT EXISTS trg_epistemic_memories_ai 
AFTER INSERT ON epistemic_memories BEGIN
    INSERT INTO epistemic_memories_fts (rowid, id, content, category)
    VALUES (new.rowid, new.id, new.content, new.category);
END;

CREATE TRIGGER IF NOT EXISTS trg_epistemic_memories_ad 
AFTER DELETE ON epistemic_memories BEGIN
    DELETE FROM epistemic_memories_fts WHERE rowid = old.rowid;
END;

CREATE TRIGGER IF NOT EXISTS trg_epistemic_memories_au 
AFTER UPDATE ON epistemic_memories BEGIN
    DELETE FROM epistemic_memories_fts WHERE rowid = old.rowid;
    INSERT INTO epistemic_memories_fts (rowid, id, content, category)
    VALUES (new.rowid, new.id, new.content, new.category);
END;

-- 3. Telemetry Events (MPSC Dehydration Buffer)
CREATE TABLE IF NOT EXISTS telemetry_events (
    id TEXT PRIMARY KEY NOT NULL,
    session_id TEXT NOT NULL,
    event_type TEXT NOT NULL,
    execution_tier TEXT NOT NULL,
    payload_json TEXT NOT NULL,
    latency_ms REAL NOT NULL,
    tokens_input INTEGER NOT NULL DEFAULT 0,
    tokens_output INTEGER NOT NULL DEFAULT 0,
    direct_cost_usd REAL NOT NULL DEFAULT 0.0,
    e3_score REAL NOT NULL DEFAULT 0.0,
    created_at INTEGER NOT NULL
) STRICT;

CREATE INDEX IF NOT EXISTS idx_telemetry_created_tier 
    ON telemetry_events (created_at DESC, execution_tier);

CREATE INDEX IF NOT EXISTS idx_telemetry_session 
    ON telemetry_events (session_id);

-- 4. Pareto Bandit Priors (Bayesian FinOps Optimization)
CREATE TABLE IF NOT EXISTS pareto_bandit_priors (
    tier_name TEXT NOT NULL,
    task_category TEXT NOT NULL,
    alpha REAL NOT NULL CHECK(alpha >= 1.0),
    beta REAL NOT NULL CHECK(beta >= 1.0),
    pull_count INTEGER NOT NULL DEFAULT 0,
    cumulative_reward REAL NOT NULL DEFAULT 0.0,
    last_updated_at INTEGER NOT NULL,
    PRIMARY KEY (tier_name, task_category)
) STRICT;

-- 5. Organ Vault (Anthropophagy Purified Code Entities)
CREATE TABLE IF NOT EXISTS organ_vault (
    id TEXT PRIMARY KEY NOT NULL,
    symbol_name TEXT NOT NULL,
    language TEXT NOT NULL,
    spdx_license TEXT NOT NULL,
    source_repository TEXT NOT NULL,
    ast_kind TEXT NOT NULL,
    dependencies_json TEXT NOT NULL,
    signature_decl TEXT NOT NULL,
    pure_implementation TEXT NOT NULL,
    cyclomatic_complexity INTEGER NOT NULL,
    loc_count INTEGER NOT NULL,
    created_at INTEGER NOT NULL
) STRICT;

CREATE INDEX IF NOT EXISTS idx_organ_symbol 
    ON organ_vault (symbol_name, language);

CREATE INDEX IF NOT EXISTS idx_organ_license 
    ON organ_vault (spdx_license);

-- 6. LadybugDB Ontological Nodes & Edges (RAM Graph Persistence)
CREATE TABLE IF NOT EXISTS ladybug_nodes (
    id TEXT PRIMARY KEY NOT NULL,
    node_type TEXT NOT NULL,
    canonical_name TEXT NOT NULL,
    metadata_json TEXT NOT NULL,
    created_at INTEGER NOT NULL
) STRICT;

CREATE INDEX IF NOT EXISTS idx_ladybug_node_type 
    ON ladybug_nodes (node_type, canonical_name);

CREATE TABLE IF NOT EXISTS ladybug_edges (
    source_id TEXT NOT NULL,
    target_id TEXT NOT NULL,
    relationship TEXT NOT NULL CHECK(relationship IN ('depends_on', 'conflicts_with', 'implements', 'relates_to')),
    weight REAL NOT NULL DEFAULT 1.0,
    created_at INTEGER NOT NULL,
    PRIMARY KEY (source_id, target_id, relationship),
    FOREIGN KEY (source_id) REFERENCES ladybug_nodes(id) ON DELETE CASCADE,
    FOREIGN KEY (target_id) REFERENCES ladybug_nodes(id) ON DELETE CASCADE
) STRICT;

-- 7. Session Checkpoints (Pre-Compress Audit Log v2)
CREATE TABLE IF NOT EXISTS session_checkpoints (
    id TEXT PRIMARY KEY NOT NULL,
    session_id TEXT NOT NULL,
    checkpoint_version INTEGER NOT NULL DEFAULT 2,
    raw_messages_blob TEXT NOT NULL,
    message_count INTEGER NOT NULL,
    pre_compress_tokens INTEGER NOT NULL,
    hash_sha256 TEXT NOT NULL,
    created_at INTEGER NOT NULL
) STRICT;

CREATE INDEX IF NOT EXISTS idx_checkpoint_session 
    ON session_checkpoints (session_id, created_at DESC);
"#;

/// Configures and bootstraps an NT-safe FrankenSQLite connection pool with required PRAGMAs.
#[instrument(skip(db_path), fields(path = %db_path.as_ref().display()))]
pub async fn create_sqlite_pool<P: AsRef<Path>>(
    db_path: P,
    max_connections: u32,
) -> Result<Pool<Sqlite>, MemoryError> {
    let path = db_path.as_ref();

    // Ensure Dev Drive ReFS validation and directory hierarchy using canonical souls_core primitives
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            ensure_refs_directory(parent)?;
        }
    } else {
        validate_refs_path(path)?;
    }

    let options = SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal)
        .busy_timeout(Duration::from_millis(5000))
        .foreign_keys(true)
        .pragma("temp_store", "MEMORY")
        .pragma("cache_size", "-64000")
        .pragma("auto_vacuum", "NONE");

    let pool = SqlitePoolOptions::new()
        .max_connections(max_connections)
        .connect_with(options)
        .await?;

    info!("FrankenSQLite pool connected successfully in STRICT WAL mode.");
    init_schema(&pool).await?;

    Ok(pool)
}

static IN_MEM_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

/// Creates an in-memory FrankenSQLite pool configured identically for testing with shared cache.
pub async fn create_in_memory_sqlite_pool() -> Result<Pool<Sqlite>, MemoryError> {
    let id = IN_MEM_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let mem_uri = format!("file:souls_mem_{id}?mode=memory&cache=shared");

    let options = SqliteConnectOptions::new()
        .filename(&mem_uri)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal)
        .busy_timeout(Duration::from_millis(5000))
        .foreign_keys(true)
        .pragma("temp_store", "MEMORY")
        .pragma("cache_size", "-64000");

    let pool = SqlitePoolOptions::new()
        .max_connections(8)
        .connect_with(options)
        .await?;

    init_schema(&pool).await?;
    Ok(pool)
}

/// Initializes the canonical schema on the database pool.
pub async fn init_schema(pool: &Pool<Sqlite>) -> Result<(), MemoryError> {
    sqlx::raw_sql(CANONICAL_DDL_MANIFEST).execute(pool).await?;
    Ok(())
}

/// Executes an atomic raw `PRAGMA wal_checkpoint(TRUNCATE);` on the SQLite pool,
/// forcing complete physical flush of WAL frame buffers into the main database file
/// on the ReFS Dev Drive and truncating the .db-wal file.
#[instrument(skip(pool))]
pub async fn wal_checkpoint_truncate(pool: &Pool<Sqlite>) -> Result<(), MemoryError> {
    sqlx::raw_sql("PRAGMA wal_checkpoint(TRUNCATE);")
        .execute(pool)
        .await?;
    info!("Executed PRAGMA wal_checkpoint(TRUNCATE) successfully.");
    Ok(())
}

impl crate::SoulsMemoryStore {
    /// Gracefully tears down the memory store by executing an atomic WAL checkpoint (TRUNCATE)
    /// to flush all uncommitted/residual WAL buffers to disk before closing connection pools.
    pub async fn shutdown(&self) -> Result<(), MemoryError> {
        let pool = self.pool();
        wal_checkpoint_truncate(&pool).await?;
        pool.close().await;
        info!("SoulsMemoryStore successfully shut down with WAL flushed and truncated.");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::Row;

    #[tokio::test]
    async fn test_concurrent_writes_zero_busy() {
        let pool = create_in_memory_sqlite_pool()
            .await
            .expect("Failed to create test pool");

        let mut tasks = Vec::new();

        // Launch 20 concurrent tasks performing simultaneous writes and reads
        for i in 0..20 {
            let pool_clone = pool.clone();
            tasks.push(tokio::spawn(async move {
                let id = format!("mem_conc_{i}");
                let session_id = format!("sess_{}", i % 3);
                let content = format!("Concurrent memory payload {i}");
                let now = 1700000000 + i;

                // Insert into epistemic_memories
                sqlx::query(
                    r#"
                    INSERT INTO epistemic_memories 
                    (id, session_id, category, content, partition, salience, access_count, created_at, last_accessed_at, updated_at)
                    VALUES (?1, ?2, 'test_conc', ?3, 'EVOLVING', 0.8, 0, ?4, ?4, ?4)
                    "#,
                )
                .bind(&id)
                .bind(&session_id)
                .bind(&content)
                .bind(now)
                .execute(&pool_clone)
                .await
                .map_err(|e| format!("Task {i} insert error: {e}"))?;

                // Read back immediately
                let row = sqlx::query("SELECT content, salience FROM epistemic_memories WHERE id = ?1")
                    .bind(&id)
                    .fetch_one(&pool_clone)
                    .await
                    .map_err(|e| format!("Task {i} fetch error: {e}"))?;

                let fetched_content: String = row.get(0);
                assert_eq!(fetched_content, content);

                Ok::<(), String>(())
            }));
        }

        for task in tasks {
            let res = task.await.expect("Task join failed");
            assert!(res.is_ok(), "Concurrent write returned error: {:?}", res.err());
        }

        // Verify total count in table
        let count_row = sqlx::query("SELECT COUNT(*) FROM epistemic_memories")
            .fetch_one(&pool)
            .await
            .unwrap();
        let total: i64 = count_row.get(0);
        assert_eq!(total, 20);
    }

    #[tokio::test]
    async fn test_strict_mode_rejection() {
        let pool = create_in_memory_sqlite_pool().await.unwrap();

        // Attempt to insert string into salience (REAL in STRICT table)
        let result = sqlx::query(
            r#"
            INSERT INTO epistemic_memories 
            (id, session_id, category, content, partition, salience, access_count, created_at, last_accessed_at, updated_at)
            VALUES ('bad_1', 'sess_1', 'cat', 'content', 'EVOLVING', 'NOT_A_FLOAT', 0, 100, 100, 100)
            "#,
        )
        .execute(&pool)
        .await;

        assert!(result.is_err(), "STRICT mode should have rejected non-float salience");
    }

    #[tokio::test]
    async fn test_fts5_triggers_and_rowid_deletion() {
        let pool = create_in_memory_sqlite_pool().await.unwrap();

        // 1. Insert memory
        sqlx::query(
            r#"
            INSERT INTO epistemic_memories 
            (id, session_id, category, content, partition, salience, access_count, created_at, last_accessed_at, updated_at)
            VALUES ('mem_fts_1', 's1', 'arch', 'FrankenSQLite bare-metal optimization', 'STABLE', 1.0, 1, 100, 100, 100)
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        // 2. Query FTS5 table
        let fts_row = sqlx::query(
            "SELECT id, content FROM epistemic_memories_fts WHERE epistemic_memories_fts MATCH 'FrankenSQLite'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        let id: String = fts_row.get(0);
        assert_eq!(id, "mem_fts_1");

        // 3. Delete memory (trigger trg_epistemic_memories_ad with rowid)
        sqlx::query("DELETE FROM epistemic_memories WHERE id = 'mem_fts_1'")
            .execute(&pool)
            .await
            .unwrap();

        // 4. Verify FTS5 is purged automatically
        let fts_after = sqlx::query(
            "SELECT id FROM epistemic_memories_fts WHERE epistemic_memories_fts MATCH 'FrankenSQLite'",
        )
        .fetch_optional(&pool)
        .await
        .unwrap();

        assert!(fts_after.is_none(), "FTS5 record must be purged on memory deletion");
    }

    #[tokio::test]
    async fn test_wal_checkpoint_truncate_and_shutdown() {
        let store = crate::SoulsMemoryStore::new_in_memory().await.unwrap();
        // Insert some data into the store's pool
        sqlx::query(
            r#"
            INSERT INTO epistemic_memories 
            (id, session_id, category, content, partition, salience, access_count, created_at, last_accessed_at, updated_at)
            VALUES ('mem_cp_1', 's1', 'checkpoint', 'WAL checkpoint test data', 'STABLE', 0.9, 1, 100, 100, 100)
            "#,
        )
        .execute(&*store.pool())
        .await
        .unwrap();

        // Checkpoint explicitly
        let cp_res = wal_checkpoint_truncate(&store.pool()).await;
        assert!(cp_res.is_ok());

        // Shutdown store which triggers wal_checkpoint_truncate and pool.close()
        let res = store.shutdown().await;
        assert!(res.is_ok());
    }
}

