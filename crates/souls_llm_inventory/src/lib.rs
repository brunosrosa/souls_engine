//! # souls_llm_inventory (Souls Engine)
//!
//! Bare-metal silicon model hub, hardware profiling, and resilient download manager.
//! Governed strictly by RFC-007, ADR-004, and ADR-007.
//!
//! # Architecture & Capabilities
//! - **Hardware Profiling:** Dynamic and agnostic evaluation of GPU VRAM, System RAM, and AVX2 extensions.
//! - **Curated Hub & Registry:** Reference manifest of elite models across the 3 Tiers and 5 Cognitive Profiles.
//! - **Hugging Face Hub Client:** On-demand search and metadata extraction for any GGUF repository.
//! - **Resilient Downloader:** Sequential FIFO queue, HTTP Range requests for resume, and SHA-256 integrity verification.
//! - **Zero-Copy Catalog:** Mmap-backed local model inspection and directory lifecycle management.

pub mod catalog;
pub mod checksum;
pub mod downloader;
pub mod error;
pub mod hf_client;
pub mod profiler;
pub mod registry;

// Re-export canonical types and functions
pub use catalog::{
    delete_model_file, inspect_gguf_file, parse_model_params_from_name,
    resolve_canonical_models_dir, scan_local_models, LocalModelMeta, ModelFormat,
};
pub use checksum::{compute_file_sha256, verify_file_sha256, IncrementalSha256};
pub use downloader::{DownloadItem, DownloadManager, DownloadState};
pub use error::InventoryError;
pub use hf_client::{extract_quant_tag, HfModelFile, HfSearchResult, HuggingFaceClient};
pub use profiler::{
    detect_cpu_brand_string, estimate_kv_cache_mb, estimate_throughput_tps,
    estimate_variant_throughput_tps, evaluate_model_fit, probe_hardware, HardwareDiagnostic,
    HardwareFit,
};
pub use registry::{
    build_dynamic_hf_catalog_item, get_curated_catalog, get_curated_catalog_from_models,
    get_recommended_triad, get_recommended_triad_from_models, BenchmarkExpectation,
    HuggingFaceModelItem, QuantizationVariant, RecommendedTriad,
};
