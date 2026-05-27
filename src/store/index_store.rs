//! Index storage and management

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VectorEntry {
    pub index: usize,
    pub vector: Vec<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VectorIndex {
    pub dimension: Option<usize>,
    pub metric: String,
    pub entries: Vec<VectorEntry>,
    pub built_at: u64,
}

/// Storage for collection indexes
///
/// Tracks both single-field, compound, and vector indexes with their built times
#[derive(Debug, Serialize, Deserialize)]
pub struct IndexStore {
    /// Single-field indexes: field_name -> {value -> [positions]}
    pub single: HashMap<String, HashMap<String, Vec<usize>>>,

    /// Compound indexes: [field1, field2, ...] -> {combined_key -> [positions]}
    pub compound: HashMap<Vec<String>, HashMap<String, Vec<usize>>>,

    /// Vector indexes: field_name -> VectorIndex
    #[serde(default)]
    pub vector: HashMap<String, VectorIndex>,

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
            vector: HashMap::new(),
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
        self.vector.clear();
        self.built_at = 0;
    }

    /// Get total number of indexes
    pub fn count(&self) -> usize {
        self.single.len() + self.compound.len() + self.vector.len()
    }
}

impl Default for IndexStore {
    fn default() -> Self {
        Self::new()
    }
}
