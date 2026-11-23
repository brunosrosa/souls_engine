//! # Headless Agentic CLI & Evolutive Harness (`souls_core::harness`)
//!
//! Subprocess dispatch, streaming NDJSON telemetry, and transactional git restoration
//! for external agentic CLIs (`agy`, `claude`, `codex`), plus `v0.9.4` Evolutive Harness
//! primitives (`SurfaceSpec`, `CordisMicrokernel`, and `CatnapMonitor`).

pub mod agentic_cli;
pub mod catnap;
pub mod cordis;
pub mod surface;

pub use agentic_cli::{
    detect_installed_clis, AgenticCliCommand, AgenticCliInfo, AgenticCliKind,
    AgenticHarnessDispatcher, HarnessEvent, HarnessOutput,
};
pub use catnap::{
    compute_runmark_fingerprint, is_lightweight_zero_vram_slice, is_within_quiet_hours,
    CatnapMonitor, CATNAP_HYSTERESIS_BAND_C, CATNAP_LIGHT_SLICE_MAX_TEMP_C,
};
pub use cordis::{CordisMicrokernel, EffectUndoOp};
pub use surface::{
    compute_lf_sha256_hex, fold_aci_observation, is_non_idempotent_tool,
    normalize_posix_rel_path, scrub_candidate_environment, ConstitutionalPinVerifier,
    SCRUB_ENV_KEYS,
};
