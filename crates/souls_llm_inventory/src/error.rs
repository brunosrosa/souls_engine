//! Typed error handling for `souls_llm_inventory`.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum InventoryError {
    #[error("Hardware probe failure: {0}")]
    HardwareProbeError(String),

    #[error("Catalog I/O failure on '{0}': {1}")]
    CatalogIoError(String, String),

    #[error("Download error for '{0}': {1}")]
    DownloadError(String, String),

    #[error("Download not found: {0}")]
    DownloadNotFound(String),

    #[error("Download interrupted or cancelled: {0}")]
    DownloadCancelled(String),

    #[error("Checksum mismatch for '{0}': expected {1}, calculated {2}")]
    ChecksumMismatch(String, String, String),

    #[error("Hugging Face API request failed: {0}")]
    HfApiError(String),

    #[error("GGUF metadata parsing error on '{0}': {1}")]
    GgufParseError(String, String),

    #[error("Model not found in inventory: {0}")]
    ModelNotFound(String),
}
