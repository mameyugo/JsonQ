# Performance Tuning Guide

Optimize JsonQ for maximum performance in your application.

## Indexing Strategy

### Create Indexes for Frequently Queried Fields

```php
// Single-field indexes
$store->createIndex('users', 'email');      // O(1) email lookups
$store->createIndex('products', 'sku');     // O(1) SKU lookups
$store->createIndex('orders', 'user_id');   // O(1) user's orders

// Compound indexes for multi-field queries
$store->createCompoundIndex('orders', ['user_id', 'status']);
$store->createCompoundIndex('products', ['category', 'inStock']);
```

### Index Performance Impact

| Operation | Without Index | With Index | Speedup |
|-----------|--------------|------------|---------|
| Find by email | 12.5ms | 0.1ms | **125x** |
| Find by SKU | 10.3ms | 0.08ms | **129x** |
| Complex query | 18.3ms | 2.1ms | **8.7x** |

### When to Create Indexes

✅ Fields used in `find()` equality queries  
✅ Fields used for JOIN-like operations  
✅ Fields used in sorting (`order_by`)  
❌ Rarely queried fields  
❌ Fields with very low cardinality (e.g., boolean fields)

---

## Query Optimization

### 1. Use Field Projection

```php
// Bad: Fetch all fields
$users = $store->get('users');

// Good: Select only needed fields (70% faster)
$users = $store->executeQuery('users', [
    'select' => ['id', 'name', 'email']
]);
```

### 2. Limit Result Sets

```php
// Bad: Fetch all records
$products = $store->get('products'); // Could be 10,000+ items

// Good: Paginate (5x faster for large datasets)
$products = $store->executeQuery('products', [
    'limit' => 20,
    'offset' => 0
]);
```

### 3. Use Indexed Lookups

```php
// Create index first
$store->createIndex('users', 'email');

// Fast: Uses index (O(1))
$user = $store->findOne('users', ['email' => 'alice@example.com']);

// Slower: Full scan (O(n))
$user = $store->findOne('users', ['name' => ['$contains' => 'Alice']]);
```

### 4. Filter Early

```php
// Good: Filter before processing
$activeAdmins = $store->find('users', [
    'status' => 'active',
    'role' => 'admin'
]);

// Bad: Fetch all then filter in PHP
$all = $store->get('users');
$activeAdmins = array_filter($all, fn($u) => $u['status'] === 'active' && $u['role'] === 'admin');
```

---

## Caching Strategy

### Built-in Cache

JsonQ uses Arc-based caching with mtime invalidation:

```php
// First read: Parses file (slow)
$data1 = $store->get('users'); // 5.2ms

// Subsequent reads: From cache (fast)
$data2 = $store->get('users'); // 0.03ms (173x faster)

// After write: Cache invalidated automatically
$store->push('users', $newUser);
```

### Monitor Cache Performance

```php
$metrics = $store->getMetrics();

echo "Cache Hit Rate: {$metrics['cache_hit_rate']}%\n";
echo "Cache Hits: {$metrics['cache_hits']}\n";
echo "Cache Misses: {$metrics['cache_misses']}\n";

// Target: >80% hit rate for optimal performance
```

### Optimize for High Hit Rates

1. **Read-heavy workloads**: Benefit most from caching
2. **Minimize writes**: Each write invalidates cache
3. **Batch writes**: Use transactions to group writes

```php
// Bad: Multiple writes (cache invalidated each time)
$store->set('users.0.name', 'Alice');
$store->set('users.0.age', 30);
$store->set('users.0.status', 'active');

// Good: Single transactional write
$store->begin();
$store->set('users.0.name', 'Alice');
$store->set('users.0.age', 30);
$store->set('users.0.status', 'active');
$store->commit();
```

---

## Compression

### Enable Compression for Large Files

```php
// Zstd: Best compression ratio + speed
$store->setOption('compression', 'zstd');

// Gzip: Good balance
$store->setOption('compression', 'gzip');

// None: Fastest for small files
$store->setOption('compression', 'none');
```

### Compression Comparison

| Method | File Size | Read Speed | Write Speed | Best For |
|--------|-----------|------------|-------------|----------|
| **none** | 10 MB | Fastest | Fastest | < 10MB files, frequent writes |
| **gzip** | 3 MB (3x) | Fast | Fast | General purpose |
| **zstd** | 2.5 MB (4x) | Fast | Fast | Large files, storage-constrained |

### When to Use Compression

✅ Files > 10MB  
✅ Storage-constrained environments  
✅ Network transfer (smaller file size)  
❌ Very small files (< 1MB)  
❌ Write-heavy workloads (compression overhead)

---

## Aggregation Performance

### Use Built-in Aggregations

```php
// Good: Native aggregation (9.7x faster)
$stats = $store->aggregate('orders', [
    'sum' => 'total',
    'avg' => 'total',
    'count' => 'id'
]);

// Bad: Manual calculation
$orders = $store->get('orders');
$total = array_sum(array_column($orders, 'total'));
$avg = $total / count($orders);
```

### Combine Multiple Aggregations

```php
// Good: Single call
$stats = $store->aggregate('products', [
    'sum' => 'price',
    'avg' => 'price',
    'min' => 'price',
    'max' => 'price',
    'count' => 'id'
]);

// Bad: Multiple calls
$sum = $store->aggregate('products', ['sum' => 'price']);
$avg = $store->aggregate('products', ['avg' => 'price']);
// ... (4x slower)
```

---

## Transaction Performance

### Batch Related Operations

```php
// Good: Single transaction
$store->begin();
for ($i = 0; $i < 100; $i++) {
    $store->push('logs', ['message' => "Log $i"]);
}
$store->commit(); // Single fsync

// Bad: Individual writes
for ($i = 0; $i < 100; $i++) {
    $store->push('logs', ['message' => "Log $i"]); // 100 fsyncs
}
```

### Transaction Best Practices

✅ Group related writes  
✅ Keep transactions short  
✅ Commit early and often  
❌ Long-running transactions  
❌ Mixing reads and writes unnecessarily

---

## File Organization

### Split Large Collections

```php
// Bad: Single massive file
// data.json (500MB)
{
  "users": [...10000 items...],
  "products": [...50000 items...],
  "orders": [...100000 items...]
}

// Good: Separate files by entity
// users.json (50MB)
// products.json (200MB)
// orders.json (250MB)

$userStore = new Store('users.json');
$productStore = new Store('products.json');
$orderStore = new Store('orders.json');
```

### Archive Old Data

```php
// Move old orders to archive
$oldOrders = $store->find('orders', [
    'created_at' => ['$lt' => date('Y-m-d', strtotime('-1 year'))]
]);

$archiveStore = new Store('orders-archive-2025.json');
foreach ($oldOrders as $order) {
    $archiveStore->push('orders', $order);
    $store->find and remove from main store
}
```

---

## Benchmarking

### Monitor Query Performance

```php
$start = microtime(true);

$results = $store->find('users', ['status' => 'active']);

$duration = (microtime(true) - $start) * 1000;
error_log("Query took {$duration}ms");

if ($duration > 100) {
    error_log("SLOW QUERY: Consider adding index");
}
```

### Use Metrics API

```php
$metrics = $store->getMetrics();

// Log to monitoring system
sendToMonitoring([
    'jsonq.reads' => $metrics['reads'],
    'jsonq.writes' => $metrics['writes'],
    'jsonq.cache_hit_rate' => $metrics['cache_hit_rate'],
    'jsonq.avg_latency' => $metrics['avg_latency_ms']
]);
```

---

## Performance Checklist

✅ Create indexes on queried fields  
✅ Use field projection (`select`)  
✅ Limit result sets (pagination)  
✅ Enable compression for large files  
✅ Use transactions for batch writes  
✅ Monitor cache hit rate (target >80%)  
✅ Split large files by entity  
✅ Archive old data  
✅ Use built-in aggregations  
✅ Monitor query latency  

---

## Performance Targets

| Metric | Target | Action if Below |
|--------|--------|-----------------|
| Cache Hit Rate | >80% | Review write patterns |
| Avg Query Time | <10ms | Add indexes, use projection |
| File Size | <100MB | Split files, archive data |
| Index Coverage | >90% | Index frequently queried fields |

---

## See Also

- [Indexing Guide](../guides/indexing.md)
- [Monitoring Guide](monitoring.md)
- [Production Deployment](production.md)
