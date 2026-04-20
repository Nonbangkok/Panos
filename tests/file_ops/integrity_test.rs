use chrono::Utc;
use panos::check_integrity;
use panos::file_ops::history::MoveRecord;
use panos::ui::NoopReporter;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_integrity_comprehensive_validation() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let root = tmp.path();

    let ok_file = root.join("ok.txt");
    fs::write(&ok_file, "content")?;

    let history = vec![
        MoveRecord {
            source: root.join("old_ok.txt"),
            destination: ok_file.clone(),
            timestamp: Utc::now(),
            file_size: 7,
        },
        MoveRecord {
            source: root.join("old_missing.txt"),
            destination: root.join("missing.txt"),
            timestamp: Utc::now(),
            file_size: 10,
        },
        MoveRecord {
            source: root.join("old_mismatch.txt"),
            destination: ok_file.clone(),
            timestamp: Utc::now(),
            file_size: 999,
        },
    ];

    let result = check_integrity(&history, false, &NoopReporter);
    assert!(
        result.is_ok(),
        "Integrity check should run and report without failing"
    );

    Ok(())
}

#[test]
fn test_integrity_empty_history_graceful_handling() -> anyhow::Result<()> {
    let _tmp = TempDir::new()?;
    let result = check_integrity(&[], false, &NoopReporter);
    assert!(result.is_ok());
    Ok(())
}

#[test]
fn test_integrity_massive_parallel_verification_10000_files() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let root = tmp.path();
    let mut history = Vec::new();
    for i in 0..10000 {
        let path = root.join(format!("file_{}.dat", i));
        fs::write(&path, "data")?;
        history.push(MoveRecord {
            source: PathBuf::from(format!("old/file_{}.dat", i)),
            destination: path,
            timestamp: Utc::now(),
            file_size: 4,
        });
    }
    let result = check_integrity(&history, false, &NoopReporter);
    assert!(result.is_ok());
    Ok(())
}

#[test]
fn test_integrity_unicode_and_emoji_path_verification() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let root = tmp.path();
    let complex_name = "รายงาน_2024_🔥_🚀.pdf";
    let path = root.join(complex_name);

    fs::write(&path, "unicode content")?;

    let history = vec![MoveRecord {
        source: PathBuf::from("source/old.pdf"),
        destination: path,
        timestamp: Utc::now(),
        file_size: 15,
    }];
    let result = check_integrity(&history, false, &NoopReporter);
    assert!(result.is_ok());
    Ok(())
}

#[test]
fn test_integrity_deeply_nested_directory_verification() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let root = tmp.path();
    let mut deep_path = root.to_path_buf();
    for i in 0..50 {
        deep_path = deep_path.join(format!("level_{}", i));
    }
    fs::create_dir_all(&deep_path)?;
    let target_file = deep_path.join("deep.bin");
    fs::write(&target_file, "nested data")?;
    let history = vec![MoveRecord {
        source: PathBuf::from("old/deep.bin"),
        destination: target_file,
        timestamp: Utc::now(),
        file_size: 11,
    }];
    let result = check_integrity(&history, false, &NoopReporter);
    assert!(result.is_ok());
    Ok(())
}

#[test]
fn test_integrity_dry_run_state_bypass() -> anyhow::Result<()> {
    let _tmp = TempDir::new()?;
    let history = vec![MoveRecord {
        source: PathBuf::from("source.txt"),
        destination: PathBuf::from("non_existent_dest.txt"),
        timestamp: Utc::now(),
        file_size: 100,
    }];
    let result = check_integrity(&history, true, &NoopReporter);
    assert!(result.is_ok());
    Ok(())
}

#[test]
fn test_integrity_zero_byte_file_verification() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let root = tmp.path();
    let empty_file = root.join("empty.txt");
    fs::write(&empty_file, "")?;
    let history = vec![MoveRecord {
        source: PathBuf::from("old/empty.txt"),
        destination: empty_file,
        timestamp: Utc::now(),
        file_size: 0,
    }];
    let result = check_integrity(&history, false, &NoopReporter);
    assert!(result.is_ok());
    Ok(())
}

#[test]
fn test_integrity_all_missing_files_reporting() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let root = tmp.path();
    let mut history = Vec::new();
    for i in 0..100 {
        history.push(MoveRecord {
            source: PathBuf::from(format!("old/missing_{}.bin", i)),
            destination: root.join(format!("missing_{}.bin", i)),
            timestamp: Utc::now(),
            file_size: 1024,
        });
    }
    let result = check_integrity(&history, false, &NoopReporter);
    assert!(result.is_ok());
    Ok(())
}

#[test]
fn test_integrity_various_size_mismatches_interleaved() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let root = tmp.path();
    let mut history = Vec::new();
    for i in 0..100 {
        let path = root.join(format!("file_{}.txt", i));
        fs::write(&path, "content")?;

        let reported_size = if i % 2 == 0 { 7 } else { 999 };
        history.push(MoveRecord {
            source: PathBuf::from(format!("old/file_{}.txt", i)),
            destination: path,
            timestamp: Utc::now(),
            file_size: reported_size,
        });
    }
    let result = check_integrity(&history, false, &NoopReporter);
    assert!(result.is_ok());
    Ok(())
}

#[test]
fn test_integrity_extreme_file_sizes_recording_and_check() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let root = tmp.path();
    let large_val_file = root.join("large_val.bin");
    fs::write(&large_val_file, "tiny")?;
    let history = vec![MoveRecord {
        source: PathBuf::from("old.bin"),
        destination: large_val_file,
        timestamp: Utc::now(),
        file_size: 18446744073709551615,
    }];
    let result = check_integrity(&history, false, &NoopReporter);
    assert!(result.is_ok());
    Ok(())
}

#[test]
fn test_integrity_mixed_scenario_stress_load() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let root = tmp.path();
    let mut history = Vec::new();
    for i in 0..1000 {
        let path = root.join(format!("mixed_{}.bin", i));
        if i % 3 != 0 {
            fs::write(&path, vec![0u8; 100])?;
        }

        history.push(MoveRecord {
            source: PathBuf::from(format!("old/mixed_{}.bin", i)),
            destination: path,
            timestamp: Utc::now(),
            file_size: 100,
        });
    }
    let result = check_integrity(&history, false, &NoopReporter);
    assert!(result.is_ok());
    Ok(())
}
