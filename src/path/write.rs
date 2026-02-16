//! Path writing and removal functions

use serde_json::{Value, Map};

/// Write value at path, creating intermediate objects/arrays as needed
///
/// Automatically creates intermediate objects and arrays based on the path structure:
/// - String keys → Object
/// - Numeric keys → Array
///
/// # Examples
///
/// ```rust
/// use jsonq::path::write_path;
/// use serde_json::json;
///
/// let mut data = json!({});
/// 
/// // Creates nested objects automatically
/// write_path(&mut data, "user.name", json!("Alice"));
/// assert_eq!(data["user"]["name"], "Alice");
///
/// // Creates arrays when path contains numbers
/// write_path(&mut data, "items.0", json!("first"));
/// assert_eq!(data["items"][0], "first");
/// ```
///
/// # Array Behavior
///
/// When writing to an array index:
/// - If index exists: overwrites the value
/// - If index == length: appends to array
/// - If index > length: fills gaps with null, then appends
pub fn write_path(root: &mut Value, path: &str, value: Value) {
    let keys: Vec<&str> = path.split('.').collect();
    let mut current = root;
    
    for (i, &key) in keys.iter().enumerate() {
        // Last key - write the value
        if i == keys.len() - 1 {
            match current {
                Value::Object(map) => {
                    map.insert(key.to_string(), value);
                }
                Value::Array(arr) => {
                    if let Ok(index) = key.parse::<usize>() {
                        if index < arr.len() {
                            arr[index] = value;
                        } else {
                            // Fill gaps with null if needed
                            while arr.len() < index {
                                arr.push(Value::Null);
                            }
                            arr.push(value);
                        }
                    }
                }
                _ => {}
            }
            return;
        }
        
        // Intermediate key - navigate or create
        let next_is_numeric = keys.get(i + 1)
            .and_then(|k| k.parse::<usize>().ok())
            .is_some();
        
        match current {
            Value::Object(map) => {
                if !map.contains_key(key) {
                    let new_value = if next_is_numeric {
                        Value::Array(vec![])
                    } else {
                        Value::Object(Map::new())
                    };
                    map.insert(key.to_string(), new_value);
                }
                current = map.get_mut(key).unwrap();
            }
            Value::Array(arr) => {
                if let Ok(index) = key.parse::<usize>() {
                    // Ensure array is large enough
                    while arr.len() <= index {
                        arr.push(Value::Object(Map::new()));
                    }
                    current = &mut arr[index];
                } else {
                    return; // Invalid array index
                }
            }
            _ => return,
        }
    }
}

/// Remove value at path
///
/// Returns `true` if the value was removed, `false` if path didn't exist.
///
/// # Examples
///
/// ```rust
/// use jsonq::path::remove_path;
/// use serde_json::json;
///
/// let mut data = json!({"user": {"name": "Alice", "age": 30}});
/// 
/// assert_eq!(remove_path(&mut data, "user.age"), true);
/// assert_eq!(data["user"].get("age"), None);
/// 
/// assert_eq!(remove_path(&mut data, "nonexistent"), false);
/// ```
///
/// # Array Removal
///
/// When removing from an array, the element is removed and subsequent
/// elements shift down (indices change).
pub fn remove_path(root: &mut Value, path: &str) -> bool {
    let mut keys: Vec<&str> = path.split('.').collect();
    
    if keys.is_empty() {
        return false;
    }
    
    let last_key = keys.pop().unwrap();
    let parent_path = keys.join(".");
    
    // Get parent (create temporary scope for borrow)
    let parent = if parent_path.is_empty() {
        Some(root)
    } else {
        super::read::read_path_mut(root, &parent_path)
    };
    
    if let Some(target) = parent {
        match target {
            Value::Object(map) => map.remove(last_key).is_some(),
            Value::Array(arr) => {
                if let Ok(index) = last_key.parse::<usize>() {
                    if index < arr.len() {
                        arr.remove(index);
                        return true;
                    }
                }
                false
            }
            _ => false,
        }
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_write_simple_key() {
        let mut data = json!({});
        write_path(&mut data, "name", json!("Alice"));
        assert_eq!(data["name"], "Alice");
    }

    #[test]
    fn test_write_creates_nested_objects() {
        let mut data = json!({});
        write_path(&mut data, "user.name", json!("Bob"));
        assert_eq!(data["user"]["name"], "Bob");
    }

    #[test]
    fn test_write_deep_nesting() {
        let mut data = json!({});
        write_path(&mut data, "a.b.c.d", json!("deep"));
        assert_eq!(data["a"]["b"]["c"]["d"], "deep");
    }

    #[test]
    fn test_write_creates_array() {
        let mut data = json!({});
        write_path(&mut data, "items.0", json!("first"));
        assert_eq!(data["items"][0], "first");
    }

    #[test]
    fn test_write_array_append() {
        let mut data = json!({"items": [1, 2]});
        write_path(&mut data, "items.2", json!(3));
        assert_eq!(data["items"], json!([1, 2, 3]));
    }

    #[test]
    fn test_write_array_with_gaps() {
        let mut data = json!({"items": []});
        write_path(&mut data, "items.3", json!("fourth"));
        
        // Should fill gaps with null
        assert_eq!(data["items"][0], Value::Null);
        assert_eq!(data["items"][1], Value::Null);
        assert_eq!(data["items"][2], Value::Null);
        assert_eq!(data["items"][3], "fourth");
    }

    #[test]
    fn test_write_overwrites_existing() {
        let mut data = json!({"key": "old"});
        write_path(&mut data, "key", json!("new"));
        assert_eq!(data["key"], "new");
    }

    #[test]
    fn test_write_mixed_object_array() {
        let mut data = json!({});
        write_path(&mut data, "users.0.name", json!("Alice"));
        assert_eq!(data["users"][0]["name"], "Alice");
    }

    #[test]
    fn test_remove_simple_key() {
        let mut data = json!({"name": "Alice", "age": 30});
        assert!(remove_path(&mut data, "age"));
        assert_eq!(data.get("age"), None);
        assert_eq!(data["name"], "Alice");
    }

    #[test]
    fn test_remove_nested_key() {
        let mut data = json!({"user": {"name": "Bob", "age": 25}});
        assert!(remove_path(&mut data, "user.age"));
        assert_eq!(data["user"].get("age"), None);
        assert_eq!(data["user"]["name"], "Bob");
    }

    #[test]
    fn test_remove_nonexistent_returns_false() {
        let mut data = json!({"key": "value"});
        assert!(!remove_path(&mut data, "nonexistent"));
    }

    #[test]
    fn test_remove_from_array() {
        let mut data = json!({"items": [1, 2, 3]});
        assert!(remove_path(&mut data, "items.1"));
        assert_eq!(data["items"], json!([1, 3]));
    }

    #[test]
    fn test_remove_invalid_array_index() {
        let mut data = json!({"items": [1, 2, 3]});
        assert!(!remove_path(&mut data, "items.10"));
        assert_eq!(data["items"], json!([1, 2, 3]));
    }
}
