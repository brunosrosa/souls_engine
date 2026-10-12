//! # storage/mod.rs — Subsistema de Armazenamento da Cloud Arena

pub mod migrations;
pub mod repository;

pub use migrations::run_migrations;
pub use repository::CloudArenaRepository;
