use panos::file_ops::hashing::{calculate_full_hash, calculate_partial_hash, get_file_size};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_hashing_consistency() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let file1 = tmp.path().join("file1.txt");
    let file2 = tmp.path().join("file2.txt");
    let file3 = tmp.path().join("file3.txt");

    fs::write(&file1, "Hello, World!")?;
    fs::write(&file2, "Hello, World!")?;
    fs::write(&file3, "Hello, Rust!")?;

    assert_eq!(calculate_full_hash(&file1)?, calculate_full_hash(&file2)?);
    assert_ne!(calculate_full_hash(&file1)?, calculate_full_hash(&file3)?);

    assert_eq!(
        calculate_partial_hash(&file1)?,
        calculate_partial_hash(&file2)?
    );
    assert_ne!(
        calculate_partial_hash(&file1)?,
        calculate_partial_hash(&file3)?
    );

    assert_eq!(get_file_size(&file1)?, get_file_size(&file2)?);
    assert_ne!(get_file_size(&file1)?, get_file_size(&file3)?);

    Ok(())
}

#[test]
fn test_partial_vs_full_hash_large_files() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let file_a = tmp.path().join("large_a.bin");
    let file_b = tmp.path().join("large_b.bin");

    let data_common = vec![b'A'; 4096];

    let mut data_a = data_common.clone();
    data_a.push(b'1');

    let mut data_b = data_common;
    data_b.push(b'2');

    fs::write(&file_a, data_a)?;
    fs::write(&file_b, data_b)?;

    assert_eq!(
        calculate_partial_hash(&file_a)?,
        calculate_partial_hash(&file_b)?,
        "Partial hashes should match because the first 4KB are identical"
    );

    assert_ne!(
        calculate_full_hash(&file_a)?,
        calculate_full_hash(&file_b)?,
        "Full hashes should NOT match because the trailing bytes are different"
    );

    Ok(())
}
