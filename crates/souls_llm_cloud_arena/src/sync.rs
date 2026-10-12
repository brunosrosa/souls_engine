//! # sync.rs — Orquestrador do Ciclo de Sincronização Diária
//!
//! Executa verificação diária (last_update < today_utc), ingestão paralela resiliente
//! e cálculo da Fronteira de Pareto nos 5 perfis cognitivos.

use crate::analytics::{
    calculate_pareto_evaluations, compute_blended_cost, compute_composite_score,
    compute_roi_index, EvaluationCandidate,
};
use crate::canonical::{get_initial_seed_models, CanonicalResolver};
use crate::config::CloudArenaConfig;
use crate::domain::{PricingTelemetrySnapshot, SyncOutcome, TaskProfileKind};
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

        // 2. Registra semente inicial de modelos e aliases no banco
        let seed_models = get_initial_seed_models();
        for item in &seed_models {
            self.repo.upsert_canonical_model(&item.model).await?;
            for (alias, src) in &item.aliases {
                self.repo.upsert_model_alias(alias, &item.model.id, src).await?;
            }
        }

        let mut warning = None;
        let mut from_cached_fallback = false;

        // 3. Ingestão paralela OpenRouter e Nous Portal
        let openrouter_snapshots = match self.openrouter.fetch_pricing_snapshots(&self.resolver, target_date).await {
            Ok(snaps) => snaps,
            Err(e) => {
                warn!("Falha ao consultar OpenRouter API ({}). Continuando com dados locais.", e);
                warning = Some(format!("OpenRouter offline: {}", e));
                from_cached_fallback = true;
                Vec::new()
            }
        };

        let nous_snapshots = match self.nous.fetch_pricing_snapshots(&self.resolver, target_date).await {
            Ok(snaps) => snaps,
            Err(e) => {
                warn!("Falha ao consultar Nous Portal ({}). Recorrendo a catálogo de fallback.", e);
                from_cached_fallback = true;
                Vec::new()
            }
        };

        // 4. Ingestão de Benchmarks
        let bench_snapshots = BenchmarkSource::load_snapshots(&self.resolver, target_date)?;

        // 5. Persiste snapshots no banco SQLite
        let mut total_pricing = 0;
        for snap in &openrouter_snapshots {
            self.repo.record_pricing_snapshot(snap).await?;
            total_pricing += 1;
        }
        for snap in &nous_snapshots {
            self.repo.record_pricing_snapshot(snap).await?;
            total_pricing += 1;
        }

        let mut total_bench = 0;
        for b in &bench_snapshots {
            self.repo.record_benchmark_snapshot(b).await?;
            total_bench += 1;
        }

        // 6. Consolida mapa de preços mais baratos por modelo canônico
        let mut best_pricing_by_model: HashMap<String, PricingTelemetrySnapshot> = HashMap::new();
        for snap in openrouter_snapshots.into_iter().chain(nous_snapshots.into_iter()) {
            best_pricing_by_model
                .entry(snap.canonical_id.clone())
                .and_modify(|existing| {
                    // Escolhe o de menor custo de prompt para o modelo
                    if snap.prompt_price_per_m < existing.prompt_price_per_m && snap.prompt_price_per_m > 0.0 {
                        *existing = snap.clone();
                    }
                })
                .or_insert(snap);
        }

        // 7. Calcula avaliações nos 5 Perfis Cognitivos
        let mut total_evals = 0;

        for &profile in TaskProfileKind::all() {
            let mut candidates = Vec::new();

            for bench in &bench_snapshots {
                if let Some(price) = best_pricing_by_model.get(&bench.canonical_id) {
                    let score = compute_composite_score(bench, price, profile);
                    let blended_cost = compute_blended_cost(price, profile);
                    let roi = compute_roi_index(score, blended_cost);

                    candidates.push(EvaluationCandidate {
                        canonical_id: bench.canonical_id.clone(),
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
            "Sincronização concluída com sucesso: {} modelos, {} snapshots de preço, {} benchmarks, {} avaliações de Pareto.",
            seed_models.len(),
            total_pricing,
            total_bench,
            total_evals
        );

        Ok(SyncOutcome {
            date: target_date.to_string(),
            models_synced: seed_models.len(),
            pricing_snapshots: total_pricing,
            benchmark_snapshots: total_bench,
            evaluations_calculated: total_evals,
            from_cached_fallback,
            warning,
        })
    }
}
