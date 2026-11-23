//! # Souls Core (`souls_core`)
//!
//! Bare-metal low-level infrastructure for the Souls Engine v7:
//! - ANSI escape terminal sanitization and structured async tracing.
//! - Adaptive token-aware smart reading (cl100k_base tiktoken) and auto-shrink context pruning.
//! - Windows 11 ReFS Block Cloning (Copy-on-Write) zero-copy NVMe preservation.
//! - Dev Drive path safety barriers.
//! - RAII Subprocess lifecycle management (zombie process eradication).
//! - Linear PII redaction and reversible opaque tokens.
//! - Asynchronous and synchronous panic containment barriers (`catch_unwind`).

pub mod ansi_filter;
pub mod audio;
pub mod context_lean;
pub mod error;
pub mod fs;
pub mod logging;
pub mod panic;
pub mod pii_redactor;
pub mod research;
pub mod google_bridge;
pub mod harness;
pub mod sandbox;
pub mod subprocess_guard;
pub mod vision;
pub mod web;

pub use ansi_filter::{ansi_density, strip_ansi, strip_ansi_escapes};
pub use context_lean::{
    compress_lean_code, count_tokens, count_tokens_async, dedup_text, detect_paradigm,
    extract_outline_signatures_polyglot, fill_template_placeholders, get_global_dedup_cache,
    lightweight_cleanup, multi_read_concurrent, rehydrate_text, smart_read_file, smart_read_text,
    smart_read_text_async, CompactionResult, DedupCache, DedupResult, FileCompaction,
    LanguageParadigm,
};
pub use error::CoreError;
pub use fs::{clone_file_refs, ensure_refs_directory, validate_refs_path};
pub use logging::{
    init_tracing, init_tracing_with_mpsc, sanitize_terminal_buffer, LogCommand, MpscBatchLogConfig,
    MpscBatchLogWriter, MpscTracingLayer, RawLogEvent,
};
pub use panic::{run_safe_sync, run_safe_task};
pub use pii_redactor::PiiRedactor;
pub use sandbox::{
    run_sandboxed_command, run_sandboxed_shell, CommandSafetyFilter, OutputTruncationPolicy,
    SandboxedResult,
};
pub use subprocess_guard::{SubprocessConfig, SubprocessGuard, SubprocessState};
pub use web::{html_to_clean_markdown, parse_duckduckgo_html, WebSearchResult};
pub use research::{
    chunk_document, clean_html_to_markdown, hash_url, normalize_url, process_raw_html_passage,
    rank_chunks, tokenize_lexical, ChunkerOptions, FilteredPassage, FilteredPassageResult,
    HtmlCleanOptions, ResearchDedupRegistry, ScoredChunk, TextChunk,
};
pub use harness::{
    compute_lf_sha256_hex, compute_runmark_fingerprint, detect_installed_clis,
    fold_aci_observation, is_lightweight_zero_vram_slice, is_non_idempotent_tool,
    is_within_quiet_hours, normalize_posix_rel_path, scrub_candidate_environment,
    AgenticCliCommand, AgenticCliInfo, AgenticCliKind, AgenticHarnessDispatcher, CatnapMonitor,
    ConstitutionalPinVerifier, CordisMicrokernel, EffectUndoOp, HarnessEvent, HarnessOutput,
    CATNAP_HYSTERESIS_BAND_C, CATNAP_LIGHT_SLICE_MAX_TEMP_C, SCRUB_ENV_KEYS,
};
pub use google_bridge::{
    DualLayerWorkspace, GoogleSessionCredentials, GoogleSessionVault, JulesMigrationRouter,
    LegacyJulesCommand, WizRpcClient, WizRpcId, WizRpcPayload, WizRpcResponse,
};
pub use audio::{
    backchannel_pcm_wav, discover_hermes_audio_tools, evaluate_pcomp_turn_taking,
    generate_pcm_wav_buffer, handoff_chime_wav, normalize_stt_bcp47_language,
    parse_sensevoice_and_filter_hallucinations, probe_audio_engines,
    provision_local_faster_whisper_model, shape_prosodic_text_for_tts, split_prosodic_sentences,
    synthesize_speech_cascade, transcribe_audio_cascade, AudioEngineInventory,
    HermesAudioToolchainProbe, BARGE_IN_MIN_SPEECH_MS, NOISE_IGNORE_MAX_MS,
    SILERO_VAD_STANDBY_HYSTERESIS_MS, ZERO_AUDIO_VRAM_CONSUMPTION_MB,
};
pub use vision::{
    assess_blind_spot, capture_live_windows_ax_tree, capture_on_demand_snapshot,
    compute_ax_tree_diff, filter_ax_tree_by_intent, generate_som_marks, parse_cdp_ax_tree,
    standard_base64_encode, AXNode, AXNodeState, AXTreeCompressionMetrics, AXTreeSerializer,
    ZERO_VLM_VRAM_CONSUMPTION_MB,
};

