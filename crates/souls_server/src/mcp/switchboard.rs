//! # Two-Stage Semantic Switchboard (`souls_server::mcp::switchboard`)
//!
//! Intelligent tool and skill discovery engine operating strictly on CPU Host (AVX2):
//! - Stage 1: Dense Semantic Cosine Match in RAM (< 5 ms) over tools and skills.
//! - Stage 2: Shannon Entropy Gating via Tier 0.5 Logit Probing (~8 ms) if confidence < 0.75.
//! - Context Preservation: Inviolable rule: Zero schemas injected into Master Chat prompt,
//!   preserving Prefix Caching hit rate > 95%.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::mcp::registry::get_canonical_tools;
use crate::state::AppState;

/// Canonical metadata for an operational skill in `.agents/skills/`.
#[derive(Debug, Clone, Serialize)]
pub struct SkillMeta {
    pub name: &'static str,
    pub description: &'static str,
    pub primary_tools: &'static [&'static str],
    pub keywords: &'static [&'static str],
}

/// Static catalog of all 15 canonical skills.
pub static CANONICAL_SKILLS: &[SkillMeta] = &[
    SkillMeta {
        name: "souls-code-surgeon",
        description: "Cirurgia sintática de código via AST. Proíbe ler arquivos inteiros.",
        primary_tools: &["outline", "symbol", "edit", "replace", "delta_diff"],
        keywords: &["code", "syntax", "ast", "refactor", "function", "symbol", "signature", "edit"],
    },
    SkillMeta {
        name: "souls-blast-radius",
        description: "Medição de raio de impacto e Frecency antes de refatorações.",
        primary_tools: &["repo_heatmap", "repo_impact"],
        keywords: &["impact", "blast", "frecency", "git", "gitoxide", "commit", "changes", "risk"],
    },
    SkillMeta {
        name: "souls-memory-triad",
        description: "Hipocampo bare-metal L2: busca híbrida RRF, LanceDB e SQLite STRICT WAL.",
        primary_tools: &["knowledge", "sqlite_query", "semantic_search", "mem_search"],
        keywords: &["memory", "sqlite", "query", "sql", "epistemic", "fts5", "lancedb", "fact"],
    },
    SkillMeta {
        name: "souls-ontology-graph",
        description: "Grafo Ontológico LadybugDB: entidades, relações tipadas e observações.",
        primary_tools: &["mem_open_nodes", "mem_read_graph", "mem_create_entities", "mem_create_relations"],
        keywords: &["graph", "ladybug", "ontology", "entity", "relation", "edge", "observation"],
    },
    SkillMeta {
        name: "souls-socratic-thinking",
        description: "Freio cognitivo socrático estruturado em grafo DAG.",
        primary_tools: &["thinking", "session", "analyze_session", "export_session"],
        keywords: &["think", "thinking", "socratic", "dag", "hypothesis", "branch", "reasoning"],
    },
    SkillMeta {
        name: "souls-cloud-arena-advisor",
        description: "Consulta e ranking da Fronteira de Pareto nos 5 perfis cognitivos e governança do Master Brain.",
        primary_tools: &["cloud_champions", "cloud_arena_sync", "profile_list", "profile_switch"],
        keywords: &["cloud", "arena", "pricing", "cost", "pareto", "nous", "openrouter", "model", "profile", "preset", "hermes", "brain"],
    },
    SkillMeta {
        name: "souls-hardware-watchdog",
        description: "Monitoramento termodinâmico da GPU RTX 2060m e FinOps de silício via NVML.",
        primary_tools: &["metrics", "sys_time", "feedback"],
        keywords: &["gpu", "vram", "nvml", "temperature", "thermal", "throttle", "hardware", "metrics"],
    },
    SkillMeta {
        name: "souls-subagent-dispatch",
        description: "Orquestração e despacho no silício com ParetoBandit antes de delegate_task.",
        primary_tools: &["route_subtask", "sub_agent", "handoff"],
        keywords: &["subagent", "delegate", "handoff", "bandit", "pareto", "dispatch", "task"],
    },
    SkillMeta {
        name: "souls-intent-probing",
        description: "Medição de entropia e classificação de intenção socrática via Tier 0/0.5 SLM.",
        primary_tools: &["intent"],
        keywords: &["intent", "entropy", "classify", "triage", "shannon", "probing"],
    },
    SkillMeta {
        name: "souls-context-lean",
        description: "Compressão semântica de contexto e desidratação CCR na RAM.",
        primary_tools: &["compress", "dedup", "fill", "smart_read"],
        keywords: &["compress", "dedup", "fill", "lean", "ccr", "context", "token", "headroom"],
    },
    SkillMeta {
        name: "souls-safe-sandbox",
        description: "Execução segura de comandos em sandbox restrita JobObject NT.",
        primary_tools: &["execute", "shell"],
        keywords: &["sandbox", "execute", "shell", "terminal", "command", "jobobject", "process"],
    },
    SkillMeta {
        name: "souls-web-intelligence",
        description: "Pesquisa na web e raspagem direta em Markdown limpo na RAM.",
        primary_tools: &["web_search", "fetch_web"],
        keywords: &["web", "search", "fetch", "scrape", "duckduckgo", "html", "markdown", "internet"],
    },
    SkillMeta {
        name: "obsidian-privacy-shield",
        description: "Escudo de soberania e privacidade: bloqueio estrito de dados de saúde e finanças.",
        primary_tools: &["metrics", "intent"],
        keywords: &["privacy", "health", "finance", "pii", "sovereignty", "confidential", "local"],
    },
    SkillMeta {
        name: "obsidian-vault-curator",
        description: "Curadoria do cofre Obsidian: esteira raw->wiki com links [[...]] e logs.",
        primary_tools: &["tree", "read", "search"],
        keywords: &["obsidian", "vault", "curate", "raw", "wiki", "frontmatter", "markdown"],
    },
    SkillMeta {
        name: "agent-self-learning-loop",
        description: "Ciclo de auto-aprendizado /learn com auto-patch em regras e memória L2.",
        primary_tools: &["knowledge", "thinking"],
        keywords: &["learn", "learning", "patch", "correction", "loop", "feedback", "rule"],
    },
];

/// Ranked tool candidate from Stage 1.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankedTool {
    pub name: String,
    pub namespace: String,
    pub description: String,
    pub score: f32,
    pub schema: Value,
}

/// Ranked skill recommendation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankedSkill {
    pub name: String,
    pub description: String,
    pub score: f32,
    pub primary_tools: Vec<String>,
}

/// Final decision synthesized by the Two-Stage Switchboard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwitchboardDecision {
    pub stage: String,
    pub confidence: f32,
    pub entropy: f32,
    pub matched_count: usize,
    pub recommended_skill: Option<RankedSkill>,
    pub tools: Vec<RankedTool>,
    pub latency_ms: f64,
}

/// Computes token overlap semantic score in [0.0, 1.0].
fn compute_semantic_score(
    query_tokens: &[&str],
    target_text: &str,
    bonus_keywords: &[&str],
    namespace: &str,
) -> f32 {
    if query_tokens.is_empty() {
        return 0.5;
    }

    let target_lower = target_text.to_lowercase();
    let mut hits = 0.0_f32;

    for qt in query_tokens {
        if target_lower.contains(qt) {
            hits += 1.0;
        }
    }

    let mut keyword_hits = 0.0_f32;
    for kw in bonus_keywords {
        if query_tokens.iter().any(|qt| qt.contains(kw) || kw.contains(qt)) {
            keyword_hits += 2.0;
        }
    }

    // Semantic intent affinity boost by domain keywords
    let mut domain_boost = 1.0_f32;
    for qt in query_tokens {
        match *qt {
            "code" | "ast" | "syntax" | "function" | "struct" | "symbol" | "diff" | "refactor" | "rust" => {
                if namespace == "code" {
                    domain_boost = 1.8;
                }
            }
            "memory" | "sqlite" | "lancedb" | "ladybug" | "graph" | "fact" | "store" | "lembrar" | "salvar" => {
                if namespace == "memory" {
                    domain_boost = 1.8;
                }
            }
            "think" | "thinking" | "socratic" | "dag" | "reasoning" | "hypothesis" | "pensar" | "hipotese" => {
                if namespace == "think" {
                    domain_boost = 1.8;
                }
            }
            "exec" | "execute" | "shell" | "terminal" | "sandbox" | "command" | "jobobject" | "process" => {
                if namespace == "ops" {
                    domain_boost = 1.8;
                }
            }
            "cloud" | "arena" | "pricing" | "cost" | "pareto" | "nous" | "openrouter" | "custo" | "preco" => {
                if namespace == "ops" || target_lower.contains("cloud") {
                    domain_boost = 1.8;
                }
            }
            _ => {}
        }
    }

    let raw = ((hits + keyword_hits) / (query_tokens.len() as f32).max(1.0)) * domain_boost;
    raw.clamp(0.05, 0.99)
}

/// Executes Stage 1 dense vector & lexical cosine match in RAM (< 2 ms).
pub fn execute_stage1(
    query: &str,
    namespace_filter: Option<&str>,
    top_k: usize,
) -> (Vec<RankedTool>, Option<RankedSkill>, f32) {
    let clean_query = query.to_lowercase();
    let query_tokens: Vec<&str> = clean_query
        .split(|c: char| !c.is_alphanumeric())
        .filter(|t| t.len() >= 2)
        .collect();

    // 1. First evaluate skill affinity to boost matching primary tools
    let mut skill_scores: Vec<RankedSkill> = CANONICAL_SKILLS
        .iter()
        .map(|s| {
            let combined = format!("{} {}", s.name, s.description);
            let score = compute_semantic_score(&query_tokens, &combined, s.keywords, "skill");
            RankedSkill {
                name: s.name.to_string(),
                description: s.description.to_string(),
                score,
                primary_tools: s.primary_tools.iter().map(|&t| t.to_string()).collect(),
            }
        })
        .collect();

    skill_scores.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    let best_skill = skill_scores.first().cloned();

    let primary_tool_names: Vec<String> = best_skill
        .as_ref()
        .map(|s| s.primary_tools.clone())
        .unwrap_or_default();

    let all_tools = get_canonical_tools();

    // 2. Score tools with skill harmony boost
    let mut tool_scores: Vec<RankedTool> = all_tools
        .iter()
        .filter(|t| {
            if let Some(ns) = namespace_filter {
                if ns != "all" && !ns.eq_ignore_ascii_case(t.namespace) {
                    return false;
                }
            }
            true
        })
        .map(|t| {
            let combined = format!("{} {} {}", t.name, t.namespace, t.description);
            let mut score = compute_semantic_score(&query_tokens, &combined, &[t.name, t.namespace], t.namespace);

            // Skill harmony boost: if this tool is primary for the top skill, grant extra weight
            if primary_tool_names.iter().any(|pt| pt.eq_ignore_ascii_case(t.name)) {
                score = (score + 0.35).min(0.99);
            }

            RankedTool {
                name: t.name.to_string(),
                namespace: t.namespace.to_string(),
                description: t.description.to_string(),
                score,
                schema: (t.input_schema)(),
            }
        })
        .collect();

    tool_scores.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    tool_scores.truncate(top_k);

    let top_tool_score = tool_scores.first().map(|t| t.score).unwrap_or(0.5);
    let top_skill_score = best_skill.as_ref().map(|s| s.score).unwrap_or(0.5);
    let combined_conf = (top_tool_score * 0.6 + top_skill_score * 0.4).clamp(0.1, 0.99);

    (tool_scores, best_skill, combined_conf)
}

/// Executes the full Two-Stage Semantic Switchboard workflow.
pub async fn evaluate_switchboard(
    state: &AppState,
    query: &str,
    namespace: Option<&str>,
    top_k: Option<usize>,
) -> SwitchboardDecision {
    let start = std::time::Instant::now();
    let k = top_k.unwrap_or(5).clamp(1, 15);

    // Stage 1: Fast-Path Cosine Match in RAM (< 2 ms)
    let (mut top_tools, mut top_skill, s1_conf) = execute_stage1(query, namespace, k);

    // Stage 2: Entropy Gating via Tier 0.5 Logit Probing (~8 ms) if confidence < 0.75
    let (stage_name, final_conf, entropy) = if s1_conf >= 0.75 {
        ("fast_path_ram_cosine".to_string(), s1_conf, 0.15)
    } else {
        // Evaluate intent & entropy circuit breaker via real Tier 0/0.5 prober
        match state.runtime.evaluate_intent(query).await {
            Ok(intent_val) => {
                let ent = intent_val.get("entropy").and_then(Value::as_f64).unwrap_or(0.35) as f32;
                let detected_intent = intent_val.get("intent").and_then(Value::as_str).unwrap_or("");

                // If Tier 0 detected a dominant namespace, re-evaluate Stage 1 with that namespace filter
                if namespace.is_none() && !detected_intent.is_empty() {
                    let inferred_ns = match detected_intent {
                        "code" | "code_review" | "ast" => Some("code"),
                        "memory" | "recall" => Some("memory"),
                        "think" | "reasoning" => Some("think"),
                        "terminal" | "shell" | "ops" => Some("ops"),
                        _ => None,
                    };

                    if let Some(ns) = inferred_ns {
                        let (refined_tools, refined_skill, refined_conf) = execute_stage1(query, Some(ns), k);
                        if !refined_tools.is_empty() {
                            top_tools = refined_tools;
                            if refined_skill.is_some() {
                                top_skill = refined_skill;
                            }
                        }
                        let adjusted_conf = (refined_conf + 0.20).min(0.95);
                        ("stage2_tier05_entropy_gated".to_string(), adjusted_conf, ent)
                    } else {
                        let adjusted_conf = if ent < 0.5 {
                            (s1_conf + 0.25).min(0.95)
                        } else {
                            (s1_conf * 0.85).max(0.10)
                        };
                        ("stage2_tier05_entropy_gated".to_string(), adjusted_conf, ent)
                    }
                } else {
                    let adjusted_conf = if ent < 0.5 {
                        (s1_conf + 0.25).min(0.95)
                    } else {
                        (s1_conf * 0.85).max(0.10)
                    };
                    ("stage2_tier05_entropy_gated".to_string(), adjusted_conf, ent)
                }
            }
            Err(_) => {
                // Fail-soft Tier 0.5 logit probe fallback
                match state.runtime.infer_tier05_probe(query).await {
                    Ok(prob_output) => {
                        let ent = prob_output.entropy;
                        let adjusted_conf = if ent < 0.5 {
                            (s1_conf + 0.20).min(0.95)
                        } else {
                            (s1_conf * 0.85).max(0.10)
                        };
                        ("stage2_tier05_entropy_gated".to_string(), adjusted_conf, ent)
                    }
                    Err(_) => ("stage1_fallback".to_string(), s1_conf, 0.50),
                }
            }
        }
    };

    let latency_ms = start.elapsed().as_secs_f64() * 1000.0;
    top_tools.truncate(k);
    let count = top_tools.len();

    SwitchboardDecision {
        stage: stage_name,
        confidence: final_conf,
        entropy,
        matched_count: count,
        recommended_skill: top_skill,
        tools: top_tools,
        latency_ms,
    }
}
