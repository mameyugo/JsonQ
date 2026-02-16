//! Index management for fast lookups
//!
//! This module provides indexing functionality for JSON collections,
//! enabling O(1) lookups instead of O(n) scans.
//!
//! # Index Types
//!
//! - **Single-field indexes**: Index on one field (e.g., "email")
//! - **Compound indexes**: Index on multiple fields (e.g., ["city", "role"])
//!
//! # Examples
//!
//! ```rust
//! use jsonq::index::IndexBuilder;
//! use serde_json::json;
//!
//! let collection = vec![
//!     json!({"id": 1, "name": "Alice", "role": "admin"}),
//!     json!({"id": 2, "name": "Bob", "role": "user"}),
//! ];
//!
//! // Build single-field index
//! let index = IndexBuilder::new()
//!     .build_single(&collection, "role");
//!
//! // Lookup by role
//! let positions = index.get("admin").unwrap();
//! assert_eq!(positions, &vec![0]);
//! ```
//!
//! # Performance
//!
//! - Build time: O(n) where n = collection size
//! - Lookup time: O(1) average case
//! - Memory: O(n) for index storage

mod builder;
mod lookup;
mod compound;

pub use builder::IndexBuilder;
pub use lookup::SingleIndex;
pub use compound::CompoundIndex;
