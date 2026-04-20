use chrono::Utc;
use panos::file_ops::history::{MoveRecord, Session};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_history_empty_session_save_load() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let root = tmp.path();
    let history_file = ".history.json";

    let session = Session::default();
    session.save(root, history_file)?;

    let loaded = Session::load(root, history_file)?;
    assert!(
        loaded.moves.is_empty(),
        "Loaded session should be empty when saved empty"
    );
    Ok(())
}

#[test]
fn test_history_massive_move_records() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let root = tmp.path();
    let history_file = "massive.json";

    let mut session = Session::default();
    for i in 0..5000 {
        session.moves.push(MoveRecord {
            source: PathBuf::from(format!("source/file_{}.txt", i)),
            destination: PathBuf::from(format!("dest/file_{}.txt", i)),
            timestamp: Utc::now(),
            file_size: 0,
        });
    }

    session.save(root, history_file)?;
    let loaded = Session::load(root, history_file)?;

    assert_eq!(
        loaded.moves.len(),
        5000,
        "Should accurately store and retrieve 5000 records"
    );
    assert_eq!(
        loaded.moves[4999].source,
        PathBuf::from("source/file_4999.txt"),
        "Last record integrity must be maintained"
    );
    Ok(())
}

#[test]
fn test_history_unicode_paths_preservation() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let root = tmp.path();
    let history_file = "unicode.json";

    let thai_path = "โฟลเดอร์/งาน_2024 🔥.pdf";
    let emoji_path = "🚀/star.jpg";

    let mut session = Session::default();
    session.moves.push(MoveRecord {
        source: PathBuf::from(thai_path),
        destination: PathBuf::from(emoji_path),
        timestamp: Utc::now(),
        file_size: 0,
    });

    session.save(root, history_file)?;
    let loaded = Session::load(root, history_file)?;

    assert_eq!(
        loaded.moves[0].source.to_str().unwrap(),
        thai_path,
        "Unicode characters in source path must be preserved"
    );
    assert_eq!(
        loaded.moves[0].destination.to_str().unwrap(),
        emoji_path,
        "Emoji characters in destination path must be preserved"
    );
    Ok(())
}

#[test]
fn test_history_special_characters_in_filename() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let root = tmp.path();
    let history_file = "special 'quotes' and \"spaces\".json";

    let mut session = Session::default();
    session.moves.push(MoveRecord {
        source: PathBuf::from("dir/file with spaces.txt"),
        destination: PathBuf::from("dir/file.with.many.dots.tar.gz"),
        timestamp: Utc::now(),
        file_size: 0,
    });

    session.save(root, history_file)?;
    let loaded = Session::load(root, history_file)?;

    assert_eq!(
        loaded.moves.len(),
        1,
        "Should handle special characters in history filename itself"
    );
    assert!(
        loaded.moves[0]
            .destination
            .to_str()
            .unwrap()
            .contains("tar.gz"),
        "Dots in filename should not affect serialization"
    );
    Ok(())
}

#[test]
fn test_history_invalid_json_format_fails() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let root = tmp.path();
    let history_file = "broken.json";

    fs::write(
        root.join(history_file),
        "{ \"moves\": [ { \"invalid\": \"json\" ] }",
    )?;

    let result = Session::load(root, history_file);
    assert!(
        result.is_err(),
        "Loading malformed JSON should return an error"
    );
    Ok(())
}

#[test]
fn test_history_overwrite_existing_history() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let root = tmp.path();
    let history_file = "overwrite.json";

    let mut session1 = Session::default();
    session1.moves.push(MoveRecord {
        source: PathBuf::from("old"),
        destination: PathBuf::from("old_dst"),
        timestamp: Utc::now(),
        file_size: 0,
    });
    session1.save(root, history_file)?;

    let mut session2 = Session::default();
    session2.moves.push(MoveRecord {
        source: PathBuf::from("new"),
        destination: PathBuf::from("new_dst"),
        timestamp: Utc::now(),
        file_size: 0,
    });
    session2.save(root, history_file)?;

    let loaded = Session::load(root, history_file)?;
    assert_eq!(
        loaded.moves.len(),
        1,
        "New save should overwrite the old history file contents"
    );
    assert_eq!(
        loaded.moves[0].source,
        PathBuf::from("new"),
        "Only the newest record should exist after overwrite"
    );
    Ok(())
}

#[test]
fn test_history_deeply_nested_paths() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let root = tmp.path();
    let history_file = "nested.json";

    let deep_path = "a/".repeat(50) + "file.txt";
    let mut session = Session::default();
    session.moves.push(MoveRecord {
        source: PathBuf::from(&deep_path),
        destination: PathBuf::from("extracted/file.txt"),
        timestamp: Utc::now(),
        file_size: 0,
    });

    session.save(root, history_file)?;
    let loaded = Session::load(root, history_file)?;

    assert_eq!(
        loaded.moves[0].source,
        PathBuf::from(deep_path),
        "Deeply nested paths should be preserved correctly"
    );
    Ok(())
}

#[test]
fn test_history_missing_history_returns_default() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let root = tmp.path();

    let loaded = Session::load(root, "non_existent.json")?;
    assert!(
        loaded.moves.is_empty(),
        "Loading missing file should return an empty default session rather than failing"
    );
    Ok(())
}

#[test]
fn test_history_massive_filename_length() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let root = tmp.path();
    let history_file = "long_filename.json";

    let long_name = "x".repeat(1000);
    let mut session = Session::default();
    session.moves.push(MoveRecord {
        source: PathBuf::from(&long_name),
        destination: PathBuf::from("dest"),
        timestamp: Utc::now(),
        file_size: 0,
    });

    session.save(root, history_file)?;
    let loaded = Session::load(root, history_file)?;

    assert_eq!(
        loaded.moves[0].source,
        PathBuf::from(long_name),
        "Extremely long filenames should be handled without truncation"
    );
    Ok(())
}

#[test]
fn test_history_exact_timestamp_preservation() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let root = tmp.path();
    let history_file = "time.json";

    let now = Utc::now();
    let mut session = Session::default();
    session.moves.push(MoveRecord {
        source: PathBuf::from("s"),
        destination: PathBuf::from("d"),
        timestamp: now,
        file_size: 0,
    });

    session.save(root, history_file)?;
    let loaded = Session::load(root, history_file)?;

    assert_eq!(
        loaded.moves[0].timestamp.to_rfc3339(),
        now.to_rfc3339(),
        "RFC3339 timestamp representation should be identical after load"
    );
    Ok(())
}
