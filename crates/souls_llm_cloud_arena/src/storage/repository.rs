//! # repository.rs — Repositório de Persistência SQLite da Cloud Arena
//!
//! Operações atômicas de leitura e escrita para o FrankenSQLite STRICT WAL.

use crate::domain::{
    BenchmarkSnapshot, CanonicalModel, ChampionEntry, DailyEvaluation,
    PricingTelemetrySnapshot, ProfileChampionReport, TaskProfileKind,
};
use crate::error::CloudArenaError;
use sqlx::{Row, SqlitePool};

#[derive(Clone)]
pub struct CloudArenaRepository {
    pool: SqlitePool,
}

impl CloudArenaRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// Insere ou atualiza um modelo canônico.
    pub async fn upsert_canonical_model(&self, m: &CanonicalModel) -> Result<(), CloudArenaError> {
        sqlx::query(
            r#"
            INSERT INTO canonical_models (
                id, display_name, family, developer, context_window,
                supports_tools, supports_structured_outputs, supports_caching, is_multimodal, created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                display_name = excluded.display_name,
                family = excluded.family,
                developer = excluded.developer,
                context_window = excluded.context_window,
                supports_tools = excluded.supports_tools,
                supports_structured_outputs = excluded.supports_structured_outputs,
                supports_caching = excluded.supports_caching,
                is_multimodal = excluded.is_multimodal;
            "#,
        )
        .bind(&m.id)
        .bind(&m.display_name)
        .bind(&m.family)
        .bind(&m.developer)
        .bind(m.context_window)
        .bind(m.supports_tools as i32)
        .bind(m.supports_structured_outputs as i32)
        .bind(m.supports_caching as i32)
        .bind(m.is_multimodal as i32)
        .bind(&m.created_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Insere ou atualiza um alias de modelo.
    pub async fn upsert_model_alias(
        &self,
        alias: &str,
        canonical_id: &str,
        source: &str,
    ) -> Result<(), CloudArenaError> {
        sqlx::query(
            r#"
            INSERT INTO model_aliases (alias, canonical_id, source)
            VALUES (?, ?, ?)
            ON CONFLICT(alias) DO UPDATE SET
                canonical_id = excluded.canonical_id,
                source = excluded.source;
            "#,
        )
        .bind(alias)
        .bind(canonical_id)
        .bind(source)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Grava um snapshot de precificação e telemetria.
    pub async fn record_pricing_snapshot(
        &self,
        s: &PricingTelemetrySnapshot,
    ) -> Result<(), CloudArenaError> {
        sqlx::query(
            r#"
            INSERT INTO pricing_telemetry_snapshots (
                canonical_id, source, prompt_price_per_m, completion_price_per_m,
                cache_read_price_per_m, cache_write_price_per_m, ttft_ms, throughput_tps, recorded_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?);
            "#,
        )
        .bind(&s.canonical_id)
        .bind(&s.source)
        .bind(s.prompt_price_per_m)
        .bind(s.completion_price_per_m)
        .bind(s.cache_read_price_per_m)
        .bind(s.cache_write_price_per_m)
        .bind(s.ttft_ms)
        .bind(s.throughput_tps)
        .bind(&s.recorded_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Grava um snapshot de benchmarks cognitivos.
    pub async fn record_benchmark_snapshot(
        &self,
        b: &BenchmarkSnapshot,
    ) -> Result<(), CloudArenaError> {
        sqlx::query(
            r#"
            INSERT INTO benchmark_snapshots (
                canonical_id, general_elo, coding_elo, hard_prompts_elo,
                bfcl_score, ifeval_score, gpqa_score, vqa_multimodal_score, recorded_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?);
            "#,
        )
        .bind(&b.canonical_id)
        .bind(b.general_elo)
        .bind(b.coding_elo)
        .bind(b.hard_prompts_elo)
        .bind(b.bfcl_score)
        .bind(b.ifeval_score)
        .bind(b.gpqa_score)
        .bind(b.vqa_multimodal_score)
        .bind(&b.recorded_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Grava uma avaliação diária de Pareto.
    pub async fn record_daily_evaluation(
        &self,
        e: &DailyEvaluation,
    ) -> Result<(), CloudArenaError> {
        sqlx::query(
            r#"
            INSERT INTO daily_evaluations (
                canonical_id, evaluation_date, profile_task, composite_score,
                task_blended_cost, roi_index, is_pareto_optimal, pareto_tag
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?);
            "#,
        )
        .bind(&e.canonical_id)
        .bind(&e.evaluation_date)
        .bind(e.profile_task.as_str())
        .bind(e.composite_score)
        .bind(e.task_blended_cost)
        .bind(e.roi_index)
        .bind(e.is_pareto_optimal as i32)
        .bind(e.pareto_tag.as_str())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Obtém a data do último snapshot gravado.
    pub async fn get_last_sync_date(&self) -> Result<Option<String>, CloudArenaError> {
        let row = sqlx::query("SELECT MAX(evaluation_date) as last_date FROM daily_evaluations;")
            .fetch_optional(&self.pool)
            .await?;

        if let Some(r) = row {
            let last_date: Option<String> = r.try_get("last_date").ok().flatten();
            return Ok(last_date);
        }
        Ok(None)
    }

    /// Obtém todos os aliases cadastrados.
    pub async fn get_model_aliases(&self) -> Result<Vec<(String, String)>, CloudArenaError> {
        let rows = sqlx::query("SELECT alias, canonical_id FROM model_aliases;")
            .fetch_all(&self.pool)
            .await?;

        let mut list = Vec::new();
        for r in rows {
            let alias: String = r.get("alias");
            let canonical_id: String = r.get("canonical_id");
            list.push((alias, canonical_id));
        }
        Ok(list)
    }

    /// Obtém os campeões de um perfil para a data mais recente.
    pub async fn get_champions(
        &self,
        profile: TaskProfileKind,
        date: &str,
    ) -> Result<ProfileChampionReport, CloudArenaError> {
        let rows = sqlx::query(
            r#"
            SELECT 
                de.canonical_id,
                cm.display_name,
                de.composite_score,
                de.task_blended_cost,
                de.roi_index,
                de.pareto_tag,
                cm.context_window,
                (SELECT source FROM pricing_telemetry_snapshots pts 
                 WHERE pts.canonical_id = de.canonical_id 
                 ORDER BY pts.recorded_at DESC LIMIT 1) as best_provider,
                (SELECT throughput_tps FROM pricing_telemetry_snapshots pts 
                 WHERE pts.canonical_id = de.canonical_id 
                 ORDER BY pts.recorded_at DESC LIMIT 1) as tps
            FROM daily_evaluations de
            JOIN canonical_models cm ON cm.id = de.canonical_id
            WHERE de.profile_task = ? AND de.evaluation_date = ?
            ORDER BY de.composite_score DESC;
            "#,
        )
        .bind(profile.as_str())
        .bind(date)
        .fetch_all(&self.pool)
        .await?;

        let mut leader = None;
        let mut best_val = None;
        let mut ultra_low = None;

        for r in rows {
            let canonical_id: String = r.get("canonical_id");
            let display_name: String = r.get("display_name");
            let composite_score: f64 = r.get("composite_score");
            let blended_cost: f64 = r.get("task_blended_cost");
            let roi_index: f64 = r.get("roi_index");
            let pareto_tag_str: String = r.get("pareto_tag");
            let context_window: i64 = r.get("context_window");
            let provider: String = r.try_get("best_provider").unwrap_or_else(|_| "openrouter".to_string());
            let throughput_tps: Option<f64> = r.try_get("tps").ok();

            let entry = ChampionEntry {
                canonical_id,
                display_name,
                provider,
                composite_score,
                blended_cost,
                roi_index,
                throughput_tps,
                context_window,
            };

            match pareto_tag_str.as_str() {
                "Leader" => {
                    if leader.is_none() {
                        leader = Some(entry);
                    }
                }
                "BestValue" => {
                    if best_val.is_none() {
                        best_val = Some(entry);
                    }
                }
                "UltraLowCost" => {
                    if ultra_low.is_none() {
                        ultra_low = Some(entry);
                    }
                }
                _ => {}
            }
        }

        Ok(ProfileChampionReport {
            profile,
            quality_leader: leader,
            best_value: best_val,
            ultra_low_cost: ultra_low,
        })
    }
}
