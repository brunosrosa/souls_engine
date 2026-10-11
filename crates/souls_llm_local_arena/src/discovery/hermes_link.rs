//! Hermes Agent model directory zero-byte hardlink synchronizer.
//!
//! Synchronizes models in the canonical Souls Engine store (`C:\Users\<user>\llm_local_models`)
//! into the flat Hermes Agent models folder (`%LOCALAPPDATA%\hermes\models`) using zero-copy NTFS/ReFS
//! hardlinks (or symlinks as fallback). Automatically purges stale links when models are pruned.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::{info, warn};
use crate::domain::ModelDescriptor;
use crate::error::ArenaError;

/// Report of hardlinks synchronization operation.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HermesSyncReport {
    pub links_created: usize,
    pub links_removed: usize,
    pub links_verified: usize,
}

/// Resolves the default Hermes Agent models directory (`%LOCALAPPDATA%\hermes\models`).
pub fn default_hermes_models_dir() -> PathBuf {
    if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
        PathBuf::from(local_app_data).join("hermes").join("models")
    } else if let Ok(user_profile) = std::env::var("USERPROFILE") {
        PathBuf::from(user_profile)
            .join("AppData")
            .join("Local")
            .join("hermes")
            .join("models")
    } else {
        PathBuf::from(r"C:\Users\rosas\AppData\Local\hermes\models")
    }
}

/// Synchronizes active models descriptors into the target Hermes models directory.
/// Purges any obsolete/pruned `.gguf` files and creates zero-byte hardlinks for active models.
pub fn sync_hermes_model_links(
    active_descriptors: &[ModelDescriptor],
    custom_hermes_dir: Option<&Path>,
) -> Result<HermesSyncReport, ArenaError> {
    let dest_dir = custom_hermes_dir
        .map(PathBuf::from)
        .unwrap_or_else(default_hermes_models_dir);

    if !dest_dir.exists() {
        std::fs::create_dir_all(&dest_dir)?;
    }

    // 1. Build map of expected active files (filename -> source path)
    let mut expected_files: HashMap<String, PathBuf> = HashMap::new();
    for desc in active_descriptors {
        if desc.file_path.exists() {
            if let Some(file_name) = desc.file_path.file_name() {
                let name_str = file_name.to_string_lossy().to_string();
                if name_str.ends_with(".gguf") {
                    expected_files.insert(name_str, desc.file_path.clone());
                }
            }
        }
        if let Some(ref mmproj) = desc.mmproj_path {
            if mmproj.exists() {
                if let Some(mmproj_name) = mmproj.file_name() {
                    let name_str = mmproj_name.to_string_lossy().to_string();
                    expected_files.insert(name_str, mmproj.clone());
                }
            }
        }
    }

    let mut report = HermesSyncReport::default();

    // 2. Scan Hermes directory and remove pruned/obsolete models
    if let Ok(entries) = std::fs::read_dir(&dest_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(file_name) = path.file_name() {
                    let name_str = file_name.to_string_lossy().to_string();
                    if name_str.ends_with(".gguf") {
                        if !expected_files.contains_key(&name_str) {
                            // File not in active set -> Purge!
                            if let Err(e) = std::fs::remove_file(&path) {
                                warn!("Failed to purge obsolete Hermes link {}: {}", path.display(), e);
                            } else {
                                info!("Purged obsolete Hermes model link: {}", name_str);
                                report.links_removed += 1;
                            }
                        }
                    }
                }
            }
        }
    }

    // 3. Create or verify hard links for expected active models
    for (file_name, source_path) in expected_files {
        let dest_file = dest_dir.join(&file_name);
        if dest_file.exists() {
            // Already present, verify it points to an accessible file
            report.links_verified += 1;
            continue;
        }

        // Try creating NTFS hard link first (0 bytes overhead)
        match std::fs::hard_link(&source_path, &dest_file) {
            Ok(()) => {
                info!("Created zero-byte HardLink for Hermes: {} -> {}", file_name, source_path.display());
                report.links_created += 1;
            }
            Err(err) => {
                // Fallback to symlink if cross-volume or permission error
                #[cfg(windows)]
                {
                    match std::os::windows::fs::symlink_file(&source_path, &dest_file) {
                        Ok(()) => {
                            info!("Created SymLink fallback for Hermes: {} -> {}", file_name, source_path.display());
                            report.links_created += 1;
                        }
                        Err(sym_err) => {
                            warn!(
                                "Failed to create link for {} (HardLink: {}, Symlink: {})",
                                file_name, err, sym_err
                            );
                        }
                    }
                }
                #[cfg(not(windows))]
                {
                    match std::os::unix::fs::symlink(&source_path, &dest_file) {
                        Ok(()) => {
                            report.links_created += 1;
                        }
                        Err(sym_err) => {
                            warn!("Failed to create link for {}: {}", file_name, sym_err);
                        }
                    }
                }
            }
        }
    }

    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use crate::domain::ModelTier;

    #[test]
    fn test_sync_hermes_model_links_creation_and_pruning() {
        let src_dir = tempdir().unwrap();
        let hermes_dir = tempdir().unwrap();

        let model_a = src_dir.path().join("model_a.gguf");
        let model_b = src_dir.path().join("model_b.gguf");
        std::fs::write(&model_a, b"model a dummy content").unwrap();
        std::fs::write(&model_b, b"model b dummy content").unwrap();

        let descriptors = vec![
            ModelDescriptor {
                file_path: model_a.clone(),
                name: "Model A".to_string(),
                model_id: "model-a".to_string(),
                tier: ModelTier::Tier1Master,
                parameters: "1B".to_string(),
                quant: "Q4".to_string(),
                size_mb: 1.0,
                ctx_max: 2048,
                has_mmproj: false,
                mmproj_path: None,
            },
            ModelDescriptor {
                file_path: model_b.clone(),
                name: "Model B".to_string(),
                model_id: "model-b".to_string(),
                tier: ModelTier::Tier1Master,
                parameters: "1B".to_string(),
                quant: "Q4".to_string(),
                size_mb: 1.0,
                ctx_max: 2048,
                has_mmproj: false,
                mmproj_path: None,
            },
        ];

        // 1. First sync -> creates 2 links
        let report1 = sync_hermes_model_links(&descriptors, Some(hermes_dir.path())).unwrap();
        assert_eq!(report1.links_created, 2);
        assert_eq!(report1.links_removed, 0);
        assert!(hermes_dir.path().join("model_a.gguf").exists());
        assert!(hermes_dir.path().join("model_b.gguf").exists());

        // 2. Second sync -> verifies 2 links
        let report2 = sync_hermes_model_links(&descriptors, Some(hermes_dir.path())).unwrap();
        assert_eq!(report2.links_created, 0);
        assert_eq!(report2.links_verified, 2);

        // 3. Prune Model B -> sync with only Model A
        let report3 = sync_hermes_model_links(&descriptors[0..1], Some(hermes_dir.path())).unwrap();
        assert_eq!(report3.links_removed, 1);
        assert!(hermes_dir.path().join("model_a.gguf").exists());
        assert!(!hermes_dir.path().join("model_b.gguf").exists());
    }
}
