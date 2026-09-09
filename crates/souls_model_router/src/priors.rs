//! Database persistence and Bayesian prior management for ParetoBandit and Model Registry.

use sqlx::{Pool, Row, Sqlite};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::instrument;

use crate::error::RouterError;

/// Canonical DDL initializing the two financial sovereignty tables in FrankenSQLite STRICT mode.
pub const ROUTER_DDL: &str = r#"
CREATE TABLE IF NOT EXISTS pareto_bandit_priors (
    tier_name TEXT NOT NULL,
    task_category TEXT NOT NULL,
    alpha REAL NOT NULL CHECK(alpha >= 1.0),
    beta REAL NOT NULL CHECK(beta >= 1.0),
    pull_count INTEGER NOT NULL DEFAULT 0,
    cumulative_reward REAL NOT NULL DEFAULT 0.0,
    last_updated_at INTEGER NOT NULL,
    PRIMARY KEY (tier_name, task_category)
) STRICT;

CREATE TABLE IF NOT EXISTS model_registry (
    model_id TEXT PRIMARY KEY NOT NULL,
    provider_type TEXT NOT NULL CHECK(provider_type IN ('LOCAL', 'CLOUD')),
    vram_base_mb INTEGER NOT NULL DEFAULT 0,
    kv_cache_cost_per_k INTEGER NOT NULL DEFAULT 0,
    max_context_window INTEGER NOT NULL DEFAULT 8192,
    specialty_tags TEXT NOT NULL DEFAULT '[]',
    score_lmarena INTEGER NOT NULL DEFAULT 1000,
    cost_input_per_m REAL NOT NULL DEFAULT 0.0,
    cost_output_per_m REAL NOT NULL DEFAULT 0.0,
    is_active INTEGER NOT NULL DEFAULT 1 CHECK(is_active IN (0, 1)),
    ema_latency_ms INTEGER NOT NULL DEFAULT 500,
    success_rate_ema REAL NOT NULL DEFAULT 1.0 CHECK(success_rate_ema >= 0.0 AND success_rate_ema <= 1.0)
) STRICT;
"#;

/// Static and dynamic operational parameters of a registered model arm.
#[derive(Debug, Clone, PartialEq)]
pub struct ModelRecord {
    pub model_id: String,
    pub provider_type: String,
    pub vram_base_mb: u32,
    pub kv_cache_cost_per_k: u32,
    pub max_context_window: u32,
    pub specialty_tags: String,
    pub score_lmarena: u32,
    pub cost_input_per_m: f64,
    pub cost_output_per_m: f64,
    pub is_active: bool,
    pub ema_latency_ms: u32,
    pub success_rate_ema: f64,
}

impl ModelRecord {
    /// Checks if this model belongs to a local bare-metal silicon tier.
    #[inline]
    pub fn is_local(&self) -> bool {
        self.provider_type.eq_ignore_ascii_case("LOCAL")
    }

    /// Checks if this model requires dGPU VRAM acceleration.
    #[inline]
    pub fn is_gpu_accelerated(&self) -> bool {
        self.is_local() && self.vram_base_mb > 0
    }
}

/// Posterior Beta parameters $\text{Beta}(\alpha, \beta)$ and pulling telemetry for a model tier and category.
#[derive(Debug, Clone, PartialEq)]
pub struct BanditPrior {
    pub tier_name: String,
    pub task_category: String,
    pub alpha: f64,
    pub beta: f64,
    pub pull_count: u64,
    pub cumulative_reward: f64,
    pub last_updated_at: i64,
}

impl Default for BanditPrior {
    fn default() -> Self {
        Self {
            tier_name: "tier1".to_string(),
            task_category: "general".to_string(),
            alpha: 1.0,
            beta: 1.0,
            pull_count: 0,
            cumulative_reward: 0.0,
            last_updated_at: current_timestamp_sec(),
        }
    }
}

/// Helper function to obtain current unix epoch in seconds.
#[inline]
pub fn current_timestamp_sec() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Ensures the router tables exist in the SQLite database pool in STRICT mode.
#[instrument(skip(pool))]
pub async fn ensure_router_tables(pool: &Pool<Sqlite>) -> Result<(), RouterError> {
    sqlx::raw_sql(ROUTER_DDL).execute(pool).await?;
    Ok(())
}

/// Fetches all active models (`is_active = 1`) from `model_registry`.
#[instrument(skip(pool))]
pub async fn fetch_active_models(pool: &Pool<Sqlite>) -> Result<Vec<ModelRecord>, RouterError> {
    let rows = sqlx::query(
        r#"
        SELECT model_id, provider_type, vram_base_mb, kv_cache_cost_per_k,
               max_context_window, specialty_tags, score_lmarena,
               cost_input_per_m, cost_output_per_m, is_active,
               ema_latency_ms, success_rate_ema
        FROM model_registry
        WHERE is_active = 1
        ORDER BY score_lmarena DESC
        "#,
    )
    .fetch_all(pool)
    .await?;

    let mut models = Vec::with_capacity(rows.len());
    for r in rows {
        let vram: i64 = r.get("vram_base_mb");
        let kv: i64 = r.get("kv_cache_cost_per_k");
        let max_ctx: i64 = r.get("max_context_window");
        let score: i64 = r.get("score_lmarena");
        let active: i64 = r.get("is_active");
        let ema_lat: i64 = r.get("ema_latency_ms");

        models.push(ModelRecord {
            model_id: r.get("model_id"),
            provider_type: r.get("provider_type"),
            vram_base_mb: vram as u32,
            kv_cache_cost_per_k: kv as u32,
            max_context_window: max_ctx as u32,
            specialty_tags: r.get("specialty_tags"),
            score_lmarena: score as u32,
            cost_input_per_m: r.get("cost_input_per_m"),
            cost_output_per_m: r.get("cost_output_per_m"),
            is_active: active == 1,
            ema_latency_ms: ema_lat as u32,
            success_rate_ema: r.get("success_rate_ema"),
        });
    }

    Ok(models)
}

/// Inserts or replaces a model specification in `model_registry`.
#[instrument(skip(pool, model))]
pub async fn upsert_model(pool: &Pool<Sqlite>, model: &ModelRecord) -> Result<(), RouterError> {
    sqlx::query(
        r#"
        INSERT INTO model_registry (
            model_id, provider_type, vram_base_mb, kv_cache_cost_per_k,
            max_context_window, specialty_tags, score_lmarena,
            cost_input_per_m, cost_output_per_m, is_active,
            ema_latency_ms, success_rate_ema
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
        ON CONFLICT(model_id) DO UPDATE SET
            provider_type = excluded.provider_type,
            vram_base_mb = excluded.vram_base_mb,
            kv_cache_cost_per_k = excluded.kv_cache_cost_per_k,
            max_context_window = excluded.max_context_window,
            specialty_tags = excluded.specialty_tags,
            score_lmarena = excluded.score_lmarena,
            cost_input_per_m = excluded.cost_input_per_m,
            cost_output_per_m = excluded.cost_output_per_m,
            is_active = excluded.is_active,
            ema_latency_ms = excluded.ema_latency_ms,
            success_rate_ema = excluded.success_rate_ema
        "#,
    )
    .bind(&model.model_id)
    .bind(&model.provider_type)
    .bind(model.vram_base_mb as i64)
    .bind(model.kv_cache_cost_per_k as i64)
    .bind(model.max_context_window as i64)
    .bind(&model.specialty_tags)
    .bind(model.score_lmarena as i64)
    .bind(model.cost_input_per_m)
    .bind(model.cost_output_per_m)
    .bind(if model.is_active { 1i64 } else { 0i64 })
    .bind(model.ema_latency_ms as i64)
    .bind(model.success_rate_ema)
    .execute(pool)
    .await?;

    Ok(())
}

/// Retrieves the prior $\text{Beta}(\alpha, \beta)$ for a tier and category.
/// If no prior is recorded yet, returns default uninformative prior $\text{Beta}(1.0, 1.0)$.
#[instrument(skip(pool))]
pub async fn fetch_prior(
    pool: &Pool<Sqlite>,
    tier_name: &str,
    task_category: &str,
) -> Result<BanditPrior, RouterError> {
    let row_opt = sqlx::query(
        r#"
        SELECT tier_name, task_category, alpha, beta, pull_count, cumulative_reward, last_updated_at
        FROM pareto_bandit_priors
        WHERE tier_name = ?1 AND task_category = ?2
        "#,
    )
    .bind(tier_name)
    .bind(task_category)
    .fetch_optional(pool)
    .await?;

    match row_opt {
        Some(r) => {
            let pull_count: i64 = r.get("pull_count");
            Ok(BanditPrior {
                tier_name: r.get("tier_name"),
                task_category: r.get("task_category"),
                alpha: r.get("alpha"),
                beta: r.get("beta"),
                pull_count: pull_count as u64,
                cumulative_reward: r.get("cumulative_reward"),
                last_updated_at: r.get("last_updated_at"),
            })
        }
        None => Ok(BanditPrior {
            tier_name: tier_name.to_string(),
            task_category: task_category.to_string(),
            alpha: 1.0,
            beta: 1.0,
            pull_count: 0,
            cumulative_reward: 0.0,
            last_updated_at: current_timestamp_sec(),
        }),
    }
}

/// Updates Beta priors supporting fractional rewards ($r \in [0.0, 1.0]$):
///
/// - $\alpha \leftarrow \alpha + r$
/// - $\beta \leftarrow \beta + (1.0 - r)$
/// - $\text{pull\_count} \leftarrow \text{pull\_count} + 1$
/// - $\text{cumulative\_reward} \leftarrow \text{cumulative\_reward} + r$
#[instrument(skip(pool))]
pub async fn update_bayesian_prior(
    pool: &Pool<Sqlite>,
    tier_name: &str,
    task_category: &str,
    reward: f64,
    now_epoch_sec: i64,
) -> Result<BanditPrior, RouterError> {
    let r_clamped = reward.clamp(0.0, 1.0);
    let penalty = 1.0 - r_clamped;

    // Atomic UPSERT ensuring table constraints alpha >= 1.0 and beta >= 1.0
    sqlx::query(
        r#"
        INSERT INTO pareto_bandit_priors (
            tier_name, task_category, alpha, beta, pull_count, cumulative_reward, last_updated_at
        ) VALUES (?1, ?2, 1.0 + ?3, 1.0 + ?4, 1, ?3, ?5)
        ON CONFLICT(tier_name, task_category) DO UPDATE SET
            alpha = pareto_bandit_priors.alpha + ?3,
            beta = pareto_bandit_priors.beta + ?4,
            pull_count = pareto_bandit_priors.pull_count + 1,
            cumulative_reward = pareto_bandit_priors.cumulative_reward + ?3,
            last_updated_at = ?5
        "#,
    )
    .bind(tier_name)
    .bind(task_category)
    .bind(r_clamped)
    .bind(penalty)
    .bind(now_epoch_sec)
    .execute(pool)
    .await?;

    fetch_prior(pool, tier_name, task_category).await
}

/// Updates dynamic EMA statistics for latency and success rate on the given model.
#[instrument(skip(pool))]
pub async fn update_model_telemetry(
    pool: &Pool<Sqlite>,
    model_id: &str,
    latency_ms: f64,
    success: bool,
) -> Result<(), RouterError> {
    let row_opt = sqlx::query(
        "SELECT ema_latency_ms, success_rate_ema FROM model_registry WHERE model_id = ?1",
    )
    .bind(model_id)
    .fetch_optional(pool)
    .await?;

    if let Some(r) = row_opt {
        let old_latency: i64 = r.get("ema_latency_ms");
        let old_success: f64 = r.get("success_rate_ema");

        // EWMA smoothing with \alpha = 0.20
        let alpha = 0.20;
        let new_lat = ((alpha * latency_ms) + ((1.0 - alpha) * (old_latency as f64))).round() as i64;
        let s_val = if success { 1.0 } else { 0.0 };
        let new_success = ((alpha * s_val) + ((1.0 - alpha) * old_success)).clamp(0.0, 1.0);

        sqlx::query(
            r#"
            UPDATE model_registry
            SET ema_latency_ms = ?1,
                success_rate_ema = ?2
            WHERE model_id = ?3
            "#,
        )
        .bind(new_lat.max(1))
        .bind(new_success)
        .bind(model_id)
        .execute(pool)
        .await?;
    }

    Ok(())
}
