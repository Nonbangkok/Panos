//! File operations module

pub mod hashing;
pub mod history;
pub mod integrity;
pub mod mover;
pub mod remover;

pub use hashing::{calculate_full_hash, calculate_partial_hash, get_file_size};
pub use history::{MoveRecord, Session};
pub use integrity::check_integrity;
pub use mover::move_file;
pub use remover::remove_empty_dirs;
