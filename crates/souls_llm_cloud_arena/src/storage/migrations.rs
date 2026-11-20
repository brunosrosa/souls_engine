//! # migrations.rs — DDL e Migrações do FrankenSQLite STRICT WAL
//!
//! Criação das 5 tabelas relacionais em modo STRICT no banco SSOT souls_heuristic_vault.db.

use crate::error::CloudArenaError;
use sqlx::SqlitePool;

pub async fn run_migrations(pool: &SqlitePool) -> Result<(), CloudArenaError> {
    // PRAGMAs obrigatórios para concorrência limpa no Windows NT
    sqlx::query("PRAGMA journal_mode = WAL;").execute(pool).await?;
    sqlx::query("PRAGMA synchronous = NORMAL;").execute(pool).await?;
    sqlx::query("PRAGMA busy_timeout = 5000;").execute(pool).await?;
    sqlx::query("PRAGMA foreign_keys = ON;").execute(pool).await?;

    let ddl = r#"
    CREATE TABLE IF NOT EXISTS canonical_models (
        id TEXT PRIMARY KEY NOT NULL,
        display_name TEXT NOT NULL,
        family TEXT NOT NULL,
        developer TEXT NOT NULL,
        context_window INTEGER NOT NULL,
        supports_tools INTEGER NOT NULL DEFAULT 0,
        supports_structured_outputs INTEGER NOT NULL DEFAULT 0,
        supports_caching INTEGER NOT NULL DEFAULT 0,
        is_multimodal INTEGER NOT NULL DEFAULT 0,
        created_at TEXT NOT NULL
    ) STRICT;

    CREATE TABLE IF NOT EXISTS model_aliases (
        alias TEXT PRIMARY KEY NOT NULL,
        canonical_id TEXT NOT NULL,
        source TEXT NOT NULL,
        FOREIGN KEY (canonical_id) REFERENCES canonical_models(id) ON DELETE CASCADE
    ) STRICT;

    CREATE TABLE IF NOT EXISTS pricing_telemetry_snapshots (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        canonical_id TEXT NOT NULL,
        source TEXT NOT NULL,
        prompt_price_per_m REAL NOT NULL,
        completion_price_per_m REAL NOT NULL,
        cache_read_price_per_m REAL NOT NULL DEFAULT 0.0,
        cache_write_price_per_m REAL NOT NULL DEFAULT 0.0,
        ttft_ms REAL,
        throughput_tps REAL,
        recorded_at TEXT NOT NULL,
        FOREIGN KEY (canonical_id) REFERENCES canonical_models(id) ON DELETE CASCADE
    ) STRICT;

    CREATE TABLE IF NOT EXISTS benchmark_snapshots (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        canonical_id TEXT NOT NULL,
        general_elo REAL,
        coding_elo REAL,
        hard_prompts_elo REAL,
        bfcl_score REAL,
        ifeval_score REAL,
        gpqa_score REAL,
        vqa_multimodal_score REAL,
        recorded_at TEXT NOT NULL,
        FOREIGN KEY (canonical_id) REFERENCES canonical_models(id) ON DELETE CASCADE
    ) STRICT;

    CREATE TABLE IF NOT EXISTS daily_evaluations (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        canonical_id TEXT NOT NULL,
        evaluation_date TEXT NOT NULL,
        profile_task TEXT NOT NULL,
        composite_score REAL NOT NULL,
        task_blended_cost REAL NOT NULL,
        roi_index REAL NOT NULL,
        is_pareto_optimal INTEGER NOT NULL,
        pareto_tag TEXT NOT NULL,
        FOREIGN KEY (canonical_id) REFERENCES canonical_models(id) ON DELETE CASCADE
    ) STRICT;

    CREATE TABLE IF NOT EXISTS hermes_usage_ledger (
        usage_key TEXT PRIMARY KEY NOT NULL,
        session_id TEXT NOT NULL,
        profile_name TEXT NOT NULL,
        source_channel TEXT NOT NULL,
        model_id TEXT NOT NULL,
        canonical_id TEXT NOT NULL,
        billing_provider TEXT NOT NULL,
        task_kind TEXT NOT NULL,
        api_call_count INTEGER NOT NULL DEFAULT 0,
        input_tokens INTEGER NOT NULL DEFAULT 0,
        output_tokens INTEGER NOT NULL DEFAULT 0,
        cache_read_tokens INTEGER NOT NULL DEFAULT 0,
        cache_write_tokens INTEGER NOT NULL DEFAULT 0,
        reasoning_tokens INTEGER NOT NULL DEFAULT 0,
        hermes_reported_cost_usd REAL NOT NULL DEFAULT 0.0,
        computed_cost_usd REAL NOT NULL DEFAULT 0.0,
        cache_saved_usd REAL NOT NULL DEFAULT 0.0,
        first_seen_ts REAL NOT NULL,
        last_seen_ts REAL NOT NULL,
        synced_at TEXT NOT NULL
    ) STRICT;

    DELETE FROM daily_evaluations
    WHERE id NOT IN (
        SELECT MAX(id)
        FROM daily_evaluations
        GROUP BY canonical_id, evaluation_date, profile_task
    );

    CREATE INDEX IF NOT EXISTS idx_pricing_lookup ON pricing_telemetry_snapshots(canonical_id, recorded_at);
    CREATE INDEX IF NOT EXISTS idx_pricing_source_date ON pricing_telemetry_snapshots(source, recorded_at, canonical_id);
    CREATE INDEX IF NOT EXISTS idx_bench_lookup ON benchmark_snapshots(canonical_id, recorded_at);
    CREATE INDEX IF NOT EXISTS idx_eval_date ON daily_evaluations(evaluation_date, profile_task);
    CREATE UNIQUE INDEX IF NOT EXISTS idx_daily_eval_unique ON daily_evaluations(canonical_id, evaluation_date, profile_task);
    CREATE INDEX IF NOT EXISTS idx_hermes_usage_last_seen ON hermes_usage_ledger(last_seen_ts);
    CREATE INDEX IF NOT EXISTS idx_hermes_usage_provider ON hermes_usage_ledger(billing_provider, last_seen_ts);
    "#;

    for statement in ddl.split(';') {
        let trimmed = statement.trim();
        if !trimmed.is_empty() {
            sqlx::query(trimmed).execute(pool).await?;
        }
    }

    Ok(())
}
