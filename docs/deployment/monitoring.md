# Monitoring Guide

Learn how to monitor JsonQ performance and health in production.

## Built-in Metrics

### Real-Time Metrics API

```php
use JsonQ\Store;

$store = new Store('data.json');

// Get current metrics
$metrics = $store->getMetrics();

print_r($metrics);
/*
Array (
    [reads] => 1523
    [writes] => 248
    [cache_hits] => 1401
    [cache_misses] => 122
    [cache_hit_rate] => 92.0
    [avg_latency_ms] => 1.2
    [last_operation] => read
    [last_operation_time] => 1708168200.123
)
*/
```

### Key Metrics Explained

| Metric | Description | Good Value | Action if Bad |
|--------|-------------|------------|---------------|
| `reads` | Total read operations | - | Monitor growth rate |
| `writes` | Total write operations | - | Monitor growth rate |
| `cache_hits` | Reads served from cache | High | - |
| `cache_misses` | Reads from disk | Low | Optimize queries |
| `cache_hit_rate` | Percentage cached | >80% | Review write patterns |
| `avg_latency_ms` | Average operation time | <10ms | Add indexes, optimize |

---

## Monitoring Strategies

### 1. Periodic Logging

```php
class JsonQMonitor {
    private Store $store;
    private string $logFile;
    
    public function __construct(Store $store, string $logFile = '/var/log/jsonq.log') {
        $this->store = $store;
        $this->logFile = $logFile;
    }
    
    public function logMetrics(): void {
        $metrics = $this->store->getMetrics();
        $stats = $this->store->stats();
        
        $log = sprintf(
            "[%s] Reads: %d, Writes: %d, Cache: %.1f%%, Latency: %.2fms, Size: %s\n",
            date('Y-m-d H:i:s'),
            $metrics['reads'],
            $metrics['writes'],
            $metrics['cache_hit_rate'],
            $metrics['avg_latency_ms'],
            $stats['file_size_h']
        );
        
        file_put_contents($this->logFile, $log, FILE_APPEND);
    }
    
    public function checkHealth(): array {
        $metrics = $this->store->getMetrics();
        $stats = $this->store->stats();
        
        $issues = [];
        
        // Check cache hit rate
        if ($metrics['cache_hit_rate'] < 80) {
            $issues[] = "Low cache hit rate: {$metrics['cache_hit_rate']}%";
        }
        
        // Check latency
        if ($metrics['avg_latency_ms'] > 50) {
            $issues[] = "High latency: {$metrics['avg_latency_ms']}ms";
        }
        
        // Check file size
        if ($stats['file_size'] > 100 * 1024 * 1024) { // 100MB
            $issues[] = "Large file size: {$stats['file_size_h']}";
        }
        
        return [
            'healthy' => empty($issues),
            'issues' => $issues,
            'metrics' => $metrics,
            'stats' => $stats
        ];
    }
}

// Usage
$monitor = new JsonQMonitor($store);

// Log metrics every minute
$monitor->logMetrics();

// Health check
$health = $monitor->checkHealth();
if (!$health['healthy']) {
    foreach ($health['issues'] as $issue) {
        trigger_error("JsonQ Health Issue: $issue", E_USER_WARNING);
    }
}
```

### 2. Integration with Monitoring Tools

#### Prometheus

```php
class PrometheusExporter {
    private Store $store;
    
    public function export(): string {
        $metrics = $this->store->getMetrics();
        $stats = $this->store->stats();
        
        return <<<METRICS
# HELP jsonq_reads_total Total number of read operations
# TYPE jsonq_reads_total counter
jsonq_reads_total {$metrics['reads']}

# HELP jsonq_writes_total Total number of write operations
# TYPE jsonq_writes_total counter
jsonq_writes_total {$metrics['writes']}

# HELP jsonq_cache_hit_rate Cache hit rate percentage
# TYPE jsonq_cache_hit_rate gauge
jsonq_cache_hit_rate {$metrics['cache_hit_rate']}

# HELP jsonq_latency_ms Average operation latency in milliseconds
# TYPE jsonq_latency_ms gauge
jsonq_latency_ms {$metrics['avg_latency_ms']}

# HELP jsonq_file_size_bytes File size in bytes
# TYPE jsonq_file_size_bytes gauge
jsonq_file_size_bytes {$stats['file_size']}
METRICS;
    }
}

// Expose metrics endpoint
header('Content-Type: text/plain');
echo (new PrometheusExporter($store))->export();
```

#### StatsD/Datadog

```php
use DataDog\DogStatsd;

function sendJsonQMetrics(Store $store, DogStatsd $statsd): void {
    $metrics = $store->getMetrics();
    
    $statsd->increment('jsonq.reads', $metrics['reads']);
    $statsd->increment('jsonq.writes', $metrics['writes']);
    $statsd->gauge('jsonq.cache_hit_rate', $metrics['cache_hit_rate']);
    $statsd->gauge('jsonq.latency', $metrics['avg_latency_ms']);
    $statsd->histogram('jsonq.operation_time', $metrics['avg_latency_ms']);
}
```

---

## Alert Configuration

### Define Thresholds

```php
class JsonQAlerting {
    private array $thresholds = [
        'cache_hit_rate_min' => 80.0,
        'latency_max_ms' => 50.0,
        'file_size_max_mb' => 100,
        'error_rate_max' => 0.01
    ];
    
    public function checkThresholds(Store $store): array {
        $metrics = $store->getMetrics();
        $stats = $store->stats();
        $alerts = [];
        
        if ($metrics['cache_hit_rate'] < $this->thresholds['cache_hit_rate_min']) {
            $alerts[] = [
                'level' => 'warning',
                'metric' => 'cache_hit_rate',
                'value' => $metrics['cache_hit_rate'],
                'threshold' => $this->thresholds['cache_hit_rate_min'],
                'message' => 'Cache hit rate below threshold'
            ];
        }
        
        if ($metrics['avg_latency_ms'] > $this->thresholds['latency_max_ms']) {
            $alerts[] = [
                'level' => 'critical',
                'metric' => 'latency',
                'value' => $metrics['avg_latency_ms'],
                'threshold' => $this->thresholds['latency_max_ms'],
                'message' => 'Average latency exceeds threshold'
            ];
        }
        
        $fileSizeMB = $stats['file_size'] / (1024 * 1024);
        if ($fileSizeMB > $this->thresholds['file_size_max_mb']) {
            $alerts[] = [
                'level' => 'warning',
                'metric' => 'file_size',
                'value' => $fileSizeMB,
                'threshold' => $this->thresholds['file_size_max_mb'],
                'message' => 'File size growing large'
            ];
        }
        
        return $alerts;
    }
    
    public function sendAlerts(array $alerts): void {
        foreach ($alerts as $alert) {
            // Send to alerting system (email, Slack, PagerDuty, etc.)
            $this->notify($alert);
        }
    }
    
    private function notify(array $alert): void {
        // Example: Send to Slack
        // slack_send_message("#alerts", json_encode($alert));
        
        // Example: Log to file
        error_log(sprintf(
            "[%s] JsonQ Alert: %s (value: %.2f, threshold: %.2f)",
            $alert['level'],
            $alert['message'],
            $alert['value'],
            $alert['threshold']
        ));
    }
}
```

---

## Dashboard Example

### Simple HTML Dashboard

```php
<?php
$store = new JsonQ\Store('data.json');
$metrics = $store->getMetrics();
$stats = $store->stats();

$cacheColor = $metrics['cache_hit_rate'] > 80 ? 'green' : 'orange';
$latencyColor = $metrics['avg_latency_ms'] < 10 ? 'green' : 
                ($metrics['avg_latency_ms'] < 50 ? 'orange' : 'red');
?>
<!DOCTYPE html>
<html>
<head>
    <title>JsonQ Monitoring</title>
    <meta http-equiv="refresh" content="5">
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        .metrics { display: grid; grid-template-columns: repeat(3, 1fr); gap: 20px; }
        .metric { padding: 20px; background: #f5f5f5; border-radius: 8px; }
        .metric h3 { margin: 0 0 10px 0; }
        .metric .value { font-size: 32px; font-weight: bold; }
        .green { color: #4caf50; }
        .orange { color: #ff9800; }
        .red { color: #f44336; }
    </style>
</head>
<body>
    <h1>📊 JsonQ Monitoring Dashboard</h1>
    <p>Last updated: <?= date('Y-m-d H:i:s') ?></p>
    
    <div class="metrics">
        <div class="metric">
            <h3>Total Operations</h3>
            <div class="value"><?= number_format($metrics['reads'] + $metrics['writes']) ?></div>
            <small>Reads: <?= number_format($metrics['reads']) ?> | Writes: <?= number_format($metrics['writes']) ?></small>
        </div>
        
        <div class="metric">
            <h3>Cache Hit Rate</h3>
            <div class="value <?= $cacheColor ?>"><?= number_format($metrics['cache_hit_rate'], 1) ?>%</div>
            <small>Hits: <?= number_format($metrics['cache_hits']) ?> | Misses: <?= number_format($metrics['cache_misses']) ?></small>
        </div>
        
        <div class="metric">
            <h3>Average Latency</h3>
            <div class="value <?= $latencyColor ?>"><?= number_format($metrics['avg_latency_ms'], 2) ?>ms</div>
            <small>Target: <10ms</small>
        </div>
        
        <div class="metric">
            <h3>File Size</h3>
            <div class="value"><?= $stats['file_size_h'] ?></div>
            <small>Path: <?= basename($stats['file_path']) ?></small>
        </div>
        
        <div class="metric">
            <h3>Active Indexes</h3>
            <div class="value"><?= $stats['active_indexes'] ?></div>
            <small><?= $stats['key_count'] ?> top-level keys</small>
        </div>
        
        <div class="metric">
            <h3>Last Operation</h3>
            <div class="value"><?= ucfirst($metrics['last_operation']) ?></div>
            <small><?= date('H:i:s', (int)$metrics['last_operation_time']) ?></small>
        </div>
    </div>
</body>
</html>
```

---

## Performance Analysis

### Track Query Performance

```php
class QueryAnalyzer {
    private array $slowQueries = [];
    private float $slowThreshold = 50.0; // ms
    
    public function trackQuery(string $collection, array $query, float $duration): void {
        if ($duration > $this->slowThreshold) {
            $this->slowQueries[] = [
                'collection' => $collection,
                'query' => json_encode($query),
                'duration_ms' => $duration,
                'timestamp' => microtime(true)
            ];
        }
    }
    
    public function getSlowQueries(int $limit = 10): array {
        usort($this->slowQueries, fn($a, $b) => $b['duration_ms'] <=> $a['duration_ms']);
        return array_slice($this->slowQueries, 0, $limit);
    }
    
    public function reportSlowQueries(): void {
        $slow = $this->getSlowQueries();
        
        if (!empty($slow)) {
            echo "⚠️ Slow Queries Detected:\n";
            foreach ($slow as $query) {
                echo sprintf(
                    "  [%.2fms] %s: %s\n",
                    $query['duration_ms'],
                    $query['collection'],
                    $query['query']
                );
            }
        }
    }
}
```

---

## Best Practices

1. **Monitor Regularly**: Check metrics at least every 5 minutes
2. **Set Alerts**: Configure alerts for critical thresholds
3. **Track Trends**: Monitor metrics over time to spot issues early
4. **Log Slow Queries**: Identify queries that need optimization
5. **Monitor File Growth**: Track file size to plan for archival
6. **Use Dashboards**: Visualize metrics for quick insights

---

## Monitoring Checklist

✅ Enable metrics collection  
✅ Set up periodic logging  
✅ Configure alerting thresholds  
✅ Create monitoring dashboard  
✅ Track slow queries  
✅ Monitor file size growth  
✅ Review metrics regularly  
✅ Integrate with existing monitoring tools  

---

## See Also

- [Performance Tuning](performance-tuning.md)
- [Production Deployment](production.md)
- [Best Practices](../guides/best-practices.md)
