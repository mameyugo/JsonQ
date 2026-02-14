//! Main validation function

use serde_json::{json, Value};
use super::types::check_type;
use super::constraints::*;

/// Validate a JSON value against a schema
///
/// Returns a vector of validation errors. Empty vector means validation passed.
///
/// # Examples
///
/// ## Simple Type Validation
///
/// ```rust
/// use jsonq::validation::validate;
/// use serde_json::json;
///
/// let value = json!("hello");
/// let schema = json!({"type": "string"});
/// let errors = validate(&value, &schema, "field");
/// assert!(errors.is_empty());
/// ```
///
/// ## Object Validation with Constraints
///
/// ```rust
/// use jsonq::validation::validate;
/// use serde_json::json;
///
/// let value = json!({
///     "name": "Alice",
///     "age": 30,
///     "email": "alice@example.com"
/// });
///
/// let schema = json!({
///     "type": "object",
///     "required": ["name", "age"],
///     "properties": {
///         "name": {"type": "string", "minLength": 1},
///         "age": {"type": "integer", "min": 0, "max": 150},
///         "email": {"type": "string", "format": "email"}
///     }
/// });
///
/// let errors = validate(&value, &schema, "user");
/// assert!(errors.is_empty());
/// ```
///
/// ## Array Validation
///
/// ```rust
/// use jsonq::validation::validate;
/// use serde_json::json;
///
/// let value = json!([1, 2, 3]);
/// let schema = json!({
///     "type": "array",
///     "items": {"type": "integer", "min": 0}
/// });
///
/// let errors = validate(&value, &schema, "numbers");
/// assert!(errors.is_empty());
/// ```
/// Public validation API
pub fn validate(value: &Value, schema: &Value, path: &str) -> Vec<Value> {
    validate_with_depth(value, schema, path, 0)
}

/// Internal validation with depth tracking
fn validate_with_depth(value: &Value, schema: &Value, path: &str, depth: usize) -> Vec<Value> {
    let config = crate::config::Config::get();
    let mut errors = Vec::new();

    // ✅ PROTECTION: Check depth limit
    if depth > config.max_validation_depth {
        errors.push(json!({
            "path": path,
            "error": format!(
                "Validation depth {} exceeds maximum allowed {}",
                depth,
                config.max_validation_depth
            )
        }));
        return errors;
    }
    
    if let Some(schema_obj) = schema.as_object() {
        // Type validation
        if let Some(expected_type) = schema_obj.get("type").and_then(|v| v.as_str()) {
            if !check_type(value, expected_type) {
                errors.push(json!({
                    "path": path,
                    "error": format!("Expected {}, found {:?}", expected_type, value)
                }));
                return errors; // Stop validation if type is wrong
            }
        }
        
        // Numeric constraints
        if value.is_number() {
            for error_msg in validate_number_constraints(value, schema) {
                errors.push(json!({
                    "path": path,
                    "error": error_msg
                }));
            }
        }
        
        // String constraints
        if value.is_string() {
            for error_msg in validate_string_constraints(value, schema) {
                errors.push(json!({
                    "path": path,
                    "error": error_msg
                }));
            }
        }
        
        // Enum validation
        if let Some(enum_values) = schema_obj.get("enum") {
            if let Some(error_msg) = validate_enum(value, enum_values) {
                errors.push(json!({
                    "path": path,
                    "error": error_msg
                }));
            }
        }
        
        // Required fields validation (for objects)
        if let Some(required) = schema_obj.get("required") {
            for error_msg in validate_required_fields(value, required) {
                errors.push(json!({
                    "path": path,
                    "error": error_msg
                }));
            }
        }
        
        // Nested properties validation
        if let (Some(Value::Object(props)), Some(obj)) = 
            (schema_obj.get("properties"), value.as_object()) 
        {
            for (prop_name, prop_schema) in props {
                let prop_path = if path.is_empty() {
                    prop_name.clone()
                } else {
                    format!("{}.{}", path, prop_name)
                };
                
                let prop_value = obj.get(prop_name).unwrap_or(&Value::Null);
                errors.extend(validate_with_depth(prop_value, prop_schema, &prop_path, depth + 1));
            }
        }
        
        // Array items validation
        if let (Some(items_schema), Some(arr)) = 
            (schema_obj.get("items"), value.as_array()) 
        {
            for (index, item) in arr.iter().enumerate() {
                let item_path = format!("{}.{}", path, index);
                errors.extend(validate_with_depth(item, items_schema, &item_path, depth + 1));
            }
        }
    }
    
    errors
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_validate_simple_string() {
        let value = json!("hello");
        let schema = json!({"type": "string"});
        let errors = validate(&value, &schema, "field");
        assert!(errors.is_empty());
    }

    #[test]
    fn test_validate_type_mismatch() {
        let value = json!(42);
        let schema = json!({"type": "string"});
        let errors = validate(&value, &schema, "field");
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_validate_number_range() {
        let value = json!(50);
        let schema = json!({"type": "integer", "min": 0, "max": 100});
        let errors = validate(&value, &schema, "age");
        assert!(errors.is_empty());
    }

    #[test]
    fn test_validate_number_out_of_range() {
        let value = json!(150);
        let schema = json!({"type": "integer", "min": 0, "max": 100});
        let errors = validate(&value, &schema, "age");
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_validate_object_with_properties() {
        let value = json!({"name": "Alice", "age": 30});
        let schema = json!({
            "type": "object",
            "properties": {
                "name": {"type": "string"},
                "age": {"type": "integer"}
            }
        });
        let errors = validate(&value, &schema, "user");
        assert!(errors.is_empty());
    }

    #[test]
    fn test_validate_required_fields() {
        let value = json!({"name": "Alice"});
        let schema = json!({
            "type": "object",
            "required": ["name", "email"]
        });
        let errors = validate(&value, &schema, "user");
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_validate_array_items() {
        let value = json!([1, 2, 3]);
        let schema = json!({
            "type": "array",
            "items": {"type": "integer", "min": 0}
        });
        let errors = validate(&value, &schema, "numbers");
        assert!(errors.is_empty());
    }

    #[test]
    fn test_validate_array_items_invalid() {
        let value = json!([1, -5, 3]);
        let schema = json!({
            "type": "array",
            "items": {"type": "integer", "min": 0}
        });
        let errors = validate(&value, &schema, "numbers");
        assert_eq!(errors.len(), 1); // -5 violates min: 0
    }

    #[test]
    fn test_validate_nested_object() {
        let value = json!({
            "user": {
                "profile": {
                    "name": "Alice"
                }
            }
        });
        let schema = json!({
            "type": "object",
            "properties": {
                "user": {
                    "type": "object",
                    "properties": {
                        "profile": {
                            "type": "object",
                            "properties": {
                                "name": {"type": "string"}
                            }
                        }
                    }
                }
            }
        });
        let errors = validate(&value, &schema, "data");
        assert!(errors.is_empty());
    }

    #[test]
    fn test_validate_enum() {
        let value = json!("active");
        let schema = json!({
            "type": "string",
            "enum": ["active", "pending", "closed"]
        });
        let errors = validate(&value, &schema, "status");
        assert!(errors.is_empty());
    }

    #[test]
    fn test_validate_enum_invalid() {
        let value = json!("unknown");
        let schema = json!({
            "type": "string",
            "enum": ["active", "pending", "closed"]
        });
        let errors = validate(&value, &schema, "status");
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_validate_email_format() {
        let value = json!("user@example.com");
        let schema = json!({"type": "string", "format": "email"});
        let errors = validate(&value, &schema, "email");
        assert!(errors.is_empty());
    }

    #[test]
    fn test_validate_complex_schema() {
        let value = json!({
            "name": "Alice",
            "age": 30,
            "email": "alice@example.com",
            "tags": ["admin", "user"]
        });
        
        let schema = json!({
            "type": "object",
            "required": ["name", "age", "email"],
            "properties": {
                "name": {"type": "string", "minLength": 1},
                "age": {"type": "integer", "min": 0, "max": 150},
                "email": {"type": "string", "format": "email"},
                "tags": {
                    "type": "array",
                    "items": {"type": "string"}
                }
            }
        });
        
        let errors = validate(&value, &schema, "user");
        assert!(errors.is_empty());
    }
}
