//! Chyros Daemon: Langevin Decay metabolism, STABLE partition immunity,
//! coordinated vector/relational purge, and NT Kernel-safe atomic DB rotation.

use std::f64::consts::PI;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use sqlx::{Pool, Row, Sqlite};
use tracing::{info, warn};

use crate::error::MemoryError;
use crate::sqlite::create_sqlite_pool;
use souls_core::fs::validate_refs_path;

/// Basic half-life constant tau in days.
pub const LANGEVIN_TAU_DAYS: f64 = 7.0;

/// Standard deviation sigma for thermal stochastic noise.
pub const LANGEVIN_SIGMA: f64 = 0.02;

/// Critical salience threshold triggering coordinated purge.
pub const PURGE_SALIENCE_THRESHOLD: f32 = 0.15;

/// Report generated upon completion of a Chyros metabolism cycle.
#[derive(Debug, Clone, PartialEq)]
pub struct ChyrosMetabolismReport {
    pub stable_nodes_preserved: usize,
    pub evolving_nodes_decayed: usize,
    pub purged_node_ids: Vec<String>,
    pub vacuum_compacted: bool,
    pub atomic_swap_succeeded: bool,
}

/// Deterministic pseudo-random Gaussian generator using Box-Muller transformation.
pub fn gaussian_noise(seed: u64, sigma: f64) -> f64 {
    // SplitMix64-inspired state progression
    let mut z1 = seed.wrapping_add(0x9E3779B97F4A7C15);
    z1 = (z1 ^ (z1 >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    z1 = (z1 ^ (z1 >> 27)).wrapping_mul(0x94D049BB133111EB);
    z1 = z1 ^ (z1 >> 31);

    let mut z2 = z1.wrapping_add(0x9E3779B97F4A7C15);
    z2 = (z2 ^ (z2 >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    z2 = (z2 ^ (z2 >> 27)).wrapping_mul(0x94D049BB133111EB);
    z2 = z2 ^ (z2 >> 31);

    let u1 = (((z1 & 0x001F_FFFF_FFFF_FFFF) as f64) + 1.0) / ((0x0020_0000_0000_0000u64) as f64);
    let u2 = (((z2 & 0x001F_FFFF_FFFF_FFFF) as f64) + 1.0) / ((0x0020_0000_0000_0000u64) as f64);

    let r = (-2.0 * u1.ln()).sqrt();
    let theta = 2.0 * PI * u2;

    r * theta.cos() * sigma
}

/// Calculates Langevin salience evolution:
/// S_m(t + dt) = S_m(t) * exp(-dt / (tau * (access_count + 1))) + N(0, sigma^2)
///
/// Golden Rule: STABLE partition has absolute immunity (salience fixed at 1.0).
pub fn compute_langevin_salience(
    current_salience: f32,
    partition: &str,
    delta_days: f64,
    access_count: i64,
    step_seed: u64,
) -> f32 {
    if partition == "STABLE" {
        return 1.0;
    }

    let dt = delta_days.max(0.0);
    let effective_tau = LANGEVIN_TAU_DAYS * (access_count.max(0) + 1) as f64;
    let decay_factor = (-dt / effective_tau).exp();

    let noise = gaussian_noise(step_seed, LANGEVIN_SIGMA);
    let new_salience = (current_salience as f64 * decay_factor) + noise;

    new_salience.clamp(0.0, 1.0) as f32
}

/// Chyros Daemon engine managing background memory metabolism and atomic compaction.
#[derive(Debug)]
pub struct ChyrosDaemon {
    db_path: PathBuf,
    cycle_counter: Arc<AtomicU64>,
}

impl ChyrosDaemon {
    /// Creates a new ChyrosDaemon instance bound to the master database path.
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self, MemoryError> {
        let p = db_path.as_ref();
        validate_refs_path(p)?;

        Ok(Self {
            db_path: p.to_path_buf(),
            cycle_counter: Arc::new(AtomicU64::new(1)),
        })
    }

    /// Creates an in-memory ChyrosDaemon instance for testing.
    pub fn new_in_memory() -> Self {
        Self {
            db_path: PathBuf::from(":memory:"),
            cycle_counter: Arc::new(AtomicU64::new(1)),
        }
    }

    /// Performs the full asynchronous Langevin metabolism cycle over the given SQLite pool.
    pub async fn run_metabolism_cycle(
        &self,
        pool: &Pool<Sqlite>,
        now_epoch_sec: i64,
    ) -> Result<(ChyrosMetabolismReport, Option<Pool<Sqlite>>), MemoryError> {
        info!("Starting Chyros Langevin metabolism cycle...");

        let rows = sqlx::query(
            "SELECT id, partition, salience, access_count, last_accessed_at FROM epistemic_memories",
        )
        .fetch_all(pool)
        .await?;

        let mut stable_preserved = 0;
        let mut evolving_decayed = 0;
        let mut purged_ids = Vec::new();
        let mut updates = Vec::new();

        let cycle = self.cycle_counter.fetch_add(1, Ordering::SeqCst);

        for (idx, row) in rows.iter().enumerate() {
            let id: String = row.get(0);
            let partition: String = row.get(1);
            let salience: f32 = row.get(2);
            let access_count: i64 = row.get(3);
            let last_accessed_at: i64 = row.get(4);

            if partition == "STABLE" {
                stable_preserved += 1;
                continue;
            }

            // Delta time in days
            let elapsed_sec = (now_epoch_sec - last_accessed_at).max(0);
            let delta_days = (elapsed_sec as f64) / 86400.0;
            let seed = (cycle ^ ((idx as u64) << 16)) ^ (now_epoch_sec as u64);

            let new_salience = compute_langevin_salience(
                salience,
                &partition,
                delta_days,
                access_count,
                seed,
            );

            if new_salience < PURGE_SALIENCE_THRESHOLD {
                purged_ids.push(id);
            } else {
                evolving_decayed += 1;
                updates.push((id, new_salience));
            }
        }

        // Apply salience decay updates
        for (id, new_sal) in updates {
            sqlx::query(
                "UPDATE epistemic_memories SET salience = ?1, updated_at = ?2 WHERE id = ?3",
            )
            .bind(new_sal)
            .bind(now_epoch_sec)
            .bind(&id)
            .execute(pool)
            .await?;
        }

        // Apply coordinated purge: delete from FrankenSQLite (FTS5 triggers execute automatically via rowid)
        for purged_id in &purged_ids {
            sqlx::query("DELETE FROM epistemic_memories WHERE id = ?1")
                .bind(purged_id)
                .execute(pool)
                .await?;
        }

        info!(
            "Metabolism completed: stable={}, evolving={}, purged={}",
            stable_preserved, evolving_decayed, purged_ids.len()
        );

        // Perform NT-safe atomic compaction if file-backed
        let (vacuum_ok, swap_ok, new_pool) = if self.db_path.to_string_lossy() == ":memory:" {
            (true, true, None)
        } else {
            match self.rotate_and_compact_db(pool).await {
                Ok(fresh_pool) => (true, true, Some(fresh_pool)),
                Err(e) => {
                    warn!("Atomic compaction skipped or failed: {e}");
                    (false, false, None)
                }
            }
        };

        let report = ChyrosMetabolismReport {
            stable_nodes_preserved: stable_preserved,
            evolving_nodes_decayed: evolving_decayed,
            purged_node_ids: purged_ids,
            vacuum_compacted: vacuum_ok,
            atomic_swap_succeeded: swap_ok,
        };

        Ok((report, new_pool))
    }

    /// Simulates a progression of N consecutive metabolism days over an in-memory or live pool
    /// returning the chronological sequence of metabolism reports.
    pub async fn simulate_multiday_progression(
        &self,
        pool: &Pool<Sqlite>,
        start_epoch_sec: i64,
        days: usize,
    ) -> Result<Vec<ChyrosMetabolismReport>, MemoryError> {
        let mut reports = Vec::with_capacity(days);
        let mut current_epoch = start_epoch_sec;
        for _ in 0..days {
            current_epoch += 86400;
            let (rep, _) = self.run_metabolism_cycle(pool, current_epoch).await?;
            reports.push(rep);
        }
        Ok(reports)
    }

    /// Executes NT-safe VACUUM INTO followed by pool closure and atomic Win32 MoveFileExW swap.
    pub async fn rotate_and_compact_db(
        &self,
        pool: &Pool<Sqlite>,
    ) -> Result<Pool<Sqlite>, MemoryError> {
        let compact_path = self.db_path.with_extension("compact.db");
        let compact_path_str = compact_path.to_string_lossy().to_string();

        if compact_path.exists() {
            let _ = std::fs::remove_file(&compact_path);
        }

        // 1. Execute VACUUM INTO on active connection
        let vacuum_sql = format!("VACUUM INTO '{}';", compact_path_str.replace('\'', "''"));
        sqlx::raw_sql(&vacuum_sql).execute(pool).await?;

        // 2. Explicitly close active pool and release all NT file descriptors
        pool.close().await;

        // Brief yield to ensure OS descriptor table flushes
        tokio::time::sleep(Duration::from_millis(50)).await;

        // 3. Atomically replace the database file using Win32 NT kernel MoveFileExW
        #[cfg(windows)]
        {
            use std::os::windows::ffi::OsStrExt;
            let src_wide: Vec<u16> = compact_path
                .as_os_str()
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();
            let dst_wide: Vec<u16> = self
                .db_path
                .as_os_str()
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();

            // SAFETY: MoveFileExW is invoked with valid null-terminated UTF-16 wide strings
            // and the MOVEFILE_REPLACE_EXISTING flag for atomic i-node replacement in NT kernel.
            let ok = unsafe {
                windows_sys::Win32::Storage::FileSystem::MoveFileExW(
                    src_wide.as_ptr(),
                    dst_wide.as_ptr(),
                    windows_sys::Win32::Storage::FileSystem::MOVEFILE_REPLACE_EXISTING,
                )
            };

            if ok == 0 {
                // SAFETY: Retrieving last Win32 error code on failure.
                let err_code = unsafe { windows_sys::Win32::Foundation::GetLastError() };
                return Err(MemoryError::Win32Error {
                    code: err_code,
                    message: format!("MoveFileExW atomic swap failed on {}", self.db_path.display()),
                });
            }
        }

        #[cfg(not(windows))]
        {
            std::fs::rename(&compact_path, &self.db_path)?;
        }

        // 4. Re-instantiate the SqlitePool pointing to the freshly compacted master DB
        let new_pool = create_sqlite_pool(&self.db_path, 16).await?;
        info!("Database successfully compacted and rotated atomically.");

        Ok(new_pool)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};
    use crate::sqlite::create_in_memory_sqlite_pool;

    #[test]
    fn test_stable_partition_immunity() {
        // Even after 365 days of zero access, STABLE remains 1.0
        let salience = compute_langevin_salience(1.0, "STABLE", 365.0, 0, 12345);
        assert_eq!(salience, 1.0, "STABLE memories must be completely immutable");
    }

    #[test]
    fn test_evolving_decay_and_purge_under_threshold() {
        // Over 90 days without access, salience decays exponentially
        let initial = 0.5;
        let decayed = compute_langevin_salience(initial, "EVOLVING", 60.0, 0, 42);
        assert!(
            decayed < initial,
            "EVOLVING salience must decrease over time"
        );
        assert!(
            decayed < PURGE_SALIENCE_THRESHOLD,
            "After 60 days without access, salience should drop below threshold (got {decayed})"
        );
    }

    #[tokio::test]
    async fn test_metabolism_cycle_execution_and_purge() {
        let pool = create_in_memory_sqlite_pool().await.unwrap();

        // 1. Insert 1 STABLE memory and 2 EVOLVING memories (one fresh, one very old)
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        // STABLE
        sqlx::query(
            r#"
            INSERT INTO epistemic_memories 
            (id, session_id, category, content, partition, salience, access_count, created_at, last_accessed_at, updated_at)
            VALUES ('mem_st', 's1', 'arch', 'Immutable rule', 'STABLE', 1.0, 0, ?1, ?1, ?1)
            "#,
        )
        .bind(now)
        .execute(&pool)
        .await
        .unwrap();

        // EVOLVING Fresh (accessed just now)
        sqlx::query(
            r#"
            INSERT INTO epistemic_memories 
            (id, session_id, category, content, partition, salience, access_count, created_at, last_accessed_at, updated_at)
            VALUES ('mem_ev_fresh', 's1', 'chat', 'Fresh context', 'EVOLVING', 0.9, 10, ?1, ?1, ?1)
            "#,
        )
        .bind(now)
        .execute(&pool)
        .await
        .unwrap();

        // EVOLVING Ancient (accessed 120 days ago, low initial salience)
        let ancient = now - (120 * 86400);
        sqlx::query(
            r#"
            INSERT INTO epistemic_memories 
            (id, session_id, category, content, partition, salience, access_count, created_at, last_accessed_at, updated_at)
            VALUES ('mem_ev_ancient', 's1', 'chat', 'Obsolete trivia', 'EVOLVING', 0.16, 0, ?1, ?1, ?1)
            "#,
        )
        .bind(ancient)
        .execute(&pool)
        .await
        .unwrap();

        let daemon = ChyrosDaemon::new_in_memory();

        let (report, _) = daemon.run_metabolism_cycle(&pool, now).await.unwrap();

        assert_eq!(report.stable_nodes_preserved, 1);
        assert!(report.purged_node_ids.contains(&"mem_ev_ancient".to_string()));

        // Verify STABLE node in DB is still 1.0
        let st_row = sqlx::query("SELECT salience FROM epistemic_memories WHERE id = 'mem_st'")
            .fetch_one(&pool)
            .await
            .unwrap();
        let st_sal: f32 = st_row.get(0);
        assert_eq!(st_sal, 1.0);

        // Verify ancient node was deleted from DB
        let ancient_row = sqlx::query("SELECT id FROM epistemic_memories WHERE id = 'mem_ev_ancient'")
            .fetch_optional(&pool)
            .await
            .unwrap();
        assert!(ancient_row.is_none(), "Ancient node must be purged from database");
    }

    #[tokio::test]
    async fn test_atomic_swap_nt_kernel_no_sharing_violation() {
        // Create an isolated test file on Dev Drive (Z:\)
        let test_db_path = PathBuf::from("Z:\\souls_engine\\.souls_data\\db\\test_swap.db");
        if let Some(parent) = test_db_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if test_db_path.exists() {
            let _ = std::fs::remove_file(&test_db_path);
        }

        let pool = create_sqlite_pool(&test_db_path, 4).await.unwrap();

        // Populate minimal data
        sqlx::query(
            r#"
            INSERT INTO epistemic_memories 
            (id, session_id, category, content, partition, salience, access_count, created_at, last_accessed_at, updated_at)
            VALUES ('mem_swap_1', 's1', 'test', 'Pre-swap data', 'STABLE', 1.0, 0, 100, 100, 100)
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        let daemon = ChyrosDaemon::new(&test_db_path).unwrap();

        // Execute rotation and atomic swap
        let new_pool_res = daemon.rotate_and_compact_db(&pool).await;
        assert!(
            new_pool_res.is_ok(),
            "Atomic swap must succeed without sharing violation: {:?}",
            new_pool_res.err()
        );

        let new_pool = new_pool_res.unwrap();
        let row = sqlx::query("SELECT content FROM epistemic_memories WHERE id = 'mem_swap_1'")
            .fetch_one(&new_pool)
            .await
            .unwrap();
        let content: String = row.get(0);
        assert_eq!(content, "Pre-swap data");

        // Clean up test DB
        new_pool.close().await;
        let _ = std::fs::remove_file(&test_db_path);
        let _ = std::fs::remove_file(test_db_path.with_extension("db-wal"));
        let _ = std::fs::remove_file(test_db_path.with_extension("db-shm"));
    }

    #[test]
    fn test_langevin_gaussian_stochasticity_distribution() {
        let n = 10_000;
        let sigma = LANGEVIN_SIGMA;
        let mut sum = 0.0;
        let mut sum_sq = 0.0;

        for i in 0..n {
            let val = gaussian_noise(i as u64, sigma);
            assert!(val.is_finite(), "Gaussian noise must never be NaN or Inf");
            sum += val;
            sum_sq += val * val;
        }

        let mean = sum / (n as f64);
        let variance = (sum_sq / (n as f64)) - (mean * mean);
        let stddev = variance.sqrt();

        assert!(
            mean.abs() < 0.01,
            "Gaussian noise mean should be close to 0.0, got: {mean}"
        );
        assert!(
            (stddev - sigma).abs() < 0.005,
            "Gaussian noise stddev should match sigma ({sigma}), got: {stddev}"
        );
    }

    #[test]
    fn test_langevin_access_count_half_life_protection() {
        let delta_days = 30.0;
        let initial_salience = 0.85;

        let node_a = compute_langevin_salience(initial_salience, "EVOLVING", delta_days, 0, 100);
        let node_b = compute_langevin_salience(initial_salience, "EVOLVING", delta_days, 5, 100);
        let node_c = compute_langevin_salience(initial_salience, "EVOLVING", delta_days, 20, 100);

        // Half-life expansion guarantees monotonic retention with access count
        assert!(
            node_c > node_b,
            "Node C (20 accesses, salience {node_c}) must retain more than Node B (5 accesses, salience {node_b})"
        );
        assert!(
            node_b > node_a,
            "Node B (5 accesses, salience {node_b}) must retain more than Node A (0 accesses, salience {node_a})"
        );
        assert!(
            node_a < PURGE_SALIENCE_THRESHOLD,
            "Node A with 0 accesses after 30 days should fall below purge threshold (got {node_a})"
        );
        assert!(
            node_c > 0.65,
            "Node C with 20 accesses should strongly retain salience (got {node_c})"
        );
    }

    #[tokio::test]
    async fn test_autodream_langevin_stress_1000_nodes_multi_cycle() {
        let pool = create_in_memory_sqlite_pool().await.unwrap();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        // 1. Bulk insert 1,000 nodes in a single atomic transaction
        let mut tx = pool.begin().await.unwrap();

        // 200 STABLE nodes
        for i in 0..200 {
            sqlx::query(
                "INSERT INTO epistemic_memories 
                (id, session_id, category, content, partition, salience, access_count, created_at, last_accessed_at, updated_at)
                VALUES (?1, 's_stress', 'arch', 'Stable rule content', 'STABLE', 1.0, 0, ?2, ?2, ?2)",
            )
            .bind(format!("stable_{i}"))
            .bind(now - (i as i64 * 86400))
            .execute(&mut *tx)
            .await
            .unwrap();
        }

        // 400 EVOLVING Active nodes (high access count = 20, accessed 3 days ago)
        for i in 0..400 {
            sqlx::query(
                "INSERT INTO epistemic_memories 
                (id, session_id, category, content, partition, salience, access_count, created_at, last_accessed_at, updated_at)
                VALUES (?1, 's_stress', 'code', 'Active working knowledge', 'EVOLVING', 0.9, 20, ?2, ?3, ?2)",
            )
            .bind(format!("active_{i}"))
            .bind(now - 30 * 86400)
            .bind(now - 3 * 86400)
            .execute(&mut *tx)
            .await
            .unwrap();
        }

        // 400 EVOLVING Decaying nodes (0 access count, accessed 75 days ago)
        for i in 0..400 {
            sqlx::query(
                "INSERT INTO epistemic_memories 
                (id, session_id, category, content, partition, salience, access_count, created_at, last_accessed_at, updated_at)
                VALUES (?1, 's_stress', 'chat', 'Obsolete transient chat', 'EVOLVING', 0.35, 0, ?2, ?2, ?2)",
            )
            .bind(format!("decaying_{i}"))
            .bind(now - 75 * 86400)
            .execute(&mut *tx)
            .await
            .unwrap();
        }

        tx.commit().await.unwrap();

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM epistemic_memories")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 1000, "Database must hold exactly 1,000 memories before cycle");

        // 2. Measure execution performance of the 1,000-node metabolism cycle
        let daemon = ChyrosDaemon::new_in_memory();
        let start = std::time::Instant::now();
        let (report, _) = daemon.run_metabolism_cycle(&pool, now).await.unwrap();
        let duration_ms = start.elapsed().as_millis();

        // 3. Assert rigorous SLAs and architectural invariants
        assert!(
            duration_ms < 500,
            "1,000-node Langevin metabolism must complete in < 500ms (got {duration_ms}ms)"
        );
        assert_eq!(
            report.stable_nodes_preserved, 200,
            "All 200 STABLE nodes must be preserved with 0 drift"
        );
        assert_eq!(
            report.evolving_nodes_decayed, 400,
            "All 400 active EVOLVING nodes must be safely decayed without purging"
        );
        assert_eq!(
            report.purged_node_ids.len(), 400,
            "All 400 obsolete EVOLVING nodes must be cleanly purged"
        );

        // 4. Verify database state post-purge
        let final_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM epistemic_memories")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(final_count, 600, "Remaining memories must be 600 (200 STABLE + 400 Active)");

        // 5. Verify STABLE nodes salience integrity
        let min_stable: f32 = sqlx::query_scalar(
            "SELECT MIN(salience) FROM epistemic_memories WHERE partition = 'STABLE'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(min_stable, 1.0, "STABLE salience must strictly remain 1.0");

        // 6. Multi-day progression simulation over remaining 600 nodes
        let progression = daemon
            .simulate_multiday_progression(&pool, now, 5)
            .await
            .unwrap();
        assert_eq!(progression.len(), 5);
        for day_rep in progression {
            assert_eq!(day_rep.stable_nodes_preserved, 200);
        }
    }
}
