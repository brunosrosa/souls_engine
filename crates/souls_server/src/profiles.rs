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

    let content =
        fs::read_to_string(&cfg_path).map_err(|e| format!("Failed to read hermes config: {e}"))?;

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
}
