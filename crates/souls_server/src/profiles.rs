//! # profiles.rs — Master Brain Profiles & Hermes Agent Synchronization
//!
//! Governs the 5 operational Master Brain profiles, dynamic Cloud Arena Pareto
//! integration, and atomic synchronization with Hermes Agent `config.yaml`.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::info;

use crate::state::AppState;
use souls_llm_cloud_arena::TaskProfileKind;
use souls_llm_inventory::{
    get_recommended_triad_from_models, probe_hardware, resolve_canonical_models_dir,
    scan_local_models,
};

/// Canonical Master Brain / Worker Profile representation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MasterBrainProfile {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(default = "default_role")]
    pub role: String, // "orchestrator" (Master Brain) ou "subagent"
    #[serde(default = "default_arena_category")]
    pub arena_category: String, // "agentic", "coding", "reasoning", "fast", "design", "local"
    pub target_model: String,
    pub provider: String,
    pub target_type: String,
    pub is_active: bool,
    #[serde(default)]
    pub is_builtin: bool,
    #[serde(default)]
    pub triad: Option<ProfileTriadSummary>,
}

fn default_role() -> String {
    "subagent".to_string()
}

fn default_arena_category() -> String {
    "coding".to_string()
}

/// Tríade de modelos da Fronteira de Pareto associada a um perfil.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProfileTriadSummary {
    pub quality_leader: Option<TriadModelItem>,
    pub best_value: Option<TriadModelItem>,
    pub ultra_low_cost: Option<TriadModelItem>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub leader: Option<TriadModelItem>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub best: Option<TriadModelItem>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub low: Option<TriadModelItem>,
}

impl ProfileTriadSummary {
    pub fn from_items(
        quality_leader: Option<TriadModelItem>,
        best_value: Option<TriadModelItem>,
        ultra_low_cost: Option<TriadModelItem>,
    ) -> Self {
        Self {
            leader: quality_leader.clone(),
            best: best_value.clone(),
            low: ultra_low_cost.clone(),
            quality_leader,
            best_value,
            ultra_low_cost,
        }
    }
}

/// Detalhes de um modelo integrante da Tríade.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TriadModelItem {
    pub canonical_id: String,
    pub provider: String,
    pub composite_score: f64,
    pub blended_cost: f64,
    pub roi_index: f64,
    pub desc: String,
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
    #[serde(default)]
    pub base_url: Option<String>,
    #[serde(default)]
    pub api_mode: String,
    #[serde(default)]
    pub profiles_updated: Vec<String>,
    #[serde(default)]
    pub unpinned_sessions: usize,
    #[serde(default)]
    pub favorites_synced: bool,
    #[serde(default)]
    pub warnings: Vec<String>,
}

/// Resolves the canonical path to Hermes Agent `config.yaml`.
/// Checks `SOULS_HERMES_CONFIG_PATH` first for hermetic testing,
/// then falls back to `%LOCALAPPDATA%\hermes\config.yaml`.
pub fn resolve_hermes_config_path() -> PathBuf {
    if let Ok(override_path) = std::env::var("SOULS_HERMES_CONFIG_PATH") {
        return PathBuf::from(override_path);
    }
    let local_appdata = std::env::var("LOCALAPPDATA")
        .or_else(|_| std::env::var("USERPROFILE").map(|p| format!(r"{}\AppData\Local", p)))
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| "C:\\Users\\Default".to_string());
            format!(r"{}\AppData\Local", home)
        });
    PathBuf::from(local_appdata)
        .join("hermes")
        .join("config.yaml")
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
                    } else if !line.starts_with(' ') && !line.starts_with('\t') && !line.is_empty()
                    {
                        inside_top_model = false;
                    }
                }
            }
        }
    }

    (model_name, provider)
}

/// Resolves the canonical Tríade of Pareto models for a given category.
pub fn resolve_triad_for_category(
    category: &str,
    reports: &[souls_llm_cloud_arena::ProfileChampionReport],
) -> ProfileTriadSummary {
    let cat_lower = category.to_lowercase();

    // Se for local bare-metal
    if cat_lower == "local" || cat_lower == "local_sovereign" {
        let diag = probe_hardware();
        let root = resolve_canonical_models_dir();
        let installed_models = scan_local_models(&root, &diag).unwrap_or_default();
        let triad = get_recommended_triad_from_models(&diag, &installed_models);

        let hw_target = if diag.has_dedicated_gpu && !diag.gpu_name.is_empty() {
            format!("dGPU {}", diag.gpu_name)
        } else if diag.has_avx2 {
            "CPU Host (AVX2)".to_string()
        } else {
            "CPU Host".to_string()
        };

        let map_item = |item: &souls_llm_inventory::HuggingFaceModelItem, default_score: f64, role_desc: &str| -> TriadModelItem {
            let canonical_id = item.local_file_path.clone().unwrap_or_else(|| {
                item.variants
                    .first()
                    .map(|v| v.file_name.clone())
                    .unwrap_or_else(|| item.model_name.clone())
            });

            let score = item
                .benchmark_scores
                .coding_score
                .or(item.benchmark_scores.mmlu_score)
                .or(item.benchmark_scores.ifeval_score)
                .map(|s| s as f64)
                .unwrap_or(default_score);

            let status_prefix = if item.is_installed {
                "Instalado localmente no silício"
            } else {
                "Recomendado para o silício"
            };

            let desc = format!("{} ({}). {}. Custo $0.00.", status_prefix, hw_target, role_desc);

            TriadModelItem {
                canonical_id,
                provider: "local_silicon".to_string(),
                composite_score: score,
                blended_cost: 0.0,
                roi_index: 9999.0,
                desc,
            }
        };

        let ql = map_item(
            &triad.tier1_coder,
            88.0,
            "Líder de qualidade para tarefas complexas e código",
        );
        let bv = map_item(
            &triad.tier05_socratico,
            86.2,
            "Pensamento deliberativo e raciocínio analítico",
        );
        let ulc = map_item(
            &triad.tier0_velocista,
            75.0,
            "Ultra-leve para triagem e ferramentas ágeis",
        );

        return ProfileTriadSummary::from_items(
            Some(ql),
            Some(bv),
            Some(ulc),
        );
    }

    let kind = match cat_lower.as_str() {
        "master" | "balanced" | "frontier_balanced" => TaskProfileKind::Executive,
        other => TaskProfileKind::from_str_lossy(other).unwrap_or(TaskProfileKind::Agentic),
    };

    if let Some(rep) = reports.iter().find(|r| r.profile == kind) {
        let ql = rep.quality_leader.as_ref().map(|c| TriadModelItem {
            canonical_id: c.canonical_id.clone(),
            provider: c.provider.clone(),
            composite_score: c.composite_score,
            blended_cost: c.blended_cost,
            roi_index: c.roi_index,
            desc: "Líder de qualidade no ranking da Cloud Arena.".to_string(),
        });
        let bv = rep.best_value.as_ref().map(|c| TriadModelItem {
            canonical_id: c.canonical_id.clone(),
            provider: c.provider.clone(),
            composite_score: c.composite_score,
            blended_cost: c.blended_cost,
            roi_index: c.roi_index,
            desc: "Ponto ótimo da curva de Pareto com máximo ROI.".to_string(),
        });
        let ulc = rep.ultra_low_cost.as_ref().map(|c| TriadModelItem {
            canonical_id: c.canonical_id.clone(),
            provider: c.provider.clone(),
            composite_score: c.composite_score,
            blended_cost: c.blended_cost,
            roi_index: c.roi_index,
            desc: "Tarifa ultrabaixa e resposta rápida.".to_string(),
        });
        return ProfileTriadSummary::from_items(ql, bv, ulc);
    }

    // Fallbacks empíricos calibrados caso a Cloud Arena ainda esteja inicializando
    match kind {
        TaskProfileKind::Coding => ProfileTriadSummary::from_items(
            Some(TriadModelItem {
                canonical_id: "anthropic/claude-3-7-sonnet".to_string(),
                provider: "nous".to_string(),
                composite_score: 96.0,
                blended_cost: 3.00,
                roi_index: 32.0,
                desc: "Líder de SWE-bench Verified e refatoração arquitetural.".to_string(),
            }),
            Some(TriadModelItem {
                canonical_id: "qwen/qwen-2.5-coder-32b-instruct".to_string(),
                provider: "openrouter".to_string(),
                composite_score: 90.8,
                blended_cost: 0.20,
                roi_index: 454.0,
                desc: "Vanguarda em Rust, Python e testes com excelente custo-benefício."
                    .to_string(),
            }),
            Some(TriadModelItem {
                canonical_id: "deepseek/deepseek-chat".to_string(),
                provider: "nous".to_string(),
                composite_score: 89.5,
                blended_cost: 0.14,
                roi_index: 639.3,
                desc: "Desempenho sólido em engenharia por apenas $0.14 por milhão.".to_string(),
            }),
        ),
        TaskProfileKind::Reasoning => ProfileTriadSummary::from_items(
            Some(TriadModelItem {
                canonical_id: "openai/o3-mini-high".to_string(),
                provider: "nous".to_string(),
                composite_score: 97.4,
                blended_cost: 4.40,
                roi_index: 22.1,
                desc: "Máximo raciocínio formal, provas matemáticas e auditoria de segurança."
                    .to_string(),
            }),
            Some(TriadModelItem {
                canonical_id: "deepseek/deepseek-r1".to_string(),
                provider: "nous".to_string(),
                composite_score: 95.8,
                blended_cost: 0.55,
                roi_index: 174.2,
                desc: "Raciocínio aberto de fronteira por uma fração do preço comercial."
                    .to_string(),
            }),
            Some(TriadModelItem {
                canonical_id: "qwen/qwq-32b-preview".to_string(),
                provider: "openrouter".to_string(),
                composite_score: 92.1,
                blended_cost: 0.18,
                roi_index: 511.7,
                desc: "Cadeia de pensamento analítica acessível e de baixo consumo.".to_string(),
            }),
        ),
        TaskProfileKind::Fast => ProfileTriadSummary::from_items(
            Some(TriadModelItem {
                canonical_id: "anthropic/claude-3-5-haiku".to_string(),
                provider: "nous".to_string(),
                composite_score: 86.5,
                blended_cost: 0.80,
                roi_index: 108.1,
                desc: "Alta precisão em comandos rápidos com baixíssima latência.".to_string(),
            }),
            Some(TriadModelItem {
                canonical_id: "google/gemini-2.0-flash".to_string(),
                provider: "nous".to_string(),
                composite_score: 88.0,
                blended_cost: 0.10,
                roi_index: 880.0,
                desc: "Velocidade impressionante de geração e suporte a contexto longo."
                    .to_string(),
            }),
            Some(TriadModelItem {
                canonical_id: "meta-llama/llama-3.2-3b-instruct".to_string(),
                provider: "openrouter".to_string(),
                composite_score: 78.0,
                blended_cost: 0.06,
                roi_index: 1300.0,
                desc: "Triagem imediata e comandos pontuais por centavos de dólar.".to_string(),
            }),
        ),
        _ => ProfileTriadSummary::from_items(
            Some(TriadModelItem {
                canonical_id: "anthropic/claude-3-7-sonnet".to_string(),
                provider: "nous".to_string(),
                composite_score: 94.2,
                blended_cost: 3.00,
                roi_index: 31.4,
                desc: "Líder técnico absoluto em compreensão geral e orquestração.".to_string(),
            }),
            Some(TriadModelItem {
                canonical_id: "deepseek/deepseek-chat".to_string(),
                provider: "nous".to_string(),
                composite_score: 91.5,
                blended_cost: 0.14,
                roi_index: 653.6,
                desc: "97% do desempenho do Claude por 1/20 do valor ($0.14/1M).".to_string(),
            }),
            Some(TriadModelItem {
                canonical_id: "google/gemini-2.0-flash".to_string(),
                provider: "nous".to_string(),
                composite_score: 88.0,
                blended_cost: 0.10,
                roi_index: 880.0,
                desc: "Ultra-rápido com latência sub-segundo e custo irrisório.".to_string(),
            }),
        ),
    }
}

/// Parses dynamic profiles declared in `[profiles.<id>]` sections of a TOML string.
pub fn parse_profiles_from_toml(content: &str) -> Vec<MasterBrainProfile> {
    let mut profiles = Vec::new();
    let mut in_profile_section = false;
    let mut current_id = String::new();
    let mut current_name = String::new();
    let mut current_desc = String::new();
    let mut current_role = "subagent".to_string();
    let mut current_cat = "coding".to_string();
    let mut current_model = String::new();
    let mut current_provider = "nous".to_string();
    let mut current_target_type = "Cloud API".to_string();

    let mut flush = |id: &str,
                     name: &str,
                     desc: &str,
                     role: &str,
                     cat: &str,
                     model: &str,
                     provider: &str,
                     target_type: &str| {
        if !id.is_empty() {
            profiles.push(MasterBrainProfile {
                id: id.to_string(),
                name: if name.is_empty() {
                    id.to_string()
                } else {
                    name.to_string()
                },
                description: desc.to_string(),
                role: role.to_string(),
                arena_category: cat.to_string(),
                target_model: if model.is_empty() {
                    "deepseek/deepseek-chat".to_string()
                } else {
                    model.to_string()
                },
                provider: provider.to_string(),
                target_type: target_type.to_string(),
                is_active: false,
                is_builtin: false,
                triad: None,
            });
        }
    };

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            let section = trimmed[1..trimmed.len() - 1].trim();
            if section.starts_with("profiles.") {
                if in_profile_section {
                    flush(
                        &current_id,
                        &current_name,
                        &current_desc,
                        &current_role,
                        &current_cat,
                        &current_model,
                        &current_provider,
                        &current_target_type,
                    );
                }
                in_profile_section = true;
                current_id = section.trim_start_matches("profiles.").trim().to_string();
                current_name = String::new();
                current_desc = String::new();
                current_role = if current_id == "orchestrator" || current_id == "frontier_balanced"
                {
                    "orchestrator".to_string()
                } else {
                    "subagent".to_string()
                };
                current_cat = "coding".to_string();
                current_model = String::new();
                current_provider = "nous".to_string();
                current_target_type = "Cloud API".to_string();
                continue;
            } else if in_profile_section {
                flush(
                    &current_id,
                    &current_name,
                    &current_desc,
                    &current_role,
                    &current_cat,
                    &current_model,
                    &current_provider,
                    &current_target_type,
                );
                in_profile_section = false;
                current_id = String::new();
            }
        } else if in_profile_section {
            if let Some((k, v)) = trimmed.split_once('=') {
                let key = k.trim();
                let val = v.trim().trim_matches('"').trim_matches('\'');
                match key {
                    "name" => current_name = val.to_string(),
                    "description" => current_desc = val.to_string(),
                    "role" => current_role = val.to_string(),
                    "arena_category" => current_cat = val.to_string(),
                    "target_model" | "model" => current_model = val.to_string(),
                    "provider" => current_provider = val.to_string(),
                    "target_type" => current_target_type = val.to_string(),
                    _ => {}
                }
            }
        }
    }

    if in_profile_section {
        flush(
            &current_id,
            &current_name,
            &current_desc,
            &current_role,
            &current_cat,
            &current_model,
            &current_provider,
            &current_target_type,
        );
    }

    profiles
}

/// Discovers, merges, and constructs dynamic Master Brain & Worker profiles.
/// Reads from `souls_config.toml`, ensures the invariant `orchestrator` exists,
/// and populates each profile with its Pareto champions from `souls_llm_cloud_arena`.
pub async fn get_available_profiles(state: &AppState) -> Vec<MasterBrainProfile> {
    let cfg_path = resolve_hermes_config_path();
    let (active_model, active_provider) = read_active_hermes_model(&cfg_path);

    // Fetch champions from Cloud Arena
    let reports = state
        .cloud_arena
        .get_all_champions(None)
        .await
        .unwrap_or_default();

    // 1. Tenta carregar perfis dinâmicos de souls_config.toml
    let toml_path = PathBuf::from(r"Z:\souls_engine\.souls_data\config\souls_config.toml");
    let mut configured_profiles = if toml_path.exists() {
        if let Ok(c) = fs::read_to_string(&toml_path) {
            parse_profiles_from_toml(&c)
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    // 2. Se a lista estiver vazia ou faltar o perfil do Orquestrador, injeta perfis base
    let has_orchestrator = configured_profiles
        .iter()
        .any(|p| p.id == "orchestrator" || p.id == "frontier_balanced");

    if !has_orchestrator {
        configured_profiles.insert(0, MasterBrainProfile {
            id: "frontier_balanced".to_string(),
            name: "Frontier Balanced (Master Brain)".to_string(),
            description: "Orquestrador mestre e diálogo diário com o desenvolvedor. Ponto ótimo da curva de Pareto.".to_string(),
            role: "orchestrator".to_string(),
            arena_category: "master".to_string(),
            target_model: "deepseek/deepseek-chat".to_string(),
            provider: "nous".to_string(),
            target_type: "Cloud API".to_string(),
            is_active: false,
            is_builtin: true,
            triad: None,
        });
    }

    // Injeta perfis canônicos ausentes (mantém os 9 built-in mesmo após customizações em souls_config.toml)
    let builtin_workers = [
        (
            "deep_swe",
            "Deep SWE 🚀",
            "Engenharia de software complexa, refatoração de AST e arquitetura de código bare-metal.",
            "coding",
            "anthropic/claude-3-7-sonnet",
            "nous",
            "Cloud API",
        ),
        (
            "deep_reasoning",
            "Deep Math 🧠",
            "Provas formais, raciocínio analítico, lógica matemática e auditoria criptográfica.",
            "reasoning",
            "openai/o3-mini-high",
            "nous",
            "Cloud API",
        ),
        (
            "ultra_fast",
            "Ultra Fast ⚡",
            "Triagem imediata, subagentes ágeis e ferramentas de baixa latência a centavos de custo.",
            "fast",
            "google/gemini-2.0-flash-001",
            "nous",
            "Cloud API",
        ),
        (
            "ux_designer",
            "UX & Frontend 🎨",
            "Geração de componentes de interface, design responsivo, protótipos e estilização CSS.",
            "design",
            "anthropic/claude-3-7-sonnet",
            "nous",
            "Cloud API",
        ),
        (
            "web_researcher",
            "Web Intelligence 🔎",
            "Pesquisa autônoma, síntese de fontes web, raspagem limpa na RAM e verificação factual.",
            "research",
            "google/gemini-2.0-flash-001",
            "nous",
            "Cloud API",
        ),
        (
            "memory_curator",
            "Memory & Graph 💾",
            "Curadoria epistêmica L2/L3, extração de entidades no LadybugDB e síntese do cofre Obsidian.",
            "curation",
            "deepseek/deepseek-chat",
            "nous",
            "Cloud API",
        ),
        (
            "security_auditor",
            "Security & Guardrails 🛡️",
            "Auditoria de segurança, sandbox NT no ReFS, higienização PII e blindagem de execução.",
            "security",
            "anthropic/claude-3-7-sonnet",
            "nous",
            "Cloud API",
        ),
        (
            "local_sovereign",
            "Local Sovereign 💻",
            "100% no silício físico local (GPU/CPU AVX2). Soberania absoluta e custo zero ($0.00).",
            "local",
            "qwen3.5-coder-3b-instruct",
            "custom",
            "Silício Bare-Metal",
        ),
    ];

    for (id, name, desc, cat, model, prov, ttype) in builtin_workers {
        if !configured_profiles.iter().any(|p| p.id == id) {
            configured_profiles.push(MasterBrainProfile {
                id: id.to_string(),
                name: name.to_string(),
                description: desc.to_string(),
                role: "subagent".to_string(),
                arena_category: cat.to_string(),
                target_model: model.to_string(),
                provider: prov.to_string(),
                target_type: ttype.to_string(),
                is_active: false,
                is_builtin: true,
                triad: None,
            });
        }
    }

    // 3. Popula a Tríade da Cloud Arena e define o estado ativo
    let matched_id = configured_profiles
        .iter()
        .find(|p| p.target_model == active_model)
        .map(|p| p.id.clone());

    for prof in &mut configured_profiles {
        prof.triad = Some(resolve_triad_for_category(&prof.arena_category, &reports));

        if let Some(ref m_id) = matched_id {
            prof.is_active = prof.id == *m_id;
        } else if prof.role == "orchestrator" {
            prof.target_model = active_model.clone();
            prof.provider = active_provider.clone();
            prof.is_active = true;
        } else {
            prof.is_active = false;
        }
    }

    configured_profiles
}

/// Atomically persists a new or updated profile to `souls_config.toml`.
pub fn save_or_update_profile(
    profile_id: &str,
    name: &str,
    description: &str,
    role: &str,
    arena_category: &str,
    target_model: &str,
    provider: &str,
    cfg_path_override: Option<&Path>,
) -> Result<(), String> {
    let cfg_path = cfg_path_override
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"Z:\souls_engine\.souls_data\config\souls_config.toml"));

    if let Some(parent) = cfg_path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    let content = if cfg_path.exists() {
        fs::read_to_string(&cfg_path).unwrap_or_default()
    } else {
        String::new()
    };

    // Remove bloco existente [profiles.<profile_id>] se já houver
    let header = format!("[profiles.{}]", profile_id);
    let mut new_lines = Vec::new();
    let mut skip = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == header {
            skip = true;
            continue;
        }
        if skip && trimmed.starts_with('[') && trimmed.ends_with(']') {
            skip = false;
        }
        if !skip {
            new_lines.push(line.to_string());
        }
    }

    // Adiciona o novo bloco
    new_lines.push(String::new());
    new_lines.push(format!("[profiles.{}]", profile_id));
    new_lines.push(format!("name = \"{}\"", name));
    new_lines.push(format!("description = \"{}\"", description));
    new_lines.push(format!("role = \"{}\"", role));
    new_lines.push(format!("arena_category = \"{}\"", arena_category));
    new_lines.push(format!("target_model = \"{}\"", target_model));
    new_lines.push(format!("provider = \"{}\"", provider));

    let updated = new_lines.join("\n");

    // Cria backup de segurança .bak
    if cfg_path.exists() {
        let bak = cfg_path.with_extension("toml.bak");
        let _ = fs::copy(&cfg_path, &bak);
    }

    fs::write(&cfg_path, updated)
        .map_err(|e| format!("Falha ao gravar perfil no souls_config.toml: {e}"))?;

    info!(profile_id = %profile_id, name = %name, "Perfil gravado com sucesso");
    Ok(())
}

/// Removes a custom profile from `souls_config.toml`. Protected against deleting the orchestrator.
pub fn delete_profile(profile_id: &str, cfg_path_override: Option<&Path>) -> Result<bool, String> {
    if profile_id == "orchestrator" || profile_id == "frontier_balanced" {
        return Err("O perfil do Orquestrador Mestre não pode ser removido.".to_string());
    }

    let cfg_path = cfg_path_override
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"Z:\souls_engine\.souls_data\config\souls_config.toml"));

    if !cfg_path.exists() {
        return Ok(false);
    }

    let content = fs::read_to_string(&cfg_path)
        .map_err(|e| format!("Falha ao ler souls_config.toml: {e}"))?;

    let header = format!("[profiles.{}]", profile_id);
    let mut new_lines = Vec::new();
    let mut skip = false;
    let mut found = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == header {
            skip = true;
            found = true;
            continue;
        }
        if skip && trimmed.starts_with('[') && trimmed.ends_with(']') {
            skip = false;
        }
        if !skip {
            new_lines.push(line.to_string());
        }
    }

    if found {
        let bak = cfg_path.with_extension("toml.bak");
        let _ = fs::copy(&cfg_path, &bak);
        fs::write(&cfg_path, new_lines.join("\n"))
            .map_err(|e| format!("Falha ao regravar souls_config.toml: {e}"))?;
        info!(profile_id = %profile_id, "Perfil removido com sucesso");
    }

    Ok(found)
}

/// Canonical Hermes Agent provider identity and transport route (`hermes_cli/providers.py`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CanonicalHermesRoute {
    pub provider: String,
    pub base_url: Option<String>,
    pub api_mode: String,
}

/// Official provider slugs accepted natively by Hermes Agent (`hermes_cli/providers.py` + `auth.py`).
pub const OFFICIAL_HERMES_PROVIDERS: &[&str] = &[
    "nous",
    "openrouter",
    "copilot",
    "custom",
    "deepseek",
    "anthropic",
    "openai-api",
    "openai-codex",
    "gemini",
    "zai",
    "xai",
    "xai-oauth",
    "alibaba",
    "alibaba-coding-plan",
    "kimi-for-coding",
    "xiaomi",
    "minimax",
    "minimax-oauth",
    "minimax-cn",
    "stepfun",
    "fireworks",
    "huggingface",
    "novita",
    "nvidia",
    "upstage",
    "arcee",
    "gmi",
    "actual",
    "nebius-token-factory",
    "ollama-cloud",
    "lmstudio",
    "azure-foundry",
    "bedrock",
    "vertex",
    "opencode",
    "opencode-go",
    "kilo",
    "vercel",
    "tencent-tokenhub",
    "tencent-tokenplan",
    "commandcode",
    "moa",
];

/// Normalizes any internal Souls Engine / Cloud Arena provider alias into an official Hermes Agent
/// provider slug, canonical inference `base_url`, and expected `api_mode`.
pub fn normalize_hermes_provider_and_route(
    raw_provider: &str,
    target_model: &str,
) -> CanonicalHermesRoute {
    let trimmed = raw_provider.trim().to_ascii_lowercase();
    let effective = if trimmed.is_empty() || trimmed == "auto" {
        let ml = target_model.to_ascii_lowercase();
        if ml.ends_with(".gguf") || ml.contains("q4_k_") || (!target_model.contains('/') && ml.contains("qwen")) {
            "custom"
        } else {
            "nous"
        }
    } else {
        trimmed.as_str()
    };

    let (provider, base_url, api_mode) = match effective {
        "nous_portal" | "nous_portal_batch" | "nous" => (
            "nous",
            Some("https://inference-api.nousresearch.com/v1"),
            "chat_completions",
        ),
        "openrouter_market" | "openrouter" => (
            "openrouter",
            Some("https://openrouter.ai/api/v1"),
            "chat_completions",
        ),
        "local_silicon" | "llamacpp" | "llama.cpp" | "llama-cpp" | "ollama" | "vllm"
        | "local" | "custom" => (
            "custom",
            Some("http://127.0.0.1:18434/v1"),
            "chat_completions",
        ),
        "lmstudio" | "lm-studio" | "lm_studio" => (
            "lmstudio",
            Some("http://127.0.0.1:1234/v1"),
            "chat_completions",
        ),
        "google" | "gemini" | "google-gemini" | "google-ai-studio" => (
            "gemini",
            Some("https://generativelanguage.googleapis.com/v1beta"),
            "chat_completions",
        ),
        "openai" | "openai-api" => (
            "openai-api",
            Some("https://api.openai.com/v1"),
            "codex_responses",
        ),
        "codex" | "openai-codex" | "chatgpt" => (
            "openai-codex",
            Some("https://chatgpt.com/backend-api/codex"),
            "codex_responses",
        ),
        "anthropic" | "claude" => (
            "anthropic",
            Some("https://api.anthropic.com"),
            "anthropic_messages",
        ),
        "deepseek" | "deep-seek" => (
            "deepseek",
            Some("https://api.deepseek.com/v1"),
            "chat_completions",
        ),
        "zai" | "z-ai" | "z.ai" | "glm" | "zhipu" => (
            "zai",
            Some("https://api.z.ai/api/paas/v4"),
            "chat_completions",
        ),
        "xai" | "x-ai" | "x.ai" | "grok" => (
            "xai",
            Some("https://api.x.ai/v1"),
            "codex_responses",
        ),
        "alibaba" | "qwen" | "dashscope" => (
            "alibaba",
            Some("https://dashscope-intl.aliyuncs.com/compatible-mode/v1"),
            "chat_completions",
        ),
        "kimi-for-coding" | "moonshot" | "kimi" | "kimi-coding" => (
            "kimi-for-coding",
            None,
            "chat_completions",
        ),
        "xiaomi" | "mimo" => (
            "xiaomi",
            Some("https://api.xiaomimimo.com/v1"),
            "chat_completions",
        ),
        "minimax" => (
            "minimax",
            Some("https://api.minimax.io/anthropic"),
            "anthropic_messages",
        ),
        "stepfun" | "step" => (
            "stepfun",
            Some("https://api.stepfun.ai/step_plan/v1"),
            "chat_completions",
        ),
        "fireworks" | "fireworks-ai" => (
            "fireworks",
            Some("https://api.fireworks.ai/inference/v1"),
            "chat_completions",
        ),
        "huggingface" | "hf" => (
            "huggingface",
            Some("https://router.huggingface.co/v1"),
            "chat_completions",
        ),
        "novita" | "novita-ai" => ("novita", None, "chat_completions"),
        "nvidia" | "nim" => (
            "nvidia",
            Some("https://integrate.api.nvidia.com/v1"),
            "chat_completions",
        ),
        "upstage" | "solar" => (
            "upstage",
            Some("https://api.upstage.ai/v1"),
            "chat_completions",
        ),
        "copilot" => ("copilot", None, "chat_completions"),
        other => (other, None, "chat_completions"),
    };

    CanonicalHermesRoute {
        provider: provider.to_string(),
        base_url: base_url.map(|u| u.to_string()),
        api_mode: api_mode.to_string(),
    }
}

/// Resolves the root directory of the Hermes installation associated with `cfg_path`.
pub fn resolve_hermes_root_dir(cfg_path_override: Option<&Path>) -> PathBuf {
    match cfg_path_override {
        Some(p) => p.parent().unwrap_or_else(|| Path::new(".")).to_path_buf(),
        None => resolve_hermes_config_path()
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf(),
    }
}

/// Discovers Hermes root `config.yaml` plus any sub-profile configs (`profiles/<name>/config.yaml`).
pub fn discover_hermes_profile_configs(hermes_root: &Path) -> Vec<(String, PathBuf)> {
    let mut out = Vec::new();
    let root_cfg = hermes_root.join("config.yaml");
    if root_cfg.exists() {
        out.push(("default".to_string(), root_cfg));
    }
    let profiles_dir = hermes_root.join("profiles");
    if let Ok(entries) = fs::read_dir(&profiles_dir) {
        let mut sub: Vec<(String, PathBuf)> = entries
            .filter_map(|e| e.ok())
            .filter_map(|e| {
                let p = e.path();
                if p.is_dir() {
                    let cfg = p.join("config.yaml");
                    if cfg.exists() {
                        let name = p.file_name()?.to_string_lossy().to_string();
                        return Some((name, cfg));
                    }
                }
                None
            })
            .collect();
        sub.sort_by(|a, b| a.0.cmp(&b.0));
        out.extend(sub);
    }
    out
}

/// Inspects whether `value` inside a YAML line represents an active setting (strips quotes/comments).
fn parse_yaml_scalar(raw: &str) -> String {
    let stripped = raw.split('#').next().unwrap_or("").trim();
    stripped
        .trim_matches('"')
        .trim_matches('\'')
        .trim()
        .to_string()
}

/// Reads full top-level `model:` fields + `moa:` + `fallback_providers:` metadata from a `config.yaml`.
#[derive(Debug, Clone, Default)]
pub struct ParsedHermesConfigAudit {
    pub default_model: String,
    pub provider: String,
    pub base_url: Option<String>,
    pub api_mode: Option<String>,
    pub moa_enabled: bool,
    pub moa_reference_count: usize,
    pub moa_uses_nous: bool,
    pub fallback_providers: Vec<Value>,
}

pub fn parse_hermes_config_audit(cfg_path: &Path) -> ParsedHermesConfigAudit {
    let mut audit = ParsedHermesConfigAudit::default();
    let Ok(content) = fs::read_to_string(cfg_path) else {
        return audit;
    };

    #[derive(Copy, Clone, PartialEq)]
    enum Section {
        Other,
        Model,
        Moa,
        Fallback,
    }
    let mut section = Section::Other;
    let mut in_moa_ref_item = false;
    let mut current_fb: Option<serde_json::Map<String, Value>> = None;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let indent = line.len() - line.trim_start().len();
        if indent == 0 {
            if let Some(fb) = current_fb.take() {
                audit.fallback_providers.push(Value::Object(fb));
            }
            in_moa_ref_item = false;
            if trimmed.starts_with("model:") {
                section = Section::Model;
            } else if trimmed.starts_with("moa:") {
                section = Section::Moa;
            } else if trimmed.starts_with("fallback_providers:") {
                section = Section::Fallback;
            } else {
                section = Section::Other;
            }
            continue;
        }

        match section {
            Section::Model => {
                if indent == 2 {
                    if let Some((k, v)) = trimmed.split_once(':') {
                        let val = parse_yaml_scalar(v);
                        match k.trim() {
                            "default" if !val.is_empty() => audit.default_model = val,
                            "provider" if !val.is_empty() => audit.provider = val,
                            "base_url" if !val.is_empty() => audit.base_url = Some(val),
                            "api_mode" if !val.is_empty() => audit.api_mode = Some(val),
                            _ => {}
                        }
                    }
                }
            }
            Section::Moa => {
                if indent == 2 {
                    in_moa_ref_item = false;
                    if let Some((k, v)) = trimmed.split_once(':') {
                        if k.trim() == "enabled" {
                            audit.moa_enabled = parse_yaml_scalar(v).eq_ignore_ascii_case("true");
                        }
                    }
                } else if trimmed.starts_with("- provider:") || trimmed.starts_with("provider:") {
                    let val = parse_yaml_scalar(trimmed.split_once(':').map(|(_, v)| v).unwrap_or(""));
                    if val.eq_ignore_ascii_case("nous") || val.eq_ignore_ascii_case("nous_portal") {
                        audit.moa_uses_nous = true;
                    }
                    if trimmed.starts_with("- ") {
                        in_moa_ref_item = true;
                    }
                } else if trimmed.starts_with("model:") && in_moa_ref_item {
                    audit.moa_reference_count += 1;
                }
            }
            Section::Fallback => {
                if trimmed.starts_with("- ") {
                    if let Some(fb) = current_fb.take() {
                        audit.fallback_providers.push(Value::Object(fb));
                    }
                    let mut map = serde_json::Map::new();
                    if let Some((k, v)) = trimmed.trim_start_matches("- ").split_once(':') {
                        map.insert(k.trim().to_string(), json!(parse_yaml_scalar(v)));
                    }
                    current_fb = Some(map);
                } else if let Some(ref mut map) = current_fb {
                    if let Some((k, v)) = trimmed.split_once(':') {
                        map.insert(k.trim().to_string(), json!(parse_yaml_scalar(v)));
                    }
                }
            }
            Section::Other => {}
        }
    }
    if let Some(fb) = current_fb.take() {
        audit.fallback_providers.push(Value::Object(fb));
    }
    audit
}

/// Mutates the top-level `model:` YAML block atomically while preserving comments and other keys.
fn rewrite_model_block_yaml(
    content: &str,
    target_model: &str,
    route: &CanonicalHermesRoute,
    clear_stale_context: bool,
) -> Result<String, String> {
    let mut inside_top_model = false;
    let mut default_replaced = false;
    let mut provider_replaced = false;
    let mut base_url_handled = false;
    let mut api_mode_handled = false;
    let mut new_lines = Vec::new();

    let emit_missing_fields = |lines: &mut Vec<String>,
                               provider_done: &mut bool,
                               base_done: &mut bool,
                               mode_done: &mut bool| {
        if !*provider_done {
            lines.push(format!("  provider: \"{}\"", route.provider));
            *provider_done = true;
        }
        if !*base_done {
            if let Some(ref url) = route.base_url {
                lines.push(format!("  base_url: '{}'", url));
            }
            *base_done = true;
        }
        if !*mode_done {
            lines.push(format!("  api_mode: {}", route.api_mode));
            *mode_done = true;
        }
    };

    for line in content.lines() {
        if line.starts_with("model:") {
            inside_top_model = true;
            new_lines.push(line.to_string());
            continue;
        }

        if inside_top_model {
            let trimmed = line.trim();
            if line.starts_with("  default:") {
                new_lines.push(format!("  default: \"{}\"", target_model));
                default_replaced = true;
                continue;
            } else if line.starts_with("  provider:") {
                new_lines.push(format!("  provider: \"{}\"", route.provider));
                provider_replaced = true;
                continue;
            } else if line.starts_with("  base_url:") {
                if let Some(ref url) = route.base_url {
                    new_lines.push(format!("  base_url: '{}'", url));
                }
                base_url_handled = true;
                continue;
            } else if line.starts_with("  api_mode:") {
                new_lines.push(format!("  api_mode: {}", route.api_mode));
                api_mode_handled = true;
                continue;
            } else if clear_stale_context
                && (line.starts_with("  context_length:") || line.starts_with("  key_env:"))
            {
                // Drop stale per-provider overrides carried from prior gateways
                continue;
            } else if !line.starts_with(' ') && !trimmed.is_empty() {
                emit_missing_fields(
                    &mut new_lines,
                    &mut provider_replaced,
                    &mut base_url_handled,
                    &mut api_mode_handled,
                );
                inside_top_model = false;
            }
        }

        new_lines.push(line.to_string());
    }

    if inside_top_model {
        emit_missing_fields(
            &mut new_lines,
            &mut provider_replaced,
            &mut base_url_handled,
            &mut api_mode_handled,
        );
    }

    if !default_replaced {
        return Err("Top-level model.default key was not found in hermes config.yaml".to_string());
    }

    Ok(format!("{}\n", new_lines.join("\n")))
}

/// Applies a canonical model + route to a single Hermes `config.yaml` file atomically.
fn write_single_hermes_config(
    cfg_path: &Path,
    target_model: &str,
    route: &CanonicalHermesRoute,
) -> Result<(String, String, bool, Option<String>), String> {
    let (prev_model, prev_provider) = read_active_hermes_model(cfg_path);
    let backup_path = cfg_path.with_extension("yaml.bak");
    let backup_created = fs::copy(cfg_path, &backup_path).is_ok();

    let content = fs::read_to_string(cfg_path)
        .map_err(|e| format!("Failed to read hermes config: {e}"))?;
    let prev_canon = normalize_hermes_provider_and_route(&prev_provider, &prev_model);
    let route_changed = prev_canon.provider != route.provider || prev_canon.base_url != route.base_url;
    let updated = rewrite_model_block_yaml(&content, target_model, route, route_changed)?;

    fs::write(cfg_path, updated.as_bytes())
        .map_err(|e| format!("Failed to write updated config.yaml: {e}"))?;

    Ok((
        prev_model,
        prev_provider,
        backup_created,
        backup_created.then(|| backup_path.to_string_lossy().to_string()),
    ))
}

/// Non-destructively adds `<provider>::<model_id>` entries into Hermes Desktop `ui-preferences.json`.
pub fn sync_hermes_desktop_favorites(
    hermes_root: &Path,
    entries: &[(String, String)],
) -> Result<bool, String> {
    let prefs_path = hermes_root.join("ui-preferences.json");
    if !prefs_path.exists() || entries.is_empty() {
        return Ok(false);
    }
    let raw = fs::read_to_string(&prefs_path)
        .map_err(|e| format!("Falha ao ler ui-preferences.json: {e}"))?;
    let mut root: Value = serde_json::from_str(&raw)
        .map_err(|e| format!("ui-preferences.json inválido: {e}"))?;

    let values_obj = root
        .as_object_mut()
        .and_then(|o| o.get_mut("values"))
        .and_then(|v| v.as_object_mut())
        .ok_or_else(|| "Campo 'values' ausente em ui-preferences.json".to_string())?;

    let mut favs: Vec<String> = match values_obj.get("hermes.desktop.favorite-models") {
        Some(Value::String(s)) => serde_json::from_str::<Vec<String>>(s).unwrap_or_default(),
        Some(Value::Array(arr)) => arr
            .iter()
            .filter_map(|x| x.as_str().map(|s| s.to_string()))
            .collect(),
        _ => Vec::new(),
    };

    let mut seen: HashSet<String> = favs.iter().cloned().collect();
    let mut prepend = Vec::new();
    let mut changed = false;
    for (prov, model_id) in entries {
        let canon = normalize_hermes_provider_and_route(prov, model_id);
        let key = format!("{}::{}", canon.provider, model_id);
        if seen.insert(key.clone()) {
            prepend.push(key);
            changed = true;
        }
    }

    if !changed {
        return Ok(false);
    }

    prepend.extend(favs.drain(..));
    let serialized = serde_json::to_string(&prepend)
        .map_err(|e| format!("Falha ao serializar favoritos: {e}"))?;
    values_obj.insert("hermes.desktop.favorite-models".to_string(), Value::String(serialized));

    if let Some(root_obj) = root.as_object_mut() {
        let rev = root_obj.get("revision").and_then(|r| r.as_i64()).unwrap_or(0);
        root_obj.insert("revision".to_string(), json!(rev + 1));
    }

    let formatted = serde_json::to_string_pretty(&root)
        .map_err(|e| format!("Falha ao formatar ui-preferences.json: {e}"))?;
    fs::write(&prefs_path, format!("{formatted}\n").as_bytes())
        .map_err(|e| format!("Falha ao gravar ui-preferences.json: {e}"))?;
    Ok(true)
}

/// Applies a target model and normalized canonical provider to the Hermes Agent `config.yaml` file atomically.
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

    let (_prev_m, prev_p) = read_active_hermes_model(&cfg_path);
    let requested_provider = provider.unwrap_or(&prev_p);
    let route = normalize_hermes_provider_and_route(requested_provider, target_model);

    let (prev_model, _old_prov, backup_created, backup_path) =
        write_single_hermes_config(&cfg_path, target_model, &route)?;

    let hermes_root = resolve_hermes_root_dir(cfg_path_override);
    let favorites_synced = sync_hermes_desktop_favorites(
        &hermes_root,
        &[(route.provider.clone(), target_model.to_string())],
    )
    .unwrap_or(false);

    info!(
        previous_model = %prev_model,
        applied_model = %target_model,
        provider = %route.provider,
        base_url = ?route.base_url,
        api_mode = %route.api_mode,
        config_path = %cfg_path.display(),
        "Hermes Master Brain profile applied atomically"
    );

    Ok(ProfileApplyResult {
        status: "success".to_string(),
        profile_id: profile_id.map(|s| s.to_string()),
        previous_model: prev_model,
        applied_model: target_model.to_string(),
        provider: route.provider,
        backup_created,
        backup_path,
        config_path: cfg_path.to_string_lossy().to_string(),
        base_url: route.base_url,
        api_mode: route.api_mode,
        profiles_updated: vec!["default".to_string()],
        unpinned_sessions: 0,
        favorites_synced,
        warnings: Vec::new(),
    })
}

/// Applies model & canonical route with optional multi-profile sync and fallback-session unpinning.
pub async fn apply_profile_to_hermes_extended(
    profile_id: Option<&str>,
    target_model: &str,
    provider: Option<&str>,
    cfg_path_override: Option<&Path>,
    apply_all_profiles: bool,
    unpin_fallback_sessions: bool,
) -> Result<ProfileApplyResult, String> {
    let mut res = apply_profile_to_hermes(profile_id, target_model, provider, cfg_path_override)?;
    let hermes_root = resolve_hermes_root_dir(cfg_path_override);
    let route = normalize_hermes_provider_and_route(&res.provider, target_model);

    if apply_all_profiles {
        for (prof_name, prof_cfg) in discover_hermes_profile_configs(&hermes_root) {
            if prof_name == "default" {
                continue;
            }
            if write_single_hermes_config(&prof_cfg, target_model, &route).is_ok() {
                res.profiles_updated.push(prof_name);
            }
        }
    }

    if unpin_fallback_sessions {
        match unpin_hermes_fallback_sessions(&hermes_root, &route, target_model).await {
            Ok(count) => res.unpinned_sessions = count,
            Err(e) => res.warnings.push(e),
        }
    }

    Ok(res)
}

/// Unpins open Hermes Desktop sessions trapped on `llamacpp` fallback inside `state.db` files.
pub async fn unpin_hermes_fallback_sessions(
    hermes_root: &Path,
    route: &CanonicalHermesRoute,
    target_model: &str,
) -> Result<usize, String> {
    let mut db_paths = Vec::new();
    let root_db = hermes_root.join("state.db");
    if root_db.exists() {
        db_paths.push(root_db);
    }
    let profiles_dir = hermes_root.join("profiles");
    if let Ok(entries) = fs::read_dir(profiles_dir) {
        for entry in entries.flatten() {
            let candidate = entry.path().join("state.db");
            if candidate.exists() {
                db_paths.push(candidate);
            }
        }
    }

    let mut total_unpinned = 0usize;
    for db_path in db_paths {
        let opts = sqlx::sqlite::SqliteConnectOptions::new()
            .filename(&db_path)
            .read_only(false)
            .create_if_missing(false)
            .busy_timeout(std::time::Duration::from_millis(3000));
        let Ok(pool) = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(opts)
            .await
        else {
            continue;
        };

        let Ok(rows) = sqlx::query(
            "SELECT id, model, model_config FROM sessions WHERE COALESCE(archived, 0) = 0",
        )
        .fetch_all(&pool)
        .await
        else {
            continue;
        };

        for row in rows {
            use sqlx::Row;
            let id: String = row.try_get("id").unwrap_or_default();
            let raw_cfg: Option<String> = row.try_get("model_config").ok();
            let mut cfg_val: Value = raw_cfg
                .as_deref()
                .and_then(|s| serde_json::from_str(s).ok())
                .unwrap_or_else(|| json!({}));

            let is_trapped_fallback = {
                let gr = cfg_val.get("gateway_runtime");
                let gr_prov = gr
                    .and_then(|g| g.get("provider"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let gr_fb = gr
                    .and_then(|g| g.get("fallback_active"))
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let top_follow = cfg_val
                    .get("follow_profile_config")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                gr_prov.eq_ignore_ascii_case("llamacpp")
                    || gr_fb
                    || (gr.is_some() && !top_follow)
            };

            if !is_trapped_fallback {
                continue;
            }

            if let Some(obj) = cfg_val.as_object_mut() {
                obj.insert("follow_profile_config".to_string(), json!(true));
                let mut runtime_obj = serde_json::Map::new();
                runtime_obj.insert("provider".to_string(), json!(route.provider));
                if let Some(ref bu) = route.base_url {
                    runtime_obj.insert("base_url".to_string(), json!(bu));
                }
                runtime_obj.insert("api_mode".to_string(), json!(route.api_mode));
                runtime_obj.insert("model".to_string(), json!(target_model));
                runtime_obj.insert("fallback_active".to_string(), json!(false));
                obj.insert("gateway_runtime".to_string(), Value::Object(runtime_obj));
            }

            if let Ok(updated_json) = serde_json::to_string(&cfg_val) {
                let res = sqlx::query(
                    "UPDATE sessions SET model = ?, model_config = ?, billing_provider = ? WHERE id = ?",
                )
                .bind(target_model)
                .bind(updated_json)
                .bind(&route.provider)
                .bind(&id)
                .execute(&pool)
                .await;
                if res.is_ok() {
                    total_unpinned += 1;
                }
            }
        }
        pool.close().await;
    }

    Ok(total_unpinned)
}

/// Toggles top-level `moa.enabled` inside `config.yaml` without disturbing `reference_models`.
pub fn set_hermes_moa_enabled(
    hermes_root: &Path,
    enabled: bool,
    all_profiles: bool,
) -> Result<Vec<String>, String> {
    let targets = discover_hermes_profile_configs(hermes_root);
    let mut updated = Vec::new();
    for (name, cfg_path) in targets {
        if !all_profiles && name != "default" {
            continue;
        }
        let content = fs::read_to_string(&cfg_path)
            .map_err(|e| format!("Falha ao ler {}: {e}", cfg_path.display()))?;
        let mut inside_moa = false;
        let mut moa_found = false;
        let mut enabled_written = false;
        let mut out_lines = Vec::new();
        for line in content.lines() {
            if line.starts_with("moa:") {
                inside_moa = true;
                moa_found = true;
                out_lines.push(line.to_string());
                continue;
            }
            if inside_moa {
                let trimmed = line.trim();
                let indent = line.len() - line.trim_start().len();
                if indent == 0 && !trimmed.is_empty() && !trimmed.starts_with('#') {
                    if !enabled_written {
                        out_lines.push(format!("  enabled: {enabled}"));
                        enabled_written = true;
                    }
                    inside_moa = false;
                } else if indent == 2 && trimmed.starts_with("enabled:") {
                    out_lines.push(format!("  enabled: {enabled}"));
                    enabled_written = true;
                    continue;
                }
            }
            out_lines.push(line.to_string());
        }
        if inside_moa && !enabled_written {
            out_lines.push(format!("  enabled: {enabled}"));
        }
        if moa_found {
            let _ = fs::copy(&cfg_path, cfg_path.with_extension("yaml.bak"));
            fs::write(&cfg_path, format!("{}\n", out_lines.join("\n")).as_bytes())
                .map_err(|e| format!("Falha ao atualizar {}: {e}", cfg_path.display()))?;
            updated.push(name);
        }
    }
    Ok(updated)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayAuthStatus {
    pub id: String,
    pub label: String,
    pub auth_mode: String,
    pub authenticated: bool,
    pub status: String,
    pub env_key: Option<String>,
    pub key_url: Option<String>,
    pub masked_hint: Option<String>,
    pub detail: String,
    pub active_profile_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileConfigAudit {
    pub profile_id: String,
    pub config_path: String,
    pub default_model: String,
    pub raw_provider: String,
    pub canonical_provider: String,
    pub provider_valid: bool,
    pub base_url: Option<String>,
    pub expected_base_url: Option<String>,
    pub base_url_aligned: bool,
    pub api_mode: Option<String>,
    pub moa_enabled: bool,
    pub moa_reference_count: usize,
    pub moa_uses_expired_nous: bool,
    pub fallback_providers: Vec<Value>,
    pub stuck_fallback_sessions: usize,
    pub issues: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HermesCustomsReport {
    pub health: String,
    pub hermes_root: String,
    pub active_profile_model: String,
    pub active_profile_provider: String,
    pub active_provider_authenticated: bool,
    pub gateways: Vec<GatewayAuthStatus>,
    pub active_gateway_ids: Vec<String>,
    pub profiles: Vec<ProfileConfigAudit>,
    pub total_stuck_fallback_sessions: usize,
    pub moa_enabled_anywhere: bool,
    pub issues: Vec<String>,
    pub needs_config_repair: bool,
    pub needs_session_unpin: bool,
    pub nous_needs_reauth: bool,
    pub favorites_count: usize,
}

fn load_env_vars_from_file(path: &Path, out: &mut HashMap<String, String>) {
    let Ok(content) = fs::read_to_string(path) else {
        return;
    };
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let cleaned = trimmed.strip_prefix("export ").unwrap_or(trimmed);
        if let Some((k, v)) = cleaned.split_once('=') {
            let key = k.trim();
            let val = v.trim().trim_matches('"').trim_matches('\'').trim();
            if !key.is_empty() && !val.is_empty() {
                out.insert(key.to_string(), val.to_string());
            }
        }
    }
}

fn mask_secret_hint(secret: &str) -> String {
    let chars: Vec<char> = secret.chars().collect();
    if chars.len() <= 6 {
        "***".to_string()
    } else {
        let prefix: String = chars.iter().take(4).collect();
        let suffix: String = chars.iter().rev().take(3).collect::<Vec<_>>().into_iter().rev().collect();
        format!("{prefix}••••{suffix}")
    }
}

async fn count_stuck_fallback_sessions_by_profile(hermes_root: &Path) -> HashMap<String, usize> {
    let mut counts = HashMap::new();
    let mut candidates = vec![("default".to_string(), hermes_root.join("state.db"))];
    if let Ok(entries) = fs::read_dir(hermes_root.join("profiles")) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                let db = p.join("state.db");
                if db.exists() {
                    if let Some(name) = p.file_name() {
                        candidates.push((name.to_string_lossy().to_string(), db));
                    }
                }
            }
        }
    }

    for (prof_name, db_path) in candidates {
        if !db_path.exists() {
            continue;
        }
        let opts = sqlx::sqlite::SqliteConnectOptions::new()
            .filename(&db_path)
            .read_only(true)
            .create_if_missing(false)
            .busy_timeout(std::time::Duration::from_millis(1500));
        let Ok(pool) = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(opts)
            .await
        else {
            continue;
        };
        let mut trapped = 0usize;
        if let Ok(rows) = sqlx::query(
            "SELECT model_config FROM sessions WHERE COALESCE(archived, 0) = 0",
        )
        .fetch_all(&pool)
        .await
        {
            for row in rows {
                use sqlx::Row;
                let raw_cfg: Option<String> = row.try_get("model_config").ok();
                if let Some(v) = raw_cfg.as_deref().and_then(|s| serde_json::from_str::<Value>(s).ok()) {
                    let gr = v.get("gateway_runtime");
                    let gr_prov = gr.and_then(|g| g.get("provider")).and_then(|x| x.as_str()).unwrap_or("");
                    let gr_fb = gr.and_then(|g| g.get("fallback_active")).and_then(|x| x.as_bool()).unwrap_or(false);
                    let follow = v.get("follow_profile_config").and_then(|x| x.as_bool()).unwrap_or(false);
                    if gr_prov.eq_ignore_ascii_case("llamacpp") || gr_fb || (gr.is_some() && !follow) {
                        trapped += 1;
                    }
                }
            }
        }
        pool.close().await;
        counts.insert(prof_name, trapped);
    }
    counts
}

/// Inspects Hermes Desktop auth (`auth.json` + `.env`), `config.yaml` routing across profiles,
/// `moa.enabled`, favorites sync, and open fallback-trapped sessions.
pub async fn inspect_hermes_customs_with_options(
    hermes_root: &Path,
    include_host_env: bool,
) -> HermesCustomsReport {
    let mut env_map: HashMap<String, String> = HashMap::new();
    if include_host_env {
        for (k, v) in std::env::vars() {
            if !v.trim().is_empty() {
                env_map.insert(k, v);
            }
        }
        let souls_data = crate::resolve_data_dir();
        for fname in ["credentials.env", ".env"] {
            let p = souls_data.join(fname);
            if p.exists() {
                load_env_vars_from_file(&p, &mut env_map);
            }
        }
    }
    load_env_vars_from_file(&hermes_root.join(".env"), &mut env_map);
    load_env_vars_from_file(&hermes_root.join("credentials.env"), &mut env_map);

    let auth_json: Option<Value> = fs::read_to_string(hermes_root.join("auth.json"))
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok());

    let nous_pool_count = auth_json
        .as_ref()
        .and_then(|v| v.get("credential_pool"))
        .and_then(|c| c.get("nous"))
        .and_then(|n| n.as_array())
        .map(|a| a.len())
        .unwrap_or(0);
    let nous_relogin_req = auth_json
        .as_ref()
        .and_then(|v| v.get("providers"))
        .and_then(|p| p.get("nous"))
        .and_then(|n| n.get("last_auth_error"))
        .and_then(|e| e.get("relogin_required"))
        .and_then(|b| b.as_bool())
        .unwrap_or(false);
    let nous_err_code = auth_json
        .as_ref()
        .and_then(|v| v.get("providers"))
        .and_then(|p| p.get("nous"))
        .and_then(|n| n.get("last_auth_error"))
        .and_then(|e| e.get("code"))
        .and_then(|s| s.as_str())
        .unwrap_or("");
    let nous_authenticated = nous_pool_count > 0 && !nous_relogin_req;

    let copilot_pool_count = auth_json
        .as_ref()
        .and_then(|v| v.get("credential_pool"))
        .and_then(|c| c.get("copilot"))
        .and_then(|n| n.as_array())
        .map(|a| a.len())
        .unwrap_or(0);
    let copilot_token = env_map.get("GITHUB_TOKEN").or_else(|| env_map.get("COPILOT_GITHUB_TOKEN"));
    let copilot_ok = copilot_pool_count > 0 || copilot_token.is_some();

    let mut gateways = Vec::new();
    gateways.push(GatewayAuthStatus {
        id: "nous".to_string(),
        label: "Nous Portal".to_string(),
        auth_mode: "oauth_device".to_string(),
        authenticated: nous_authenticated,
        status: if nous_authenticated {
            "ok".to_string()
        } else if nous_relogin_req {
            "expired".to_string()
        } else {
            "missing".to_string()
        },
        env_key: None,
        key_url: Some("https://portal.nousresearch.com".to_string()),
        masked_hint: if nous_authenticated {
            Some(format!("{nous_pool_count} sessão OAuth"))
        } else {
            None
        },
        detail: if nous_authenticated {
            "Sessão OAuth Nous Portal ativa.".to_string()
        } else if nous_relogin_req {
            format!(
                "Sessão expirada no Hermes ({}) — reconecte com `hermes login` para evitar queda para modelo local.",
                if nous_err_code.is_empty() { "token expirado" } else { nous_err_code }
            )
        } else {
            "Nenhuma sessão OAuth Nous conectada — autentique via `hermes login`.".to_string()
        },
        active_profile_count: 0,
    });

    struct KeyGatewaySpec {
        id: &'static str,
        label: &'static str,
        envs: &'static [&'static str],
        url: &'static str,
    }
    let key_specs: &[KeyGatewaySpec] = &[
        KeyGatewaySpec {
            id: "openrouter",
            label: "OpenRouter",
            envs: &["OPENROUTER_API_KEY"],
            url: "https://openrouter.ai/settings/keys",
        },
        KeyGatewaySpec {
            id: "deepseek",
            label: "DeepSeek Direct",
            envs: &["DEEPSEEK_API_KEY"],
            url: "https://platform.deepseek.com/api_keys",
        },
        KeyGatewaySpec {
            id: "anthropic",
            label: "Anthropic Direct",
            envs: &["ANTHROPIC_API_KEY", "ANTHROPIC_TOKEN"],
            url: "https://console.anthropic.com/settings/keys",
        },
        KeyGatewaySpec {
            id: "openai-api",
            label: "OpenAI Direct",
            envs: &["OPENAI_API_KEY"],
            url: "https://platform.openai.com/api-keys",
        },
        KeyGatewaySpec {
            id: "gemini",
            label: "Google AI Studio",
            envs: &["GEMINI_API_KEY", "GOOGLE_API_KEY"],
            url: "https://aistudio.google.com/apikey",
        },
        KeyGatewaySpec {
            id: "zai",
            label: "Z.AI / GLM",
            envs: &["ZAI_API_KEY", "GLM_API_KEY", "Z_AI_API_KEY"],
            url: "https://z.ai/manage-apikey/apikey-list",
        },
        KeyGatewaySpec {
            id: "xai",
            label: "xAI / Grok",
            envs: &["XAI_API_KEY"],
            url: "https://console.x.ai/",
        },
        KeyGatewaySpec {
            id: "nvidia",
            label: "NVIDIA NIM",
            envs: &["NVIDIA_API_KEY"],
            url: "https://build.nvidia.com/settings/api-keys",
        },
    ];

    for spec in key_specs {
        let found = spec
            .envs
            .iter()
            .find_map(|k| env_map.get(*k).map(|v| (*k, v)));
        gateways.push(GatewayAuthStatus {
            id: spec.id.to_string(),
            label: spec.label.to_string(),
            auth_mode: "api_key".to_string(),
            authenticated: found.is_some(),
            status: if found.is_some() { "ok" } else { "missing" }.to_string(),
            env_key: Some(spec.envs[0].to_string()),
            key_url: Some(spec.url.to_string()),
            masked_hint: found.map(|(_, v)| mask_secret_hint(v)),
            detail: if let Some((k, _)) = found {
                format!("Chave configurada em {k}.")
            } else {
                format!("Chave {} ainda não configurada.", spec.envs[0])
            },
            active_profile_count: 0,
        });
    }

    gateways.push(GatewayAuthStatus {
        id: "copilot".to_string(),
        label: "GitHub Copilot".to_string(),
        auth_mode: "oauth_external".to_string(),
        authenticated: copilot_ok,
        status: if copilot_ok { "ok" } else { "missing" }.to_string(),
        env_key: Some("GITHUB_TOKEN".to_string()),
        key_url: Some("https://github.com/settings/tokens".to_string()),
        masked_hint: copilot_token.map(|v| mask_secret_hint(v)),
        detail: if copilot_ok {
            "Token/Copilot ativo no Hermes.".to_string()
        } else {
            "GitHub Copilot não autenticado.".to_string()
        },
        active_profile_count: 0,
    });

    gateways.push(GatewayAuthStatus {
        id: "custom".to_string(),
        label: "Silício Local (llama.cpp)".to_string(),
        auth_mode: "local".to_string(),
        authenticated: true,
        status: "local".to_string(),
        env_key: None,
        key_url: None,
        masked_hint: Some("127.0.0.1:18434".to_string()),
        detail: "Inferência local soberana na GPU/CPU.".to_string(),
        active_profile_count: 0,
    });

    let stuck_by_profile = count_stuck_fallback_sessions_by_profile(hermes_root).await;
    let cfgs = discover_hermes_profile_configs(hermes_root);
    let mut profiles = Vec::new();
    let mut global_issues = Vec::new();
    let mut needs_config_repair = false;
    let mut needs_session_unpin = false;
    let mut moa_enabled_anywhere = false;

    for (prof_id, cfg_path) in cfgs {
        let parsed = parse_hermes_config_audit(&cfg_path);
        let expected = normalize_hermes_provider_and_route(&parsed.provider, &parsed.default_model);
        let provider_valid = OFFICIAL_HERMES_PROVIDERS.contains(&parsed.provider.trim());
        let base_url_aligned = match (&parsed.base_url, &expected.base_url) {
            (Some(actual), Some(exp)) => actual.trim_end_matches('/') == exp.trim_end_matches('/'),
            (None, None) => true,
            (Some(_), None) => true,
            (None, Some(_)) => expected.provider != "nous" && expected.provider != "openrouter" && expected.provider != "custom",
        };
        let stuck = *stuck_by_profile.get(&prof_id).unwrap_or(&0);
        if stuck > 0 {
            needs_session_unpin = true;
        }
        if parsed.moa_enabled {
            moa_enabled_anywhere = true;
        }

        let mut p_issues = Vec::new();
        if !provider_valid {
            needs_config_repair = true;
            p_issues.push(format!(
                "Provedor '{}' inválido para o Hermes Desktop (corrigir para '{}').",
                parsed.provider, expected.provider
            ));
        }
        if !base_url_aligned {
            needs_config_repair = true;
            p_issues.push(format!(
                "URL base divergente para '{}' (esperado: {}).",
                expected.provider,
                expected.base_url.as_deref().unwrap_or("padrão")
            ));
        }
        if parsed.moa_enabled && parsed.moa_uses_nous && !nous_authenticated {
            p_issues.push(format!(
                "Modo paralelo (MoA) está ativo com {} modelos Nous mesmo sem sessão Nous válida.",
                parsed.moa_reference_count
            ));
        }
        if stuck > 0 {
            p_issues.push(format!(
                "{} sessão(ões) presa(s) em fallback local neste perfil.",
                stuck
            ));
        }

        if let Some(gw) = gateways.iter_mut().find(|g| g.id == expected.provider) {
            gw.active_profile_count += 1;
        }

        profiles.push(ProfileConfigAudit {
            profile_id: prof_id,
            config_path: cfg_path.to_string_lossy().to_string(),
            default_model: parsed.default_model,
            raw_provider: parsed.provider,
            canonical_provider: expected.provider,
            provider_valid,
            base_url: parsed.base_url,
            expected_base_url: expected.base_url,
            base_url_aligned,
            api_mode: parsed.api_mode,
            moa_enabled: parsed.moa_enabled,
            moa_reference_count: parsed.moa_reference_count,
            moa_uses_expired_nous: parsed.moa_uses_nous && !nous_authenticated,
            fallback_providers: parsed.fallback_providers,
            stuck_fallback_sessions: stuck,
            issues: p_issues,
        });
    }

    let (active_profile_model, active_profile_provider, active_provider_authenticated) = profiles
        .iter()
        .find(|p| p.profile_id == "default")
        .or_else(|| profiles.first())
        .map(|p| {
            let auth_ok = gateways
                .iter()
                .find(|g| g.id == p.canonical_provider)
                .map(|g| g.authenticated)
                .unwrap_or(false);
            (p.default_model.clone(), p.canonical_provider.clone(), auth_ok)
        })
        .unwrap_or_else(|| ("deepseek/deepseek-chat".to_string(), "nous".to_string(), nous_authenticated));

    if !active_provider_authenticated {
        global_issues.push(format!(
            "O provedor ativo '{}' não possui credencial válida no Hermes Desktop; novas conversas cairão no fallback local.",
            active_profile_provider
        ));
    }
    if needs_config_repair {
        global_issues.push("Foram detectados identificadores ou URLs incompatíveis em config.yaml do Hermes.".to_string());
    }
    let total_stuck_fallback_sessions: usize = profiles.iter().map(|p| p.stuck_fallback_sessions).sum();
    if total_stuck_fallback_sessions > 0 {
        global_issues.push(format!(
            "{} conversa(s) aberta(s) no Hermes estão presas em modo local de emergência.",
            total_stuck_fallback_sessions
        ));
    }

    let favorites_count = fs::read_to_string(hermes_root.join("ui-preferences.json"))
        .ok()
        .and_then(|raw| serde_json::from_str::<Value>(&raw).ok())
        .and_then(|v| {
            v.get("values")
                .and_then(|vals| vals.get("hermes.desktop.favorite-models"))
                .and_then(|f| f.as_str())
                .and_then(|s| serde_json::from_str::<Vec<String>>(s).ok())
                .map(|arr| arr.len())
        })
        .unwrap_or(0);

    let active_gateway_ids: Vec<String> = gateways
        .iter()
        .filter(|g| g.authenticated && g.id != "custom")
        .map(|g| g.id.clone())
        .collect();

    let health = if !active_provider_authenticated || needs_config_repair {
        "critical"
    } else if total_stuck_fallback_sessions > 0 || (moa_enabled_anywhere && !nous_authenticated) {
        "warning"
    } else {
        "healthy"
    }
    .to_string();

    HermesCustomsReport {
        health,
        hermes_root: hermes_root.to_string_lossy().to_string(),
        active_profile_model,
        active_profile_provider,
        active_provider_authenticated,
        gateways,
        active_gateway_ids,
        profiles,
        total_stuck_fallback_sessions,
        moa_enabled_anywhere,
        issues: global_issues,
        needs_config_repair,
        needs_session_unpin,
        nous_needs_reauth: !nous_authenticated,
        favorites_count,
    }
}

/// Repairs Hermes `config.yaml` files (canonical slugs + base URLs), optional `moa.enabled`,
/// fallback-pinned sessions in `state.db`, and favorites sync.
pub async fn repair_hermes_customs(
    hermes_root: &Path,
    repair_configs: bool,
    unpin_sessions: bool,
    moa_enabled: Option<bool>,
) -> Result<Value, String> {
    let mut repaired_configs = Vec::new();
    let mut moa_updated = Vec::new();
    let mut fav_entries = Vec::new();
    let mut unpinned_sessions = 0usize;

    if repair_configs {
        for (prof_name, cfg_path) in discover_hermes_profile_configs(hermes_root) {
            let parsed = parse_hermes_config_audit(&cfg_path);
            if parsed.default_model.is_empty() {
                continue;
            }
            let route = normalize_hermes_provider_and_route(&parsed.provider, &parsed.default_model);
            let _ = write_single_hermes_config(&cfg_path, &parsed.default_model, &route)?;
            fav_entries.push((route.provider.clone(), parsed.default_model.clone()));
            repaired_configs.push(prof_name);
        }
    }

    if let Some(target_moa) = moa_enabled {
        moa_updated = set_hermes_moa_enabled(hermes_root, target_moa, true)?;
    }

    if unpin_sessions {
        let root_cfg = hermes_root.join("config.yaml");
        let (m, p) = read_active_hermes_model(&root_cfg);
        let route = normalize_hermes_provider_and_route(&p, &m);
        unpinned_sessions = unpin_hermes_fallback_sessions(hermes_root, &route, &m).await?;
    }

    let favorites_synced = sync_hermes_desktop_favorites(hermes_root, &fav_entries).unwrap_or(false);

    Ok(json!({
        "status": "success",
        "repaired_configs": repaired_configs,
        "moa_updated_profiles": moa_updated,
        "unpinned_sessions": unpinned_sessions,
        "favorites_synced": favorites_synced
    }))
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

    #[test]
    fn test_dynamic_profiles_toml_lifecycle() {
        let dir = std::env::temp_dir().join(format!(
            "souls_dyn_profile_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = fs::create_dir_all(&dir);
        let toml_file = dir.join("souls_config.toml");

        // 1. Salva um novo perfil customizado de subagente
        save_or_update_profile(
            "security_auditor",
            "Security Auditor 🛡️",
            "Auditoria de vulnerabilidades e AST",
            "subagent",
            "reasoning",
            "openai/o3-mini-high",
            "nous",
            Some(&toml_file),
        )
        .unwrap();

        // 2. Lê e verifica parse
        let content = fs::read_to_string(&toml_file).unwrap();
        let parsed = parse_profiles_from_toml(&content);
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].id, "security_auditor");
        assert_eq!(parsed[0].name, "Security Auditor 🛡️");
        assert_eq!(parsed[0].arena_category, "reasoning");
        assert_eq!(parsed[0].role, "subagent");

        // 3. Verifica proteção contra deleção de orchestrator
        let del_orch = delete_profile("orchestrator", Some(&toml_file));
        assert!(del_orch.is_err());

        // 4. Deleta o perfil customizado com sucesso
        let del_sec = delete_profile("security_auditor", Some(&toml_file)).unwrap();
        assert!(del_sec);
        let after_content = fs::read_to_string(&toml_file).unwrap();
        assert!(!after_content.contains("security_auditor"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_resolve_triad_for_category() {
        let triad_coding = resolve_triad_for_category("coding", &[]);
        assert!(triad_coding.quality_leader.is_some());
        assert!(triad_coding.best_value.is_some());
        assert!(triad_coding.ultra_low_cost.is_some());
        assert_eq!(
            triad_coding.quality_leader.as_ref().unwrap().canonical_id,
            "anthropic/claude-3-7-sonnet"
        );
        assert_eq!(triad_coding.leader, triad_coding.quality_leader);
        assert_eq!(triad_coding.best, triad_coding.best_value);
        assert_eq!(triad_coding.low, triad_coding.ultra_low_cost);

        let triad_local = resolve_triad_for_category("local", &[]);
        assert_eq!(
            triad_local.quality_leader.as_ref().unwrap().provider,
            "local_silicon"
        );
        assert_eq!(triad_local.best_value.as_ref().unwrap().blended_cost, 0.0);
        assert_eq!(triad_local.leader, triad_local.quality_leader);
        assert_eq!(triad_local.best, triad_local.best_value);
        assert_eq!(triad_local.low, triad_local.ultra_low_cost);

        // Verify dynamic hardware string: must not contain static RTX 2060m
        let desc = &triad_local.quality_leader.as_ref().unwrap().desc;
        assert!(!desc.contains("RTX 2060m"), "Hardware string should be dynamic, but was: {}", desc);
    }

    #[test]
    fn test_normalize_hermes_provider_and_route_canonicals() {
        let r_nous = normalize_hermes_provider_and_route("nous_portal", "openai/gpt-6.1-sol-pro");
        assert_eq!(r_nous.provider, "nous");
        assert_eq!(
            r_nous.base_url.as_deref(),
            Some("https://inference-api.nousresearch.com/v1")
        );
        assert_eq!(r_nous.api_mode, "chat_completions");

        let r_or = normalize_hermes_provider_and_route("openrouter_market", "meituan/longcat-2.5-preview:free");
        assert_eq!(r_or.provider, "openrouter");
        assert_eq!(r_or.base_url.as_deref(), Some("https://openrouter.ai/api/v1"));

        let r_local = normalize_hermes_provider_and_route("local_silicon", "Qwen3.5-4B.Q4_K_M.gguf");
        assert_eq!(r_local.provider, "custom");
        assert_eq!(r_local.base_url.as_deref(), Some("http://127.0.0.1:18434/v1"));

        let r_ant = normalize_hermes_provider_and_route("anthropic", "anthropic/claude-opus-5.5");
        assert_eq!(r_ant.provider, "anthropic");
        assert_eq!(r_ant.api_mode, "anthropic_messages");
    }

    #[tokio::test]
    async fn test_customs_inspect_repair_favorites_and_session_unpin() {
        let dir = std::env::temp_dir().join(format!(
            "souls_customs_test_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let prof_dir = dir.join("profiles").join("financas");
        fs::create_dir_all(&prof_dir).unwrap();

        let bad_cfg = r#"model:
  default: "openai/gpt-6.1-sol-pro"
  provider: "nous_portal"
  base_url: 'https://stale.example/v1'
  api_mode: chat_completions
  context_length: 131072
moa:
  reference_models:
    - provider: nous
      model: stepfun/step-5-preview:free
      enabled: true
  enabled: true
fallback_providers:
  - provider: llamacpp
    model: Qwen3.5-4B-Uncensored-Q4_K_M
"#;
        fs::write(dir.join("config.yaml"), bad_cfg).unwrap();
        fs::write(prof_dir.join("config.yaml"), bad_cfg).unwrap();
        fs::write(
            dir.join("auth.json"),
            r#"{"providers":{"nous":{"last_auth_error":{"relogin_required":true,"code":"invalid_grant"}}},"credential_pool":{"nous":[],"copilot":["tok"]}}"#,
        )
        .unwrap();
        fs::write(dir.join(".env"), "OPENROUTER_API_KEY=sk-or-v1-test123456\n").unwrap();
        fs::write(
            dir.join("ui-preferences.json"),
            r#"{"version":1,"generation":"abc","revision":4,"values":{"hermes.desktop.favorite-models":"[\"nous::deepseek/deepseek-chat\"]"}}"#,
        )
        .unwrap();

        // Mock Hermes state.db with trapped llamacpp fallback session
        let db_path = dir.join("state.db");
        let opts = sqlx::sqlite::SqliteConnectOptions::new()
            .filename(&db_path)
            .create_if_missing(true);
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(opts)
            .await
            .unwrap();
        sqlx::query("CREATE TABLE sessions (id TEXT PRIMARY KEY, model TEXT, model_config TEXT, billing_provider TEXT, archived INTEGER DEFAULT 0)")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO sessions (id, model, model_config, billing_provider, archived) VALUES (?, ?, ?, ?, 0)")
            .bind("local-stuck-1")
            .bind("Qwen3.5-4B-Uncensored-Q4_K_M")
            .bind(r#"{"gateway_runtime":{"provider":"llamacpp","model":"Qwen3.5-4B-Uncensored-Q4_K_M","fallback_active":true}}"#)
            .bind("llamacpp")
            .execute(&pool)
            .await
            .unwrap();
        pool.close().await;

        let report_before = inspect_hermes_customs_with_options(&dir, false).await;
        assert_eq!(report_before.health, "critical");
        assert!(report_before.needs_config_repair);
        assert!(report_before.nous_needs_reauth);
        assert_eq!(report_before.total_stuck_fallback_sessions, 1);
        assert!(report_before.moa_enabled_anywhere);
        assert!(report_before.active_gateway_ids.contains(&"openrouter".to_string()));
        assert!(report_before.active_gateway_ids.contains(&"copilot".to_string()));

        // Apply OpenRouter model across all profiles + unpin sessions
        let apply_res = apply_profile_to_hermes_extended(
            Some("custom"),
            "openai/gpt-6.1-sol-pro",
            Some("openrouter_market"),
            Some(&dir.join("config.yaml")),
            true,
            true,
        )
        .await
        .unwrap();
        assert_eq!(apply_res.provider, "openrouter");
        assert_eq!(apply_res.unpinned_sessions, 1);
        assert!(apply_res.favorites_synced);
        assert_eq!(apply_res.profiles_updated.len(), 2);

        let updated_root = fs::read_to_string(dir.join("config.yaml")).unwrap();
        assert!(updated_root.contains(r#"provider: "openrouter""#));
        assert!(updated_root.contains("https://openrouter.ai/api/v1"));
        assert!(!updated_root.contains("context_length:"));
        assert!(!updated_root.contains('\r'));

        let prefs_raw = fs::read_to_string(dir.join("ui-preferences.json")).unwrap();
        assert!(prefs_raw.contains("openrouter::openai/gpt-6.1-sol-pro"));
        assert!(prefs_raw.contains("nous::deepseek/deepseek-chat"));

        // Disable MoA via repair_hermes_customs
        let _ = repair_hermes_customs(&dir, false, false, Some(false))
            .await
            .unwrap();
        let report_after = inspect_hermes_customs_with_options(&dir, false).await;
        assert_eq!(report_after.health, "healthy");
        assert!(!report_after.needs_config_repair);
        assert_eq!(report_after.total_stuck_fallback_sessions, 0);
        assert!(!report_after.moa_enabled_anywhere);

        let _ = fs::remove_dir_all(&dir);
    }
}
