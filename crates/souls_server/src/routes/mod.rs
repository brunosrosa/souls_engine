//! Route definitions and router construction for `souls_server`.

pub mod chat;
pub mod cloud;
pub mod health;
pub mod models;

use axum::routing::{get, post};
use axum::Router;
use tower_http::trace::TraceLayer;

use crate::mcp::transport::{
    mcp_get_handler, mcp_post_handler, messages_legacy_handler, sse_legacy_handler,
};
use crate::state::AppState;

/// Constructs the primary Axum router.
pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health::health_handler))
        .route("/v1/models", get(models::models_handler))
        .route("/v1/chat/completions", post(chat::chat_completions_handler))
        .route("/api/v1/cloud/champions", get(cloud::cloud_champions_handler))
        .route("/api/v1/cloud/sync", post(cloud::cloud_sync_handler))
        // Modern Streamable HTTP endpoint (MCP 2025-03-26+)
        .route("/mcp", post(mcp_post_handler).get(mcp_get_handler))
        // Legacy HTTP+SSE endpoints (MCP 2024-11-05)
        .route("/sse", get(sse_legacy_handler))
        .route("/messages", post(messages_legacy_handler))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
