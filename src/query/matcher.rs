//! MongoDB-style pattern matching

use serde_json::Value;
use crate::path::read_path;
use super::operators::{apply_operator, check_logical_operator};

/// Check if an item matches a MongoDB-style condition
///
/// # Condition Format
///
/// ```json
/// {
///   "field": value,              // Simple equality
///   "field": {"$op": value},     // Operator
///   "$or": [condition1, ...],    // Logical OR
///   "$and": [condition1, ...],   // Logical AND
///   "$not": condition            // Logical NOT
/// }
/// ```
///
/// # Examples
///
/// ## Simple Equality
///
/// ```rust
/// use jsonq::query::matches;
/// use serde_json::json;
///
/// let item = json!({"name": "Alice", "age": 30});
/// assert!(matches(&item, &json!({"name": "Alice"})));
/// assert!(!matches(&item, &json!({"name": "Bob"})));
/// ```
///
/// ## Comparison Operators
///
/// ```rust
/// use jsonq::query::matches;
/// use serde_json::json;
///
/// let item = json!({"age": 30});
/// assert!(matches(&item, &json!({"age": {"$gte": 18}})));
/// assert!(matches(&item, &json!({"age": {"$lt": 50}})));
/// ```
///
/// ## Logical Operators
///
/// ```rust
/// use jsonq::query::matches;
/// use serde_json::json;
///
/// let item = json!({"age": 30, "role": "admin"});
///
/// // OR condition
/// let cond = json!({"$or": [
///     {"age": {"$lt": 18}},
///     {"role": "admin"}
/// ]});
/// assert!(matches(&item, &cond));
///
/// // AND condition (implicit)
/// assert!(matches(&item, &json!({
///     "age": {"$gte": 18},
///     "role": "admin"
/// })));
/// ```
pub fn matches(item: &Value, condition: &Value) -> bool {
    let cond_obj = match condition.as_object() {
        Some(obj) => obj,
        None => return false,
    };
    
    // Check all conditions (implicit AND)
    cond_obj.iter().all(|(key, value)| {
        // Logical operators
        if key.starts_with('$') {
            return check_logical_operator(item, key, value, matches);
        }
        
        // Field-based condition
        let item_value = read_path(item, key);
        
        // Check if value is an operator object
        if let Some(op_obj) = value.as_object() {
            // All operators must match
            return op_obj.iter().all(|(operator, expected)| {
                apply_operator(item_value, operator, expected)
            });
        }
        
        // Simple equality
        item_value == Some(value)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn sample_item() -> Value {
        json!({
            "name": "Alice",
            "age": 30,
            "city": "NYC",
            "role": "admin",
            "score": 95
        })
    }

    // Simple equality
    #[test]
    fn test_matches_simple_equality() {
        let item = sample_item();
        assert!(matches(&item, &json!({"name": "Alice"})));
        assert!(matches(&item, &json!({"age": 30})));
        assert!(!matches(&item, &json!({"name": "Bob"})));
    }

    #[test]
    fn test_matches_multiple_fields() {
        let item = sample_item();
        assert!(matches(&item, &json!({
            "name": "Alice",
            "city": "NYC"
        })));
        assert!(!matches(&item, &json!({
            "name": "Alice",
            "city": "LA"
        })));
    }

    // Comparison operators
    #[test]
    fn test_matches_gt() {
        let item = sample_item();
        assert!(matches(&item, &json!({"age": {"$gt": 25}})));
        assert!(!matches(&item, &json!({"age": {"$gt": 50}})));
    }

    #[test]
    fn test_matches_gte() {
        let item = sample_item();
        assert!(matches(&item, &json!({"age": {"$gte": 30}})));
        assert!(matches(&item, &json!({"age": {"$gte": 25}})));
    }

    #[test]
    fn test_matches_lt() {
        let item = sample_item();
        assert!(matches(&item, &json!({"age": {"$lt": 50}})));
        assert!(!matches(&item, &json!({"age": {"$lt": 20}})));
    }

    #[test]
    fn test_matches_lte() {
        let item = sample_item();
        assert!(matches(&item, &json!({"age": {"$lte": 30}})));
        assert!(matches(&item, &json!({"age": {"$lte": 50}})));
    }

    #[test]
    fn test_matches_ne() {
        let item = sample_item();
        assert!(matches(&item, &json!({"role": {"$ne": "user"}})));
        assert!(!matches(&item, &json!({"role": {"$ne": "admin"}})));
    }

    // Array operators
    #[test]
    fn test_matches_in() {
        let item = sample_item();
        assert!(matches(&item, &json!({
            "role": {"$in": ["admin", "viewer"]}
        })));
        assert!(!matches(&item, &json!({
            "role": {"$in": ["user", "guest"]}
        })));
    }

    #[test]
    fn test_matches_nin() {
        let item = sample_item();
        assert!(matches(&item, &json!({
            "role": {"$nin": ["user", "guest"]}
        })));
        assert!(!matches(&item, &json!({
            "role": {"$nin": ["admin", "user"]}
        })));
    }

    // String operators
    #[test]
    fn test_matches_contains() {
        let item = sample_item();
        assert!(matches(&item, &json!({
            "name": {"$contains": "lic"}
        })));
        assert!(!matches(&item, &json!({
            "name": {"$contains": "Bob"}
        })));
    }

    #[test]
    fn test_matches_starts_with() {
        let item = sample_item();
        assert!(matches(&item, &json!({
            "name": {"$startsWith": "Ali"}
        })));
        assert!(!matches(&item, &json!({
            "name": {"$startsWith": "Bob"}
        })));
    }

    #[test]
    fn test_matches_ends_with() {
        let item = sample_item();
        assert!(matches(&item, &json!({
            "name": {"$endsWith": "ice"}
        })));
        assert!(!matches(&item, &json!({
            "name": {"$endsWith": "Bob"}
        })));
    }

    // Logical operators
    #[test]
    fn test_matches_or() {
        let item = sample_item();
        assert!(matches(&item, &json!({
            "$or": [
                {"age": {"$lt": 18}},
                {"role": "admin"}
            ]
        })));
        assert!(!matches(&item, &json!({
            "$or": [
                {"age": {"$lt": 18}},
                {"role": "user"}
            ]
        })));
    }

    #[test]
    fn test_matches_and() {
        let item = sample_item();
        assert!(matches(&item, &json!({
            "$and": [
                {"age": {"$gte": 18}},
                {"role": "admin"}
            ]
        })));
        assert!(!matches(&item, &json!({
            "$and": [
                {"age": {"$lt": 18}},
                {"role": "admin"}
            ]
        })));
    }

    #[test]
    fn test_matches_not() {
        let item = sample_item();
        assert!(matches(&item, &json!({
            "$not": {"role": "user"}
        })));
        assert!(!matches(&item, &json!({
            "$not": {"role": "admin"}
        })));
    }

    #[test]
    fn test_matches_nor() {
        let item = sample_item();
        assert!(matches(&item, &json!({
            "$nor": [
                {"age": {"$lt": 18}},
                {"role": "user"}
            ]
        })));
        assert!(!matches(&item, &json!({
            "$nor": [
                {"age": {"$gte": 18}},
                {"role": "user"}
            ]
        })));
    }

    // Complex conditions
    #[test]
    fn test_matches_complex() {
        let item = sample_item();
        assert!(matches(&item, &json!({
            "age": {"$gte": 18, "$lt": 65},
            "role": {"$in": ["admin", "user"]},
            "score": {"$gt": 80}
        })));
    }

    #[test]
    fn test_matches_nested_or() {
        let item = sample_item();
        assert!(matches(&item, &json!({
            "$or": [
                {
                    "$and": [
                        {"age": {"$lt": 18}},
                        {"role": "user"}
                    ]
                },
                {
                    "$and": [
                        {"age": {"$gte": 18}},
                        {"role": "admin"}
                    ]
                }
            ]
        })));
    }

    #[test]
    fn test_matches_nested_fields() {
        let item = json!({
            "user": {
                "profile": {
                    "age": 30
                }
            }
        });
        
        assert!(matches(&item, &json!({
            "user.profile.age": {"$gte": 18}
        })));
    }
}
