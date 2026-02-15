//! Storage configuration options

/// Compression methods for storage
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CompressionMethod {
    None,
    Gzip,
    Zstd,
}

/// Storage engine configuration
///
/// Controls performance vs safety tradeoffs
#[derive(Debug, Clone)]
pub struct StoreOpts {
    /// Pretty-print JSON output (slower writes, easier debugging)
    pub pretty: bool,
    
    /// Force fsync after writes (slower, guaranteed durability)
    pub fsync: bool,

    /// Compression method
    pub compression: CompressionMethod,
}

impl Default for StoreOpts {
    fn default() -> Self {
        Self {
            pretty: false,  // Compact JSON by default (faster)
            fsync: false,   // Skip fsync by default (faster)
            compression: CompressionMethod::None,
        }
    }
}

impl StoreOpts {
    /// Create options optimized for production (fast, compact)
    pub fn production() -> Self {
        Self {
            pretty: false,
            fsync: false,
            compression: CompressionMethod::None,
        }
    }
    
    /// Create options optimized for development (readable, safe)
    pub fn development() -> Self {
        Self {
            pretty: true,
            fsync: true,
            compression: CompressionMethod::None,
        }
    }
    
    /// Create options for maximum safety (slow but durable)
    pub fn safe() -> Self {
        Self {
            pretty: false,
            fsync: true,
            compression: CompressionMethod::None,
        }
    }
}
