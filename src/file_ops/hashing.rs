//! Hashing module

use std::hash::Hasher;
use twox_hash::XxHash64;
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
 
    let mut hasher = XxHash64::with_seed(0);
    hasher.write(&buffer[..n]);
 
    Ok(format!("{:016x}", hasher.finish()))
}

pub fn calculate_full_hash<P: AsRef<Path>>(path: P) -> Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = XxHash64::with_seed(0);
    let mut buffer = [0u8; 8192]; // 8KB buffer for efficient reading
 
    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hasher.write(&buffer[..n]);
    }
 
    Ok(format!("{:016x}", hasher.finish()))
}
