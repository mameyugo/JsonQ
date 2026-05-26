//! Cache implementation for stored data

use serde_json::Value;
use std::sync::Arc;

/// Cached data with modification time tracking
///
/// Uses Arc for cheap cloning and mtime for invalidation
#[derive(Debug, Clone)]
pub struct CachedData {
    /// Shared reference to the cached JSON data
    pub data: Arc<Value>,

    /// Modification time (nanoseconds since UNIX epoch)
    /// Used to detect file changes and invalidate cache
    pub mtime: u64,

    /// File inode (on Unix systems) to detect atomic renames
    pub inode: u64,
}

impl CachedData {
    /// Create a new cache entry
    pub fn new(data: Arc<Value>, mtime: u64, inode: u64) -> Self {
        Self { data, mtime, inode }
    }

    /// Check if cache is still valid for given file mtime and inode
    pub fn is_valid(&self, file_mtime: u64, file_inode: u64) -> bool {
        self.mtime == file_mtime && self.inode == file_inode
    }
}
