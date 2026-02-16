//! Safe Regex Execution
//!
//! Provides a thread-safe cache for compiled regular expressions
//! and enforces size limits to prevent memory exhaustion.

use regex::Regex;
use std::collections::HashMap;
use std::sync::OnceLock;
use std::sync::RwLock;

/// Maximum size of the regex cache (number of patterns)
const MAX_CACHE_SIZE: usize = 1000;

/// Maximum size of a compiled regex pattern in bytes (roughly)
const MAX_REGEX_SIZE: usize = 1024 * 1024; // 1MB

/// Global regex cache
static REGEX_CACHE: OnceLock<RwLock<HashMap<String, Regex>>> = OnceLock::new();

/// Get or compile a regex pattern
pub fn get_regex(pattern: &str) -> Result<Regex, String> {
    let cache = REGEX_CACHE.get_or_init(|| RwLock::new(HashMap::new()));

    // Check cache first
    {
        let read_guard = cache
            .read()
            .map_err(|e| format!("Cache lock error: {}", e))?;
        if let Some(re) = read_guard.get(pattern) {
            return Ok(re.clone());
        }
    }

    // Safety check: Don't let the cache grow indefinitely
    {
        let read_guard = cache
            .read()
            .map_err(|e| format!("Cache lock error: {}", e))?;
        if read_guard.len() >= MAX_CACHE_SIZE {
            // Optional: Implement LRU or just clear if full
            drop(read_guard);
            let mut write_guard = cache
                .write()
                .map_err(|e| format!("Cache lock error: {}", e))?;
            write_guard.clear();
        }
    }

    // Compile regex with size limit
    let re = regex::RegexBuilder::new(pattern)
        .size_limit(MAX_REGEX_SIZE)
        .build()
        .map_err(|e| format!("Regex compilation failed: {}", e))?;

    // Update cache
    let mut write_guard = cache
        .write()
        .map_err(|e| format!("Cache lock error: {}", e))?;
    write_guard.insert(pattern.to_string(), re.clone());

    Ok(re)
}

/// Maximum size of input string for regex matching (100KB to prevent stalls)
const MAX_INPUT_SIZE: usize = 100 * 1024;

/// Check if a string matches a regex pattern
pub fn is_match(text: &str, pattern: &str) -> bool {
    if text.len() > MAX_INPUT_SIZE {
        tracing::warn!("Regex input too large ({}), skipping match", text.len());
        return false;
    }

    match get_regex(pattern) {
        Ok(re) => re.is_match(text),
        Err(e) => {
            tracing::warn!("Regex error for pattern '{}': {}", pattern, e);
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regex_match() {
        assert!(is_match(
            "hello@example.com",
            r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$"
        ));
        assert!(!is_match(
            "invalid-email",
            r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$"
        ));
    }

    #[test]
    fn test_invalid_regex() {
        assert!(!is_match("test", "[invalid"));
    }

    #[test]
    fn test_cache_hits() {
        let pattern = "a+b";
        let _ = get_regex(pattern).unwrap();
        let _ = get_regex(pattern).unwrap(); // Should come from cache
    }
}
