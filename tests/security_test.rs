use jsonq::config::Config;
use jsonq::security::{validate_file_size, validate_path, validate_path_depth};
use jsonq::validation::validate;
use serde_json::json;
use std::fs;
use std::sync::Arc;
use tempfile::tempdir;

#[test]
fn test_config_singleton() {
    Config::init();
    Config::update(|cfg| {
        cfg.max_file_size = 12345;
    });

    let cfg = Config::get();
    assert_eq!(cfg.max_file_size, 12345);
}

#[test]
fn test_path_validation_extension() {
    Config::init();
    Config::update(|cfg| {
        cfg.allowed_extensions = vec!["json".to_string()];
    });

    // Valid
    assert!(validate_path("test.json").is_ok());

    // Invalid
    assert!(validate_path("test.txt").is_err());
    assert!(validate_path("test").is_err());
}

#[test]
fn test_path_validation_base_path() {
    Config::init();
    let dir = tempdir().unwrap();
    let base_path = dir.path().to_path_buf();

    Config::update(|cfg| {
        cfg.base_path = Some(base_path.clone());
        cfg.allowed_extensions = vec!["json".to_string()];
    });

    // Inside base path
    let inside = base_path.join("data.json");
    fs::write(&inside, "{}").unwrap();
    assert!(validate_path(inside.to_str().unwrap()).is_ok());

    // Outside base path
    assert!(validate_path("/etc/passwd.json").is_err());
    assert!(validate_path("../outside.json").is_err());
}

#[test]
fn test_file_size_validation() {
    Config::init();
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("large.json");

    // Create a 1KB file
    let data = "x".repeat(1024);
    fs::write(&file_path, data).unwrap();

    // Limit to 500 bytes
    Config::update(|cfg| {
        cfg.max_file_size = 500;
    });

    assert!(validate_file_size(&file_path).is_err());

    // Limit to 2KB
    Config::update(|cfg| {
        cfg.max_file_size = 2048;
    });

    assert!(validate_file_size(&file_path).is_ok());
}

#[test]
fn test_validation_depth_limit() {
    Config::init();
    Config::update(|cfg| {
        cfg.max_validation_depth = 3;
    });

    // Level 1
    let v1 = json!({"a": 1});
    let s1 = json!({"type": "object", "properties": {"a": {"type": "integer"}}});
    assert!(validate(&v1, &s1, "").is_empty());

    // Level 4 (Exceeds 3)
    let v4 = json!({"a": {"b": {"c": {"d": 1}}}});
    let s4 = json!({
        "type": "object",
        "properties": {
            "a": {
                "type": "object",
                "properties": {
                    "b": {
                        "type": "object",
                        "properties": {
                            "c": {
                                "type": "object",
                                "properties": {
                                    "d": {"type": "integer"}
                                }
                            }
                        }
                    }
                }
            }
        }
    });

    let errors = validate(&v4, &s4, "root");
    assert!(!errors.is_empty());
    assert!(errors[0]["error"]
        .as_str()
        .unwrap()
        .contains("depth 4 exceeds maximum allowed 3"));
}

#[test]
fn test_dot_notation_depth_limit() {
    Config::init();
    Config::update(|cfg| {
        cfg.max_path_depth = 2;
    });

    assert!(validate_path_depth("a.b").is_ok());
    assert!(validate_path_depth("a.b.c").is_err());
}
