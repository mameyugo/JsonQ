# JsonQ Advanced Features

This document describes the advanced features introduced in Phase 3 of JsonQ development.

## 1. Safe Regex Execution
JsonQ now supports true regular expression matching via the `$regex` operator. To prevent ReDoS (Regular Expression Denial of Service) attacks, the engine includes:
- **Compilation Limits**: Maximum size for compiled regex patterns.
- **Backtracking Limits**: Prevention of exponential backtracking.
- **Thread-Safe Cache**: Compiled patterns are cached to improve performance.

**Usage:**
```php
$s->find('users', ['email' => ['$regex' => '@gmail\.com$']]);
```

## 2. Storage Compression
You can now compress your JSON files using Gzip or Zstd.
- **Transparent Decompression**: The engine automatically detects the compression format upon reading.
- **Configurable Writing**: Choose the compression method for writes.

**Usage:**
```php
$s->setOption('compression', 'zstd'); // Options: none, gzip, zstd
```

## 3. Metrics & Observability
Track the performance of your store in real-time.
- **Reads/Writes**: Total number of operations.
- **Cache Hits/Misses**: Monitor the effectiveness of the internal cache.
- **Latency**: Average read latency in milliseconds.

**Usage:**
```php
$stats = $s->getMetrics();
echo "Reads: " . $stats['reads'] . ", Avg Latency: " . $stats['avg_latency_ms'] . "ms";
```

## 4. Query Optimizer
The internal query engine now intelligently selects the best index for multi-condition queries.
- **Selectivity Estimation**: It picks the index for the field that is expected to return the fewest results.
- **Compound Index Support**: Seamlessly integrates with existing single and compound indexes.

## 5. Performance Tips
- Use **Zstd** for the best balance between compression ratio and speed.
- **Batch writes** using `setMany` or transactions to minimize compression overhead.
- Monitor **Cache Hits**; if the hit rate is low, consider your indexing strategy.
