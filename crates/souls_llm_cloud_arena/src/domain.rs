//! # domain.rs — Entidades de Domínio da Cloud Arena
//!
//! Modelagem de dados para modelos canônicos, snapshots diários de precificação,
//! telemetria de provedores, benchmarks cognitivos e avaliações de Pareto nos 5 perfis.

use serde::{Deserialize, Serialize};

/// Perfil de carga computacional e afinidade cognitiva.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskProfileKind {
    Coding,
    Agentic,
    Reasoning,
    Fast,
    Design,
}

impl TaskProfileKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Coding => "coding",
            Self::Agentic => "agentic",
            Self::Reasoning => "reasoning",
            Self::Fast => "fast",
            Self::Design => "design",
        }
    }

    pub fn all() -> &'static [TaskProfileKind] {
        &[
            Self::Coding,
            Self::Agentic,
            Self::Reasoning,
            Self::Fast,
            Self::Design,
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
}

impl ParetoTag {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Leader => "Leader",
            Self::BestValue => "BestValue",
            Self::UltraLowCost => "UltraLowCost",
            Self::Dominated => "Dominated",
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
