//! # profiles.rs — Matriz Ponderada dos 9 Perfis Cognitivos (ADR-009)
//!
//! Pesos normalizados de benchmarks cognitivos e estrutura de custos por perfil de tarefa.

use crate::domain::TaskProfileKind;

#[derive(Debug, Clone, Copy)]
pub struct CostWeights {
    pub w_cache: f64,
    pub w_in: f64,
    pub w_out: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct BenchmarkWeights {
    pub coding_elo: f64,
    pub hard_elo: f64,
    pub bfcl: f64,
    pub ifeval: f64,
    pub gpqa: f64,
    pub vqa: f64,
    pub tps: f64,
    pub ttft: f64,
}

pub fn get_profile_weights(profile: TaskProfileKind) -> (BenchmarkWeights, CostWeights) {
    match profile {
        TaskProfileKind::Executive => (
            BenchmarkWeights {
                coding_elo: 0.10,
                hard_elo: 0.30,
                bfcl: 0.20,
                ifeval: 0.25,
                gpqa: 0.15,
                vqa: 0.0,
                tps: 0.0,
                ttft: 0.0,
            },
            CostWeights {
                w_cache: 0.50,
                w_in: 0.30,
                w_out: 0.20,
            },
        ),
        TaskProfileKind::Coding => (
            BenchmarkWeights {
                coding_elo: 0.40,
                hard_elo: 0.20,
                bfcl: 0.25,
                ifeval: 0.15,
                gpqa: 0.0,
                vqa: 0.0,
                tps: 0.0,
                ttft: 0.0,
            },
            CostWeights {
                w_cache: 0.40,
                w_in: 0.40,
                w_out: 0.20,
            },
        ),
        TaskProfileKind::Agentic => (
            BenchmarkWeights {
                coding_elo: 0.0,
                hard_elo: 0.15,
                bfcl: 0.45,
                ifeval: 0.30,
                gpqa: 0.0,
                vqa: 0.0,
                tps: 0.10,
                ttft: 0.0,
            },
            CostWeights {
                w_cache: 0.60,
                w_in: 0.30,
                w_out: 0.10,
            },
        ),
        TaskProfileKind::Reasoning => (
            BenchmarkWeights {
                coding_elo: 0.0,
                hard_elo: 0.35,
                bfcl: 0.0,
                ifeval: 0.20,
                gpqa: 0.45,
                vqa: 0.0,
                tps: 0.0,
                ttft: 0.0,
            },
            CostWeights {
                w_cache: 0.20,
                w_in: 0.40,
                w_out: 0.40,
            },
        ),
        TaskProfileKind::Fast => (
            BenchmarkWeights {
                coding_elo: 0.15,
                hard_elo: 0.0,
                bfcl: 0.0,
                ifeval: 0.15,
                gpqa: 0.0,
                vqa: 0.0,
                tps: 0.40,
                ttft: 0.30,
            },
            CostWeights {
                w_cache: 0.10,
                w_in: 0.80,
                w_out: 0.10,
            },
        ),
        TaskProfileKind::Design => (
            BenchmarkWeights {
                coding_elo: 0.40,
                hard_elo: 0.0,
                bfcl: 0.0,
                ifeval: 0.15,
                gpqa: 0.0,
                vqa: 0.30,
                tps: 0.15,
                ttft: 0.0,
            },
            CostWeights {
                w_cache: 0.30,
                w_in: 0.40,
                w_out: 0.30,
            },
        ),
        TaskProfileKind::Research => (
            BenchmarkWeights {
                coding_elo: 0.0,
                hard_elo: 0.35,
                bfcl: 0.0,
                ifeval: 0.30,
                gpqa: 0.25,
                vqa: 0.0,
                tps: 0.10,
                ttft: 0.0,
            },
            CostWeights {
                w_cache: 0.65,
                w_in: 0.25,
                w_out: 0.10,
            },
        ),
        TaskProfileKind::Security => (
            BenchmarkWeights {
                coding_elo: 0.35,
                hard_elo: 0.20,
                bfcl: 0.0,
                ifeval: 0.10,
                gpqa: 0.35,
                vqa: 0.0,
                tps: 0.0,
                ttft: 0.0,
            },
            CostWeights {
                w_cache: 0.40,
                w_in: 0.40,
                w_out: 0.20,
            },
        ),
        TaskProfileKind::Curation => (
            BenchmarkWeights {
                coding_elo: 0.0,
                hard_elo: 0.35,
                bfcl: 0.15,
                ifeval: 0.35,
                gpqa: 0.0,
                vqa: 0.0,
                tps: 0.15,
                ttft: 0.0,
            },
            CostWeights {
                w_cache: 0.45,
                w_in: 0.35,
                w_out: 0.20,
            },
        ),
    }
}
