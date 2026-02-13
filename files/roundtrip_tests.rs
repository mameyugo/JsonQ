//! Roundtrip conversion tests
//!
//! Tests that Value → Zval → Value preserves data correctly

use jsonq::conversion::{value_to_zval, zval_to_value};
use serde_json::json;

#[test]
fn roundtrip_null() {
    let original = json!(null);
    let zval = value_to_zval(&original);
    let result = zval_to_value(&zval);
    assert_eq!(original, result);
}

#[test]
fn roundtrip_bool_true() {
    let original = json!(true);
    let zval = value_to_zval(&original);
    let result = zval_to_value(&zval);
    assert_eq!(original, result);
}

#[test]
fn roundtrip_bool_false() {
    let original = json!(false);
    let zval = value_to_zval(&original);
    let result = zval_to_value(&zval);
    assert_eq!(original, result);
}

#[test]
fn roundtrip_integer() {
    let original = json!(42);
    let zval = value_to_zval(&original);
    let result = zval_to_value(&zval);
    assert_eq!(original, result);
}

#[test]
fn roundtrip_negative_integer() {
    let original = json!(-17);
    let zval = value_to_zval(&original);
    let result = zval_to_value(&zval);
    assert_eq!(original, result);
}

#[test]
fn roundtrip_float() {
    let original = json!(3.14);
    let zval = value_to_zval(&original);
    let result = zval_to_value(&zval);
    
    // Use approximate comparison for floats
    let orig_f = original.as_f64().unwrap();
    let result_f = result.as_f64().unwrap();
    assert!((orig_f - result_f).abs() < 1e-10);
}

#[test]
fn roundtrip_string() {
    let original = json!("Hello, World!");
    let zval = value_to_zval(&original);
    let result = zval_to_value(&zval);
    assert_eq!(original, result);
}

#[test]
fn roundtrip_empty_string() {
    let original = json!("");
    let zval = value_to_zval(&original);
    let result = zval_to_value(&zval);
    assert_eq!(original, result);
}

#[test]
fn roundtrip_unicode() {
    let original = json!("Hello 世界 🌍");
    let zval = value_to_zval(&original);
    let result = zval_to_value(&zval);
    assert_eq!(original, result);
}

#[test]
fn roundtrip_array_simple() {
    let original = json!([1, 2, 3]);
    let zval = value_to_zval(&original);
    let result = zval_to_value(&zval);
    assert_eq!(original, result);
}

#[test]
fn roundtrip_array_mixed() {
    let original = json!([null, true, 42, "text"]);
    let zval = value_to_zval(&original);
    let result = zval_to_value(&zval);
    assert_eq!(original, result);
}

#[test]
fn roundtrip_object_simple() {
    let original = json!({"name": "Alice", "age": 30});
    let zval = value_to_zval(&original);
    let result = zval_to_value(&zval);
    assert_eq!(original, result);
}

#[test]
fn roundtrip_nested_object() {
    let original = json!({
        "user": {
            "name": "Alice",
            "scores": [95, 87, 92]
        }
    });
    let zval = value_to_zval(&original);
    let result = zval_to_value(&zval);
    assert_eq!(original, result);
}

#[test]
fn roundtrip_deeply_nested() {
    let original = json!({
        "level1": {
            "level2": {
                "level3": {
                    "level4": {
                        "data": "deep",
                        "values": [1, 2, 3]
                    }
                }
            }
        }
    });
    let zval = value_to_zval(&original);
    let result = zval_to_value(&zval);
    assert_eq!(original, result);
}

#[test]
fn roundtrip_complex_structure() {
    let original = json!({
        "users": [
            {"id": 1, "name": "Alice"},
            {"id": 2, "name": "Bob"}
        ],
        "meta": {
            "total": 2,
            "active": true
        }
    });
    let zval = value_to_zval(&original);
    let result = zval_to_value(&zval);
    assert_eq!(original, result);
}

#[test]
fn roundtrip_large_array() {
    let original = json!((0..100).collect::<Vec<i32>>());
    let zval = value_to_zval(&original);
    let result = zval_to_value(&zval);
    assert_eq!(original, result);
}

#[test]
fn roundtrip_special_characters() {
    let original = json!({
        "quotes": "She said \"Hello\"",
        "newlines": "Line1\nLine2",
        "tabs": "Col1\tCol2",
        "unicode": "café ☕"
    });
    let zval = value_to_zval(&original);
    let result = zval_to_value(&zval);
    assert_eq!(original, result);
}

#[test]
fn roundtrip_numeric_strings() {
    let original = json!({
        "string_number": "42",
        "string_bool": "true",
        "string_null": "null"
    });
    let zval = value_to_zval(&original);
    let result = zval_to_value(&zval);
    assert_eq!(original, result);
}
