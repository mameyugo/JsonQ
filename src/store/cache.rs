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
    
    /// Modification time (seconds since UNIX epoch)
    /// Used to detect file changes and invalidate cache
    pub mtime: u64,
}

impl CachedData {
    /// Create a new cache entry
    pub fn new(data: Arc<Value>, mtime: u64) -> Self {
        Self { data, mtime }
    }
    
    /// Check if cache is still valid for given file mtime
    pub fn is_valid(&self, file_mtime: u64) -> bool {
        self.mtime == file_mtime
    }
}
