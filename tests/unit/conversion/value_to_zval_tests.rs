//! Tests for value_to_zval conversion
//!
//! Tests conversion from serde_json::Value to PHP Zval

use jsonq::conversion::value_to_zval;
use serde_json::json;

#[test]
fn test_null_conversion() {
    let val = json!(null);
    let zval = value_to_zval(&val);
    assert!(zval.is_null());
}

#[test]
fn test_bool_true() {
    let val = json!(true);
    let zval = value_to_zval(&val);
    assert!(zval.is_bool());
    assert_eq!(zval.bool().unwrap(), true);
}

#[test]
fn test_bool_false() {
    let val = json!(false);
    let zval = value_to_zval(&val);
    assert!(zval.is_bool());
    assert_eq!(zval.bool().unwrap(), false);
}

#[test]
fn test_integer_positive() {
    let val = json!(42);
    let zval = value_to_zval(&val);
    assert!(zval.is_long());
    assert_eq!(zval.long().unwrap(), 42);
}

#[test]
fn test_integer_negative() {
    let val = json!(-17);
    let zval = value_to_zval(&val);
    assert!(zval.is_long());
    assert_eq!(zval.long().unwrap(), -17);
}

#[test]
fn test_integer_zero() {
    let val = json!(0);
    let zval = value_to_zval(&val);
    assert!(zval.is_long());
    assert_eq!(zval.long().unwrap(), 0);
}

#[test]
fn test_float_conversion() {
    let val = json!(3.14);
    let zval = value_to_zval(&val);
    assert!(zval.is_double());
    let result = zval.double().unwrap();
    assert!((result - 3.14).abs() < 0.0001);
}

#[test]
fn test_float_negative() {
    let val = json!(-2.718);
    let zval = value_to_zval(&val);
    assert!(zval.is_double());
    let result = zval.double().unwrap();
    assert!((result - (-2.718)).abs() < 0.0001);
}

#[test]
fn test_string_simple() {
    let val = json!("Hello, World!");
    let zval = value_to_zval(&val);
    assert!(zval.is_string());
    assert_eq!(zval.str().unwrap(), "Hello, World!");
}

#[test]
fn test_string_empty() {
    let val = json!("");
    let zval = value_to_zval(&val);
    assert!(zval.is_string());
    assert_eq!(zval.str().unwrap(), "");
}

#[test]
fn test_string_unicode() {
    let val = json!("Hello 世界 🌍");
    let zval = value_to_zval(&val);
    assert!(zval.is_string());
    assert_eq!(zval.str().unwrap(), "Hello 世界 🌍");
}

#[test]
fn test_string_with_quotes() {
    let val = json!("She said \"Hello\"");
    let zval = value_to_zval(&val);
    assert!(zval.is_string());
    assert_eq!(zval.str().unwrap(), "She said \"Hello\"");
}

#[test]
fn test_string_with_newlines() {
    let val = json!("Line 1\nLine 2\nLine 3");
    let zval = value_to_zval(&val);
    assert!(zval.is_string());
    assert_eq!(zval.str().unwrap(), "Line 1\nLine 2\nLine 3");
}

#[test]
fn test_array_empty() {
    let val = json!([]);
    let zval = value_to_zval(&val);
    assert!(zval.is_array());
    let ht = zval.array().unwrap();
    assert_eq!(ht.len(), 0);
}

#[test]
fn test_array_simple() {
    let val = json!([1, 2, 3]);
    let zval = value_to_zval(&val);
    assert!(zval.is_array());
    let ht = zval.array().unwrap();
    assert_eq!(ht.len(), 3);
}

#[test]
fn test_array_mixed_types() {
    let val = json!([1, "two", true, null]);
    let zval = value_to_zval(&val);
    assert!(zval.is_array());
    let ht = zval.array().unwrap();
    assert_eq!(ht.len(), 4);
}

#[test]
fn test_object_empty() {
    let val = json!({});
    let zval = value_to_zval(&val);
    assert!(zval.is_array()); // Objects are hashtables in PHP
    let ht = zval.array().unwrap();
    assert_eq!(ht.len(), 0);
}

#[test]
fn test_object_simple() {
    let val = json!({"name": "Alice", "age": 30});
    let zval = value_to_zval(&val);
    assert!(zval.is_array());
    let ht = zval.array().unwrap();
    assert_eq!(ht.len(), 2);
}

#[test]
fn test_nested_array() {
    let val = json!([[1, 2], [3, 4]]);
    let zval = value_to_zval(&val);
    assert!(zval.is_array());
    let ht = zval.array().unwrap();
    assert_eq!(ht.len(), 2);
}

#[test]
fn test_nested_object() {
    let val = json!({
        "user": {
            "name": "Alice",
            "address": {
                "city": "NYC"
            }
        }
    });
    let zval = value_to_zval(&val);
    assert!(zval.is_array());
}

#[test]
fn test_deeply_nested_structure() {
    let val = json!({
        "level1": {
            "level2": {
                "level3": {
                    "data": "deep"
                }
            }
        }
    });
    let zval = value_to_zval(&val);
    assert!(zval.is_array());
}

#[test]
fn test_large_number() {
    let val = json!(u64::MAX);
    let zval = value_to_zval(&val);
    // Very large numbers become doubles
    assert!(zval.is_double() || zval.is_long());
}

#[test]
fn test_string_that_looks_like_number() {
    let val = json!("42");
    let zval = value_to_zval(&val);
    assert!(zval.is_string());
    assert_eq!(zval.str().unwrap(), "42");
}

#[test]
fn test_string_that_looks_like_bool() {
    let val = json!("true");
    let zval = value_to_zval(&val);
    assert!(zval.is_string());
    assert_eq!(zval.str().unwrap(), "true");
}
