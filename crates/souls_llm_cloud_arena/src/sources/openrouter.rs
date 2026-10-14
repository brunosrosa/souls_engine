//! # openrouter.rs — Ingestor da API OpenRouter
//!
//! Consome o endpoint /api/v1/models da OpenRouter, extraindo tarifas de mercado
//! (Market Price) para comparação com o Nous Portal, e os benchmarks oficiais
//! do Artificial Analysis (Intelligence Index, Coding Index e Agentic Index).

use crate::canonical::CanonicalResolver;
use crate::domain::{BenchmarkSnapshot, PricingTelemetrySnapshot};
use crate::error::CloudArenaError;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use serde::Deserialize;
use std::time::Duration;
use tracing::info;

#[derive(Debug, Clone)]
pub struct OpenRouterIngestionResult {
    pub pricing_snapshots: Vec<PricingTelemetrySnapshot>,
    pub benchmark_snapshots: Vec<BenchmarkSnapshot>,
}

#[derive(Debug, Deserialize)]
pub struct OpenRouterModelsResponse {
    pub data: Vec<OpenRouterModelItem>,
}

#[derive(Debug, Deserialize)]
pub struct OpenRouterModelItem {
    pub id: String,
    pub name: Option<String>,
    pub context_length: Option<i64>,
    pub pricing: Option<OpenRouterPricing>,
    pub benchmarks: Option<OpenRouterBenchmarks>,
}

#[derive(Debug, Deserialize)]
pub struct OpenRouterPricing {
    pub prompt: Option<serde_json::Value>,
    pub completion: Option<serde_json::Value>,
    pub input_cache_read: Option<serde_json::Value>,
    pub input_cache_write: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct OpenRouterBenchmarks {
    pub artificial_analysis: Option<ArtificialAnalysisScores>,
}

#[derive(Debug, Deserialize)]
pub struct ArtificialAnalysisScores {
    pub intelligence_index: Option<f64>,
    pub coding_index: Option<f64>,
    pub agentic_index: Option<f64>,
}

pub struct OpenRouterClient {
    client: reqwest::Client,
    base_url: String,
    _api_key: Option<String>,
}

impl OpenRouterClient {
    pub fn new(base_url: &str, api_key: Option<String>, timeout_secs: u64) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert("HTTP-Referer", HeaderValue::from_static("https://soulsengine.ai"));
        headers.insert("X-Title", HeaderValue::from_static("Souls Engine Bare-Metal"));

        if let Some(ref key) = api_key {
            if let Ok(val) = HeaderValue::from_str(&format!("Bearer {}", key)) {
                headers.insert(AUTHORIZATION, val);
            }
        }

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(timeout_secs))
            .build()
            .unwrap_or_default();

        Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
            _api_key: api_key,
        }
    }

    /// Executa requisição para /api/v1/models e normaliza para snapshots de precificação de mercado e benchmarks do Artificial Analysis.
    pub async fn fetch_market_pricing_and_benchmarks(
        &self,
        resolver: &CanonicalResolver,
        recorded_at: &str,
    ) -> Result<OpenRouterIngestionResult, CloudArenaError> {
        let url = format!("{}/models", self.base_url);
        info!("Consultando OpenRouter API em {} para precificação de mercado e benchmarks...", url);
        let resp = self.client.get(&url).send().await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(CloudArenaError::SyncFailed(format!(
                "OpenRouter API retornou status {}: {}",
                status, body
            )));
        }

        let payload: OpenRouterModelsResponse = resp.json().await?;
        let mut pricing_snapshots = Vec::new();
        let mut benchmark_snapshots = Vec::new();

        for item in payload.data {
            let canonical_id = match resolver.try_resolve(&item.id) {
                Some(id) => id,
                None => continue,
            };

            // 1. Extração de preços spot de mercado
            if let Some(ref p) = item.pricing {
                let prompt_val = parse_price_value(p.prompt.as_ref());
                let comp_val = parse_price_value(p.completion.as_ref());
                let cread_val = parse_price_value(p.input_cache_read.as_ref());
                let cwrite_val = parse_price_value(p.input_cache_write.as_ref());

                let prompt_price_m = prompt_val * 1_000_000.0;
                let comp_price_m = comp_val * 1_000_000.0;
                let cache_read_m = if cread_val > 0.0 {
                    cread_val * 1_000_000.0
                } else {
                    prompt_price_m * 0.25
                };
                let cache_write_m = if cwrite_val > 0.0 {
                    cwrite_val * 1_000_000.0
                } else {
                    prompt_price_m * 1.25
                };

                let throughput = estimate_throughput(&canonical_id);

                pricing_snapshots.push(PricingTelemetrySnapshot {
                    canonical_id: canonical_id.clone(),
                    source: "openrouter_market".to_string(),
                    prompt_price_per_m: prompt_price_m,
                    completion_price_per_m: comp_price_m,
                    cache_read_price_per_m: cache_read_m,
                    cache_write_price_per_m: cache_write_m,
                    ttft_ms: Some(450.0),
                    throughput_tps: Some(throughput),
                    recorded_at: recorded_at.to_string(),
                });
            }

            // 2. Extração de Benchmarks do Artificial Analysis
            if let Some(ref bm) = item.benchmarks {
                if let Some(ref aa) = bm.artificial_analysis {
                    if aa.intelligence_index.is_some() || aa.coding_index.is_some() || aa.agentic_index.is_some() {
                        benchmark_snapshots.push(BenchmarkSnapshot {
                            canonical_id: canonical_id.clone(),
                            general_elo: aa.intelligence_index, // Artificial Analysis Intelligence Index
                            coding_elo: aa.coding_index,        // Artificial Analysis Coding Index
                            hard_prompts_elo: aa.agentic_index, // Artificial Analysis Agentic Index
                            bfcl_score: aa.agentic_index,
                            ifeval_score: None,
                            gpqa_score: None,
                            vqa_multimodal_score: None,
                            recorded_at: recorded_at.to_string(),
                        });
                    }
                }
            }
        }

        info!(
            "OpenRouter processado com sucesso: {} snapshots de mercado e {} benchmarks do Artificial Analysis.",
            pricing_snapshots.len(),
            benchmark_snapshots.len()
        );

        Ok(OpenRouterIngestionResult {
            pricing_snapshots,
            benchmark_snapshots,
        })
    }

    /// Método de compatibilidade para extração de snapshots de preços da OpenRouter.
    pub async fn fetch_pricing_snapshots(
        &self,
        resolver: &CanonicalResolver,
        recorded_at: &str,
    ) -> Result<Vec<PricingTelemetrySnapshot>, CloudArenaError> {
        let res = self.fetch_market_pricing_and_benchmarks(resolver, recorded_at).await?;
        Ok(res.pricing_snapshots)
    }
}

/// Faz parsing flexível de campos de preço (número ou string).
fn parse_price_value(val: Option<&serde_json::Value>) -> f64 {
    match val {
        Some(serde_json::Value::Number(n)) => n.as_f64().unwrap_or(0.0),
        Some(serde_json::Value::String(s)) => s.parse::<f64>().unwrap_or(0.0),
        _ => 0.0,
    }
}

/// Estima throughput médio de mercado em TPS para modelos conhecidos.
fn estimate_throughput(canonical_id: &str) -> f64 {
    let lower = canonical_id.to_lowercase();
    if lower.contains("haiku") || lower.contains("flash") || lower.contains("mini") || lower.contains("luna") {
        130.0
    } else if lower.contains("coder-32b") || lower.contains("sol") {
        95.0
    } else if lower.contains("deepseek-v3") {
        85.0
    } else if lower.contains("deepseek-r1") || lower.contains("astra") || lower.contains("opus") {
        45.0
    } else if lower.contains("405b") {
        35.0
    } else {
        70.0
    }
}
