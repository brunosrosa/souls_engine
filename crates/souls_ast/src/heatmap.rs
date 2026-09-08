//! Commit Heatmap via gitoxide (gix).
//!
//! Stateless and zero database I/O. Computes Frecency using Langevin Exponential Decay:
//! F(a) = \sum Changes * e^{-\lambda \Delta t}

use std::collections::HashMap;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::error::AstError;

/// Canonical decay constant if half_life is not provided (~1h55min half-life).
pub const DEFAULT_LAMBDA: f64 = 0.0001;

/// Maximum frecency score cap to prevent overflow in massive repositories.
pub const MAX_SCORE: f64 = 1000.0;

/// Individual file thermal entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FileHeatEntry {
    /// Normalized relative path of the file.
    pub file_path: String,
    /// Computed frecency thermal score.
    pub frecency_score: f64,
    /// Total number of commits/changes modifying this file within the window.
    pub commit_count: u32,
    /// Unix timestamp (epoch seconds) of the most recent modification.
    pub last_commit_epoch: i64,
}

/// Computes lambda from half-life in days: \lambda = ln(2) / (half_life_days * 86400)
#[inline]
pub fn lambda_from_half_life_days(half_life_days: f64) -> f64 {
    if half_life_days <= 0.0 {
        DEFAULT_LAMBDA
    } else {
        std::f64::consts::LN_2 / (half_life_days * 86400.0)
    }
}

/// Calculates Langevin exponential decay score for a sequence of change epochs:
/// F = \sum e^{-\lambda \Delta t}
#[inline]
pub fn compute_langevin_frecency(changes: &[(u32, i64)], now: i64, lambda: f64) -> f64 {
    let mut sum: f64 = 0.0;
    for &(count, epoch) in changes {
        let dt = (now - epoch).max(0) as f64;
        sum += (count as f64) * (-lambda * dt).exp();
    }
    sum.min(MAX_SCORE)
}

/// Inspects local Git repository using `gix` and calculates Frecency heatmap.
pub fn calculate_repo_frecency(
    repo_path: &Path,
    time_window_days: u32,
    half_life_days: f64,
) -> Result<Vec<FileHeatEntry>, AstError> {
    let repo = gix::open(repo_path).map_err(|e| AstError::GitError(format!("open repo failed: {e}")))?;

    let head_commit = match repo.head_commit() {
        Ok(commit) => commit,
        Err(_) => return Ok(Vec::new()),
    };

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    let lambda = lambda_from_half_life_days(half_life_days);
    let window_secs = if time_window_days > 0 {
        Some((time_window_days as i64) * 86400)
    } else {
        None
    };

    let mut file_changes: HashMap<String, Vec<(u32, i64)>> = HashMap::new();

    let revwalk = repo
        .rev_walk([head_commit.id()])
        .all()
        .map_err(|e| AstError::GitError(format!("rev_walk failed: {e}")))?;

    for commit_info in revwalk {
        let commit_info = commit_info.map_err(|e| AstError::GitError(format!("rev_walk item error: {e}")))?;
        let commit_obj = commit_info
            .object()
            .map_err(|e| AstError::GitError(format!("read commit object failed: {e}")))?;
        let time = commit_obj
            .time()
            .map_err(|e| AstError::GitError(format!("read commit time failed: {e}")))?;
        let commit_epoch = time.seconds;

        if let Some(max_age) = window_secs {
            if now - commit_epoch > max_age {
                continue;
            }
        }

        let current_tree = commit_obj
            .tree()
            .map_err(|e| AstError::GitError(format!("read tree failed: {e}")))?;

        let parent_tree = if let Some(parent_id) = commit_obj.parent_ids().next() {
            if let Ok(parent_obj) = repo.find_object(parent_id) {
                if let Ok(parent_commit) = parent_obj.try_into_commit() {
                    parent_commit.tree().ok()
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        let mut changed_files = Vec::new();
        if let Some(ref parent) = parent_tree {
            if let Ok(mut changes) = current_tree.changes() {
                changes.track_path();
                let mut changes_collector = |change: gix::object::tree::diff::Change| {
                    let path = change.location.to_string();
                    changed_files.push(path);
                    Ok::<_, std::convert::Infallible>(gix::object::tree::diff::Action::Continue)
                };
                let _ = changes.for_each_to_obtain_tree(parent, &mut changes_collector);
            }
        } else {
            // Initial commit: all blobs in tree
            for entry in current_tree.iter() {
                if let Ok(entry) = entry {
                    if entry.mode().is_blob() {
                        let path = entry.filename().to_string();
                        changed_files.push(path);
                    }
                }
            }
        }

        for path in changed_files {
            let normalized = path.replace('\\', "/");
            file_changes
                .entry(normalized)
                .or_default()
                .push((1, commit_epoch));
        }
    }

    let mut results: Vec<FileHeatEntry> = file_changes
        .into_iter()
        .map(|(path, changes)| {
            let score = compute_langevin_frecency(&changes, now, lambda);
            let count = changes.len() as u32;
            let last_epoch = changes.iter().map(|(_, t)| *t).max().unwrap_or(now);
            FileHeatEntry {
                file_path: path,
                frecency_score: score,
                commit_count: count,
                last_commit_epoch: last_epoch,
            }
        })
        .collect();

    results.sort_by(|a, b| {
        b.frecency_score
            .partial_cmp(&a.frecency_score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.file_path.cmp(&b.file_path))
    });

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lambda_derivation_from_half_life() {
        let half_life_days = 7.0;
        let lambda = lambda_from_half_life_days(half_life_days);
        assert!(lambda > 0.0);
        // Half life check: at dt = 7 days (7 * 86400s), score should be 0.5
        let dt = 7.0 * 86400.0;
        let decayed = (-lambda * dt).exp();
        assert!((decayed - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_compute_langevin_frecency_decay() {
        let now = 1_000_000i64;
        let lambda = 0.0001;

        // Recent modification (dt = 0)
        let recent = vec![(1, now)];
        let score_recent = compute_langevin_frecency(&recent, now, lambda);
        assert!((score_recent - 1.0).abs() < 1e-6);

        // Old modification (dt = 6931 seconds -> approx half life)
        let old = vec![(1, now - 6931)];
        let score_old = compute_langevin_frecency(&old, now, lambda);
        assert!((score_old - 0.5).abs() < 0.01);

        // Multiple changes accumulation
        let multiple = vec![(1, now), (1, now - 100), (2, now - 200)];
        let score_multiple = compute_langevin_frecency(&multiple, now, lambda);
        assert!(score_multiple > 3.0);
    }

    #[test]
    fn test_synthetic_gix_repo_frecency_calculation() {
        let temp_dir = tempfile::tempdir().expect("create temp dir");
        let repo = gix::init(temp_dir.path()).expect("init git repo");

        // Write blob for file A
        let blob_a = repo.write_blob(b"fn compute_a() {}").expect("write blob a");
        let blob_b = repo.write_blob(b"fn compute_b() {}").expect("write blob b");

        let mut tree1 = gix::objs::Tree::empty();
        tree1.entries.push(gix::objs::tree::Entry {
            mode: gix::objs::tree::EntryKind::Blob.into(),
            filename: "file_a.rs".into(),
            oid: blob_a.detach(),
        });
        tree1.entries.push(gix::objs::tree::Entry {
            mode: gix::objs::tree::EntryKind::Blob.into(),
            filename: "file_b.rs".into(),
            oid: blob_b.detach(),
        });
        let tree1_id = repo.write_object(&tree1).expect("write tree1");

        let sig = gix::actor::SignatureRef {
            name: "Souls BareMetal".into(),
            email: "engineer@souls.engine".into(),
            time: gix::date::Time::now_local_or_utc(),
        };

        let no_parents: [gix::ObjectId; 0] = [];
        let commit1 = repo
            .commit_as(sig, sig, "HEAD", "Commit 1: add file_a and file_b", tree1_id, no_parents)
            .expect("create commit 1");

        // Commit 2: modify file_a
        let blob_a2 = repo.write_blob(b"fn compute_a_v2() {}").expect("write blob a2");
        let mut tree2 = gix::objs::Tree::empty();
        tree2.entries.push(gix::objs::tree::Entry {
            mode: gix::objs::tree::EntryKind::Blob.into(),
            filename: "file_a.rs".into(),
            oid: blob_a2.detach(),
        });
        tree2.entries.push(gix::objs::tree::Entry {
            mode: gix::objs::tree::EntryKind::Blob.into(),
            filename: "file_b.rs".into(),
            oid: blob_b.detach(),
        });
        let tree2_id = repo.write_object(&tree2).expect("write tree2");

        let _commit2 = repo
            .commit_as(sig, sig, "HEAD", "Commit 2: modify file_a", tree2_id, [commit1.detach()])
            .expect("create commit 2");

        let heatmap = calculate_repo_frecency(temp_dir.path(), 30, 7.0)
            .expect("calculate_repo_frecency should succeed");

        assert!(!heatmap.is_empty(), "heatmap must contain entries");
        assert_eq!(heatmap.len(), 2, "expected 2 tracked files");

        // file_a.rs was changed in both commits, file_b.rs only in the first
        let entry_a = heatmap.iter().find(|e| e.file_path == "file_a.rs").expect("file_a in heatmap");
        let entry_b = heatmap.iter().find(|e| e.file_path == "file_b.rs").expect("file_b in heatmap");

        assert_eq!(entry_a.commit_count, 2);
        assert_eq!(entry_b.commit_count, 1);
        assert!(entry_a.frecency_score > entry_b.frecency_score, "file_a must have higher frecency score");
    }
}

