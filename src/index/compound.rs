//! Compound (multi-field) index structure

use std::collections::HashMap;

/// Compound index: combined_key → [document positions]
///
/// Maps combinations of multiple field values to document positions.
/// The combined key is created by joining field values with "|".
///
/// # Examples
///
/// ```rust
/// use jsonq::index::CompoundIndex;
///
/// let mut index = CompoundIndex::new();
/// 
/// // Index on (city, role)
/// index.insert("NYC|admin".to_string(), vec![0, 3]);
/// index.insert("LA|user".to_string(), vec![1, 2]);
///
/// assert_eq!(index.get("NYC|admin"), Some(&vec![0, 3]));
/// ```
#[derive(Debug, Clone)]
pub struct CompoundIndex {
    /// Maps combined keys to document positions
    map: HashMap<String, Vec<usize>>,
}

impl CompoundIndex {
    /// Create a new empty compound index
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }
    
    /// Insert a combined key → positions mapping
    pub fn insert(&mut self, combined_key: String, positions: Vec<usize>) {
        self.map.insert(combined_key, positions);
    }
    
    /// Get positions for a combined key
    pub fn get(&self, combined_key: &str) -> Option<&Vec<usize>> {
        self.map.get(combined_key)
    }
    
    /// Get number of unique combinations
    pub fn unique_count(&self) -> usize {
        self.map.len()
    }
    
    /// Get total number of entries
    pub fn total_entries(&self) -> usize {
        self.map.values().map(|v| v.len()).sum()
    }
    
    /// Clear the index
    pub fn clear(&mut self) {
        self.map.clear();
    }
}

impl Default for CompoundIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compound_index_new() {
        let index = CompoundIndex::new();
        assert_eq!(index.unique_count(), 0);
    }

    #[test]
    fn test_compound_index_insert_and_get() {
        let mut index = CompoundIndex::new();
        index.insert("NYC|admin".to_string(), vec![0, 3]);
        
        assert_eq!(index.get("NYC|admin"), Some(&vec![0, 3]));
        assert_eq!(index.get("LA|user"), None);
    }

    #[test]
    fn test_compound_index_multiple_combinations() {
        let mut index = CompoundIndex::new();
        index.insert("NYC|admin".to_string(), vec![0]);
        index.insert("NYC|user".to_string(), vec![1, 2]);
        index.insert("LA|admin".to_string(), vec![3]);
        
        assert_eq!(index.unique_count(), 3);
        assert_eq!(index.total_entries(), 4);
    }

    #[test]
    fn test_compound_index_clear() {
        let mut index = CompoundIndex::new();
        index.insert("key1|key2".to_string(), vec![1, 2, 3]);
        
        index.clear();
        
        assert_eq!(index.unique_count(), 0);
        assert_eq!(index.get("key1|key2"), None);
    }
}
