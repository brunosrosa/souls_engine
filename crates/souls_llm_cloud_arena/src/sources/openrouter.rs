//! # openrouter.rs — Ingestor da API OpenRouter
//!
//! Consome o endpoint /api/v1/models público ou autenticado,
//! extraindo janelas de contexto, precificação spot por 1M tokens e telemetria.

use crate::canonical::CanonicalResolver;
use crate::domain::PricingTelemetrySnapshot;
use crate::error::CloudArenaError;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use serde::Deserialize;
use std::time::Duration;

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
}

#[derive(Debug, Deserialize)]
pub struct OpenRouterPricing {
    pub prompt: Option<serde_json::Value>,
    pub completion: Option<serde_json::Value>,
    #[serde(rename = "request")]
    pub _request: Option<serde_json::Value>,
    #[serde(rename = "image")]
    pub _image: Option<serde_json::Value>,
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

    /// Executa requisição para /api/v1/models e normaliza para snapshots de precificação.
    pub async fn fetch_pricing_snapshots(
        &self,
        resolver: &CanonicalResolver,
        recorded_at: &str,
    ) -> Result<Vec<PricingTelemetrySnapshot>, CloudArenaError> {
        let url = format!("{}/models", self.base_url);
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
        let mut snapshots = Vec::new();

        for item in payload.data {
            let canonical_id = match resolver.try_resolve(&item.id) {
                Some(id) => id,
                None => continue,
            };

            let (prompt_price_m, completion_price_m) = if let Some(p) = item.pricing {
                let prompt_val = parse_price_value(p.prompt);
                let comp_val = parse_price_value(p.completion);
                // Preço da OpenRouter é em USD por token unitário; multiplica por 1M para USD/1M tokens
                (prompt_val * 1_000_000.0, comp_val * 1_000_000.0)
            } else {
                (0.0, 0.0)
            };

            // Estima Throughput típico com base em família/tamanho
            let throughput = estimate_throughput(&canonical_id);

            snapshots.push(PricingTelemetrySnapshot {
                canonical_id,
                source: "openrouter".to_string(),
                prompt_price_per_m: prompt_price_m,
                completion_price_per_m: completion_price_m,
                cache_read_price_per_m: prompt_price_m * 0.25, // Cache read com desconto padrão de 75%
                cache_write_price_per_m: prompt_price_m * 1.25,
                ttft_ms: Some(450.0),
                throughput_tps: Some(throughput),
                recorded_at: recorded_at.to_string(),
            });
        }

        Ok(snapshots)
    }
}

/// Faz parsing flexível de campos de preço (que podem vir como número ou string no JSON).
fn parse_price_value(val: Option<serde_json::Value>) -> f64 {
    match val {
        Some(serde_json::Value::Number(n)) => n.as_f64().unwrap_or(0.0),
        Some(serde_json::Value::String(s)) => s.parse::<f64>().unwrap_or(0.0),
        _ => 0.0,
    }
}

/// Estima throughput médio de mercado em TPS para modelos conhecidos.
fn estimate_throughput(canonical_id: &str) -> f64 {
    if canonical_id.contains("haiku") || canonical_id.contains("flash") || canonical_id.contains("mini") {
        120.0
    } else if canonical_id.contains("coder-32b") {
        95.0
    } else if canonical_id.contains("deepseek-v3") {
        85.0
    } else if canonical_id.contains("deepseek-r1") || canonical_id.contains("o1") {
        45.0
    } else if canonical_id.contains("405b") {
        35.0
    } else {
        70.0
    }
}
