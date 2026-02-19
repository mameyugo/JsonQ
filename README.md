<div align="center">

# 🚀 JsonQ

**High-Performance JSON Storage Engine for PHP**

[![PHP Version](https://img.shields.io/badge/PHP-8.1%20|%208.2%20|%208.3%20|%208.4-777BB4?logo=php)](https://php.net)
[![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange?logo=rust)](https://rust-lang.org)
[![License](https://img.shields.io/badge/License-PHP%203.01-blue)](LICENSE)
[![Performance](https://img.shields.io/badge/Performance-2--10x%20faster-brightgreen)](docs/BENCHMARKS.md)

*MongoDB-style queries | JSON Schema validation | ACID transactions | 10x performance*

[Features](#-features) • [Quick Start](#-quick-start) • [Documentation](#-documentation) • [Benchmarks](#-performance) • [Contributing](#-contributing)

Version: **0.4.1**

</div>

---

## 📊 What is JsonQ?

JsonQ is a **blazing-fast PHP extension** written in Rust that provides a file-based JSON storage engine with MongoDB-style queries, schema validation, indexing, and transactions.

### Why JsonQ?

- 🚀 **2-10x faster** than pure PHP JSON handling
- 💾 **File-based** - No server required, zero configuration
- 🔍 **MongoDB-style queries** - Familiar syntax with 17+ operators
- ✅ **Schema validation** - JSON Schema subset support
- 🔒 **ACID transactions** - Atomic operations with rollback
- 📊 **Indexing** - Hash-based O(1) lookups
- 🛡️ **Thread-safe** - OS-level file locking with fs2
- ⚡ **SIMD-accelerated** - Fast parsing with simd-json

---

## ✨ Features

### Core Storage
- ✅ **CRUD Operations** with dot-notation path access
- ✅ **Atomic Writes** (tmp + fsync + rename) for crash safety
- ✅ **Memory-mapped I/O** for zero-copy reads
- ✅ **Arc-based Caching** with mtime invalidation
- ✅ **Compression Support** (Gzip, Zstd) - *v0.3.2*
- ✅ **Native Streaming** (v0.4.0) - Iterate gigabytes of data with low memory
- ✅ **Stream Filtering** - Apply MongoDB-style queries while streaming

### Query Engine
- ✅ **MongoDB-style Matching**: `$eq`, `$ne`, `$gt`, `$gte`, `$lt`, `$lte`, `$in`, `$nin`
- ✅ **String Operators**: `$contains`, `$startsWith`, `$endsWith`, `$regex`
- ✅ **Logical Operators**: `$and`, `$or`, `$not`, `$nor`
- ✅ **Array Operators**: `$size`, `$all`, `$elemMatch`
- ✅ **Type Checking**: `$exists`, `$type`
- ✅ **Field Projection**: `select()` for whitelisting fields - *v0.3.2*
- ✅ **Query Optimizer**: Intelligent index selection - *v0.3.2*

### Fluent Query Builder
- ✅ **Chainable Methods**: `where()`, `orWhere()`, `orderBy()`, `limit()`, `skip()`
- ✅ **Filtering**: Comparison, string, array operators
- ✅ **Sorting**: Ascending/descending on any field
- ✅ **Pagination**: `skip` + `limit` support

### Aggregation & Analysis
- ✅ **Functions**: `sum()`, `avg()`, `min()`, `max()`, `count()`
- ✅ **Grouping**: `groupBy()` with field-based grouping
- ✅ **Field Extraction**: `pluck()` for column extraction

### Validation & Schema
- ✅ **JSON Schema Validation** (subset)
- ✅ **Type Constraints**: string, number, boolean, array, object
- ✅ **String Formats**: email, URL, IPv4, date, UUID
- ✅ **Number Constraints**: min, max, multipleOf
- ✅ **Array Constraints**: minItems, maxItems, uniqueItems
- ✅ **Required Fields** and enum validation
- ✅ **Conditional Logic**: if/then/else, oneOf, anyOf

### Indexing
- ✅ **Single Field Indexes**: O(1) equality lookups
- ✅ **Compound Indexes**: Multi-field indexing
- ✅ **Hash-based**: MD5 hashing for fast lookups
- ✅ **Auto-optimization**: Automatic use in `find()` queries

### Transactions
- ✅ **ACID Guarantees**: Atomic, Consistent, Isolated, Durable
- ✅ **Begin/Commit/Rollback**: Full transaction support
- ✅ **Isolation**: Transaction-local changes until commit

### Advanced Features *(v0.3.0)*
- ✅ **Safe Regex** - ReDoS protection with backtracking limits
- ✅ **Metrics API** - Real-time observability (reads, writes, cache stats, latency)
- ✅ **Compression** - Transparent Gzip/Zstd support
- ✅ **Query Optimizer** - Intelligent index selection for complex queries

### Collection Methods *(NEW - v0.3.2)*
- ✅ **`select(fields)`** - Project specific fields (whitelist)
- ✅ **`except(fields)`** - Exclude specific fields (blacklist)
- ✅ **`column(field)`** - Extract values from single column
- ✅ **`chunk(size)`** - Split results into groups
- ✅ **`implode(field, separator)`** - Join column values into string
- ✅ **`keys(path)`** - Get object keys
- ✅ **`values(path)`** - Get object values
- ✅ **`toJson(pretty)`** - Serialize results to JSON string

> **Legend**: ✅ Confirmed | ⏳ Pending verification

---

## 🚀 Quick Start

### Installation

#### Via APT (Debian/Ubuntu)

```bash
# Add repository
curl -fsSL https://mameyugo.github.io/JsonQ/jsonq-archive-keyring.gpg | sudo gpg --dearmor -o /usr/share/keyrings/jsonq-archive-keyring.gpg
echo "deb [signed-by=/usr/share/keyrings/jsonq-archive-keyring.gpg] https://mameyugo.github.io/JsonQ stable main" | sudo tee /etc/apt/sources.list.d/jsonq.list

# Install
sudo apt update
sudo apt install php8.3-jsonq

# Enable extension
php -m | grep jsonq
```

#### Via Curl (Quick Install)

```bash
curl -fsSL https://raw.githubusercontent.com/mameyugo/JsonQ/main/scripts/install.sh | sudo bash
```

#### Build from Source

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone https://github.com/mameyugo/JsonQ.git
cd JsonQ
cargo build --release

# Install
sudo cp target/release/libjsonq.so $(php-config --extension-dir)/jsonq.so
echo "extension=jsonq.so" | sudo tee /etc/php/8.3/mods-available/jsonq.ini
sudo phpenmod jsonq
```

### Hello World

```php
<?php
use JsonQ\Store;

// Create store
$store = new Store('data.json');

// Write data
$store->set('users', [
    ['id' => 1, 'name' => 'Alice', 'role' => 'admin', 'age' => 30],
    ['id' => 2, 'name' => 'Bob', 'role' => 'user', 'age' => 25],
    ['id' => 3, 'name' => 'Charlie', 'role' => 'user', 'age' => 35],
]);

// Query with MongoDB-style syntax
$admins = $store->find('users', ['role' => 'admin']);
// Returns: [['id' => 1, 'name' => 'Alice', ...]]

// Fluent query builder
$results = $store->executeQuery('users', [
    'where' => [
        ['field' => 'age', 'op' => '>=', 'value' => 25]
    ],
    'order_by' => ['field' => 'age', 'direction' => 'desc'],
    'limit' => 10
]);

// Aggregation
$avgAge = $store->aggregate('users', ['avg' => 'age']);
// Returns: ['avg' => 30.0]

echo "✅ JsonQ is working!\n";
```

---

## 📚 API Reference

### Basic Operations

#### CRUD

```php
// Create/Update
$store->set('user.profile', ['name' => 'Alice', 'age' => 30]);

// Read
$profile = $store->get('user.profile');
// Returns: ['name' => 'Alice', 'age' => 30]

// Check existence
$exists = $store->has('user.profile'); // true

// Delete
$store->remove('user.profile');

// Count elements
$count = $store->count('users'); // 3
```

#### Array Operations

```php
// Push to array
$store->push('users', ['id' => 4, 'name' => 'Dave']);

// Merge objects
$store->merge('config', ['debug' => true]);

// Increment/Decrement
$store->increment('stats.views'); // views++
$store->decrement('inventory.stock', 5); // stock -= 5
```

---

### MongoDB-style Queries

#### Comparison Operators

```php
// Greater than
$adults = $store->find('users', ['age' => ['$gt' => 18]]);

// Range queries
$midAge = $store->find('users', [
    'age' => ['$gte' => 25, '$lte' => 35]
]);

// Not equal
$active = $store->find('users', ['status' => ['$ne' => 'deleted']]);
```

#### Array Operators

```php
// In array
$roles = $store->find('users', [
    'role' => ['$in' => ['admin', 'moderator']]
]);

// Not in array
$regular = $store->find('users', [
    'role' => ['$nin' => ['admin', 'guest']]
]);

// Array size
$teams = $store->find('projects', [
    'members' => ['$size' => 5]
]);
```

#### String Operators

```php
// Contains substring
$gmailUsers = $store->find('users', [
    'email' => ['$contains' => '@gmail.com']
]);

// Starts with
$adminUsers = $store->find('users', [
    'username' => ['$startsWith' => 'admin_']
]);

// Ends with
$txtFiles = $store->find('files', [
    'name' => ['$endsWith' => '.txt']
]);

// Regex matching (with ReDoS protection)
$phoneNumbers = $store->find('contacts', [
    'phone' => ['$regex' => '^\+1-\d{3}-\d{3}-\d{4}$']
]);
```

#### Logical Operators

```php
// AND (implicit)
$seniorAdmins = $store->find('users', [
    'role' => 'admin',
    'age' => ['$gte' => 30]
]);

// OR
$privileged = $store->find('users', [
    '$or' => [
        ['role' => 'admin'],
        ['role' => 'moderator']
    ]
]);

// NOT
$notGuests = $store->find('users', [
    '$not' => ['role' => 'guest']
]);

// Complex nested logic
$results = $store->find('users', [
    '$and' => [
        ['age' => ['$gte' => 18]],
        ['$or' => [
            ['role' => 'admin'],
            ['verified' => true]
        ]]
    ]
]);
```

---

### Fluent Query Builder

```php
$results = $store->executeQuery('products', [
    // Filtering
    'where' => [
        ['field' => 'price', 'op' => '>', 'value' => 100],
        ['field' => 'inStock', 'op' => '=', 'value' => true]
    ],
    
    // Sorting
    'order_by' => [
        'field' => 'price',
        'direction' => 'desc' // or 'asc'
    ],
    
    // Pagination
    'skip' => 10,   // Offset
    'limit' => 20,  // Max results
    
    // Projection (v0.3.0)
    'select' => ['name', 'price', 'category']
]);
```

**Available Operators:**
- Comparison: `=`, `!=`, `>`, `>=`, `<`, `<=`
- Array: `in`, `not in`
- String: `contains`, `startsWith`, `endsWith`
- Range: `between` (expects array `[min, max]`)

---

### Aggregation

```php
// Single aggregation
$total = $store->aggregate('orders', ['sum' => 'amount']);
// Returns: ['sum' => 15750.50]

// Multiple aggregations
$stats = $store->aggregate('products', [
    'sum' => 'price',
    'avg' => 'price',
    'min' => 'price',
    'max' => 'price',
    'count' => 'id'
]);
// Returns: ['sum' => 5000, 'avg' => 250, 'min' => 50, 'max' => 1000, 'count' => 20]

// Group by
$byCategory = $store->groupBy('products', 'category');
// Returns: ['electronics' => [...], 'books' => [...]]

// Extract column values
$names = $store->pluck('users', 'name');
// Returns: ['Alice', 'Bob', 'Charlie']
```

---

### Collection Methods *(NEW)*

```php
// Select specific fields (projection)
$results = $store->executeQuery('users', [
    'select' => ['name', 'email'] // Only return name and email
]);

// Extract column values
$emails = $store->column('users', 'email');
// Returns: ['alice@example.com', 'bob@example.com', ...]

// Split into chunks
$chunks = $store->chunk('users', 10);
// Returns: [[user1..10], [user11..20], ...]

// Join column values
$nameList = $store->implode('users', 'name', ', ');
// Returns: "Alice, Bob, Charlie, Dave"

// Get object keys
$keys = $store->keys('user.profile');
// Returns: ['name', 'email', 'age', 'verified']

// Get object values
$values = $store->values('user.profile');
// Returns: ['Alice', 'alice@example.com', 30, true]

// Serialize to JSON
$json = $store->toJson('users');
$prettyJson = $store->toJson('users', true); // Pretty-print
```

---

### Schema Validation

```php
// Define schema
$schema = [
    'type' => 'object',
    'required' => ['name', 'email', 'age'],
    'properties' => [
        'name' => [
            'type' => 'string',
            'minLength' => 2,
            'maxLength' => 50
        ],
        'email' => [
            'type' => 'string',
            'format' => 'email'
        ],
        'age' => [
            'type' => 'integer',
            'minimum' => 18,
            'maximum' => 120
        ],
        'role' => [
            'type' => 'string',
            'enum' => ['admin', 'user', 'guest']
        ]
    ]
];

// Validate single document
$user = ['name' => 'Alice', 'email' => 'alice@example.com', 'age' => 30];
$isValid = $store->validate($user, $schema);
// Returns: true

// Validate collection
$result = $store->validateCollection('users', $schema);
// Returns: ['valid' => true, 'errors' => []]
```

**Supported Constraints:**
- **Types**: string, number, integer, boolean, array, object, null
- **String**: minLength, maxLength, pattern, format (email, url, ipv4, date, uuid)
- **Number**: minimum, maximum, multipleOf
- **Array**: minItems, maxItems, uniqueItems, items
- **Object**: required, properties, additionalProperties
- **Enum**: Fixed set of allowed values
- **Conditional**: if/then/else, oneOf, anyOf

---

### Indexing

```php
// Create single-field index
$store->createIndex('users', 'email');

// Create compound index
$store->createCompoundIndex('orders', ['customerId', 'status']);

// Direct index lookup (O(1))
$user = $store->indexLookup('users', 'email', 'alice@example.com');

// List all indexes
$indexes = $store->listIndexes();
// Returns: ['users.email' => 'single', 'orders.customerId+status' => 'compound']

// Drop index
$store->dropIndex('users.email');

// Drop all indexes
$store->dropAllIndexes();
```

**Performance Impact:**
- Indexed `find()` queries: **~10x faster**
- Index memory overhead: **~2-5% of data size**
- Write penalty: **<5% slower** (hash computation)

---

### Transactions

```php
try {
    // Begin transaction
    $store->begin();
    
    // Perform multiple operations
    $store->set('accounts.A', ['balance' => 900]);
    $store->set('accounts.B', ['balance' => 1100]);
    $store->set('logs', ['transfer' => 100]);
    
    // Commit atomically
    $store->commit();
    
    echo "✅ Transaction committed\n";
    
} catch (Exception $e) {
    // Rollback on error
    $store->rollback();
    echo "❌ Transaction rolled back: " . $e->getMessage() . "\n";
}
```

**Guarantees:**
- **Atomicity**: All changes commit together or none
- **Consistency**: Schema validation enforced
- **Isolation**: Changes invisible until commit
- **Durability**: fsync ensures disk persistence

---

### Advanced Features *(v0.3.0)*

#### Compression

```php
// Enable Zstd compression (best compression ratio + speed)
$store->setOption('compression', 'zstd');

// Or use Gzip
$store->setOption('compression', 'gzip');

// Disable compression
$store->setOption('compression', 'none');

// Transparent decompression - reads work automatically
$data = $store->get('users');
```

**Compression Comparison:**
| Method | Ratio | Speed | Best For |
|--------|-------|-------|----------|
| **none** | 1.0x | Fastest | Small files, frequent writes |
| **gzip** | 2-3x | Fast | Good balance |
| **zstd** | 2.5-4x | Fastest | Large files, best compression |

#### Metrics & Observability

```php
// Get real-time metrics
$metrics = $store->getMetrics();

echo "Reads: " . $metrics['reads'] . "\n";
echo "Writes: " . $metrics['writes'] . "\n";
echo "Cache Hits: " . $metrics['cache_hits'] . "\n";
echo "Cache Misses: " . $metrics['cache_misses'] . "\n";
echo "Hit Rate: " . $metrics['cache_hit_rate'] . "%\n";
echo "Avg Latency: " . $metrics['avg_latency_ms'] . "ms\n";
```

**Tracked Metrics:**
- Read/write counters
- Cache hit/miss ratio
- Average read latency
- Last operation timestamp

---

### Backup & Restore

```php
// Create backup
$store->backup('/backups/data-' . date('Y-m-d') . '.json');

// Restore from backup
$store->restore('/backups/data-2024-01-15.json');

// Get file stats
$stats = $store->stats();
echo "File size: " . $stats['file_size'] . " bytes\n";
echo "Last modified: " . $stats['modified_at'] . "\n";
```

---

### Global Configuration

```php
// Get current config
$config = jsonq_get_config();

// Set max file size
jsonq_set_max_file_size('100M'); // or '1G', '500K'

// Set allowed extensions
jsonq_set_allowed_extensions('json,db');

// Set base path (security restriction)
jsonq_set_base_path('/var/www/data');

// Clear base path restriction
jsonq_clear_base_path();
```

---

## 🏎️ Performance

### Benchmarks (vs Pure PHP)

| Operation | JsonQ | Pure PHP | Speedup |
|-----------|-------|----------|---------|
| **Parse JSON** | 0.8ms | 5.2ms | **6.5x faster** |
| **Find (no index)** | 1.2ms | 12.5ms | **10.4x faster** |
| **Find (indexed)** | 0.1ms | 12.5ms | **125x faster** |
| **Aggregate (sum)** | 0.9ms | 8.7ms | **9.7x faster** |
| **Complex query** | 2.1ms | 18.3ms | **8.7x faster** |
| **Write + fsync** | 1.5ms | 3.2ms | **2.1x faster** |

*Benchmark environment: PHP 8.3, 10K documents, Intel i7-12700K*

### Optimization Tips

1. **Use Indexes**: 10-100x speedup for equality lookups
2. **Enable Compression**: 50-75% disk space reduction
3. **Batch Writes**: Use transactions for multiple operations
4. **Monitor Metrics**: Track cache hit rate and optimize queries
5. **Project Fields**: Use `select` to return only needed data

---

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────┐
│             PHP Application Layer               │
└─────────────────────────────────────────────────┘
                      ↕ FFI (ext-php-rs)
┌─────────────────────────────────────────────────┐
│              Rust Core (JsonQ)                  │
├─────────────────────────────────────────────────┤
│  1. Parser        - simd-json (SIMD parsing)    │
│  2. Cache         - Arc + mtime invalidation    │
│  3. Query Engine  - MongoDB + JSONPath          │
│  4. Index Manager - Hash-based O(1) lookups     │
│  5. Transactions  - ACID with rollback          │
│  6. Storage       - memmap2 + file locking      │
│  7. Security      - Path validation, ReDoS      │
└─────────────────────────────────────────────────┘
                      ↕ File System
┌─────────────────────────────────────────────────┐
│           data.json + indexes/                  │
└─────────────────────────────────────────────────┘
```

---

## 🤝 Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Development Setup

```bash
# Clone repository
git clone https://github.com/mameyugo/JsonQ.git
cd JsonQ

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build debug
cargo build

# Build release
cargo build --release

# Run Rust tests
cargo test

# Run PHP integration tests
php tests/run_tests.php

# Run benchmarks
php examples/benchmark_v2.php
```

### Project Structure

```
JsonQ/
├── src/                # Rust implementation
│   ├── conversion/     # PHP ↔ Rust FFI
│   ├── store/          # Storage engine
│   ├── query/          # Query execution
│   ├── index/          # Indexing system
│   └── validation/     # Schema validation
├── tests/
│   ├── integration/    # PHP integration tests
│   └── unit/           # Rust unit tests
├── stubs/              # PHP IDE stubs
└── docs/               # Documentation
```

---

## 📄 License

JsonQ is licensed under [The PHP License, version 3.01](LICENSE).

---

## 🙏 Acknowledgments

- **simd-json** - SIMD-accelerated JSON parsing
- **ext-php-rs** - Safe Rust-PHP FFI bindings
- **serde** - Serialization framework
- **MongoDB** - Query syntax inspiration
- The Rust and PHP communities

---

## 📞 Support

- **Issues**: [GitHub Issues](https://github.com/mameyugo/JsonQ/issues)
- **Discussions**: [GitHub Discussions](https://github.com/mameyugo/JsonQ/discussions)
- **Email**: info@mameyugo.com

---

## 🗺️ Roadmap

### v0.4.0 (Q2 2026)
- [ ] Complete collection methods (`except`, `column`, `chunk`, `implode`, `keys`, `values`, `toJson`)
- [ ] JSONPath full support
- [ ] Query optimizer improvements
- [ ] Multi-stage aggregation pipelines

### v0.5.0 (Q3 2026)
- [ ] JOIN operations across collections
- [ ] Full-text search with stemming
- [ ] Watch API for change streams
- [ ] GraphQL-like query DSL

### v1.0.0 (Q4 2026)
- [ ] Production-ready stable API
- [ ] >95% test coverage
- [ ] Comprehensive documentation
- [ ] Performance parity with MongoDB for common queries

---

<div align="center">

**Made with ❤️ in Rust | Powered by 🚀 SIMD**

[⭐ Star on GitHub](https://github.com/mameyugo/JsonQ) • [📖 Documentation](https://github.com/mameyugo/JsonQ/blob/main/docs/API.md) • [🐛 Report Bug](https://github.com/mameyugo/JsonQ/issues)

</div>
