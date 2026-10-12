//! # benchmarks.rs — Base de Benchmarks Cognitivos
//!
//! Integração de avaliações empíricas consolidadas (LMSYS Arena Elo,
//! Berkeley Function Calling Leaderboard, SWE-bench, IFEval, GPQA e VQA).

use crate::canonical::CanonicalResolver;
use crate::domain::BenchmarkSnapshot;
use crate::error::CloudArenaError;

pub struct BenchmarkSource;

impl BenchmarkSource {
    /// Carrega snapshots de benchmarks cognitivos consolidados com resolução canônica.
    pub fn load_snapshots(
        resolver: &CanonicalResolver,
        recorded_at: &str,
    ) -> Result<Vec<BenchmarkSnapshot>, CloudArenaError> {
        let raw_dataset = get_embedded_benchmark_seed();
        let mut snapshots = Vec::new();

        for item in raw_dataset {
            let canonical_id = resolver.resolve(&item.raw_id);
            snapshots.push(BenchmarkSnapshot {
                canonical_id,
                general_elo: Some(item.general_elo),
                coding_elo: Some(item.coding_elo),
                hard_prompts_elo: Some(item.hard_prompts_elo),
                bfcl_score: Some(item.bfcl_score),
                ifeval_score: Some(item.ifeval_score),
                gpqa_score: Some(item.gpqa_score),
                vqa_multimodal_score: Some(item.vqa_multimodal_score),
                recorded_at: recorded_at.to_string(),
            });
        }

        Ok(snapshots)
    }
}

struct RawBenchmarkSeed {
    raw_id: &'static str,
    general_elo: f64,
    coding_elo: f64,
    hard_prompts_elo: f64,
    bfcl_score: f64,
    ifeval_score: f64,
    gpqa_score: f64,
    vqa_multimodal_score: f64,
}

fn get_embedded_benchmark_seed() -> Vec<RawBenchmarkSeed> {
    vec![
        RawBenchmarkSeed {
            raw_id: "claude-3-7-sonnet",
            general_elo: 1380.0,
            coding_elo: 1420.0,
            hard_prompts_elo: 1395.0,
            bfcl_score: 95.5,
            ifeval_score: 93.8,
            gpqa_score: 74.2,
            vqa_multimodal_score: 94.0,
        },
        RawBenchmarkSeed {
            raw_id: "claude-3-5-sonnet",
            general_elo: 1360.0,
            coding_elo: 1400.0,
            hard_prompts_elo: 1375.0,
            bfcl_score: 93.0,
            ifeval_score: 91.5,
            gpqa_score: 70.0,
            vqa_multimodal_score: 92.5,
        },
        RawBenchmarkSeed {
            raw_id: "deepseek-v3",
            general_elo: 1340.0,
            coding_elo: 1365.0,
            hard_prompts_elo: 1330.0,
            bfcl_score: 89.2,
            ifeval_score: 87.5,
            gpqa_score: 65.0,
            vqa_multimodal_score: 75.0,
        },
        RawBenchmarkSeed {
            raw_id: "deepseek-r1",
            general_elo: 1375.0,
            coding_elo: 1390.0,
            hard_prompts_elo: 1425.0,
            bfcl_score: 86.0,
            ifeval_score: 88.0,
            gpqa_score: 76.5,
            vqa_multimodal_score: 70.0,
        },
        RawBenchmarkSeed {
            raw_id: "qwen-2.5-coder-32b",
            general_elo: 1300.0,
            coding_elo: 1360.0,
            hard_prompts_elo: 1290.0,
            bfcl_score: 84.5,
            ifeval_score: 83.0,
            gpqa_score: 58.0,
            vqa_multimodal_score: 65.0,
        },
        RawBenchmarkSeed {
            raw_id: "gpt-4o",
            general_elo: 1350.0,
            coding_elo: 1360.0,
            hard_prompts_elo: 1340.0,
            bfcl_score: 91.0,
            ifeval_score: 88.5,
            gpqa_score: 64.0,
            vqa_multimodal_score: 90.0,
        },
        RawBenchmarkSeed {
            raw_id: "gpt-4o-mini",
            general_elo: 1280.0,
            coding_elo: 1290.0,
            hard_prompts_elo: 1260.0,
            bfcl_score: 82.0,
            ifeval_score: 81.0,
            gpqa_score: 48.0,
            vqa_multimodal_score: 84.0,
        },
        RawBenchmarkSeed {
            raw_id: "o3-mini",
            general_elo: 1365.0,
            coding_elo: 1385.0,
            hard_prompts_elo: 1390.0,
            bfcl_score: 88.0,
            ifeval_score: 89.0,
            gpqa_score: 75.0,
            vqa_multimodal_score: 60.0,
        },
        RawBenchmarkSeed {
            raw_id: "llama-3.3-70b",
            general_elo: 1320.0,
            coding_elo: 1315.0,
            hard_prompts_elo: 1305.0,
            bfcl_score: 85.0,
            ifeval_score: 86.0,
            gpqa_score: 56.0,
            vqa_multimodal_score: 60.0,
        },
        RawBenchmarkSeed {
            raw_id: "hermes-3-llama-3.1-405b",
            general_elo: 1335.0,
            coding_elo: 1330.0,
            hard_prompts_elo: 1320.0,
            bfcl_score: 88.0,
            ifeval_score: 87.0,
            gpqa_score: 61.0,
            vqa_multimodal_score: 60.0,
        },
        RawBenchmarkSeed {
            raw_id: "hermes-3-llama-3.1-70b",
            general_elo: 1300.0,
            coding_elo: 1295.0,
            hard_prompts_elo: 1280.0,
            bfcl_score: 83.0,
            ifeval_score: 84.0,
            gpqa_score: 52.0,
            vqa_multimodal_score: 60.0,
        },
        RawBenchmarkSeed {
            raw_id: "gemini-2.0-flash",
            general_elo: 1345.0,
            coding_elo: 1350.0,
            hard_prompts_elo: 1335.0,
            bfcl_score: 89.5,
            ifeval_score: 89.0,
            gpqa_score: 65.5,
            vqa_multimodal_score: 93.0,
        },
    ]
}
