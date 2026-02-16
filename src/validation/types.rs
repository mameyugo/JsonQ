//! Type checking for JSON values

use serde_json::Value;

/// Check if a value matches the expected type
///
/// # Supported Types
///
/// - `"string"` - JSON string
/// - `"number"` - Any JSON number (integer or float)
/// - `"integer"` - JSON number that is a whole number
/// - `"boolean"` - JSON boolean (true/false)
/// - `"array"` - JSON array
/// - `"object"` - JSON object
/// - `"null"` - JSON null
///
/// # Examples
///
/// ```rust
/// use jsonq::validation::check_type;
/// use serde_json::json;
///
/// assert!(check_type(&json!("hello"), "string"));
/// assert!(check_type(&json!(42), "number"));
/// assert!(check_type(&json!(42), "integer"));
/// assert!(!check_type(&json!(3.14), "integer"));
/// assert!(check_type(&json!(true), "boolean"));
/// ```
pub fn check_type(value: &Value, expected_type: &str) -> bool {
    match expected_type {
        "string" => value.is_string(),
        "number" => value.is_number(),
        "integer" => {
            // Must be a number with no fractional part
            value.is_number() && value.as_f64().map(|f| f.fract() == 0.0).unwrap_or(false)
        }
        "boolean" => value.is_boolean(),
        "array" => value.is_array(),
        "object" => value.is_object(),
        "null" => value.is_null(),
        _ => true, // Unknown types always pass
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_check_type_string() {
        assert!(check_type(&json!("hello"), "string"));
        assert!(check_type(&json!(""), "string"));
        assert!(!check_type(&json!(42), "string"));
    }

    #[test]
    fn test_check_type_number() {
        assert!(check_type(&json!(42), "number"));
        assert!(check_type(&json!(3.14), "number"));
        assert!(check_type(&json!(-17), "number"));
        assert!(!check_type(&json!("42"), "number"));
    }

    #[test]
    fn test_check_type_integer() {
        assert!(check_type(&json!(42), "integer"));
        assert!(check_type(&json!(0), "integer"));
        assert!(check_type(&json!(-5), "integer"));
        assert!(!check_type(&json!(3.14), "integer"));
        assert!(check_type(&json!(3.0), "integer"));
    }

    #[test]
    fn test_check_type_boolean() {
        assert!(check_type(&json!(true), "boolean"));
        assert!(check_type(&json!(false), "boolean"));
        assert!(!check_type(&json!(1), "boolean"));
    }

    #[test]
    fn test_check_type_array() {
        assert!(check_type(&json!([]), "array"));
        assert!(check_type(&json!([1, 2, 3]), "array"));
        assert!(!check_type(&json!({}), "array"));
    }

    #[test]
    fn test_check_type_object() {
        assert!(check_type(&json!({}), "object"));
        assert!(check_type(&json!({"key": "value"}), "object"));
        assert!(!check_type(&json!([]), "object"));
    }

    #[test]
    fn test_check_type_null() {
        assert!(check_type(&json!(null), "null"));
        assert!(!check_type(&json!(0), "null"));
        assert!(!check_type(&json!("null"), "null"));
    }

    #[test]
    fn test_check_type_unknown() {
        // Unknown types always pass
        assert!(check_type(&json!(42), "unknown"));
        assert!(check_type(&json!("test"), "custom-type"));
    }
}
