//! # souls_llm_cloud_arena (Souls Engine v7)
//!
//! Ingestão diária, análise de custo-benefício e cálculo da Fronteira de Pareto
//! para modelos de linguagem em nuvem (Nous Research Portal e OpenRouter)
//! sob os 5 perfis cognitivos do Souls Engine bare-metal.

#![forbid(unsafe_code)]

pub mod analytics;
pub mod canonical;
pub mod config;
pub mod domain;
pub mod error;
pub mod report;
pub mod sources;
pub mod storage;
pub mod sync;

pub use config::{resolve_heuristic_vault_path, CloudArenaConfig};
pub use domain::{
    BenchmarkSnapshot, CanonicalModel, ChampionEntry, DailyEvaluation, ParetoTag,
    PricingTelemetrySnapshot, ProfileChampionReport, SyncOutcome, TaskProfileKind,
};
pub use error::CloudArenaError;
pub use report::{generate_markdown_report, print_champions_table};
pub use storage::CloudArenaRepository;
pub use sync::CloudArenaSync;

use sqlx::sqlite::SqlitePoolOptions;
use std::sync::Arc;

/// Fachada unificada de alto nível da Cloud Arena.
#[derive(Clone)]
pub struct CloudArenaCoordinator {
    sync: Arc<CloudArenaSync>,
}

impl CloudArenaCoordinator {
    /// Inicializa o coordenador conectando ao banco Heuristic Vault em Dev Drive ReFS.
    pub async fn new(config: CloudArenaConfig) -> Result<Self, CloudArenaError> {
        let db_url = format!("sqlite://{}?mode=rwc", config.database_path.to_string_lossy());
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&db_url)
            .await?;

        // Executa migrations DDL STRICT
        storage::run_migrations(&pool).await?;

        let sync = Arc::new(CloudArenaSync::new(config, pool));
        Ok(Self { sync })
    }

    /// Cria uma instância conectada a um pool existente.
    pub fn from_pool(config: CloudArenaConfig, pool: sqlx::SqlitePool) -> Self {
        let sync = Arc::new(CloudArenaSync::new(config, pool));
        Self { sync }
    }

    /// Dispara a sincronização diária se ainda não executada hoje UTC.
    pub async fn sync_if_needed(&self) -> Result<Option<SyncOutcome>, CloudArenaError> {
        self.sync.sync_if_needed().await
    }

    /// Força a sincronização com a data especificada ou a data atual UTC.
    pub async fn force_sync(&self, target_date: Option<&str>) -> Result<SyncOutcome, CloudArenaError> {
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let date = target_date.unwrap_or(&today);
        self.sync.force_sync(date).await
    }

    /// Obtém os campeões de Pareto para todos os 5 perfis cognitivos.
    pub async fn get_all_champions(&self, date: Option<&str>) -> Result<Vec<ProfileChampionReport>, CloudArenaError> {
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let d = date.unwrap_or(&today);

        let mut reports = Vec::new();
        for &profile in TaskProfileKind::all() {
            let champ = self.sync.repo.get_champions(profile, d).await?;
            reports.push(champ);
        }

        Ok(reports)
    }

    /// Exporta o relatório completo de campeões em formato Markdown.
    pub async fn export_markdown_report(&self, date: Option<&str>) -> Result<String, CloudArenaError> {
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let d = date.unwrap_or(&today);
        let champions = self.get_all_champions(Some(d)).await?;
        Ok(generate_markdown_report(d, &champions))
    }
}
