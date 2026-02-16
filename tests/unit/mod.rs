//! Unit tests for JsonQ modules
//!
//! This directory contains isolated unit tests for each module.
//! Integration tests with PHP runtime are in tests/*.php
//!
//! NOTE: The 'conversion' module tests are EXCLUDED from cargo test
//! because they require a PHP runtime.

// mod conversion;
use std::sync::Arc; // ⚠️ Requires PHP runtime, causes SIGSEGV

mod store;
mod path;
mod utils;
mod validation;
mod index;
mod query;
