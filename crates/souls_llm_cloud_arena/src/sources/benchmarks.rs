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
                    let mut best_by_id: std::collections::HashMap<String, f64> =
                        std::collections::HashMap::new();

                    for r in rows {
                        let row = r.row;
                        let rating = match row.rating {
                            Some(r) => r,
                            None => continue,
                        };

                        let resolved = resolve_lmsys_row(
                            resolver,
                            &row.model_name,
                            row.organization.as_deref(),
                        );

                        if let Some(canonical_id) = resolved {
                            best_by_id
                                .entry(canonical_id)
                                .and_modify(|prev| {
                                    if rating > *prev {
                                        *prev = rating;
                                    }
                                })
                                .or_insert(rating);
                        }
                    }

                    for (canonical_id, rating) in best_by_id {
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
                    vqa_multimodal_score: if item.vqa_multimodal_score > 0.0 {
                        Some(item.vqa_multimodal_score)
                    } else {
                        None
                    },
                    recorded_at: recorded_at.to_string(),
                });
            }
        }

        Ok(snapshots)
    }
}

/// Normaliza slugs do LMSYS Chatbot Arena (espaços, sufixos `(High)`/`(Max)`, prefixos de organização).
fn resolve_lmsys_row(
    resolver: &CanonicalResolver,
    model_name: &str,
    org: Option<&str>,
) -> Option<String> {
    let trimmed = model_name.trim();
    if let Some(id) = resolver.try_resolve(trimmed) {
        return Some(id);
    }

    let mapped_org: Option<String> = org.map(|o| {
        let low = o.trim().to_lowercase();
        match low.as_str() {
            "alibaba" => "qwen".to_string(),
            "zai" => "z-ai".to_string(),
            "xai" => "x-ai".to_string(),
            "moonshot" => "moonshotai".to_string(),
            "thinky" => "thinkingmachines".to_string(),
            "mistral" => "mistralai".to_string(),
            _ => low,
        }
    });

    if let Some(ref o) = mapped_org {
        if let Some(id) = resolver.try_resolve(&format!("{}/{}", o, trimmed)) {
            return Some(id);
        }
    }

    // Remove sufixo entre parênteses no final, ex: "Gemini 4 Carbon (High)" -> "Gemini 4 Carbon"
    let base_name = if let Some(idx) = trimmed.rfind(" (") {
        trimmed[..idx].trim()
    } else {
        trimmed
    };

    let hyphenated = base_name.to_lowercase().replace(' ', "-");
    if let Some(id) = resolver.try_resolve(&hyphenated) {
        return Some(id);
    }
    if let Some(ref o) = mapped_org {
        if let Some(id) = resolver.try_resolve(&format!("{}/{}", o, hyphenated)) {
            return Some(id);
        }
    }

    None
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
        // Fronteira Suprema (Tier Frontier)
        RawBenchmarkSeed {
            raw_id: "anthropic/claude-fable-5.2",
            general_elo: 64.6,
            coding_elo: 82.0,
            hard_prompts_elo: 1525.0,
            bfcl_score: 94.0,
            ifeval_score: 95.0,
            gpqa_score: 88.5,
            vqa_multimodal_score: 94.5,
        },
        RawBenchmarkSeed {
            raw_id: "google/gemini-4-carbon",
            general_elo: 60.0,
            coding_elo: 80.5,
            hard_prompts_elo: 1538.4,
            bfcl_score: 93.0,
            ifeval_score: 94.5,
            gpqa_score: 87.5,
            vqa_multimodal_score: 96.5,
        },
        RawBenchmarkSeed {
            raw_id: "anthropic/claude-opus-5.5",
            general_elo: 59.1,
            coding_elo: 81.0,
            hard_prompts_elo: 1513.7,
            bfcl_score: 95.0,
            ifeval_score: 95.0,
            gpqa_score: 86.5,
            vqa_multimodal_score: 94.5,
        },
        RawBenchmarkSeed {
            raw_id: "anthropic/claude-fable-5.1",
            general_elo: 53.4,
            coding_elo: 81.6,
            hard_prompts_elo: 1512.0,
            bfcl_score: 57.9,
            ifeval_score: 93.5,
            gpqa_score: 85.5,
            vqa_multimodal_score: 92.0,
        },
        RawBenchmarkSeed {
            raw_id: "openai/gpt-6.2-astra",
            general_elo: 58.0,
            coding_elo: 78.5,
            hard_prompts_elo: 1510.0,
            bfcl_score: 93.0,
            ifeval_score: 94.0,
            gpqa_score: 87.0,
            vqa_multimodal_score: 94.0,
        },
        RawBenchmarkSeed {
            raw_id: "openai/gpt-6-astra",
            general_elo: 56.9,
            coding_elo: 76.9,
            hard_prompts_elo: 56.9,
            bfcl_score: 51.0,
            ifeval_score: 93.0,
            gpqa_score: 86.0,
            vqa_multimodal_score: 93.5,
        },
        RawBenchmarkSeed {
            raw_id: "openai/gpt-6.1-sol",
            general_elo: 54.6,
            coding_elo: 77.5,
            hard_prompts_elo: 1485.0,
            bfcl_score: 92.5,
            ifeval_score: 93.0,
            gpqa_score: 82.5,
            vqa_multimodal_score: 93.0,
        },
        RawBenchmarkSeed {
            raw_id: "openai/gpt-6.1-sol-pro",
            general_elo: 54.8,
            coding_elo: 78.0,
            hard_prompts_elo: 1490.0,
            bfcl_score: 93.0,
            ifeval_score: 93.2,
            gpqa_score: 83.5,
            vqa_multimodal_score: 93.0,
        },
        RawBenchmarkSeed {
            raw_id: "openai/gpt-6-sol",
            general_elo: 44.2,
            coding_elo: 74.5,
            hard_prompts_elo: 1460.0,
            bfcl_score: 90.0,
            ifeval_score: 91.5,
            gpqa_score: 77.5,
            vqa_multimodal_score: 91.0,
        },
        RawBenchmarkSeed {
            raw_id: "anthropic/claude-sonnet-5.5",
            general_elo: 54.5,
            coding_elo: 78.5,
            hard_prompts_elo: 1469.1,
            bfcl_score: 94.0,
            ifeval_score: 93.8,
            gpqa_score: 80.5,
            vqa_multimodal_score: 94.0,
        },
        RawBenchmarkSeed {
            raw_id: "google/gemini-4-hydrogen",
            general_elo: 53.9,
            coding_elo: 75.5,
            hard_prompts_elo: 1502.1,
            bfcl_score: 90.5,
            ifeval_score: 92.0,
            gpqa_score: 81.0,
            vqa_multimodal_score: 94.0,
        },
        RawBenchmarkSeed {
            raw_id: "google/gemini-4-argon",
            general_elo: 50.4,
            coding_elo: 76.5,
            hard_prompts_elo: 1529.5,
            bfcl_score: 92.0,
            ifeval_score: 93.5,
            gpqa_score: 83.5,
            vqa_multimodal_score: 96.0,
        },
        RawBenchmarkSeed {
            raw_id: "openai/gpt-6.1-terra",
            general_elo: 51.3,
            coding_elo: 74.0,
            hard_prompts_elo: 1465.0,
            bfcl_score: 90.0,
            ifeval_score: 91.5,
            gpqa_score: 79.5,
            vqa_multimodal_score: 91.0,
        },
        RawBenchmarkSeed {
            raw_id: "anthropic/claude-opus-5",
            general_elo: 44.2,
            coding_elo: 78.0,
            hard_prompts_elo: 1506.5,
            bfcl_score: 56.5,
            ifeval_score: 93.0,
            gpqa_score: 82.0,
            vqa_multimodal_score: 93.0,
        },
        // Alta Capacidade & ROI (Mid-Frontier)
        RawBenchmarkSeed {
            raw_id: "qwen/qwen3.8-max-prime",
            general_elo: 1485.3,
            coding_elo: 77.0,
            hard_prompts_elo: 1480.0,
            bfcl_score: 92.0,
            ifeval_score: 91.5,
            gpqa_score: 80.0,
            vqa_multimodal_score: 90.5,
        },
        RawBenchmarkSeed {
            raw_id: "qwen/qwen3.8-max-0902",
            general_elo: 33.7,
            coding_elo: 76.2,
            hard_prompts_elo: 33.7,
            bfcl_score: 56.0,
            ifeval_score: 90.5,
            gpqa_score: 79.0,
            vqa_multimodal_score: 89.0,
        },
        RawBenchmarkSeed {
            raw_id: "openai/gpt-5.5",
            general_elo: 1466.2,
            coding_elo: 74.9,
            hard_prompts_elo: 1466.2,
            bfcl_score: 36.4,
            ifeval_score: 91.0,
            gpqa_score: 80.0,
            vqa_multimodal_score: 91.0,
        },
        RawBenchmarkSeed {
            raw_id: "openai/gpt-5.4",
            general_elo: 1452.5,
            coding_elo: 71.1,
            hard_prompts_elo: 1452.5,
            bfcl_score: 88.0,
            ifeval_score: 90.0,
            gpqa_score: 77.0,
            vqa_multimodal_score: 89.5,
        },
        RawBenchmarkSeed {
            raw_id: "meta/muse-wonder",
            general_elo: 43.6,
            coding_elo: 72.0,
            hard_prompts_elo: 1503.0,
            bfcl_score: 87.5,
            ifeval_score: 90.5,
            gpqa_score: 78.5,
            vqa_multimodal_score: 91.0,
        },
        RawBenchmarkSeed {
            raw_id: "meta/muse-spark-1.3",
            general_elo: 36.6,
            coding_elo: 75.8,
            hard_prompts_elo: 1491.6,
            bfcl_score: 55.5,
            ifeval_score: 89.5,
            gpqa_score: 76.0,
            vqa_multimodal_score: 89.0,
        },
        RawBenchmarkSeed {
            raw_id: "x-ai/grok-4.7",
            general_elo: 35.5,
            coding_elo: 76.0,
            hard_prompts_elo: 1455.0,
            bfcl_score: 88.5,
            ifeval_score: 89.0,
            gpqa_score: 78.5,
            vqa_multimodal_score: 88.0,
        },
        RawBenchmarkSeed {
            raw_id: "x-ai/grok-4.6",
            general_elo: 31.0,
            coding_elo: 75.9,
            hard_prompts_elo: 1430.0,
            bfcl_score: 52.5,
            ifeval_score: 88.5,
            gpqa_score: 76.5,
            vqa_multimodal_score: 87.5,
        },
        RawBenchmarkSeed {
            raw_id: "moonshotai/kimi-k3.1-pro",
            general_elo: 46.5,
            coding_elo: 73.5,
            hard_prompts_elo: 1459.1,
            bfcl_score: 88.0,
            ifeval_score: 89.5,
            gpqa_score: 79.0,
            vqa_multimodal_score: 88.0,
        },
        RawBenchmarkSeed {
            raw_id: "moonshotai/kimi-k3",
            general_elo: 17.4,
            coding_elo: 72.0,
            hard_prompts_elo: 1474.6,
            bfcl_score: 29.3,
            ifeval_score: 87.0,
            gpqa_score: 73.0,
            vqa_multimodal_score: 84.0,
        },
        RawBenchmarkSeed {
            raw_id: "z-ai/glm-5.3-prime",
            general_elo: 1475.0,
            coding_elo: 75.5,
            hard_prompts_elo: 1470.0,
            bfcl_score: 90.0,
            ifeval_score: 90.0,
            gpqa_score: 77.5,
            vqa_multimodal_score: 0.0,
        },
        RawBenchmarkSeed {
            raw_id: "z-ai/glm-5.3",
            general_elo: 31.7,
            coding_elo: 74.8,
            hard_prompts_elo: 1474.9,
            bfcl_score: 53.1,
            ifeval_score: 89.0,
            gpqa_score: 75.5,
            vqa_multimodal_score: 0.0,
        },
        RawBenchmarkSeed {
            raw_id: "xiaomi/mimo-v2.6-pro",
            general_elo: 29.9,
            coding_elo: 73.0,
            hard_prompts_elo: 1491.6,
            bfcl_score: 87.5,
            ifeval_score: 89.0,
            gpqa_score: 75.0,
            vqa_multimodal_score: 86.0,
        },
        RawBenchmarkSeed {
            raw_id: "deepseek/deepseek-v4-pro",
            general_elo: 20.8,
            coding_elo: 71.0,
            hard_prompts_elo: 1451.3,
            bfcl_score: 85.5,
            ifeval_score: 88.0,
            gpqa_score: 74.5,
            vqa_multimodal_score: 0.0,
        },
        // Velocidade, Triagem & Ultra Econômicos
        RawBenchmarkSeed {
            raw_id: "google/gemini-3.8-flash",
            general_elo: 30.8,
            coding_elo: 76.3,
            hard_prompts_elo: 1499.6,
            bfcl_score: 40.2,
            ifeval_score: 88.5,
            gpqa_score: 73.5,
            vqa_multimodal_score: 91.5,
        },
        RawBenchmarkSeed {
            raw_id: "anthropic/claude-haiku-5.5",
            general_elo: 36.5,
            coding_elo: 67.5,
            hard_prompts_elo: 1430.0,
            bfcl_score: 84.0,
            ifeval_score: 89.0,
            gpqa_score: 70.5,
            vqa_multimodal_score: 86.0,
        },
        RawBenchmarkSeed {
            raw_id: "openai/gpt-6-luna",
            general_elo: 27.9,
            coding_elo: 65.0,
            hard_prompts_elo: 1410.0,
            bfcl_score: 82.5,
            ifeval_score: 87.5,
            gpqa_score: 68.5,
            vqa_multimodal_score: 84.5,
        },
        RawBenchmarkSeed {
            raw_id: "deepseek/deepseek-v4.1-flash",
            general_elo: 28.6,
            coding_elo: 66.0,
            hard_prompts_elo: 1463.6,
            bfcl_score: 81.0,
            ifeval_score: 86.5,
            gpqa_score: 70.0,
            vqa_multimodal_score: 83.5,
        },
        RawBenchmarkSeed {
            raw_id: "deepseek/deepseek-v4-flash-vision-exp",
            general_elo: 1435.0,
            coding_elo: 65.5,
            hard_prompts_elo: 1435.0,
            bfcl_score: 80.0,
            ifeval_score: 86.0,
            gpqa_score: 68.0,
            vqa_multimodal_score: 88.0,
        },
        RawBenchmarkSeed {
            raw_id: "deepseek/deepseek-v4-flash",
            general_elo: 1432.0,
            coding_elo: 56.2,
            hard_prompts_elo: 1432.0,
            bfcl_score: 22.2,
            ifeval_score: 84.5,
            gpqa_score: 67.0,
            vqa_multimodal_score: 0.0,
        },
        RawBenchmarkSeed {
            raw_id: "z-ai/glm-5.3-flash",
            general_elo: 27.8,
            coding_elo: 71.5,
            hard_prompts_elo: 27.8,
            bfcl_score: 50.9,
            ifeval_score: 82.0,
            gpqa_score: 65.0,
            vqa_multimodal_score: 83.0,
        },
        RawBenchmarkSeed {
            raw_id: "xiaomi/mimo-v2.6-flash",
            general_elo: 26.7,
            coding_elo: 63.5,
            hard_prompts_elo: 26.7,
            bfcl_score: 79.0,
            ifeval_score: 82.0,
            gpqa_score: 64.0,
            vqa_multimodal_score: 82.0,
        },
        RawBenchmarkSeed {
            raw_id: "minimax/minimax-m3",
            general_elo: 18.9,
            coding_elo: 58.6,
            hard_prompts_elo: 1428.1,
            bfcl_score: 29.5,
            ifeval_score: 82.5,
            gpqa_score: 64.0,
            vqa_multimodal_score: 81.0,
        },
        RawBenchmarkSeed {
            raw_id: "qwen/qwen3.8-27b",
            general_elo: 20.1,
            coding_elo: 68.1,
            hard_prompts_elo: 1436.6,
            bfcl_score: 45.8,
            ifeval_score: 84.0,
            gpqa_score: 66.0,
            vqa_multimodal_score: 82.0,
        },
        RawBenchmarkSeed {
            raw_id: "qwen/qwen3.7-flash",
            general_elo: 1410.0,
            coding_elo: 58.0,
            hard_prompts_elo: 1410.0,
            bfcl_score: 76.5,
            ifeval_score: 83.0,
            gpqa_score: 62.0,
            vqa_multimodal_score: 79.0,
        },
        RawBenchmarkSeed {
            raw_id: "tencent/hy4",
            general_elo: 34.6,
            coding_elo: 66.0,
            hard_prompts_elo: 1435.0,
            bfcl_score: 81.0,
            ifeval_score: 85.5,
            gpqa_score: 69.0,
            vqa_multimodal_score: 83.0,
        },
        RawBenchmarkSeed {
            raw_id: "tencent/hy4-flash",
            general_elo: 1395.0,
            coding_elo: 56.0,
            hard_prompts_elo: 1395.0,
            bfcl_score: 75.0,
            ifeval_score: 82.0,
            gpqa_score: 61.0,
            vqa_multimodal_score: 77.0,
        },
        RawBenchmarkSeed {
            raw_id: "inclusionai/ling-3.1-flash",
            general_elo: 29.6,
            coding_elo: 60.5,
            hard_prompts_elo: 1405.0,
            bfcl_score: 77.0,
            ifeval_score: 83.0,
            gpqa_score: 63.0,
            vqa_multimodal_score: 0.0,
        },
        RawBenchmarkSeed {
            raw_id: "inclusionai/ling-3.0-flash",
            general_elo: 1385.0,
            coding_elo: 54.0,
            hard_prompts_elo: 1385.0,
            bfcl_score: 74.0,
            ifeval_score: 81.0,
            gpqa_score: 60.0,
            vqa_multimodal_score: 0.0,
        },
        // Universal Multi-Cloud Gateway Seeds (v0.7.3)
        RawBenchmarkSeed {
            raw_id: "deepseek/deepseek-chat",
            general_elo: 1480.0,
            coding_elo: 74.0,
            hard_prompts_elo: 1470.0,
            bfcl_score: 88.0,
            ifeval_score: 90.0,
            gpqa_score: 75.0,
            vqa_multimodal_score: 0.0,
        },
        RawBenchmarkSeed {
            raw_id: "deepseek/deepseek-reasoner",
            general_elo: 1520.0,
            coding_elo: 77.5,
            hard_prompts_elo: 1530.0,
            bfcl_score: 89.0,
            ifeval_score: 91.5,
            gpqa_score: 84.0,
            vqa_multimodal_score: 0.0,
        },
        RawBenchmarkSeed {
            raw_id: "meta/llama-3.3-70b-instruct",
            general_elo: 1440.0,
            coding_elo: 65.0,
            hard_prompts_elo: 1430.0,
            bfcl_score: 83.0,
            ifeval_score: 88.0,
            gpqa_score: 70.0,
            vqa_multimodal_score: 0.0,
        },
        RawBenchmarkSeed {
            raw_id: "anthropic/claude-3.5-sonnet",
            general_elo: 1495.0,
            coding_elo: 75.5,
            hard_prompts_elo: 1490.0,
            bfcl_score: 91.0,
            ifeval_score: 92.0,
            gpqa_score: 78.0,
            vqa_multimodal_score: 91.0,
        },
        RawBenchmarkSeed {
            raw_id: "openai/gpt-4o",
            general_elo: 1475.0,
            coding_elo: 70.0,
            hard_prompts_elo: 1465.0,
            bfcl_score: 88.5,
            ifeval_score: 90.5,
            gpqa_score: 74.0,
            vqa_multimodal_score: 91.5,
        },
    ]
}
