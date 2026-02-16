//! Constraint validation functions

use serde_json::Value;
use crate::utils::as_u64;

/// Validate numeric constraints (min/max)
///
/// # Examples
///
/// ```rust
/// use jsonq::validation::validate_number_constraints;
/// use serde_json::json;
///
/// let constraints = json!({"min": 0, "max": 100});
/// assert!(validate_number_constraints(&json!(50), &constraints).is_empty());
/// assert!(!validate_number_constraints(&json!(150), &constraints).is_empty());
/// ```
pub fn validate_number_constraints(value: &Value, schema: &Value) -> Vec<String> {
    let mut errors = Vec::new();
    
    if let Some(num) = value.as_f64() {
        // Check minimum
        if let Some(min) = schema.get("min")
            .or_else(|| schema.get("minimum"))
            .and_then(|v| v.as_f64())
        {
            if num < min {
                errors.push(format!("Value {} < minimum {}", num, min));
            }
        }
        
        // Check maximum
        if let Some(max) = schema.get("max")
            .or_else(|| schema.get("maximum"))
            .and_then(|v| v.as_f64())
        {
            if num > max {
                errors.push(format!("Value {} > maximum {}", num, max));
            }
        }
    }
    
    errors
}

/// Validate string constraints (minLength, maxLength, pattern, format)
///
/// # Examples
///
/// ```rust
/// use jsonq::validation::validate_string_constraints;
/// use serde_json::json;
///
/// let constraints = json!({"minLength": 3, "maxLength": 10});
/// assert!(validate_string_constraints(&json!("hello"), &constraints).is_empty());
/// assert!(!validate_string_constraints(&json!("hi"), &constraints).is_empty());
/// ```
pub fn validate_string_constraints(value: &Value, schema: &Value) -> Vec<String> {
    let mut errors = Vec::new();
    
    if let Some(s) = value.as_str() {
        let len = s.len() as u64;
        
        // Check minLength
        if let Some(min_len) = schema.get("minLength").and_then(as_u64) {
            if len < min_len {
                errors.push(format!("String length {} < minLength {}", len, min_len));
            }
        }
        
        // Check maxLength
        if let Some(max_len) = schema.get("maxLength").and_then(as_u64) {
            if len > max_len {
                errors.push(format!("String length {} > maxLength {}", len, max_len));
            }
        }
        
        // Check pattern (simple substring match)
        if let Some(pattern) = schema.get("pattern").and_then(|v| v.as_str()) {
            if !s.contains(pattern) {
                errors.push(format!("Value '{}' does not match pattern '{}'", s, pattern));
            }
        }
        
        // Check format
        if let Some(format) = schema.get("format").and_then(|v| v.as_str()) {
            match format {
                "email" => {
                    if !s.contains('@') || !s.contains('.') {
                        errors.push("Invalid email format".to_string());
                    }
                }
                _ => {} // Unknown formats ignored
            }
        }
    }
    
    errors
}

/// Validate enum constraint
///
/// # Examples
///
/// ```rust
/// use jsonq::validation::validate_enum;
/// use serde_json::json;
///
/// let allowed = json!(["active", "pending", "closed"]);
/// assert!(validate_enum(&json!("active"), &allowed).is_none());
/// assert!(validate_enum(&json!("invalid"), &allowed).is_some());
/// ```
pub fn validate_enum(value: &Value, allowed_values: &Value) -> Option<String> {
    if let Some(choices) = allowed_values.as_array() {
        if !choices.contains(value) {
            return Some(format!("Value {:?} not in enum {:?}", value, choices));
        }
    }
    None
}

/// Validate required fields in an object
///
/// # Examples
///
/// ```rust
/// use jsonq::validation::validate_required_fields;
/// use serde_json::json;
///
/// let obj = json!({"name": "Alice"});
/// let required = json!(["name", "email"]);
/// let errors = validate_required_fields(&obj, &required);
/// assert_eq!(errors.len(), 1); // Missing "email"
/// ```
pub fn validate_required_fields(value: &Value, required: &Value) -> Vec<String> {
    let mut errors = Vec::new();
    
    if let (Some(obj), Some(req_fields)) = (value.as_object(), required.as_array()) {
        for field in req_fields {
            if let Some(field_name) = field.as_str() {
                if !obj.contains_key(field_name) {
                    errors.push(format!("Required field '{}' missing", field_name));
                }
            }
        }
    }
    
    errors
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // Number constraints tests
    #[test]
    fn test_number_min_constraint() {
        let schema = json!({"min": 10});
        assert!(validate_number_constraints(&json!(15), &schema).is_empty());
        assert!(!validate_number_constraints(&json!(5), &schema).is_empty());
    }

    #[test]
    fn test_number_max_constraint() {
        let schema = json!({"max": 100});
        assert!(validate_number_constraints(&json!(50), &schema).is_empty());
        assert!(!validate_number_constraints(&json!(150), &schema).is_empty());
    }

    #[test]
    fn test_number_min_max_range() {
        let schema = json!({"min": 0, "max": 100});
        assert!(validate_number_constraints(&json!(50), &schema).is_empty());
        assert!(!validate_number_constraints(&json!(-10), &schema).is_empty());
        assert!(!validate_number_constraints(&json!(200), &schema).is_empty());
    }

    // String constraints tests
    #[test]
    fn test_string_min_length() {
        let schema = json!({"minLength": 3});
        assert!(validate_string_constraints(&json!("hello"), &schema).is_empty());
        assert!(!validate_string_constraints(&json!("hi"), &schema).is_empty());
    }

    #[test]
    fn test_string_max_length() {
        let schema = json!({"maxLength": 5});
        assert!(validate_string_constraints(&json!("hi"), &schema).is_empty());
        assert!(!validate_string_constraints(&json!("hello world"), &schema).is_empty());
    }

    #[test]
    fn test_string_pattern() {
        let schema = json!({"pattern": "@"});
        assert!(validate_string_constraints(&json!("user@example.com"), &schema).is_empty());
        assert!(!validate_string_constraints(&json!("invalid"), &schema).is_empty());
    }

    #[test]
    fn test_string_email_format() {
        let schema = json!({"format": "email"});
        assert!(validate_string_constraints(&json!("user@example.com"), &schema).is_empty());
        assert!(!validate_string_constraints(&json!("invalid"), &schema).is_empty());
    }

    // Enum tests
    #[test]
    fn test_enum_valid() {
        let allowed = json!(["red", "green", "blue"]);
        assert!(validate_enum(&json!("red"), &allowed).is_none());
        assert!(validate_enum(&json!("green"), &allowed).is_none());
    }

    #[test]
    fn test_enum_invalid() {
        let allowed = json!(["red", "green", "blue"]);
        assert!(validate_enum(&json!("yellow"), &allowed).is_some());
    }

    // Required fields tests
    #[test]
    fn test_required_fields_all_present() {
        let obj = json!({"name": "Alice", "email": "alice@example.com"});
        let required = json!(["name", "email"]);
        assert!(validate_required_fields(&obj, &required).is_empty());
    }

    #[test]
    fn test_required_fields_missing() {
        let obj = json!({"name": "Alice"});
        let required = json!(["name", "email"]);
        let errors = validate_required_fields(&obj, &required);
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_required_fields_multiple_missing() {
        let obj = json!({});
        let required = json!(["name", "email", "age"]);
        let errors = validate_required_fields(&obj, &required);
        assert_eq!(errors.len(), 3);
    }
}
