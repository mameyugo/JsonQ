//! Cleanup system for temporary files
//!
//! Handles orphaned temporary files from:
//! - Crashed writes (.tmp files)
//! - Abandoned transactions (.tx files)
//! - Stale locks (.lock files)

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// Maximum age for temporary files before cleanup (1 hour)
const MAX_TEMP_FILE_AGE: Duration = Duration::from_secs(3600);

/// Clean up orphaned temporary files
///
/// Removes:
/// - .tmp files older than 1 hour
/// - .tx files older than 1 hour
/// - .lock files that aren't actually locked
pub fn cleanup_temp_files(data_path: &Path) -> Result<usize, String> {
    let mut cleaned = 0;

    let parent = data_path
        .parent()
        .ok_or_else(|| "Cannot determine parent directory".to_string())?;

    if !parent.exists() {
        return Ok(0);
    }

    let base_name = data_path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| "Invalid filename".to_string())?;

    // Clean .tmp files
    cleaned += cleanup_extension(parent, base_name, "tmp")?;

    // Clean .tx (transaction) files
    cleaned += cleanup_extension(parent, base_name, "tx")?;

    // Clean orphaned .lock files
    cleaned += cleanup_locks(parent, base_name)?;

    Ok(cleaned)
}

/// Clean files with specific extension
fn cleanup_extension(parent: &Path, base_name: &str, ext: &str) -> Result<usize, String> {
    let mut cleaned = 0;

    // Pattern: basename.json.ext or basename.ext
    let patterns = vec![
        format!("{}.json.{}", base_name, ext),
        format!("{}.{}", base_name, ext),
    ];

    for pattern in patterns {
        let temp_file = parent.join(&pattern);

        if temp_file.exists() {
            if is_file_old_enough(&temp_file)? {
                fs::remove_file(&temp_file)
                    .map_err(|e| format!("Failed to remove {}: {}", temp_file.display(), e))?;
                cleaned += 1;
            }
        }
    }

    Ok(cleaned)
}

/// Clean orphaned lock files
fn cleanup_locks(parent: &Path, base_name: &str) -> Result<usize, String> {
    let lock_file = parent.join(format!("{}.json.lock", base_name));

    if !lock_file.exists() {
        return Ok(0);
    }

    // Check if file is old enough
    if !is_file_old_enough(&lock_file)? {
        return Ok(0);
    }

    // Try to acquire exclusive lock (non-blocking)
    // If we can lock it, it means no process is using it
    if let Ok(file) = fs::File::open(&lock_file) {
        use fs2::FileExt;

        match file.try_lock_exclusive() {
            Ok(_) => {
                // Lock was available = file is orphaned
                drop(file);
                fs::remove_file(&lock_file).map_err(|e| format!("Failed to remove lock: {}", e))?;
                Ok(1)
            }
            Err(_) => {
                // Lock is held by another process = leave it
                Ok(0)
            }
        }
    } else {
        Ok(0)
    }
}

/// Check if file is older than MAX_TEMP_FILE_AGE
fn is_file_old_enough(path: &Path) -> Result<bool, String> {
    let metadata = fs::metadata(path).map_err(|e| format!("Failed to read metadata: {}", e))?;

    let modified = metadata
        .modified()
        .map_err(|e| format!("Failed to get modification time: {}", e))?;

    let age = SystemTime::now()
        .duration_since(modified)
        .unwrap_or(Duration::from_secs(0));

    Ok(age > MAX_TEMP_FILE_AGE)
}

/// RAII guard for temporary files
///
/// Automatically deletes the temporary file when dropped,
/// unless explicitly kept via `keep()` method.
pub struct TempFileGuard {
    path: PathBuf,
    keep: bool,
}

impl TempFileGuard {
    /// Create a new temporary file guard
    pub fn new<P: Into<PathBuf>>(path: P) -> Result<Self, String> {
        Ok(Self {
            path: path.into(),
            keep: false,
        })
    }

    /// Keep the file (don't delete on drop)
    pub fn keep(&mut self) {
        self.keep = true;
    }

    /// Get path to temporary file
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempFileGuard {
    fn drop(&mut self) {
        if !self.keep && self.path.exists() {
            let _ = fs::remove_file(&self.path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_temp_file_guard_auto_delete() {
        let temp_dir = env::temp_dir();
        let test_path = temp_dir.join("guard_test.tmp");

        fs::write(&test_path, b"test").unwrap();
        assert!(test_path.exists());

        {
            let _guard = TempFileGuard::new(&test_path).unwrap();
            // guard goes out of scope without keep()
        }

        // File should be deleted
        assert!(!test_path.exists());
    }

    #[test]
    fn test_temp_file_guard_keep() {
        let temp_dir = env::temp_dir();
        let test_path = temp_dir.join("guard_keep_test.tmp");

        fs::write(&test_path, b"test").unwrap();
        assert!(test_path.exists());

        {
            let mut guard = TempFileGuard::new(&test_path).unwrap();
            guard.keep(); // Keep the file
        }

        // File should still exist
        assert!(test_path.exists());

        // Cleanup
        fs::remove_file(&test_path).unwrap();
    }
}
