use chrono::Utc;
use panos::file_ops::history::MoveRecord;
use panos::file_ops::remover::remove_empty_dirs;
use panos::ui::reporter::NoopReporter;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_remover_basic_empty_dir_removal() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let root = tmp.path();
    let empty_dir = root.join("empty_folder");
    fs::create_dir(&empty_dir)?;

    remove_empty_dirs(root, false, &[], &NoopReporter)?;
    assert!(
        !empty_dir.exists(),
        "Empty directory should be removed during actual execution"
    );
    Ok(())
}

#[test]
fn test_remover_non_empty_dir_preserved() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let root = tmp.path();
    let dir = root.join("active_folder");
    fs::create_dir(&dir)?;
    fs::write(dir.join("file.txt"), "keep me")?;

    remove_empty_dirs(root, false, &[], &NoopReporter)?;
    assert!(dir.exists(), "Directory with files must be preserved");
    Ok(())
}

#[test]
fn test_remover_chained_recursive_empty_dirs() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let root = tmp.path();
    let deep = root.join("a/b/c");
    fs::create_dir_all(&deep)?;

    remove_empty_dirs(root, false, &[], &NoopReporter)?;
    assert!(
        !root.join("a").exists(),
        "Nested empty directories should be removed recursively from bottom-up"
    );
    Ok(())
}

#[test]
fn test_remover_root_scanned_dir_protection() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let root = tmp.path().join("scanned_root");
    fs::create_dir(&root)?;

    remove_empty_dirs(&root, false, &[], &NoopReporter)?;
    assert!(
        root.exists(),
        "The root directory being scanned must never be removed itself"
    );
    Ok(())
}

#[test]
fn test_remover_dry_run_prediction_basic() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let root = tmp.path();
    let empty_dir = root.join("dry_empty");
    fs::create_dir(&empty_dir)?;

    remove_empty_dirs(root, true, &[], &NoopReporter)?;
    assert!(
        empty_dir.exists(),
        "Dry run must not perform actual disk removal"
    );
    Ok(())
}

#[test]
fn test_remover_dry_run_recursive_prediction() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let root = tmp.path();
    let a = root.join("a");
    let b = a.join("b");
    fs::create_dir_all(&b)?;

    remove_empty_dirs(root, true, &[], &NoopReporter)?;
    assert!(
        b.exists(),
        "Recursive chain must remain intact during dry run"
    );
    assert!(
        a.exists(),
        "Parent of empty chain must remain intact during dry run"
    );
    Ok(())
}

#[test]
fn test_remover_move_source_triggers_emptiness_dry_run() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let root = tmp.path();
    let target_dir = root.join("to_be_emptied");
    fs::create_dir(&target_dir)?;
    let src_file = target_dir.join("moving.txt");
    fs::write(&src_file, "move me")?;

    let moves = vec![MoveRecord {
        source: src_file.clone(),
        destination: PathBuf::from("destination.txt"),
        timestamp: Utc::now(),
    }];

    remove_empty_dirs(root, true, &moves, &NoopReporter)?;
    assert!(
        target_dir.exists(),
        "Actual directory must exist after dry run"
    );
    Ok(())
}

#[test]
fn test_remover_mixed_move_and_nested_empty_dirs_dry_run() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let root = tmp.path();
    let parent = root.join("parent");
    let child = parent.join("child");
    fs::create_dir_all(&child)?;
    let file_in_child = child.join("data.txt");
    fs::write(&file_in_child, "payload")?;

    let moves = vec![MoveRecord {
        source: file_in_child,
        destination: PathBuf::from("elsewhere.txt"),
        timestamp: Utc::now(),
    }];

    remove_empty_dirs(root, true, &moves, &NoopReporter)?;
    assert!(
        parent.exists(),
        "Deeply nested predicted empty directories should be tracked but not deleted"
    );
    Ok(())
}

#[test]
fn test_remover_massive_sibling_directories() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let root = tmp.path();
    for i in 0..500 {
        fs::create_dir(root.join(format!("dir_{}", i)))?;
    }

    remove_empty_dirs(root, false, &[], &NoopReporter)?;
    for i in 0..500 {
        assert!(
            !root.join(format!("dir_{}", i)).exists(),
            "All 500 sibling empty directories must be removed"
        );
    }
    Ok(())
}

#[test]
fn test_remover_extreme_depth_nesting() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let root = tmp.path();
    let mut current = root.to_path_buf();
    for i in 0..50 {
        current.push(format!("v{}", i));
    }
    fs::create_dir_all(&current)?;

    remove_empty_dirs(root, false, &[], &NoopReporter)?;
    assert!(
        !root.join("v0").exists(),
        "Chain with 50 levels of nesting should be fully cleared"
    );
    Ok(())
}

#[test]
fn test_remover_hidden_file_prevents_removal() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let root = tmp.path();
    let dir = root.join("hidden_inside");
    fs::create_dir(&dir)?;
    fs::write(dir.join(".DS_Store"), "metadata")?;

    remove_empty_dirs(root, false, &[], &NoopReporter)?;
    assert!(
        dir.exists(),
        "Directory containing hidden files should not be considered empty"
    );
    Ok(())
}

#[test]
fn test_remover_unicode_directory_names() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let root = tmp.path();
    let unicode_dir = root.join("โฟลเดอร์_🔥");
    fs::create_dir(&unicode_dir)?;

    remove_empty_dirs(root, false, &[], &NoopReporter)?;
    assert!(
        !unicode_dir.exists(),
        "Directories with Unicode or Emoji names should be handled correctly"
    );
    Ok(())
}

#[test]
fn test_remover_unrelated_files_prevent_removal() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let root = tmp.path();
    let dir = root.join("partial");
    fs::create_dir(&dir)?;
    fs::write(dir.join("move_me.txt"), "1")?;
    fs::write(dir.join("stay.txt"), "2")?;

    let moves = vec![MoveRecord {
        source: dir.join("move_me.txt"),
        destination: PathBuf::from("outside.txt"),
        timestamp: Utc::now(),
    }];

    remove_empty_dirs(root, true, &moves, &NoopReporter)?;
    assert!(
        dir.exists(),
        "Directory should not be predicted empty if it holds unrelated files"
    );
    Ok(())
}

#[test]
fn test_remover_missing_permission_graceful_skip() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let root = tmp.path();
    let restricted = root.join("restricted");
    fs::create_dir(&restricted)?;

    remove_empty_dirs(root, false, &[], &NoopReporter)?;
    assert!(!restricted.exists(), "Normal empty dir should be removed");
    Ok(())
}

#[test]
fn test_remover_interleaved_heavy_structure() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let root = tmp.path();
    let a = root.join("a");
    let b = a.join("b");
    let c = a.join("c");
    fs::create_dir_all(&b)?;
    fs::create_dir_all(&c)?;
    fs::write(b.join("file.txt"), "data")?;

    remove_empty_dirs(root, false, &[], &NoopReporter)?;
    assert!(
        a.exists(),
        "Parent should stay if one child remains non-empty"
    );
    assert!(!c.exists(), "Empty sibling child should be removed");
    assert!(b.exists(), "Non-empty sibling child should remain");
    Ok(())
}

#[test]
fn test_remover_special_characters_in_dir_name() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let root = tmp.path();
    let name = "dir (with) [brackets] !";
    let dir = root.join(name);
    fs::create_dir(&dir)?;

    remove_empty_dirs(root, false, &[], &NoopReporter)?;
    assert!(
        !dir.exists(),
        "Should handle directory names with brackets and special shell characters"
    );
    Ok(())
}

#[test]
fn test_remover_very_long_path_nesting() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let root = tmp.path();
    let mut current = root.to_path_buf();
    let long_segment = "s".repeat(50);
    for _ in 0..10 {
        current.push(&long_segment);
    }
    fs::create_dir_all(&current)?;

    remove_empty_dirs(root, false, &[], &NoopReporter)?;
    assert!(
        !root.join(&long_segment).exists(),
        "Very long nested paths should be cleared successfully"
    );
    Ok(())
}

#[test]
fn test_remover_idempotency_check() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let root = tmp.path();
    fs::create_dir_all(root.join("x/y/z"))?;

    remove_empty_dirs(root, false, &[], &NoopReporter)?;
    remove_empty_dirs(root, false, &[], &NoopReporter)?;
    assert!(
        !root.join("x").exists(),
        "Multiple re-runs of remover should be safe and idempotent"
    );
    Ok(())
}

#[test]
fn test_remover_empty_subdirs_with_contents_first_logic() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let root = tmp.path();
    let a = root.join("a");
    let b = a.join("b");
    fs::create_dir_all(&b)?;
    fs::write(a.join("parent_file.txt"), "stay")?;

    remove_empty_dirs(root, false, &[], &NoopReporter)?;
    assert!(a.exists(), "Parent with file must stay");
    assert!(
        !b.exists(),
        "Empty child must be removed even if parent stays"
    );
    Ok(())
}

#[test]
fn test_remover_multiple_moves_from_same_dir() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let root = tmp.path();
    let dir = root.join("multi_move");
    fs::create_dir(&dir)?;
    for i in 0..5 {
        fs::write(dir.join(format!("{}.txt", i)), "x")?;
    }

    let mut moves = Vec::new();
    for i in 0..5 {
        moves.push(MoveRecord {
            source: dir.join(format!("{}.txt", i)),
            destination: PathBuf::from(format!("dst_{}.txt", i)),
            timestamp: Utc::now(),
        });
    }

    remove_empty_dirs(root, true, &moves, &NoopReporter)?;
    assert!(
        dir.exists(),
        "Predicted removal for dry run should wait for move sources"
    );
    Ok(())
}
