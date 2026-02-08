# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2025-02-08

### Added

- **Core storage engine** — `JsonQ\\Store` PHP class backed by Rust
  - Memory-mapped file reads (`memmap2`) for zero-copy access
  - Atomic writes (tmp + fsync + rename) for crash safety
  - Arc-based mtime cache for hot-read performance
  - Direct Zval ↔ serde_json::Value conversion (no JSON string overhead)

- **CRUD operations**
  - `get()` / `set()` with dot-notation path access
  - `has()`, `count()`, `keys()` for inspection
  - `remove()`, `push()`, `merge()` for mutations
  - `increment()` / `decrement()` for atomic numeric updates

- **MongoDB-style query engine**
  - Comparison: `$eq`, `$ne`, `$gt`, `$gte`, `$lt`, `$lte`
  - Array: `$in`, `$nin`
  - String: `$contains`, `$startsWith`, `$endsWith`
  - Type: `$exists`, `$size`, `$type`
  - Logical: `$and`, `$or`, `$not`
  - `find()` and `findOne()` methods

- **Fluent query engine**
  - `executeQuery()` with `where`, `order_by`, `limit`, `offset`, `select`
  - Operators: `=`, `!=`, `>`, `>=`, `<`, `<=`, `in`, `not in`, `contains`, `starts_with`, `ends_with`, `between`

- **Aggregation**
  - `aggregate()`: sum, avg, min, max, count
  - `groupBy()` for field-based grouping
  - `pluck()` for field extraction

- **Schema validation** (JSON Schema subset)
  - Type checking, required fields, property validation
  - String constraints: minLength, maxLength, format (email, url, ipv4, date, uuid)
  - Number constraints: min, max
  - Array constraints: minItems, maxItems, uniqueItems, items
  - Enum validation
  - Conditional: if/then/else, oneOf, anyOf
  - `validate()` and `validateCollection()`

- **In-memory indexes**
  - `createIndex()` for O(1) equality lookups
  - `createCompoundIndex()` for multi-field indexes
  - `indexLookup()` for direct hash lookups
  - Automatic index usage in `find()` for simple equality
  - `listIndexes()`, `dropIndex()`, `dropAllIndexes()`

- **Utilities**
  - `stats()` for file and data metadata
  - `backup()` / `restore()` for data safety
  - `jsonq_version()` standalone function

- **Project infrastructure**
  - PHP stubs for IDE autocompletion
  - 67 integration tests
  - GitHub Actions CI pipeline
  - MIT license

[Unreleased]: https://github.com/mamel/JsonQ/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/mamel/JsonQ/releases/tag/v0.1.0
