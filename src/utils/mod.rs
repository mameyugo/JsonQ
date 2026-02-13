//! Utility functions for JSON manipulation
//!
//! This module provides helper functions for common JSON operations:
//! - Type conversion helpers
//! - Value key generation for indexing
//! - Deep search in JSON structures
//! - Deep merging of JSON values
//!
//! # Examples
//!
//! ```rust
//! use jsonq::utils::{as_u64, value_key, search_in_value, merge_values};
//! use serde_json::json;
//!
//! // Type conversion
//! let num = json!(42);
//! assert_eq!(as_u64(&num), Some(42));
//!
//! // Value key for indexing
//! let key = value_key(Some(&json!("Alice")));
//! assert_eq!(key, "Alice");
//!
//! // Deep search
//! let data = json!({"user": {"name": "Alice"}});
//! assert!(search_in_value(&data, "alice"));
//!
//! // Deep merge
//! let mut base = json!({"a": 1});
//! merge_values(&mut base, &json!({"b": 2}));
//! assert_eq!(base, json!({"a": 1, "b": 2}));
//! ```

mod conversion;
mod indexing;
mod search;
mod merge;

pub use conversion::as_u64;
pub use indexing::value_key;
pub use search::search_in_value;
pub use merge::merge_values;
