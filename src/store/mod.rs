//! Storage engine for JsonQ
//!
//! This module implements the core storage engine with:
//! - Atomic file writes with crash safety
//! - Memory-mapped reads for performance  
//! - Arc-based caching with mtime invalidation
//! - Transaction support (begin/commit/rollback)
//! - Single and compound indexes
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │         StoreInner (Engine)             │
//! ├─────────────────────────────────────────┤
//! │  Cache (Arc<Value> + mtime)             │
//! │  Indexes (HashMap<String, IndexStore>)  │
//! │  Options (pretty, fsync)                │
//! │  Transactions (in_transaction, tx_data) │
//! └─────────────────────────────────────────┘
//!           ↓
//! ┌─────────────────────────────────────────┐
//! │     File System (JSON + indexes)        │
//! │  data.json, data.json.tmp               │
//! └─────────────────────────────────────────┘
//! ```
//!
//! # Examples
//!
//! ```rust,no_run
//! use jsonq::store::StoreInner;
//! use serde_json::json;
//! use std::sync::Arc;
//!
//! let store = StoreInner::new("/path/to/data.json".to_string()).unwrap();
//! store.write(Arc::new(json!({"key": "value"}))).unwrap();
//! let data = store.read().unwrap();
//! ```

pub mod cache;
pub mod cleanup;
pub mod index_store;
pub mod inner;
pub mod locking;
pub mod options;
pub mod transaction;

pub use cache::CachedData;
pub use index_store::{IndexStore, VectorIndex, VectorEntry};
pub use inner::StoreInner;
pub use locking::LockGuard;
pub use options::StoreOpts;
pub use transaction::TransactionState;
