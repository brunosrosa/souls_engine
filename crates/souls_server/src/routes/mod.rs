//! Route definitions and router construction for `souls_server`.

pub mod chat;
pub mod cloud;
pub mod dashboard;
pub mod health;
pub mod inventory;
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
        // Dashboard & Branding Web Endpoints
        .route("/", get(dashboard::dashboard_html_handler))
        .route("/dashboard", get(dashboard::dashboard_html_handler))
        .route("/dashboard/icon/:color", get(dashboard::dashboard_icon_handler))
        .route("/api/v1/telemetry", get(dashboard::telemetry_handler))
        .route("/api/v1/telemetry/stream", get(dashboard::telemetry_stream_handler))
        .route("/api/v1/models/local", get(dashboard::local_models_handler))
        .route("/api/v1/router/matrix", get(dashboard::router_matrix_handler))
        .route("/api/v1/settings/hermes-preset", post(dashboard::hermes_preset_handler))
        .route("/api/v1/settings/config", get(dashboard::get_config_handler).post(dashboard::post_config_handler))
        .route("/api/v1/profiles", get(dashboard::list_profiles_handler).post(dashboard::create_profile_handler))
        .route("/api/v1/profiles/:id", axum::routing::delete(dashboard::delete_profile_handler))
        .route("/api/v1/system/checkpoint", post(dashboard::system_checkpoint_handler))
        .route("/api/v1/system/curate-obsidian", post(dashboard::system_curate_obsidian_handler))
        // Core API Endpoints
        .route("/health", get(health::health_handler))
        .route("/v1/models", get(models::models_handler))
        .route("/v1/chat/completions", post(chat::chat_completions_handler))
        .route("/api/v1/cloud/champions", get(cloud::cloud_champions_handler))
        .route("/api/v1/cloud/models", get(cloud::cloud_ranked_models_handler))
        .route("/api/v1/cloud/sync", post(cloud::cloud_sync_handler))
        // Model Hub & Inventory Endpoints (RFC-007)
        .route("/api/v1/inventory/hardware", get(inventory::inventory_hardware_handler))
        .route("/api/v1/inventory/recommendations", get(inventory::inventory_recommendations_handler))
        .route("/api/v1/inventory/models", get(inventory::inventory_models_handler))
        .route("/api/v1/inventory/search", get(inventory::inventory_search_handler))
        .route("/api/v1/inventory/repo-files", get(inventory::inventory_repo_files_handler))
        .route("/api/v1/inventory/download", post(inventory::inventory_download_handler))
        .route("/api/v1/inventory/downloads", get(inventory::inventory_list_downloads_handler))
        .route("/api/v1/inventory/download/:id/pause", post(inventory::inventory_pause_download_handler))
        .route("/api/v1/inventory/download/:id/resume", post(inventory::inventory_resume_download_handler))
        .route("/api/v1/inventory/download/:id/cancel", post(inventory::inventory_cancel_download_handler))
        .route("/api/v1/inventory/models/:filename", axum::routing::delete(inventory::inventory_delete_model_handler))
        .route("/api/v1/inventory/downloads/stream", get(inventory::inventory_download_stream_handler))
        // Modern Streamable HTTP endpoint (MCP 2025-03-26+)
        .route("/mcp", post(mcp_post_handler).get(mcp_get_handler))
        // Legacy HTTP+SSE endpoints (MCP 2024-11-05)
        .route("/sse", get(sse_legacy_handler))
        .route("/messages", post(messages_legacy_handler))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
