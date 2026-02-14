//! File locking for concurrent access protection
//!
//! Provides cross-platform file locking to protect against race conditions
//! between processes (PHP-FPM, Apache, multiple CLI scripts).
//!
//! # Lock Types
//!
//! - **Shared lock** (read): Multiple processes can hold simultaneously
//! - **Exclusive lock** (write): Only one process can hold at a time
//!
//! # Lock Files
//!
//! Creates a `.lock` file alongside the JSON file:
//! - `/path/to/data.json` → `/path/to/data.json.lock`
//!
//! Locks are automatically released when the guard is dropped (RAII pattern).

use fs2::FileExt;
use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};

/// RAII guard for file locks
///
/// Automatically releases the lock when dropped.
/// The lock file is kept open for the duration of the lock.
#[derive(Debug)]
pub struct LockGuard {
    #[allow(dead_code)]
    file: File,
    #[allow(dead_code)]
    lock_type: LockType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LockType {
    Shared,
    Exclusive,
}

impl LockGuard {
    /// Acquire a shared (read) lock
    ///
    /// Multiple processes can hold shared locks simultaneously.
    /// Blocks if an exclusive lock is held.
    pub fn read(data_path: &Path) -> Result<Self, String> {
        let lock_path = Self::lock_path(data_path);
        let file = Self::open_lock_file(&lock_path)?;
        
        file.lock_shared()
            .map_err(|e| format!("Failed to acquire read lock on {:?}: {}", lock_path, e))?;
        
        Ok(Self {
            file,
            lock_type: LockType::Shared,
        })
    }
    
    /// Acquire an exclusive (write) lock
    ///
    /// Only one process can hold an exclusive lock at a time.
    /// Blocks if any lock (shared or exclusive) is held.
    pub fn write(data_path: &Path) -> Result<Self, String> {
        let lock_path = Self::lock_path(data_path);
        let file = Self::open_lock_file(&lock_path)?;
        
        file.lock_exclusive()
            .map_err(|e| format!("Failed to acquire write lock on {:?}: {}", lock_path, e))?;
        
        Ok(Self {
            file,
            lock_type: LockType::Exclusive,
        })
    }
    
    /// Try to acquire a shared lock without blocking
    pub fn try_read(data_path: &Path) -> Result<Self, String> {
        let lock_path = Self::lock_path(data_path);
        let file = Self::open_lock_file(&lock_path)?;
        
        file.try_lock_shared()
            .map_err(|e| format!("Failed to acquire read lock (non-blocking): {}", e))?;
        
        Ok(Self {
            file,
            lock_type: LockType::Shared,
        })
    }
    
    /// Try to acquire an exclusive lock without blocking
    pub fn try_write(data_path: &Path) -> Result<Self, String> {
        let lock_path = Self::lock_path(data_path);
        let file = Self::open_lock_file(&lock_path)?;
        
        file.try_lock_exclusive()
            .map_err(|e| format!("Failed to acquire write lock (non-blocking): {}", e))?;
        
        Ok(Self {
            file,
            lock_type: LockType::Exclusive,
        })
    }
    
    /// Get path to lock file for a data file
    fn lock_path(data_path: &Path) -> PathBuf {
        let mut lock_path = data_path.to_path_buf();
        let mut extension = lock_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_string();
        
        extension.push_str(".lock");
        lock_path.set_extension(&extension);
        
        lock_path
    }
    
    /// Open or create lock file
    fn open_lock_file(lock_path: &Path) -> Result<File, String> {
        if let Some(parent) = lock_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create lock directory: {}", e))?;
        }
        
        OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(lock_path)
            .map_err(|e| format!("Failed to open lock file: {}", e))
    }
}

impl Drop for LockGuard {
    fn drop(&mut self) {
        let _ = self.file.unlock();
    }
}
