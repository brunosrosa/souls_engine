//! Discovery and inspection subroutines for models on host storage.

pub mod hermes_link;
pub mod metadata;
pub mod scanner;
pub mod vault;

pub use hermes_link::{default_hermes_models_dir, sync_hermes_model_links, HermesSyncReport};
pub use metadata::{enrich_with_gguf_metadata, inspect_model_file};
pub use scanner::{default_model_directories, scan_local_models};
pub use vault::{resolve_vault_db_path, VaultManager};
