//! # domain.rs — Entidades de Domínio da Cloud Arena
//!
//! Modelagem de dados para modelos canônicos, snapshots diários de precificação,
//! telemetria de provedores, benchmarks cognitivos e avaliações de Pareto nos 9 perfis (ADR-009).

use serde::{Deserialize, Serialize};

/// Perfil de carga computacional e afinidade cognitiva (Taxonomia Canônica ADR-009).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskProfileKind {
    Executive,
    Coding,
    Agentic,
    Reasoning,
    Fast,
    Design,
    Research,
    Security,
    Curation,
}

impl TaskProfileKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Executive => "executive",
            Self::Coding => "coding",
            Self::Agentic => "agentic",
            Self::Reasoning => "reasoning",
            Self::Fast => "fast",
            Self::Design => "design",
            Self::Research => "research",
            Self::Security => "security",
            Self::Curation => "curation",
        }
    }

    /// Converte strings e aliases legados (ex: "orchestrator", "maestro", "brain") para o perfil canônico.
    pub fn from_str_lossy(raw: &str) -> Option<Self> {
        match raw.trim().to_lowercase().as_str() {
            "executive" | "orchestrator" | "maestro" | "brain" | "master_brain" => Some(Self::Executive),
            "coding" | "swe" | "code" => Some(Self::Coding),
            "agentic" | "tools" | "tool_calling" => Some(Self::Agentic),
            "reasoning" | "math" | "logic" => Some(Self::Reasoning),
            "fast" | "triage" | "quick" => Some(Self::Fast),
            "design" | "frontend" | "vision" | "ui" => Some(Self::Design),
            "research" | "rag" | "synthesis" => Some(Self::Research),
            "security" | "audit" | "sec" => Some(Self::Security),
            "curation" | "writing" | "wiki" | "obsidian" => Some(Self::Curation),
            _ => None,
        }
    }

    pub fn all() -> &'static [TaskProfileKind] {
        &[
            Self::Executive,
            Self::Coding,
            Self::Agentic,
            Self::Reasoning,
            Self::Fast,
            Self::Design,
            Self::Research,
            Self::Security,
            Self::Curation,
        ]
    }
}

impl std::fmt::Display for TaskProfileKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Classificação determinística na Fronteira de Pareto.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParetoTag {
    Leader,
    BestValue,
    UltraLowCost,
    Dominated,
    Estimated,
}

impl ParetoTag {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Leader => "Leader",
            Self::BestValue => "BestValue",
            Self::UltraLowCost => "UltraLowCost",
            Self::Dominated => "Dominated",
            Self::Estimated => "Estimated",
        }
    }
}

/// Entidade normalizada de um modelo de linguagem em nuvem.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalModel {
    pub id: String,
    pub display_name: String,
    pub family: String,
    pub developer: String,
    pub context_window: i64,
    pub supports_tools: bool,
    pub supports_structured_outputs: bool,
    pub supports_caching: bool,
    pub is_multimodal: bool,
    pub created_at: String,
}

impl CanonicalModel {
    /// Verifica se o modelo é um LLM gerador de texto/chat (exclui embeddings, áudio puro, geração de imagem e modelos sem janela útil).
    pub fn is_generative_llm(&self) -> bool {
        if self.context_window < 4096 {
            return false;
        }
        let lower = self.id.to_lowercase();
        let blocked_tokens = [
            "embed",
            "minilm",
            "bge-",
            "e5-",
            "rerank",
            "whisper",
            "tts",
            "moderation",
            "flux",
            "dall-e",
            "stable-diffusion",
            "imagen",
            "sora",
            "kling",
        ];
        !blocked_tokens.iter().any(|tok| lower.contains(tok))
    }

    /// Verifica elegibilidade técnica estrita do modelo para disputar o Pódio em determinado perfil cognitivo.
    pub fn is_eligible_for_profile(&self, profile: TaskProfileKind) -> bool {
        if !self.is_generative_llm() {
            return false;
        }
        match profile {
            TaskProfileKind::Executive => self.supports_tools && self.context_window >= 64_000,
            TaskProfileKind::Agentic => self.supports_tools,
            TaskProfileKind::Design => self.is_multimodal,
            TaskProfileKind::Research => self.context_window >= 200_000,
            TaskProfileKind::Coding
            | TaskProfileKind::Reasoning
            | TaskProfileKind::Security
            | TaskProfileKind::Curation => self.context_window >= 32_000,
            TaskProfileKind::Fast => self.context_window >= 8_192,
        }
    }
}

/// Snapshot pontual de preços e métricas operacionais por provedor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingTelemetrySnapshot {
    pub canonical_id: String,
    pub source: String,
    pub prompt_price_per_m: f64,
    pub completion_price_per_m: f64,
    pub cache_read_price_per_m: f64,
    pub cache_write_price_per_m: f64,
    pub ttft_ms: Option<f64>,
    pub throughput_tps: Option<f64>,
    pub recorded_at: String,
}

/// Snapshot pontual de benchmarks cognitivos consolidados.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkSnapshot {
    pub canonical_id: String,
    pub general_elo: Option<f64>,
    pub coding_elo: Option<f64>,
    pub hard_prompts_elo: Option<f64>,
    pub bfcl_score: Option<f64>,
    pub ifeval_score: Option<f64>,
    pub gpqa_score: Option<f64>,
    pub vqa_multimodal_score: Option<f64>,
    pub recorded_at: String,
}

/// Avaliação diária consolidada com notas ponderadas e Fronteira de Pareto.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyEvaluation {
    pub canonical_id: String,
    pub evaluation_date: String,
    pub profile_task: TaskProfileKind,
    pub composite_score: f64,
    pub task_blended_cost: f64,
    pub roi_index: f64,
    pub is_pareto_optimal: bool,
    pub pareto_tag: ParetoTag,
}

/// Relatório de campeões em um determinado perfil cognitivo.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChampionEntry {
    pub canonical_id: String,
    pub display_name: String,
    pub provider: String,
    pub composite_score: f64,
    pub blended_cost: f64,
    pub roi_index: f64,
    pub throughput_tps: Option<f64>,
    pub context_window: i64,
}

/// Campeões consolidados de um perfil.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileChampionReport {
    pub profile: TaskProfileKind,
    pub quality_leader: Option<ChampionEntry>,
    pub best_value: Option<ChampionEntry>,
    pub ultra_low_cost: Option<ChampionEntry>,
}

/// Relatório executivo do sincronizador.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncOutcome {
    pub date: String,
    pub models_synced: usize,
    pub pricing_snapshots: usize,
    pub benchmark_snapshots: usize,
    pub evaluations_calculated: usize,
    pub from_cached_fallback: bool,
    pub warning: Option<String>,
}
