//! # Souls Core (`souls_core`)
//!
//! Bare-metal low-level infrastructure for the Souls Engine v7:
//! - ANSI escape terminal sanitization and structured async tracing.
//! - Adaptive token-aware smart reading (cl100k_base tiktoken) and auto-shrink context pruning.
//! - Windows 11 ReFS Block Cloning (Copy-on-Write) zero-copy NVMe preservation.
//! - Dev Drive path safety barriers.

pub mod ansi_filter;
pub mod context_lean;
pub mod error;
pub mod fs;
pub mod logging;

pub use ansi_filter::{ansi_density, strip_ansi, strip_ansi_escapes};
pub use context_lean::{
    count_tokens, count_tokens_async, detect_paradigm, extract_outline_signatures_polyglot,
    lightweight_cleanup, multi_read_concurrent, smart_read_file, smart_read_text,
    smart_read_text_async, FileCompaction, LanguageParadigm,
};
pub use error::CoreError;
pub use fs::{clone_file_refs, ensure_refs_directory, validate_refs_path};
pub use logging::{init_tracing, sanitize_terminal_buffer};
