//! # profiles.rs — Matriz Ponderada dos 5 Perfis Cognitivos
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
    }
}
