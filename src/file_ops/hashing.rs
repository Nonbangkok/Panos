//! Hashing module

use hex::encode;
use sha2::{Digest, Sha256};
use std::fs::{File, metadata};
use std::io::{Read, Result};
use std::path::Path;

pub fn get_file_size<P: AsRef<Path>>(path: P) -> Result<u64> {
    Ok(metadata(path)?.len())
}

pub fn calculate_partial_hash<P: AsRef<Path>>(path: P) -> Result<String> {
    let mut file = File::open(path)?;
    let mut buffer = [0u8; 4096]; // 4KB buffer for efficient reading
    let n = file.read(&mut buffer)?;

    let mut hasher = Sha256::new();
    hasher.update(&buffer[..n]);

    Ok(encode(hasher.finalize()))
}

pub fn calculate_full_hash<P: AsRef<Path>>(path: P) -> Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192]; // 8KB buffer for efficient reading

    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }

    Ok(encode(hasher.finalize()))
}
