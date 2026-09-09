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

/// Thread-safe immutable application state shared across Axum handlers.
#[derive(Clone)]
pub struct AppState {
    pub memory: Arc<SoulsMemoryStore>,
    pub runtime: Arc<InferenceRuntime>,
    pub router: Arc<ParetoBanditRouter>,
    active_connections: Arc<AtomicUsize>,
    start_time: Instant,
}

impl AppState {
    /// Constructs a new AppState with provided engine components.
    pub fn new(
        memory: Arc<SoulsMemoryStore>,
        runtime: Arc<InferenceRuntime>,
        router: Arc<ParetoBanditRouter>,
    ) -> Self {
        Self {
            memory,
            runtime,
            router,
            active_connections: Arc::new(AtomicUsize::new(0)),
            start_time: Instant::now(),
        }
    }

    /// Acquires an RAII connection guard incrementing the active connections count.
    #[inline]
    pub fn acquire_connection(&self) -> ConnectionGuard {
        ConnectionGuard::new(Arc::clone(&self.active_connections))
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
