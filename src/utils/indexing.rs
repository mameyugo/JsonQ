//! Value key generation for indexing

use serde_json::Value;

/// Generate string key from a JSON value for indexing
///
/// Converts any JSON value to a string suitable for use as a hash map key.
/// This is used internally for building indexes.
///
/// # Conversion Rules
///
/// - `null` → `"null"`
/// - `true` → `"true"`, `false` → `"false"`
/// - Numbers → their string representation
/// - Strings → the string itself
/// - Arrays/Objects → their JSON representation
///
/// # Examples
///
/// ```rust
/// use jsonq::utils::value_key;
/// use serde_json::json;
///
/// assert_eq!(value_key(Some(&json!("Alice"))), "Alice");
/// assert_eq!(value_key(Some(&json!(42))), "42");
/// assert_eq!(value_key(Some(&json!(true))), "true");
/// assert_eq!(value_key(Some(&json!(null))), "null");
/// assert_eq!(value_key(None), "null");
/// ```
///
/// # Array/Object Handling
///
/// ```rust
/// use jsonq::utils::value_key;
/// use serde_json::json;
///
/// let arr_key = value_key(Some(&json!([1, 2, 3])));
/// assert_eq!(arr_key, "[1,2,3]");
///
/// let obj_key = value_key(Some(&json!({"key": "value"})));
/// assert!(obj_key.contains("key"));
/// ```
pub fn value_key(value: Option<&Value>) -> String {
    match value {
        None => "null".to_string(),
        Some(Value::String(s)) => s.clone(),
        Some(v) => v.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_value_key_none() {
        assert_eq!(value_key(None), "null");
    }

    #[test]
    fn test_value_key_null() {
        assert_eq!(value_key(Some(&json!(null))), "null");
    }

    #[test]
    fn test_value_key_bool() {
        assert_eq!(value_key(Some(&json!(true))), "true");
        assert_eq!(value_key(Some(&json!(false))), "false");
    }

    #[test]
    fn test_value_key_integer() {
        assert_eq!(value_key(Some(&json!(0))), "0");
        assert_eq!(value_key(Some(&json!(42))), "42");
        assert_eq!(value_key(Some(&json!(-17))), "-17");
    }

    #[test]
    fn test_value_key_float() {
        assert_eq!(value_key(Some(&json!(3.14))), "3.14");
        assert_eq!(value_key(Some(&json!(0.0))), "0.0");
    }

    #[test]
    fn test_value_key_string() {
        assert_eq!(value_key(Some(&json!("hello"))), "hello");
        assert_eq!(value_key(Some(&json!(""))), "");
        assert_eq!(value_key(Some(&json!("Alice"))), "Alice");
    }

    #[test]
    fn test_value_key_array() {
        let key = value_key(Some(&json!([1, 2, 3])));
        assert_eq!(key, "[1,2,3]");
    }

    #[test]
    fn test_value_key_object() {
        let key = value_key(Some(&json!({"name": "Alice"})));
        assert!(key.contains("name"));
        assert!(key.contains("Alice"));
    }

    #[test]
    fn test_value_key_empty_array() {
        assert_eq!(value_key(Some(&json!([]))), "[]");
    }

    #[test]
    fn test_value_key_empty_object() {
        assert_eq!(value_key(Some(&json!({}))), "{}");
    }

    #[test]
    fn test_value_key_consistent() {
        // Same value should produce same key
        let val = json!("test");
        assert_eq!(value_key(Some(&val)), value_key(Some(&val)));
    }
}
