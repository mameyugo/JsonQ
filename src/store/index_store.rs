//! Index storage and management

use std::collections::HashMap;

/// Storage for collection indexes
///
/// Tracks both single-field and compound indexes with their build times
#[derive(Debug)]
pub struct IndexStore {
    /// Single-field indexes: field_name -> {value -> [positions]}
    pub single: HashMap<String, HashMap<String, Vec<usize>>>,
    
    /// Compound indexes: [field1, field2, ...] -> {combined_key -> [positions]}
    pub compound: HashMap<Vec<String>, HashMap<String, Vec<usize>>>,
    
    /// When these indexes were built (Unix timestamp)
    /// Used to invalidate indexes when data changes
    pub built_at: u64,
}

impl IndexStore {
    /// Create an empty index store
    pub fn new() -> Self {
        Self {
            single: HashMap::new(),
            compound: HashMap::new(),
            built_at: 0,
        }
    }
    
    /// Check if indexes are still valid for given data modification time
    pub fn is_valid(&self, data_mtime: u64) -> bool {
        self.built_at >= data_mtime
    }
    
    /// Clear all indexes
    pub fn clear(&mut self) {
        self.single.clear();
        self.compound.clear();
        self.built_at = 0;
    }
    
    /// Get total number of indexes
    pub fn count(&self) -> usize {
        self.single.len() + self.compound.len()
    }
}

impl Default for IndexStore {
    fn default() -> Self {
        Self::new()
    }
}
