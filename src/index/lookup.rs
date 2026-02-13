//! Single-field index structure

use std::collections::HashMap;

/// Single-field index: field_value → [document positions]
///
/// Maps unique values of a field to the positions (indices) of documents
/// containing that value.
///
/// # Examples
///
/// ```rust
/// use jsonq::index::SingleIndex;
///
/// let mut index = SingleIndex::new();
/// index.insert("admin".to_string(), vec![0, 2, 5]);
/// index.insert("user".to_string(), vec![1, 3, 4]);
///
/// assert_eq!(index.get("admin"), Some(&vec![0, 2, 5]));
/// assert_eq!(index.get("guest"), None);
/// ```
#[derive(Debug, Clone)]
pub struct SingleIndex {
    /// Maps field values to document positions
    map: HashMap<String, Vec<usize>>,
}

impl SingleIndex {
    /// Create a new empty index
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }
    
    /// Insert a value → positions mapping
    pub fn insert(&mut self, value: String, positions: Vec<usize>) {
        self.map.insert(value, positions);
    }
    
    /// Get positions for a value
    pub fn get(&self, value: &str) -> Option<&Vec<usize>> {
        self.map.get(value)
    }
    
    /// Get number of unique values in the index
    pub fn unique_count(&self) -> usize {
        self.map.len()
    }
    
    /// Get total number of entries (sum of all position lists)
    pub fn total_entries(&self) -> usize {
        self.map.values().map(|v| v.len()).sum()
    }
    
    /// Clear the index
    pub fn clear(&mut self) {
        self.map.clear();
    }
}

impl Default for SingleIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_index_new() {
        let index = SingleIndex::new();
        assert_eq!(index.unique_count(), 0);
        assert_eq!(index.total_entries(), 0);
    }

    #[test]
    fn test_single_index_insert_and_get() {
        let mut index = SingleIndex::new();
        index.insert("admin".to_string(), vec![0, 2, 5]);
        
        assert_eq!(index.get("admin"), Some(&vec![0, 2, 5]));
        assert_eq!(index.get("user"), None);
    }

    #[test]
    fn test_single_index_multiple_values() {
        let mut index = SingleIndex::new();
        index.insert("admin".to_string(), vec![0, 2]);
        index.insert("user".to_string(), vec![1, 3, 4]);
        index.insert("guest".to_string(), vec![5]);
        
        assert_eq!(index.unique_count(), 3);
        assert_eq!(index.total_entries(), 6);
    }

    #[test]
    fn test_single_index_clear() {
        let mut index = SingleIndex::new();
        index.insert("test".to_string(), vec![1, 2, 3]);
        
        index.clear();
        
        assert_eq!(index.unique_count(), 0);
        assert_eq!(index.get("test"), None);
    }

    #[test]
    fn test_single_index_overwrite() {
        let mut index = SingleIndex::new();
        index.insert("key".to_string(), vec![1, 2]);
        index.insert("key".to_string(), vec![3, 4]);
        
        assert_eq!(index.get("key"), Some(&vec![3, 4]));
    }
}
