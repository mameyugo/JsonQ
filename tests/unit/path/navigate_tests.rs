//! Tests for navigation utilities

use jsonq::path::navigate::{split_path_key, is_array_index, path_depth};
use std::sync::Arc;

#[test]
fn test_split_simple_path() {
    let (parent, key) = split_path_key("name");
    assert_eq!(parent, "");
    assert_eq!(key, "name");
}

#[test]
fn test_split_nested_path() {
    let (parent, key) = split_path_key("user.name");
    assert_eq!(parent, "user");
    assert_eq!(key, "name");
}

#[test]
fn test_split_deep_path() {
    let (parent, key) = split_path_key("user.profile.settings.theme");
    assert_eq!(parent, "user.profile.settings");
    assert_eq!(key, "theme");
}

#[test]
fn test_split_empty_path() {
    let (parent, key) = split_path_key("");
    assert_eq!(parent, "");
    assert_eq!(key, "");
}

#[test]
fn test_split_with_dots_in_middle() {
    let (parent, key) = split_path_key("a.b.c.d.e");
    assert_eq!(parent, "a.b.c.d");
    assert_eq!(key, "e");
}

#[test]
fn test_is_array_index_valid_numbers() {
    assert!(is_array_index("0"));
    assert!(is_array_index("1"));
    assert!(is_array_index("10"));
    assert!(is_array_index("42"));
    assert!(is_array_index("100"));
    assert!(is_array_index("999"));
}

#[test]
fn test_is_array_index_invalid() {
    assert!(!is_array_index(""));
    assert!(!is_array_index("a"));
    assert!(!is_array_index("abc"));
    assert!(!is_array_index("1a"));
    assert!(!is_array_index("a1"));
    assert!(!is_array_index("key"));
}

#[test]
fn test_is_array_index_negative() {
    assert!(!is_array_index("-1"));
    assert!(!is_array_index("-10"));
}

#[test]
fn test_is_array_index_with_spaces() {
    assert!(!is_array_index(" 1"));
    assert!(!is_array_index("1 "));
    assert!(!is_array_index(" 1 "));
}

#[test]
fn test_path_depth_empty() {
    assert_eq!(path_depth(""), 0);
}

#[test]
fn test_path_depth_single() {
    assert_eq!(path_depth("key"), 1);
}

#[test]
fn test_path_depth_nested() {
    assert_eq!(path_depth("a.b"), 2);
    assert_eq!(path_depth("a.b.c"), 3);
    assert_eq!(path_depth("a.b.c.d"), 4);
}

#[test]
fn test_path_depth_complex() {
    assert_eq!(path_depth("user.profile.settings.privacy.level"), 5);
}

#[test]
fn test_path_depth_with_numbers() {
    assert_eq!(path_depth("items.0"), 2);
    assert_eq!(path_depth("users.0.name"), 3);
}
