# 🚀 JsonQ - High-Performance JSON Storage Engine for PHP

[![Build Status](https://github.com/mameyugo/JsonQ/actions/workflows/build.yml/badge.svg)](https://github.com/mameyugo/JsonQ/actions)
[![PHP Version](https://img.shields.io/badge/PHP-8.1%20%7C%208.2%20%7C%208.3%20%7C%208.4-blue)](https://www.php.net/)
[![License](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)
[![Version](https://img.shields.io/badge/version-0.2.2-orange)](https://github.com/mameyugo/JsonQ/releases)

> **A blazing-fast, feature-rich JSON database engine built in Rust, designed as a drop-in replacement for simple databases and configuration storage.**

JsonQ transforms JSON files into a powerful, queryable database with **MongoDB-style queries**, **JSONPath support**, **ACID transactions**, and **automatic indexing** - all while being **2-20x faster** than vanilla PHP.

---

## ⚡ Why JsonQ?

### **The Problem with Vanilla PHP**

```php
// ❌ Vanilla PHP: Slow, unsafe, no queries
$data = json_decode(file_get_contents('data.json'), true);  // Load ENTIRE file
$data['users'][] = $newUser;                                // Modify in memory
file_put_contents('data.json', json_encode($data));         // ⚠️ Not crash-safe

// Finding users older than 18 requires manual loops
$adults = array_filter($data['users'], fn($u) => $u['age'] >= 18);

// No transactions - corruption risk if script crashes
// No indexes - O(n) searches every time
// No memory optimization - 500MB file = 500MB RAM
```

### **The JsonQ Solution**

```php
// ✅ JsonQ: Fast, safe, powerful queries
$store = new JsonQ\Store('data.json');
$store->push('users', $newUser);                            // ✅ Atomic, crash-safe

// MongoDB-style queries with ReDoS protection
$adults = $store->find('users', ['age' => ['$gte' => 18]]);

// ACID transactions
$store->beginTransaction();
$store->set('balance', 1000);
$store->set('status', 'active');
$store->commit();  // ✅ All-or-nothing

// O(1) indexed lookups
$store->createIndex('users', 'email');
$user = $store->findOne('users', ['email' => 'alice@example.com']);  // Instant!

// Memory-efficient: 500MB file uses <50MB RAM
```

---

## 🎯 Key Features

### 🔥 **Performance**
- **2-20x faster** than vanilla PHP for most operations
- **SIMD-accelerated** JSON parsing (uses simd-json, Rust port of simdjson)
- **Memory-mapped I/O** - handle gigabyte files with minimal RAM
- **Smart caching** - Arc-based with automatic invalidation
- **18% memory reduction** via key deduplication

### 💾 **Storage & I/O**
- **Atomic writes** - crash-safe with temp file + rename pattern
- **File locking** - multi-process safe (PHP-FPM, workers)
- **Compression** - gzip/zstd support for storage efficiency
- **Stream I/O** - write 100MB files with <10MB RAM
- **JSONL support** - append-only logs without reparsing

### 🔍 **Query Engine**
- **MongoDB-style queries** - 17 operators ($eq, $gt, $in, $regex, $exists...)
- **JSONPath** - full support including slices and multi-key selection
- **Fluent API** - chainable query builder
- **Safe regex** - ReDoS protection with backtracking limits
- **Full-text search** - case-insensitive substring matching

### 🗂️ **Indexing & Aggregation**
- **Hash indexes** - O(1) equality lookups
- **Compound indexes** - multi-field indexing
- **Aggregations** - sum, avg, min, max, count
- **Group by** - aggregate by field values
- **Auto-invalidation** - indexes update on writes

### 🔒 **Data Integrity**
- **ACID transactions** - begin/commit/rollback
- **JSON Schema validation** - enforce data contracts
- **UTF-8 SIMD validation** - 13 GB/s validation speed
- **Type safety** - Rust's type system prevents bugs

### 🛡️ **Security**
- **Path depth limits** - prevent deeply nested attacks
- **File size limits** - configurable max file size
- **Extension whitelist** - only allow .json files
- **Base path restriction** - sandbox file access
- **ReDoS protection** - regex timeout enforcement

---

## 📊 Performance Benchmarks

### **Real-World Performance vs Vanilla PHP**

| Operation | Vanilla PHP | JsonQ | Improvement |
|-----------|-------------|-------|-------------|
| **Read (cached)** | 115 ms | 58 ms | **2.0x faster** ✅ |
| **Write (atomic)** | 11 ms | 39 ms | 3.5x slower ⚠️ |
| **Find (regex)** | 140 ms | 64 ms | **2.2x faster** ✅ |
| **Aggregation** | 375 ms | 21 ms | **18x faster** 🔥 |
| **Indexed lookup** | 150 ms | 5 ms | **30x faster** 🚀 |
| **Memory (10K records)** | 3.0 MB | 2.5 MB | **18% less** ✅ |

*Benchmarks: 100K records (10MB), PHP 8.3, Ubuntu 24.04*

### **Why is JsonQ faster?**

1. **SIMD Acceleration** - Uses AVX2/SSE instructions for blazing JSON parsing
2. **Rust Performance** - Zero-cost abstractions, no garbage collection overhead
3. **Smart Caching** - Parse once, reuse forever (until file changes)
4. **Memory-Mapped I/O** - OS-level caching, no PHP memory overhead
5. **Compiled Regex** - Rust regex engine outperforms PHP PCRE in loops

### **The Write Trade-off**

JsonQ writes are **3.5x slower** than vanilla PHP because we prioritize **data integrity**:

| Feature | Vanilla PHP | JsonQ |
|---------|-------------|-------|
| Atomic writes | ❌ No | ✅ Yes (temp + rename) |
| Crash safety | ❌ Risk of corruption | ✅ All-or-nothing |
| Consistent state | ⚠️ Partial writes possible | ✅ Never corrupted |
| Power failure safety | ❌ Data loss | ✅ Data survives |

> **Real-world impact:** Most apps are **read-heavy** (100+ reads per write). Even with 3x slower writes, JsonQ is **2-3x faster overall** for typical workloads.

---

## 🚀 Installation

### **Option 1: Via Debian Package (Ubuntu/Debian)**

```bash
# Add JsonQ repository
curl -fsSL https://mameyugo.github.io/JsonQ/KEY.gpg | sudo gpg --dearmor -o /usr/share/keyrings/jsonq-archive-keyring.gpg
echo "deb [signed-by=/usr/share/keyrings/jsonq-archive-keyring.gpg] https://mameyugo.github.io/JsonQ/debian stable main" | sudo tee /etc/apt/sources.list.d/jsonq.list

# Install
sudo apt update
sudo apt install php-jsonq

# Verify
php -m | grep jsonq
php -r "echo jsonq_version();"  # Should output: 0.2.2
```

### **Option 2: Via PIE (All Platforms)**

```bash
# Install PIE
curl -sL https://github.com/php/pie/releases/latest/download/pie.phar -o pie.phar
chmod +x pie.phar
sudo mv pie.phar /usr/local/bin/pie

# Install JsonQ
pie install mameyugo/jsonq

# Enable in php.ini
echo "extension=jsonq" | sudo tee -a $(php --ini | grep "Loaded Configuration" | sed -e "s|.*:\s*||")
```

### **Option 3: From Source**

```bash
# Prerequisites
sudo apt install php8.3-dev libclang-dev cargo

# Build
git clone https://github.com/mameyugo/JsonQ.git
cd JsonQ
cargo build --release

# Install
sudo cp target/release/libjsonq.so $(php-config --extension-dir)/jsonq.so
echo "extension=jsonq.so" | sudo tee /etc/php/8.3/cli/conf.d/20-jsonq.ini

# Verify
php -m | grep jsonq
```

**Platform-specific notes:**
- **macOS:** `brew install php rust llvm`, set `LIBCLANG_PATH=$(brew --prefix llvm)/lib`
- **Windows:** Download pre-built DLL from [Releases](https://github.com/mameyugo/JsonQ/releases)

---

## 📖 Quick Start

### **Basic CRUD**

```php
use JsonQ\Store;

$store = new Store('database.json');  // Creates file if it doesn't exist

// Create
$store->set('users.0', [
    'name' => 'Alice',
    'email' => 'alice@example.com',
    'age' => 30
]);

// Read
$name = $store->get('users.0.name');  // "Alice"
$exists = $store->has('users.0.email');  // true
$count = $store->count('users');  // 1

// Update
$store->set('users.0.age', 31);
$store->increment('users.0.login_count');

// Delete
$store->remove('users.0.temporary_token');

// Array operations
$store->push('users', $newUser);  // Append
$store->merge('config', ['theme' => 'dark']);  // Deep merge
```

### **MongoDB-Style Queries**

```php
// Simple equality
$admins = $store->find('users', ['role' => 'admin']);

// Comparison operators
$adults = $store->find('users', ['age' => ['$gte' => 18]]);
$youngAdults = $store->find('users', [
    'age' => ['$gte' => 18, '$lt' => 30]
]);

// Array operators
$premiumUsers = $store->find('users', [
    'plan' => ['$in' => ['premium', 'enterprise']]
]);

// Existence checks
$verified = $store->find('users', ['email_verified_at' => ['$exists' => true]]);

// Regex (with ReDoS protection)
$gmailUsers = $store->find('users', [
    'email' => ['$regex' => '@gmail\.com$']
]);

// Logical operators
$activeAdmins = $store->find('users', [
    '$and' => [
        ['role' => 'admin'],
        ['status' => 'active']
    ]
]);

// Type checks
$store->find('products', ['price' => ['$type' => 'number']]);

// String operators
$store->find('users', ['name' => ['$startsWith' => 'John']]);
$store->find('products', ['title' => ['$contains' => 'laptop']]);

// Size checks
$store->find('users', ['tags' => ['$size' => 3]]);  // Exactly 3 tags
```

**Supported operators:**
- **Comparison:** `$eq`, `$ne`, `$gt`, `$gte`, `$lt`, `$lte`
- **Array:** `$in`, `$nin`, `$size`
- **Existence:** `$exists`
- **Type:** `$type`
- **String:** `$regex`, `$contains`, `$startsWith`, `$endsWith`
- **Logical:** `$and`, `$or`

### **JSONPath Queries**

```php
// Access nested data
$results = jsonq_query_node('data.json', '$.users[*].name');
// ["Alice", "Bob", "Charlie"]

// Array slicing
$first10 = jsonq_query_node('data.json', '$.items[0:10]');
$last5 = jsonq_query_node('data.json', '$.items[-5:]');
$everyOther = jsonq_query_node('data.json', '$.items[::2]');

// Multi-key selection
$userInfo = jsonq_query_node('data.json', '$.users[0]["name","email","age"]');
// {"name": "Alice", "email": "alice@...", "age": 30}

// Filters
$adults = jsonq_query_node('data.json', '$.users[?(@.age >= 18)]');
$admins = jsonq_query_node('data.json', '$.users[?(@.role == "admin")]');

// Recursive descent
$allPrices = jsonq_query_node('data.json', '$..price');
```

### **Indexing for Performance**

```php
// Create index for O(1) lookups
$store->createIndex('users', 'email');

// Now this is instant (no full scan)
$user = $store->findOne('users', ['email' => 'alice@example.com']);

// Compound indexes
$store->createIndex('orders', 'user_id');
$store->createIndex('orders', 'status');

// Queries on indexed fields are 30x faster
$userOrders = $store->find('orders', ['user_id' => 123]);
$pending = $store->find('orders', ['status' => 'pending']);
```

### **Transactions (ACID)**

```php
// Begin transaction
$store->beginTransaction();

try {
    // Multiple operations
    $store->set('accounts.checking', 1000);
    $store->set('accounts.savings', 5000);
    $store->increment('transaction_count');
    
    // Commit (atomic)
    $store->commit();
} catch (Exception $e) {
    // Rollback on error
    $store->rollback();
}

// Example: Money transfer
$store->beginTransaction();
$store->decrement('accounts.alice.balance', 100);
$store->increment('accounts.bob.balance', 100);
$store->commit();  // Both succeed or both fail
```

### **Aggregations**

```php
// Sum, average, min, max, count
$totalSales = $store->aggregate('orders', 'amount', 'sum');
$avgAge = $store->aggregate('users', 'age', 'avg');
$oldestUser = $store->aggregate('users', 'age', 'max');
$userCount = $store->aggregate('users', 'id', 'count');

// Group by
$salesByMonth = $store->groupBy('orders', 'month');
// ["2024-01" => [...], "2024-02" => [...]]

// Pluck specific fields
$emails = $store->pluck('users', ['email']);
// ["alice@...", "bob@...", "charlie@..."]

$nameAndEmail = $store->pluck('users', ['name', 'email']);
// [["name" => "Alice", "email" => "..."], ...]
```

### **Stream I/O (Memory-Efficient)**

```php
// Write 100MB file with <10MB RAM usage
jsonq_write_to_file('huge_dataset.json', 'output.json', pretty: false);

// Pretty-print for humans
jsonq_write_to_file('data.json', 'readable.json', pretty: true);

// Memory stats
$stats = jsonq_memory_stats('large_file.json');
echo "Unique keys: {$stats['unique_keys']}\n";
echo "Memory saved: {$stats['memory_saved_percent']}%\n";
```

### **JSONL Support (Append-Only Logs)**

```php
// Append events without reparsing entire file
jsonq_append_jsonl('events.jsonl', json_encode([
    'event' => 'user_login',
    'user_id' => 123,
    'timestamp' => time()
]));

// Read all events lazily
$events = jsonq_read_jsonl('events.jsonl');
foreach ($events as $eventJson) {
    $event = json_decode($eventJson, true);
    processEvent($event);
}
```

### **JSON Schema Validation**

```php
// Define schema
$userSchema = [
    'type' => 'object',
    'required' => ['name', 'email', 'age'],
    'properties' => [
        'name' => ['type' => 'string', 'minLength' => 1],
        'email' => ['type' => 'string', 'format' => 'email'],
        'age' => ['type' => 'integer', 'min' => 0, 'max' => 150]
    ],
    'additionalProperties' => false
];

// Validate single document
$result = $store->validate('users.0', $userSchema);
if (!$result['valid']) {
    foreach ($result['errors'] as $error) {
        echo "{$error['path']}: {$error['error']}\n";
    }
}

// Validate entire collection
$result = $store->validateCollection('users', $userSchema);
echo "Valid: {$result['valid_items']} / {$result['total_items']}\n";

// Set schema for automatic validation
$store->setSchema('users', $userSchema);
$store->push('users', $invalidUser);  // Throws exception if invalid
```

### **Compression**

```php
// Enable compression for storage efficiency
$store->setOption('compression', 'zstd');  // or 'gzip'
$store->set('large_data', $bigArray);

// Files are automatically compressed/decompressed
// ~3-5x smaller on disk for repetitive data

// Disable compression
$store->setOption('compression', 'none');
```

### **Configuration & Security**

```php
// Set maximum file size (prevent abuse)
jsonq_set_max_file_size('100M');  // Default: 100MB

// Restrict allowed extensions
jsonq_set_allowed_extensions('json,db');

// Sandbox to base path
jsonq_set_base_path('/var/www/data');
// Now only files under /var/www/data are accessible

// Get current config
$config = jsonq_get_config();
print_r($config);
```

---

## 🆚 Comparisons

### **vs Vanilla PHP (`json_decode`/`json_encode`)**

| Feature | Vanilla PHP | JsonQ |
|---------|-------------|-------|
| **Performance** | Baseline | 2-20x faster |
| **Memory usage** | 100% | 82% (-18%) |
| **Crash safety** | ❌ No | ✅ Yes |
| **Queries** | Manual loops | MongoDB-style |
| **Indexes** | ❌ No | ✅ O(1) lookups |
| **Transactions** | ❌ No | ✅ ACID |
| **File locking** | ❌ No | ✅ Multi-process safe |
| **Compression** | ❌ No | ✅ gzip/zstd |

**Use JsonQ when:** You need performance, safety, or advanced queries  
**Use vanilla PHP when:** Extremely simple one-time JSON parsing

---

### **vs pecl-jsonpath**

| Feature | pecl-jsonpath | JsonQ |
|---------|---------------|-------|
| **JSONPath syntax** | ✅ Full | ✅ Full |
| **MongoDB queries** | ❌ No | ✅ 17 operators |
| **Persistence** | ❌ No | ✅ Yes |
| **Indexing** | ❌ No | ✅ Yes |
| **Transactions** | ❌ No | ✅ Yes |
| **Stream I/O** | ❌ No | ✅ Yes |
| **ReDoS protection** | ❌ No | ✅ Yes |

**Use pecl-jsonpath when:** Only need JSONPath queries in-memory  
**Use JsonQ when:** Need JSONPath + persistence + queries + safety

---

### **vs simdjson-plus-php-ext**

| Feature | simdjson-plus | JsonQ |
|---------|---------------|-------|
| **Purpose** | JSON parser | JSON database |
| **Parsing speed** | 4x vs PHP | 2x vs PHP |
| **Encoding speed** | 2.5x vs PHP | 1.8x vs PHP |
| **Persistence** | ❌ No | ✅ Yes |
| **Queries** | ❌ No | ✅ Yes |
| **Indexing** | ❌ No | ✅ Yes |
| **Use together** | ✅ Yes - complementary! |

**Use simdjson-plus when:** Need fastest possible parsing in-memory  
**Use JsonQ when:** Need database features + queries  
**Use both when:** Parse APIs with simdjson, store with JsonQ

---

### **vs SQLite**

| Feature | SQLite | JsonQ |
|---------|--------|-------|
| **Setup** | Create schema | None - just use JSON |
| **Migrations** | Required | Not needed |
| **Schema changes** | ALTER TABLE | Just add fields |
| **Nested data** | ❌ Complex | ✅ Native |
| **JSON queries** | ⚠️ Limited | ✅ Full MongoDB-style |
| **File format** | Binary | Human-readable JSON |

**Use SQLite when:** Need SQL, complex relations, >1GB data  
**Use JsonQ when:** Prototyping, config, simple data, <100MB

---

## 📚 Advanced Examples

### **E-commerce Order System**

```php
$store = new Store('orders.json');

// Create indexes
$store->createIndex('orders', 'user_id');
$store->createIndex('orders', 'status');

// Add order with transaction
$store->beginTransaction();
$orderId = uniqid('order_');
$store->set("orders.$orderId", [
    'user_id' => 123,
    'items' => [...],
    'total' => 99.99,
    'status' => 'pending',
    'created_at' => time()
]);
$store->increment('stats.total_orders');
$store->commit();

// Find user's orders (O(1) with index)
$userOrders = $store->find('orders', ['user_id' => 123]);

// Find pending orders
$pending = $store->find('orders', ['status' => 'pending']);

// Get sales stats
$totalRevenue = $store->aggregate('orders', 'total', 'sum');
$avgOrderValue = $store->aggregate('orders', 'total', 'avg');

// Find high-value orders
$bigOrders = $store->find('orders', [
    'total' => ['$gte' => 500],
    'status' => ['$in' => ['pending', 'processing']]
]);
```

### **User Session Manager**

```php
$sessions = new Store('sessions.json');
$sessions->setOption('compression', 'zstd');  // Compress old sessions

// Store session
$sessionId = bin2hex(random_bytes(16));
$sessions->set("sessions.$sessionId", [
    'user_id' => 123,
    'ip' => $_SERVER['REMOTE_ADDR'],
    'created_at' => time(),
    'expires_at' => time() + 3600,
    'data' => $_SESSION
]);

// Get active sessions
$active = $sessions->find('sessions', [
    'expires_at' => ['$gt' => time()]
]);

// Cleanup expired sessions
$expired = $sessions->find('sessions', [
    'expires_at' => ['$lt' => time()]
]);
foreach ($expired as $id => $session) {
    $sessions->remove("sessions.$id");
}

// Get sessions by user
$sessions->createIndex('sessions', 'user_id');
$userSessions = $sessions->find('sessions', ['user_id' => 123]);
```

### **Analytics Event Logger**

```php
// Append-only log for high-performance writes
$eventFile = 'events_' . date('Y-m-d') . '.jsonl';

// Log event (sub-millisecond writes)
jsonq_append_jsonl($eventFile, json_encode([
    'event' => 'page_view',
    'url' => $_SERVER['REQUEST_URI'],
    'user_id' => $userId ?? null,
    'timestamp' => microtime(true),
    'user_agent' => $_SERVER['HTTP_USER_AGENT']
]));

// Process events later (lazy reading, minimal memory)
$events = jsonq_read_jsonl($eventFile);
$pageViews = [];
foreach ($events as $eventJson) {
    $event = json_decode($eventJson, true);
    if ($event['event'] === 'page_view') {
        $pageViews[] = $event;
    }
}

// Aggregate daily stats
$uniqueUsers = count(array_unique(array_column($pageViews, 'user_id')));
$popularPages = array_count_values(array_column($pageViews, 'url'));
```

### **Configuration Manager**

```php
$config = new Store('config.json');

// Set defaults
$defaults = [
    'app' => [
        'name' => 'MyApp',
        'version' => '1.0.0',
        'debug' => false
    ],
    'database' => [
        'host' => 'localhost',
        'port' => 3306
    ],
    'features' => [
        'analytics' => true,
        'beta_features' => false
    ]
];
$config->merge('', $defaults);

// Override with environment-specific config
$config->merge('app', ['debug' => $_ENV['APP_DEBUG'] ?? false]);

// Feature flags
if ($config->get('features.beta_features')) {
    enableBetaFeatures();
}

// Atomic config updates
$config->beginTransaction();
$config->set('app.version', '1.1.0');
$config->set('features.new_ui', true);
$config->commit();

// Validate config schema
$result = $config->validate('database', [
    'type' => 'object',
    'required' => ['host', 'port'],
    'properties' => [
        'host' => ['type' => 'string'],
        'port' => ['type' => 'integer', 'min' => 1, 'max' => 65535]
    ]
]);
```

---

## 🛠️ API Reference

### **Store Class**

#### **CRUD Operations**
```php
$store->get(string $path): mixed
$store->set(string $path, mixed $value): bool
$store->has(string $path): bool
$store->remove(string $path): bool
$store->count(string $path): int
$store->keys(string $path): array
$store->push(string $path, mixed $value): bool
$store->merge(string $path, array $value): bool
$store->increment(string $path, int $amount = 1): bool
$store->decrement(string $path, int $amount = 1): bool
```

#### **Queries**
```php
$store->find(string $collection, array $conditions): array
$store->findOne(string $collection, array $conditions): ?array
$store->search(string $collection, string $keyword): array
$store->executeQuery(string $collection, array $query): array
```

#### **Aggregations**
```php
$store->aggregate(string $path, string $field, string $func): float
$store->groupBy(string $collection, string $field): array
$store->pluck(string $collection, array $fields): array
```

#### **Indexing**
```php
$store->createIndex(string $collection, string $field): bool
$store->dropIndex(string $collection, string $field): bool
$store->dropAllIndexes(): int
```

#### **Transactions**
```php
$store->beginTransaction(): bool
$store->commit(): bool
$store->rollback(): bool
$store->inTransaction(): bool
```

#### **Validation**
```php
$store->validate(string $path, array $schema): array
$store->validateCollection(string $collection, array $schema): array
$store->setSchema(string $collection, array $schema): bool
```

#### **Options**
```php
$store->setOption(string $key, mixed $value): bool
$store->getMetrics(): array
```

### **Global Functions**

```php
// Version
jsonq_version(): string

// Configuration
jsonq_get_config(): array
jsonq_set_max_file_size(string $size): bool
jsonq_set_allowed_extensions(string $extensions): bool
jsonq_set_base_path(string $path): bool
jsonq_clear_base_path(): bool

// Stream I/O
jsonq_write_to_file(string $path, string $output, bool $pretty): bool
jsonq_append_jsonl(string $path, string $record): bool
jsonq_read_jsonl(string $path): array

// Memory
jsonq_memory_stats(string $path): array

// JSONPath
jsonq_query_node(string $path, string $query): array
```

---

## 🏗️ Architecture

### **How JsonQ Works**

```
┌─────────────────────────────────────────────────────────┐
│                    PHP Application                       │
│  $store = new JsonQ\Store('data.json')                  │
└────────────────────┬────────────────────────────────────┘
                     │
                     │ ext-php-rs FFI
                     ▼
┌─────────────────────────────────────────────────────────┐
│                   Rust Core Engine                       │
│                                                          │
│  ┌────────────┐  ┌─────────────┐  ┌──────────────┐    │
│  │ Query      │  │ Transaction │  │ Index        │    │
│  │ Engine     │  │ Manager     │  │ Manager      │    │
│  │ (MongoDB)  │  │ (ACID)      │  │ (O(1) hash)  │    │
│  └────────────┘  └─────────────┘  └──────────────┘    │
│                                                          │
│  ┌────────────┐  ┌─────────────┐  ┌──────────────┐    │
│  │ Parser     │  │ Cache       │  │ Validation   │    │
│  │ (simd-json)│  │ (Arc+mtime) │  │ (Schema+UTF8)│    │
│  └────────────┘  └─────────────┘  └──────────────┘    │
│                                                          │
│  ┌────────────┐  ┌─────────────┐  ┌──────────────┐    │
│  │ File I/O   │  │ Compression │  │ Security     │    │
│  │ (mmap+lock)│  │ (gzip/zstd) │  │ (limits)     │    │
│  └────────────┘  └─────────────┘  └──────────────┘    │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
              ┌──────────────┐
              │  data.json   │
              │ (On Disk)    │
              └──────────────┘
```

### **Key Components**

1. **Parser Layer** - simd-json for SIMD-accelerated parsing
2. **Cache Layer** - Arc-based smart caching with mtime tracking
3. **Query Engine** - MongoDB-style + JSONPath query execution
4. **Index Manager** - Hash-based O(1) equality lookups
5. **Transaction Layer** - ACID guarantees with rollback
6. **Storage Layer** - Memory-mapped I/O with file locking
7. **Security Layer** - Path/size validation, ReDoS protection

---

## 🤝 Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) before submitting PRs.

### **Development Setup**

```bash
git clone https://github.com/mameyugo/JsonQ.git
cd JsonQ

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build
cargo build --release

# Run tests
cargo test
php tests/run_tests.php
```

### **Running Benchmarks**

```bash
php examples/benchmark_v2.php
```

---

## 📄 License

JsonQ is open-source software licensed under the [MIT License](LICENSE).

---

## 🙏 Acknowledgments

- **simd-json** - Rust port of simdjson for blazing fast parsing
- **ext-php-rs** - Safe Rust-PHP FFI bindings
- **serde** - Serialization framework
- The Rust and PHP communities

---

## 📞 Support

- **Issues:** [GitHub Issues](https://github.com/mameyugo/JsonQ/issues)
- **Discussions:** [GitHub Discussions](https://github.com/mameyugo/JsonQ/discussions)
- **Email:** info@mameyugo.com

---

## 🗺️ Roadmap

### **v0.3.0** (Q2 2026)
- [ ] JOIN operations across collections
- [ ] Multi-stage aggregation pipelines
- [ ] Full-text search with stemming
- [ ] Query optimizer improvements

### **v0.4.0** (Q3 2026)
- [ ] Replication support
- [ ] Watch API for change streams
- [ ] GraphQL-like query DSL
- [ ] Built-in migrations

### **v1.0.0** (Q4 2026)
- [ ] Production-ready stable API
- [ ] Complete test coverage (>95%)
- [ ] Comprehensive documentation
- [ ] Performance benchmarks vs major DBs

---

<p align="center">
  <strong>Made with ❤️ in Rust | Powered by 🚀 simd-json</strong><br>
  <sub>Give it a ⭐ if JsonQ saved you time!</sub>
</p>
