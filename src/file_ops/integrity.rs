//! File integrity check operations

use anyhow::Result;
use rayon::prelude::*;
use std::fs;
use tracing::info;

use crate::file_ops::history::MoveRecord;
use crate::ui::ProgressReporter;

pub struct IntegrityReport {
    pub total_checked: usize,
    pub successful: usize,
    pub failures: Vec<(String, String)>, // (Path, Error Message)
}

pub fn check_integrity(
    history: &[MoveRecord],
    dry_run: bool,
    reporter: &dyn ProgressReporter,
) -> Result<()> {
    if !history.is_empty() {
        let report = verify_moves(&history, dry_run, &*reporter)?;

        info!("\n🛡 Integrity Check:");
        info!("  Checked: {} files", report.total_checked);
        info!("  Successful: {}", report.successful);

        if !report.failures.is_empty() {
            info!("  ⚠️ FAILURES detected:");
            for (path, err) in report.failures {
                info!("    - {}: {}", path, err);
            }
        } else {
            info!("  ✅ All files verified successfully!");
        }
    }

    Ok(())
}

fn verify_moves(
    history: &[MoveRecord],
    dry_run: bool,
    reporter: &dyn ProgressReporter,
) -> Result<IntegrityReport> {
    if history.is_empty() {
        return Ok(IntegrityReport {
            total_checked: 0,
            successful: 0,
            failures: vec![],
        });
    }

    if dry_run {
        info!(
            "[DRY RUN] Would check integrity of {} files...",
            history.len()
        );
        return Ok(IntegrityReport {
            total_checked: history.len(),
            successful: history.len(),
            failures: vec![],
        });
    }

    reporter.start(
        Some(history.len() as u64),
        "Checking integrity...".to_string(),
    );

    let results: Vec<Option<(String, String)>> = history
        .par_iter()
        .map(|record| {
            let result = if !record.destination.exists() {
                Some((
                    record.destination.to_string_lossy().to_string(),
                    "Missing at destination".to_string(),
                ))
            } else {
                match fs::metadata(&record.destination) {
                    Ok(meta) => {
                        if meta.len() != record.file_size {
                            Some((
                                record.destination.to_string_lossy().to_string(),
                                format!(
                                    "Size mismatch: expected {}, found {}",
                                    record.file_size,
                                    meta.len()
                                ),
                            ))
                        } else {
                            None
                        }
                    }
                    Err(e) => Some((
                        record.destination.to_string_lossy().to_string(),
                        format!("Metadata error: {}", e),
                    )),
                }
            };
            reporter.update(1, "".to_string());
            result
        })
        .collect();

    let failures: Vec<(String, String)> = results.into_iter().flatten().collect();
    let successful = history.len() - failures.len();

    reporter.finish("Integrity check complete!".to_string());

    Ok(IntegrityReport {
        total_checked: history.len(),
        successful,
        failures,
    })
}
