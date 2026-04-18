//! Undo logic with file history

use anyhow::{Context, Result};
use tracing::{info, warn};
// use tracing::{error, info, warn};

use crate::config::Config;
use crate::file_ops::{Session, move_file};

pub fn run_undo(config: &Config) -> Result<()> {
    // 1. Load history log
    let session = Session::load(&config.source_dir).context("Failed to load history log")?;

    if session.moves.is_empty() {
        info!("No recent moves found to undo.");
        return Ok(());
    }

    info!("⏪ Undoing {} file movements...", session.moves.len());

    // 2. Loop in reverse (Important: Use .rev() to move the latest file back first)
    for record in session.moves.iter().rev() {
        if record.destination.exists() {
            info!("Restoring: {:?} -> {:?}", record.destination, record.source);

            // Move back
            // if let Err(e) = std::fs::rename(&record.destination, &record.source) {
            //     error!("Failed to restore {:?}: {}", record.destination, e);
            // }

            // ตัวอย่างการใช้ใน undo.rs
            if let Some(original_dir) = record.source.parent() {
                move_file(&record.destination, original_dir, false)?;
            }
        } else {
            warn!(
                "Could not find file at {:?}, skipping...",
                record.destination
            );
        }
    }

    // 3. After Undo is completed, delete the history file to prevent duplication
    let history_path = config.source_dir.join(".panos_history.json");
    let _ = std::fs::remove_file(history_path);

    info!("✅ Undo operation completed.");
    Ok(())
}
