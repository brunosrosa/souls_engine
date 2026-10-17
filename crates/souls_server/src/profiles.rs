//! # profiles.rs — Master Brain Profiles & Hermes Agent Synchronization
//!
//! Governs the 5 operational Master Brain profiles, dynamic Cloud Arena Pareto
//! integration, and atomic synchronization with Hermes Agent `config.yaml`.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::info;

use crate::state::AppState;
use souls_llm_cloud_arena::TaskProfileKind;

/// Canonical Master Brain profile identifier.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MasterBrainProfile {
    pub id: String,
    pub name: String,
    pub description: String,
    pub target_model: String,
    pub provider: String,
    pub target_type: String,
    pub is_active: bool,
}

/// Result returned after applying a profile or model switch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileApplyResult {
    pub status: String,
    pub profile_id: Option<String>,
    pub previous_model: String,
    pub applied_model: String,
    pub provider: String,
    pub backup_created: bool,
    pub backup_path: Option<String>,
    pub config_path: String,
}

/// Resolves the canonical path to Hermes Agent `config.yaml`.
/// Checks `SOULS_HERMES_CONFIG_PATH` first for hermetic testing,
/// then falls back to `%LOCALAPPDATA%\hermes\config.yaml`.
pub fn resolve_hermes_config_path() -> PathBuf {
    if let Ok(override_path) = std::env::var("SOULS_HERMES_CONFIG_PATH") {
        return PathBuf::from(override_path);
    }
    let local_appdata = std::env::var("LOCALAPPDATA")
        .unwrap_or_else(|_| "C:\\Users\\rosas\\AppData\\Local".to_string());
    PathBuf::from(local_appdata).join("hermes").join("config.yaml")
}

/// Reads the currently configured model and provider from Hermes `config.yaml`.
pub fn read_active_hermes_model(cfg_path: &Path) -> (String, String) {
    let mut model_name = "deepseek/deepseek-chat".to_string();
    let mut provider = "nous".to_string();

    if cfg_path.exists() {
        if let Ok(content) = fs::read_to_string(cfg_path) {
            let mut inside_top_model = false;
            for line in content.lines() {
                let trimmed = line.trim();
                if line.starts_with("model:") {
                    inside_top_model = true;
                    continue;
                }
                if inside_top_model {
                    if trimmed.starts_with("default:") {
                        let val = trimmed
                            .trim_start_matches("default:")
                            .trim()
                            .trim_matches('"')
                            .trim_matches('\'');
                        if !val.is_empty() {
                            model_name = val.to_string();
                        }
                    } else if trimmed.starts_with("provider:") {
                        let val = trimmed
                            .trim_start_matches("provider:")
                            .trim()
                            .trim_matches('"')
                            .trim_matches('\'');
                        if !val.is_empty() {
                            provider = val.to_string();
                        }
                    } else if !line.starts_with(' ') && !line.starts_with('\t') && !line.is_empty() {
                        inside_top_model = false;
                    }
                }
            }
        }
    }

    (model_name, provider)
}

/// Discovers and constructs the 5 canonical Master Brain profiles, dynamically
/// populated with current Pareto champions from `souls_llm_cloud_arena`.
pub async fn get_available_profiles(state: &AppState) -> Vec<MasterBrainProfile> {
    let cfg_path = resolve_hermes_config_path();
    let (active_model, _) = read_active_hermes_model(&cfg_path);

    // Fetch champions from Cloud Arena
    let reports = state.cloud_arena.get_all_champions(None).await.unwrap_or_default();

    // Helper closures to resolve champions by profile
    let find_champ = |kind: TaskProfileKind, leader: bool| -> Option<String> {
        reports.iter()
            .find(|r| r.profile == kind)
            .and_then(|r| {
                if leader {
                    r.quality_leader.as_ref().map(|c| c.canonical_id.clone())
                } else {
                    r.best_value.as_ref().map(|c| c.canonical_id.clone())
                }
            })
    };

    let balanced_model = find_champ(TaskProfileKind::Agentic, false)
        .or_else(|| find_champ(TaskProfileKind::Coding, false))
        .unwrap_or_else(|| "deepseek/deepseek-chat".to_string());

    let swe_model = find_champ(TaskProfileKind::Coding, true)
        .unwrap_or_else(|| "anthropic/claude-3-7-sonnet".to_string());

    let reasoning_model = find_champ(TaskProfileKind::Reasoning, true)
        .or_else(|| find_champ(TaskProfileKind::Reasoning, false))
        .unwrap_or_else(|| "deepseek/deepseek-reasoner".to_string());

    let fast_model = find_champ(TaskProfileKind::Fast, false)
        .unwrap_or_else(|| "google/gemini-2.0-flash-001".to_string());

    let local_model = "qwen3.5-coder-3b-instruct".to_string();

    vec![
        MasterBrainProfile {
            id: "frontier_balanced".to_string(),
            name: "Frontier Balanced (Best Value ROI)".to_string(),
            description: "Campeão geral de Pareto por custo-benefício para diálogo e orquestração diária equilibrada.".to_string(),
            target_model: balanced_model.clone(),
            provider: "nous".to_string(),
            target_type: "Cloud API".to_string(),
            is_active: active_model == balanced_model,
        },
        MasterBrainProfile {
            id: "deep_swe".to_string(),
            name: "Deep Software Engineering (Quality Leader)".to_string(),
            description: "Líder de qualidade em engenharia de software complexa, refatoração de AST e arquitetura de sistemas.".to_string(),
            target_model: swe_model.clone(),
            provider: "nous".to_string(),
            target_type: "Cloud API".to_string(),
            is_active: active_model == swe_model,
        },
        MasterBrainProfile {
            id: "deep_reasoning".to_string(),
            name: "Formal Logic & Deep Reasoning".to_string(),
            description: "Especialista em raciocínio matemático, provas formais e auditoria de segurança com thinking tokens expandidos.".to_string(),
            target_model: reasoning_model.clone(),
            provider: "nous".to_string(),
            target_type: "Cloud API".to_string(),
            is_active: active_model == reasoning_model,
        },
        MasterBrainProfile {
            id: "ultra_fast".to_string(),
            name: "Ultra Fast & Low Latency".to_string(),
            description: "Máxima velocidade de resposta e custo mínimo para triagem rápida de mensagens e tarefas curtas.".to_string(),
            target_model: fast_model.clone(),
            provider: "nous".to_string(),
            target_type: "Cloud API".to_string(),
            is_active: active_model == fast_model,
        },
        MasterBrainProfile {
            id: "local_sovereign".to_string(),
            name: "Local Bare-Metal Sovereign (Zero Cost / Privacy)".to_string(),
            description: "Execução 100% no silício bare-metal local (GPU RTX 2060m / CPU AVX2). Soberania absoluta de dados e custo zero.".to_string(),
            target_model: local_model.clone(),
            provider: "custom".to_string(),
            target_type: "Silício Bare-Metal".to_string(),
            is_active: active_model == local_model,
        },
    ]
}

/// Applies a target model and provider to the Hermes Agent `config.yaml` file atomically.
/// Creates a `.bak` safety backup and preserves all other sections and YAML comments.
pub fn apply_profile_to_hermes(
    profile_id: Option<&str>,
    target_model: &str,
    provider: Option<&str>,
    cfg_path_override: Option<&Path>,
) -> Result<ProfileApplyResult, String> {
    let cfg_path = match cfg_path_override {
        Some(p) => p.to_path_buf(),
        None => resolve_hermes_config_path(),
    };

    if !cfg_path.exists() {
        return Err(format!(
            "Hermes config.yaml not found at expected path: {}",
            cfg_path.display()
        ));
    }

    let (prev_model, prev_provider) = read_active_hermes_model(&cfg_path);
    let new_provider = provider.unwrap_or(&prev_provider);

    // Create safety backup (.bak)
    let backup_path = cfg_path.with_extension("yaml.bak");
    let mut backup_created = false;
    if fs::copy(&cfg_path, &backup_path).is_ok() {
        backup_created = true;
    }

    let content = fs::read_to_string(&cfg_path)
        .map_err(|e| format!("Failed to read hermes config: {e}"))?;

    let mut inside_top_model = false;
    let mut default_replaced = false;
    let mut provider_replaced = false;
    let mut new_lines = Vec::new();

    for line in content.lines() {
        if line.starts_with("model:") {
            inside_top_model = true;
            new_lines.push(line.to_string());
            continue;
        }

        if inside_top_model {
            if line.starts_with("  default:") {
                new_lines.push(format!("  default: \"{}\"", target_model));
                default_replaced = true;
                continue;
            } else if line.starts_with("  provider:") {
                new_lines.push(format!("  provider: \"{}\"", new_provider));
                provider_replaced = true;
                continue;
            } else if !line.starts_with(' ') && !line.is_empty() {
                // Leaving model block: if provider wasn't in the original block, inject it
                if !provider_replaced && provider.is_some() {
                    new_lines.push(format!("  provider: \"{}\"", new_provider));
                    provider_replaced = true;
                }
                inside_top_model = false;
            }
        }

        new_lines.push(line.to_string());
    }

    // In case model: block was at the very end of file
    if inside_top_model && !provider_replaced && provider.is_some() {
        new_lines.push(format!("  provider: \"{}\"", new_provider));
    }

    if !default_replaced {
        return Err("Top-level model.default key was not found in hermes config.yaml".to_string());
    }

    fs::write(&cfg_path, new_lines.join("\n"))
        .map_err(|e| format!("Failed to write updated config.yaml: {e}"))?;

    info!(
        previous_model = %prev_model,
        applied_model = %target_model,
        provider = %new_provider,
        config_path = %cfg_path.display(),
        "Hermes Master Brain profile applied atomically"
    );

    Ok(ProfileApplyResult {
        status: "success".to_string(),
        profile_id: profile_id.map(|s| s.to_string()),
        previous_model: prev_model,
        applied_model: target_model.to_string(),
        provider: new_provider.to_string(),
        backup_created,
        backup_path: if backup_created {
            Some(backup_path.to_string_lossy().to_string())
        } else {
            None
        },
        config_path: cfg_path.to_string_lossy().to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_profile_atomic_with_backup() {
        let dir = std::env::temp_dir().join(format!(
            "souls_profile_test_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = fs::create_dir_all(&dir);
        let cfg_file = dir.join("config.yaml");

        let initial_yaml = r#"mcp_servers:
  souls_engine:
    url: "http://127.0.0.1:9123/mcp"
    transport: "http"

model:
  default: "deepseek/deepseek-chat"
  provider: "nous"

terminal:
  shell: "powershell"
"#;
        fs::write(&cfg_file, initial_yaml).unwrap();

        let (initial_model, initial_provider) = read_active_hermes_model(&cfg_file);
        assert_eq!(initial_model, "deepseek/deepseek-chat");
        assert_eq!(initial_provider, "nous");

        // Apply new profile
        let result = apply_profile_to_hermes(
            Some("deep_swe"),
            "anthropic/claude-3-7-sonnet",
            Some("openrouter"),
            Some(&cfg_file),
        )
        .unwrap();

        assert_eq!(result.status, "success");
        assert_eq!(result.previous_model, "deepseek/deepseek-chat");
        assert_eq!(result.applied_model, "anthropic/claude-3-7-sonnet");
        assert_eq!(result.provider, "openrouter");
        assert!(result.backup_created);

        // Verify backup content
        let backup_file = cfg_file.with_extension("yaml.bak");
        assert!(backup_file.exists());
        let backup_content = fs::read_to_string(&backup_file).unwrap();
        assert!(backup_content.contains(r#"default: "deepseek/deepseek-chat""#));

        // Verify updated content
        let (updated_model, updated_provider) = read_active_hermes_model(&cfg_file);
        assert_eq!(updated_model, "anthropic/claude-3-7-sonnet");
        assert_eq!(updated_provider, "openrouter");

        // Ensure other blocks were untouched
        let updated_content = fs::read_to_string(&cfg_file).unwrap();
        assert!(updated_content.contains("mcp_servers:"));
        assert!(updated_content.contains("terminal:"));

        let _ = fs::remove_dir_all(&dir);
    }
}
