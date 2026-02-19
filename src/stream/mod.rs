//! Streaming JSON reader for JsonQ
//!
//! Provides memory-efficient iteration over large JSON files using
//! JSON Pointer (RFC 6901) for navigation.
//!
//! # Architecture
//!
//! ```text
//! File → BufReader → StreamVisitor (Thread) → Channel → StreamReader → FilteredStream → PHPIterator
//! ```
//!
//! # Memory model
//!
//! Only one JSON item is held in memory at any time, regardless of file size.
//! A 1GB JSON file with 10M records uses ~2-5MB of memory during streaming.

pub mod filter;
pub mod pointer;
pub mod reader;

pub use filter::{FilteredStream, StreamFilter};
pub use pointer::JsonPointer;
pub use reader::StreamReader;
