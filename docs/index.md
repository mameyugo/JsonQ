# JsonQ Documentation

<div align="center">

![JsonQ Logo](https://via.placeholder.com/150x150/4F46E5/FFFFFF?text=JsonQ)

**High-Performance JSON Storage Engine for PHP**

[![PHP 8.1+](https://img.shields.io/badge/PHP-8.1%2B-777BB4?logo=php)](https://php.net)
[![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange?logo=rust)](https://rust-lang.org)
[![License](https://img.shields.io/badge/License-PHP%203.01-blue)](../LICENSE)
[![Performance](https://img.shields.io/badge/Performance-10x%20faster-brightgreen)](advanced/benchmarks.md)

[Installation](getting-started/installation.md) •
[Quick Start](getting-started/quick-start.md) •
[API Reference](api/store-class.md) •
[Examples](examples/rest-api.md)

</div>

---

## 🎯 What is JsonQ?

JsonQ is a **PHP extension written in Rust** that provides a blazing-fast, file-based JSON storage engine. It combines the simplicity of JSON with MongoDB-style queries, giving you the power of a database without the complexity.

### Key Features

- ⚡ **10x Faster** than pure PHP JSON handling
- 🔍 **MongoDB-style Queries** with 17+ operators
- ✅ **JSON Schema Validation** for data integrity
- 📊 **Indexing** for O(1) lookups
- 🔒 **ACID Transactions** with rollback support
- 🚀 **SIMD-Accelerated** parsing with simd-json
- 💾 **File-based** - No server required
- 🛡️ **Thread-safe** with OS-level file locking

---

## 🚀 Quick Links

### Getting Started
- [Installation Guide](getting-started/installation.md) - Install JsonQ in 2 minutes
- [Quick Start Tutorial](getting-started/quick-start.md) - Your first app in 5 minutes

### Core Guides
- [Querying Data](guides/queries.md) - MongoDB-style & Fluent queries
- [Schema Validation](guides/schema-validation.md) - Data integrity
- [Indexing](guides/indexing.md) - Performance optimization
- [Transactions](guides/transactions.md) - ACID guarantees

### API Reference
- [JsonQ\Store Class](api/store-class.md) - Complete API

### Real-World Examples
- [REST API](examples/rest-api.md) - Build an API with JsonQ

### Production
- [Deployment Guide](deployment/production.md) - Go to production

---

## ⚡ Why JsonQ?

### Simple Yet Powerful

```php
// Create a store
$store = new JsonQ\Store('data.json');

// Write data
$store->set('users', [
    ['id' => 1, 'name' => 'Alice', 'role' => 'admin', 'age' => 30],
    ['id' => 2, 'name' => 'Bob', 'role' => 'user', 'age' => 25],
]);

// Query with MongoDB-style syntax
$admins = $store->find('users', ['role' => 'admin']);

// Advanced queries
$results = $store->find('users', [
    'age' => ['$gte' => 25],
    'role' => ['$in' => ['admin', 'moderator']]
]);

// Aggregation
$avgAge = $store->aggregate('users', 'age', 'avg'); // 27.5
```

### Performance That Matters

| Operation | JsonQ | Pure PHP | Speedup |
|-----------|-------|----------|---------|
| Parse 10K records | 0.8ms | 5.2ms | **6.5x** |
| Find (indexed) | 0.1ms | 12.5ms | **125x** |
| Complex query | 2.1ms | 18.3ms | **8.7x** |
| Aggregation | 0.9ms | 8.7ms | **9.7x** |

[See detailed benchmarks →](advanced/benchmarks.md)

### Battle-Tested Features

✅ **MongoDB-Compatible**
- 17+ query operators
- Fluent query builder
- Aggregation functions
- Compound indexes

✅ **Production-Ready**
- ACID transactions
- File locking
- Crash recovery
- Schema validation

✅ **Developer-Friendly**
- Simple API
- Great documentation
- Full IDE support
- Rich examples

---

## 📚 Learn by Example

### Basic CRUD

```php
$store = new JsonQ\Store('app.json');

// Create
$store->set('user.profile', ['name' => 'Alice', 'age' => 30]);

// Read
$profile = $store->get('user.profile');

// Update
$store->set('user.profile.age', 31);

// Delete
$store->remove('user.profile.age');
```

### Powerful Queries

```php
// Find users over 18 who are admins
$results = $store->find('users', [
    '$and' => [
        ['age' => ['$gt' => 18]],
        ['role' => 'admin']
    ]
]);

// Fluent query with sorting and pagination
$users = $store->executeQuery('users', [
    'where' => [
        ['field' => 'active', 'op' => '=', 'value' => true]
    ],
    'order_by' => ['field' => 'created_at', 'direction' => 'desc'],
    'limit' => 20,
    'offset' => 40
]);
```

### Transactions

```php
try {
    $store->beginTransaction();
    $store->set('accounts.A.balance', 900);
    $store->set('accounts.B.balance', 1100);
    $store->commit();
} catch (Exception $e) {
    $store->rollback();
}
```

---

## 🎓 Next Steps

### New to JsonQ?

1. [Install JsonQ](getting-started/installation.md) (2 minutes)
2. [Try the Quick Start](getting-started/quick-start.md) (5 minutes)
3. [Build a REST API](examples/rest-api.md) (15 minutes)

### Ready for Production?

1. [Deployment Checklist](deployment/production.md)
2. [Backup Strategy](deployment/production.md#backup-strategy)

---

## 💬 Community & Support

- **Issues**: [GitHub Issues](https://github.com/mameyugo/JsonQ/issues)
- **Discussions**: [GitHub Discussions](https://github.com/mameyugo/JsonQ/discussions)
- **Email**: info@mameyugo.com
- **Documentation**: You're here! 📚

---

## 🤝 Contributing

We welcome contributions! See [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines.

---

## 📄 License

JsonQ is licensed under [The PHP License, version 3.01](../LICENSE).

---

<div align="center">

**Made with ❤️ in Rust | Powered by 🚀 SIMD**

[⭐ Star on GitHub](https://github.com/mameyugo/JsonQ) •
[📖 Read the Docs](getting-started/installation.md) •
[🐛 Report Bug](https://github.com/mameyugo/JsonQ/issues)

</div>
