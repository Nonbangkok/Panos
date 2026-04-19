//! Empty directory removing and handling operations

use crate::file_ops::MoveRecord;
use anyhow::Result;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use tracing::info;
use walkdir::WalkDir;

pub fn remove_empty_dirs(
    root: &std::path::Path,
    dry_run: bool,
    predicted_move: &[MoveRecord],
) -> Result<()> {
    // Map source paths to a HashSet for O(1) lookups
    let move_sources: HashSet<_> = predicted_move.iter().map(|m| m.source.clone()).collect();

    // Track directories that WOULD be empty in dry_run mode
    let mut would_be_empty_dirs = HashSet::new();

    for entry in WalkDir::new(root)
        .contents_first(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if !entry.file_type().is_dir() || entry.path() == root {
            continue;
        }

        let path = entry.path();

        let is_empty: bool = if !dry_run {
            // Actual check for real execution
            is_actually_empty(path)
        } else {
            // Predictive check for dry run
            predict_directory_empty(path, &move_sources, &would_be_empty_dirs)
        };

        if is_empty {
            if dry_run {
                info!("[DRY RUN] Would remove empty directory: {:?}", path);
                would_be_empty_dirs.insert(path.to_path_buf());
            } else {
                info!("Removing empty directory: {:?}", path);
                std::fs::remove_dir(path)?;
            }
        }
    }

    Ok(())
}

fn is_actually_empty(path: &Path) -> bool {
    match std::fs::read_dir(path) {
        Ok(mut entries) => entries.next().is_none(),
        Err(e) => {
            tracing::error!("Could not read directory {:?}: {}", path, e);
            false
        }
    }
}

fn predict_directory_empty(
    path: &Path,
    move_sources: &HashSet<PathBuf>,
    would_be_empty_dirs: &HashSet<PathBuf>,
) -> bool {
    match std::fs::read_dir(path) {
        Ok(entries) => {
            let mut all_will_be_gone = true;
            for entry in entries.filter_map(|e| e.ok()) {
                let p = entry.path();

                if p.is_dir() {
                    if !would_be_empty_dirs.contains(&p) {
                        all_will_be_gone = false;
                        break;
                    }
                } else if p.is_file() {
                    if !move_sources.contains(&p) {
                        all_will_be_gone = false;
                        break;
                    }
                } else {
                    // Other file types (symlinks, etc.) are not moved, so directory won't be empty
                    all_will_be_gone = false;
                    break;
                }
            }
            all_will_be_gone
        }
        Err(e) => {
            tracing::error!("Could not read directory {:?}: {}", path, e);
            false
        }
    }
}
