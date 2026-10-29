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
pub mod context_lean;
pub mod error;
pub mod fs;
pub mod logging;
pub mod panic;
pub mod pii_redactor;
pub mod sandbox;
pub mod subprocess_guard;
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

