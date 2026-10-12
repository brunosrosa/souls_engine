//! # canonical/mod.rs — Subsistema de Resolução Canônica

pub mod resolver;
pub mod seed_aliases;

pub use resolver::CanonicalResolver;
pub use seed_aliases::{get_initial_seed_models, ModelSeed};
