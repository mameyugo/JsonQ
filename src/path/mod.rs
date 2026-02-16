//! JSON path navigation utilities
//!
//! This module provides functions for navigating and manipulating JSON structures
//! using dot-notation paths (e.g., "user.address.city").
//!
//! # Path Syntax
//!
//! - `.` separates nested keys: `"user.name"` → `data["user"]["name"]`
//! - Numbers access array indices: `"items.0"` → `data["items"][0]`
//! - Empty path `""` refers to root
//!
//! # Examples
//!
//! ```rust
//! use jsonq::path::{read_path, write_path};
//! use serde_json::json;
//!
//! let data = json!({"user": {"name": "Alice"}});
//!
//! // Read
//! let name = read_path(&data, "user.name");
//! assert_eq!(name, Some(&json!("Alice")));
//!
//! // Write
//! let mut data = json!({});
//! write_path(&mut data, "user.name", json!("Bob"));
//! ```

pub mod navigate;
pub mod read;
pub mod write;

pub use navigate::split_path_key;
pub use read::{read_nested, read_path, read_path_mut};
pub use write::{remove_path, write_path};
