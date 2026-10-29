//! Route definitions and router construction for `souls_server`.

pub mod arena;
pub mod chat;
pub mod cloud;
pub mod dashboard;
pub mod health;
pub mod inventory;
pub mod memory;
pub mod models;
pub mod sessions;
pub mod switchboard;

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
        // Local Silicon Arena Endpoints
        .route("/api/v1/arena/models", get(arena::arena_models_handler))
        .route("/api/v1/arena/benchmarks", get(arena::arena_benchmarks_handler))
        .route("/api/v1/arena/benchmark/run", post(arena::arena_run_benchmark_handler))
        .route("/api/v1/arena/sync", post(arena::arena_sync_handler))
        // Two-Stage Semantic Switchboard Endpoints (<1.7ms)
        .route("/api/v1/switchboard/evaluate", post(switchboard::evaluate_handler))
        .route("/api/v1/switchboard/skills", get(switchboard::list_skills_handler))
        .route("/api/v1/switchboard/health", get(switchboard::health_handler))
        // Sub-Agent Sessions & Handoff Endpoints (L3 Isolation)
        .route("/api/v1/sessions/subagents", get(sessions::list_subagents_handler))
        .route("/api/v1/sessions/subagents/cleanup", post(sessions::cleanup_subagents_handler))
        .route("/api/v1/sessions/subagents/:id", get(sessions::get_subagent_details_handler))
        .route("/api/v1/sessions/subagents/spawn", post(sessions::spawn_subagent_handler))
        .route("/api/v1/sessions/subagents/:id/status", post(sessions::update_status_handler))
        .route("/api/v1/sessions/subagents/:id/diary", post(sessions::record_diary_handler))
        .route("/api/v1/sessions/subagents/:id/complete", post(sessions::complete_subagent_handler))
        .route("/api/v1/sessions/handoffs", post(sessions::create_handoff_handler))
        .route("/api/v1/sessions/handoffs/:agent_id", get(sessions::get_handoff_handler))
        // Bidirectional Memory Bridge & Frozen Snapshot Endpoints (Item A)
        .route("/api/v1/memory/snapshot", get(memory::get_snapshot_handler))
        .route("/api/v1/memory/snapshot/pin", post(memory::pin_snapshot_handler))
        .route("/api/v1/memory/recall", post(memory::recall_handler))
        .route("/api/v1/memory/mirror", post(memory::mirror_handler))
        .route("/api/v1/memory/turn", post(memory::turn_handler))
        .route("/api/v1/memory/checkpoint", post(memory::checkpoint_handler))
        .route("/api/v1/memory/overview", get(memory::overview_handler))
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
