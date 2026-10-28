//! FrankenSQLite STRICT WAL connection pool and canonical DDL migration engine.
//!
//! Enforces Windows NT-safe PRAGMAs, strictly typed tables (`STRICT`),
//! transactional FTS5 triggers with rowid synchronization, and Dev Drive ReFS anchoring.

use std::path::Path;
use std::time::Duration;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};
use sqlx::{Connection, Pool, Row, Sqlite, SqliteConnection};
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

-- 8. Sub-Agent Lifecycle Sessions
CREATE TABLE IF NOT EXISTS sub_agent_sessions (
    agent_id TEXT PRIMARY KEY NOT NULL,
    parent_session_id TEXT NOT NULL,
    task_description TEXT NOT NULL,
    assigned_tier TEXT NOT NULL,
    status TEXT NOT NULL CHECK(status IN ('SPAWNED', 'RUNNING', 'COMPLETED', 'FAILED', 'KILLED')),
    created_at INTEGER NOT NULL,
    completed_at INTEGER
) STRICT;

CREATE INDEX IF NOT EXISTS idx_subagent_parent 
    ON sub_agent_sessions (parent_session_id, status);

-- 9. Handoff Payloads (Transição de Contexto entre Agentes)
CREATE TABLE IF NOT EXISTS handoff_payloads (
    id TEXT PRIMARY KEY NOT NULL,
    source_session_id TEXT NOT NULL,
    target_agent_id TEXT NOT NULL,
    context_summary TEXT NOT NULL,
    variables_json TEXT NOT NULL,
    created_at INTEGER NOT NULL
) STRICT;

CREATE INDEX IF NOT EXISTS idx_handoff_target 
    ON handoff_payloads (target_agent_id);

-- 10. Sub-Agent Operational Diaries
CREATE TABLE IF NOT EXISTS sub_agent_diaries (
    id TEXT PRIMARY KEY NOT NULL,
    agent_id TEXT NOT NULL,
    session_id TEXT NOT NULL,
    content TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    FOREIGN KEY (agent_id) REFERENCES sub_agent_sessions(agent_id) ON DELETE CASCADE
) STRICT;

CREATE INDEX IF NOT EXISTS idx_diaries_agent 
    ON sub_agent_diaries (agent_id, created_at ASC);

-- 11. Socratic Thinking Sessions (DAG Metacognitivo)
CREATE TABLE IF NOT EXISTS socratic_sessions (
    session_id TEXT PRIMARY KEY NOT NULL,
    paradigm TEXT NOT NULL,
    metadata_json TEXT NOT NULL DEFAULT '{}',
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
) STRICT;

-- 12. Socratic Thoughts (Nós da Árvore de Raciocínio)
CREATE TABLE IF NOT EXISTS socratic_thoughts (
    thought_id TEXT PRIMARY KEY NOT NULL,
    session_id TEXT NOT NULL,
    thought_number INTEGER NOT NULL,
    total_thoughts INTEGER NOT NULL,
    thought_content TEXT NOT NULL,
    is_revision INTEGER NOT NULL DEFAULT 0,
    revises_thought INTEGER,
    branch_from_thought INTEGER,
    branch_id TEXT NOT NULL DEFAULT 'main',
    duration_ms INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL,
    FOREIGN KEY (session_id) REFERENCES socratic_sessions(session_id) ON DELETE CASCADE
) STRICT;

CREATE INDEX IF NOT EXISTS idx_thoughts_session_order 
    ON socratic_thoughts (session_id, thought_number ASC);
CREATE INDEX IF NOT EXISTS idx_thoughts_branch 
    ON socratic_thoughts (session_id, branch_id);
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

/// Drains and closes a SQLite pool completely, then opens an exclusive short-lived
/// connection to execute `PRAGMA wal_checkpoint(TRUNCATE);` directly on the physical database file,
/// guaranteeing zero SQLITE_BUSY contention and truncating `.db-wal` to 0 bytes.
#[instrument(skip(pool))]
pub async fn drain_pool_and_exclusive_checkpoint(
    pool: &Pool<Sqlite>,
    db_path: &Path,
) -> Result<(), MemoryError> {
    // 1. Definitively close the connection pool to physically release all descriptors
    pool.close().await;

    // 2. Open an exclusive single-shot connection directly to the file if it exists physically
    if db_path.exists() && db_path != Path::new(":memory:") {
        let options = SqliteConnectOptions::new()
            .filename(db_path)
            .journal_mode(SqliteJournalMode::Wal)
            .busy_timeout(Duration::from_millis(5000));

        let mut conn = SqliteConnection::connect_with(&options).await?;

        // 3. Execute PRAGMA wal_checkpoint(TRUNCATE); directly on the exclusive connection
        sqlx::raw_sql("PRAGMA wal_checkpoint(TRUNCATE);")
            .execute(&mut conn)
            .await?;

        // 4. Explicitly close and drop the temporary exclusive connection
        conn.close().await?;
    }

    info!("Pool drained and exclusive WAL checkpoint (TRUNCATE) completed successfully.");
    Ok(())
}

/// Executes an ultra-safe read-only SQL query on the FrankenSQLite STRICT database.
/// Strictly enforces that the statement is a pure SELECT or WITH query and rejects any mutation commands.
pub async fn execute_read_only_query(
    pool: &Pool<Sqlite>,
    query: &str,
) -> Result<Vec<serde_json::Value>, MemoryError> {
    let trimmed = query.trim();
    let lower = trimmed.to_lowercase();

    // 1. Must start with SELECT or WITH
    if !lower.starts_with("select") && !lower.starts_with("with") {
        return Err(MemoryError::ReadOnlyViolation(format!(
            "Query must start with SELECT or WITH. Received: '{trimmed}'"
        )));
    }

    // 2. Reject mutation keywords
    let forbidden_keywords = [
        "insert ", "update ", "delete ", "drop ", "alter ", "create ", "attach ",
        "detach ", "pragma ", "vacuum ", "reindex ", "replace ",
    ];
    for kw in forbidden_keywords {
        if lower.contains(kw) {
            return Err(MemoryError::ReadOnlyViolation(format!(
                "Forbidden mutation keyword '{kw}' detected in query: '{trimmed}'"
            )));
        }
    }

    // 3. Execute query and convert rows dynamically into JSON objects
    use sqlx::Column;
    let rows = sqlx::query(trimmed).fetch_all(pool).await?;
    let mut results = Vec::with_capacity(rows.len());

    for row in rows {
        let mut map = serde_json::Map::new();
        for col in row.columns() {
            let col_name = col.name();
            if let Ok(val) = row.try_get::<String, _>(col_name) {
                map.insert(col_name.to_string(), serde_json::Value::String(val));
            } else if let Ok(val) = row.try_get::<i64, _>(col_name) {
                map.insert(col_name.to_string(), serde_json::json!(val));
            } else if let Ok(val) = row.try_get::<f64, _>(col_name) {
                map.insert(col_name.to_string(), serde_json::json!(val));
            } else if let Ok(val) = row.try_get::<Option<String>, _>(col_name) {
                map.insert(
                    col_name.to_string(),
                    val.map(serde_json::Value::String).unwrap_or(serde_json::Value::Null),
                );
            } else {
                map.insert(col_name.to_string(), serde_json::Value::Null);
            }
        }
        results.push(serde_json::Value::Object(map));
    }

    Ok(results)
}

/// Upserts an epistemic memory record into the FrankenSQLite database.
pub async fn upsert_epistemic_memory(
    pool: &Pool<Sqlite>,
    record: &souls_protocol::EpistemicMemoryRecord,
) -> Result<(), MemoryError> {
    let partition_str = match record.partition {
        souls_protocol::EpistemicPartition::Stable => "STABLE",
        souls_protocol::EpistemicPartition::Evolving => "EVOLVING",
    };

    sqlx::query(
        r#"
        INSERT INTO epistemic_memories
        (id, session_id, category, content, partition, salience, access_count, source_ref, created_at, last_accessed_at, updated_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
        ON CONFLICT(id) DO UPDATE SET
            category = excluded.category,
            content = excluded.content,
            salience = excluded.salience,
            access_count = excluded.access_count,
            source_ref = excluded.source_ref,
            updated_at = excluded.updated_at
        "#,
    )
    .bind(&record.id)
    .bind(&record.session_id)
    .bind(&record.category)
    .bind(&record.content)
    .bind(partition_str)
    .bind(record.salience as f64)
    .bind(record.access_count)
    .bind(&record.source_ref)
    .bind(record.created_at)
    .bind(record.last_accessed_at)
    .bind(record.updated_at)
    .execute(pool)
    .await?;

    Ok(())
}

/// Retrieves an epistemic memory record by ID.
pub async fn get_epistemic_memory(
    pool: &Pool<Sqlite>,
    id: &str,
) -> Result<Option<souls_protocol::EpistemicMemoryRecord>, MemoryError> {
    let row = sqlx::query(
        r#"
        SELECT id, session_id, category, content, partition, salience, access_count, source_ref, created_at, last_accessed_at, updated_at
        FROM epistemic_memories
        WHERE id = ?1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    if let Some(r) = row {
        let part_str: String = r.get(4);
        let partition = match part_str.as_str() {
            "STABLE" => souls_protocol::EpistemicPartition::Stable,
            "EVOLVING" => souls_protocol::EpistemicPartition::Evolving,
            other => return Err(MemoryError::InvalidPartition(other.to_string())),
        };

        Ok(Some(souls_protocol::EpistemicMemoryRecord {
            id: r.get(0),
            session_id: r.get(1),
            category: r.get(2),
            content: r.get(3),
            partition,
            salience: r.get::<f64, _>(5) as f32,
            access_count: r.get(6),
            source_ref: r.get(7),
            created_at: r.get(8),
            last_accessed_at: r.get(9),
            updated_at: r.get(10),
        }))
    } else {
        Ok(None)
    }
}

/// Performs BM25 lexical search over epistemic memories via SQLite FTS5.
pub async fn search_epistemic_fts5(
    pool: &Pool<Sqlite>,
    query: &str,
    limit: usize,
) -> Result<Vec<(String, String, String)>, MemoryError> {
    let sanitized_query = query.replace(['"', '*', ':', '\''], "");
    let trimmed = sanitized_query.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }

    let rows = sqlx::query(
        r#"
        SELECT id, content, category
        FROM epistemic_memories_fts
        WHERE epistemic_memories_fts MATCH ?1
        LIMIT ?2
        "#,
    )
    .bind(trimmed)
    .bind(limit as i64)
    .fetch_all(pool)
    .await?;

    let results = rows
        .into_iter()
        .map(|r| (r.get(0), r.get(1), r.get(2)))
        .collect();

    Ok(results)
}

/// DTO representing a durable session pre-compress checkpoint (Audit Log v2).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct SessionCheckpointRecord {
    pub id: String,
    pub session_id: String,
    pub checkpoint_version: i64,
    pub raw_messages_blob: String,
    pub message_count: i64,
    pub pre_compress_tokens: i64,
    pub hash_sha256: String,
    pub created_at: i64,
}

/// Persists a pre-compress session checkpoint into the STRICT `session_checkpoints` table.
pub async fn insert_session_checkpoint(
    pool: &Pool<Sqlite>,
    record: &SessionCheckpointRecord,
) -> Result<(), MemoryError> {
    sqlx::query(
        r#"
        INSERT INTO session_checkpoints
        (id, session_id, checkpoint_version, raw_messages_blob, message_count, pre_compress_tokens, hash_sha256, created_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
        ON CONFLICT(id) DO UPDATE SET
            raw_messages_blob = excluded.raw_messages_blob,
            message_count = excluded.message_count,
            pre_compress_tokens = excluded.pre_compress_tokens,
            hash_sha256 = excluded.hash_sha256,
            created_at = excluded.created_at
        "#,
    )
    .bind(&record.id)
    .bind(&record.session_id)
    .bind(record.checkpoint_version)
    .bind(&record.raw_messages_blob)
    .bind(record.message_count)
    .bind(record.pre_compress_tokens)
    .bind(&record.hash_sha256)
    .bind(record.created_at)
    .execute(pool)
    .await?;

    Ok(())
}

/// Lists recent session checkpoints, optionally filtered by `session_id`.
pub async fn list_session_checkpoints(
    pool: &Pool<Sqlite>,
    session_id: Option<&str>,
    limit: usize,
) -> Result<Vec<SessionCheckpointRecord>, MemoryError> {
    let rows = if let Some(sid) = session_id {
        sqlx::query(
            r#"
            SELECT id, session_id, checkpoint_version, raw_messages_blob, message_count, pre_compress_tokens, hash_sha256, created_at
            FROM session_checkpoints
            WHERE session_id = ?1
            ORDER BY created_at DESC
            LIMIT ?2
            "#,
        )
        .bind(sid)
        .bind(limit as i64)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query(
            r#"
            SELECT id, session_id, checkpoint_version, raw_messages_blob, message_count, pre_compress_tokens, hash_sha256, created_at
            FROM session_checkpoints
            ORDER BY created_at DESC
            LIMIT ?1
            "#,
        )
        .bind(limit as i64)
        .fetch_all(pool)
        .await?
    };

    Ok(rows
        .into_iter()
        .map(|r| SessionCheckpointRecord {
            id: r.get(0),
            session_id: r.get(1),
            checkpoint_version: r.get(2),
            raw_messages_blob: r.get(3),
            message_count: r.get(4),
            pre_compress_tokens: r.get(5),
            hash_sha256: r.get(6),
            created_at: r.get(7),
        })
        .collect())
}

/// Lists epistemic memories ordered by salience and updated_at, optionally filtered by partition.
pub async fn list_epistemic_memories(
    pool: &Pool<Sqlite>,
    partition: Option<souls_protocol::EpistemicPartition>,
    limit: usize,
) -> Result<Vec<souls_protocol::EpistemicMemoryRecord>, MemoryError> {
    let rows = if let Some(part) = partition {
        let part_str = match part {
            souls_protocol::EpistemicPartition::Stable => "STABLE",
            souls_protocol::EpistemicPartition::Evolving => "EVOLVING",
        };
        sqlx::query(
            r#"
            SELECT id, session_id, category, content, partition, salience, access_count, source_ref, created_at, last_accessed_at, updated_at
            FROM epistemic_memories
            WHERE partition = ?1
            ORDER BY salience DESC, updated_at DESC
            LIMIT ?2
            "#,
        )
        .bind(part_str)
        .bind(limit as i64)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query(
            r#"
            SELECT id, session_id, category, content, partition, salience, access_count, source_ref, created_at, last_accessed_at, updated_at
            FROM epistemic_memories
            ORDER BY salience DESC, updated_at DESC
            LIMIT ?1
            "#,
        )
        .bind(limit as i64)
        .fetch_all(pool)
        .await?
    };

    let mut out = Vec::with_capacity(rows.len());
    for r in rows {
        let part_str: String = r.get(4);
        let partition = match part_str.as_str() {
            "STABLE" => souls_protocol::EpistemicPartition::Stable,
            "EVOLVING" => souls_protocol::EpistemicPartition::Evolving,
            other => return Err(MemoryError::InvalidPartition(other.to_string())),
        };
        out.push(souls_protocol::EpistemicMemoryRecord {
            id: r.get(0),
            session_id: r.get(1),
            category: r.get(2),
            content: r.get(3),
            partition,
            salience: r.get::<f64, _>(5) as f32,
            access_count: r.get(6),
            source_ref: r.get(7),
            created_at: r.get(8),
            last_accessed_at: r.get(9),
            updated_at: r.get(10),
        });
    }
    Ok(out)
}

/// Deletes an epistemic memory record by ID, returning true if a row was removed.
pub async fn delete_epistemic_memory(
    pool: &Pool<Sqlite>,
    id: &str,
) -> Result<bool, MemoryError> {
    let res = sqlx::query("DELETE FROM epistemic_memories WHERE id = ?1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(res.rows_affected() > 0)
}

/// Counts epistemic memory records grouped by partition `(stable_count, evolving_count)`.
pub async fn count_epistemic_partitions(
    pool: &Pool<Sqlite>,
) -> Result<(usize, usize), MemoryError> {
    let rows = sqlx::query(
        r#"
        SELECT partition, COUNT(*) as cnt
        FROM epistemic_memories
        GROUP BY partition
        "#,
    )
    .fetch_all(pool)
    .await?;

    let mut stable = 0usize;
    let mut evolving = 0usize;
    for r in rows {
        let part: String = r.get(0);
        let cnt: i64 = r.get(1);
        match part.as_str() {
            "STABLE" => stable = cnt.max(0) as usize,
            "EVOLVING" => evolving = cnt.max(0) as usize,
            _ => {}
        }
    }
    Ok((stable, evolving))
}

impl crate::SoulsMemoryStore {
    /// Gracefully tears down the memory store by draining the connection pool,
    /// then establishing an exclusive single-shot connection to truncate the WAL to zero bytes.
    pub async fn shutdown(&self) -> Result<(), MemoryError> {
        let pool = self.pool();
        drain_pool_and_exclusive_checkpoint(&pool, self.db_path()).await
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
        let test_dir = Path::new("Z:\\souls_engine\\.souls_data\\spool\\test_wal_shutdown");
        let _ = std::fs::remove_dir_all(test_dir);
        std::fs::create_dir_all(test_dir).expect("Failed to create test directory");
        let db_path = test_dir.join("checkpoint_test.db");
        let wal_path = test_dir.join("checkpoint_test.db-wal");

        let store = crate::SoulsMemoryStore::new(&db_path, 4).await.unwrap();

        // 1. Insert multiple records into epistemic_memories to dirty the WAL log
        for i in 0..10 {
            sqlx::query(
                r#"
                INSERT INTO epistemic_memories 
                (id, session_id, category, content, partition, salience, access_count, created_at, last_accessed_at, updated_at)
                VALUES (?1, 's1', 'checkpoint', 'WAL checkpoint test data', 'STABLE', 0.9, 1, 100, 100, 100)
                "#,
            )
            .bind(format!("mem_cp_{i}"))
            .execute(&*store.pool())
            .await
            .unwrap();
        }

        // 2. WAL file must exist on disk and contain active frames (> 0 bytes)
        assert!(wal_path.exists(), "WAL file should exist after writes");
        let wal_size_before = std::fs::metadata(&wal_path).unwrap().len();
        assert!(wal_size_before > 0, "WAL size must be > 0 bytes before shutdown");

        // 3. Shutdown store: pool.close() -> exclusive connection -> PRAGMA wal_checkpoint(TRUNCATE) -> conn.close()
        let res = store.shutdown().await;
        assert!(res.is_ok(), "Shutdown sequence must succeed cleanly");

        // 4. Verify WAL file is deterministically truncated to zero bytes
        if wal_path.exists() {
            let wal_size_after = std::fs::metadata(&wal_path).unwrap().len();
            assert_eq!(
                wal_size_after, 0,
                "WAL file must be deterministically truncated to 0 bytes after shutdown"
            );
        }

        // Cleanup
        let _ = std::fs::remove_file(&db_path);
        let _ = std::fs::remove_file(&wal_path);
        let _ = std::fs::remove_dir_all(test_dir);
    }

    #[tokio::test]
    async fn test_read_only_query_rejection_and_execution() {
        let pool = create_in_memory_sqlite_pool().await.unwrap();

        // 1. Insert memory for query
        let mem = souls_protocol::EpistemicMemoryRecord {
            id: "mem_ro_1".to_string(),
            session_id: "sess_ro".to_string(),
            category: "readonly".to_string(),
            content: "Safe read only test".to_string(),
            partition: souls_protocol::EpistemicPartition::Stable,
            salience: 1.0,
            access_count: 0,
            source_ref: None,
            created_at: 100,
            last_accessed_at: 100,
            updated_at: 100,
        };
        upsert_epistemic_memory(&pool, &mem).await.unwrap();

        // 2. Safe SELECT query should succeed
        let rows = execute_read_only_query(&pool, "SELECT id, content FROM epistemic_memories WHERE id = 'mem_ro_1'")
            .await
            .unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0]["id"], "mem_ro_1");
        assert_eq!(rows[0]["content"], "Safe read only test");

        // 3. Mutation statements must be rejected
        let drop_res = execute_read_only_query(&pool, "DROP TABLE epistemic_memories").await;
        assert!(matches!(drop_res, Err(MemoryError::ReadOnlyViolation(_))));

        let insert_res = execute_read_only_query(&pool, "INSERT INTO epistemic_memories (id) VALUES ('x')").await;
        assert!(matches!(insert_res, Err(MemoryError::ReadOnlyViolation(_))));

        let update_res = execute_read_only_query(&pool, "UPDATE epistemic_memories SET salience = 0").await;
        assert!(matches!(update_res, Err(MemoryError::ReadOnlyViolation(_))));
    }

    #[tokio::test]
    async fn test_new_strict_tables_validation() {
        let pool = create_in_memory_sqlite_pool().await.unwrap();

        // Attempt invalid status in sub_agent_sessions (CHECK constraint violation)
        let bad_sub = sqlx::query(
            "INSERT INTO sub_agent_sessions (agent_id, parent_session_id, task_description, assigned_tier, status, created_at) VALUES ('a1', 'p1', 'task', 'Tier1', 'INVALID_STATUS', 100)"
        )
        .execute(&pool)
        .await;
        assert!(bad_sub.is_err(), "Invalid status must violate CHECK constraint");

        // Valid subagent insertion
        let valid_sub = sqlx::query(
            "INSERT INTO sub_agent_sessions (agent_id, parent_session_id, task_description, assigned_tier, status, created_at) VALUES ('a1', 'p1', 'task', 'Tier1', 'SPAWNED', 100)"
        )
        .execute(&pool)
        .await;
        assert!(valid_sub.is_ok());

        // Attempt non-integer in socratic_thoughts total_thoughts (STRICT violation)
        let bad_thought = sqlx::query(
            "INSERT INTO socratic_thoughts (thought_id, session_id, thought_number, total_thoughts, thought_content, created_at) VALUES ('t1', 's1', 1, 'NOT_INT', 'content', 100)"
        )
        .execute(&pool)
        .await;
        assert!(bad_thought.is_err(), "Non-integer must violate STRICT mode");
    }
}


