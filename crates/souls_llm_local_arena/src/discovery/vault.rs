//! SQLite Vault sync and persistence for model discovery and telemetry.

use std::path::{Path, PathBuf};
use std::collections::{HashMap, HashSet};
use sqlx::{sqlite::SqlitePoolOptions, Pool, Row, Sqlite};
use tracing::info;
use crate::domain::model::ModelDescriptor;
use crate::domain::result::{CognitiveEvalResult, ContextPressureResult, HardwareProfileResult, MultimodalVisionResult, SpeculativeResult};
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
            "ALTER TABLE model_registry ADD COLUMN score_profile_coding REAL;",
            "ALTER TABLE model_registry ADD COLUMN score_profile_agentic REAL;",
            "ALTER TABLE model_registry ADD COLUMN score_profile_reasoning REAL;",
            "ALTER TABLE model_registry ADD COLUMN score_profile_fast REAL;",
            "ALTER TABLE model_registry ADD COLUMN silicon_cost_index REAL;",
            "ALTER TABLE model_registry ADD COLUMN roi_silicon_coding REAL;",
            "ALTER TABLE model_registry ADD COLUMN roi_silicon_agentic REAL;",
            "ALTER TABLE model_registry ADD COLUMN roi_silicon_reasoning REAL;",
            "ALTER TABLE model_registry ADD COLUMN roi_silicon_fast REAL;",
            "ALTER TABLE model_registry ADD COLUMN pareto_coding_tag TEXT;",
            "ALTER TABLE model_registry ADD COLUMN pareto_agentic_tag TEXT;",
            "ALTER TABLE model_registry ADD COLUMN pareto_reasoning_tag TEXT;",
            "ALTER TABLE model_registry ADD COLUMN pareto_fast_tag TEXT;",
            "CREATE TABLE IF NOT EXISTS model_task_profiles (
                model_id TEXT PRIMARY KEY,
                file_path TEXT NOT NULL,
                tier TEXT NOT NULL,
                parameters TEXT NOT NULL,
                quantization TEXT NOT NULL,
                score_coding REAL NOT NULL,
                score_agentic REAL NOT NULL,
                score_reasoning REAL NOT NULL,
                score_fast REAL NOT NULL,
                silicon_cost REAL NOT NULL,
                roi_coding REAL NOT NULL,
                roi_agentic REAL NOT NULL,
                roi_reasoning REAL NOT NULL,
                roi_fast REAL NOT NULL,
                is_pareto_coding INTEGER NOT NULL,
                is_pareto_agentic INTEGER NOT NULL,
                is_pareto_reasoning INTEGER NOT NULL,
                is_pareto_fast INTEGER NOT NULL,
                pareto_tag_coding TEXT NOT NULL,
                pareto_tag_agentic TEXT NOT NULL,
                pareto_tag_reasoning TEXT NOT NULL,
                pareto_tag_fast TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );",
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

    /// Records multimodal vision evaluation result into the vault.
    pub async fn record_multimodal_vision(
        &self,
        res: &MultimodalVisionResult,
    ) -> Result<(), ArenaError> {
        let vqa_score = if res.status == "PAIRED_QUALIFIED" { 1.0 } else { 0.0 };
        let has_mmproj = if res.status == "PAIRED_QUALIFIED" { 1 } else { 0 };

        let _ = sqlx::query(
            "UPDATE model_registry SET
                has_mmproj_sidecar = ?1,
                mmproj_file_path = ?2,
                score_vision_vqa = ?3
             WHERE model_id = ?4 OR file_path LIKE ?5;"
        )
        .bind(has_mmproj)
        .bind(&res.sidecar_path)
        .bind(vqa_score)
        .bind(&res.model_id)
        .bind(format!("%{}%", res.model_id))
        .execute(&self.pool)
        .await;

        sqlx::query(
            "INSERT INTO arena_telemetry (
                file_path, prompt_id, model_id, benchmark_type, engine, ttft_ms, tpot_ms, tps,
                vram_peak_mb, ram_mb, e3_score, status, created_at
            ) VALUES (?1, 'type3_vlm_pairing', ?2, 'MultimodalVision', 'llama-cuda', ?3, 0.0, 0.0, 0.0, 0.0, ?4, ?5, datetime('now'));"
        )
        .bind(&res.sidecar_path)
        .bind(&res.model_id)
        .bind(res.multimodal_ttft_ms)
        .bind(res.projection_latency_ms)
        .bind(&res.status)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Records speculative draft evaluation result into the vault.
    pub async fn record_speculative_draft(
        &self,
        res: &SpeculativeResult,
    ) -> Result<(), ArenaError> {
        let _ = sqlx::query(
            "UPDATE model_registry SET
                mtp_acceptance_rate = ?1
             WHERE model_id = ?2 OR file_path LIKE ?3;"
        )
        .bind(res.acceptance_rate_alpha)
        .bind(&res.model_id)
        .bind(format!("%{}%", res.model_id))
        .execute(&self.pool)
        .await;

        sqlx::query(
            "INSERT INTO arena_telemetry (
                file_path, prompt_id, model_id, benchmark_type, engine, ttft_ms, tpot_ms, tps,
                vram_peak_mb, ram_mb, e3_score, status, created_at
            ) VALUES (?1, 'type4_draft_acceleration', ?2, 'SpeculativeDraft', 'llama-cuda', 0.0, 0.0, ?3, 0.0, 0.0, ?4, ?5, datetime('now'));"
        )
        .bind(&res.drafter_id)
        .bind(&res.model_id)
        .bind(res.speculative_tps)
        .bind(res.speedup_ratio)
        .bind(&res.verdict)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Recalcula os scores dos 4 perfis de tarefa e a Fronteira de Pareto
    /// a partir da telemetria física consolidada e persiste em `model_task_profiles`
    /// e `model_registry`.
    pub async fn recalculate_and_sync_task_profiles(&self) -> Result<usize, ArenaError> {
        let rows = sqlx::query(
            "SELECT mr.file_path,
                    COALESCE(mr.model_id, mr.model_name) as model_id,
                    COALESCE(mr.tier_classified, 'Tier 1') as tier_classified,
                    COALESCE(mr.parameters, 'Unknown') as parameters,
                    COALESCE(mr.quantization, 'GGUF') as quantization,
                    COALESCE(mr.tps, 0.0) as tps,
                    COALESCE(mr.ttft_ms, 0.0) as ttft_ms,
                    COALESCE(mr.vram_peak_mb, 0.0) as vram_peak_mb,
                    COALESCE(mr.score_code_ast, 0.0) as score_code_ast,
                    COALESCE(mr.score_json_tools, 0.0) as score_json_tools,
                    COALESCE(mr.score_reasoning, 0.0) as score_reasoning,
                    COALESCE(mr.mtp_acceptance_rate, 0.0) as mtp_acceptance_rate
             FROM model_registry mr
             WHERE mr.is_active = 1;"
        )
        .fetch_all(&self.pool)
        .await?;

        // 1. Mapeia última telemetria de pressão de contexto
        let pressure_rows = sqlx::query(
            "SELECT model_id, status FROM arena_telemetry WHERE benchmark_type = 'ContextPressure' ORDER BY id DESC;"
        )
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default();

        let mut pressure_map: HashMap<String, String> = HashMap::new();
        for pr in pressure_rows {
            let m_id: String = pr.get("model_id");
            let status: String = pr.get("status");
            pressure_map.entry(m_id).or_insert(status);
        }

        // 2. Mapeia última telemetria de speculative drafting
        let spec_rows = sqlx::query(
            "SELECT model_id, e3_score FROM arena_telemetry WHERE benchmark_type = 'SpeculativeDraft' ORDER BY id DESC;"
        )
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default();

        let mut spec_map: HashMap<String, f64> = HashMap::new();
        for sr in spec_rows {
            let m_id: String = sr.get("model_id");
            let speedup: f64 = sr.get("e3_score");
            spec_map.entry(m_id).or_insert(speedup);
        }

        const SAFE_WDDM_CEILING: f64 = 5294.0;

        struct EvaluatedModel {
            model_id: String,
            file_path: String,
            tier: String,
            parameters: String,
            quantization: String,
            score_coding: f64,
            score_agentic: f64,
            score_reasoning: f64,
            score_fast: f64,
            c_silicon: f64,
            roi_coding: f64,
            roi_agentic: f64,
            roi_reasoning: f64,
            roi_fast: f64,
        }

        let mut models = Vec::new();
        for r in rows {
            let model_id: String = r.get("model_id");
            let file_path: String = r.get("file_path");
            let tier: String = r.get("tier_classified");
            let clean_tier = tier.split('(').next().unwrap_or("Tier 1").trim().to_string();
            let parameters: String = r.get("parameters");
            let quantization: String = r.get("quantization");

            let tps: f64 = r.get("tps");
            let ttft: f64 = r.get("ttft_ms");
            let vram: f64 = r.get("vram_peak_mb");

            let ast = (r.get::<f64, _>("score_code_ast")) * 100.0;
            let json = (r.get::<f64, _>("score_json_tools")) * 100.0;
            let cot = (r.get::<f64, _>("score_reasoning")) * 100.0;
            let mtp_alpha: f64 = r.get("mtp_acceptance_rate");

            let ctx_status = pressure_map.get(&model_id).map(|s| s.as_str()).unwrap_or("UNKNOWN");
            let ctx_score = match ctx_status {
                "PASS_STABLE" => 100.0,
                "ATTENTION_DEGRADED" => 50.0,
                _ => 15.0,
            };

            let spec_speedup = *spec_map.get(&model_id).unwrap_or(&1.0);
            let spec_score = if mtp_alpha > 0.0 {
                (mtp_alpha * 100.0).min(100.0)
            } else {
                ((spec_speedup - 1.0) * 200.0).clamp(0.0, 100.0)
            };

            let s_tps_50 = (tps / 50.0 * 100.0).min(100.0);
            let s_tps_150 = (tps / 150.0 * 100.0).min(100.0);
            let s_ttft = (100.0 - ttft / 50.0).clamp(0.0, 100.0);
            let s_vram_free = ((1.0 - vram / SAFE_WDDM_CEILING) * 100.0).clamp(0.0, 100.0);

            let score_coding = 0.40 * ast + 0.25 * ctx_score + 0.20 * s_tps_50 + 0.15 * s_ttft;
            let score_agentic = 0.45 * json + 0.25 * ctx_score + 0.15 * s_ttft + 0.15 * s_tps_50;
            let score_reasoning = 0.45 * cot + 0.35 * ctx_score + 0.20 * ast;
            let score_fast = 0.45 * s_tps_150 + 0.25 * s_vram_free + 0.20 * s_ttft + 0.10 * spec_score;

            let c_silicon = 0.50 * (vram / SAFE_WDDM_CEILING) + 0.30 * (ttft / 3000.0) + 0.20 * (30.0 / (tps + 1.0));
            let damp = (1.0 + c_silicon + 1.0).ln();

            let roi_coding = if damp > 0.0 { score_coding / damp } else { 0.0 };
            let roi_agentic = if damp > 0.0 { score_agentic / damp } else { 0.0 };
            let roi_reasoning = if damp > 0.0 { score_reasoning / damp } else { 0.0 };
            let roi_fast = if damp > 0.0 { score_fast / damp } else { 0.0 };

            models.push(EvaluatedModel {
                model_id,
                file_path,
                tier: clean_tier,
                parameters,
                quantization,
                score_coding: (score_coding * 10.0).round() / 10.0,
                score_agentic: (score_agentic * 10.0).round() / 10.0,
                score_reasoning: (score_reasoning * 10.0).round() / 10.0,
                score_fast: (score_fast * 10.0).round() / 10.0,
                c_silicon: (c_silicon * 1000.0).round() / 1000.0,
                roi_coding: (roi_coding * 10.0).round() / 10.0,
                roi_agentic: (roi_agentic * 10.0).round() / 10.0,
                roi_reasoning: (roi_reasoning * 10.0).round() / 10.0,
                roi_fast: (roi_fast * 10.0).round() / 10.0,
            });
        }

        // Pareto calculation helper
        let calc_pareto = |pool: &[&EvaluatedModel], get_s: fn(&EvaluatedModel) -> f64| -> HashSet<String> {
            let mut pareto = HashSet::new();
            for m1 in pool {
                let s1 = get_s(m1);
                let c1 = m1.c_silicon;
                let mut dominated = false;
                for m2 in pool {
                    if m1.model_id == m2.model_id {
                        continue;
                    }
                    let s2 = get_s(m2);
                    let c2 = m2.c_silicon;
                    if s2 >= s1 && c2 <= c1 && (s2 > s1 || c2 < c1) {
                        dominated = true;
                        break;
                    }
                }
                if !dominated {
                    pareto.insert(m1.model_id.clone());
                }
            }
            pareto
        };

        // Assign tags helper
        let assign_tags = |pool: &[&EvaluatedModel],
                           pareto: &HashSet<String>,
                           get_s: fn(&EvaluatedModel) -> f64,
                           get_r: fn(&EvaluatedModel) -> f64|
         -> HashMap<String, String> {
            let mut by_score = pool.to_vec();
            by_score.sort_by(|a, b| get_s(b).partial_cmp(&get_s(a)).unwrap_or(std::cmp::Ordering::Equal));

            let mut by_roi = pool.to_vec();
            by_roi.sort_by(|a, b| get_r(b).partial_cmp(&get_r(a)).unwrap_or(std::cmp::Ordering::Equal));

            let quality_champ = by_score.iter().find(|m| pareto.contains(&m.model_id)).map(|m| m.model_id.clone());
            let roi_champ = by_roi.iter().find(|m| pareto.contains(&m.model_id)).map(|m| m.model_id.clone());

            let mut tags = HashMap::new();
            let mut fb_count = 1;
            for m in &by_score {
                let tag = if Some(&m.model_id) == quality_champ.as_ref() && Some(&m.model_id) == roi_champ.as_ref() {
                    "🏆 Campeão Absoluto (Qualidade & ROI)".to_string()
                } else if Some(&m.model_id) == quality_champ.as_ref() {
                    "🏆 Campeão de Qualidade".to_string()
                } else if Some(&m.model_id) == roi_champ.as_ref() {
                    "⚡ Campeão de Custo/Eficiência".to_string()
                } else if pareto.contains(&m.model_id) {
                    let t = format!("🔗 Fallback Local #{} (Pareto)", fb_count);
                    fb_count += 1;
                    t
                } else if fb_count <= 2 {
                    let t = format!("🔗 Fallback Local #{}", fb_count);
                    fb_count += 1;
                    t
                } else {
                    "⚪ Alternativa Subordinada".to_string()
                };
                tags.insert(m.model_id.clone(), tag);
            }
            tags
        };

        // 1. Coding Profile
        let coding_pool: Vec<&EvaluatedModel> = models.iter().filter(|m| m.tier == "Tier 1").collect();
        let pareto_coding = calc_pareto(&coding_pool, |m| m.score_coding);
        let tags_coding = assign_tags(&coding_pool, &pareto_coding, |m| m.score_coding, |m| m.roi_coding);

        // 2. Agentic Profile
        let agentic_pool: Vec<&EvaluatedModel> = models.iter().filter(|m| m.tier == "Tier 1").collect();
        let pareto_agentic = calc_pareto(&agentic_pool, |m| m.score_agentic);
        let tags_agentic = assign_tags(&agentic_pool, &pareto_agentic, |m| m.score_agentic, |m| m.roi_agentic);

        // 3. Reasoning Profile
        let reason_pool: Vec<&EvaluatedModel> = models.iter().filter(|m| m.tier == "Tier 1" || m.tier == "Tier 2").collect();
        let pareto_reason = calc_pareto(&reason_pool, |m| m.score_reasoning);
        let tags_reason = assign_tags(&reason_pool, &pareto_reason, |m| m.score_reasoning, |m| m.roi_reasoning);

        // 4. Fast Streaming Profile
        let fast_pool: Vec<&EvaluatedModel> = models.iter().filter(|m| m.tier == "Tier 0" || m.tier == "Tier 0.5" || m.tier == "Tier 1" || m.tier == "Tier 4").collect();
        let pareto_fast = calc_pareto(&fast_pool, |m| m.score_fast);
        let tags_fast = assign_tags(&fast_pool, &pareto_fast, |m| m.score_fast, |m| m.roi_fast);

        // Persist all into SQLite
        let mut synced = 0;

        for m in &models {
            let is_p_code = if pareto_coding.contains(&m.model_id) { 1 } else { 0 };
            let is_p_ag = if pareto_agentic.contains(&m.model_id) { 1 } else { 0 };
            let is_p_re = if pareto_reason.contains(&m.model_id) { 1 } else { 0 };
            let is_p_fa = if pareto_fast.contains(&m.model_id) { 1 } else { 0 };

            let t_code = tags_coding.get(&m.model_id).cloned().unwrap_or_else(|| "⚪ N/A".to_string());
            let t_ag = tags_agentic.get(&m.model_id).cloned().unwrap_or_else(|| "⚪ N/A".to_string());
            let t_re = tags_reason.get(&m.model_id).cloned().unwrap_or_else(|| "⚪ N/A".to_string());
            let t_fa = tags_fast.get(&m.model_id).cloned().unwrap_or_else(|| "⚪ N/A".to_string());

            sqlx::query(
                "INSERT INTO model_task_profiles (
                    model_id, file_path, tier, parameters, quantization,
                    score_coding, score_agentic, score_reasoning, score_fast,
                    silicon_cost, roi_coding, roi_agentic, roi_reasoning, roi_fast,
                    is_pareto_coding, is_pareto_agentic, is_pareto_reasoning, is_pareto_fast,
                    pareto_tag_coding, pareto_tag_agentic, pareto_tag_reasoning, pareto_tag_fast,
                    updated_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, datetime('now'))
                ON CONFLICT(model_id) DO UPDATE SET
                    file_path=excluded.file_path,
                    tier=excluded.tier,
                    parameters=excluded.parameters,
                    quantization=excluded.quantization,
                    score_coding=excluded.score_coding,
                    score_agentic=excluded.score_agentic,
                    score_reasoning=excluded.score_reasoning,
                    score_fast=excluded.score_fast,
                    silicon_cost=excluded.silicon_cost,
                    roi_coding=excluded.roi_coding,
                    roi_agentic=excluded.roi_agentic,
                    roi_reasoning=excluded.roi_reasoning,
                    roi_fast=excluded.roi_fast,
                    is_pareto_coding=excluded.is_pareto_coding,
                    is_pareto_agentic=excluded.is_pareto_agentic,
                    is_pareto_reasoning=excluded.is_pareto_reasoning,
                    is_pareto_fast=excluded.is_pareto_fast,
                    pareto_tag_coding=excluded.pareto_tag_coding,
                    pareto_tag_agentic=excluded.pareto_tag_agentic,
                    pareto_tag_reasoning=excluded.pareto_tag_reasoning,
                    pareto_tag_fast=excluded.pareto_tag_fast,
                    updated_at=datetime('now');"
            )
            .bind(&m.model_id)
            .bind(&m.file_path)
            .bind(&m.tier)
            .bind(&m.parameters)
            .bind(&m.quantization)
            .bind(m.score_coding)
            .bind(m.score_agentic)
            .bind(m.score_reasoning)
            .bind(m.score_fast)
            .bind(m.c_silicon)
            .bind(m.roi_coding)
            .bind(m.roi_agentic)
            .bind(m.roi_reasoning)
            .bind(m.roi_fast)
            .bind(is_p_code)
            .bind(is_p_ag)
            .bind(is_p_re)
            .bind(is_p_fa)
            .bind(&t_code)
            .bind(&t_ag)
            .bind(&t_re)
            .bind(&t_fa)
            .execute(&self.pool)
            .await?;

            sqlx::query(
                "UPDATE model_registry SET
                    score_profile_coding = ?1,
                    score_profile_agentic = ?2,
                    score_profile_reasoning = ?3,
                    score_profile_fast = ?4,
                    silicon_cost_index = ?5,
                    roi_silicon_coding = ?6,
                    roi_silicon_agentic = ?7,
                    roi_silicon_reasoning = ?8,
                    roi_silicon_fast = ?9,
                    pareto_coding_tag = ?10,
                    pareto_agentic_tag = ?11,
                    pareto_reasoning_tag = ?12,
                    pareto_fast_tag = ?13
                 WHERE model_id = ?14 OR file_path = ?15;"
            )
            .bind(m.score_coding)
            .bind(m.score_agentic)
            .bind(m.score_reasoning)
            .bind(m.score_fast)
            .bind(m.c_silicon)
            .bind(m.roi_coding)
            .bind(m.roi_agentic)
            .bind(m.roi_reasoning)
            .bind(m.roi_fast)
            .bind(&t_code)
            .bind(&t_ag)
            .bind(&t_re)
            .bind(&t_fa)
            .bind(&m.model_id)
            .bind(&m.file_path)
            .execute(&self.pool)
            .await?;

            synced += 1;
        }

        info!("Recalculated task profiles and synced {} models to vault SSOT.", synced);
        Ok(synced)
    }
}

