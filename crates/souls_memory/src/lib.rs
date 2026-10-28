//! SOULS ENGINE — Memory Subsystem (Hipocampo L3 Soberano).
//!
//! Provides FrankenSQLite STRICT WAL persistence, CPU-accelerated RRF hybrid fusion,
//! LanceDB columnar vector storage (0 MB VRAM via Arrow Memory Model),
//! Langevin Decay thermal metabolism with STABLE immunity, LadybugDB ontological causal graph,
//! delegated sub-agent lifecycle management, turn handoff payloads, and socratic thinking DAG.

pub mod chyros;
pub mod error;
pub mod ladybug;
pub mod lancedb;
pub mod mpsc_bridge;
pub mod rrf;
pub mod session_manager;
pub mod snapshot;
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
pub use lancedb::{get_vector_table_schema, LanceDbVectorStore, VECTOR_DIMENSION};
pub use mpsc_bridge::{MemoryBridgeOp, MemoryMpscBridge};
pub use rrf::{LexicalCandidate, RrfFusionEngine, UnifiedMatch, VectorCandidate};
pub use session_manager::{
    HandoffPayloadRecord, SubAgentDiaryRecord, SubAgentManager, SubAgentSessionRecord,
    SubAgentStatus,
};
pub use snapshot::{
    deduplicate_and_budget_matches, normalize_for_dedup, parse_markdown_entries,
    resolve_default_hermes_home, sha256_hex, FrozenMemorySnapshot, HermesBootstrapFiles,
    SnapshotCoordinator, DEFAULT_PREFETCH_MAX_CHARS, DEFAULT_PREFETCH_TOP_K,
};
pub use sqlite::{
    count_epistemic_partitions, create_in_memory_sqlite_pool, create_sqlite_pool,
    delete_epistemic_memory, execute_read_only_query, get_epistemic_memory,
    insert_session_checkpoint, list_epistemic_memories, list_session_checkpoints,
    search_epistemic_fts5, upsert_epistemic_memory, SessionCheckpointRecord,
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
    lance_store: LanceDbVectorStore,
    chyros: Arc<ChyrosDaemon>,
    sub_agent_manager: Arc<SubAgentManager>,
    thinking_sessions: Arc<DashMap<String, Arc<RwLock<ThinkingEngine>>>>,
    bridge: Arc<MemoryMpscBridge>,
    snapshots: Arc<SnapshotCoordinator>,
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

        // Resolve LanceDB vectors directory adjacent to sqlite vault
        let lance_dir = path
            .parent()
            .map(|p| p.join("souls_vectors.lance"))
            .unwrap_or_else(|| PathBuf::from(r"Z:\souls_engine\.souls_data\souls_vectors.lance"));
        let lance_store = LanceDbVectorStore::new(lance_dir);

        let chyros = Arc::new(ChyrosDaemon::new(&path)?);
        let sub_agent_manager = Arc::new(SubAgentManager::new(pool_arc.clone()));
        let thinking_sessions = Arc::new(DashMap::new());
        let bridge = Arc::new(MemoryMpscBridge::spawn(pool_arc.clone(), 1024));
        let snapshots = Arc::new(SnapshotCoordinator::new());

        Ok(Self {
            db_path: path,
            pool_swap: Arc::new(ArcSwap::from_pointee((*pool_arc).clone())),
            ladybug,
            rrf,
            lance_store,
            chyros,
            sub_agent_manager,
            thinking_sessions,
            bridge,
            snapshots,
        })
    }

    /// Bootstraps an in-memory memory store for hermetic unit and integration testing.
    pub async fn new_in_memory() -> Result<Self, MemoryError> {
        let pool = create_in_memory_sqlite_pool().await?;
        let pool_arc = Arc::new(pool);

        let ladybug = LadybugOntologyGraph::new();
        let rrf = RrfFusionEngine::default();
        let temp_lance = std::env::temp_dir().join(format!("souls_lance_test_{}", std::process::id()));
        let lance_store = LanceDbVectorStore::new(temp_lance);

        let chyros = Arc::new(ChyrosDaemon::new_in_memory());
        let sub_agent_manager = Arc::new(SubAgentManager::new(pool_arc.clone()));
        let thinking_sessions = Arc::new(DashMap::new());
        let bridge = Arc::new(MemoryMpscBridge::spawn(pool_arc.clone(), 256));
        let snapshots = Arc::new(SnapshotCoordinator::new());

        Ok(Self {
            db_path: PathBuf::from(":memory:"),
            pool_swap: Arc::new(ArcSwap::from_pointee((*pool_arc).clone())),
            ladybug,
            rrf,
            lance_store,
            chyros,
            sub_agent_manager,
            thinking_sessions,
            bridge,
            snapshots,
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

    /// Accesses the LanceDB columnar vector store.
    pub fn lance_store(&self) -> &LanceDbVectorStore {
        &self.lance_store
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

    /// Accesses the `SnapshotCoordinator` managing global and session-pinned frozen snapshots.
    pub fn snapshots(&self) -> &SnapshotCoordinator {
        &self.snapshots
    }

    /// Pins or refreshes the `FrozenMemorySnapshot` for a given `session_id`.
    pub async fn pin_session_snapshot(
        &self,
        session_id: &str,
        force_refresh: bool,
        hermes_home: Option<&Path>,
        now_epoch_sec: i64,
    ) -> Result<Arc<FrozenMemorySnapshot>, MemoryError> {
        let pool = self.pool();
        self.snapshots
            .pin_session_snapshot(session_id, force_refresh, &pool, hermes_home, now_epoch_sec)
            .await
    }

    /// Mirrors a built-in Hermes memory write (`on_memory_write`) into the `STABLE` partition.
    pub async fn mirror_hermes_write(
        &self,
        action: &str,
        target: &str,
        content: &str,
        session_id: &str,
        now_epoch_sec: i64,
    ) -> Result<Option<String>, MemoryError> {
        let pool = self.pool();
        self.snapshots
            .mirror_hermes_write(&pool, action, target, content, session_id, now_epoch_sec)
            .await
    }

    /// Executes Hybrid RRF recall and deduplicates candidates against the session's `FrozenMemorySnapshot`,
    /// enforcing `top_k` (default 4) and `max_chars` (default 800 characters).
    pub async fn recall_for_session(
        &self,
        session_id: &str,
        query: &str,
        query_vector: Option<Vec<f32>>,
        top_k: usize,
        max_chars: usize,
    ) -> Result<Vec<UnifiedMatch>, MemoryError> {
        let fetch_limit = top_k.max(4) * 3;
        let raw_matches = self.hybrid_recall(query, query_vector, fetch_limit).await?;
        let snap = self
            .snapshots
            .get_session_snapshot(session_id)
            .unwrap_or_else(|| self.snapshots.current_global());
        Ok(deduplicate_and_budget_matches(&snap, raw_matches, top_k, max_chars))
    }

    /// Persists a pre-compress session checkpoint (`on_pre_compress` Audit Log v2).
    pub async fn save_pre_compress_checkpoint(
        &self,
        record: &SessionCheckpointRecord,
    ) -> Result<(), MemoryError> {
        let pool = self.pool();
        insert_session_checkpoint(&pool, record).await
    }

    /// Lists recent pre-compress session checkpoints.
    pub async fn list_checkpoints(
        &self,
        session_id: Option<&str>,
        limit: usize,
    ) -> Result<Vec<SessionCheckpointRecord>, MemoryError> {
        let pool = self.pool();
        list_session_checkpoints(&pool, session_id, limit).await
    }

    /// Lists epistemic memories ordered by salience, optionally filtered by partition.
    pub async fn list_epistemic(
        &self,
        partition: Option<souls_protocol::EpistemicPartition>,
        limit: usize,
    ) -> Result<Vec<souls_protocol::EpistemicMemoryRecord>, MemoryError> {
        let pool = self.pool();
        list_epistemic_memories(&pool, partition, limit).await
    }

    /// Returns `(stable_count, evolving_count)` from `epistemic_memories`.
    pub async fn partition_counts(&self) -> Result<(usize, usize), MemoryError> {
        let pool = self.pool();
        count_epistemic_partitions(&pool).await
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

    /// Executes full Hybrid Recall fusing FTS5 lexical BM25 and LanceDB vector cosine similarity via RRF.
    pub async fn hybrid_recall(
        &self,
        query: &str,
        query_vector: Option<Vec<f32>>,
        limit: usize,
    ) -> Result<Vec<UnifiedMatch>, MemoryError> {
        // 1. Lexical retrieval from FTS5
        let fts_raw = self.search_epistemic_fts5(query, limit * 2).await?;
        let lexical_candidates: Vec<LexicalCandidate> = fts_raw
            .into_iter()
            .enumerate()
            .map(|(idx, (id, content, category))| LexicalCandidate {
                id,
                category,
                content,
                score: 1.0 / (idx + 1) as f64,
            })
            .collect();

        // 2. Vector retrieval from LanceDB (if embedding provided)
        let vector_candidates = if let Some(vec) = query_vector {
            self.lance_store.search_vectors(vec, limit * 2, None).await.unwrap_or_default()
        } else {
            Vec::new()
        };

        // 3. Reciprocal Rank Fusion blending
        let empty_tombstones = std::collections::HashSet::new();
        let (mut fused, _) = self.rrf.fuse_with_query(query, &lexical_candidates, &vector_candidates, &empty_tombstones);
        fused.truncate(limit);
        Ok(fused)
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
