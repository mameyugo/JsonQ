# Deployment to Production

JsonQ is designed for production use. Follow this guide to ensure reliability and performance.

## Server Requirements

- **PHP**: 8.1+ with FPM or CLI
- **OS**: Linux (Debian/Ubuntu/CentOS)
- **Filesystem**: Local filesystem or block storage (EBS). **Do not use NFS or networked filesystems** due to file locking requirements.

## Configuration Checklist

### 1. Enable `fsync`
For data durability, ensure `fsync` is enabled. This ensures data is physically written to disk after every write.

```php
$store->setOption('fsync', true);
```

### 2. Compression
If you have large datasets (>10MB), enable compression to save disk space and I/O, at the cost of slight CPU increase.

```php
$store->setOption('compression', 'zstd'); // Recommended
```

### 3. Base Path Security
Restrict JsonQ access to a specific directory to prevent unauthorized file access.

```php
// In your bootstrap code
jsonq_set_base_path('/var/www/data/storage');
```

## Performance Tuning

### Memory Limits
JsonQ loads the JSON file into memory (via mmap). Ensure your PHP memory limit (`memory_limit` in php.ini) is sufficient for your dataset size + overhead.
Typically, for a 100MB JSON file, allow 512MB RAM.

### Opcache
Enable PHP Opcache for the extension wrapper scripts if you use them.

## Monitoring

Monitor these metrics using `getMetrics()`:

- **Cache Hits/Misses**: High miss rate implies frequent file changes or low memory.
- **Latency**: Track `avg_latency_ms`.
- **Write Errors**: Watch for permissions errors.

## Backups

Backing up even potentially active files is safe with JsonQ's atomic commit strategy, but for a consistent snapshot:

1. Use `$store->backup('/path/to/backup.json')`.
2. Or simply copy the `.json` file. Since writes are atomic (rename), you will always copy a valid JSON file (either the old version or the new one).


