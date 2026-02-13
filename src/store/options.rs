//! Storage configuration options

/// Storage engine configuration
///
/// Controls performance vs safety tradeoffs
#[derive(Debug, Clone)]
pub struct StoreOpts {
    /// Pretty-print JSON output (slower writes, easier debugging)
    pub pretty: bool,
    
    /// Force fsync after writes (slower, guaranteed durability)
    pub fsync: bool,
}

impl Default for StoreOpts {
    fn default() -> Self {
        Self {
            pretty: false,  // Compact JSON by default (faster)
            fsync: false,   // Skip fsync by default (faster)
        }
    }
}

impl StoreOpts {
    /// Create options optimized for production (fast, compact)
    pub fn production() -> Self {
        Self {
            pretty: false,
            fsync: false,
        }
    }
    
    /// Create options optimized for development (readable, safe)
    pub fn development() -> Self {
        Self {
            pretty: true,
            fsync: true,
        }
    }
    
    /// Create options for maximum safety (slow but durable)
    pub fn safe() -> Self {
        Self {
            pretty: false,
            fsync: true,
        }
    }
}
