//! File moving and handling operations

use anyhow::Result;
use chrono::Utc;
use tracing::info;

use crate::file_ops::history::MoveRecord;

/// Move a file to destination with conflict resolution
pub fn move_file(
    src_path: &std::path::Path,
    dest_path: &std::path::Path,
    dry_run: bool,
) -> Result<Option<MoveRecord>> {
    // Check if source and destination are the same
    if src_path == dest_path {
        return Ok(None);
    }

    let mut final_dest = dest_path.to_path_buf();

    // Create parent directory if it doesn't exist
    if !dry_run {
        if let Some(parent) = final_dest.parent() {
            std::fs::create_dir_all(parent)?;
        }
    }

    // Handle conflict
    if final_dest.exists() && src_path != final_dest {
        let stem: &str = src_path
            .file_stem()
            .and_then(|s: &std::ffi::OsStr| s.to_str())
            .unwrap_or("file");
        let extension: &str = src_path
            .extension()
            .and_then(|e: &std::ffi::OsStr| e.to_str())
            .unwrap_or("");

        let mut count: i32 = 1;
        while final_dest.exists() {
            let new_name: String = if extension.is_empty() {
                format!("{}_{}", stem, count)
            } else {
                format!("{}_{}.{}", stem, count, extension)
            };
            final_dest = final_dest.with_file_name(new_name);
            count += 1;
        }
    }

    if dry_run {
        info!("[DRY RUN] Would move {:?} to {:?}", src_path, final_dest);
    } else {
        info!("Moving {:?} to {:?}", src_path, final_dest);
        if let Err(e) = std::fs::rename(src_path, &final_dest) {
            if e.kind() == std::io::ErrorKind::CrossesDevices {
                std::fs::copy(src_path, &final_dest)?;
                std::fs::remove_file(src_path)?;
            } else {
                return Err(e.into());
            }
        }
    }

    Ok(Some(MoveRecord {
        source: src_path.to_path_buf(),
        destination: final_dest,
        timestamp: Utc::now(),
    }))
}
