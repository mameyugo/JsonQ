//! Deep merge for JSON values

use serde_json::Value;

/// Deep merge two JSON values
///
/// Merges `new` into `base`, combining objects recursively and
/// concatenating arrays.
///
/// # Merge Behavior
///
/// - **Both objects**: Recursively merge properties
/// - **Both arrays**: Concatenate (base + new)
/// - **Different types**: Replace base with new
///
/// # Examples
///
/// ## Object Merge
///
/// ```rust
/// use jsonq::utils::merge_values;
/// use serde_json::json;
///
/// let mut base = json!({"a": 1, "b": {"x": 10}});
/// let new = json!({"b": {"y": 20}, "c": 3});
/// 
/// merge_values(&mut base, &new);
/// 
/// assert_eq!(base, json!({
///     "a": 1,
///     "b": {"x": 10, "y": 20},
///     "c": 3
/// }));
/// ```
///
/// ## Array Concatenation
///
/// ```rust
/// use jsonq::utils::merge_values;
/// use serde_json::json;
///
/// let mut base = json!([1, 2, 3]);
/// let new = json!([4, 5]);
/// 
/// merge_values(&mut base, &new);
/// assert_eq!(base, json!([1, 2, 3, 4, 5]));
/// ```
///
/// ## Type Replacement
///
/// ```rust
/// use jsonq::utils::merge_values;
/// use serde_json::json;
///
/// let mut base = json!({"value": 42});
/// let new = json!({"value": "text"});
/// 
/// merge_values(&mut base, &new);
/// assert_eq!(base["value"], "text");
/// ```
pub fn merge_values(base: &mut Value, new: &Value) {
    match (base, new) {
        // Both objects: recursively merge
        (Value::Object(base_map), Value::Object(new_map)) => {
            for (key, new_value) in new_map {
                base_map
                    .entry(key.clone())
                    .and_modify(|base_value| merge_values(base_value, new_value))
                    .or_insert_with(|| new_value.clone());
            }
        }
        
        // Both arrays: concatenate
        (Value::Array(base_arr), Value::Array(new_arr)) => {
            base_arr.extend(new_arr.clone());
        }
        
        // Different types or base is not object/array: replace
        (base, new) => {
            *base = new.clone();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_merge_empty_objects() {
        let mut base = json!({});
        let new = json!({});
        merge_values(&mut base, &new);
        assert_eq!(base, json!({}));
    }

    #[test]
    fn test_merge_into_empty_object() {
        let mut base = json!({});
        let new = json!({"a": 1, "b": 2});
        merge_values(&mut base, &new);
        assert_eq!(base, json!({"a": 1, "b": 2}));
    }

    #[test]
    fn test_merge_from_empty_object() {
        let mut base = json!({"a": 1, "b": 2});
        let new = json!({});
        merge_values(&mut base, &new);
        assert_eq!(base, json!({"a": 1, "b": 2}));
    }

    #[test]
    fn test_merge_objects_no_overlap() {
        let mut base = json!({"a": 1});
        let new = json!({"b": 2});
        merge_values(&mut base, &new);
        assert_eq!(base, json!({"a": 1, "b": 2}));
    }

    #[test]
    fn test_merge_objects_with_overlap() {
        let mut base = json!({"a": 1, "b": 2});
        let new = json!({"b": 20, "c": 3});
        merge_values(&mut base, &new);
        assert_eq!(base, json!({"a": 1, "b": 20, "c": 3}));
    }

    #[test]
    fn test_merge_nested_objects() {
        let mut base = json!({
            "user": {
                "name": "Alice",
                "age": 30
            }
        });
        let new = json!({
            "user": {
                "email": "alice@example.com"
            }
        });
        
        merge_values(&mut base, &new);
        
        assert_eq!(base, json!({
            "user": {
                "name": "Alice",
                "age": 30,
                "email": "alice@example.com"
            }
        }));
    }

    #[test]
    fn test_merge_deeply_nested_objects() {
        let mut base = json!({
            "a": {
                "b": {
                    "c": 1
                }
            }
        });
        let new = json!({
            "a": {
                "b": {
                    "d": 2
                }
            }
        });
        
        merge_values(&mut base, &new);
        
        assert_eq!(base, json!({
            "a": {
                "b": {
                    "c": 1,
                    "d": 2
                }
            }
        }));
    }

    #[test]
    fn test_merge_arrays() {
        let mut base = json!([1, 2, 3]);
        let new = json!([4, 5]);
        merge_values(&mut base, &new);
        assert_eq!(base, json!([1, 2, 3, 4, 5]));
    }

    #[test]
    fn test_merge_empty_arrays() {
        let mut base = json!([]);
        let new = json!([1, 2]);
        merge_values(&mut base, &new);
        assert_eq!(base, json!([1, 2]));
    }

    #[test]
    fn test_merge_array_into_empty() {
        let mut base = json!([1, 2]);
        let new = json!([]);
        merge_values(&mut base, &new);
        assert_eq!(base, json!([1, 2]));
    }

    #[test]
    fn test_merge_replaces_different_types() {
        let mut base = json!({"value": 42});
        let new = json!({"value": "text"});
        merge_values(&mut base, &new);
        assert_eq!(base["value"], "text");
    }

    #[test]
    fn test_merge_number_to_object() {
        let mut base = json!(42);
        let new = json!({"key": "value"});
        merge_values(&mut base, &new);
        assert_eq!(base, json!({"key": "value"}));
    }

    #[test]
    fn test_merge_object_to_array() {
        let mut base = json!({"a": 1});
        let new = json!([1, 2, 3]);
        merge_values(&mut base, &new);
        assert_eq!(base, json!([1, 2, 3]));
    }

    #[test]
    fn test_merge_preserves_types() {
        let mut base = json!({
            "string": "hello",
            "number": 42,
            "bool": true,
            "null": null
        });
        let new = json!({
            "string": "world",
            "number": 99
        });
        
        merge_values(&mut base, &new);
        
        assert_eq!(base["string"], "world");
        assert_eq!(base["number"], 99);
        assert_eq!(base["bool"], true);
        assert_eq!(base["null"], json!(null));
    }

    #[test]
    fn test_merge_complex_structure() {
        let mut base = json!({
            "config": {
                "database": {
                    "host": "localhost",
                    "port": 5432
                },
                "cache": {
                    "enabled": true
                }
            },
            "features": ["auth", "api"]
        });
        
        let new = json!({
            "config": {
                "database": {
                    "username": "admin"
                },
                "logging": {
                    "level": "info"
                }
            },
            "features": ["websockets"]
        });
        
        merge_values(&mut base, &new);
        
        assert_eq!(base["config"]["database"]["host"], "localhost");
        assert_eq!(base["config"]["database"]["username"], "admin");
        assert_eq!(base["config"]["logging"]["level"], "info");
        assert_eq!(base["features"], json!(["auth", "api", "websockets"]));
    }

    #[test]
    fn test_merge_overwrites_nested_value() {
        let mut base = json!({
            "user": {
                "settings": {
                    "theme": "light"
                }
            }
        });
        let new = json!({
            "user": {
                "settings": {
                    "theme": "dark"
                }
            }
        });
        
        merge_values(&mut base, &new);
        assert_eq!(base["user"]["settings"]["theme"], "dark");
    }
}
