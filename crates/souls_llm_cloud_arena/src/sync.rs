//! # sync.rs — Orquestrador do Ciclo de Sincronização Diária
//!
//! Executa verificação diária, ingestão direta da Nous Portal Inference API
//! (catálogo canônico oficial de 342+ modelos reais e tarifas), OpenRouter
//! (preço de mercado e benchmarks Artificial Analysis) e LMSYS Arena (Hugging Face),
//! calculando a Fronteira de Pareto nos 5 perfis cognitivos.

use crate::analytics::{
    calculate_pareto_evaluations, compute_blended_cost, compute_composite_score,
    compute_roi_index, EvaluationCandidate,
};
use crate::canonical::CanonicalResolver;
use crate::config::CloudArenaConfig;
use crate::domain::{CanonicalModel, PricingTelemetrySnapshot, SyncOutcome, TaskProfileKind};
use crate::error::CloudArenaError;
use crate::sources::{BenchmarkSource, NousPortalClient, OpenRouterClient};
use crate::storage::{run_migrations, CloudArenaRepository};
use chrono::Utc;
use sqlx::SqlitePool;
use std::collections::HashMap;
use tracing::{info, warn};

pub struct CloudArenaSync {
    pub config: CloudArenaConfig,
    pub repo: CloudArenaRepository,
    pub resolver: CanonicalResolver,
    pub openrouter: OpenRouterClient,
    pub nous: NousPortalClient,
}

impl CloudArenaSync {
    pub fn new(config: CloudArenaConfig, pool: SqlitePool) -> Self {
        let repo = CloudArenaRepository::new(pool);
        let resolver = CanonicalResolver::new();
        let openrouter = OpenRouterClient::new(
            &config.openrouter_base_url,
            config.openrouter_api_key.clone(),
            config.request_timeout_secs,
        );
        let nous = NousPortalClient::new(
            &config.nous_base_url,
            config.nous_api_key.clone(),
            config.request_timeout_secs,
        );

        Self {
            config,
            repo,
            resolver,
            openrouter,
            nous,
        }
    }

    /// Verifica a data do último snapshot; se não for hoje UTC, dispara sincronização.
    pub async fn sync_if_needed(&self) -> Result<Option<SyncOutcome>, CloudArenaError> {
        let today = Utc::now().format("%Y-%m-%d").to_string();
        let last_date = self.repo.get_last_sync_date().await?;

        if let Some(ref d) = last_date {
            if d == &today {
                info!("Cloud Arena já está atualizada para hoje ({}). Sincronização dispensada.", today);
                return Ok(None);
            }
        }

        info!("Iniciando atualização diária da Cloud Arena (Data alvo: {})...", today);
        let outcome = self.force_sync(&today).await?;
        Ok(Some(outcome))
    }

    /// Executa o pipeline de sincronização completo para uma data alvo.
    pub async fn force_sync(&self, target_date: &str) -> Result<SyncOutcome, CloudArenaError> {
        // 1. Assegura migrations SQLite STRICT
        run_migrations(self.repo.pool()).await?;

        let mut warning = None;
        let mut from_cached_fallback = false;
        let mut resolver = self.resolver.clone();

        // 2. Ingestão direta do catálogo primário da Nous Research Inference API
        let nous_result = self.nous.fetch_catalog(target_date).await?;
        if nous_result.canonical_models.len() < 50 {
            from_cached_fallback = true;
            warning = Some("Utilizando catálogo offline de fallback para o Nous Portal.".to_string());
        }

        // 3. Registra os modelos canônicos e aliases dinamicamente no banco e no resolver
        for model in &nous_result.canonical_models {
            self.repo.upsert_canonical_model(model).await?;
            resolver.register_canonical(&model.id);
        }

        for (alias, canonical_id, source) in &nous_result.aliases {
            let _ = self.repo.upsert_model_alias(alias, canonical_id, source).await;
            resolver.register_alias(alias, canonical_id);
        }

        // 4. Ingestão OpenRouter (Preços de mercado spot + Artificial Analysis Intelligence Index)
        let openrouter_result = match self
            .openrouter
            .fetch_market_pricing_and_benchmarks(&resolver, target_date)
            .await
        {
            Ok(res) => res,
            Err(e) => {
                warn!("Falha ao consultar OpenRouter ({}). Continuando sem preços de mercado.", e);
                warning = Some(format!("OpenRouter offline: {}", e));
                crate::sources::openrouter::OpenRouterIngestionResult {
                    pricing_snapshots: Vec::new(),
                    benchmark_snapshots: Vec::new(),
                }
            }
        };

        // 5. Ingestão LMSYS Chatbot Arena via Hugging Face API
        let lmsys_snapshots = match BenchmarkSource::fetch_lmsys_arena(&resolver, target_date).await {
            Ok(snaps) => snaps,
            Err(e) => {
                warn!("Falha ao consultar LMSYS Arena ({}). Prosseguindo com benchmarks disponíveis.", e);
                Vec::new()
            }
        };

        // 6. Grava snapshots de precificação no banco
        let mut total_pricing = 0;
        for snap in &nous_result.pricing_snapshots {
            self.repo.record_pricing_snapshot(snap).await?;
            total_pricing += 1;
        }
        for snap in &nous_result.batch_snapshots {
            self.repo.record_pricing_snapshot(snap).await?;
            total_pricing += 1;
        }
        for snap in &openrouter_result.pricing_snapshots {
            self.repo.record_pricing_snapshot(snap).await?;
            total_pricing += 1;
        }

        // 7. Consolidação de benchmarks (Artificial Analysis + LMSYS + Heurística para todos os canônicos)
        let mut bench_by_model: HashMap<String, crate::domain::BenchmarkSnapshot> = HashMap::new();

        // 7.1 Artificial Analysis da OpenRouter
        for b in openrouter_result.benchmark_snapshots {
            bench_by_model.insert(b.canonical_id.clone(), b);
        }

        // 7.2 Enriquece com LMSYS Elo humano
        for lmsys in lmsys_snapshots {
            bench_by_model
                .entry(lmsys.canonical_id.clone())
                .and_modify(|existing| {
                    if existing.general_elo.is_none() {
                        existing.general_elo = lmsys.general_elo;
                    }
                    if existing.coding_elo.is_none() {
                        existing.coding_elo = lmsys.coding_elo;
                    }
                    if existing.hard_prompts_elo.is_none() {
                        existing.hard_prompts_elo = lmsys.hard_prompts_elo;
                    }
                })
                .or_insert(lmsys);
        }

        // 7.3 Garante que todos os 342 modelos canônicos tenham benchmark válido
        for m in &nous_result.canonical_models {
            bench_by_model.entry(m.id.clone()).or_insert_with(|| {
                let baseline = estimate_baseline_score(m);
                crate::domain::BenchmarkSnapshot {
                    canonical_id: m.id.clone(),
                    general_elo: Some(baseline),
                    coding_elo: Some(baseline),
                    hard_prompts_elo: Some(baseline),
                    bfcl_score: if m.supports_tools { Some(baseline + 5.0) } else { Some(baseline - 10.0) },
                    ifeval_score: Some(baseline),
                    gpqa_score: Some(baseline - 5.0),
                    vqa_multimodal_score: if m.is_multimodal { Some(baseline + 5.0) } else { None },
                    recorded_at: target_date.to_string(),
                }
            });
        }

        // Persiste todos os benchmarks consolidados
        let mut total_bench = 0;
        for (_, b) in &bench_by_model {
            self.repo.record_benchmark_snapshot(b).await?;
            total_bench += 1;
        }

        // 8. Mapa de preços primários de execução (Nous Portal)
        let mut execution_pricing_by_model: HashMap<String, PricingTelemetrySnapshot> = HashMap::new();
        for snap in &nous_result.pricing_snapshots {
            execution_pricing_by_model.insert(snap.canonical_id.clone(), snap.clone());
        }

        // 9. Calcula avaliações na Fronteira de Pareto nos 5 Perfis Cognitivos
        let mut total_evals = 0;

        for &profile in TaskProfileKind::all() {
            let mut candidates = Vec::new();

            for (canonical_id, bench) in &bench_by_model {
                if let Some(price) = execution_pricing_by_model.get(canonical_id) {
                    let score = compute_composite_score(bench, price, profile);
                    let blended_cost = compute_blended_cost(price, profile);
                    let roi = compute_roi_index(score, blended_cost);

                    candidates.push(EvaluationCandidate {
                        canonical_id: canonical_id.clone(),
                        composite_score: score,
                        blended_cost,
                        roi_index: roi,
                    });
                }
            }

            let evals = calculate_pareto_evaluations(&candidates, profile, target_date);
            for eval in evals {
                self.repo.record_daily_evaluation(&eval).await?;
                total_evals += 1;
            }
        }

        info!(
            "Sincronização Cloud Arena concluída: {} modelos canônicos, {} tarifas, {} benchmarks, {} avaliações de Pareto.",
            nous_result.canonical_models.len(),
            total_pricing,
            total_bench,
            total_evals
        );

        Ok(SyncOutcome {
            date: target_date.to_string(),
            models_synced: nous_result.canonical_models.len(),
            pricing_snapshots: total_pricing,
            benchmark_snapshots: total_bench,
            evaluations_calculated: total_evals,
            from_cached_fallback,
            warning,
        })
    }
}

/// Estima baseline prudente de inteligência (0-100) para modelos sem benchmark verificado.
fn estimate_baseline_score(m: &CanonicalModel) -> f64 {
    let lower = m.id.to_lowercase();
    if lower.contains("opus") || lower.contains("astra") || lower.contains("argon") {
        53.0
    } else if lower.contains("sonnet") || lower.contains("sol") || lower.contains("prime") {
        50.0
    } else if lower.contains("flash") || lower.contains("haiku") || lower.contains("luna") || lower.contains("mini") {
        44.0
    } else if lower.contains("405b") || lower.contains("70b") {
        47.0
    } else if m.context_window >= 500_000 {
        45.0
    } else {
        40.0
    }
}
