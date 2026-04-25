//! File organization module

pub mod scanner;
pub mod undo;
pub mod watcher;

pub use scanner::organize;
pub use undo::run_undo;
pub use watcher::{WatcherPaths, watch_mode};
