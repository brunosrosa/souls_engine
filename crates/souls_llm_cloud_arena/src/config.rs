//! # config.rs — Governança de Configuração e Credenciais
//!
//! Resolve chaves de API com fallbacks graciosos e caminhos do Dev Drive ReFS.

use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct CloudArenaConfig {
    pub database_path: PathBuf,
    pub nous_api_key: Option<String>,
    pub openrouter_api_key: Option<String>,
    pub nous_base_url: String,
    pub openrouter_base_url: String,
    pub request_timeout_secs: u64,
}

impl Default for CloudArenaConfig {
    fn default() -> Self {
        Self::load()
    }
}

impl CloudArenaConfig {
    /// Carrega as configurações a partir do ambiente, .env e arquivos de fallback do Hermes.
    pub fn load() -> Self {
        let database_path = resolve_heuristic_vault_path();
        let mut nous_key = std::env::var("NOUS_API_KEY").ok().filter(|s| !s.trim().is_empty());
        let mut openrouter_key = std::env::var("OPENROUTER_API_KEY").ok().filter(|s| !s.trim().is_empty());

        // Fallback 1: Leitura do .env em Z:\souls_engine\.souls_data\.env
        if nous_key.is_none() || openrouter_key.is_none() {
            let env_path = PathBuf::from(r"Z:\souls_engine\.souls_data\.env");
            if env_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&env_path) {
                    for line in content.lines() {
                        let line = line.trim();
                        if line.starts_with('#') || line.is_empty() {
                            continue;
                        }
                        if let Some((k, v)) = line.split_once('=') {
                            let k = k.trim();
                            let v = v.trim().trim_matches('"').trim_matches('\'').to_string();
                            if k == "NOUS_API_KEY" && nous_key.is_none() {
                                nous_key = Some(v);
                            } else if k == "OPENROUTER_API_KEY" && openrouter_key.is_none() {
                                openrouter_key = Some(v);
                            }
                        }
                    }
                }
            }
        }

        // Fallback 2: Leitura do config.yaml do Hermes
        if nous_key.is_none() || openrouter_key.is_none() {
            if let Some(hermes_cfg_path) = find_hermes_config_file() {
                if let Ok(content) = std::fs::read_to_string(&hermes_cfg_path) {
                    for line in content.lines() {
                        let line = line.trim();
                        if (line.starts_with("nous_api_key:") || line.starts_with("nous_key:")) && nous_key.is_none() {
                            if let Some((_, v)) = line.split_once(':') {
                                let v = v.trim().trim_matches('"').trim_matches('\'').to_string();
                                if !v.is_empty() {
                                    nous_key = Some(v);
                                }
                            }
                        } else if (line.starts_with("openrouter_api_key:") || line.starts_with("openrouter_key:")) && openrouter_key.is_none() {
                            if let Some((_, v)) = line.split_once(':') {
                                let v = v.trim().trim_matches('"').trim_matches('\'').to_string();
                                if !v.is_empty() {
                                    openrouter_key = Some(v);
                                }
                            }
                        }
                    }
                }
            }
        }

        Self {
            database_path,
            nous_api_key: nous_key,
            openrouter_api_key: openrouter_key,
            nous_base_url: "https://inference-api.nousresearch.com/v1".to_string(),
            openrouter_base_url: "https://openrouter.ai/api/v1".to_string(),
            request_timeout_secs: 15,
        }
    }
}

/// Resolve o caminho absoluto canônico do banco Heuristic Vault em Dev Drive ReFS.
pub fn resolve_heuristic_vault_path() -> PathBuf {
    PathBuf::from(r"Z:\souls_engine\.souls_data\db\souls_heuristic_vault.db")
}

/// Tenta localizar o config.yaml do Hermes Agent no Windows NT.
fn find_hermes_config_file() -> Option<PathBuf> {
    let mut candidates = Vec::new();
    if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
        candidates.push(Path::new(&local_app_data).join("hermes").join("config.yaml"));
    }
    if let Ok(user_profile) = std::env::var("USERPROFILE") {
        candidates.push(Path::new(&user_profile).join(".hermes").join("config.yaml"));
    }

    candidates.into_iter().find(|p| p.exists())
}
