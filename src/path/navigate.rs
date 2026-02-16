//! Path navigation utilities

/// Split path into parent and key
///
/// Returns `(parent_path, last_key)` tuple.
///
/// # Examples
///
/// ```rust
/// use jsonq::path::split_path_key;
///
/// assert_eq!(split_path_key("user.name"), ("user", "name"));
/// assert_eq!(split_path_key("a.b.c"), ("a.b", "c"));
/// assert_eq!(split_path_key("single"), ("", "single"));
/// assert_eq!(split_path_key(""), ("", ""));
/// ```
pub fn split_path_key(path: &str) -> (&str, &str) {
    if path.is_empty() {
        return ("", "");
    }
    
    if let Some(pos) = path.rfind('.') {
        (&path[..pos], &path[pos + 1..])
    } else {
        ("", path)
    }
}

/// Check if a path segment represents an array index
///
/// # Examples
///
/// ```rust
/// use jsonq::path::navigate::is_array_index;
///
/// assert!(is_array_index("0"));
/// assert!(is_array_index("42"));
/// assert!(!is_array_index("key"));
/// assert!(!is_array_index(""));
/// ```
pub fn is_array_index(key: &str) -> bool {
    key.parse::<usize>().is_ok()
}

/// Get path depth (number of segments)
///
/// # Examples
///
/// ```rust
/// use jsonq::path::navigate::path_depth;
///
/// assert_eq!(path_depth(""), 0);
/// assert_eq!(path_depth("key"), 1);
/// assert_eq!(path_depth("user.name"), 2);
/// assert_eq!(path_depth("a.b.c.d"), 4);
/// ```
pub fn path_depth(path: &str) -> usize {
    if path.is_empty() {
        0
    } else {
        path.split('.').count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_path_key_nested() {
        let (parent, key) = split_path_key("user.profile.name");
        assert_eq!(parent, "user.profile");
        assert_eq!(key, "name");
    }

    #[test]
    fn test_split_path_key_single() {
        let (parent, key) = split_path_key("name");
        assert_eq!(parent, "");
        assert_eq!(key, "name");
    }

    #[test]
    fn test_split_path_key_empty() {
        let (parent, key) = split_path_key("");
        assert_eq!(parent, "");
        assert_eq!(key, "");
    }

    #[test]
    fn test_is_array_index_valid() {
        assert!(is_array_index("0"));
        assert!(is_array_index("1"));
        assert!(is_array_index("42"));
        assert!(is_array_index("999"));
    }

    #[test]
    fn test_is_array_index_invalid() {
        assert!(!is_array_index(""));
        assert!(!is_array_index("key"));
        assert!(!is_array_index("a1"));
        assert!(!is_array_index("1a"));
        assert!(!is_array_index("-1"));
    }

    #[test]
    fn test_path_depth() {
        assert_eq!(path_depth(""), 0);
        assert_eq!(path_depth("key"), 1);
        assert_eq!(path_depth("a.b"), 2);
        assert_eq!(path_depth("a.b.c"), 3);
        assert_eq!(path_depth("user.address.city.zipcode"), 4);
    }
}
