//! # Google Universe Bridge & NotebookLM Reverse Engineering (`souls_core::google_bridge`)
//!
//! Provides encrypted local credential vaults, multi-account OAuth2 PKCE management,
//! BMAD prompt-injection/taint/inbox governance, Google Workspace & Life Hub REST clients,
//! Google Apps Script 24/7, Google Forms/Chat, Multi-Cadence Proactive Pulse,
//! Tier 0 MinionS corpus preparation, Wiz `batchexecute` RPC + `nlm` CLI bridge for NotebookLM,
//! Custom Gems catalog & dispatch, and Jules-to-Antigravity (`agy`) fleet migration.

#[forbid(unsafe_code)]
pub mod apps_script;
#[forbid(unsafe_code)]
pub mod dual_workspace;
#[forbid(unsafe_code)]
pub mod forms_chat;
#[forbid(unsafe_code)]
pub mod gems;
#[forbid(unsafe_code)]
pub mod inbox;
#[forbid(unsafe_code)]
pub mod life;
#[forbid(unsafe_code)]
pub mod minions_google;
#[forbid(unsafe_code)]
pub mod oauth;
#[forbid(unsafe_code)]
pub mod pulse;
#[forbid(unsafe_code)]
pub mod security;
#[forbid(unsafe_code)]
pub mod session;
#[forbid(unsafe_code)]
pub mod wiz_rpc;
#[forbid(unsafe_code)]
pub mod workspace;

pub use apps_script::{
    builtin_apps_script_templates, scan_apps_script_security, ClaspCliBridge,
    GoogleAppsScriptClient,
};
pub use dual_workspace::{DualLayerWorkspace, JulesMigrationRouter, LegacyJulesCommand};
pub use forms_chat::{
    check_chat_eligibility, compute_form_stats, GoogleChatClient, GoogleFormsClient,
};
pub use gems::{
    build_secondary_stream_generate_payload, dispatch_gem_official_api, GemsCatalogStore,
    SecondaryStreamGeneratePayload,
};
pub use inbox::{AgentInboxStore, NewProposalInput};
pub use life::{
    check_keep_eligibility, compute_commute_alert, decode_xml_entities, extract_video_id,
    parse_captions_xml, quick_capture_to_obsidian, upcoming_birthdays, GoogleTasksClient,
    PeopleClient, RouteComputation, RoutesV2Client, YouTubeClient,
};
pub use minions_google::{
    prepare_google_corpus_tier0, CachedCorpusChunks, Tier0MinionPrep,
};
pub use oauth::{
    base64url_decode, base64url_encode, build_google_auth_url, exchange_code_for_tokens,
    find_binary_in_dirs, find_binary_on_path, import_gws_credentials,
    import_gws_credentials_from_dir, missing_v0921_scopes, parse_google_client_secret_json,
    recommended_scopes_for_kind, refresh_access_token, run_google_doctor, ImportedGwsConfig,
    OAuth2Config, OAuthTokenResponse, ParsedOAuthClientJson, PkceChallenge,
};
pub use pulse::{
    dispatch_external_pulse_alert, is_within_quiet_hours, should_dispatch_external,
    ProactivePulseStore, PulseRuntimeState, PulseSecrets,
};
pub use security::{
    evaluate_action_policy, redact_google_secrets, sanitize_for_external_llm,
    wrap_untrusted_content, PolicyVerdict, SessionTaintTracker, DEFAULT_TAINT_TTL_SECS,
};
pub use session::{
    default_google_data_dir, GoogleAccountRecord, GoogleMultiAccountVault,
    GoogleSessionCredentials, GoogleSessionVault,
};
pub use wiz_rpc::{
    compute_sapisidhash, sha1_digest, sha1_hex, NlmCliBridge, WizRpcClient, WizRpcId,
    WizRpcPayload, WizRpcResponse,
};
pub use workspace::{DocsScratchpadStore, GoogleWorkspaceClient, GwsCliBridge};
