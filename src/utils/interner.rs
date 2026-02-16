use std::collections::HashMap;
use std::sync::Arc;

/// String interner to deduplicate repeated JSON keys
/// Reduces memory usage in datasets with uniform structure
pub struct KeyInterner {
    cache: HashMap<String, Arc<str>>,
}

impl KeyInterner {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    /// Intern a key, returning a shared reference
    /// If the key already exists, returns the existing reference
    pub fn intern(&mut self, key: &str) -> Arc<str> {
        if let Some(existing) = self.cache.get(key) {
            return existing.clone();
        }

        let arc: Arc<str> = Arc::from(key);
        self.cache.insert(key.to_string(), arc.clone());
        arc
    }

    /// Clear the cache (useful after processing large batches if needed)
    pub fn clear(&mut self) {
        self.cache.clear();
    }

    /// Return deduplication statistics
    pub fn stats(&self) -> InternerStats {
        InternerStats {
            unique_keys: self.cache.len(),
            // Arc::strong_count includes the reference in the map itself
            // So total references in use externally = strong_count - 1
            total_references: self.cache.values()
                .map(|arc| Arc::strong_count(arc).saturating_sub(1))
                .sum(),
        }
    }
}

pub struct InternerStats {
    pub unique_keys: usize,
    pub total_references: usize,
}

impl Default for KeyInterner {
    fn default() -> Self {
        Self::new()
    }
}
