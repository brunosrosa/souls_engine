//! SOULS ENGINE — Memory Subsystem (Hipocampo L3 Soberano).
//!
//! Provides FrankenSQLite STRICT WAL persistence, CPU-accelerated RRF hybrid fusion,
//! Langevin Decay thermal metabolism with STABLE immunity, and LadybugDB ontological causal graph.

pub mod chyros;
pub mod error;
pub mod ladybug;
pub mod rrf;
pub mod sqlite;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use arc_swap::ArcSwap;
use sqlx::{Pool, Sqlite};
use tracing::info;

pub use chyros::{ChyrosDaemon, ChyrosMetabolismReport};
pub use error::MemoryError;
pub use ladybug::{LadybugOntologyGraph, OntologicalEdge, OntologicalNode};
pub use rrf::{LexicalCandidate, RrfFusionEngine, UnifiedMatch, VectorCandidate};
pub use sqlite::{create_in_memory_sqlite_pool, create_sqlite_pool};

/// Central governance coordinator for all memory, persistence, and graph subsystems.
#[derive(Debug, Clone)]
pub struct SoulsMemoryStore {
    db_path: PathBuf,
    pool_swap: Arc<ArcSwap<Pool<Sqlite>>>,
    ladybug: LadybugOntologyGraph,
    rrf: RrfFusionEngine,
    chyros: Arc<ChyrosDaemon>,
}

impl SoulsMemoryStore {
    /// Bootstraps the memory store against a canonical persistent Dev Drive ReFS path.
    pub async fn new<P: AsRef<Path>>(
        db_path: P,
        max_connections: u32,
    ) -> Result<Self, MemoryError> {
        let path = db_path.as_ref().to_path_buf();
        let pool = create_sqlite_pool(&path, max_connections).await?;
        let ladybug = LadybugOntologyGraph::new();
        ladybug.load_from_db(&pool).await?;

        let rrf = RrfFusionEngine::default();
        let chyros = Arc::new(ChyrosDaemon::new(&path)?);

        Ok(Self {
            db_path: path,
            pool_swap: Arc::new(ArcSwap::from_pointee(pool)),
            ladybug,
            rrf,
            chyros,
        })
    }

    /// Bootstraps an in-memory memory store for hermetic unit and integration testing.
    pub async fn new_in_memory() -> Result<Self, MemoryError> {
        let pool = create_in_memory_sqlite_pool().await?;
        let ladybug = LadybugOntologyGraph::new();
        let rrf = RrfFusionEngine::default();
        let chyros = Arc::new(ChyrosDaemon::new_in_memory());

        Ok(Self {
            db_path: PathBuf::from(":memory:"),
            pool_swap: Arc::new(ArcSwap::from_pointee(pool)),
            ladybug,
            rrf,
            chyros,
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
