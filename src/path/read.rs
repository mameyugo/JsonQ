//! Path reading functions

use serde_json::Value;

/// Read value at path (immutable reference)
///
/// Returns `Some(&Value)` if path exists, `None` otherwise.
///
/// # Examples
///
/// ```rust
/// use jsonq::path::read_path;
/// use serde_json::json;
///
/// let data = json!({"user": {"name": "Alice", "age": 30}});
///
/// assert_eq!(read_path(&data, "user.name"), Some(&json!("Alice")));
/// assert_eq!(read_path(&data, "user.age"), Some(&json!(30)));
/// assert_eq!(read_path(&data, "user.email"), None);
/// ```
///
/// # Array Access
///
/// ```rust
/// use jsonq::path::read_path;
/// use serde_json::json;
///
/// let data = json!({"items": [1, 2, 3]});
/// assert_eq!(read_path(&data, "items.0"), Some(&json!(1)));
/// assert_eq!(read_path(&data, "items.2"), Some(&json!(3)));
/// ```
pub fn read_path<'a>(root: &'a Value, path: &str) -> Option<&'a Value> {
    // Empty path returns root
    if path.is_empty() {
        return Some(root);
    }

    let mut current = root;

    for key in path.split('.') {
        current = match current {
            Value::Object(map) => map.get(key)?,
            Value::Array(arr) => {
                let index = key.parse::<usize>().ok()?;
                arr.get(index)?
            }
            _ => return None,
        };
    }

    Some(current)
}

/// Read value at path (mutable reference)
///
/// Returns `Some(&mut Value)` if path exists, `None` otherwise.
///
/// # Examples
///
/// ```rust
/// use jsonq::path::read_path_mut;
/// use serde_json::json;
///
/// let mut data = json!({"counter": 0});
///
/// if let Some(counter) = read_path_mut(&mut data, "counter") {
///     *counter = json!(42);
/// }
///
/// assert_eq!(data["counter"], 42);
/// ```
pub fn read_path_mut<'a>(root: &'a mut Value, path: &str) -> Option<&'a mut Value> {
    // Empty path returns root
    if path.is_empty() {
        return Some(root);
    }

    let mut current = root;

    for key in path.split('.') {
        current = match current {
            Value::Object(map) => map.get_mut(key)?,
            Value::Array(arr) => {
                let index = key.parse::<usize>().ok()?;
                arr.get_mut(index)?
            }
            _ => return None,
        };
    }

    Some(current)
}

/// Read nested value from within a Value (using path relative to that value)
///
/// This is an alias for `read_path` but makes the intent clearer when
/// navigating within an already-extracted value.
///
/// # Examples
///
/// ```rust
/// use jsonq::path::read_nested;
/// use serde_json::json;
///
/// let user = json!({"profile": {"settings": {"theme": "dark"}}});
///
/// // Navigate within the user object
/// let theme = read_nested(&user, "profile.settings.theme");
/// assert_eq!(theme, Some(&json!("dark")));
/// ```
pub fn read_nested<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
    read_path(value, path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_read_empty_path_returns_root() {
        let data = json!({"key": "value"});
        assert_eq!(read_path(&data, ""), Some(&data));
    }

    #[test]
    fn test_read_simple_key() {
        let data = json!({"name": "Alice"});
        assert_eq!(read_path(&data, "name"), Some(&json!("Alice")));
    }

    #[test]
    fn test_read_nested_object() {
        let data = json!({"user": {"name": "Bob"}});
        assert_eq!(read_path(&data, "user.name"), Some(&json!("Bob")));
    }

    #[test]
    fn test_read_deep_nesting() {
        let data = json!({
            "a": {"b": {"c": {"d": "deep"}}}
        });
        assert_eq!(read_path(&data, "a.b.c.d"), Some(&json!("deep")));
    }

    #[test]
    fn test_read_array_index() {
        let data = json!({"items": [1, 2, 3]});
        assert_eq!(read_path(&data, "items.0"), Some(&json!(1)));
        assert_eq!(read_path(&data, "items.2"), Some(&json!(3)));
    }

    #[test]
    fn test_read_nested_array() {
        let data = json!({"users": [{"name": "Alice"}, {"name": "Bob"}]});
        assert_eq!(read_path(&data, "users.0.name"), Some(&json!("Alice")));
        assert_eq!(read_path(&data, "users.1.name"), Some(&json!("Bob")));
    }

    #[test]
    fn test_read_nonexistent_key() {
        let data = json!({"key": "value"});
        assert_eq!(read_path(&data, "nonexistent"), None);
    }

    #[test]
    fn test_read_invalid_array_index() {
        let data = json!({"items": [1, 2, 3]});
        assert_eq!(read_path(&data, "items.10"), None);
        assert_eq!(read_path(&data, "items.invalid"), None);
    }

    #[test]
    fn test_read_path_mut_simple() {
        let mut data = json!({"counter": 0});

        if let Some(counter) = read_path_mut(&mut data, "counter") {
            *counter = json!(42);
        }

        assert_eq!(data["counter"], 42);
    }

    #[test]
    fn test_read_path_mut_nested() {
        let mut data = json!({"user": {"score": 100}});

        if let Some(score) = read_path_mut(&mut data, "user.score") {
            *score = json!(200);
        }

        assert_eq!(data["user"]["score"], 200);
    }
}
