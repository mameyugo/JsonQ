//! Query engine for JSON collections
//!
//! This module provides MongoDB-style querying and fluent query building
//! for filtering, sorting, and projecting JSON data.
//!
//! # Query Types
//!
//! - **MongoDB-style matching**: `{"age": {"$gte": 18}, "role": "admin"}`
//! - **Fluent queries**: Filter → Sort → Paginate → Project
//!
//! # Examples
//!
//! ## MongoDB-style Matching
//!
//! ```rust
//! use jsonq::query::matches;
//! use serde_json::json;
//!
//! let item = json!({"name": "Alice", "age": 30, "role": "admin"});
//! let condition = json!({"age": {"$gte": 18}, "role": "admin"});
//!
//! assert!(matches(&item, &condition));
//! ```
//!
//! ## Fluent Query
//!
//! ```rust
//! use jsonq::query::execute_query;
//! use serde_json::json;
//!
//! let collection = vec![
//!     json!({"name": "Alice", "age": 30}),
//!     json!({"name": "Bob", "age": 25}),
//! ];
//!
//! let query = json!({
//!     "where": [{"field": "age", "op": ">=", "value": 25}],
//!     "order_by": {"field": "age", "direction": "desc"},
//!     "limit": 10
//! });
//!
//! let results = execute_query(&collection, &query);
//! ```

pub mod error;
pub mod executor;
mod fluent;
mod matcher;
mod operators;
pub mod optimizer;
pub mod path;
pub mod regex_safe;
pub mod vector;
pub mod sql;

pub use fluent::execute_query;
pub use matcher::matches;
pub use operators::{apply_operator, check_logical_operator};
