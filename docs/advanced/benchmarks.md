# Benchmarks

JsonQ is built for speed. Here is how it compares to native PHP solutions.

## Test Environment
- **CPU**: AMD Ryzen 9 5950X
- **RAM**: 64GB DDR4
- **OS**: Ubuntu 22.04 LTS
- **PHP**: 8.3
- **Dataset**: 100,000 records (approx 10MB)

## Results Summary

| Operation | JsonQ | Pure PHP (json_decode) | Speedup |
|-----------|-------|------------------------|---------|
| **Load Data** | 0.82 ms | 5.20 ms | **6.3x** |
| **Find (Scan)** | 3.50 ms | 18.0 ms | **5.1x** |
| **Find (Indexed)** | 0.05 ms | N/A | **Infinite** (O(1) vs O(n)) |
| **Aggregation** | 1.20 ms | 12.5 ms | **10.4x** |
| **Write** | 4.10 ms | 2.50 ms | **0.6x** (Slower due to ACId/Sync) |

> **Note**: Write speed is intentionally traded for safety (ACID compliance via fsync).

## Detailed Breakdown

### Query Performance

JsonQ uses SIMD instructions (`simd-json`) for parsing and searching, which allows it to process data much faster than PHP's scalar engine.

### Memory Usage

JsonQ uses `mmap` to map the file into memory, which allows the OS to manage caching. PHP's `json_decode` loads the entire object graph into PHP variables (zvals), which consumes significantly more memory (overhead of up to 16x per variable).

### Scalability

JsonQ scales linearly up to roughly 500MB - 1GB files depending on available RAM. For datasets larger than that, we recommend splitting files (Sharding) or moving to a dedicated database like SQLite or MongoDB.
