//! Frozen Memory Snapshot & Passive Hermes Bootstrap Ingestion.
//!
//! Enforces Clause 4 of `AGENTS.md`:
//! - Passive cold-boot reading of Hermes Agent's `MEMORY.md`, `USER.md`, and `SOUL.md`
//!   (direct write-back to those files is strictly forbidden).
//! - Deterministic SHA-256 content fingerprinting and idempotent upsert into the `STABLE`
//!   epistemic partition (`salience = 1.0`).
//! - Session-pinned `FrozenMemorySnapshot` so each master session or delegated subagent
//!   operates over an immutable point-in-time view (100% prefix cache preservation and
//!   zero mid-session drift), rotating only at session boundaries (`/new`, `/reset`) or
//!   explicit manual refresh.
//! - Deduplication of RRF hybrid recall results against the session's `FrozenMemorySnapshot`
//!   within a strict character budget (`top_k = 4`, `max_chars = 800`).

#![forbid(unsafe_code)]

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use arc_swap::ArcSwap;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{Pool, Sqlite};
use tracing::{debug, info};

use crate::error::MemoryError;
use crate::rrf::UnifiedMatch;
use crate::sqlite::{count_epistemic_partitions, list_epistemic_memories, upsert_epistemic_memory};
use souls_protocol::{EpistemicMemoryRecord, EpistemicPartition};

/// Default maximum number of RRF matches injected during automatic turn prefetch.
pub const DEFAULT_PREFETCH_TOP_K: usize = 4;

/// Default maximum character budget (~200 tokens) for automatic turn prefetch overlay.
pub const DEFAULT_PREFETCH_MAX_CHARS: usize = 800;

/// Raw contents passively read from the Hermes Agent home directory.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct HermesBootstrapFiles {
    pub hermes_home: Option<String>,
    pub memory_md: String,
    pub user_md: String,
    pub soul_md: String,
}

impl HermesBootstrapFiles {
    /// Passively reads `memories/MEMORY.md`, `memories/USER.md`, and `SOUL.md` from the
    /// given Hermes home directory (or resolves `%LOCALAPPDATA%\hermes` on Windows if omitted).
    /// Never modifies or creates files on disk.
    pub fn read_passive(hermes_home: Option<&Path>) -> Self {
        let resolved_home = hermes_home.map(PathBuf::from).or_else(resolve_default_hermes_home);

        let Some(home) = resolved_home else {
            return Self::default();
        };

        let memory_path = home.join("memories").join("MEMORY.md");
        let user_path = home.join("memories").join("USER.md");
        let soul_path = home.join("SOUL.md");

        let memory_md = fs::read_to_string(&memory_path).unwrap_or_default();
        let user_md = fs::read_to_string(&user_path).unwrap_or_default();
        let soul_md = fs::read_to_string(&soul_path).unwrap_or_default();

        Self {
            hermes_home: Some(home.to_string_lossy().into_owned()),
            memory_md,
            user_md,
            soul_md,
        }
    }
}

/// Resolves the default Hermes home directory (`HERMES_HOME` env or `%LOCALAPPDATA%\hermes`).
pub fn resolve_default_hermes_home() -> Option<PathBuf> {
    if let Ok(custom) = std::env::var("HERMES_HOME") {
        let p = PathBuf::from(custom.trim());
        if p.exists() {
            return Some(p);
        }
    }
    if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
        let p = PathBuf::from(local_app_data).join("hermes");
        if p.exists() {
            return Some(p);
        }
    }
    None
}

/// Immutable point-in-time snapshot of the foundational memory state pinned to a session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrozenMemorySnapshot {
    pub snapshot_id: String,
    pub composite_sha256: String,
    pub memory_md_sha256: String,
    pub user_md_sha256: String,
    pub soul_md_sha256: String,
    pub hermes_home: Option<String>,
    pub memory_entries: Vec<String>,
    pub user_entries: Vec<String>,
    pub soul_summary: String,
    pub stable_truths_count: usize,
    pub evolving_count: usize,
    pub pinned_at: i64,
    #[serde(skip)]
    pub normalized_fingerprints: HashSet<String>,
}

impl Default for FrozenMemorySnapshot {
    fn default() -> Self {
        let empty_hash = sha256_hex("");
        Self {
            snapshot_id: format!("snap_{}", &empty_hash[..16]),
            composite_sha256: empty_hash.clone(),
            memory_md_sha256: empty_hash.clone(),
            user_md_sha256: empty_hash.clone(),
            soul_md_sha256: empty_hash,
            hermes_home: None,
            memory_entries: Vec::new(),
            user_entries: Vec::new(),
            soul_summary: String::new(),
            stable_truths_count: 0,
            evolving_count: 0,
            pinned_at: 0,
            normalized_fingerprints: HashSet::new(),
        }
    }
}

/// Computes the lowercase hexadecimal SHA-256 digest of a string.
pub fn sha256_hex(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let digest = hasher.finalize();
    let mut hex = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write;
        let _ = write!(&mut hex, "{byte:02x}");
    }
    hex
}

/// Normalizes text into an alphanumeric lowercase fingerprint for O(1) deduplication.
pub fn normalize_for_dedup(input: &str) -> String {
    let mut out = String::with_capacity(input.len().min(320));
    let mut prev_space = false;
    for ch in input.chars() {
        if ch.is_alphanumeric() {
            for lower in ch.to_lowercase() {
                out.push(lower);
            }
            prev_space = false;
        } else if ch.is_whitespace() && !prev_space && !out.is_empty() {
            out.push(' ');
            prev_space = true;
        }
        if out.len() >= 320 {
            break;
        }
    }
    out.trim().to_string()
}

/// Splits a markdown memory file (`MEMORY.md` or `USER.md`) into discrete semantic entries.
/// Ignores top-level markdown comments (`<!-- ... -->`) and empty lines.
pub fn parse_markdown_entries(markdown: &str) -> Vec<String> {
    let mut entries = Vec::new();
    let mut current = String::new();

    for raw_line in markdown.lines() {
        let trimmed = raw_line.trim();
        if trimmed.is_empty() || trimmed.starts_with("<!--") || trimmed.ends_with("-->") {
            if !current.is_empty() {
                entries.push(current.trim().to_string());
                current.clear();
            }
            continue;
        }

        // Heading lines start a fresh context or get skipped if purely structural
        if trimmed.starts_with('#') {
            if !current.is_empty() {
                entries.push(current.trim().to_string());
                current.clear();
            }
            continue;
        }

        // Bullet items start a new discrete entry
        if let Some(stripped) = trimmed
            .strip_prefix("- ")
            .or_else(|| trimmed.strip_prefix("* "))
            .or_else(|| trimmed.strip_prefix("• "))
        {
            if !current.is_empty() {
                entries.push(current.trim().to_string());
                current.clear();
            }
            let item = stripped.trim();
            if !item.is_empty() {
                current.push_str(item);
            }
        } else {
            if !current.is_empty() {
                current.push(' ');
            }
            current.push_str(trimmed);
        }
    }

    if !current.is_empty() {
        entries.push(current.trim().to_string());
    }

    entries
}

/// Manages the global and session-pinned `FrozenMemorySnapshot` instances.
#[derive(Debug)]
pub struct SnapshotCoordinator {
    global_snapshot: ArcSwap<FrozenMemorySnapshot>,
    session_snapshots: DashMap<String, Arc<FrozenMemorySnapshot>>,
}

impl Default for SnapshotCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

impl SnapshotCoordinator {
    /// Creates a new `SnapshotCoordinator` with an initial default snapshot.
    pub fn new() -> Self {
        Self {
            global_snapshot: ArcSwap::from_pointee(FrozenMemorySnapshot::default()),
            session_snapshots: DashMap::new(),
        }
    }

    /// Returns the currently active global `FrozenMemorySnapshot`.
    pub fn current_global(&self) -> Arc<FrozenMemorySnapshot> {
        self.global_snapshot.load_full()
    }

    /// Returns the snapshot pinned to `session_id`, if any.
    pub fn get_session_snapshot(&self, session_id: &str) -> Option<Arc<FrozenMemorySnapshot>> {
        self.session_snapshots.get(session_id).map(|r| r.value().clone())
    }

    /// Returns the number of active session pins.
    pub fn pinned_sessions_count(&self) -> usize {
        self.session_snapshots.len()
    }

    /// Passively ingests `HermesBootstrapFiles` into the `STABLE` epistemic partition
    /// and constructs a fresh `FrozenMemorySnapshot`, atomically updating `global_snapshot`.
    pub async fn ingest_and_build_snapshot(
        &self,
        pool: &Pool<Sqlite>,
        files: &HermesBootstrapFiles,
        now_epoch_sec: i64,
    ) -> Result<Arc<FrozenMemorySnapshot>, MemoryError> {
        let memory_entries = parse_markdown_entries(&files.memory_md);
        let user_entries = parse_markdown_entries(&files.user_md);
        let soul_trimmed = files.soul_md.trim();
        let soul_summary = if soul_trimmed.chars().count() > 400 {
            let truncated: String = soul_trimmed.chars().take(400).collect();
            format!("{truncated}...")
        } else {
            soul_trimmed.to_string()
        };

        // 1. Upsert MEMORY.md entries into STABLE partition
        for entry in &memory_entries {
            let digest = sha256_hex(entry);
            let record = EpistemicMemoryRecord {
                id: format!("hermes:memory:{}", &digest[..16]),
                session_id: "hermes_cold_boot".to_string(),
                category: "hermes_memory".to_string(),
                content: entry.clone(),
                partition: EpistemicPartition::Stable,
                salience: 1.0,
                access_count: 1,
                source_ref: Some("MEMORY.md".to_string()),
                created_at: now_epoch_sec,
                last_accessed_at: now_epoch_sec,
                updated_at: now_epoch_sec,
            };
            upsert_epistemic_memory(pool, &record).await?;
        }

        // 2. Upsert USER.md entries into STABLE partition
        for entry in &user_entries {
            let digest = sha256_hex(entry);
            let record = EpistemicMemoryRecord {
                id: format!("hermes:user:{}", &digest[..16]),
                session_id: "hermes_cold_boot".to_string(),
                category: "hermes_user".to_string(),
                content: entry.clone(),
                partition: EpistemicPartition::Stable,
                salience: 1.0,
                access_count: 1,
                source_ref: Some("USER.md".to_string()),
                created_at: now_epoch_sec,
                last_accessed_at: now_epoch_sec,
                updated_at: now_epoch_sec,
            };
            upsert_epistemic_memory(pool, &record).await?;
        }

        // 3. Upsert SOUL.md summary into STABLE partition if non-empty
        if !soul_summary.is_empty() {
            let digest = sha256_hex(&soul_summary);
            let record = EpistemicMemoryRecord {
                id: format!("hermes:soul:{}", &digest[..16]),
                session_id: "hermes_cold_boot".to_string(),
                category: "hermes_soul".to_string(),
                content: soul_summary.clone(),
                partition: EpistemicPartition::Stable,
                salience: 1.0,
                access_count: 1,
                source_ref: Some("SOUL.md".to_string()),
                created_at: now_epoch_sec,
                last_accessed_at: now_epoch_sec,
                updated_at: now_epoch_sec,
            };
            upsert_epistemic_memory(pool, &record).await?;
        }

        // 4. Query all STABLE truths and partition counts to build normalized fingerprints
        let stable_records = list_epistemic_memories(pool, Some(EpistemicPartition::Stable), 500).await?;
        let (stable_count, evolving_count) = count_epistemic_partitions(pool).await?;

        let mut fingerprints = HashSet::with_capacity(
            memory_entries.len() + user_entries.len() + stable_records.len() + 1,
        );
        for e in &memory_entries {
            let norm = normalize_for_dedup(e);
            if !norm.is_empty() {
                fingerprints.insert(norm);
            }
        }
        for e in &user_entries {
            let norm = normalize_for_dedup(e);
            if !norm.is_empty() {
                fingerprints.insert(norm);
            }
        }
        for rec in &stable_records {
            // Only add to dedup fingerprint if it originated from Hermes files so that
            // other STABLE architectural memories can still be recalled when relevant,
            // while avoiding repeating MEMORY.md / USER.md / SOUL.md entries!
            if matches!(
                rec.source_ref.as_deref(),
                Some("MEMORY.md") | Some("USER.md") | Some("SOUL.md")
            ) {
                let norm = normalize_for_dedup(&rec.content);
                if !norm.is_empty() {
                    fingerprints.insert(norm);
                }
            }
        }

        let memory_md_sha256 = sha256_hex(&files.memory_md);
        let user_md_sha256 = sha256_hex(&files.user_md);
        let soul_md_sha256 = sha256_hex(&files.soul_md);
        let composite_payload = format!(
            "{}:{}:{}:{}",
            memory_md_sha256, user_md_sha256, soul_md_sha256, stable_count
        );
        let composite_sha256 = sha256_hex(&composite_payload);
        let snapshot_id = format!("snap_{}", &composite_sha256[..16]);

        let snapshot = Arc::new(FrozenMemorySnapshot {
            snapshot_id: snapshot_id.clone(),
            composite_sha256,
            memory_md_sha256,
            user_md_sha256,
            soul_md_sha256,
            hermes_home: files.hermes_home.clone(),
            memory_entries,
            user_entries,
            soul_summary,
            stable_truths_count: stable_count,
            evolving_count,
            pinned_at: now_epoch_sec,
            normalized_fingerprints: fingerprints,
        });

        self.global_snapshot.store(snapshot.clone());
        info!(
            snapshot_id = %snapshot_id,
            stable_count,
            evolving_count,
            "FrozenMemorySnapshot built and stored atomically."
        );

        Ok(snapshot)
    }

    /// Pins a `FrozenMemorySnapshot` for a specific `session_id`.
    /// - If `force_refresh` is `false` and the session already has a pinned snapshot, returns it untouched.
    /// - If `force_refresh` is `true` (e.g. `/new`, `/reset`, or manual dashboard refresh) or if the global
    ///   snapshot has not been initialized yet (`pinned_at == 0`), reads `HermesBootstrapFiles` and rebuilds.
    pub async fn pin_session_snapshot(
        &self,
        session_id: &str,
        force_refresh: bool,
        pool: &Pool<Sqlite>,
        hermes_home: Option<&Path>,
        now_epoch_sec: i64,
    ) -> Result<Arc<FrozenMemorySnapshot>, MemoryError> {
        if !force_refresh {
            if let Some(existing) = self.session_snapshots.get(session_id) {
                debug!(session_id, snapshot_id = %existing.snapshot_id, "Returning existing pinned FrozenMemorySnapshot.");
                return Ok(existing.value().clone());
            }
        }

        let current = self.current_global();
        let snap = if force_refresh || current.pinned_at == 0 {
            let files = HermesBootstrapFiles::read_passive(hermes_home);
            self.ingest_and_build_snapshot(pool, &files, now_epoch_sec).await?
        } else {
            current
        };

        self.session_snapshots
            .insert(session_id.to_string(), snap.clone());
        Ok(snap)
    }

    /// Mirrors a write notification (`on_memory_write`) from Hermes Agent into the L3 vault.
    /// Does NOT mutate the already-pinned `FrozenMemorySnapshot` of active sessions, preserving
    /// 100% prefix cache hit rate until the next session boundary.
    pub async fn mirror_hermes_write(
        &self,
        pool: &Pool<Sqlite>,
        action: &str,
        target: &str,
        content: &str,
        session_id: &str,
        now_epoch_sec: i64,
    ) -> Result<Option<String>, MemoryError> {
        let trimmed = content.trim();
        if trimmed.is_empty() {
            return Ok(None);
        }

        let (prefix, category, source_file) = if target.eq_ignore_ascii_case("user") || target.eq_ignore_ascii_case("profile") {
            ("hermes:user", "hermes_user", "USER.md")
        } else {
            ("hermes:memory", "hermes_memory", "MEMORY.md")
        };

        let digest = sha256_hex(trimmed);
        let record_id = format!("{prefix}:{}", &digest[..16]);

        if action.eq_ignore_ascii_case("remove") || action.eq_ignore_ascii_case("delete") {
            crate::sqlite::delete_epistemic_memory(pool, &record_id).await?;
            return Ok(Some(record_id));
        }

        let record = EpistemicMemoryRecord {
            id: record_id.clone(),
            session_id: if session_id.is_empty() {
                "hermes_mirror".to_string()
            } else {
                session_id.to_string()
            },
            category: category.to_string(),
            content: trimmed.to_string(),
            partition: EpistemicPartition::Stable,
            salience: 1.0,
            access_count: 1,
            source_ref: Some(source_file.to_string()),
            created_at: now_epoch_sec,
            last_accessed_at: now_epoch_sec,
            updated_at: now_epoch_sec,
        };

        upsert_epistemic_memory(pool, &record).await?;
        Ok(Some(record_id))
    }
}

/// Filters RRF `UnifiedMatch` candidates against the session's `FrozenMemorySnapshot`
/// to eliminate duplicate facts already present in `MEMORY.md`/`USER.md`, and enforces
/// `top_k` (default 4) and `max_chars` (default 800 characters).
pub fn deduplicate_and_budget_matches(
    snapshot: &FrozenMemorySnapshot,
    matches: Vec<UnifiedMatch>,
    top_k: usize,
    max_chars: usize,
) -> Vec<UnifiedMatch> {
    let effective_top_k = if top_k == 0 { DEFAULT_PREFETCH_TOP_K } else { top_k };
    let effective_max_chars = if max_chars == 0 {
        DEFAULT_PREFETCH_MAX_CHARS
    } else {
        max_chars
    };

    let mut seen = snapshot.normalized_fingerprints.clone();
    let mut selected = Vec::with_capacity(effective_top_k);
    let mut used_chars = 0usize;

    for mut m in matches {
        if selected.len() >= effective_top_k || used_chars >= effective_max_chars {
            break;
        }

        let norm = normalize_for_dedup(&m.content);
        if norm.is_empty() || seen.contains(&norm) {
            continue;
        }

        let remaining = effective_max_chars.saturating_sub(used_chars);
        if remaining < 24 {
            break;
        }

        let content_chars = m.content.chars().count();
        if content_chars > remaining {
            let truncated: String = m.content.chars().take(remaining.saturating_sub(3)).collect();
            m.content = format!("{}...", truncated.trim_end());
        }

        used_chars += m.content.chars().count();
        seen.insert(norm);
        selected.push(m);
    }

    selected
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SoulsMemoryStore;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_passive_hermes_bootstrap_and_session_immutability() {
        let temp_dir = TempDir::new().expect("temp dir");
        let memories_dir = temp_dir.path().join("memories");
        fs::create_dir_all(&memories_dir).expect("create memories dir");

        let memory_file = memories_dir.join("MEMORY.md");
        let user_file = memories_dir.join("USER.md");
        let soul_file = temp_dir.path().join("SOUL.md");

        let orig_memory = "# Workspace Notes\n- Dev Drive is mounted on Z:\\souls_engine\n- SQLite operates in STRICT WAL mode\n";
        let orig_user = "# User Profile\n- Prefers Spec-Driven Development (SDD) and TDD\n";
        let orig_soul = "Bare-Metal Principal Engineer identity.";

        fs::write(&memory_file, orig_memory).expect("write MEMORY.md");
        fs::write(&user_file, orig_user).expect("write USER.md");
        fs::write(&soul_file, orig_soul).expect("write SOUL.md");

        let store = SoulsMemoryStore::new_in_memory().await.expect("in-memory store");

        // 1. Pin snapshot for session_alpha
        let snap1 = store
            .pin_session_snapshot("session_alpha", true, Some(temp_dir.path()), 1_700_000_000)
            .await
            .expect("pin snapshot");

        assert_eq!(snap1.memory_entries.len(), 2);
        assert_eq!(snap1.user_entries.len(), 1);
        assert!(snap1.stable_truths_count >= 4);

        // Verify original files on disk were NEVER modified (Clause 4 strict read-only check)
        assert_eq!(fs::read_to_string(&memory_file).unwrap(), orig_memory);
        assert_eq!(fs::read_to_string(&user_file).unwrap(), orig_user);

        // 2. Modify MEMORY.md on disk and mirror a new write mid-session
        fs::write(
            &memory_file,
            "# Workspace Notes\n- Dev Drive is mounted on Z:\\souls_engine\n- New mid-session note\n",
        )
        .expect("update MEMORY.md");

        store
            .mirror_hermes_write(
                "add",
                "memory",
                "New mid-session note",
                "session_alpha",
                1_700_000_050,
            )
            .await
            .expect("mirror write");

        // 3. Calling pin_session_snapshot with force_refresh = false on session_alpha MUST return the original frozen snapshot
        let snap_same = store
            .pin_session_snapshot("session_alpha", false, Some(temp_dir.path()), 1_700_000_100)
            .await
            .expect("re-read pinned snapshot");
        assert_eq!(snap_same.snapshot_id, snap1.snapshot_id);
        assert_eq!(snap_same.composite_sha256, snap1.composite_sha256);

        // 4. Calling with force_refresh = true (e.g., on /new session boundary) rotates to the updated snapshot
        let snap_refreshed = store
            .pin_session_snapshot("session_beta", true, Some(temp_dir.path()), 1_700_000_200)
            .await
            .expect("refresh snapshot for new session");
        assert_ne!(snap_refreshed.composite_sha256, snap1.composite_sha256);
    }

    #[tokio::test]
    async fn test_recall_deduplication_and_budget_enforcement() {
        let store = SoulsMemoryStore::new_in_memory().await.expect("in-memory store");
        let files = HermesBootstrapFiles {
            hermes_home: None,
            memory_md: "- Dev Drive is mounted on Z:\\souls_engine\n".to_string(),
            user_md: "- Prefers Spec-Driven Development (SDD)\n".to_string(),
            soul_md: String::new(),
        };

        let snap = store
            .snapshots()
            .ingest_and_build_snapshot(&store.pool(), &files, 1_700_000_000)
            .await
            .expect("ingest");
        store
            .snapshots()
            .pin_session_snapshot("sess_1", false, &store.pool(), None, 1_700_000_000)
            .await
            .expect("pin");

        // Also insert an EVOLVING architectural insight not in MEMORY.md
        let evolving_rec = EpistemicMemoryRecord {
            id: "evo_1".to_string(),
            session_id: "sess_1".to_string(),
            category: "architecture".to_string(),
            content: "Dev Drive ReFS uses LanceDB Arrow zero-copy vectors with 0 MB VRAM".to_string(),
            partition: EpistemicPartition::Evolving,
            salience: 0.9,
            access_count: 1,
            source_ref: Some("turn_sync".to_string()),
            created_at: 1_700_000_010,
            last_accessed_at: 1_700_000_010,
            updated_at: 1_700_000_010,
        };
        store.upsert_epistemic_memory(&evolving_rec).await.expect("upsert evo");

        // Query for "Dev Drive" — both the MEMORY.md entry and evo_1 match in FTS5,
        // but recall_for_session MUST deduplicate the MEMORY.md entry!
        let recalled = store
            .recall_for_session("sess_1", "Dev Drive", None, 4, 800)
            .await
            .expect("recall for session");

        assert_eq!(recalled.len(), 1);
        assert_eq!(recalled[0].id, "evo_1");
        assert!(recalled[0].content.len() <= 800);
        assert!(!snap.normalized_fingerprints.is_empty());
    }
}

