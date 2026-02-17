# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.1] - 2026-02-16

### Added

- **Collection Methods**
  - `except(fields)` - Exclude specific fields from results (blacklist).
  - `column(field)` - Extract values from a single column.
  - `chunk(size)` - Split results into chunks.
  - `implode(field, separator)` - Join column values into a string.
  - `keys(path)` - Get object keys at path.
  - `values(path)` - Get object values at path.
  - `toJson(pretty)` - Serialize results to JSON string.

### Changed

- `executeQuery` now supports `except` alongside `select` (mutually exclusive).

## [0.3.0] - 2026-02-16

### Added

- **Advanced JSONPath Support**
  - Recursive descent (`..`).
  - Wildcard operator (`*`).
  - Slice notation (`[start:end:step]`).
  - Multi-key selection (`['a','b']`).
  - Filter expressions (`[?(@.age > 18)]`).

- **Stream I/O**
  - `write_to_stream` for low-memory writing of large files.
  - `append_jsonl` for appending to JSON Lines files.
  - `read_jsonl_iter` for lazy reading of JSON Lines files.

- **Performance**
  - `simdutf` integration for ultra-fast UTF-8 validation.
  - Key deduplication (interning) for reduced memory usage.
  - Computed property exclusion in `jsonq_memory_stats`.

- **Developer Experience**
  - Visual error reporting (`^` marker) for query syntax errors.
  - `jsonq_version()` function.


## [0.2.0] - 2025-02-15

### Added

- **Safe Regex Execution**
  - Thread-safe regex cache with size and backtracking limits to prevent ReDoS attacks.
  - Integrated into `$regex` operator in `find()`.

- **Storage Compression**
  - Gzip and Zstd compression support.
  - Transparent decompression (auto-detects magic numbers).
  - Configurable via `setOption('compression', 'zstd')`.

- **Metrics & Observability**
  - Real-time tracking of reads, writes, cache hits/misses, and average latency.
  - Exposed via `JsonStore::getMetrics()`.

- **Query Optimizer**
  - Intelligent index selection based on query complexity and index availability.
  - Automatic optimization for multi-condition `find()` queries.

### Changed

- Updated `$regex` operator to support true regular expressions instead of simple substring matching.
- Refactored `JsonStore` option methods for better stability and return types.

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
  - The PHP License, version 3.01

[Unreleased]: https://github.com/mameyugo/JsonQ/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/mameyugo/JsonQ/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/mameyugo/JsonQ/releases/tag/v0.1.0
