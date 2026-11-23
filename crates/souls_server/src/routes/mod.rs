//! Route definitions and router construction for `souls_server`.

pub mod arena;
pub mod audio;
pub mod chat;
pub mod cloud;
pub mod connections;
pub mod dashboard;
pub mod google;
pub mod harness;
pub mod health;
pub mod inbox;
pub mod inventory;
pub mod memory;
pub mod models;
pub mod sessions;
pub mod switchboard;
pub mod vision;

use axum::routing::{delete, get, post};
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
        // Milestone v0.9.2: Google Life Hub, Multi-Account OAuth2 PKCE, Notebooks Audio & BMAD Agent Inbox
        .route("/api/v1/inbox/proposals", get(inbox::list_proposals_handler))
        .route("/api/v1/inbox/proposals/:id", get(inbox::get_proposal_handler))
        .route("/api/v1/inbox/proposals/:id/approve", post(inbox::approve_proposal_handler))
        .route("/api/v1/inbox/proposals/:id/reject", post(inbox::reject_proposal_handler))
        .route("/api/v1/inbox/audit", get(inbox::list_audit_handler))
        .route("/api/v1/google/accounts", get(google::list_accounts_handler).post(google::upsert_account_handler))
        .route("/api/v1/google/accounts/:id", delete(google::delete_account_handler))
        .route("/api/v1/google/accounts/:id/policy", post(google::update_account_policy_handler))
        .route("/api/v1/google/accounts/import-gws", post(google::import_gws_handler))
        .route("/api/v1/google/oauth/start", post(google::oauth_start_handler))
        .route("/api/v1/google/oauth/upload-credentials", post(google::oauth_upload_credentials_handler))
        .route("/api/v1/google/oauth/callback", get(google::oauth_callback_handler))
        .route("/api/v1/google/nlm/login", post(google::nlm_login_handler))
        .route("/api/v1/google/gws/login", post(google::gws_login_handler))
        .route("/api/v1/google/doctor", get(google::google_doctor_handler))
        .route("/api/v1/google/briefing", get(google::briefing_handler))
        .route("/api/v1/google/gmail/triage", get(google::gmail_triage_handler))
        .route("/api/v1/google/gmail/draft", post(google::gmail_draft_handler))
        .route("/api/v1/google/gmail/bulk", post(google::gmail_bulk_handler))
        .route("/api/v1/google/gmail/send", post(google::gmail_send_handler))
        .route("/api/v1/google/calendar/events", get(google::calendar_events_handler).post(google::calendar_create_event_handler))
        .route("/api/v1/google/calendar/focus-slots", get(google::calendar_focus_slots_handler))
        .route("/api/v1/google/drive/search", get(google::drive_search_handler))
        .route("/api/v1/google/drive/action", post(google::drive_action_handler))
        .route("/api/v1/google/docs/scratchpad", get(google::docs_scratchpad_get_handler).post(google::docs_scratchpad_edit_handler))
        .route("/api/v1/google/docs/scratchpad/push", post(google::docs_scratchpad_push_handler))
        .route("/api/v1/google/sheets/analyze", post(google::sheets_analyze_handler))
        .route("/api/v1/google/sheets/write", post(google::sheets_write_handler))
        .route("/api/v1/google/slides/plan", post(google::slides_plan_handler))
        .route("/api/v1/google/tasks", get(google::tasks_list_handler).post(google::tasks_mutate_handler))
        .route("/api/v1/google/quick-capture", post(google::quick_capture_handler))
        .route("/api/v1/google/contacts", get(google::contacts_handler))
        .route("/api/v1/google/maps/commute", post(google::maps_commute_handler))
        .route("/api/v1/google/youtube/transcript", post(google::youtube_transcript_handler))
        .route("/api/v1/google/gems", get(google::gems_list_handler).post(google::gems_mutate_handler))
        .route("/api/v1/google/notebooks", get(google::notebooks_list_handler))
        .route("/api/v1/google/notebooks/:id", get(google::notebook_get_handler))
        .route("/api/v1/google/notebooks/:id/sources", post(google::notebook_add_source_handler))
        .route("/api/v1/google/notebooks/:id/sync-drive", post(google::notebook_sync_drive_handler))
        .route("/api/v1/google/notebooks/:id/query", post(google::notebook_query_handler))
        .route("/api/v1/google/notebooks/:id/studio", post(google::notebook_studio_handler))
        .route("/api/v1/google/notebooks/:id/audio/:artifact_id", get(google::notebook_audio_stream_handler))
        // Milestone v0.9.2.1: Proactive Pulse, Apps Script 24/7, Forms, Chat, MinionS & Auto-Sync
        .route("/api/v1/google/pulse", get(google::pulse_status_handler))
        .route("/api/v1/google/pulse/run", post(google::pulse_run_handler))
        .route(
            "/api/v1/google/pulse/config",
            get(google::pulse_get_config_handler).post(google::pulse_update_config_handler),
        )
        .route("/api/v1/google/pulse/pending", get(google::pulse_pending_handler))
        .route("/api/v1/google/pulse/ack", post(google::pulse_ack_handler))
        .route("/api/v1/google/events/stream", get(google::google_events_stream_handler))
        .route("/api/v1/google/apps-script/templates", get(google::apps_script_templates_handler))
        .route("/api/v1/google/apps-script/create", post(google::apps_script_create_handler))
        .route("/api/v1/google/apps-script/:id/content", get(google::apps_script_get_content_handler))
        .route("/api/v1/google/apps-script/:id/push", post(google::apps_script_push_propose_handler))
        .route("/api/v1/google/apps-script/:id/deploy", post(google::apps_script_deploy_propose_handler))
        .route("/api/v1/google/apps-script/:id/run", post(google::apps_script_run_propose_handler))
        .route("/api/v1/google/apps-script/:id/processes", get(google::apps_script_processes_handler))
        .route("/api/v1/google/forms", post(google::forms_create_propose_handler))
        .route("/api/v1/google/forms/:id", get(google::forms_get_handler))
        .route("/api/v1/google/forms/:id/responses", get(google::forms_responses_handler))
        .route("/api/v1/google/chat/spaces", get(google::chat_spaces_handler))
        .route("/api/v1/google/chat/send", post(google::chat_send_propose_handler))
        .route("/api/v1/google/analyze/minions", post(google::analyze_minions_handler))
        .route(
            "/api/v1/google/notebooks/sync-links",
            get(google::sync_links_list_handler).post(google::sync_links_upsert_handler),
        )
        .route(
            "/api/v1/google/notebooks/sync-links/:id",
            delete(google::sync_links_delete_handler),
        )
        .route(
            "/api/v1/google/notebooks/sync-links/:id/toggle",
            post(google::sync_links_toggle_handler),
        )
        // Milestone v0.8.3: Cognitive Vision O(1) & AXTree LEAN (ADR-029)
        .route("/api/v1/vision/status", get(vision::vision_status_handler))
        .route("/api/v1/vision/axtree/serialize", post(vision::serialize_axtree_handler))
        .route("/api/v1/vision/axtree/hit-test", post(vision::hit_test_handler))
        // Milestone v0.9.3: Hybrid Vision, Sovereign PT-BR Audio & OpenAI Audio Gateway
        .route("/api/v1/vision/axtree/live", post(vision::live_axtree_handler))
        .route("/api/v1/vision/axtree/diff", post(vision::diff_axtree_handler))
        .route("/api/v1/vision/snapshot", post(vision::snapshot_vision_handler))
        .route("/v1/audio/speech", post(audio::openai_audio_speech_handler))
        .route("/v1/audio/transcriptions", post(audio::openai_audio_transcriptions_handler))
        .route("/api/v1/audio/status", get(audio::audio_status_handler))
        .route("/api/v1/audio/transcribe", post(audio::transcribe_audio_handler))
        .route("/api/v1/audio/synthesize", post(audio::synthesize_audio_handler))
        .route(
            "/api/v1/audio/rapport",
            get(audio::get_rapport_handler).post(audio::post_rapport_handler),
        )
        .route("/api/v1/audio/provision", post(audio::provision_audio_handler))
        .route("/api/v1/hermes/voice-customs", get(audio::hermes_voice_customs_handler))
        .route("/api/v1/hermes/voice-customs/sync", post(audio::hermes_voice_customs_sync_handler))
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
        // Milestone v0.9.4: Souls Evolutive Harness (Cordis, Catnap, Tombstones, WeEvolve & Ralph Loop)
        .route("/api/v1/harness/overview", get(harness::overview_handler))
        .route("/api/v1/harness/cues", get(harness::cues_handler))
        .route("/api/v1/harness/catnap/config", post(harness::update_catnap_config_handler))
        .route("/api/v1/harness/catnap/trigger", post(harness::trigger_catnap_slice_handler))
        .route("/api/v1/harness/cordis/mount", post(harness::mount_cordis_fiber_handler))
        .route("/api/v1/harness/cordis/dispose", post(harness::dispose_cordis_fiber_handler))
        .route("/api/v1/harness/cordis/reactivate", post(harness::reactivate_cordis_fiber_handler))
        .route("/api/v1/harness/tombstones/resolve", post(harness::resolve_tombstone_handler))
        .route("/api/v1/harness/tombstones/restore", post(harness::restore_tombstone_handler))
        .route("/api/v1/harness/ralph/crystallize", post(harness::crystallize_ralph_skill_handler))
        .route("/api/v1/harness/ralph/approve", post(harness::approve_ralph_skill_handler))
        .route("/api/v1/harness/weevolve/signal", post(harness::record_weevolve_signal_handler))
        // Core API Endpoints
        .route("/health", get(health::health_handler))
        .route("/v1/models", get(models::models_handler))
        .route("/v1/chat/completions", post(chat::chat_completions_handler))
        .route("/api/v1/cloud/champions", get(cloud::cloud_champions_handler))
        .route("/api/v1/cloud/models", get(cloud::cloud_ranked_models_handler))
        .route("/api/v1/cloud/sync", post(cloud::cloud_sync_handler))
        .route("/api/v1/cloud/finops", get(cloud::cloud_finops_handler))
        .route("/api/v1/hermes/customs", get(cloud::hermes_customs_handler))
        .route("/api/v1/hermes/customs/repair", post(cloud::hermes_customs_repair_handler))
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
