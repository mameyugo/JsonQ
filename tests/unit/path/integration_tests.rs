//! Integration tests combining read/write/navigate

use jsonq::path::{read_path, remove_path, write_path};
use serde_json::json;
use std::sync::Arc;

#[test]
fn test_crud_workflow() {
    let mut data = json!({});

    // Create
    write_path(&mut data, "user.name", json!("Alice"));
    write_path(&mut data, "user.age", json!(30));

    // Read
    assert_eq!(read_path(&data, "user.name"), Some(&json!("Alice")));
    assert_eq!(read_path(&data, "user.age"), Some(&json!(30)));

    // Update
    write_path(&mut data, "user.age", json!(31));
    assert_eq!(read_path(&data, "user.age"), Some(&json!(31)));

    // Delete
    remove_path(&mut data, "user.age");
    assert_eq!(read_path(&data, "user.age"), None);
}

#[test]
fn test_build_complex_structure() {
    let mut data = json!({});

    write_path(&mut data, "company.name", json!("TechCorp"));
    write_path(&mut data, "company.employees.0.name", json!("Alice"));
    write_path(&mut data, "company.employees.0.role", json!("Engineer"));
    write_path(&mut data, "company.employees.1.name", json!("Bob"));
    write_path(&mut data, "company.employees.1.role", json!("Manager"));

    assert_eq!(read_path(&data, "company.name"), Some(&json!("TechCorp")));
    assert_eq!(
        read_path(&data, "company.employees.0.name"),
        Some(&json!("Alice"))
    );
    assert_eq!(
        read_path(&data, "company.employees.1.role"),
        Some(&json!("Manager"))
    );
}

#[test]
fn test_array_manipulation() {
    let mut data = json!({"items": []});

    // Add items
    write_path(&mut data, "items.0", json!("first"));
    write_path(&mut data, "items.1", json!("second"));
    write_path(&mut data, "items.2", json!("third"));

    assert_eq!(data["items"].as_array().unwrap().len(), 3);

    // Remove middle item
    remove_path(&mut data, "items.1");
    assert_eq!(data["items"].as_array().unwrap().len(), 2);
    assert_eq!(read_path(&data, "items.0"), Some(&json!("first")));
    assert_eq!(read_path(&data, "items.1"), Some(&json!("third")));
}

#[test]
fn test_nested_array_of_objects() {
    let mut data = json!({});

    write_path(&mut data, "users.0.name", json!("Alice"));
    write_path(&mut data, "users.0.tags.0", json!("admin"));
    write_path(&mut data, "users.0.tags.1", json!("dev"));
    write_path(&mut data, "users.1.name", json!("Bob"));
    write_path(&mut data, "users.1.tags.0", json!("user"));

    assert_eq!(read_path(&data, "users.0.tags.1"), Some(&json!("dev")));
    assert_eq!(read_path(&data, "users.1.tags.0"), Some(&json!("user")));
}

#[test]
fn test_deep_path_navigation() {
    let mut data = json!({});

    let deep_path = "level1.level2.level3.level4.level5.value";
    write_path(&mut data, deep_path, json!("deep"));

    assert_eq!(read_path(&data, deep_path), Some(&json!("deep")));

    // Verify intermediate levels exist
    assert!(read_path(&data, "level1").is_some());
    assert!(read_path(&data, "level1.level2").is_some());
    assert!(read_path(&data, "level1.level2.level3").is_some());
}

#[test]
fn test_partial_path_removal() {
    let mut data = json!({
        "user": {
            "profile": {
                "name": "Alice",
                "age": 30,
                "email": "alice@example.com"
            },
            "settings": {
                "theme": "dark"
            }
        }
    });

    // Remove one field from profile
    remove_path(&mut data, "user.profile.age");

    // Other fields still exist
    assert_eq!(read_path(&data, "user.profile.name"), Some(&json!("Alice")));
    assert_eq!(
        read_path(&data, "user.profile.email"),
        Some(&json!("alice@example.com"))
    );
    assert_eq!(
        read_path(&data, "user.settings.theme"),
        Some(&json!("dark"))
    );

    // Removed field is gone
    assert_eq!(read_path(&data, "user.profile.age"), None);
}

#[test]
fn test_overwrite_type_change() {
    let mut data = json!({"value": 42});

    // Change from number to object
    write_path(&mut data, "value", json!({"nested": "object"}));
    assert_eq!(read_path(&data, "value.nested"), Some(&json!("object")));

    // Change from object to array
    write_path(&mut data, "value", json!([1, 2, 3]));
    assert_eq!(read_path(&data, "value.1"), Some(&json!(2)));
}

#[test]
fn test_write_read_all_types() {
    let mut data = json!({});

    write_path(&mut data, "types.string", json!("text"));
    write_path(&mut data, "types.number", json!(42));
    write_path(&mut data, "types.float", json!(3.14));
    write_path(&mut data, "types.bool", json!(true));
    write_path(&mut data, "types.null", json!(null));
    write_path(&mut data, "types.array", json!([1, 2, 3]));
    write_path(&mut data, "types.object", json!({"key": "value"}));

    assert_eq!(read_path(&data, "types.string"), Some(&json!("text")));
    assert_eq!(read_path(&data, "types.number"), Some(&json!(42)));
    assert_eq!(read_path(&data, "types.float"), Some(&json!(3.14)));
    assert_eq!(read_path(&data, "types.bool"), Some(&json!(true)));
    assert_eq!(read_path(&data, "types.null"), Some(&json!(null)));
    assert_eq!(read_path(&data, "types.array"), Some(&json!([1, 2, 3])));
    assert_eq!(read_path(&data, "types.object.key"), Some(&json!("value")));
}

#[test]
fn test_realistic_user_data_scenario() {
    let mut data = json!({});

    // Create user
    write_path(&mut data, "users.0.id", json!(1));
    write_path(&mut data, "users.0.name", json!("Alice"));
    write_path(&mut data, "users.0.email", json!("alice@example.com"));
    write_path(&mut data, "users.0.roles", json!(["admin", "editor"]));
    write_path(&mut data, "users.0.profile.bio", json!("Software engineer"));
    write_path(&mut data, "users.0.profile.location", json!("NYC"));

    // Verify structure
    assert_eq!(read_path(&data, "users.0.id"), Some(&json!(1)));
    assert_eq!(
        read_path(&data, "users.0.roles"),
        Some(&json!(["admin", "editor"]))
    );
    assert_eq!(
        read_path(&data, "users.0.profile.bio"),
        Some(&json!("Software engineer"))
    );

    // Update email
    write_path(&mut data, "users.0.email", json!("alice.new@example.com"));
    assert_eq!(
        read_path(&data, "users.0.email"),
        Some(&json!("alice.new@example.com"))
    );

    // Remove bio
    remove_path(&mut data, "users.0.profile.bio");
    assert_eq!(read_path(&data, "users.0.profile.bio"), None);
    assert_eq!(
        read_path(&data, "users.0.profile.location"),
        Some(&json!("NYC"))
    );
}
