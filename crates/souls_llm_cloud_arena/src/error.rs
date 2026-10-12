//! # error.rs — Tipos de Erro da Crate souls_llm_cloud_arena
//!
//! Governança de erros estruturados para ingestão de APIs de nuvem,
//! normalização de slugs, banco SQLite STRICT e cálculo de Pareto.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CloudArenaError {
    #[error("Erro de requisição HTTP: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Erro de banco de dados SQLite: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Erro de serialização/deserialização JSON: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Configuração ausente ou inválida: {0}")]
    Config(String),

    #[error("Modelo não encontrado: {0}")]
    ModelNotFound(String),

    #[error("Erro de parsing/normalização: {0}")]
    Parse(String),

    #[error("Falha durante o ciclo de sincronização: {0}")]
    SyncFailed(String),

    #[error("Erro interno de I/O: {0}")]
    Io(#[from] std::io::Error),
}
