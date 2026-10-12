//! # MCP Session Manager (`souls_server::mcp::session`)
//!
//! Governs active Model Context Protocol client connections, SSE channels,
//! periodic keep-alive heartbeats (15s), and idle session eviction (30min TTL).

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use axum::response::sse::Event;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, info};

static SESSION_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Canonical keep-alive interval: 15 seconds.
pub const MCP_KEEP_ALIVE_INTERVAL: Duration = Duration::from_secs(15);

/// Canonical session TTL: 30 minutes (1800 seconds).
pub const MCP_SESSION_TTL_SECS: u64 = 1800;

fn current_epoch_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Active MCP Client Session holding SSE transmitter channel.
pub struct McpSession {
    pub session_id: String,
    pub tx: mpsc::Sender<Event>,
    pub created_at: Instant,
    pub last_active_epoch: AtomicU64,
    pub client_info: RwLock<Option<serde_json::Value>>,
}

impl McpSession {
    pub fn new(session_id: String, tx: mpsc::Sender<Event>) -> Self {
        Self {
            session_id,
            tx,
            created_at: Instant::now(),
            last_active_epoch: AtomicU64::new(current_epoch_secs()),
            client_info: RwLock::new(None),
        }
    }

    /// Touches the session to refresh the idle TTL.
    #[inline]
    pub fn touch(&self) {
        self.last_active_epoch
            .store(current_epoch_secs(), Ordering::Relaxed);
    }

    /// Checks if the session has exceeded the specified TTL in seconds.
    #[inline]
    pub fn is_expired(&self, ttl_secs: u64) -> bool {
        let last = self.last_active_epoch.load(Ordering::Relaxed);
        let now = current_epoch_secs();
        now.saturating_sub(last) > ttl_secs
    }
}

/// Thread-safe in-memory MCP Session Manager.
#[derive(Clone)]
pub struct McpSessionManager {
    sessions: Arc<RwLock<HashMap<String, Arc<McpSession>>>>,
}

impl Default for McpSessionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl McpSessionManager {
    pub fn new() -> Self {
        let sessions: Arc<RwLock<HashMap<String, Arc<McpSession>>>> =
            Arc::new(RwLock::new(HashMap::new()));
        let background_sessions = Arc::clone(&sessions);

        // Background worker sweeping idle sessions every 60 seconds
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
                let mut map = background_sessions.write().await;
                map.retain(|id, session| {
                    let keep = !session.is_expired(MCP_SESSION_TTL_SECS);
                    if !keep {
                        debug!("Evicting idle MCP session: {id}");
                    }
                    keep
                });
            }
        });

        Self { sessions }
    }

    /// Generates a unique monotonic nanosecond session ID.
    pub fn generate_session_id() -> String {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let counter = SESSION_COUNTER.fetch_add(1, Ordering::Relaxed);
        format!("mcp-{nanos:x}-{counter:x}")
    }

    /// Registers a new active session with its SSE sender.
    pub async fn create_session(&self, tx: mpsc::Sender<Event>) -> Arc<McpSession> {
        let session_id = Self::generate_session_id();
        let session = Arc::new(McpSession::new(session_id.clone(), tx));
        let mut map = self.sessions.write().await;
        map.insert(session_id.clone(), Arc::clone(&session));
        info!("MCP session registered: {session_id} (active: {})", map.len());
        session
    }

    /// Retrieves an existing session by ID and touches its TTL.
    pub async fn get_session(&self, session_id: &str) -> Option<Arc<McpSession>> {
        let map = self.sessions.read().await;
        if let Some(session) = map.get(session_id) {
            session.touch();
            Some(Arc::clone(session))
        } else {
            None
        }
    }

    /// Explicitly removes a disconnected session.
    pub async fn remove_session(&self, session_id: &str) -> Option<Arc<McpSession>> {
        let mut map = self.sessions.write().await;
        let removed = map.remove(session_id);
        if removed.is_some() {
            info!("MCP session removed: {session_id} (active: {})", map.len());
        }
        removed
    }

    /// Sends a JSON-RPC response or notification event to an active session.
    pub async fn send_event(&self, session_id: &str, event: Event) -> Result<(), String> {
        let session = self
            .get_session(session_id)
            .await
            .ok_or_else(|| format!("Session not found: {session_id}"))?;

        session
            .tx
            .send(event)
            .await
            .map_err(|e| format!("Failed to send event to session {session_id}: {e}"))
    }

    /// Sweeps and evicts idle sessions exceeding the TTL.
    pub async fn sweep_expired(&self, ttl_secs: u64) -> usize {
        let mut map = self.sessions.write().await;
        let initial_count = map.len();
        map.retain(|id, session| {
            let keep = !session.is_expired(ttl_secs);
            if !keep {
                debug!("Evicting expired MCP session: {id}");
            }
            keep
        });
        initial_count.saturating_sub(map.len())
    }

    /// Returns the number of currently active sessions.
    pub async fn active_count(&self) -> usize {
        self.sessions.read().await.len()
    }
}
