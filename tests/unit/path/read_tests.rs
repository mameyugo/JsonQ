//! Tests for path reading functions

use jsonq::path::{read_path, read_path_mut, read_nested};
use std::sync::Arc;
use serde_json::json;

#[test]
fn test_read_root_with_empty_path() {
    let data = json!({"key": "value"});
    assert_eq!(read_path(&data, ""), Some(&data));
}

#[test]
fn test_read_top_level_key() {
    let data = json!({"name": "Alice", "age": 30});
    assert_eq!(read_path(&data, "name"), Some(&json!("Alice")));
    assert_eq!(read_path(&data, "age"), Some(&json!(30)));
}

#[test]
fn test_read_nested_object() {
    let data = json!({
        "user": {
            "name": "Bob",
            "email": "bob@example.com"
        }
    });
    assert_eq!(read_path(&data, "user.name"), Some(&json!("Bob")));
    assert_eq!(read_path(&data, "user.email"), Some(&json!("bob@example.com")));
}

#[test]
fn test_read_deeply_nested() {
    let data = json!({
        "level1": {
            "level2": {
                "level3": {
                    "value": 42
                }
            }
        }
    });
    assert_eq!(read_path(&data, "level1.level2.level3.value"), Some(&json!(42)));
}

#[test]
fn test_read_array_by_index() {
    let data = json!({"items": [10, 20, 30]});
    assert_eq!(read_path(&data, "items.0"), Some(&json!(10)));
    assert_eq!(read_path(&data, "items.1"), Some(&json!(20)));
    assert_eq!(read_path(&data, "items.2"), Some(&json!(30)));
}

#[test]
fn test_read_nested_array() {
    let data = json!({
        "users": [
            {"name": "Alice", "age": 30},
            {"name": "Bob", "age": 25}
        ]
    });
    assert_eq!(read_path(&data, "users.0.name"), Some(&json!("Alice")));
    assert_eq!(read_path(&data, "users.1.age"), Some(&json!(25)));
}

#[test]
fn test_read_nonexistent_key() {
    let data = json!({"key": "value"});
    assert_eq!(read_path(&data, "nonexistent"), None);
}

#[test]
fn test_read_nonexistent_nested() {
    let data = json!({"user": {"name": "Alice"}});
    assert_eq!(read_path(&data, "user.age"), None);
    assert_eq!(read_path(&data, "user.profile.bio"), None);
}

#[test]
fn test_read_invalid_array_index() {
    let data = json!({"items": [1, 2, 3]});
    assert_eq!(read_path(&data, "items.10"), None);
    assert_eq!(read_path(&data, "items.999"), None);
}

#[test]
fn test_read_non_numeric_array_index() {
    let data = json!({"items": [1, 2, 3]});
    assert_eq!(read_path(&data, "items.invalid"), None);
    assert_eq!(read_path(&data, "items.abc"), None);
}

#[test]
fn test_read_through_non_object() {
    let data = json!({"value": 42});
    assert_eq!(read_path(&data, "value.something"), None);
}

#[test]
fn test_read_path_mut_basic() {
    let mut data = json!({"counter": 10});
    
    if let Some(counter) = read_path_mut(&mut data, "counter") {
        *counter = json!(20);
    }
    
    assert_eq!(data["counter"], 20);
}

#[test]
fn test_read_path_mut_nested() {
    let mut data = json!({
        "user": {
            "score": 100
        }
    });
    
    if let Some(score) = read_path_mut(&mut data, "user.score") {
        *score = json!(150);
    }
    
    assert_eq!(data["user"]["score"], 150);
}

#[test]
fn test_read_path_mut_array() {
    let mut data = json!({"items": [1, 2, 3]});
    
    if let Some(item) = read_path_mut(&mut data, "items.1") {
        *item = json!(99);
    }
    
    assert_eq!(data["items"][1], 99);
}

#[test]
fn test_read_nested_alias() {
    let user = json!({
        "profile": {
            "settings": {
                "theme": "dark"
            }
        }
    });
    
    let theme = read_nested(&user, "profile.settings.theme");
    assert_eq!(theme, Some(&json!("dark")));
}

#[test]
fn test_read_complex_structure() {
    let data = json!({
        "company": {
            "departments": [
                {
                    "name": "Engineering",
                    "employees": [
                        {"name": "Alice", "role": "Dev"},
                        {"name": "Bob", "role": "Lead"}
                    ]
                },
                {
                    "name": "Sales",
                    "employees": [
                        {"name": "Charlie", "role": "Rep"}
                    ]
                }
            ]
        }
    });
    
    assert_eq!(
        read_path(&data, "company.departments.0.employees.1.name"),
        Some(&json!("Bob"))
    );
}

#[test]
fn test_read_various_types() {
    let data = json!({
        "string": "hello",
        "number": 42,
        "float": 3.14,
        "bool": true,
        "null": null,
        "array": [1, 2, 3],
        "object": {"nested": "value"}
    });
    
    assert_eq!(read_path(&data, "string"), Some(&json!("hello")));
    assert_eq!(read_path(&data, "number"), Some(&json!(42)));
    assert_eq!(read_path(&data, "float"), Some(&json!(3.14)));
    assert_eq!(read_path(&data, "bool"), Some(&json!(true)));
    assert_eq!(read_path(&data, "null"), Some(&json!(null)));
    assert_eq!(read_path(&data, "array"), Some(&json!([1, 2, 3])));
    assert_eq!(read_path(&data, "object.nested"), Some(&json!("value")));
}
