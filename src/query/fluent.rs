//! Fluent query execution engine

use serde_json::{Map, Value};
use crate::path::read_path;
use crate::utils::as_u64;

/// Execute a fluent query on a collection
///
/// # Query Format
///
/// ```json
/// {
///   "where": [                    // Filter conditions
///     {"field": "age", "op": ">=", "value": 18}
///   ],
///   "order_by": {                 // Sorting
///     "field": "age",
///     "direction": "desc"
///   },
///   "skip": 10,                   // Pagination offset
///   "limit": 20,                  // Max results
///   "select": ["name", "age"]     // Field projection
/// }
/// ```
///
/// # Examples
///
/// ## Simple Filter
///
/// ```rust
/// use jsonq::query::execute_query;
/// use serde_json::json;
///
/// let collection = vec![
///     json!({"name": "Alice", "age": 30}),
///     json!({"name": "Bob", "age": 25}),
///     json!({"name": "Charlie", "age": 35}),
/// ];
///
/// let query = json!({
///     "where": [{"field": "age", "op": ">=", "value": 30}]
/// });
///
/// let results = execute_query(&collection, &query);
/// assert_eq!(results.len(), 2); // Alice and Charlie
/// ```
///
/// ## Sort and Limit
///
/// ```rust
/// use jsonq::query::execute_query;
/// use serde_json::json;
///
/// let collection = vec![
///     json!({"name": "Alice", "age": 30}),
///     json!({"name": "Bob", "age": 25}),
///     json!({"name": "Charlie", "age": 35}),
/// ];
///
/// let query = json!({
///     "order_by": {"field": "age", "direction": "desc"},
///     "limit": 2
/// });
///
/// let results = execute_query(&collection, &query);
/// assert_eq!(results[0]["name"], "Charlie"); // Oldest first
/// ```
///
/// ## Field Projection
///
/// ```rust
/// use jsonq::query::execute_query;
/// use serde_json::json;
///
/// let collection = vec![
///     json!({"name": "Alice", "age": 30, "email": "alice@example.com"}),
/// ];
///
/// let query = json!({
///     "select": ["name", "age"]
/// });
///
/// let results = execute_query(&collection, &query);
/// assert!(results[0].get("email").is_none()); // Email excluded
/// ```
pub fn execute_query(collection: &[Value], query: &Value) -> Vec<Value> {
    let mut results = collection.to_vec();
    
    // 1. Filter (where conditions)
    if let Some(where_array) = query.get("where").and_then(|w| w.as_array()) {
        results = results.into_iter().filter(|item| {
            where_array.iter().all(|condition| check_condition(item, condition))
        }).collect();
    }
    
    // 2. Sort (order_by)
    if let Some(order_by) = query.get("order_by").or_else(|| query.get("sort")) {
        sort_results(&mut results, order_by);
    }
    
    // 3. Skip (offset/pagination)
    if let Some(skip) = query.get("skip").or_else(|| query.get("offset")).and_then(as_u64) {
        if (skip as usize) < results.len() {
            results = results.split_off(skip as usize);
        } else {
            results.clear();
        }
    }
    
    // 4. Limit (max results)
    if let Some(limit) = query.get("limit").and_then(as_u64) {
        results.truncate(limit as usize);
    }
    
    // 5. Select (field projection)
    if let Some(select) = query.get("select").and_then(|s| s.as_array()) {
        let fields: Vec<&str> = select.iter().filter_map(|v| v.as_str()).collect();
        results = project_fields(&results, &fields);
    }
    
    results
}

/// Check if an item matches a where condition
fn check_condition(item: &Value, condition: &Value) -> bool {
    let field = condition.get("field").and_then(|v| v.as_str()).unwrap_or("");
    let operator = condition.get("op").and_then(|v| v.as_str()).unwrap_or("=");
    let expected = condition.get("value").unwrap_or(&Value::Null);
    
    let item_value = read_path(item, field);
    
    match operator {
        "=" | "eq" => item_value == Some(expected),
        "!=" | "ne" => item_value != Some(expected),
        ">" | "gt" => {
            item_value.and_then(|v| v.as_f64()) > expected.as_f64()
        }
        ">=" | "gte" => {
            item_value.and_then(|v| v.as_f64()) >= expected.as_f64()
        }
        "<" | "lt" => {
            item_value.and_then(|v| v.as_f64()) < expected.as_f64()
        }
        "<=" | "lte" => {
            item_value.and_then(|v| v.as_f64()) <= expected.as_f64()
        }
        "in" => {
            expected.as_array()
                .map(|arr| arr.contains(item_value.unwrap_or(&Value::Null)))
                .unwrap_or(false)
        }
        "contains" => {
            let substring = expected.as_str().unwrap_or("");
            item_value
                .and_then(|v| v.as_str())
                .map(|s| s.contains(substring))
                .unwrap_or(false)
        }
        "startsWith" => {
            let prefix = expected.as_str().unwrap_or("");
            item_value
                .and_then(|v| v.as_str())
                .map(|s| s.starts_with(prefix))
                .unwrap_or(false)
        }
        "endsWith" => {
            let suffix = expected.as_str().unwrap_or("");
            item_value
                .and_then(|v| v.as_str())
                .map(|s| s.ends_with(suffix))
                .unwrap_or(false)
        }
        "between" => {
            if let Some(arr) = expected.as_array() {
                if arr.len() == 2 {
                    let val = item_value.and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let min = arr[0].as_f64().unwrap_or(0.0);
                    let max = arr[1].as_f64().unwrap_or(0.0);
                    return val >= min && val <= max;
                }
            }
            false
        }
        _ => false,
    }
}

/// Sort results by field and direction
fn sort_results(results: &mut [Value], order_by: &Value) {
    let field = order_by.get("field").and_then(|v| v.as_str()).unwrap_or("");
    let direction = order_by.get("direction").and_then(|v| v.as_str()).unwrap_or("asc");
    let desc = direction == "desc" || order_by.get("desc").and_then(|v| v.as_bool()).unwrap_or(false);
    
    results.sort_by(|a, b| {
        let val_a = read_path(a, field);
        let val_b = read_path(b, field);
        
        let cmp = match (val_a, val_b) {
            (Some(Value::Number(x)), Some(Value::Number(y))) => {
                x.as_f64().partial_cmp(&y.as_f64()).unwrap_or(std::cmp::Ordering::Equal)
            }
            (Some(Value::String(x)), Some(Value::String(y))) => {
                x.cmp(y)
            }
            _ => std::cmp::Ordering::Equal,
        };
        
        if desc { cmp.reverse() } else { cmp }
    });
}

/// Project specific fields from results
fn project_fields(results: &[Value], fields: &[&str]) -> Vec<Value> {
    if fields.len() == 1 {
        // Single field projection returns array of values
        let field = fields[0];
        return results.iter()
            .map(|item| read_path(item, field).cloned().unwrap_or(Value::Null))
            .collect();
    }
    
    // Multiple fields projection returns array of objects
    results.iter().map(|item| {
        let mut obj = Map::new();
        for &field in fields {
            if let Some(value) = read_path(item, field) {
                obj.insert(field.to_string(), value.clone());
            }
        }
        Value::Object(obj)
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn sample_collection() -> Vec<Value> {
        vec![
            json!({"name": "Alice", "age": 30, "city": "NYC", "score": 95}),
            json!({"name": "Bob", "age": 25, "city": "LA", "score": 82}),
            json!({"name": "Charlie", "age": 35, "city": "NYC", "score": 88}),
            json!({"name": "Diana", "age": 28, "city": "Chicago", "score": 91}),
            json!({"name": "Eve", "age": 22, "city": "LA", "score": 76}),
        ]
    }

    // Filter tests
    #[test]
    fn test_execute_query_where_simple() {
        let collection = sample_collection();
        let query = json!({
            "where": [{"field": "city", "op": "=", "value": "NYC"}]
        });
        
        let results = execute_query(&collection, &query);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_execute_query_where_multiple() {
        let collection = sample_collection();
        let query = json!({
            "where": [
                {"field": "age", "op": ">=", "value": 25},
                {"field": "score", "op": ">", "value": 85}
            ]
        });
        
        let results = execute_query(&collection, &query);
        assert_eq!(results.len(), 3); // Alice, Charlie, Diana
    }

    #[test]
    fn test_execute_query_where_between() {
        let collection = sample_collection();
        let query = json!({
            "where": [{"field": "age", "op": "between", "value": [25, 30]}]
        });
        
        let results = execute_query(&collection, &query);
        assert_eq!(results.len(), 3); // Bob, Diana, Alice
    }

    // Sort tests
    #[test]
    fn test_execute_query_sort_asc() {
        let collection = sample_collection();
        let query = json!({
            "order_by": {"field": "age", "direction": "asc"}
        });
        
        let results = execute_query(&collection, &query);
        assert_eq!(results[0]["name"], "Eve"); // Youngest
        assert_eq!(results[4]["name"], "Charlie"); // Oldest
    }

    #[test]
    fn test_execute_query_sort_desc() {
        let collection = sample_collection();
        let query = json!({
            "order_by": {"field": "age", "direction": "desc"}
        });
        
        let results = execute_query(&collection, &query);
        assert_eq!(results[0]["name"], "Charlie"); // Oldest
        assert_eq!(results[4]["name"], "Eve"); // Youngest
    }

    // Pagination tests
    #[test]
    fn test_execute_query_limit() {
        let collection = sample_collection();
        let query = json!({"limit": 2});
        
        let results = execute_query(&collection, &query);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_execute_query_skip() {
        let collection = sample_collection();
        let query = json!({"skip": 2});
        
        let results = execute_query(&collection, &query);
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_execute_query_skip_and_limit() {
        let collection = sample_collection();
        let query = json!({
            "skip": 1,
            "limit": 2
        });
        
        let results = execute_query(&collection, &query);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0]["name"], "Bob");
    }

    // Projection tests
    #[test]
    fn test_execute_query_select_single() {
        let collection = sample_collection();
        let query = json!({
            "select": ["name"]
        });
        
        let results = execute_query(&collection, &query);
        assert_eq!(results.len(), 5);
        assert_eq!(results[0], "Alice");
    }

    #[test]
    fn test_execute_query_select_multiple() {
        let collection = sample_collection();
        let query = json!({
            "select": ["name", "age"]
        });
        
        let results = execute_query(&collection, &query);
        assert_eq!(results.len(), 5);
        assert!(results[0].get("name").is_some());
        assert!(results[0].get("age").is_some());
        assert!(results[0].get("city").is_none());
    }

    // Complex queries
    #[test]
    fn test_execute_query_complex() {
        let collection = sample_collection();
        let query = json!({
            "where": [{"field": "age", "op": ">=", "value": 25}],
            "order_by": {"field": "score", "direction": "desc"},
            "limit": 2,
            "select": ["name", "score"]
        });
        
        let results = execute_query(&collection, &query);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0]["name"], "Alice"); // Highest score
        assert_eq!(results[1]["name"], "Diana"); // Second highest
    }

    #[test]
    fn test_execute_query_string_operators() {
        let collection = sample_collection();
        
        // Contains
        let query = json!({
            "where": [{"field": "name", "op": "contains", "value": "li"}]
        });
        let results = execute_query(&collection, &query);
        assert_eq!(results.len(), 2); // Alice, Charlie
        
        // Starts with
        let query = json!({
            "where": [{"field": "name", "op": "startsWith", "value": "A"}]
        });
        let results = execute_query(&collection, &query);
        assert_eq!(results.len(), 1); // Alice
    }

    #[test]
    fn test_execute_query_empty_collection() {
        let collection: Vec<Value> = vec![];
        let query = json!({"where": [{"field": "age", "op": ">", "value": 0}]});
        
        let results = execute_query(&collection, &query);
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_execute_query_no_filters() {
        let collection = sample_collection();
        let query = json!({});
        
        let results = execute_query(&collection, &query);
        assert_eq!(results.len(), 5);
    }
}
