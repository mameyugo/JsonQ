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
//! ```rust
//! use jsonq::store::StoreInner;
//! use serde_json::json;
//!
//! let store = StoreInner::new("/path/to/data.json".to_string());
//! store.write(&json!({"key": "value"}));
//! let data = store.read().unwrap();
//! ```

pub mod options;
pub mod cache;
pub mod transaction;
pub mod index_store;
pub mod locking;
pub mod inner;

pub use options::StoreOpts;
pub use cache::CachedData;
pub use inner::StoreInner;
pub use index_store::IndexStore;
pub use transaction::TransactionState;
pub use locking::LockGuard;
