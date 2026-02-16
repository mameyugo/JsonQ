//! Query operators for MongoDB-style matching

use serde_json::Value;

/// Apply a comparison operator
///
/// # Supported Operators
///
/// - `$eq` - Equal
/// - `$ne` - Not equal
/// - `$gt` - Greater than
/// - `$gte` - Greater than or equal
/// - `$lt` - Less than
/// - `$lte` - Less than or equal
/// - `$in` - Value in array
/// - `$nin` - Value not in array
/// - `$exists` - Field exists
/// - `$regex` - Regular expression (substring match)
/// - `$contains` - String contains substring
/// - `$startsWith` - String starts with prefix
/// - `$endsWith` - String ends with suffix
///
/// # Examples
///
/// ```rust
/// use jsonq::query::apply_operator;
/// use serde_json::json;
///
/// // Numeric comparison
/// assert!(apply_operator(Some(&json!(30)), "$gte", &json!(18)));
/// assert!(!apply_operator(Some(&json!(15)), "$gte", &json!(18)));
///
/// // String operators
/// assert!(apply_operator(Some(&json!("hello")), "$contains", &json!("ell")));
/// assert!(apply_operator(Some(&json!("test@example.com")), "$endsWith", &json!(".com")));
/// ```
pub fn apply_operator(item_value: Option<&Value>, operator: &str, expected: &Value) -> bool {
    match operator {
        "$eq" => item_value == Some(expected),
        "$ne" => item_value != Some(expected),
        
        "$gt" => {
            item_value.and_then(|v| v.as_f64())
                > expected.as_f64()
        }
        
        "$gte" => {
            item_value.and_then(|v| v.as_f64())
                >= expected.as_f64()
        }
        
        "$lt" => {
            item_value.and_then(|v| v.as_f64())
                < expected.as_f64()
        }
        
        "$lte" => {
            item_value.and_then(|v| v.as_f64())
                <= expected.as_f64()
        }
        
        "$in" => {
            expected.as_array()
                .map(|arr| arr.contains(item_value.unwrap_or(&Value::Null)))
                .unwrap_or(false)
        }
        
        "$nin" => {
            !expected.as_array()
                .map(|arr| arr.contains(item_value.unwrap_or(&Value::Null)))
                .unwrap_or(true)
        }
        
        "$exists" => {
            expected.as_bool() == Some(item_value.is_some())
        }
        
        "$regex" => {
            let pattern = expected.as_str().unwrap_or("");
            item_value
                .and_then(|v| v.as_str())
                .map(|s| crate::query::regex_safe::is_match(s, pattern))
                .unwrap_or(false)
        }
        
        "$contains" => {
            let pattern = expected.as_str().unwrap_or("");
            item_value
                .and_then(|v| v.as_str())
                .map(|s| s.contains(pattern))
                .unwrap_or(false)
        }
        
        "$startsWith" => {
            let prefix = expected.as_str().unwrap_or("");
            item_value
                .and_then(|v| v.as_str())
                .map(|s| s.starts_with(prefix))
                .unwrap_or(false)
        }
        
        "$endsWith" => {
            let suffix = expected.as_str().unwrap_or("");
            item_value
                .and_then(|v| v.as_str())
                .map(|s| s.ends_with(suffix))
                .unwrap_or(false)
        }
        
        _ => false, // Unknown operator
    }
}

/// Check logical operators ($or, $and, $nor, $not)
///
/// # Examples
///
/// ```rust
/// use jsonq::query::check_logical_operator;
/// use serde_json::json;
///
/// let item = json!({"age": 30, "role": "admin"});
///
/// // $or
/// let or_cond = json!([
///     {"age": {"$lt": 18}},
///     {"role": "admin"}
/// ]);
/// assert!(check_logical_operator(&item, "$or", &or_cond, |i, c| true));
/// ```
pub fn check_logical_operator<F>(
    item: &Value,
    operator: &str,
    conditions: &Value,
    matcher: F,
) -> bool
where
    F: Fn(&Value, &Value) -> bool,
{
    match operator {
        "$or" => {
            conditions.as_array()
                .map(|arr| arr.iter().any(|cond| matcher(item, cond)))
                .unwrap_or(false)
        }
        
        "$and" => {
            conditions.as_array()
                .map(|arr| arr.iter().all(|cond| matcher(item, cond)))
                .unwrap_or(false)
        }
        
        "$nor" => {
            !conditions.as_array()
                .map(|arr| arr.iter().any(|cond| matcher(item, cond)))
                .unwrap_or(true)
        }
        
        "$not" => {
            !matcher(item, conditions)
        }
        
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // Comparison operators
    #[test]
    fn test_eq_operator() {
        assert!(apply_operator(Some(&json!(42)), "$eq", &json!(42)));
        assert!(!apply_operator(Some(&json!(42)), "$eq", &json!(43)));
    }

    #[test]
    fn test_ne_operator() {
        assert!(apply_operator(Some(&json!(42)), "$ne", &json!(43)));
        assert!(!apply_operator(Some(&json!(42)), "$ne", &json!(42)));
    }

    #[test]
    fn test_gt_operator() {
        assert!(apply_operator(Some(&json!(50)), "$gt", &json!(30)));
        assert!(!apply_operator(Some(&json!(30)), "$gt", &json!(50)));
        assert!(!apply_operator(Some(&json!(30)), "$gt", &json!(30)));
    }

    #[test]
    fn test_gte_operator() {
        assert!(apply_operator(Some(&json!(50)), "$gte", &json!(30)));
        assert!(apply_operator(Some(&json!(30)), "$gte", &json!(30)));
        assert!(!apply_operator(Some(&json!(20)), "$gte", &json!(30)));
    }

    #[test]
    fn test_lt_operator() {
        assert!(apply_operator(Some(&json!(20)), "$lt", &json!(30)));
        assert!(!apply_operator(Some(&json!(30)), "$lt", &json!(20)));
    }

    #[test]
    fn test_lte_operator() {
        assert!(apply_operator(Some(&json!(20)), "$lte", &json!(30)));
        assert!(apply_operator(Some(&json!(30)), "$lte", &json!(30)));
        assert!(!apply_operator(Some(&json!(40)), "$lte", &json!(30)));
    }

    // Array operators
    #[test]
    fn test_in_operator() {
        let values = json!(["admin", "user", "viewer"]);
        assert!(apply_operator(Some(&json!("admin")), "$in", &values));
        assert!(!apply_operator(Some(&json!("guest")), "$in", &values));
    }

    #[test]
    fn test_nin_operator() {
        let values = json!(["admin", "user"]);
        assert!(apply_operator(Some(&json!("guest")), "$nin", &values));
        assert!(!apply_operator(Some(&json!("admin")), "$nin", &values));
    }

    // Existence operator
    #[test]
    fn test_exists_operator() {
        assert!(apply_operator(Some(&json!(42)), "$exists", &json!(true)));
        assert!(apply_operator(None, "$exists", &json!(false)));
        assert!(!apply_operator(Some(&json!(42)), "$exists", &json!(false)));
        assert!(!apply_operator(None, "$exists", &json!(true)));
    }

    // String operators
    #[test]
    fn test_contains_operator() {
        assert!(apply_operator(Some(&json!("hello world")), "$contains", &json!("world")));
        assert!(!apply_operator(Some(&json!("hello")), "$contains", &json!("world")));
    }

    #[test]
    fn test_regex_operator() {
        assert!(apply_operator(Some(&json!("test@example.com")), "$regex", &json!("@")));
        assert!(!apply_operator(Some(&json!("invalid")), "$regex", &json!("@")));
    }

    #[test]
    fn test_starts_with_operator() {
        assert!(apply_operator(Some(&json!("hello world")), "$startsWith", &json!("hello")));
        assert!(!apply_operator(Some(&json!("world hello")), "$startsWith", &json!("hello")));
    }

    #[test]
    fn test_ends_with_operator() {
        assert!(apply_operator(Some(&json!("hello world")), "$endsWith", &json!("world")));
        assert!(!apply_operator(Some(&json!("world hello")), "$endsWith", &json!("world")));
    }

    #[test]
    fn test_unknown_operator() {
        assert!(!apply_operator(Some(&json!(42)), "$unknown", &json!(42)));
    }
}
