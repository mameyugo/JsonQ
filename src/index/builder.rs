//! Index builder for creating indexes from collections

use serde_json::Value;
use super::{SingleIndex, CompoundIndex};
use crate::path::read_path;
use crate::utils::value_key;

/// Builder for creating indexes from JSON collections
///
/// # Examples
///
/// ```rust
/// use jsonq::index::IndexBuilder;
/// use serde_json::json;
///
/// let collection = vec![
///     json!({"id": 1, "role": "admin", "city": "NYC"}),
///     json!({"id": 2, "role": "user", "city": "LA"}),
///     json!({"id": 3, "role": "admin", "city": "NYC"}),
/// ];
///
/// let builder = IndexBuilder::new();
///
/// // Single-field index
/// let role_index = builder.build_single(&collection, "role");
/// assert_eq!(role_index.unique_count(), 2);
///
/// // Compound index
/// let compound_index = builder.build_compound(&collection, &["city", "role"]);
/// assert_eq!(compound_index.unique_count(), 2);
/// ```
pub struct IndexBuilder;

impl IndexBuilder {
    /// Create a new index builder
    pub fn new() -> Self {
        Self
    }
    
    /// Build a single-field index
    ///
    /// Creates an index on a single field, mapping each unique value
    /// to the positions of documents containing that value.
    ///
    /// # Arguments
    ///
    /// * `collection` - Array of JSON objects to index
    /// * `field` - Dot-notation path to the field to index
    ///
    /// # Examples
    ///
    /// ```rust
    /// use jsonq::index::IndexBuilder;
    /// use serde_json::json;
    ///
    /// let docs = vec![
    ///     json!({"name": "Alice", "role": "admin"}),
    ///     json!({"name": "Bob", "role": "user"}),
    ///     json!({"name": "Charlie", "role": "admin"}),
    /// ];
    ///
    /// let builder = IndexBuilder::new();
    /// let index = builder.build_single(&docs, "role");
    ///
    /// assert_eq!(index.get("admin").unwrap().len(), 2);
    /// assert_eq!(index.get("user").unwrap().len(), 1);
    /// ```
    pub fn build_single(&self, collection: &[Value], field: &str) -> SingleIndex {
        let mut index = SingleIndex::new();
        let mut temp_map: std::collections::HashMap<String, Vec<usize>> = 
            std::collections::HashMap::new();
        
        for (position, item) in collection.iter().enumerate() {
            let field_value = read_path(item, field);
            let key = value_key(field_value);
            
            temp_map.entry(key).or_insert_with(Vec::new).push(position);
        }
        
        for (key, positions) in temp_map {
            index.insert(key, positions);
        }
        
        index
    }
    
    /// Build a compound index on multiple fields
    ///
    /// Creates an index on multiple fields by combining their values
    /// into a single key (joined with "|").
    ///
    /// # Arguments
    ///
    /// * `collection` - Array of JSON objects to index
    /// * `fields` - Array of field paths to index together
    ///
    /// # Examples
    ///
    /// ```rust
    /// use jsonq::index::IndexBuilder;
    /// use serde_json::json;
    ///
    /// let docs = vec![
    ///     json!({"city": "NYC", "role": "admin"}),
    ///     json!({"city": "NYC", "role": "user"}),
    ///     json!({"city": "LA", "role": "admin"}),
    /// ];
    ///
    /// let builder = IndexBuilder::new();
    /// let index = builder.build_compound(&docs, &["city", "role"]);
    ///
    /// assert_eq!(index.unique_count(), 3);
    /// assert!(index.get("NYC|admin").is_some());
    /// ```
    pub fn build_compound(&self, collection: &[Value], fields: &[&str]) -> CompoundIndex {
        let mut index = CompoundIndex::new();
        let mut temp_map: std::collections::HashMap<String, Vec<usize>> = 
            std::collections::HashMap::new();
        
        for (position, item) in collection.iter().enumerate() {
            // Build combined key from all fields
            let keys: Vec<String> = fields
                .iter()
                .map(|field| {
                    let field_value = read_path(item, field);
                    value_key(field_value)
                })
                .collect();
            
            let combined_key = keys.join("|");
            temp_map.entry(combined_key).or_insert_with(Vec::new).push(position);
        }
        
        for (key, positions) in temp_map {
            index.insert(key, positions);
        }
        
        index
    }
}

impl Default for IndexBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn sample_collection() -> Vec<Value> {
        vec![
            json!({"id": 1, "name": "Alice", "role": "admin", "city": "NYC"}),
            json!({"id": 2, "name": "Bob", "role": "user", "city": "LA"}),
            json!({"id": 3, "name": "Charlie", "role": "admin", "city": "NYC"}),
            json!({"id": 4, "name": "Diana", "role": "user", "city": "Chicago"}),
            json!({"id": 5, "name": "Eve", "role": "viewer", "city": "LA"}),
        ]
    }

    #[test]
    fn test_build_single_index() {
        let collection = sample_collection();
        let builder = IndexBuilder::new();
        let index = builder.build_single(&collection, "role");
        
        assert_eq!(index.unique_count(), 3);
        assert_eq!(index.get("admin").unwrap(), &vec![0, 2]);
        assert_eq!(index.get("user").unwrap(), &vec![1, 3]);
        assert_eq!(index.get("viewer").unwrap(), &vec![4]);
    }

    #[test]
    fn test_build_single_index_nested_field() {
        let collection = vec![
            json!({"user": {"profile": {"role": "admin"}}}),
            json!({"user": {"profile": {"role": "user"}}}),
            json!({"user": {"profile": {"role": "admin"}}}),
        ];
        
        let builder = IndexBuilder::new();
        let index = builder.build_single(&collection, "user.profile.role");
        
        assert_eq!(index.unique_count(), 2);
        assert_eq!(index.get("admin").unwrap(), &vec![0, 2]);
    }

    #[test]
    fn test_build_compound_index() {
        let collection = sample_collection();
        let builder = IndexBuilder::new();
        let index = builder.build_compound(&collection, &["city", "role"]);
        
        assert_eq!(index.unique_count(), 4);
        assert_eq!(index.get("NYC|admin").unwrap(), &vec![0, 2]);
        assert_eq!(index.get("LA|user").unwrap(), &vec![1]);
        assert_eq!(index.get("Chicago|user").unwrap(), &vec![3]);
        assert_eq!(index.get("LA|viewer").unwrap(), &vec![4]);
    }

    #[test]
    fn test_build_compound_index_three_fields() {
        let collection = vec![
            json!({"a": 1, "b": 2, "c": 3}),
            json!({"a": 1, "b": 2, "c": 4}),
            json!({"a": 1, "b": 3, "c": 3}),
        ];
        
        let builder = IndexBuilder::new();
        let index = builder.build_compound(&collection, &["a", "b", "c"]);
        
        assert_eq!(index.unique_count(), 3);
        assert!(index.get("1|2|3").is_some());
        assert!(index.get("1|2|4").is_some());
        assert!(index.get("1|3|3").is_some());
    }

    #[test]
    fn test_build_index_empty_collection() {
        let collection: Vec<Value> = vec![];
        let builder = IndexBuilder::new();
        let index = builder.build_single(&collection, "role");
        
        assert_eq!(index.unique_count(), 0);
    }

    #[test]
    fn test_build_index_missing_field() {
        let collection = vec![
            json!({"name": "Alice"}),
            json!({"name": "Bob"}),
        ];
        
        let builder = IndexBuilder::new();
        let index = builder.build_single(&collection, "role");
        
        // All missing values map to "null"
        assert_eq!(index.unique_count(), 1);
        assert_eq!(index.get("null").unwrap(), &vec![0, 1]);
    }

    #[test]
    fn test_build_index_mixed_types() {
        let collection = vec![
            json!({"value": "text"}),
            json!({"value": 42}),
            json!({"value": true}),
            json!({"value": null}),
        ];
        
        let builder = IndexBuilder::new();
        let index = builder.build_single(&collection, "value");
        
        assert_eq!(index.unique_count(), 4);
        assert!(index.get("text").is_some());
        assert!(index.get("42").is_some());
        assert!(index.get("true").is_some());
        assert!(index.get("null").is_some());
    }

    #[test]
    fn test_build_index_duplicate_values() {
        let collection = vec![
            json!({"role": "admin"}),
            json!({"role": "admin"}),
            json!({"role": "admin"}),
        ];
        
        let builder = IndexBuilder::new();
        let index = builder.build_single(&collection, "role");
        
        assert_eq!(index.unique_count(), 1);
        assert_eq!(index.get("admin").unwrap(), &vec![0, 1, 2]);
    }
}
