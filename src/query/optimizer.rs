//! Query Optimizer
//!
//! Analyzes query conditions to select the most efficient execution plan
//! by leveraging available indexes.

use crate::store::inner::StoreInner;
use serde_json::{Map, Value};

/// Type of execution plan
#[derive(Debug)]
pub enum ExecutionPlan {
    /// Full collection scan
    FullScan,
    /// Partial scan using an index as a filter
    IndexedScan {
        collection: String,
        field: String,
        value: Value,
        remaining_conditions: Map<String, Value>,
    },
}

/// Analyze a query and choose the best execution plan
pub fn optimize_query(inner: &StoreInner, collection: &str, conditions: &Value) -> ExecutionPlan {
    let cond_obj = match conditions.as_object() {
        Some(o) => o,
        None => return ExecutionPlan::FullScan,
    };

    if cond_obj.is_empty() {
        return ExecutionPlan::FullScan;
    }

    // Best candidate for indexing:
    // 1. Exact matches ($eq or direct value)
    // 2. $in matches (where we can union multiple index hits)

    let indexes = inner.indexes().read().unwrap();
    let store_indices = match indexes.get(collection) {
        Some(s) => s,
        None => return ExecutionPlan::FullScan,
    };

    let mut best_field = None;
    let mut best_value = None;
    let mut min_entries = usize::MAX;

    for (field, condition) in cond_obj {
        if field.starts_with('$') {
            continue;
        } // Skip logical operators for simple optimizer

        // Direct equality or $eq
        let value = if condition.is_object() {
            if let Some(eq_val) = condition.get("$eq") {
                Some(eq_val)
            } else {
                None
            }
        } else {
            Some(condition)
        };

        if let Some(val) = value {
            if let Some(idx) = store_indices.single.get(field) {
                // If we have an index, estimate selectivity
                let key = crate::utils::value_key(Some(val));
                let count = idx.get(&key).map(|v| v.len()).unwrap_or(0);

                if count < min_entries {
                    min_entries = count;
                    best_field = Some(field.clone());
                    best_value = Some(val.clone());
                }
            }
        }
    }

    if let (Some(field), Some(value)) = (best_field, best_value) {
        let mut remaining = cond_obj.clone();
        remaining.remove(&field);

        ExecutionPlan::IndexedScan {
            collection: collection.to_string(),
            field,
            value,
            remaining_conditions: remaining,
        }
    } else {
        ExecutionPlan::FullScan
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::inner::StoreInner;
    use serde_json::json;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_optimize_full_scan() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.json");
        let path_str = path.to_str().unwrap().to_string();
        fs::write(&path, "{}").unwrap();
        let inner = StoreInner::new(path_str).unwrap();

        // Empty conditions object
        let plan = optimize_query(&inner, "users", &json!({}));
        assert!(matches!(plan, ExecutionPlan::FullScan));

        // Non-object conditions
        let plan = optimize_query(&inner, "users", &json!(1));
        assert!(matches!(plan, ExecutionPlan::FullScan));
    }

    #[test]
    fn test_optimize_indexed_scan() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.json");
        let path_str = path.to_str().unwrap().to_string();
        fs::write(
            &path,
            json!({"users": [
                {"name": "Alice", "age": 30},
                {"name": "Bob", "age": 25}
            ]})
            .to_string(),
        )
        .unwrap();
        let inner = StoreInner::new(path_str).unwrap();

        // Build index
        inner.build_index("users", "name").unwrap();

        // Query with indexed field
        let query = json!({"name": "Alice", "age": 30});
        let plan = optimize_query(&inner, "users", &query);

        if let ExecutionPlan::IndexedScan {
            field,
            value,
            remaining_conditions,
            ..
        } = plan
        {
            assert_eq!(field, "name");
            assert_eq!(value, json!("Alice"));
            assert_eq!(remaining_conditions.len(), 1);
            assert!(remaining_conditions.contains_key("age"));
        } else {
            panic!("Expected IndexedScan");
        }
    }
}
