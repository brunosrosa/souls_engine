//! Route definitions and router construction for `souls_server`.

pub mod arena;
pub mod chat;
pub mod cloud;
pub mod connections;
pub mod dashboard;
pub mod google;
pub mod health;
pub mod inventory;
pub mod memory;
pub mod models;
pub mod sessions;
pub mod switchboard;
pub mod vision;

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
        .route("/api/v1/router/fallback/chain", post(dashboard::router_fallback_chain_handler))
        .route("/api/v1/router/quarantines", get(dashboard::router_quarantines_get_handler).post(dashboard::router_quarantines_reset_handler))
        .route("/api/v1/router/providers", get(dashboard::router_providers_handler))
        .route("/api/v1/settings/hermes-preset", post(dashboard::hermes_preset_handler))
        .route("/api/v1/settings/config", get(dashboard::get_config_handler).post(dashboard::post_config_handler))
        .route("/api/v1/settings/config/validate", post(dashboard::validate_config_toml_handler))
        .route("/api/v1/settings/config/restore", post(dashboard::restore_config_backup_handler))
        .route("/api/v1/profiles", get(dashboard::list_profiles_handler).post(dashboard::create_profile_handler))
        .route("/api/v1/profiles/:id", axum::routing::delete(dashboard::delete_profile_handler))
        .route("/api/v1/system/checkpoint", post(dashboard::system_checkpoint_handler))
        .route("/api/v1/system/pulse", get(dashboard::system_pulse_handler))
        .route("/api/v1/system/storage/provision", post(dashboard::system_storage_provision_handler))
        .route("/api/v1/system/vault/validate", post(dashboard::system_vault_validate_handler))
        .route("/api/v1/system/curate-obsidian", post(dashboard::system_curate_obsidian_handler))
        .route("/api/v1/vault/sanitize", post(dashboard::vault_sanitize_handler))
        .route("/api/v1/vault/sample-note", get(dashboard::vault_sample_note_handler))
        .route("/api/v1/system/agents-md", get(dashboard::agents_md_handler))
        .route("/api/v1/tools/catalog", get(dashboard::tools_catalog_handler))
        // Milestone v0.6.0: Skills Hub, Zero-Trust Explorer & Agentic Forge Endpoints
        .route("/api/v1/skills/installed", get(dashboard::skills_hub_list_handler))
        .route("/api/v1/skills/detail", get(dashboard::skills_hub_detail_handler))
        .route("/api/v1/skills/sync", post(dashboard::skills_hub_sync_handler))
        .route("/api/v1/skills/state", post(dashboard::skills_hub_toggle_state_handler))
        .route("/api/v1/skills/marketplace/search", get(dashboard::skills_hub_marketplace_search_handler))
        .route("/api/v1/skills/quarantine", get(dashboard::skills_hub_quarantine_list_handler))
        .route("/api/v1/skills/quarantine/fetch", post(dashboard::skills_hub_quarantine_fetch_handler))
        .route("/api/v1/skills/quarantine/discard", post(dashboard::skills_hub_quarantine_discard_handler))
        .route("/api/v1/skills/quarantine/clear", post(dashboard::skills_hub_quarantine_clear_handler))
        .route("/api/v1/skills/forge/generate", post(dashboard::skills_hub_forge_generate_handler))
        .route("/api/v1/skills/diff", post(dashboard::skills_hub_diff_handler))
        .route("/api/v1/skills/rollback", post(dashboard::skills_hub_rollback_handler))
        .route("/api/v1/skills/autofix", post(dashboard::skills_hub_autofix_handler))
        .route("/api/v1/skills/delete", post(dashboard::skills_hub_delete_handler))
        // Milestone v0.6.0 (Delivery 2): Hermes Personas & Governance Inspector Endpoints
        .route("/api/v1/personas/list", get(dashboard::personas_hub_list_handler))
        .route("/api/v1/personas/detail", get(dashboard::personas_hub_detail_handler))
        .route("/api/v1/personas/avatar", get(dashboard::personas_hub_avatar_handler))
        .route("/api/v1/personas/update", post(dashboard::personas_hub_update_handler))
        .route("/api/v1/personas/align", post(dashboard::personas_hub_align_handler))
        .route("/api/v1/personas/create", post(dashboard::personas_hub_create_handler))
        .route("/api/v1/personas/refine-soul", post(dashboard::personas_hub_refine_soul_handler))
        .route("/api/v1/personas/diff", post(dashboard::personas_hub_diff_handler))
        .route("/api/v1/personas/archive", post(dashboard::personas_hub_archive_handler))
        .route("/api/v1/personas/rollback", post(dashboard::personas_hub_rollback_handler))
        // Milestone v0.7.0: Deep Research Hub & Mission Control Plane Endpoints
        .route("/api/v1/research/missions", get(dashboard::research_hub_list_handler).post(dashboard::research_hub_create_handler))
        .route("/api/v1/research/mission", get(dashboard::research_hub_detail_handler))
        .route("/api/v1/research/plan", post(dashboard::research_hub_plan_handler))
        .route("/api/v1/research/mission/cancel", post(dashboard::research_hub_cancel_handler))
        .route("/api/v1/research/mission/archive", post(dashboard::research_hub_archive_handler))
        .route("/api/v1/research/doctor", get(dashboard::research_hub_doctor_handler))
        .route("/api/v1/research/stream", get(dashboard::research_hub_stream_handler))
        .route("/api/v1/research/mission/steer", post(dashboard::research_hub_steer_handler))
        .route("/api/v1/research/report", get(dashboard::research_hub_report_handler))
        .route("/api/v1/research/provision", post(dashboard::research_hub_provision_handler))
        .route("/api/v1/research/mission/refresh", post(dashboard::research_hub_refresh_handler))
        .route("/api/v1/research/mission/branch", post(dashboard::research_hub_branch_handler))
        .route("/api/v1/research/diff", get(dashboard::research_hub_diff_handler))
        .route("/api/v1/research/promote", post(dashboard::research_hub_promote_handler))
        .route("/api/v1/research/domains", get(dashboard::research_hub_domains_get_handler).post(dashboard::research_hub_domains_post_handler))
        .route("/api/v1/research/feedback", post(dashboard::research_hub_feedback_post_handler))
        .route("/api/v1/research/scorecard", get(dashboard::research_hub_scorecard_handler))
        .route("/api/v1/research/settings", get(dashboard::research_hub_settings_get_handler).post(dashboard::research_hub_settings_post_handler))
        .route("/api/v1/research/skill-patch", post(dashboard::research_hub_skill_patch_handler))
        .route("/api/v1/research/tier0/process", post(dashboard::research_tier0_process_handler))
        .route("/api/v1/research/extract-pdf", post(dashboard::research_extract_pdf_handler))
        .route("/api/v1/research/cache/lookup", post(dashboard::research_cache_lookup_handler))
        .route("/api/v1/research/cache/purge", post(dashboard::research_cache_purge_handler))
        .route("/api/v1/router/incident", post(dashboard::router_incident_report_handler))
        // Milestone v0.8.1: Global Connections & Agentic CLIs Endpoints
        .route("/api/v1/connections/providers", get(connections::list_providers_handler))
        .route("/api/v1/connections/test", post(connections::test_provider_handler))
        .route("/api/v1/connections/clis", get(connections::list_clis_handler))
        .route("/api/v1/connections/cli/dispatch", post(connections::dispatch_cli_handler))
        .route("/api/v1/system/credentials/save", post(connections::save_credential_handler))
        // Milestone v0.8.2: Google Universe Bridge & NotebookLM Reverse Engineering
        .route("/api/v1/google/session", get(google::get_session_handler).post(google::save_session_handler))
        .route("/api/v1/google/notebooklm/rpc", post(google::wiz_rpc_handler))
        .route("/api/v1/google/workspace/status", get(google::workspace_status_handler))
        .route("/api/v1/google/workspace/link", post(google::workspace_link_handler))
        .route("/api/v1/google/jules/migrate", post(google::jules_migrate_handler))
        // Milestone v0.8.3: Cognitive Vision O(1) & AXTree LEAN (ADR-029)
        .route("/api/v1/vision/status", get(vision::vision_status_handler))
        .route("/api/v1/vision/axtree/serialize", post(vision::serialize_axtree_handler))
        .route("/api/v1/vision/axtree/hit-test", post(vision::hit_test_handler))
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
