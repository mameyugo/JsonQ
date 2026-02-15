//! Metrics and Observability system
//!
//! Tracks operational statistics using atomic counters for low overhead.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

/// Operational metrics for a store
#[derive(Debug)]
pub struct Metrics {
    pub reads: AtomicU64,
    pub writes: AtomicU64,
    pub cache_hits: AtomicU64,
    pub cache_misses: AtomicU64,
    pub total_latency_ns: AtomicU64,
}

impl Default for Metrics {
    fn default() -> Self {
        Self {
            reads: AtomicU64::new(0),
            writes: AtomicU64::new(0),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
            total_latency_ns: AtomicU64::new(0),
        }
    }
}

impl Metrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_read(&self) {
        self.reads.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_write(&self) {
        self.writes.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_cache_hit(&self) {
        self.cache_hits.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_cache_miss(&self) {
        self.cache_misses.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_latency(&self, duration: Duration) {
        self.total_latency_ns.fetch_add(duration.as_nanos() as u64, Ordering::Relaxed);
    }

    pub fn snapshot(&self) -> MetricsSnapshot {
        let reads = self.reads.load(Ordering::Relaxed);
        let latency_ns = self.total_latency_ns.load(Ordering::Relaxed);
        
        MetricsSnapshot {
            reads,
            writes: self.writes.load(Ordering::Relaxed),
            cache_hits: self.cache_hits.load(Ordering::Relaxed),
            cache_misses: self.cache_misses.load(Ordering::Relaxed),
            avg_latency_ms: if reads > 0 {
                (latency_ns as f64 / reads as f64) / 1_000_000.0
            } else {
                0.0
            },
        }
    }
}

/// A point-in-time snapshot of metrics
#[derive(Debug, serde::Serialize)]
pub struct MetricsSnapshot {
    pub reads: u64,
    pub writes: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub avg_latency_ms: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_recording() {
        let metrics = Metrics::new();
        metrics.record_read();
        metrics.record_write();
        metrics.record_cache_hit();
        metrics.record_cache_miss();
        metrics.record_latency(Duration::from_millis(100));

        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.reads, 1);
        assert_eq!(snapshot.writes, 1);
        assert_eq!(snapshot.cache_hits, 1);
        assert_eq!(snapshot.cache_misses, 1);
        assert_eq!(snapshot.avg_latency_ms, 100.0);
    }

    #[test]
    fn test_avg_latency_calculation() {
        let metrics = Metrics::new();
        metrics.record_read();
        metrics.record_latency(Duration::from_millis(10));
        metrics.record_read();
        metrics.record_latency(Duration::from_millis(20));

        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.reads, 2);
        assert_eq!(snapshot.avg_latency_ms, 15.0);
    }
}
