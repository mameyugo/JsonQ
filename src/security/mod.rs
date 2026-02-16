//! Security validation for file paths and operations

use std::path::{Path, PathBuf};
use std::fs;
use crate::config::Config;

/// Validate a file path against security policies
pub fn validate_path(path: &str) -> Result<PathBuf, String> {
    let config = Config::get();
    
    // 1. Convert to PathBuf
    let path_buf = PathBuf::from(path);
    
    // 2. Check for directory traversal in input components
    if path_buf.components().any(|c| matches!(c, std::path::Component::ParentDir)) {
        return Err("Directory traversal (..) is not allowed in paths".to_string());
    }

    // 3. Check extension BEFORE canonicalization
    if let Some(ext) = path_buf.extension() {
        let ext_str = ext.to_string_lossy().to_lowercase();
        if !config.allowed_extensions.contains(&ext_str) {
            return Err(format!(
                "File extension '.{}' not allowed. Allowed: {}",
                ext_str,
                config.allowed_extensions.join(", ")
            ));
        }
    } else {
        return Err("File must have an extension".to_string());
    }
    
    // 3. Canonicalize or normalize path
    let canonical = match path_buf.canonicalize() {
        Ok(p) => p,
        Err(_) => {
            // Find existing ancestor
            let mut current = path_buf.clone();
            let mut parts = Vec::new();
            
            while !current.exists() {
                if let Some(name) = current.file_name() {
                    parts.push(name.to_owned());
                }
                if let Some(parent) = current.parent() {
                    current = parent.to_path_buf();
                    if current.as_os_str().is_empty() {
                        current = PathBuf::from(".");
                        break;
                    }
                } else {
                    break;
                }
            }
            
            let mut base = current.canonicalize()
                .map_err(|e| format!("Invalid base for path: {}", e))?;
            
            for part in parts.into_iter().rev() {
                base.push(part);
            }
            base
        }
    };
    
    // 4. Check for directory traversal (should be handled by canonicalize, but double check)
    let canonical_str = canonical.to_string_lossy();
    if canonical_str.contains("..") {
        return Err("Directory traversal detected in canonical path".to_string());
    }
    
    // 5. Validate against base_path if configured
    if let Some(base_path) = &config.base_path {
        let base_canonical = base_path.canonicalize()
            .map_err(|e| format!("Invalid base path: {}", e))?;
        
        if !canonical.starts_with(&base_canonical) {
            return Err(format!(
                "Path must be within base directory: {}",
                base_canonical.display()
            ));
        }
    }
    
    Ok(canonical)
}

/// Validate file size against maximum limit
pub fn validate_file_size(path: &Path) -> Result<(), String> {
    let config = Config::get();
    
    if !path.exists() {
        return Ok(());
    }
    
    let metadata = fs::metadata(path)
        .map_err(|e| format!("Cannot read file metadata: {}", e))?;
    
    let file_size = metadata.len();
    
    if file_size > config.max_file_size {
        return Err(format!(
            "File size {} bytes exceeds maximum allowed {} bytes ({:.2} MB > {:.2} MB)",
            file_size,
            config.max_file_size,
            file_size as f64 / (1024.0 * 1024.0),
            config.max_file_size as f64 / (1024.0 * 1024.0)
        ));
    }
    
    Ok(())
}

/// Validate that a path depth doesn't exceed maximum
pub fn validate_path_depth(path: &str) -> Result<(), String> {
    let config = Config::get();
    
    let depth = if path.is_empty() {
        0
    } else {
        path.split('.').count()
    };
    
    if depth > config.max_path_depth {
        return Err(format!(
            "Path depth {} exceeds maximum allowed {}",
            depth,
            config.max_path_depth
        ));
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reject_directory_traversal() {
        assert!(validate_path("../../etc/passwd.json").is_err());
    }

    #[test]
    fn test_reject_wrong_extension() {
        Config::update(|cfg| {
            cfg.allowed_extensions = vec!["json".to_string()];
        });
        assert!(validate_path("data.xml").is_err());
    }

    #[test]
    fn test_accept_valid_extension() {
        Config::update(|cfg| {
            cfg.allowed_extensions = vec!["json".to_string()];
        });
        let result = validate_path("test.json");
        assert!(result.is_ok());
    }
}
