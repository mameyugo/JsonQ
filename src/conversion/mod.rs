//! Type conversion between PHP and Rust
//!
//! This module handles bidirectional conversion between PHP's Zval types
//! and Rust's serde_json::Value types, ensuring type safety and proper
//! memory management across the FFI boundary.
//!
//! # Examples
//!
//! ```rust,no_run
//! use jsonq::conversion::{zval_to_value, value_to_zval};
//! use serde_json::json;
//!
//! // PHP → Rust
//! // let php_value = /* some Zval from PHP */;
//! // let rust_value = zval_to_value(&php_value);
//!
//! // Rust → PHP
//! let data = json!({"name": "Alice", "age": 30});
//! let zval = value_to_zval(&data);
//! ```
//!
//! # Type Mapping
//!
//! | PHP Type          | Rust Type (serde_json::Value) |
//! |-------------------|-------------------------------|
//! | `null`            | `Value::Null`                 |
//! | `bool`            | `Value::Bool`                 |
//! | `int`             | `Value::Number` (i64)         |
//! | `float`           | `Value::Number` (f64)         |
//! | `string`          | `Value::String`               |
//! | Sequential array  | `Value::Array`                |
//! | Associative array | `Value::Object`               |
//!
//! # Safety
//!
//! - Strings are always duplicated to prevent use-after-free
//! - Arrays are properly detected as sequential or associative
//! - Numeric conversions handle overflow gracefully

mod value_to_zval;
mod zval_to_value;

pub use value_to_zval::value_to_zval;
pub use zval_to_value::{ht_to_value, zval_to_value};
