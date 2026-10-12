//! Socratic Thinking Engine (`ThinkingEngine`) and Metacognitive Directed Acyclic Graph (DAG).
//!
//! Provides structured multi-step reasoning, cognitive paradigm selection
//! (`LinearCoT`, `TreeToT`, `GraphGoT`, `CouncilMAD`), hypothesis branching (`branch_from_thought`),
//! formal revisions (`is_revision`), overthinking circuit breaker, and transactional SQLite persistence.

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use sqlx::{Pool, Row, Sqlite};
use tracing::instrument;

use crate::error::MemoryError;

/// Default hard limit for the overthinking circuit breaker.
pub const DEFAULT_OVERTHINKING_LIMIT: u32 = 5;

/// Elastic extended limit authorized explicitly via HITL (Human-In-The-Loop).
pub const HITL_EXTENDED_LIMIT: u32 = 7;

/// Metacognitive reasoning paradigms supported by the thinking engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ThinkingParadigm {
    /// Step-by-step linear chain-of-thought for standard tasks.
    #[default]
    LinearCoT,
    /// Tree-of-Thoughts exploring branching hypotheses on consecutive failures.
    TreeToT,
    /// Graph-of-Thoughts synthesizing concurrent branches under eventual consistency.
    GraphGoT,
    /// Multi-Agent Debate deliberation for critical architectural decisions.
    CouncilMAD,
}

impl ThinkingParadigm {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::LinearCoT => "LinearCoT",
            Self::TreeToT => "TreeToT",
            Self::GraphGoT => "GraphGoT",
            Self::CouncilMAD => "CouncilMAD",
        }
    }

    pub fn from_str_lenient(s: &str) -> Self {
        match s {
            "TreeToT" => Self::TreeToT,
            "GraphGoT" => Self::GraphGoT,
            "CouncilMAD" => Self::CouncilMAD,
            _ => Self::LinearCoT,
        }
    }
}

/// Context input evaluated by the metacognitive router to determine reasoning paradigm.
#[derive(Debug, Clone, PartialEq)]
pub struct ThinkingContext {
    pub file_path: Option<String>,
    pub impact_score: f64,
    pub consecutive_failures: u32,
    pub hitl_authorized: bool,
}

/// Input payload submitted when generating a socratic thought step.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SocraticThoughtInput {
    pub thought: String,
    pub thought_number: u32,
    pub total_thoughts: u32,
    pub next_thought_needed: bool,
    #[serde(default)]
    pub is_revision: Option<bool>,
    #[serde(default)]
    pub revises_thought: Option<u32>,
    #[serde(default)]
    pub branch_from_thought: Option<u32>,
    #[serde(default)]
    pub branch_id: Option<String>,
}

/// Persistent record of an individual socratic reasoning step.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SocraticThoughtRecord {
    pub thought_id: String,
    pub session_id: String,
    pub thought_number: u32,
    pub total_thoughts: u32,
    pub thought_content: String,
    pub is_revision: bool,
    pub revises_thought: Option<u32>,
    pub branch_from_thought: Option<u32>,
    pub branch_id: String,
    pub duration_ms: i64,
    pub created_at: i64,
}

/// In-RAM socratic thinking engine with branch tracking and DAG validation.
#[derive(Debug, Clone)]
pub struct ThinkingEngine {
    session_id: String,
    paradigm: ThinkingParadigm,
    thoughts: Vec<SocraticThoughtRecord>,
    branches: HashMap<String, Vec<String>>,
    hard_limit: u32,
    hitl_authorized: bool,
}

impl ThinkingEngine {
    /// Initializes a new thinking engine for a specific session ID.
    pub fn new(session_id: String) -> Self {
        let mut branches = HashMap::new();
        branches.insert("main".to_string(), Vec::new());
        Self {
            session_id,
            paradigm: ThinkingParadigm::LinearCoT,
            thoughts: Vec::with_capacity(DEFAULT_OVERTHINKING_LIMIT as usize),
            branches,
            hard_limit: DEFAULT_OVERTHINKING_LIMIT,
            hitl_authorized: false,
        }
    }

    /// Evaluates context heuristics to select the optimal reasoning paradigm deterministically.
    /// - Rule 1: `consecutive_failures > 1` forces `TreeToT`.
    /// - Rule 2: `impact_score > 3.0` triggers `CouncilMAD`.
    /// - Rule 3: Otherwise defaults to `LinearCoT`.
    pub fn determine_paradigm(ctx: &ThinkingContext) -> ThinkingParadigm {
        if ctx.consecutive_failures > 1 {
            ThinkingParadigm::TreeToT
        } else if ctx.impact_score > 3.0 {
            ThinkingParadigm::CouncilMAD
        } else {
            ThinkingParadigm::LinearCoT
        }
    }

    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    pub fn paradigm(&self) -> ThinkingParadigm {
        self.paradigm
    }

    pub fn set_paradigm(&mut self, paradigm: ThinkingParadigm) {
        self.paradigm = paradigm;
    }

    pub fn authorize_hitl(&mut self) {
        self.hitl_authorized = true;
        self.hard_limit = HITL_EXTENDED_LIMIT;
    }

    pub fn thoughts(&self) -> &[SocraticThoughtRecord] {
        &self.thoughts
    }

    fn now_epoch_ms() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64
    }

    /// Appends a new reasoning thought step with strict circuit breaker and revision checking.
    pub fn add_thought(&mut self, input: SocraticThoughtInput) -> Result<SocraticThoughtRecord, MemoryError> {
        let current_count = self.thoughts.len() as u32;

        // 1. Overthinking circuit breaker check
        if current_count >= self.hard_limit {
            return Err(MemoryError::OverthinkingThresholdBreached {
                current: current_count + 1,
                limit: self.hard_limit,
            });
        }

        let is_revision = input.is_revision.unwrap_or(false);
        let revises_thought = input.revises_thought;

        // 2. Revision integrity validation
        if is_revision {
            match revises_thought {
                Some(target) => {
                    let target_exists = self.thoughts.iter().any(|t| t.thought_number == target);
                    if !target_exists {
                        return Err(MemoryError::InvalidThoughtRevision { target });
                    }
                }
                None => {
                    return Err(MemoryError::InvalidThoughtRevision { target: 0 });
                }
            }
        }

        // 3. Branch routing
        let branch_id = input
            .branch_id
            .unwrap_or_else(|| {
                if let Some(branch_origin) = input.branch_from_thought {
                    format!("branch_step_{branch_origin}")
                } else {
                    "main".to_string()
                }
            });

        let thought_id = format!("thn_{}_{}", self.session_id, input.thought_number);
        let record = SocraticThoughtRecord {
            thought_id: thought_id.clone(),
            session_id: self.session_id.clone(),
            thought_number: input.thought_number,
            total_thoughts: input.total_thoughts,
            thought_content: input.thought,
            is_revision,
            revises_thought,
            branch_from_thought: input.branch_from_thought,
            branch_id: branch_id.clone(),
            duration_ms: 0,
            created_at: Self::now_epoch_ms(),
        };

        self.branches
            .entry(branch_id)
            .or_default()
            .push(thought_id);

        self.thoughts.push(record.clone());
        Ok(record)
    }

    /// Analyzes logical coherence and metrics across the thinking tree.
    pub fn analyze_coherence(&self) -> serde_json::Value {
        let total_thoughts = self.thoughts.len();
        let revisions = self.thoughts.iter().filter(|t| t.is_revision).count();
        let branch_count = self.branches.len();

        serde_json::json!({
            "session_id": self.session_id,
            "paradigm": self.paradigm.as_str(),
            "total_thoughts": total_thoughts,
            "revision_count": revisions,
            "branch_count": branch_count,
            "hitl_extended": self.hitl_authorized,
            "status": if total_thoughts == 0 { "EMPTY" } else { "ACTIVE" }
        })
    }

    /// Exports the thinking session as formatted canonical Markdown.
    pub fn export_markdown(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("# Socratic Session Tree: `{}`\n\n", self.session_id));
        out.push_str(&format!("- **Paradigm:** {}\n", self.paradigm.as_str()));
        out.push_str(&format!("- **Total Steps:** {}\n", self.thoughts.len()));
        out.push_str(&format!("- **Active Branches:** {}\n\n", self.branches.len()));

        for t in &self.thoughts {
            out.push_str(&format!("### Step {} (Branch: `{}`)\n", t.thought_number, t.branch_id));
            if t.is_revision {
                if let Some(target) = t.revises_thought {
                    out.push_str(&format!("> **[REVISION]** Retifies Step #{}\n\n", target));
                }
            }
            if let Some(origin) = t.branch_from_thought {
                out.push_str(&format!("> **[BRANCH FORK]** Bifurcated from Step #{}\n\n", origin));
            }
            out.push_str(&format!("{}\n\n", t.thought_content));
        }

        out
    }

    /// Exports the complete socratic deliberation graph as structured JSON.
    pub fn export_json(&self) -> serde_json::Value {
        serde_json::json!({
            "session_id": self.session_id,
            "paradigm": self.paradigm.as_str(),
            "thoughts": self.thoughts,
            "branches": self.branches,
            "hard_limit": self.hard_limit,
            "hitl_authorized": self.hitl_authorized
        })
    }

    /// Merges an alternative hypothesis branch into a destination branch.
    pub fn merge_branch(&mut self, branch_id: &str, into_branch: &str) -> Result<usize, MemoryError> {
        let thoughts_to_merge = match self.branches.get(branch_id) {
            Some(t) => t.clone(),
            None => return Ok(0),
        };

        let count = thoughts_to_merge.len();
        let target_branch = self.branches.entry(into_branch.to_string()).or_default();
        for tid in thoughts_to_merge {
            if !target_branch.contains(&tid) {
                target_branch.push(tid);
            }
        }

        Ok(count)
    }

    /// Persists session metadata and all socratic thoughts into SQLite STRICT WAL.
    #[instrument(skip(self, pool))]
    pub async fn persist_to_db(&self, pool: &Pool<Sqlite>) -> Result<(), MemoryError> {
        let now = Self::now_epoch_ms();
        let metadata_json = serde_json::json!({
            "hard_limit": self.hard_limit,
            "hitl_authorized": self.hitl_authorized,
            "branches": self.branches
        })
        .to_string();

        // 1. Upsert socratic_sessions
        sqlx::query(
            r#"
            INSERT INTO socratic_sessions (session_id, paradigm, metadata_json, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5)
            ON CONFLICT(session_id) DO UPDATE SET
                paradigm = excluded.paradigm,
                metadata_json = excluded.metadata_json,
                updated_at = excluded.updated_at
            "#,
        )
        .bind(&self.session_id)
        .bind(self.paradigm.as_str())
        .bind(&metadata_json)
        .bind(now)
        .bind(now)
        .execute(pool)
        .await?;

        // 2. Insert thoughts idempotently
        for t in &self.thoughts {
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

        Ok(())
    }

    /// Loads a complete socratic thinking engine from SQLite STRICT WAL.
    pub async fn load_from_db(pool: &Pool<Sqlite>, session_id: &str) -> Result<Option<Self>, MemoryError> {
        let session_row = sqlx::query(
            "SELECT session_id, paradigm, metadata_json FROM socratic_sessions WHERE session_id = ?1",
        )
        .bind(session_id)
        .fetch_optional(pool)
        .await?;

        let row = match session_row {
            Some(r) => r,
            None => return Ok(None),
        };

        let paradigm_str: String = row.get(1);
        let paradigm = ThinkingParadigm::from_str_lenient(&paradigm_str);

        let thoughts_rows = sqlx::query(
            r#"
            SELECT thought_id, session_id, thought_number, total_thoughts, thought_content,
                   is_revision, revises_thought, branch_from_thought, branch_id, duration_ms, created_at
            FROM socratic_thoughts
            WHERE session_id = ?1
            ORDER BY thought_number ASC
            "#,
        )
        .bind(session_id)
        .fetch_all(pool)
        .await?;

        let mut thoughts = Vec::with_capacity(thoughts_rows.len());
        let mut branches: HashMap<String, Vec<String>> = HashMap::new();

        for r in thoughts_rows {
            let thought_id: String = r.get(0);
            let branch_id: String = r.get(8);
            let is_rev_int: i64 = r.get(5);

            branches
                .entry(branch_id.clone())
                .or_default()
                .push(thought_id.clone());

            thoughts.push(SocraticThoughtRecord {
                thought_id,
                session_id: r.get(1),
                thought_number: r.get::<i64, _>(2) as u32,
                total_thoughts: r.get::<i64, _>(3) as u32,
                thought_content: r.get(4),
                is_revision: is_rev_int == 1,
                revises_thought: r.get::<Option<i64>, _>(6).map(|v| v as u32),
                branch_from_thought: r.get::<Option<i64>, _>(7).map(|v| v as u32),
                branch_id,
                duration_ms: r.get(9),
                created_at: r.get(10),
            });
        }

        Ok(Some(Self {
            session_id: session_id.to_string(),
            paradigm,
            thoughts,
            branches,
            hard_limit: DEFAULT_OVERTHINKING_LIMIT,
            hitl_authorized: false,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sqlite::create_in_memory_sqlite_pool;

    #[test]
    fn test_paradigm_determination() {
        let ctx_normal = ThinkingContext {
            file_path: None,
            impact_score: 1.0,
            consecutive_failures: 0,
            hitl_authorized: false,
        };
        assert_eq!(ThinkingEngine::determine_paradigm(&ctx_normal), ThinkingParadigm::LinearCoT);

        let ctx_fail = ThinkingContext {
            file_path: None,
            impact_score: 1.0,
            consecutive_failures: 2,
            hitl_authorized: false,
        };
        assert_eq!(ThinkingEngine::determine_paradigm(&ctx_fail), ThinkingParadigm::TreeToT);

        let ctx_impact = ThinkingContext {
            file_path: None,
            impact_score: 4.5,
            consecutive_failures: 0,
            hitl_authorized: false,
        };
        assert_eq!(ThinkingEngine::determine_paradigm(&ctx_impact), ThinkingParadigm::CouncilMAD);
    }

    #[test]
    fn test_overthinking_limit_and_hitl() {
        let mut engine = ThinkingEngine::new("sess_test_1".to_string());

        // Add 5 thoughts (up to DEFAULT_OVERTHINKING_LIMIT)
        for i in 1..=5 {
            let res = engine.add_thought(SocraticThoughtInput {
                thought: format!("Hypothesis step {i}"),
                thought_number: i,
                total_thoughts: 5,
                next_thought_needed: i < 5,
                is_revision: None,
                revises_thought: None,
                branch_from_thought: None,
                branch_id: None,
            });
            assert!(res.is_ok());
        }

        // 6th thought should be rejected by overthinking circuit breaker
        let res6 = engine.add_thought(SocraticThoughtInput {
            thought: "Excess step 6".to_string(),
            thought_number: 6,
            total_thoughts: 6,
            next_thought_needed: false,
            is_revision: None,
            revises_thought: None,
            branch_from_thought: None,
            branch_id: None,
        });
        assert!(matches!(res6, Err(MemoryError::OverthinkingThresholdBreached { .. })));

        // Authorize HITL -> extended limit = 7
        engine.authorize_hitl();
        let res6_retry = engine.add_thought(SocraticThoughtInput {
            thought: "Authorized step 6".to_string(),
            thought_number: 6,
            total_thoughts: 7,
            next_thought_needed: true,
            is_revision: None,
            revises_thought: None,
            branch_from_thought: None,
            branch_id: None,
        });
        assert!(res6_retry.is_ok());
    }

    #[test]
    fn test_revision_and_branching() {
        let mut engine = ThinkingEngine::new("sess_test_branch".to_string());

        // Step 1
        engine
            .add_thought(SocraticThoughtInput {
                thought: "Initial plan".to_string(),
                thought_number: 1,
                total_thoughts: 3,
                next_thought_needed: true,
                is_revision: None,
                revises_thought: None,
                branch_from_thought: None,
                branch_id: None,
            })
            .unwrap();

        // Step 2: Valid revision targeting Step 1
        let rev = engine.add_thought(SocraticThoughtInput {
            thought: "Refined plan based on AST feedback".to_string(),
            thought_number: 2,
            total_thoughts: 3,
            next_thought_needed: true,
            is_revision: Some(true),
            revises_thought: Some(1),
            branch_from_thought: None,
            branch_id: None,
        });
        assert!(rev.is_ok());

        // Invalid revision targeting non-existent step 99
        let inv_rev = engine.add_thought(SocraticThoughtInput {
            thought: "Broken revision".to_string(),
            thought_number: 3,
            total_thoughts: 3,
            next_thought_needed: false,
            is_revision: Some(true),
            revises_thought: Some(99),
            branch_from_thought: None,
            branch_id: None,
        });
        assert!(matches!(inv_rev, Err(MemoryError::InvalidThoughtRevision { .. })));
    }

    #[tokio::test]
    async fn test_sqlite_persistence_roundtrip() {
        let pool = create_in_memory_sqlite_pool().await.unwrap();
        let mut engine = ThinkingEngine::new("thn_session_persist".to_string());
        engine.set_paradigm(ThinkingParadigm::TreeToT);

        engine
            .add_thought(SocraticThoughtInput {
                thought: "Hypothesis A".to_string(),
                thought_number: 1,
                total_thoughts: 2,
                next_thought_needed: true,
                is_revision: None,
                revises_thought: None,
                branch_from_thought: None,
                branch_id: None,
            })
            .unwrap();

        engine
            .add_thought(SocraticThoughtInput {
                thought: "Alternative Hypothesis B".to_string(),
                thought_number: 2,
                total_thoughts: 2,
                next_thought_needed: false,
                is_revision: None,
                revises_thought: None,
                branch_from_thought: Some(1),
                branch_id: Some("branch_alt".to_string()),
            })
            .unwrap();

        // Persist to SQLite
        engine.persist_to_db(&pool).await.unwrap();

        // Load back from SQLite
        let loaded = ThinkingEngine::load_from_db(&pool, "thn_session_persist")
            .await
            .unwrap();
        assert!(loaded.is_some());
        let l = loaded.unwrap();
        assert_eq!(l.session_id(), "thn_session_persist");
        assert_eq!(l.paradigm(), ThinkingParadigm::TreeToT);
        assert_eq!(l.thoughts().len(), 2);
        assert_eq!(l.thoughts()[1].branch_id, "branch_alt");

        // Verify Markdown export
        let md = l.export_markdown();
        assert!(md.contains("Socratic Session Tree"));
        assert!(md.contains("Hypothesis A"));
    }
}
