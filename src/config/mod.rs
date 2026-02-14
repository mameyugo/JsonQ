//! Global configuration for JsonQ extension

pub mod php_ini;

use std::sync::RwLock;
use std::path::PathBuf;

/// Global configuration singleton
static CONFIG: RwLock<Option<Config>> = RwLock::new(None);

/// Configuration for JsonQ extension
#[derive(Debug, Clone)]
pub struct Config {
    /// Maximum file size in bytes (default: 100MB)
    pub max_file_size: u64,
    
    /// Maximum validation depth (default: 100)
    pub max_validation_depth: usize,
    
    /// Maximum path depth for dot notation (default: 50)
    pub max_path_depth: usize,
    
    /// Allowed file extensions (default: ["json"])
    pub allowed_extensions: Vec<String>,
    
    /// Base path - files must be within this directory
    /// None = no restriction (default)
    pub base_path: Option<PathBuf>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            max_file_size: 100 * 1024 * 1024,  // 100MB
            max_validation_depth: 100,
            max_path_depth: 50,
            allowed_extensions: vec!["json".to_string()],
            base_path: None,
        }
    }
}

impl Config {
    /// Initialize global configuration
    pub fn init() {
        let mut config = CONFIG.write().unwrap();
        if config.is_none() {
            *config = Some(Self::default());
        }
    }
    
    /// Get current configuration (read-only)
    pub fn get() -> Config {
        CONFIG.read().unwrap()
            .clone()
            .unwrap_or_default()
    }
    
    /// Update configuration
    pub fn update<F>(f: F)
    where
        F: FnOnce(&mut Config),
    {
        let mut config = CONFIG.write().unwrap();
        if let Some(cfg) = config.as_mut() {
            f(cfg);
        } else {
            let mut new_config = Config::default();
            f(&mut new_config);
            *config = Some(new_config);
        }
    }
    
    /// Parse file size string (e.g., "50M", "1G", "500K")
    pub fn parse_size(s: &str) -> Result<u64, String> {
        let s = s.trim().to_uppercase();
        
        if s.is_empty() {
            return Err("Empty size string".to_string());
        }
        
        let (num_part, suffix) = if s.ends_with('G') {
            (&s[..s.len()-1], 1024 * 1024 * 1024)
        } else if s.ends_with('M') {
            (&s[..s.len()-1], 1024 * 1024)
        } else if s.ends_with('K') {
            (&s[..s.len()-1], 1024)
        } else if s.ends_with('B') {
            (&s[..s.len()-1], 1)
        } else {
            (s.as_str(), 1)
        };
        
        num_part.parse::<u64>()
            .map(|n| n * suffix)
            .map_err(|e| format!("Invalid size format: {}", e))
    }
    
    /// Parse comma-separated extensions
    pub fn parse_extensions(s: &str) -> Vec<String> {
        s.split(',')
            .map(|e| e.trim().to_lowercase())
            .filter(|e| !e.is_empty())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_size() {
        assert_eq!(Config::parse_size("100M").unwrap(), 100 * 1024 * 1024);
        assert_eq!(Config::parse_size("1G").unwrap(), 1024 * 1024 * 1024);
        assert_eq!(Config::parse_size("500K").unwrap(), 500 * 1024);
        assert_eq!(Config::parse_size("1024").unwrap(), 1024);
        assert_eq!(Config::parse_size("1024B").unwrap(), 1024);
    }

    #[test]
    fn test_parse_extensions() {
        let exts = Config::parse_extensions("json, db, data");
        assert_eq!(exts, vec!["json", "db", "data"]);
        
        let exts = Config::parse_extensions("JSON,DB");
        assert_eq!(exts, vec!["json", "db"]);
    }

    #[test]
    fn test_default_config() {
        let cfg = Config::default();
        assert_eq!(cfg.max_file_size, 100 * 1024 * 1024);
        assert_eq!(cfg.max_validation_depth, 100);
        assert_eq!(cfg.allowed_extensions, vec!["json"]);
        assert!(cfg.base_path.is_none());
    }
}
