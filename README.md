<p align="center">
  <h1 align="center">JsonQ</h1>
  <p align="center">
    <strong>High-performance JSON file storage engine for PHP, powered by Rust</strong>
  </p>
  <p align="center">
    <a href="https://github.com/mameyugo/JsonQ/actions"><img src="https://github.com/mameyugo/JsonQ/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
    <a href="https://packagist.org/packages/mameyugo/jsonq"><img src="https://img.shields.io/packagist/v/mameyugo/jsonq" alt="Version"></a>
    <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License"></a>
    <img src="https://img.shields.io/badge/PHP-8.1%2B-8892BF.svg" alt="PHP 8.1+">
    <img src="https://img.shields.io/badge/Rust-1.75%2B-DEA584.svg" alt="Rust 1.75+">
  </p>
</p>

---

**JsonQ** is a native PHP extension written in Rust that provides a complete JSON file storage engine with MongoDB-style queries, safe regex support, storage compression, real-time metrics, fluent query builder, aggregation, schema validation, and in-memory indexes — all without requiring a database server.

## Why JsonQ?

| Feature | `json_encode`/`file_put_contents` | SQLite | **JsonQ** |
|---------|-----------------------------------|--------|-----------|
| Setup | None | Minimal | None |
| Query engine | ❌ Manual loops | ✅ SQL | ✅ MongoDB + Fluent |
| **Performance (reads)** | Baseline | Fast | **2-4x faster** 🚀 |
| **Performance (aggregations)** | Baseline | Fast | **13-32x faster** 🔥 |
| Schema validation | ❌ | Partial | ✅ JSON Schema |
| Indexes | ❌ | ✅ | ✅ In-memory |
| Atomic writes | ❌ | ✅ | ✅ fsync + rename |
| Memory-mapped I/O | ❌ | ✅ | ✅ |
| Zero-copy cache | ❌ | ❌ | ✅ Arc-based |
| Safe Regex (ReDoS-safe) | ❌ | ❌ | ✅ |
| Storage Compression | ❌ | ❌ | ✅ Gzip / Zstd |
| Metrics/Observability | ❌ | ❌ | ✅ |

## Quick Start

```php
use JsonQ\\Store;

$store = new Store('/path/to/data.json');

// Write
$store->set('users', [
    ['name' => 'Alice', 'age' => 30, 'role' => 'admin'],
    ['name' => 'Bob',   'age' => 25, 'role' => 'user'],
]);

// MongoDB-style queries
$admins = $store->find('users', ['role' => 'admin']);
$young  = $store->find('users', ['age' => ['$lt' => 28]]);

// Fluent queries
$result = $store->executeQuery('users', [
    'where'    => [['field' => 'age', 'op' => '>=', 'value' => 25]],
    'order_by' => ['field' => 'name', 'direction' => 'asc'],
    'limit'    => 10,
]);

// Aggregation
$avg = $store->aggregate('users', 'age', 'avg');

// Indexes for O(1) lookups
$store->createIndex('users', 'role');
$admins = $store->indexLookup('users', 'role', 'admin');
```

## ⚡ Performance Highlights

JsonQ is **blazingly fast** for read-heavy workloads — exactly where most applications spend their time:

- 🚀 **2.1x faster** cached reads vs `json_decode` + `file_get_contents`
- 🔍 **3.6-4.2x faster** queries vs PHP's `array_filter`
- 📊 **13-32x faster** aggregations vs `array_sum` / `array_column`
- 🎯 **O(1) indexed lookups** with in-memory HashMap indexes

**Trade-off**: Writes are 1.8-4.2x slower than plain PHP because JsonQ uses atomic writes (`fsync` + `rename`) to guarantee crash safety and prevent data corruption. For read-heavy apps (90%+ reads), JsonQ is still **2-3x faster overall**.

👉 See [Performance](#performance) section for detailed benchmarks.

## Installation


### Requirements

- PHP 8.1 or later (with `php-dev` headers)
- Rust 1.75 or later
- `libclang-dev` (for bindgen)

### From Source

```bash
git clone https://github.com/mameyugo/JsonQ.git
cd JsonQ

# Install system dependencies (Ubuntu/Debian)
sudo apt-get install php8.3-dev libclang-dev

# Build
cargo build --release

# Install the extension
sudo cp target/release/libjsonq.so $(php-config --extension-dir)/jsonq.so
echo "extension=jsonq.so" | sudo tee /etc/php/8.3/cli/conf.d/20-JsonQ.ini

# Verify
php -m | grep JsonQ
php -r "echo jsonq_version();"
```

### macOS

```bash
brew install php rust llvm
export LIBCLANG_PATH=$(brew --prefix llvm)/lib
cargo build --release
sudo cp target/release/libjsonq.dylib $(php-config --extension-dir)/jsonq.so
```

## API Reference

### CRUD Operations

```php
$store = new JsonQ\\Store('data.json');  // Creates file if needed

// Read
$store->get('path.to.value');           // Dot-notation access
$store->has('path.to.value');           // Check existence
$store->count('users');                 // Array/object length
$store->keys('config');                 // Top-level keys

// Write
$store->set('path.to.value', $data);   // Set (creates intermediates)
$store->remove('path.to.value');       // Delete
$store->push('users', $newUser);       // Append to array
$store->merge('config', $overrides);   // Deep merge objects
$store->increment('stats.views');      // Increment number
$store->decrement('stats.stock', 5);   // Decrement by amount
```

### Query Engine

#### MongoDB-Style

```php
// Simple equality
$store->find('users', ['name' => 'Alice']);
$store->findOne('users', ['id' => 42]);

// Comparison operators
$store->find('users', ['age' => ['$gte' => 18, '$lt' => 65]]);

// Logical operators
$store->find('users', [
    '$or' => [
        ['role' => 'admin'],
        ['age' => ['$gte' => 30]],
    ]
]);

// String operators
$store->find('users', ['email' => ['$regex' => '@gmail\.com$']]);
$store->find('users', ['email' => ['$contains' => '@gmail']]);
$store->find('users', ['name' => ['$startsWith' => 'A']]);

// Array/type operators
$store->find('users', ['tags' => ['$size' => 3]]);
$store->find('users', ['age' => ['$type' => 'integer']]);
$store->find('users', ['email' => ['$exists' => true]]);
```

**Supported operators:** `$eq`, `$ne`, `$gt`, `$gte`, `$lt`, `$lte`, `$in`, `$nin`, `$regex`, `$contains`, `$startsWith`, `$endsWith`, `$exists`, `$size`, `$type`, `$and`, `$or`, `$not`

#### Fluent Queries

```php
$store->executeQuery('users', [
    'where'    => [
        ['field' => 'status', 'op' => '=', 'value' => 'active'],
        ['field' => 'age', 'op' => 'between', 'value' => [18, 65]],
    ],
    'order_by' => ['field' => 'created_at', 'direction' => 'desc'],
    'offset'   => 0,
    'limit'    => 20,
    'select'   => ['name', 'email'],
]);
```

**Supported operators:** `=`, `!=`, `<>`, `>`, `>=`, `<`, `<=`, `in`, `not in`, `contains`, `starts_with`, `ends_with`, `between`

### Aggregation

```php
$store->aggregate('orders', 'total', 'sum');    // Sum
$store->aggregate('users', 'age', 'avg');       // Average
$store->aggregate('products', 'price', 'min');  // Minimum
$store->aggregate('products', 'price', 'max');  // Maximum
$store->aggregate('users', 'id', 'count');      // Count

$store->groupBy('users', 'department');          // Group by field
$store->pluck('users', ['name', 'email']);       // Extract fields
```

### Indexes

```php
// Single field index — O(1) equality lookups
$store->createIndex('users', 'email');

// Compound index
$store->createCompoundIndex('users', ['city', 'status']);

// Direct index lookup (bypasses scan)
$results = $store->indexLookup('users', 'email', 'alice@example.com');

// Management
$store->listIndexes();              // Active indexes with stats
$store->dropIndex('users');         // Drop collection indexes
$store->dropAllIndexes();           // Drop all indexes
```

### Metrics & Observability

```php
// Get real-time operational statistics
$metrics = $store->getMetrics();

/**
 * $metrics = [
 *   'reads'          => 1250,   // Total read operations
 *   'writes'         => 450,    // Total write operations
 *   'cache_hits'     => 1100,   // Successful cache lookups
 *   'cache_misses'   => 150,    // Hard disk reads
 *   'avg_latency_ms' => 0.015   // Average read latency
 * ]
 */
```

### Schema Validation

```php
$result = $store->validate('user', [
    'type'       => 'object',
    'required'   => ['name', 'email', 'age'],
    'properties' => [
        'name'  => ['type' => 'string', 'minLength' => 1, 'maxLength' => 100],
        'email' => ['type' => 'string', 'format' => 'email'],
        'age'   => ['type' => 'integer', 'min' => 0, 'max' => 150],
    ],
]);
// $result = ['valid' => true, 'error_count' => 0, 'errors' => []]

// Validate entire collection
$result = $store->validateCollection('users', [
    'type'     => 'object',
    'required' => ['name', 'email'],
]);
// $result = ['valid' => false, 'total_items' => 100, 'invalid_items' => 3, ...]
```

**Supported validations:** `type`, `required`, `properties`, `minLength`, `maxLength`, `min`, `max`, `format` (email, url, ipv4, date, uuid), `enum`, `minItems`, `maxItems`, `uniqueItems`, `items`, `additionalProperties`, `nullable`, `oneOf`, `anyOf`, `if`/`then`/`else`

### Utilities

```php
$store->stats();                        // File size, keys, index count
$store->backup();                       // Auto-timestamped backup
$store->backup('/path/to/backup.json'); // Custom path
$store->restore('/path/to/backup.json');
```

### Options

```php
$store->setOption('pretty', true);      // Pretty-print JSON output
$store->setOption('fsync', true);       // Enable fsync for crash safety
$store->setOption('compression', 'zstd'); // "none", "gzip", "zstd"
$store->getOption('pretty');            // Read option value
```

### Transactions

```php
$store->beginTransaction();

$store->set('users.0.name', 'Updated');
$store->increment('stats.writes');
$store->push('log', ['action' => 'update']);

$store->commit();   // Single atomic write
// or $store->rollback();  // Discard all changes
```

### Batch Operations

```php
// Set many keys in one write (instead of N individual writes)
$store->setMany([
    'config.version' => '2.0',
    'config.debug'   => false,
    'stats.deploys'  => 42,
]);

// Remove many paths in one write
$store->removeMany(['temp.cache', 'old.data', 'deprecated']);
```

### Full-Text Search

```php
// Case-insensitive search across all string fields (including nested)
$results = $store->search('products', 'wireless');
```

### Import / Export

```php
$json = $store->toJson(true);     // Export as JSON string (pretty)
$store->fromJson($jsonString);    // Import from JSON string
$store->getAll();                  // Get entire data structure
$store->clear();                   // Reset to {}
```

## Architecture

```
┌─────────────────────────────┐
│        PHP Userland         │
│   new JsonQ\\Store($path)    │
└──────────┬──────────────────┘
           │ Native PHP class (ext-php-rs)
           │ Zero serialization overhead
┌──────────▼──────────────────┐
│      Rust Extension Core    │
│                             │
│  ┌─────────┐  ┌──────────┐ │
│  │  Cache   │  │  Indexes │ │
│  │ Arc<Data>│  │ HashMap  │ │
│  └────┬────┘  └────┬─────┘ │
│       │             │       │
│  ┌────▼─────────────▼────┐  │
│  │    Query Engine       │  │
│  │  MongoDB + Fluent     │  │
│  └───────────┬───────────┘  │
│              │              │
│  ┌───────────▼───────────┐  │
│  │   File I/O Layer      │  │
│  │  mmap read │ atomic   │  │
│  │  mtime     │ write    │  │
│  └───────────────────────┘  │
└─────────────────────────────┘
```

**Key design decisions:**

- **Memory-mapped reads** via `memmap2` for zero-copy file access on large files
- **Arc-based caching** with mtime invalidation — hot reads avoid all I/O
- **Atomic writes** — write to `.tmp`, `fsync`, then `rename` for crash safety
- **Direct Zval ↔ Value conversion** — no intermediate JSON string serialization
- **In-memory indexes** — HashMap-based for O(1) equality lookups

## Performance

JsonQ delivers **exceptional performance** for read-heavy workloads, queries, and aggregations — exactly where most applications spend their time. Here's how JsonQ compares to standard PHP JSON operations (`json_encode`/`json_decode` + `file_get_contents`/`file_put_contents`):

### Benchmark Results (Latest v0.2.0)

**Test Environment**: PHP 8.3, Ubuntu 24.04, tested with 100 / 1K / 10K records

#### 🚀 Where JsonQ Excels

| Operation | 100 records | 1K records | 10K records | Advantage |
|-----------|-------------|------------|-------------|-----------|
| **Read (cached)** | 0.058 ms | 0.522 ms | 6.063 ms | **2.0-2.2x faster** |
| **Find (scan)** | 0.036 ms | 0.293 ms | 3.791 ms | **3.6-4.6x faster** |
| **Find (indexed)** | 0.036 ms | 0.231 ms | 2.792 ms | **O(1) lookup** |
| **Complex queries** | 0.151 ms | 1.143 ms | 13.865 ms | **1.3-1.8x faster** |
| **Aggregations** | 0.013 ms | 0.032 ms | 0.821 ms | **11-38x faster** 🔥 |

**Why JsonQ dominates here**:
- **Zero-copy reads**: Arc-based caching eliminates repeated deserialization
- **Rust-native filtering**: Compiled code vs PHP's interpreted array operations
- **Optimized aggregations**: Direct numeric operations without array_sum/array_column overhead
- **Memory-mapped I/O**: Large files read without loading into memory
- **Smart indexes**: HashMap-based O(1) lookups for equality searches

#### ⚖️ Write Performance Trade-off

| Operation | 100 records | 1K records | 10K records | PHP Advantage |
|-----------|-------------|------------|-------------|---------------|
| **Write** | 0.454 ms | 4.609 ms | 39.174 ms | **3.6-4.1x faster** |

**Why PHP is faster at writes** (and why that's okay):

JsonQ intentionally prioritizes **data integrity** over raw write speed:

✅ **Atomic writes**: Write to `.tmp` → `fsync()` → `rename()` ensures crash safety  
✅ **No data loss**: Power failure during write? Your data is safe  
✅ **Consistent state**: Never see partial/corrupted JSON files  

Standard PHP's `file_put_contents()` is faster because it skips these safety guarantees. If your application crashes mid-write, you may lose data or corrupt your JSON file.

> **Real-world impact**: Most applications are read-heavy. Even if writes are 3x slower, if you do 100 reads for every 1 write, JsonQ is still **2-3x faster overall**.

### Performance Characteristics by Dataset Size

| Dataset Size | Use Case | JsonQ Performance |
|--------------|----------|-------------------|
| **100-1K records** | User sessions, configs, small caches | Sub-millisecond reads, ~1-5ms writes |
| **1K-10K records** | Product catalogs, user databases | ~0.5-6ms reads, ~5-40ms writes |
| **10K-100K records** | Analytics data, logs, large catalogs | ~6-60ms reads, ~40-400ms writes |

### When to Choose JsonQ

✅ **Perfect for**:
- Read-heavy applications (90%+ reads)
- Applications requiring queries, filters, or aggregations
- Scenarios where data integrity is critical
- Projects needing MongoDB-style queries without a database
- Rapid prototyping with production-ready performance

⚠️ **Consider alternatives if**:
- Your workload is write-heavy (50%+ writes)
- You only need simple key-value storage without queries
- You're optimizing for absolute minimum write latency

### Run Benchmarks Yourself

```bash
php examples/benchmark.php
```

The benchmark suite tests write, read, find, indexed lookup, complex queries, and aggregations across multiple dataset sizes.

## Testing

```bash
# Run PHP integration tests
php -d "extension=$(pwd)/target/release/libjsonq.so" tests/run_tests.php

# Run Rust unit tests
cargo test
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

MIT — see [LICENSE](LICENSE) for details.
