//! Core storage engine implementation

use super::options::CompressionMethod;
use super::transaction::TransactionState;
use super::{CachedData, IndexStore, LockGuard, StoreOpts};
use crate::metrics::Metrics;
use crate::store::cleanup::{cleanup_temp_files, TempFileGuard};
use crate::utils::interner::KeyInterner;
use memmap2::Mmap;
use serde_json::Value;
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

/// Core storage engine
///
/// Manages JSON file storage with caching, transactions, and indexing
pub struct StoreInner {
    /// Path to the JSON file
    pub(crate) path: PathBuf,

    /// Cached data with mtime tracking
    pub(crate) cache: RwLock<Option<CachedData>>,

    /// Indexes per collection
    pub(crate) indexes: RwLock<HashMap<String, IndexStore>>,

    /// Storage options (pretty, fsync)
    pub(crate) opts: RwLock<StoreOpts>,

    /// Transaction state
    pub(crate) transaction: TransactionState,

    /// Active file lock during transaction
    pub(crate) tx_lock: RwLock<Option<LockGuard>>,

    /// Operational metrics (shared globally)
    /// Operational metrics (shared globally)
    pub(crate) metrics: &'static Metrics,

    /// Key interner for deduplication (tracked for stats)
    pub(crate) interner: RwLock<KeyInterner>,
}

// SAFETY: StoreInner is thread-safe because it uses internal synchronization (RwLock)
// for all mutable access and file-level locking for multi-process safety.
unsafe impl Send for StoreInner {}
unsafe impl Sync for StoreInner {}

impl StoreInner {
    /// Create a new store instance with path validation
    pub fn new(path: String) -> Result<Self, String> {
        // ✅ SECURITY: Validate path before using it
        let validated_path = crate::security::validate_path(&path)?;

        // ✅ SECURITY: Check file size if it exists
        crate::security::validate_file_size(&validated_path)?;

        // Create parent directories if needed
        if let Some(parent) = validated_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        // Create empty file if it doesn't exist
        if !validated_path.exists() {
            if validated_path.extension().and_then(|s| s.to_str()) == Some("jsonl") {
                // JSONL files start empty
                let _ = File::create(&validated_path);
            } else {
                // Regular JSON files start with empty object
                let _ = std::fs::write(&validated_path, "{}");
            }
        }

        let instance = Self {
            path: validated_path,
            cache: RwLock::new(None),
            indexes: RwLock::new(HashMap::new()),
            opts: RwLock::new(StoreOpts::default()),
            transaction: TransactionState::new(),
            tx_lock: RwLock::new(None),
            metrics: Metrics::global(),
            interner: RwLock::new(KeyInterner::new()),
        };

        // Recover any pending transaction
        let _ = instance.recover_transaction();

        // ✅ AUTO-CLEANUP: Clean orphaned temp files on initialization
        match cleanup_temp_files(&instance.path) {
            Ok(count) if count > 0 => {
                tracing::info!(
                    "Cleaned {} orphaned temp files for {:?}",
                    count,
                    instance.path
                );
            }
            _ => {}
        }

        tracing::debug!("Store instance created for {:?}", instance.path);

        Ok(instance)
    }

    /// Get file modification time (seconds since UNIX epoch)
    pub fn mtime(&self) -> u64 {
        fs::metadata(&self.path)
            .and_then(|m| m.modified())
            .map(|t| {
                t.duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos() as u64
            })
            .unwrap_or(0)
    }

    /// Get file inode (0 if not on Unix)
    pub fn inode(&self) -> u64 {
        fs::metadata(&self.path)
            .map(|m| {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::MetadataExt;
                    m.ino()
                }
                #[cfg(not(unix))]
                0
            })
            .unwrap_or(0)
    }

    /// Read data from file with caching
    ///
    /// Returns cached data if available and still valid (mtime matches).
    /// Otherwise, reads from disk using memory-mapped file.
    ///
    /// **Thread-safety & Process-safety:**
    /// - Acquires shared (read) file lock before accessing data
    /// - Lock is released automatically when function returns
    /// - Multiple readers can access concurrently
    /// - Blocks if a write lock is held
    pub fn read(&self) -> Result<Arc<Value>, String> {
        // If in transaction, return transaction data (no lock needed for local tx state)
        if self.transaction.is_active() {
            if let Some(data) = self.transaction.get_data() {
                return Ok(data);
            }
        }

        // Acquire shared lock for reading
        let _lock = LockGuard::read(&self.path)?;

        // Internal read logic that assumes lock is held
        self.read_without_lock()
    }

    /// Internal read without acquiring lock (assumes caller holds lock)
    fn read_without_lock(&self) -> Result<Arc<Value>, String> {
        let start = std::time::Instant::now();
        self.metrics.record_read();

        // ✅ SECURITY: Validate file size before reading
        crate::security::validate_file_size(&self.path)?;

        let mt = self.mtime();
        let ino = self.inode();

        // Check cache validity
        {
            let cache = self
                .cache
                .read()
                .map_err(|e| format!("Cache lock poisoned: {}", e))?;
            if let Some(ref cached) = *cache {
                if cached.is_valid(mt, ino) {
                    self.metrics.record_cache_hit();
                    self.metrics.record_latency(start.elapsed());
                    return Ok(cached.data.clone());
                }
            }
        }

        self.metrics.record_cache_miss();

        // Cache miss or invalid - read from disk
        let file = File::open(&self.path).map_err(|e| format!("Failed to open file: {}", e))?;

        // SAFETY: Only mapping a read-only file descriptor that we've already
        // validated for size. The Mmap remains valid as long as the file is open.
        // The `mmap` variable is dropped when it goes out of scope, unmapping the memory.
        let mmap = unsafe { Mmap::map(&file).map_err(|e| format!("Failed to mmap file: {}", e))? };

        // Detect compression by magic numbers
        let content = if mmap.len() >= 2 && mmap[0] == 0x1F && mmap[1] == 0x8B {
            // Gzip detected
            #[cfg(feature = "compression")]
            {
                use std::io::Read;
                let mut decoder = flate2::read::GzDecoder::new(&mmap[..]);
                let mut buf = Vec::new();
                decoder
                    .read_to_end(&mut buf)
                    .map_err(|e| format!("Gzip decompression failed: {}", e))?;
                buf
            }
            #[cfg(not(feature = "compression"))]
            return Err("Gzip detected but compression feature is disabled".to_string());
        } else if mmap.len() >= 4 && &mmap[0..4] == &[0x28, 0xB5, 0x2F, 0xFD] {
            // Zstd detected
            #[cfg(feature = "compression")]
            {
                zstd::decode_all(&mmap[..])
                    .map_err(|e| format!("Zstd decompression failed: {}", e))?
            }
            #[cfg(not(feature = "compression"))]
            return Err("Zstd detected but compression feature is disabled".to_string());
        } else {
            // Assume uncompressed JSON
            mmap.to_vec()
        };

        // Try simd-json first for faster parsing (requires mutable buffer)
        // Falls back to serde_json if simd-json fails
        // Try simd-json first for faster parsing (requires mutable buffer)
        // Falls back to serde_json if simd-json fails
        // Use SIMD-accelerated UTF-8 validation
        if !crate::validation::utf8::validate_utf8(&content) {
            return Err("Invalid UTF-8 sequence".to_string());
        }

        let data: Value = {
            let mut content_mut = content.clone();
            match simd_json::from_slice(&mut content_mut) {
                Ok(value) => value,
                Err(_) => {
                    // Fallback to serde_json if simd-json fails
                    // (e.g., on unsupported architectures or malformed JSON)
                    serde_json::from_slice(&content)
                        .map_err(|e| format!("Failed to parse JSON: {}", e))?
                }
            }
        };

        // Update interner with new keys
        let data = if let Ok(mut interner) = self.interner.write() {
            interner.clear();
            Self::intern_keys(data, &mut interner)
        } else {
            // If lock poisoned, skip interning (fail safe)
            data
        };

        let arc_data = Arc::new(data);

        // Update cache
        {
            let mut cache = self
                .cache
                .write()
                .map_err(|e| format!("Cache lock poisoned: {}", e))?;
            *cache = Some(CachedData::new(arc_data.clone(), mt, ino));
        }

        self.metrics.record_latency(start.elapsed());
        Ok(arc_data)
    }

    /// Write data to file or transaction buffer
    ///
    /// If in transaction: buffers data in memory (no disk write)
    /// If not in transaction: writes atomically to disk via flush()
    pub fn write(&self, data: Arc<Value>) -> Result<(), String> {
        if self.transaction.is_active() {
            self.transaction.update_data(data);
            return Ok(());
        }

        self.flush(data)
    }

    /// Flush data to disk atomically with exclusive lock
    pub fn flush(&self, data: Arc<Value>) -> Result<(), String> {
        // Acquire exclusive lock for writing
        let _lock = LockGuard::write(&self.path)?;

        self.flush_without_lock(data)
    }

    /// Internal flush without acquiring lock (assumes caller holds lock)
    fn flush_without_lock(&self, data: Arc<Value>) -> Result<(), String> {
        let start = std::time::Instant::now();
        let opts = self
            .opts
            .read()
            .map_err(|e| format!("Options lock poisoned: {}", e))?;

        // Serialize JSON
        let json_str = if opts.pretty {
            serde_json::to_string_pretty(&*data)
        } else {
            serde_json::to_string(&*data)
        }
        .map_err(|e| format!("JSON serialization failed: {}", e))?;

        // Prepare bytes (possibly compressed)
        let final_bytes = match opts.compression {
            CompressionMethod::None => json_str.as_bytes().to_vec(),
            CompressionMethod::Gzip => {
                #[cfg(feature = "compression")]
                {
                    use std::io::Write;
                    let mut encoder =
                        flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
                    encoder
                        .write_all(json_str.as_bytes())
                        .map_err(|e| e.to_string())?;
                    encoder.finish().map_err(|e| e.to_string())?
                }
                #[cfg(not(feature = "compression"))]
                return Err("Gzip requested but compression feature is disabled".to_string());
            }
            CompressionMethod::Zstd => {
                #[cfg(feature = "compression")]
                {
                    zstd::encode_all(json_str.as_bytes(), 3).map_err(|e| e.to_string())?
                }
                #[cfg(not(feature = "compression"))]
                return Err("Zstd requested but compression feature is disabled".to_string());
            }
        };

        // Write to temporary file
        let tmp_path = self.path.with_extension("tmp");
        let mut temp_guard = TempFileGuard::new(&tmp_path)?;

        {
            let mut file = File::create(temp_guard.path())
                .map_err(|e| format!("Failed to create temp file: {}", e))?;

            file.write_all(&final_bytes)
                .map_err(|e| format!("Failed to write data: {}", e))?;

            // Force flush to disk if fsync enabled
            if opts.fsync {
                file.sync_all()
                    .map_err(|e| format!("fsync failed: {}", e))?;
            }
        }

        // Atomic rename
        fs::rename(temp_guard.path(), &self.path)
            .map_err(|e| format!("Failed to rename temp file: {}", e))?;

        // ✅ SUCCESS: Keep the temp file
        temp_guard.keep();

        // Update cache with new data (reuse Arc, no clone!)
        let mut cache = self
            .cache
            .write()
            .map_err(|e| format!("Cache lock poisoned: {}", e))?;
        *cache = Some(CachedData::new(data, self.mtime(), self.inode()));

        // Invalidate all indexes as file content (and mtime) has changed
        let mut indexes = self
            .indexes
            .write()
            .map_err(|e| format!("Index lock poisoned: {}", e))?;
        indexes.clear();

        // Flush completed
        self.metrics.record_write();

        tracing::info!(
            "Flush completed in {:?} for {:?}",
            start.elapsed(),
            self.path
        );

        Ok(())
    }

    /// Mutate data functionally with optimized locking
    ///
    /// Reads current data, applies mutation function, writes result.
    /// Uses a single write lock for the entire operation to prevent
    /// race conditions.
    pub fn mutate<F>(&self, f: F) -> Result<(), String>
    where
        F: FnOnce(&mut Value),
    {
        // Acquire exclusive (write) lock for the entire read-modify-write operation
        // only if we do not already hold it in an active transaction.
        let _lock = if self.transaction.is_active() {
            None
        } else {
            Some(LockGuard::write(&self.path)?)
        };

        // Read current data (directly, bypasses the lock acquisition in read())
        let arc_data = if self.transaction.is_active() {
            self.transaction.get_data().ok_or("No transaction data")?
        } else {
            self.read_without_lock()?
        };

        let mut data = (*arc_data).clone();
        f(&mut data);

        // Write (directly, bypasses the lock acquisition in flush())
        if self.transaction.is_active() {
            self.transaction.update_data(Arc::new(data));
            Ok(())
        } else {
            self.flush_without_lock(Arc::new(data))
        }
    }

    /// Get storage options.
    ///
    /// This method attempts to acquire a read lock on the options.
    /// If the lock is poisoned (e.g., a thread holding the lock panicked),
    /// it will recover the inner value and return a clone of it.
    pub fn get_opts(&self) -> StoreOpts {
        self.opts
            .read()
            .map(|o| o.clone())
            .unwrap_or_else(|e| e.into_inner().clone())
    }

    /// Set storage options.
    ///
    /// This method attempts to acquire a write lock on the options.
    /// If the lock is poisoned (e.g., a thread holding the lock panicked),
    /// it will recover the inner value and update it.
    pub fn set_opts(&self, opts: StoreOpts) {
        match self.opts.write() {
            Ok(mut o) => *o = opts,
            Err(e) => *e.into_inner() = opts,
        }
    }

    /// Begin a transaction
    pub fn begin_transaction(&self) -> Result<(), String> {
        if self.transaction.is_active() {
            return Err("Transaction already active".to_string());
        }

        // Acquire write lock to ensure we have exclusive access during TX
        let lock = LockGuard::write(&self.path)?;

        let tx_file = self.path.with_extension("tx");

        // Get current state without re-acquiring lock
        let current_data = self.read_without_lock()?;

        let tx_data_json = serde_json::json!({
            "started_at": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            "snapshot": current_data.as_ref()
        });

        fs::write(
            &tx_file,
            serde_json::to_vec(&tx_data_json).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;

        // Store the lock in the active transaction lock
        if let Ok(mut tx_lock) = self.tx_lock.write() {
            *tx_lock = Some(lock);
        } else if let Err(e) = self.tx_lock.write() {
            *e.into_inner() = Some(lock);
        }

        self.transaction.begin(current_data)
    }

    /// Commit current transaction
    pub fn commit(&self) -> Result<(), String> {
        if !self.transaction.is_active() {
            return Err("No active transaction".to_string());
        }

        let data = self.transaction.get_data().ok_or("No transaction data")?;

        // Flush data to disk without re-acquiring lock (we already hold it)
        self.flush_without_lock(data)?;

        // Consumes transaction state
        let _ = self.transaction.commit()?;

        // Release transaction lock
        if let Ok(mut tx_lock) = self.tx_lock.write() {
            *tx_lock = None;
        } else if let Err(e) = self.tx_lock.write() {
            *e.into_inner() = None;
        }

        // Remove transaction file
        let tx_file = self.path.with_extension("tx");
        let _ = fs::remove_file(&tx_file);
        Ok(())
    }

    /// Rollback current transaction
    pub fn rollback(&self) -> Result<(), String> {
        self.transaction.rollback()?;

        // Release transaction lock
        if let Ok(mut tx_lock) = self.tx_lock.write() {
            *tx_lock = None;
        } else if let Err(e) = self.tx_lock.write() {
            *e.into_inner() = None;
        }

        let tx_file = self.path.with_extension("tx");
        if tx_file.exists() {
            let _ = fs::remove_file(&tx_file);
        }
        Ok(())
    }

    /// Check if transaction is active
    pub fn in_transaction(&self) -> bool {
        self.transaction.is_active()
    }

    /// Recover from any pending transaction
    pub fn recover_transaction(&self) -> Result<(), String> {
        let tx_file = self.path.with_extension("tx");
        if tx_file.exists() {
            // Try to acquire an exclusive lock to see if another process is active.
            // If it succeeds, the transaction file is orphaned (crashed process) and can be removed.
            if LockGuard::try_write(&self.path).is_ok() {
                let _ = fs::remove_file(&tx_file);
            }
        }
        Ok(())
    }

    /// Manual cleanup of temporary files
    pub fn cleanup(&self) -> Result<usize, String> {
        cleanup_temp_files(&self.path)
    }

    /// Get reference to indexes
    pub fn indexes(&self) -> &RwLock<HashMap<String, IndexStore>> {
        &self.indexes
    }

    /// Get reference to path
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    /// TEMPORAL: Se migrará al módulo query/index
    pub fn build_index(&self, coll: &str, field: &str) -> Result<(), String> {
        let mt = self.mtime();
        let cd = self.read()?; // This handles its own locking
        let arr = match crate::path::read_path(&cd, coll) {
            Some(Value::Array(a)) => a,
            _ => return Err(format!("'{}' not array", coll)),
        };
        let mut idx: HashMap<String, Vec<usize>> = HashMap::new();
        for (i, item) in arr.iter().enumerate() {
            idx.entry(crate::utils::value_key(crate::path::read_nested(
                item, field,
            )))
            .or_default()
            .push(i);
        }
        let mut indexes = self
            .indexes
            .write()
            .map_err(|e| format!("Index lock poisoned: {}", e))?;
        let store = indexes
            .entry(coll.to_string())
            .or_insert_with(IndexStore::new);
        store.single.insert(field.to_string(), idx);
        store.built_at = mt;
        drop(indexes); // Drop lock before persistence
        self.persist_index(coll)?;
        Ok(())
    }

    pub fn build_compound(&self, coll: &str, fields: &[String]) -> Result<(), String> {
        let cd = self.read()?; // This handles its own locking
        let arr = match crate::path::read_path(&cd, coll) {
            Some(Value::Array(a)) => a,
            _ => return Err(format!("'{}' not array", coll)),
        };
        let mut idx: HashMap<String, Vec<usize>> = HashMap::new();
        for (i, item) in arr.iter().enumerate() {
            let k: String = fields
                .iter()
                .map(|f| crate::utils::value_key(crate::path::read_nested(item, f)))
                .collect::<Vec<_>>()
                .join("|");
            idx.entry(k).or_default().push(i);
        }
        let mut indexes = self
            .indexes
            .write()
            .map_err(|e| format!("Index lock poisoned: {}", e))?;
        let store = indexes
            .entry(coll.to_string())
            .or_insert_with(IndexStore::new);
        store.compound.insert(fields.to_vec(), idx);
        store.built_at = self.mtime();
        drop(indexes); // Drop lock before persistence
        self.persist_index(coll)?;
        Ok(())
    }

    pub fn idx_lookup(&self, coll: &str, field: &str, value: &Value) -> Option<Vec<usize>> {
        // Try to load index first
        let _ = self.ensure_index_loaded(coll, field);

        // ✅ FIX #3: Acquire shared lock to ensure mtime check and access are atomic
        let _lock = LockGuard::read(&self.path).ok()?;

        let indexes = self.indexes.read().ok()?;
        let store = indexes.get(coll)?;

        let mt = self.mtime();
        // Index validity vs current file mtime
        if store.built_at < mt {
            return None;
        }
        store
            .single
            .get(field)?
            .get(&crate::utils::value_key(Some(value)))
            .cloned()
    }

    // ══════════ PERSISTENCE ══════════

    fn index_file_path(&self, collection: &str, field: &str) -> PathBuf {
        let hash = format!("{:x}", md5::compute(field));
        self.path
            .with_extension(format!("{}.{}.idx", collection, hash))
    }

    /// Persists the indexes for a given collection to disk.
    ///
    /// This method acquires a read lock on the indexes to ensure thread-safe access
    /// while iterating and serializing index data.
    fn persist_index(&self, collection: &str) -> Result<(), String> {
        let indexes = self
            .indexes
            .read()
            .map_err(|e| format!("Index lock poisoned: {}", e))?;

        if let Some(store) = indexes.get(collection) {
            // Persist single indexes
            for (field, idx_map) in &store.single {
                let path = self.index_file_path(collection, field);
                let data =
                    bincode::serialize(&(store.built_at, idx_map)).map_err(|e| e.to_string())?;
                fs::write(&path, data).map_err(|e| e.to_string())?;
            }
        }
        Ok(())
    }

    /// Ensures that a specific index is loaded into memory.
    ///
    /// This method first attempts to acquire a read lock to check if the index is already loaded and valid.
    /// If not, it attempts to load the index from disk.
    fn ensure_index_loaded(&self, collection: &str, field: &str) -> bool {
        {
            let indexes = self.indexes.read().ok(); // Use ok() to handle poisoned lock gracefully
            if let Some(indexes_guard) = indexes {
                if let Some(store) = indexes_guard.get(collection) {
                    if store.single.contains_key(field) {
                        if store.built_at >= self.mtime() {
                            return true;
                        }
                    }
                }
            }
        }

        self.load_index_from_disk(collection, field).is_ok()
    }

    /// Loads an index from disk into memory.
    ///
    /// This method acquires a write lock on the indexes to safely insert the loaded index data.
    /// It handles potential poisoning of the lock.
    fn load_index_from_disk(&self, collection: &str, field: &str) -> Result<(), String> {
        let path = self.index_file_path(collection, field);
        if !path.exists() {
            return Err("Index file not found".to_string());
        }

        let data = fs::read(&path).map_err(|e| e.to_string())?;
        let (built_at, idx_map): (u64, HashMap<String, Vec<usize>>) =
            bincode::deserialize(&data).map_err(|e| e.to_string())?;

        let current_mtime = self.mtime();
        if built_at < current_mtime {
            let _ = fs::remove_file(&path);
            return Err("Index is stale".to_string());
        }

        let mut indexes = self
            .indexes
            .write()
            .map_err(|e| format!("Index lock poisoned: {}", e))?;
        let store = indexes
            .entry(collection.to_string())
            .or_insert_with(IndexStore::new);

        store.single.insert(field.to_string(), idx_map);
        store.built_at = built_at;

        Ok(())
    }

    /// Drops all indexes currently held in memory.
    ///
    /// This method acquires a write lock on the indexes to clear them.
    /// If the lock is poisoned, it returns 0, indicating no indexes were dropped.
    pub fn drop_all_indexes(&self) -> usize {
        if let Ok(mut indexes) = self.indexes.write() {
            let count = indexes.len();
            indexes.clear();
            count
        } else {
            0
        }
    }

    // ══════════ OUTPUT STREAMING ══════════

    /// Write content to a stream without intermediate buffers
    pub fn write_to_stream<W: Write>(&self, mut writer: W) -> Result<(), String> {
        let data = self.read()?;
        serde_json::to_writer(&mut writer, &*data)
            .map_err(|e| format!("Error writing to stream: {}", e))
    }

    /// Write pretty-printed content to a stream
    pub fn write_to_stream_pretty<W: Write>(&self, mut writer: W) -> Result<(), String> {
        let data = self.read()?;
        serde_json::to_writer_pretty(&mut writer, &*data)
            .map_err(|e| format!("Error writing to stream: {}", e))
    }

    // ══════════ JSONL SUPPORT ══════════

    /// Append a record to the file in JSONL format
    pub fn append_jsonl(&self, record: &Value) -> Result<(), String> {
        // Acquire write lock to ensure thread safety
        let _lock = LockGuard::write(&self.path)?;

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|e| format!("Cannot open file for append: {}", e))?;

        // Serialize without newline
        serde_json::to_writer(&mut file, record)
            .map_err(|e| format!("Error writing record: {}", e))?;

        // Append newline
        file.write_all(b"\n")
            .map_err(|e| format!("Error writing newline: {}", e))?;

        // Invalidate cache and indexes as file changed
        self.invalidate_cache_and_indexes();

        Ok(())
    }

    /// Read JSONL file line by line
    pub fn read_jsonl_iter(&self) -> Result<impl Iterator<Item = Value>, String> {
        // Validate file existence/security
        crate::security::validate_path(self.path.to_str().unwrap_or(""))?;

        let file = File::open(&self.path).map_err(|e| format!("Cannot open file: {}", e))?;

        let reader = BufReader::new(file);

        Ok(reader
            .lines()
            .filter_map(|line| line.ok())
            .filter(|line| !line.trim().is_empty())
            .filter_map(|line| serde_json::from_str(&line).ok()))
    }

    /// Helper to invalidate cache and indexes
    fn invalidate_cache_and_indexes(&self) {
        if let Ok(mut cache) = self.cache.write() {
            *cache = None;
        }
        if let Ok(mut indexes) = self.indexes.write() {
            indexes.clear();
        }
    }

    // ══════════ KEY INTERNING ══════════

    /// Recursive key interning
    // Note: With standard serde_json, keys are still Strings, so this
    // mainly tracks duplication statistics in the interner.
    fn intern_keys(value: Value, interner: &mut KeyInterner) -> Value {
        match value {
            Value::Object(map) => {
                let new_map: serde_json::Map<String, Value> = map
                    .into_iter()
                    .map(|(k, v)| {
                        // Intern the key (adds to interner cache)
                        // Then convert back to String (clones)
                        let _arc = interner.intern(&k);
                        let interned_value = Self::intern_keys(v, interner);
                        // We use the original string k, but the interner has recorded it
                        (k, interned_value)
                    })
                    .collect();
                Value::Object(new_map)
            }
            Value::Array(arr) => Value::Array(
                arr.into_iter()
                    .map(|v| Self::intern_keys(v, interner))
                    .collect(),
            ),
            _ => value,
        }
    }

    /// Get memory stats from interner
    pub fn memory_stats(&self) -> (usize, usize) {
        if let Ok(interner) = self.interner.read() {
            let stats = interner.stats();
            (stats.unique_keys, stats.total_references)
        } else {
            (0, 0)
        }
    }
}
