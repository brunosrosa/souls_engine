//! Sub-agent lifecycle coordinator, execution diaries, and turn handoff engine.
//!
//! Enforces transactional state transitions in FrankenSQLite STRICT mode
//! (`sub_agent_sessions`, `handoff_payloads`, `sub_agent_diaries`) with in-RAM
//! fast-path caching via `DashMap`.

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Row, Sqlite};
use tracing::instrument;

use crate::error::MemoryError;

/// Lifecycle state of a spawned sub-agent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SubAgentStatus {
    Spawned,
    Running,
    Completed,
    Failed,
    Killed,
}

impl SubAgentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Spawned => "SPAWNED",
            Self::Running => "RUNNING",
            Self::Completed => "COMPLETED",
            Self::Failed => "FAILED",
            Self::Killed => "KILLED",
        }
    }

    pub fn from_str_strict(s: &str) -> Result<Self, MemoryError> {
        let normalized = s.trim().to_ascii_uppercase();
        match normalized.as_str() {
            "SPAWNED" => Ok(Self::Spawned),
            "RUNNING" => Ok(Self::Running),
            "COMPLETED" => Ok(Self::Completed),
            "FAILED" => Ok(Self::Failed),
            "KILLED" => Ok(Self::Killed),
            _ => Err(MemoryError::Database(sqlx::Error::Decode(
                format!("Invalid SubAgentStatus: '{s}'").into(),
            ))),
        }
    }
}

/// Persistent record of a delegated sub-agent execution context.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubAgentSessionRecord {
    pub agent_id: String,
    pub parent_session_id: String,
    pub task_description: String,
    pub assigned_tier: String,
    pub status: SubAgentStatus,
    pub created_at: i64,
    pub completed_at: Option<i64>,
}

/// Persistent handoff payload transferring memory variables and context summary.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HandoffPayloadRecord {
    pub id: String,
    pub source_session_id: String,
    pub target_agent_id: String,
    pub context_summary: String,
    pub variables_json: String,
    pub created_at: i64,
}

/// Operational journal note written by a sub-agent during execution.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubAgentDiaryRecord {
    pub id: String,
    pub agent_id: String,
    pub session_id: String,
    pub content: String,
    pub created_at: i64,
}

/// Coordinator governing sub-agent lifecycle, diaries, and handoff transactions.
#[derive(Debug, Clone)]
pub struct SubAgentManager {
    pool: Arc<Pool<Sqlite>>,
    active_cache: Arc<DashMap<String, SubAgentSessionRecord>>,
}

impl SubAgentManager {
    /// Creates a new sub-agent manager bound to the SQLite connection pool.
    pub fn new(pool: Arc<Pool<Sqlite>>) -> Self {
        Self {
            pool,
            active_cache: Arc::new(DashMap::new()),
        }
    }

    fn now_sec() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64
    }

    /// Spawns a new sub-agent session and registers it in SQLite STRICT WAL and in-RAM cache.
    #[instrument(skip(self))]
    pub async fn spawn_subagent(
        &self,
        parent_session_id: &str,
        task_description: &str,
        assigned_tier: &str,
    ) -> Result<SubAgentSessionRecord, MemoryError> {
        let now = Self::now_sec();
        let agent_id = format!("sub_{}_{}", now, self.active_cache.len() + 1);

        let record = SubAgentSessionRecord {
            agent_id: agent_id.clone(),
            parent_session_id: parent_session_id.to_string(),
            task_description: task_description.to_string(),
            assigned_tier: assigned_tier.to_string(),
            status: SubAgentStatus::Spawned,
            created_at: now,
            completed_at: None,
        };

        sqlx::query(
            r#"
            INSERT INTO sub_agent_sessions 
            (agent_id, parent_session_id, task_description, assigned_tier, status, created_at, completed_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
        )
        .bind(&record.agent_id)
        .bind(&record.parent_session_id)
        .bind(&record.task_description)
        .bind(&record.assigned_tier)
        .bind(record.status.as_str())
        .bind(record.created_at)
        .bind(record.completed_at)
        .execute(&*self.pool)
        .await?;

        self.active_cache.insert(agent_id, record.clone());
        Ok(record)
    }

    /// Retrieves a sub-agent session by ID, checking cache first and falling back to SQLite.
    pub async fn get_subagent(&self, agent_id: &str) -> Result<Option<SubAgentSessionRecord>, MemoryError> {
        if let Some(cached) = self.active_cache.get(agent_id) {
            return Ok(Some(cached.clone()));
        }

        let row = sqlx::query(
            r#"
            SELECT agent_id, parent_session_id, task_description, assigned_tier, status, created_at, completed_at
            FROM sub_agent_sessions
            WHERE agent_id = ?1
            "#,
        )
        .bind(agent_id)
        .fetch_optional(&*self.pool)
        .await?;

        if let Some(r) = row {
            let status_str: String = r.get(4);
            let status = SubAgentStatus::from_str_strict(&status_str)?;
            let record = SubAgentSessionRecord {
                agent_id: r.get(0),
                parent_session_id: r.get(1),
                task_description: r.get(2),
                assigned_tier: r.get(3),
                status,
                created_at: r.get(5),
                completed_at: r.get(6),
            };
            self.active_cache.insert(agent_id.to_string(), record.clone());
            Ok(Some(record))
        } else {
            Ok(None)
        }
    }

    /// Updates sub-agent status (e.g. RUNNING, COMPLETED, FAILED) with automatic completion timestamp.
    #[instrument(skip(self))]
    pub async fn update_status(
        &self,
        agent_id: &str,
        status: SubAgentStatus,
    ) -> Result<SubAgentSessionRecord, MemoryError> {
        let now = Self::now_sec();
        let completed_at = match status {
            SubAgentStatus::Completed | SubAgentStatus::Failed | SubAgentStatus::Killed => Some(now),
            _ => None,
        };

        let rows_affected = sqlx::query(
            r#"
            UPDATE sub_agent_sessions 
            SET status = ?1, completed_at = ?2
            WHERE agent_id = ?3
            "#,
        )
        .bind(status.as_str())
        .bind(completed_at)
        .bind(agent_id)
        .execute(&*self.pool)
        .await?
        .rows_affected();

        if rows_affected == 0 {
            return Err(MemoryError::SubAgentNotFound(agent_id.to_string()));
        }

        // Invalidate cached entry so get_subagent fetches fresh record from DB
        self.active_cache.remove(agent_id);

        let record = self
            .get_subagent(agent_id)
            .await?
            .ok_or_else(|| MemoryError::SubAgentNotFound(agent_id.to_string()))?;

        Ok(record)
    }

    /// Lists all sub-agents associated with a parent session.
    pub async fn list_subagents_by_parent(
        &self,
        parent_session_id: &str,
    ) -> Result<Vec<SubAgentSessionRecord>, MemoryError> {
        let rows = sqlx::query(
            r#"
            SELECT agent_id, parent_session_id, task_description, assigned_tier, status, created_at, completed_at
            FROM sub_agent_sessions
            WHERE parent_session_id = ?1
            ORDER BY created_at ASC
            "#,
        )
        .bind(parent_session_id)
        .fetch_all(&*self.pool)
        .await?;

        let mut results = Vec::with_capacity(rows.len());
        for r in rows {
            let status_str: String = r.get(4);
            let status = SubAgentStatus::from_str_strict(&status_str)?;
            results.push(SubAgentSessionRecord {
                agent_id: r.get(0),
                parent_session_id: r.get(1),
                task_description: r.get(2),
                assigned_tier: r.get(3),
                status,
                created_at: r.get(5),
                completed_at: r.get(6),
            });
        }

        Ok(results)
    }

    /// Records an operational diary thought from a running sub-agent.
    #[instrument(skip(self))]
    pub async fn record_diary(
        &self,
        agent_id: &str,
        session_id: &str,
        content: &str,
    ) -> Result<SubAgentDiaryRecord, MemoryError> {
        let now = Self::now_sec();
        let diary_id = format!("diary_{}_{}", now, fastrand_u32());

        let record = SubAgentDiaryRecord {
            id: diary_id.clone(),
            agent_id: agent_id.to_string(),
            session_id: session_id.to_string(),
            content: content.to_string(),
            created_at: now,
        };

        sqlx::query(
            r#"
            INSERT INTO sub_agent_diaries (id, agent_id, session_id, content, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
        )
        .bind(&record.id)
        .bind(&record.agent_id)
        .bind(&record.session_id)
        .bind(&record.content)
        .bind(record.created_at)
        .execute(&*self.pool)
        .await?;

        Ok(record)
    }

    /// Lists all diary entries recorded by a specific sub-agent.
    pub async fn list_diaries(&self, agent_id: &str) -> Result<Vec<SubAgentDiaryRecord>, MemoryError> {
        let rows = sqlx::query(
            r#"
            SELECT id, agent_id, session_id, content, created_at
            FROM sub_agent_diaries
            WHERE agent_id = ?1
            ORDER BY created_at ASC
            "#,
        )
        .bind(agent_id)
        .fetch_all(&*self.pool)
        .await?;

        let results = rows
            .into_iter()
            .map(|r| SubAgentDiaryRecord {
                id: r.get(0),
                agent_id: r.get(1),
                session_id: r.get(2),
                content: r.get(3),
                created_at: r.get(4),
            })
            .collect();

        Ok(results)
    }

    /// Creates and persists an atomic handoff payload targeting a sub-agent.
    #[instrument(skip(self, variables))]
    pub async fn create_handoff(
        &self,
        source_session_id: &str,
        target_agent_id: &str,
        context_summary: &str,
        variables: serde_json::Value,
    ) -> Result<HandoffPayloadRecord, MemoryError> {
        let now = Self::now_sec();
        let handoff_id = format!("hdf_{}_{}", now, fastrand_u32());
        let variables_json = variables.to_string();

        let record = HandoffPayloadRecord {
            id: handoff_id.clone(),
            source_session_id: source_session_id.to_string(),
            target_agent_id: target_agent_id.to_string(),
            context_summary: context_summary.to_string(),
            variables_json,
            created_at: now,
        };

        sqlx::query(
            r#"
            INSERT INTO handoff_payloads 
            (id, source_session_id, target_agent_id, context_summary, variables_json, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
        )
        .bind(&record.id)
        .bind(&record.source_session_id)
        .bind(&record.target_agent_id)
        .bind(&record.context_summary)
        .bind(&record.variables_json)
        .bind(record.created_at)
        .execute(&*self.pool)
        .await?;

        Ok(record)
    }

    /// Retrieves the most recent handoff payload targeted to a given agent ID.
    pub async fn get_handoff(&self, target_agent_id: &str) -> Result<Option<HandoffPayloadRecord>, MemoryError> {
        let row = sqlx::query(
            r#"
            SELECT id, source_session_id, target_agent_id, context_summary, variables_json, created_at
            FROM handoff_payloads
            WHERE target_agent_id = ?1
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .bind(target_agent_id)
        .fetch_optional(&*self.pool)
        .await?;

        Ok(row.map(|r| HandoffPayloadRecord {
            id: r.get(0),
            source_session_id: r.get(1),
            target_agent_id: r.get(2),
            context_summary: r.get(3),
            variables_json: r.get(4),
            created_at: r.get(5),
        }))
    }
}

fn fastrand_u32() -> u32 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    (now & 0xFFFFFFFF) as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sqlite::create_in_memory_sqlite_pool;

    #[tokio::test]
    async fn test_subagent_lifecycle_and_diaries() {
        let pool = Arc::new(create_in_memory_sqlite_pool().await.unwrap());
        let manager = SubAgentManager::new(pool);

        // 1. Spawn subagent
        let session = manager
            .spawn_subagent("sess_parent_1", "Optimize AST parse tree", "Tier1")
            .await
            .unwrap();
        assert_eq!(session.status, SubAgentStatus::Spawned);
        assert_eq!(session.parent_session_id, "sess_parent_1");

        // 2. Transition to RUNNING
        let running = manager
            .update_status(&session.agent_id, SubAgentStatus::Running)
            .await
            .unwrap();
        assert_eq!(running.status, SubAgentStatus::Running);
        assert!(running.completed_at.is_none());

        // 3. Record operational diary
        let diary = manager
            .record_diary(&session.agent_id, "sess_parent_1", "Identified 4 hotspots in AST outline")
            .await
            .unwrap();
        assert_eq!(diary.agent_id, session.agent_id);

        let diaries = manager.list_diaries(&session.agent_id).await.unwrap();
        assert_eq!(diaries.len(), 1);
        assert_eq!(diaries[0].content, "Identified 4 hotspots in AST outline");

        // 4. Create and retrieve handoff payload
        let handoff = manager
            .create_handoff(
                "sess_parent_1",
                &session.agent_id,
                "Context handoff for AST task",
                serde_json::json!({ "focus": "crates/souls_ast" }),
            )
            .await
            .unwrap();
        assert_eq!(handoff.target_agent_id, session.agent_id);

        let retrieved_hdf = manager.get_handoff(&session.agent_id).await.unwrap();
        assert!(retrieved_hdf.is_some());
        assert_eq!(retrieved_hdf.unwrap().context_summary, "Context handoff for AST task");

        // 5. Complete subagent
        let completed = manager
            .update_status(&session.agent_id, SubAgentStatus::Completed)
            .await
            .unwrap();
        assert_eq!(completed.status, SubAgentStatus::Completed);
        assert!(completed.completed_at.is_some());

        // 6. List by parent
        let list = manager.list_subagents_by_parent("sess_parent_1").await.unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].agent_id, session.agent_id);
    }

    #[test]
    fn test_subagent_status_resilient_parsing() {
        assert_eq!(SubAgentStatus::from_str_strict("spawned").unwrap(), SubAgentStatus::Spawned);
        assert_eq!(SubAgentStatus::from_str_strict("  Running  ").unwrap(), SubAgentStatus::Running);
        assert_eq!(SubAgentStatus::from_str_strict("Completed").unwrap(), SubAgentStatus::Completed);
        assert_eq!(SubAgentStatus::from_str_strict("failed").unwrap(), SubAgentStatus::Failed);
        assert_eq!(SubAgentStatus::from_str_strict("KILLED").unwrap(), SubAgentStatus::Killed);
        assert!(SubAgentStatus::from_str_strict("invalid_status").is_err());
    }
}
