//! SQLite Vault sync and persistence for model discovery and telemetry.

use std::path::{Path, PathBuf};
use sqlx::{sqlite::SqlitePoolOptions, Pool, Row, Sqlite};
use tracing::info;
use crate::domain::model::ModelDescriptor;
use crate::domain::result::{CognitiveEvalResult, ContextPressureResult, HardwareProfileResult};
use crate::error::ArenaError;

/// Resolves the heuristic vault database path.
pub fn resolve_vault_db_path() -> PathBuf {
    let candidate_paths = [
        PathBuf::from("Z:/souls_engine/.souls_data/db/souls_heuristic_vault.db"),
        PathBuf::from(".souls_data/db/souls_heuristic_vault.db"),
        PathBuf::from(".souls_data/souls_heuristic_vault.db"),
    ];

    for p in &candidate_paths {
        if p.exists() {
            return p.clone();
        }
    }

    // Default target
    candidate_paths[0].clone()
}

/// Manages connection pool and queries on the Heuristic Vault.
pub struct VaultManager {
    pool: Pool<Sqlite>,
}

impl VaultManager {
    /// Connects to the vault database, ensuring required tables exist.
    pub async fn connect<P: AsRef<Path>>(db_path: P) -> Result<Self, ArenaError> {
        let path = db_path.as_ref();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let db_url = format!("sqlite://{}?mode=rwc", path.to_string_lossy().replace('\\', "/"));
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&db_url)
            .await?;

        // Initialize schema if not already created
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS model_registry (
                file_path TEXT PRIMARY KEY NOT NULL,
                model_name TEXT NOT NULL,
                model_id TEXT,
                tier_classified TEXT,
                parameters TEXT,
                quantization TEXT,
                file_size_bytes INTEGER DEFAULT 0,
                context_length INTEGER DEFAULT 8192,
                has_mmproj_sidecar INTEGER DEFAULT 0,
                mmproj_file_path TEXT,
                engine_selected TEXT,
                ttft_ms REAL DEFAULT 0.0,
                tpot_ms REAL DEFAULT 0.0,
                tps REAL DEFAULT 0.0,
                vram_peak_mb REAL DEFAULT 0.0,
                e3_score REAL DEFAULT 0.0,
                is_active INTEGER DEFAULT 1,
                last_benchmarked_epoch INTEGER DEFAULT 0
            );"
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS arena_telemetry (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                file_path TEXT,
                model_id TEXT,
                benchmark_type TEXT,
                engine TEXT,
                ttft_ms REAL DEFAULT 0.0,
                tpot_ms REAL DEFAULT 0.0,
                tps REAL DEFAULT 0.0,
                vram_peak_mb REAL DEFAULT 0.0,
                ram_mb REAL DEFAULT 0.0,
                e3_score REAL DEFAULT 0.0,
                status TEXT,
                created_at INTEGER NOT NULL
            );"
        )
        .execute(&pool)
        .await?;

        // Graceful non-breaking migrations for pre-existing legacy vault tables
        let migrations = [
            "ALTER TABLE model_registry ADD COLUMN model_id TEXT;",
            "ALTER TABLE model_registry ADD COLUMN tier_classified TEXT;",
            "ALTER TABLE model_registry ADD COLUMN tps REAL DEFAULT 0.0;",
            "ALTER TABLE model_registry ADD COLUMN engine_selected TEXT;",
            "ALTER TABLE model_registry ADD COLUMN last_benchmarked_epoch INTEGER DEFAULT 0;",
            "ALTER TABLE arena_telemetry ADD COLUMN model_id TEXT;",
            "ALTER TABLE arena_telemetry ADD COLUMN benchmark_type TEXT;",
            "ALTER TABLE arena_telemetry ADD COLUMN engine TEXT;",
            "ALTER TABLE arena_telemetry ADD COLUMN tps REAL DEFAULT 0.0;",
            "ALTER TABLE arena_telemetry ADD COLUMN ram_mb REAL DEFAULT 0.0;",
            "ALTER TABLE arena_telemetry ADD COLUMN status TEXT;",
        ];
        for mig in migrations {
            let _ = sqlx::query(mig).execute(&pool).await;
        }

        Ok(Self { pool })
    }

    /// Synchronizes discovered descriptors into `model_registry`.
    pub async fn sync_models(&self, descriptors: &[ModelDescriptor]) -> Result<usize, ArenaError> {
        let mut count = 0;
        for desc in descriptors {
            let path_str = desc.file_path.to_string_lossy().to_string();
            let size_bytes = (desc.size_mb * 1024.0 * 1024.0) as i64;
            let mmproj_str = desc.mmproj_path.as_ref().map(|p| p.to_string_lossy().to_string());

            sqlx::query(
                "INSERT INTO model_registry (
                    file_path, model_name, model_id, tier_classified, parameters,
                    quantization, file_size_bytes, context_length, has_mmproj_sidecar, mmproj_file_path, is_active
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 1)
                ON CONFLICT(file_path) DO UPDATE SET
                    model_name = excluded.model_name,
                    model_id = excluded.model_id,
                    tier_classified = excluded.tier_classified,
                    parameters = excluded.parameters,
                    quantization = excluded.quantization,
                    file_size_bytes = excluded.file_size_bytes,
                    context_length = excluded.context_length,
                    has_mmproj_sidecar = excluded.has_mmproj_sidecar,
                    mmproj_file_path = excluded.mmproj_file_path;"
            )
            .bind(&path_str)
            .bind(&desc.name)
            .bind(&desc.model_id)
            .bind(desc.tier.as_str())
            .bind(&desc.parameters)
            .bind(&desc.quant)
            .bind(size_bytes)
            .bind(desc.ctx_max as i64)
            .bind(if desc.has_mmproj { 1 } else { 0 })
            .bind(mmproj_str)
            .execute(&self.pool)
            .await?;

            count += 1;
        }
        info!("Vault synchronized {} models successfully.", count);
        Ok(count)
    }

    /// Retrieves all registered models from the vault.
    pub async fn list_registered_models(&self) -> Result<Vec<ModelDescriptor>, ArenaError> {
        let rows = sqlx::query(
            "SELECT file_path, model_name,
                    COALESCE(model_id, model_name) as model_id,
                    COALESCE(tier_classified, '') as tier_classified,
                    parameters, quantization, file_size_bytes, context_length,
                    COALESCE(has_mmproj_sidecar, 0) as has_mmproj,
                    mmproj_file_path as mmproj_path
             FROM model_registry
             ORDER BY file_size_bytes ASC;"
        )
        .fetch_all(&self.pool)
        .await?;

        let mut list = Vec::new();
        for r in rows {
            let model_id: String = r.get("model_id");
            let name: String = r.get("model_name");
            let file_path: String = r.get("file_path");
            let tier_str: String = r.get("tier_classified");
            let parameters: String = r.get("parameters");
            let quant: String = r.get("quantization");
            let size_bytes: i64 = r.get("file_size_bytes");
            let ctx_max: i64 = r.get("context_length");
            let has_mmproj: i64 = r.get("has_mmproj");
            let mmproj_str: Option<String> = r.get("mmproj_path");

            let tier = crate::domain::model::ModelTier::parse(&tier_str)
                .unwrap_or(crate::domain::model::ModelTier::Tier1Master);

            list.push(ModelDescriptor {
                model_id,
                name,
                file_path: PathBuf::from(file_path),
                tier,
                parameters,
                quant,
                size_mb: size_bytes as f64 / (1024.0 * 1024.0),
                ctx_max: ctx_max as u32,
                has_mmproj: has_mmproj != 0,
                mmproj_path: mmproj_str.map(PathBuf::from),
            });
        }

        Ok(list)
    }

    /// Records hardware profile telemetry into the vault.
    pub async fn record_hardware_profile(
        &self,
        res: &HardwareProfileResult,
    ) -> Result<(), ArenaError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        // 1. Update model_registry summary
        sqlx::query(
            "UPDATE model_registry SET
                engine_selected = ?1,
                ttft_ms = ?2,
                tpot_ms = ?3,
                tps = ?4,
                vram_peak_mb = ?5,
                e3_score = ?6,
                last_benchmarked_epoch = ?7
             WHERE model_id = ?8 OR file_path = ?9;"
        )
        .bind(&res.engine)
        .bind(res.ttft_ms)
        .bind(res.tpot_ms)
        .bind(res.tps)
        .bind(res.peak_vram_mb)
        .bind(res.e3_score)
        .bind(now)
        .bind(&res.model_id)
        .bind(&res.model_path)
        .execute(&self.pool)
        .await?;

        // 2. Insert into chronological telemetry log
        sqlx::query(
            "INSERT INTO arena_telemetry (
                file_path, prompt_id, model_id, benchmark_type, engine, ttft_ms, tpot_ms, tps,
                vram_peak_mb, ram_mb, e3_score, status, created_at
            ) VALUES (?1, ?2, ?3, 'HardwareProfile', ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, datetime('now'));"
        )
        .bind(&res.model_path)
        .bind("type1_hardware_profile")
        .bind(&res.model_id)
        .bind(&res.engine)
        .bind(res.ttft_ms)
        .bind(res.tpot_ms)
        .bind(res.tps)
        .bind(res.peak_vram_mb)
        .bind(res.ram_mb)
        .bind(res.e3_score)
        .bind(&res.status)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Records cognitive evaluation result into the vault.
    pub async fn record_cognitive_eval(
        &self,
        res: &CognitiveEvalResult,
    ) -> Result<(), ArenaError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        // 1. Update model_registry specific track scores
        let col_score = if res.track.contains("2A") || res.track.contains("RustAst") {
            "score_code_ast"
        } else if res.track.contains("2B") || res.track.contains("Json") {
            "score_json_tools"
        } else {
            "score_reasoning"
        };
        let score_val = res.accuracy_pct / 100.0;

        let update_sql = format!(
            "UPDATE model_registry SET
                {} = ?1,
                last_benchmarked_epoch = ?2
             WHERE model_id = ?3 OR file_path = ?4;",
            col_score
        );

        let _ = sqlx::query(&update_sql)
            .bind(score_val)
            .bind(now)
            .bind(&res.model_id)
            .bind(&res.model_path)
            .execute(&self.pool)
            .await;

        // 2. Insert into chronological telemetry log
        sqlx::query(
            "INSERT INTO arena_telemetry (
                file_path, prompt_id, model_id, benchmark_type, engine, ttft_ms, tpot_ms, tps,
                vram_peak_mb, ram_mb, e3_score, status, created_at
            ) VALUES (?1, ?2, ?3, 'CognitiveEval', 'llama-cuda', ?4, 0.0, ?5, 0.0, 0.0, ?6, ?7, datetime('now'));"
        )
        .bind(&res.model_path)
        .bind(&res.track)
        .bind(&res.model_id)
        .bind(res.avg_ttft_ms)
        .bind(res.avg_tps)
        .bind(res.e3_score)
        .bind(&res.status)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Records context pressure evaluation result into the vault.
    pub async fn record_context_pressure(
        &self,
        res: &ContextPressureResult,
    ) -> Result<(), ArenaError> {
        let max_ctx = res.max_effective_context as f64;
        sqlx::query(
            "INSERT INTO arena_telemetry (
                file_path, prompt_id, model_id, benchmark_type, engine, ttft_ms, tpot_ms, tps,
                vram_peak_mb, ram_mb, e3_score, status, created_at
            ) VALUES (?1, 'type5_needle_pressure', ?2, 'ContextPressure', 'llama-cuda', 0.0, 0.0, 0.0, 0.0, 0.0, ?3, ?4, datetime('now'));"
        )
        .bind(&res.model_id)
        .bind(&res.model_id)
        .bind(max_ctx)
        .bind(&res.status)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

