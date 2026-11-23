//! Thread-safe shared application state and RAII connection tracking for `souls_server`.
//!
//! Enforces:
//! - Clean, immutable references on the hot-path (no state Mutex for GGUF handles;
//!   lifetime managed by `InferenceRuntime`).
//! - RAII `ConnectionGuard` guaranteeing zero leakage of active connection counters.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;

use souls_inference_runtime::InferenceRuntime;
use souls_memory::SoulsMemoryStore;
use souls_model_router::ParetoBanditRouter;

use souls_llm_cloud_arena::CloudArenaCoordinator;

/// RAII Guard that increments active connection count on creation
/// and atomically decrements it on drop, preventing leaks on early returns or panics.
#[derive(Debug)]
pub struct ConnectionGuard {
    counter: Arc<AtomicUsize>,
}

impl ConnectionGuard {
    /// Creates a new guard and increments the active connection counter.
    pub fn new(counter: Arc<AtomicUsize>) -> Self {
        counter.fetch_add(1, Ordering::SeqCst);
        Self { counter }
    }
}

impl Drop for ConnectionGuard {
    fn drop(&mut self) {
        self.counter.fetch_sub(1, Ordering::SeqCst);
    }
}

/// Zero-overhead RAII marker for passive observability endpoints (`/health`, `/telemetry`,
/// `/harness/overview`, `/harness/cues`) that must neither count as active load nor
/// reset Catnap idle time.
#[derive(Debug)]
pub struct PassiveConnectionGuard;

/// Thread-safe immutable application state shared across Axum handlers.
#[derive(Clone)]
pub struct AppState {
    pub memory: Arc<SoulsMemoryStore>,
    pub runtime: Arc<InferenceRuntime>,
    pub router: Arc<ParetoBanditRouter>,
    pub cloud_arena: Arc<CloudArenaCoordinator>,
    pub mcp_sessions: Arc<crate::mcp::McpSessionManager>,
    pub inventory_downloader: Arc<souls_llm_inventory::DownloadManager>,
    pub hf_client: Arc<souls_llm_inventory::HuggingFaceClient>,
    pub local_arena: Arc<tokio::sync::Mutex<souls_llm_local_arena::ArenaRunner>>,
    pub research: Arc<crate::research_hub::ResearchIndex>,
    pub google_hub: Arc<crate::google_hub::GoogleHubService>,
    pub evolutive_harness: Arc<crate::routes::harness::EvolutiveHarnessService>,
    active_connections: Arc<AtomicUsize>,
    start_time: Instant,
}

impl AppState {
    /// Constructs a new AppState with provided engine components.
    pub fn new(
        memory: Arc<SoulsMemoryStore>,
        runtime: Arc<InferenceRuntime>,
        router: Arc<ParetoBanditRouter>,
        cloud_arena: Arc<CloudArenaCoordinator>,
    ) -> Self {
        Self {
            memory,
            runtime,
            router,
            cloud_arena,
            mcp_sessions: Arc::new(crate::mcp::McpSessionManager::new()),
            inventory_downloader: Arc::new(souls_llm_inventory::DownloadManager::new()),
            hf_client: Arc::new(souls_llm_inventory::HuggingFaceClient::new()),
            local_arena: Arc::new(tokio::sync::Mutex::new(souls_llm_local_arena::ArenaRunner::new())),
            research: Arc::new(crate::research_hub::ResearchIndex::new(crate::research_hub::ResearchRoots::from_system())),
            google_hub: Arc::new(crate::google_hub::GoogleHubService::from_system()),
            evolutive_harness: Arc::new(crate::routes::harness::EvolutiveHarnessService::from_system()),
            active_connections: Arc::new(AtomicUsize::new(0)),
            start_time: Instant::now(),
        }
    }

    /// Overrides the `GoogleHubService` instance for isolated unit/integration tests.
    pub fn with_google_hub(mut self, google_hub: Arc<crate::google_hub::GoogleHubService>) -> Self {
        self.google_hub = google_hub;
        self
    }

    /// Overrides the `EvolutiveHarnessService` instance for isolated unit/integration tests.
    pub fn with_evolutive_harness(
        mut self,
        evolutive_harness: Arc<crate::routes::harness::EvolutiveHarnessService>,
    ) -> Self {
        self.evolutive_harness = evolutive_harness;
        self
    }

    /// Acquires an RAII connection guard incrementing active connections AND signaling
    /// real user/agent work activity (`< 1 ms` Catnap preemption).
    #[inline]
    pub fn acquire_connection(&self) -> ConnectionGuard {
        self.evolutive_harness.notify_activity();
        ConnectionGuard::new(Arc::clone(&self.active_connections))
    }

    /// Acquires a passive observability marker guard (health/telemetry polling)
    /// WITHOUT incrementing `active_connections`, resetting Catnap idle time, or causing preemption.
    #[inline]
    pub fn acquire_passive_connection(&self) -> PassiveConnectionGuard {
        PassiveConnectionGuard
    }

    /// Returns the current number of active HTTP connections.
    #[inline]
    pub fn active_connections(&self) -> usize {
        self.active_connections.load(Ordering::SeqCst)
    }

    /// Returns the uptime of the server in seconds.
    #[inline]
    pub fn uptime_secs(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }
}
