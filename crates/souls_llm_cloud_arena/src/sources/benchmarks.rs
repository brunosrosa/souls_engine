//! # benchmarks.rs — Base de Benchmarks Cognitivos & LMSYS Chatbot Arena
//!
//! Integração de avaliações empíricas consolidadas:
//! 1. LMSYS Chatbot Arena Leaderboard oficial (via Hugging Face Datasets API)
//! 2. Benchmarks consolidados locais de fallback (LMSYS Elo, BFCL, IFEval, GPQA).

use crate::canonical::CanonicalResolver;
use crate::domain::BenchmarkSnapshot;
use crate::error::CloudArenaError;
use serde::Deserialize;
use std::time::Duration;
use tracing::{info, warn};

pub struct BenchmarkSource;

#[derive(Debug, Deserialize)]
struct HfLmsysResponse {
    rows: Option<Vec<HfLmsysRowWrapper>>,
}

#[derive(Debug, Deserialize)]
struct HfLmsysRowWrapper {
    row: HfLmsysRow,
}

#[derive(Debug, Deserialize)]
struct HfLmsysRow {
    model_name: String,
    organization: Option<String>,
    rating: Option<f64>,
    #[serde(rename = "rank")]
    _rank: Option<f64>,
}

impl BenchmarkSource {
    /// Consulta o dataset oficial da LMSYS Chatbot Arena via Hugging Face API
    /// e converte as avaliações humanas em snapshots de benchmark com Elo rating real.
    pub async fn fetch_lmsys_arena(
        resolver: &CanonicalResolver,
        recorded_at: &str,
    ) -> Result<Vec<BenchmarkSnapshot>, CloudArenaError> {
        let url = "https://datasets-server.huggingface.co/rows?dataset=lmarena-ai/leaderboard-dataset&config=text&split=latest&offset=0&limit=100";
        info!("Consultando LMSYS Chatbot Arena via Hugging Face API...");

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap_or_default();

        let resp_res = client
            .get(url)
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) SoulsEngine/v7")
            .send()
            .await;

        match resp_res {
            Ok(resp) if resp.status().is_success() => {
                let payload: HfLmsysResponse = resp.json().await?;
                let mut snapshots = Vec::new();

                if let Some(rows) = payload.rows {
                    for r in rows {
                        let row = r.row;
                        let rating = match row.rating {
                            Some(r) => r,
                            None => continue,
                        };

                        // Tenta resolver por "org/model_name" ou diretamente por "model_name"
                        let mut resolved = resolver.try_resolve(&row.model_name);
                        if resolved.is_none() {
                            if let Some(ref org) = row.organization {
                                resolved = resolver.try_resolve(&format!("{}/{}", org, row.model_name));
                            }
                        }

                        if let Some(canonical_id) = resolved {
                            snapshots.push(BenchmarkSnapshot {
                                canonical_id,
                                general_elo: Some(rating),
                                coding_elo: Some(rating),
                                hard_prompts_elo: Some(rating),
                                bfcl_score: None,
                                ifeval_score: None,
                                gpqa_score: None,
                                vqa_multimodal_score: None,
                                recorded_at: recorded_at.to_string(),
                            });
                        }
                    }
                }

                info!(
                    "LMSYS Chatbot Arena ingerido com sucesso: {} modelos mapeados.",
                    snapshots.len()
                );
                Ok(snapshots)
            }
            Ok(resp) => {
                warn!("Hugging Face API retornou status HTTP {}: utilizando seed de fallback.", resp.status());
                Self::load_snapshots(resolver, recorded_at)
            }
            Err(e) => {
                warn!("Falha ao consultar LMSYS Arena no Hugging Face ({}): utilizando seed de fallback.", e);
                Self::load_snapshots(resolver, recorded_at)
            }
        }
    }

    /// Carrega snapshots de benchmarks cognitivos consolidados locais (fallback gracioso).
    pub fn load_snapshots(
        resolver: &CanonicalResolver,
        recorded_at: &str,
    ) -> Result<Vec<BenchmarkSnapshot>, CloudArenaError> {
        let raw_dataset = get_embedded_benchmark_seed();
        let mut snapshots = Vec::new();

        for item in raw_dataset {
            if let Some(canonical_id) = resolver.try_resolve(item.raw_id) {
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
            raw_id: "anthropic/claude-opus-5.5",
            general_elo: 1515.5,
            coding_elo: 1520.0,
            hard_prompts_elo: 1530.0,
            bfcl_score: 97.0,
            ifeval_score: 95.0,
            gpqa_score: 82.0,
            vqa_multimodal_score: 95.0,
        },
        RawBenchmarkSeed {
            raw_id: "anthropic/claude-sonnet-5.5",
            general_elo: 1470.6,
            coding_elo: 1490.0,
            hard_prompts_elo: 1475.0,
            bfcl_score: 95.5,
            ifeval_score: 93.8,
            gpqa_score: 74.2,
            vqa_multimodal_score: 94.0,
        },
        RawBenchmarkSeed {
            raw_id: "google/gemini-4-argon",
            general_elo: 1533.2,
            coding_elo: 1510.0,
            hard_prompts_elo: 1520.0,
            bfcl_score: 96.0,
            ifeval_score: 94.0,
            gpqa_score: 79.0,
            vqa_multimodal_score: 96.0,
        },
        RawBenchmarkSeed {
            raw_id: "openai/gpt-6-sol",
            general_elo: 1480.0,
            coding_elo: 1495.0,
            hard_prompts_elo: 1485.0,
            bfcl_score: 94.5,
            ifeval_score: 93.0,
            gpqa_score: 76.0,
            vqa_multimodal_score: 92.0,
        },
        RawBenchmarkSeed {
            raw_id: "deepseek/deepseek-v4.1-flash",
            general_elo: 1462.5,
            coding_elo: 1460.0,
            hard_prompts_elo: 1450.0,
            bfcl_score: 91.0,
            ifeval_score: 89.0,
            gpqa_score: 71.0,
            vqa_multimodal_score: 88.0,
        },
        RawBenchmarkSeed {
            raw_id: "z-ai/glm-5.3-prime",
            general_elo: 1471.4,
            coding_elo: 1470.0,
            hard_prompts_elo: 1465.0,
            bfcl_score: 92.0,
            ifeval_score: 90.0,
            gpqa_score: 72.0,
            vqa_multimodal_score: 89.0,
        },
        RawBenchmarkSeed {
            raw_id: "qwen/qwen3.8-max-prime",
            general_elo: 1482.5,
            coding_elo: 1485.0,
            hard_prompts_elo: 1475.0,
            bfcl_score: 93.0,
            ifeval_score: 91.0,
            gpqa_score: 73.0,
            vqa_multimodal_score: 90.0,
        },
    ]
}
