//! Deep search in JSON structures

use serde_json::Value;

/// Search for a keyword anywhere in a JSON value (case-insensitive)
///
/// Recursively searches through strings, objects, and arrays looking for
/// the keyword. The search is case-insensitive.
///
/// # Search Behavior
///
/// - **Strings**: Checks if keyword is a substring (case-insensitive)
/// - **Objects**: Recursively searches all values
/// - **Arrays**: Recursively searches all elements
/// - **Other types**: Converts to string and checks for keyword
///
/// # Examples
///
/// ```rust
/// use jsonq::utils::search_in_value;
/// use serde_json::json;
///
/// let data = json!({
///     "user": {
///         "name": "Alice",
///         "email": "alice@example.com"
///     }
/// });
///
/// assert!(search_in_value(&data, "alice"));
/// assert!(search_in_value(&data, "ALICE")); // Case-insensitive
/// assert!(search_in_value(&data, "example.com"));
/// assert!(!search_in_value(&data, "bob"));
/// ```
///
/// # Nested Arrays
///
/// ```rust
/// use jsonq::utils::search_in_value;
/// use serde_json::json;
///
/// let data = json!({
///     "tags": ["rust", "programming", "web"]
/// });
///
/// assert!(search_in_value(&data, "rust"));
/// assert!(search_in_value(&data, "PROGRAMMING"));
/// ```
///
/// # Performance
///
/// This is a full tree traversal with O(n) complexity where n is the total
/// number of values in the structure. Use indexes for better performance
/// on large datasets.
pub fn search_in_value(value: &Value, keyword: &str) -> bool {
    let keyword_lower = keyword.to_lowercase();
    search_recursive(value, &keyword_lower)
}

fn search_recursive(value: &Value, keyword_lower: &str) -> bool {
    match value {
        Value::String(s) => s.to_lowercase().contains(keyword_lower),
        
        Value::Object(map) => {
            map.values().any(|v| search_recursive(v, keyword_lower))
        }
        
        Value::Array(arr) => {
            arr.iter().any(|v| search_recursive(v, keyword_lower))
        }
        
        // For other types (numbers, booleans, null), convert to string
        _ => value.to_string().to_lowercase().contains(keyword_lower),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_search_in_string() {
        let data = json!("Hello World");
        assert!(search_in_value(&data, "hello"));
        assert!(search_in_value(&data, "world"));
        assert!(search_in_value(&data, "HELLO")); // Case-insensitive
        assert!(!search_in_value(&data, "goodbye"));
    }

    #[test]
    fn test_search_in_number() {
        let data = json!(42);
        assert!(search_in_value(&data, "42"));
        assert!(search_in_value(&data, "4")); // Substring
        assert!(!search_in_value(&data, "43"));
    }

    #[test]
    fn test_search_in_boolean() {
        assert!(search_in_value(&json!(true), "true"));
        assert!(search_in_value(&json!(false), "false"));
        assert!(!search_in_value(&json!(true), "false"));
    }

    #[test]
    fn test_search_in_null() {
        assert!(search_in_value(&json!(null), "null"));
    }

    #[test]
    fn test_search_in_object() {
        let data = json!({
            "name": "Alice",
            "age": 30,
            "city": "NYC"
        });
        
        assert!(search_in_value(&data, "alice"));
        assert!(search_in_value(&data, "30"));
        assert!(search_in_value(&data, "nyc"));
        assert!(!search_in_value(&data, "bob"));
    }

    #[test]
    fn test_search_in_array() {
        let data = json!(["apple", "banana", "cherry"]);
        
        assert!(search_in_value(&data, "apple"));
        assert!(search_in_value(&data, "BANANA"));
        assert!(search_in_value(&data, "cherry"));
        assert!(!search_in_value(&data, "orange"));
    }

    #[test]
    fn test_search_in_nested_object() {
        let data = json!({
            "user": {
                "profile": {
                    "name": "Alice",
                    "bio": "Software Engineer"
                }
            }
        });
        
        assert!(search_in_value(&data, "alice"));
        assert!(search_in_value(&data, "software"));
        assert!(search_in_value(&data, "engineer"));
    }

    #[test]
    fn test_search_in_nested_array() {
        let data = json!({
            "users": [
                {"name": "Alice"},
                {"name": "Bob"},
                {"name": "Charlie"}
            ]
        });
        
        assert!(search_in_value(&data, "alice"));
        assert!(search_in_value(&data, "bob"));
        assert!(search_in_value(&data, "charlie"));
        assert!(!search_in_value(&data, "david"));
    }

    #[test]
    fn test_search_mixed_types() {
        let data = json!({
            "string": "hello",
            "number": 42,
            "bool": true,
            "null": null,
            "array": [1, 2, 3],
            "object": {"nested": "value"}
        });
        
        assert!(search_in_value(&data, "hello"));
        assert!(search_in_value(&data, "42"));
        assert!(search_in_value(&data, "true"));
        assert!(search_in_value(&data, "null"));
        assert!(search_in_value(&data, "value"));
    }

    #[test]
    fn test_search_partial_match() {
        let data = json!("programming");
        assert!(search_in_value(&data, "prog"));
        assert!(search_in_value(&data, "gram"));
        assert!(search_in_value(&data, "ing"));
    }

    #[test]
    fn test_search_empty_string() {
        let data = json!("test");
        assert!(search_in_value(&data, "")); // Empty string matches everything
    }

    #[test]
    fn test_search_case_insensitive() {
        let data = json!("HeLLo WoRLd");
        assert!(search_in_value(&data, "hello world"));
        assert!(search_in_value(&data, "HELLO WORLD"));
        assert!(search_in_value(&data, "HeLLo WoRLd"));
    }

    #[test]
    fn test_search_in_complex_structure() {
        let data = json!({
            "company": {
                "name": "TechCorp",
                "departments": [
                    {
                        "name": "Engineering",
                        "employees": [
                            {"name": "Alice", "role": "Developer"},
                            {"name": "Bob", "role": "Lead"}
                        ]
                    }
                ]
            }
        });
        
        assert!(search_in_value(&data, "techcorp"));
        assert!(search_in_value(&data, "engineering"));
        assert!(search_in_value(&data, "alice"));
        assert!(search_in_value(&data, "developer"));
        assert!(!search_in_value(&data, "sales"));
    }
}
