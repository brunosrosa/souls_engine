//! Non-blocking asynchronous MPSC bridge (`MemoryMpscBridge`) for memory persistence.
//!
//! Decouples SQLite STRICT write I/O from Tokio request handlers and MCP dispatcher loops,
//! buffering writes and draining them in background worker tasks to eliminate latency spikes
//! and protect SSDs against write amplification.

use std::sync::Arc;
use sqlx::{Pool, Sqlite};
use tokio::sync::mpsc::{self, Sender};
use tracing::{error, info, instrument};

use crate::error::MemoryError;
use crate::session_manager::SubAgentDiaryRecord;
use crate::thinking::SocraticThoughtRecord;
use souls_protocol::EpistemicMemoryRecord;

/// Enqueued background memory operation.
#[derive(Debug, Clone)]
pub enum MemoryBridgeOp {
    /// Sub-agent operational journal entry.
    RecordDiary(SubAgentDiaryRecord),
    /// Socratic thinking step persistence.
    PersistThought(SocraticThoughtRecord),
    /// Pre-compression session checkpoint snapshot.
    SaveCheckpoint {
        id: String,
        session_id: String,
        raw_blob: String,
        message_count: i64,
        pre_compress_tokens: i64,
        hash_sha256: String,
        created_at: i64,
    },
    /// Epistemic memory fact storage with automatic FTS5 sync.
    UpsertEpistemic(EpistemicMemoryRecord),
}

/// Asynchronous memory persistence bridge handle.
#[derive(Debug, Clone)]
pub struct MemoryMpscBridge {
    tx: Sender<MemoryBridgeOp>,
}

impl MemoryMpscBridge {
    /// Spawns a background worker consuming operations from an MPSC buffer (default: 1024).
    pub fn spawn(pool: Arc<Pool<Sqlite>>, buffer_capacity: usize) -> Self {
        let (tx, mut rx) = mpsc::channel::<MemoryBridgeOp>(buffer_capacity);

        tokio::spawn(async move {
            info!("MemoryMpscBridge worker started in background.");
            while let Some(op) = rx.recv().await {
                if let Err(e) = Self::handle_op(&pool, op).await {
                    error!("Error executing MemoryBridgeOp in background worker: {e}");
                }
            }
            info!("MemoryMpscBridge worker shut down.");
        });

        Self { tx }
    }

    /// Enqueues an operation into the bridge without blocking synchronously.
    pub fn try_send(&self, op: MemoryBridgeOp) -> Result<(), MemoryError> {
        self.tx.try_send(op).map_err(|e| {
            MemoryError::Database(sqlx::Error::Io(std::io::Error::new(
                std::io::ErrorKind::WouldBlock,
                format!("MemoryMpscBridge queue full or disconnected: {e}"),
            )))
        })
    }

    /// Asynchronously waits to enqueue an operation if buffer is experiencing transient pressure.
    pub async fn send_async(&self, op: MemoryBridgeOp) -> Result<(), MemoryError> {
        self.tx.send(op).await.map_err(|e| {
            MemoryError::Database(sqlx::Error::Io(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                format!("MemoryMpscBridge sender channel closed: {e}"),
            )))
        })
    }

    #[instrument(skip(pool, op))]
    async fn handle_op(pool: &Pool<Sqlite>, op: MemoryBridgeOp) -> Result<(), sqlx::Error> {
        match op {
            MemoryBridgeOp::RecordDiary(diary) => {
                sqlx::query(
                    r#"
                    INSERT INTO sub_agent_diaries (id, agent_id, session_id, content, created_at)
                    VALUES (?1, ?2, ?3, ?4, ?5)
                    "#,
                )
                .bind(&diary.id)
                .bind(&diary.agent_id)
                .bind(&diary.session_id)
                .bind(&diary.content)
                .bind(diary.created_at)
                .execute(pool)
                .await?;
            }
            MemoryBridgeOp::PersistThought(t) => {
                sqlx::query(
                    r#"
                    INSERT OR IGNORE INTO socratic_thoughts
                    (thought_id, session_id, thought_number, total_thoughts, thought_content,
                     is_revision, revises_thought, branch_from_thought, branch_id, duration_ms, created_at)
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
                    "#,
                )
                .bind(&t.thought_id)
                .bind(&t.session_id)
                .bind(t.thought_number as i64)
                .bind(t.total_thoughts as i64)
                .bind(&t.thought_content)
                .bind(if t.is_revision { 1 } else { 0 })
                .bind(t.revises_thought.map(|v| v as i64))
                .bind(t.branch_from_thought.map(|v| v as i64))
                .bind(&t.branch_id)
                .bind(t.duration_ms)
                .bind(t.created_at)
                .execute(pool)
                .await?;
            }
            MemoryBridgeOp::SaveCheckpoint {
                id,
                session_id,
                raw_blob,
                message_count,
                pre_compress_tokens,
                hash_sha256,
                created_at,
            } => {
                sqlx::query(
                    r#"
                    INSERT INTO session_checkpoints
                    (id, session_id, checkpoint_version, raw_messages_blob, message_count, pre_compress_tokens, hash_sha256, created_at)
                    VALUES (?1, ?2, 2, ?3, ?4, ?5, ?6, ?7)
                    "#,
                )
                .bind(&id)
                .bind(&session_id)
                .bind(&raw_blob)
                .bind(message_count)
                .bind(pre_compress_tokens)
                .bind(&hash_sha256)
                .bind(created_at)
                .execute(pool)
                .await?;
            }
            MemoryBridgeOp::UpsertEpistemic(mem) => {
                let partition_str = match mem.partition {
                    souls_protocol::EpistemicPartition::Stable => "STABLE",
                    souls_protocol::EpistemicPartition::Evolving => "EVOLVING",
                };

                sqlx::query(
                    r#"
                    INSERT INTO epistemic_memories
                    (id, session_id, category, content, partition, salience, access_count, source_ref, created_at, last_accessed_at, updated_at)
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
                    ON CONFLICT(id) DO UPDATE SET
                        content = excluded.content,
                        category = excluded.category,
                        salience = excluded.salience,
                        access_count = excluded.access_count,
                        updated_at = excluded.updated_at
                    "#,
                )
                .bind(&mem.id)
                .bind(&mem.session_id)
                .bind(&mem.category)
                .bind(&mem.content)
                .bind(partition_str)
                .bind(mem.salience as f64)
                .bind(mem.access_count)
                .bind(&mem.source_ref)
                .bind(mem.created_at)
                .bind(mem.last_accessed_at)
                .bind(mem.updated_at)
                .execute(pool)
                .await?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sqlite::create_in_memory_sqlite_pool;
    use sqlx::Row;
    use std::time::Duration;

    #[tokio::test]
    async fn test_mpsc_bridge_background_drain() {
        let pool = Arc::new(create_in_memory_sqlite_pool().await.unwrap());
        let bridge = MemoryMpscBridge::spawn(pool.clone(), 64);

        // 1. Send Checkpoint
        let op_ckpt = MemoryBridgeOp::SaveCheckpoint {
            id: "ckpt_1".to_string(),
            session_id: "sess_mpsc".to_string(),
            raw_blob: r#"[{"role":"user","content":"hello"}]"#.to_string(),
            message_count: 1,
            pre_compress_tokens: 15,
            hash_sha256: "fake_hash_123".to_string(),
            created_at: 1000,
        };
        bridge.try_send(op_ckpt).unwrap();

        // 2. Send Epistemic Memory
        let mem = EpistemicMemoryRecord {
            id: "mem_bridge_1".to_string(),
            session_id: "sess_mpsc".to_string(),
            category: "architecture".to_string(),
            content: "Dev Drive ReFS eliminates NVMe contention".to_string(),
            partition: souls_protocol::EpistemicPartition::Stable,
            salience: 1.0,
            access_count: 0,
            source_ref: None,
            created_at: 1000,
            last_accessed_at: 1000,
            updated_at: 1000,
        };
        bridge.send_async(MemoryBridgeOp::UpsertEpistemic(mem)).await.unwrap();

        // Small yield to let background worker drain
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Verify Checkpoint persisted
        let row_ckpt = sqlx::query("SELECT id, message_count FROM session_checkpoints WHERE id = 'ckpt_1'")
            .fetch_one(&*pool)
            .await
            .unwrap();
        assert_eq!(row_ckpt.get::<String, _>(0), "ckpt_1");
        assert_eq!(row_ckpt.get::<i64, _>(1), 1);

        // Verify Epistemic Memory persisted and FTS5 triggered
        let row_mem = sqlx::query("SELECT id, content FROM epistemic_memories WHERE id = 'mem_bridge_1'")
            .fetch_one(&*pool)
            .await
            .unwrap();
        assert_eq!(row_mem.get::<String, _>(0), "mem_bridge_1");

        let row_fts = sqlx::query("SELECT id FROM epistemic_memories_fts WHERE epistemic_memories_fts MATCH 'ReFS'")
            .fetch_one(&*pool)
            .await
            .unwrap();
        assert_eq!(row_fts.get::<String, _>(0), "mem_bridge_1");
    }
}
