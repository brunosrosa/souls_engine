//! SOULS ENGINE — Memory Subsystem (Hipocampo L3 Soberano).
//!
//! Provides FrankenSQLite STRICT WAL persistence, CPU-accelerated RRF hybrid fusion,
//! Langevin Decay thermal metabolism with STABLE immunity, LadybugDB ontological causal graph,
//! delegated sub-agent lifecycle management, turn handoff payloads, and socratic thinking DAG.

pub mod chyros;
pub mod error;
pub mod ladybug;
pub mod mpsc_bridge;
pub mod rrf;
pub mod session_manager;
pub mod sqlite;
pub mod thinking;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use arc_swap::ArcSwap;
use dashmap::DashMap;
use sqlx::{Pool, Sqlite};
use tokio::sync::RwLock;
use tracing::info;

pub use chyros::{ChyrosDaemon, ChyrosMetabolismReport};
pub use error::MemoryError;
pub use ladybug::{LadybugOntologyGraph, OntologicalEdge, OntologicalNode};
pub use mpsc_bridge::{MemoryBridgeOp, MemoryMpscBridge};
pub use rrf::{LexicalCandidate, RrfFusionEngine, UnifiedMatch, VectorCandidate};
pub use session_manager::{
    HandoffPayloadRecord, SubAgentDiaryRecord, SubAgentManager, SubAgentSessionRecord,
    SubAgentStatus,
};
pub use sqlite::{
    create_in_memory_sqlite_pool, create_sqlite_pool, execute_read_only_query,
    get_epistemic_memory, search_epistemic_fts5, upsert_epistemic_memory,
};
pub use thinking::{
    SocraticThoughtInput, SocraticThoughtRecord, ThinkingContext, ThinkingEngine,
    ThinkingParadigm,
};
pub use souls_protocol::{EpistemicMemoryRecord, EpistemicPartition};

/// Central governance coordinator for all memory, persistence, and graph subsystems.
#[derive(Debug, Clone)]
pub struct SoulsMemoryStore {
    db_path: PathBuf,
    pool_swap: Arc<ArcSwap<Pool<Sqlite>>>,
    ladybug: LadybugOntologyGraph,
    rrf: RrfFusionEngine,
    chyros: Arc<ChyrosDaemon>,
    sub_agent_manager: Arc<SubAgentManager>,
    thinking_sessions: Arc<DashMap<String, Arc<RwLock<ThinkingEngine>>>>,
    bridge: Arc<MemoryMpscBridge>,
}

impl SoulsMemoryStore {
    /// Bootstraps the memory store against a canonical persistent Dev Drive ReFS path.
    pub async fn new<P: AsRef<Path>>(
        db_path: P,
        max_connections: u32,
    ) -> Result<Self, MemoryError> {
        let path = db_path.as_ref().to_path_buf();
        let pool = create_sqlite_pool(&path, max_connections).await?;
        let pool_arc = Arc::new(pool);

        let ladybug = LadybugOntologyGraph::new();
        ladybug.load_from_db(&pool_arc).await?;

        let rrf = RrfFusionEngine::default();
        let chyros = Arc::new(ChyrosDaemon::new(&path)?);
        let sub_agent_manager = Arc::new(SubAgentManager::new(pool_arc.clone()));
        let thinking_sessions = Arc::new(DashMap::new());
        let bridge = Arc::new(MemoryMpscBridge::spawn(pool_arc.clone(), 1024));

        Ok(Self {
            db_path: path,
            pool_swap: Arc::new(ArcSwap::from_pointee((*pool_arc).clone())),
            ladybug,
            rrf,
            chyros,
            sub_agent_manager,
            thinking_sessions,
            bridge,
        })
    }

    /// Bootstraps an in-memory memory store for hermetic unit and integration testing.
    pub async fn new_in_memory() -> Result<Self, MemoryError> {
        let pool = create_in_memory_sqlite_pool().await?;
        let pool_arc = Arc::new(pool);

        let ladybug = LadybugOntologyGraph::new();
        let rrf = RrfFusionEngine::default();
        let chyros = Arc::new(ChyrosDaemon::new_in_memory());
        let sub_agent_manager = Arc::new(SubAgentManager::new(pool_arc.clone()));
        let thinking_sessions = Arc::new(DashMap::new());
        let bridge = Arc::new(MemoryMpscBridge::spawn(pool_arc.clone(), 256));

        Ok(Self {
            db_path: PathBuf::from(":memory:"),
            pool_swap: Arc::new(ArcSwap::from_pointee((*pool_arc).clone())),
            ladybug,
            rrf,
            chyros,
            sub_agent_manager,
            thinking_sessions,
            bridge,
        })
    }

    /// Returns the database path.
    pub fn db_path(&self) -> &Path {
        &self.db_path
    }

    /// Retrieves an atomic reference to the active FrankenSQLite connection pool.
    pub fn pool(&self) -> Arc<Pool<Sqlite>> {
        self.pool_swap.load_full()
    }

    /// Accesses the in-memory LadybugDB causal graph.
    pub fn ladybug(&self) -> &LadybugOntologyGraph {
        &self.ladybug
    }

    /// Accesses the RRF fusion engine.
    pub fn rrf(&self) -> &RrfFusionEngine {
        &self.rrf
    }

    /// Accesses the Chyros metabolism daemon.
    pub fn chyros(&self) -> &ChyrosDaemon {
        &self.chyros
    }

    /// Accesses the sub-agent lifecycle coordinator.
    pub fn sub_agent_manager(&self) -> &SubAgentManager {
        &self.sub_agent_manager
    }

    /// Accesses the non-blocking background MPSC persistence bridge.
    pub fn bridge(&self) -> &MemoryMpscBridge {
        &self.bridge
    }

    /// Retrieves or instantiates in RAM a socratic thinking engine for a given session.
    pub async fn get_or_create_thinking_engine(
        &self,
        session_id: &str,
    ) -> Arc<RwLock<ThinkingEngine>> {
        if let Some(existing) = self.thinking_sessions.get(session_id) {
            return existing.clone();
        }

        // Check if session can be rehydrated from FrankenSQLite
        let pool = self.pool();
        let engine = if let Ok(Some(loaded)) = ThinkingEngine::load_from_db(&pool, session_id).await {
            loaded
        } else {
            ThinkingEngine::new(session_id.to_string())
        };

        let engine_arc = Arc::new(RwLock::new(engine));
        self.thinking_sessions
            .insert(session_id.to_string(), engine_arc.clone());
        engine_arc
    }

    /// Executes an ultra-safe read-only SQL query on the active FrankenSQLite pool.
    pub async fn execute_read_only_query(
        &self,
        query: &str,
    ) -> Result<Vec<serde_json::Value>, MemoryError> {
        let pool = self.pool();
        execute_read_only_query(&pool, query).await
    }

    /// Upserts an epistemic memory record into the active database pool.
    pub async fn upsert_epistemic_memory(
        &self,
        record: &souls_protocol::EpistemicMemoryRecord,
    ) -> Result<(), MemoryError> {
        let pool = self.pool();
        upsert_epistemic_memory(&pool, record).await
    }

    /// Retrieves an epistemic memory record from the active database pool.
    pub async fn get_epistemic_memory(
        &self,
        id: &str,
    ) -> Result<Option<souls_protocol::EpistemicMemoryRecord>, MemoryError> {
        let pool = self.pool();
        get_epistemic_memory(&pool, id).await
    }

    /// Performs BM25 lexical search over epistemic memories using SQLite FTS5.
    pub async fn search_epistemic_fts5(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<(String, String, String)>, MemoryError> {
        let pool = self.pool();
        search_epistemic_fts5(&pool, query, limit).await
    }

    /// Runs a full Chyros Langevin metabolism cycle. If atomic file rotation occurs,
    /// seamlessly updates the internal `SqlitePool` via `ArcSwap` with zero reader lockups.
    pub async fn run_chyros_metabolism(
        &self,
        now_epoch_sec: i64,
    ) -> Result<ChyrosMetabolismReport, MemoryError> {
        let current_pool = self.pool();
        let (report, maybe_new_pool) = self
            .chyros
            .run_metabolism_cycle(&current_pool, now_epoch_sec)
            .await?;

        if let Some(new_pool) = maybe_new_pool {
            self.pool_swap.store(Arc::new(new_pool));
            info!("SoulsMemoryStore pool rotated atomically via ArcSwap.");
        }

        Ok(report)
    }
}
