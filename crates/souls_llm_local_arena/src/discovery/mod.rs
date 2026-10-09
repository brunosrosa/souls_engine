//! Discovery and inspection subroutines for models on host storage.

pub mod metadata;
pub mod scanner;
pub mod vault;

pub use metadata::{enrich_with_gguf_metadata, inspect_model_file};
pub use scanner::{default_model_directories, scan_local_models};
pub use vault::{resolve_vault_db_path, VaultManager};
