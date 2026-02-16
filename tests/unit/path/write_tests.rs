//! Tests for path writing functions

use jsonq::path::{write_path, remove_path};
use std::sync::Arc;
use serde_json::json;

#[test]
fn test_write_to_empty_object() {
    let mut data = json!({});
    write_path(&mut data, "key", json!("value"));
    assert_eq!(data["key"], "value");
}

#[test]
fn test_write_creates_nested_objects() {
    let mut data = json!({});
    write_path(&mut data, "user.name", json!("Alice"));
    
    assert_eq!(data["user"]["name"], "Alice");
    assert!(data["user"].is_object());
}

#[test]
fn test_write_deep_nesting() {
    let mut data = json!({});
    write_path(&mut data, "a.b.c.d.e", json!("deep"));
    
    assert_eq!(data["a"]["b"]["c"]["d"]["e"], "deep");
}

#[test]
fn test_write_overwrites_existing() {
    let mut data = json!({"key": "old"});
    write_path(&mut data, "key", json!("new"));
    
    assert_eq!(data["key"], "new");
}

#[test]
fn test_write_creates_array() {
    let mut data = json!({});
    write_path(&mut data, "items.0", json!("first"));
    
    assert!(data["items"].is_array());
    assert_eq!(data["items"][0], "first");
}

#[test]
fn test_write_array_multiple_items() {
    let mut data = json!({});
    write_path(&mut data, "items.0", json!(1));
    write_path(&mut data, "items.1", json!(2));
    write_path(&mut data, "items.2", json!(3));
    
    assert_eq!(data["items"], json!([1, 2, 3]));
}

#[test]
fn test_write_array_with_gap() {
    let mut data = json!({"items": []});
    write_path(&mut data, "items.3", json!("fourth"));
    
    // Should fill [null, null, null, "fourth"]
    assert_eq!(data["items"].as_array().unwrap().len(), 4);
    assert_eq!(data["items"][0], json!(null));
    assert_eq!(data["items"][1], json!(null));
    assert_eq!(data["items"][2], json!(null));
    assert_eq!(data["items"][3], "fourth");
}

#[test]
fn test_write_overwrites_array_element() {
    let mut data = json!({"items": [1, 2, 3]});
    write_path(&mut data, "items.1", json!(99));
    
    assert_eq!(data["items"], json!([1, 99, 3]));
}

#[test]
fn test_write_mixed_object_array() {
    let mut data = json!({});
    write_path(&mut data, "users.0.name", json!("Alice"));
    write_path(&mut data, "users.0.age", json!(30));
    write_path(&mut data, "users.1.name", json!("Bob"));
    
    assert_eq!(data["users"][0]["name"], "Alice");
    assert_eq!(data["users"][0]["age"], 30);
    assert_eq!(data["users"][1]["name"], "Bob");
}

#[test]
fn test_write_various_types() {
    let mut data = json!({});
    
    write_path(&mut data, "string", json!("hello"));
    write_path(&mut data, "number", json!(42));
    write_path(&mut data, "float", json!(3.14));
    write_path(&mut data, "bool", json!(true));
    write_path(&mut data, "null", json!(null));
    write_path(&mut data, "array", json!([1, 2, 3]));
    write_path(&mut data, "object", json!({"nested": "value"}));
    
    assert_eq!(data["string"], "hello");
    assert_eq!(data["number"], 42);
    assert_eq!(data["float"], 3.14);
    assert_eq!(data["bool"], true);
    assert_eq!(data["null"], json!(null));
    assert_eq!(data["array"], json!([1, 2, 3]));
    assert_eq!(data["object"]["nested"], "value");
}

#[test]
fn test_remove_top_level_key() {
    let mut data = json!({"name": "Alice", "age": 30});
    
    assert!(remove_path(&mut data, "age"));
    assert_eq!(data.get("age"), None);
    assert_eq!(data["name"], "Alice");
}

#[test]
fn test_remove_nested_key() {
    let mut data = json!({
        "user": {
            "name": "Bob",
            "age": 25,
            "email": "bob@example.com"
        }
    });
    
    assert!(remove_path(&mut data, "user.age"));
    assert_eq!(data["user"].get("age"), None);
    assert_eq!(data["user"]["name"], "Bob");
}

#[test]
fn test_remove_nonexistent_returns_false() {
    let mut data = json!({"key": "value"});
    
    assert!(!remove_path(&mut data, "nonexistent"));
    assert_eq!(data, json!({"key": "value"}));
}

#[test]
fn test_remove_from_array() {
    let mut data = json!({"items": [10, 20, 30, 40]});
    
    assert!(remove_path(&mut data, "items.1"));
    assert_eq!(data["items"], json!([10, 30, 40]));
}

#[test]
fn test_remove_array_first_element() {
    let mut data = json!({"items": ["a", "b", "c"]});
    
    assert!(remove_path(&mut data, "items.0"));
    assert_eq!(data["items"], json!(["b", "c"]));
}

#[test]
fn test_remove_array_last_element() {
    let mut data = json!({"items": [1, 2, 3]});
    
    assert!(remove_path(&mut data, "items.2"));
    assert_eq!(data["items"], json!([1, 2]));
}

#[test]
fn test_remove_invalid_array_index() {
    let mut data = json!({"items": [1, 2, 3]});
    
    assert!(!remove_path(&mut data, "items.10"));
    assert_eq!(data["items"], json!([1, 2, 3]));
}

#[test]
fn test_remove_from_nested_array() {
    let mut data = json!({
        "users": [
            {"name": "Alice"},
            {"name": "Bob"},
            {"name": "Charlie"}
        ]
    });
    
    assert!(remove_path(&mut data, "users.1"));
    assert_eq!(data["users"].as_array().unwrap().len(), 2);
    assert_eq!(data["users"][0]["name"], "Alice");
    assert_eq!(data["users"][1]["name"], "Charlie");
}

#[test]
fn test_remove_deeply_nested() {
    let mut data = json!({
        "a": {
            "b": {
                "c": {
                    "target": "remove me",
                    "keep": "this"
                }
            }
        }
    });
    
    assert!(remove_path(&mut data, "a.b.c.target"));
    assert_eq!(data["a"]["b"]["c"].get("target"), None);
    assert_eq!(data["a"]["b"]["c"]["keep"], "this");
}

#[test]
fn test_write_then_remove() {
    let mut data = json!({});
    
    write_path(&mut data, "temp.value", json!(42));
    assert_eq!(data["temp"]["value"], 42);
    
    remove_path(&mut data, "temp.value");
    assert_eq!(data["temp"].get("value"), None);
}

#[test]
fn test_multiple_writes_same_path() {
    let mut data = json!({});
    
    write_path(&mut data, "counter", json!(1));
    write_path(&mut data, "counter", json!(2));
    write_path(&mut data, "counter", json!(3));
    
    assert_eq!(data["counter"], 3);
}
