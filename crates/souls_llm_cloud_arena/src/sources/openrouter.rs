//! # openrouter.rs — Ingestor da API OpenRouter
//!
//! Consome o endpoint /api/v1/models da OpenRouter, extraindo tarifas de mercado
//! (Market Price) para comparação com o Nous Portal, e os benchmarks oficiais
//! do Artificial Analysis (Intelligence Index, Coding Index e Agentic Index).

use crate::canonical::CanonicalResolver;
use crate::domain::{BenchmarkSnapshot, CanonicalModel, PricingTelemetrySnapshot};
use crate::error::CloudArenaError;
use crate::sources::nous::{format_model_display_name, infer_developer_and_family, parse_price_value};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use serde::Deserialize;
use std::collections::HashSet;
use std::time::Duration;
use tracing::info;

#[derive(Debug, Clone)]
pub struct OpenRouterIngestionResult {
    pub canonical_models: Vec<CanonicalModel>,
    pub pricing_snapshots: Vec<PricingTelemetrySnapshot>,
    pub benchmark_snapshots: Vec<BenchmarkSnapshot>,
}

#[derive(Debug, Deserialize)]
pub struct OpenRouterModelsResponse {
    pub data: Vec<OpenRouterModelItem>,
}

#[derive(Debug, Deserialize)]
pub struct OpenRouterArchitecture {
    pub modality: Option<String>,
    pub input_modalities: Option<Vec<String>>,
    pub output_modalities: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct OpenRouterModelItem {
    pub id: String,
    pub name: Option<String>,
    pub context_length: Option<i64>,
    pub architecture: Option<OpenRouterArchitecture>,
    pub pricing: Option<OpenRouterPricing>,
    pub supported_parameters: Option<Vec<String>>,
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
        let mut canonical_models = Vec::new();
        let mut pricing_snapshots = Vec::new();
        let mut benchmark_snapshots = Vec::new();
        let mut seen_canonicals = HashSet::new();

        for item in payload.data {
            let raw_id = item.id.trim().to_string();
            if raw_id.is_empty() || raw_id.ends_with(":batch") || raw_id.ends_with(":nitro") {
                continue;
            }
            let clean_id = raw_id.trim_start_matches('~').to_lowercase();

            let canonical_id = match resolver.try_resolve(&clean_id) {
                Some(id) => id,
                None => clean_id.clone(),
            };

            let is_free = canonical_id.ends_with(":free");
            let supports_tools = item
                .supported_parameters
                .as_ref()
                .map(|params| params.iter().any(|p| p == "tools" || p == "tool_choice"))
                .unwrap_or(false);
            let supports_structured_outputs = item
                .supported_parameters
                .as_ref()
                .map(|params| params.iter().any(|p| p == "response_format"))
                .unwrap_or(false);

            let (prompt_price_m, comp_price_m, cache_read_m, cache_write_m) = if is_free {
                (0.0, 0.0, 0.0, 0.0)
            } else if let Some(ref p) = item.pricing {
                let prompt_val = parse_price_value(p.prompt.as_ref());
                let comp_val = parse_price_value(p.completion.as_ref());
                let cread_val = parse_price_value(p.input_cache_read.as_ref());
                let cwrite_val = parse_price_value(p.input_cache_write.as_ref());

                let pm = prompt_val * 1_000_000.0;
                let cm = comp_val * 1_000_000.0;
                let cr = if cread_val > 0.0 {
                    cread_val * 1_000_000.0
                } else {
                    pm * 0.25
                };
                let cw = if cwrite_val > 0.0 {
                    cwrite_val * 1_000_000.0
                } else {
                    pm * 1.25
                };
                (pm, cm, cr, cw)
            } else {
                (0.0, 0.0, 0.0, 0.0)
            };

            let supports_caching = cache_read_m > 0.0;
            let is_multimodal = item
                .architecture
                .as_ref()
                .map(|arch| {
                    let mod_str = arch.modality.as_deref().unwrap_or("").to_lowercase();
                    let inputs = arch.input_modalities.as_ref();
                    mod_str.contains("image")
                        || mod_str.contains("vision")
                        || mod_str.contains("audio")
                        || inputs
                            .map(|inps| inps.iter().any(|i| i == "image" || i == "vision" || i == "audio"))
                            .unwrap_or(false)
                })
                .unwrap_or(false);

            let (developer, family) = infer_developer_and_family(&canonical_id);
            let display_name = format_model_display_name(item.name.clone(), &canonical_id);
            let context_window = item.context_length.unwrap_or(128_000);

            let model_entry = CanonicalModel {
                id: canonical_id.clone(),
                display_name,
                family,
                developer,
                context_window,
                supports_tools,
                supports_structured_outputs,
                supports_caching,
                is_multimodal,
                created_at: recorded_at.to_string(),
            };

            if !model_entry.is_generative_llm() {
                continue;
            }

            if seen_canonicals.insert(canonical_id.clone()) {
                canonical_models.push(model_entry);

                // 1. Extração de preços spot de mercado (inclusive $0.00 para modelos :free)
                if item.pricing.is_some() || is_free {
                    let throughput = crate::sources::nous::estimate_throughput(&canonical_id);
                    let ttft = crate::sources::nous::estimate_ttft(&canonical_id);

                    pricing_snapshots.push(PricingTelemetrySnapshot {
                        canonical_id: canonical_id.clone(),
                        source: "openrouter_market".to_string(),
                        prompt_price_per_m: prompt_price_m,
                        completion_price_per_m: comp_price_m,
                        cache_read_price_per_m: cache_read_m,
                        cache_write_price_per_m: cache_write_m,
                        ttft_ms: Some(ttft),
                        throughput_tps: Some(throughput),
                        recorded_at: recorded_at.to_string(),
                    });
                }
            }

            // 2. Extração de Benchmarks do Artificial Analysis
            if let Some(ref bm) = item.benchmarks {
                if let Some(ref aa) = bm.artificial_analysis {
                    if aa.intelligence_index.is_some() || aa.coding_index.is_some() || aa.agentic_index.is_some() {
                        benchmark_snapshots.push(BenchmarkSnapshot {
                            canonical_id: canonical_id.clone(),
                            general_elo: aa.intelligence_index,      // Artificial Analysis Intelligence Index
                            coding_elo: aa.coding_index,             // Artificial Analysis Coding Index
                            hard_prompts_elo: aa.intelligence_index, // Inteligência/Raciocínio Geral (sem confundir com agentic)
                            bfcl_score: aa.agentic_index,            // Artificial Analysis Agentic Index
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
            "OpenRouter processado com sucesso: {} modelos canônicos, {} snapshots de mercado e {} benchmarks do Artificial Analysis.",
            canonical_models.len(),
            pricing_snapshots.len(),
            benchmark_snapshots.len()
        );

        Ok(OpenRouterIngestionResult {
            canonical_models,
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
