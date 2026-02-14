//! Core storage engine implementation

use super::{StoreOpts, CachedData, IndexStore, LockGuard};
use super::transaction::TransactionState;
use serde_json::Value;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use memmap2::Mmap;

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
}

impl StoreInner {
    /// Create a new store instance
    ///
    /// Creates the file and parent directories if they don't exist.
    /// Initializes with empty JSON object `{}` if file doesn't exist.
    pub fn new(path: String) -> Self {
        let p = PathBuf::from(&path);
        
        // Create parent directories if needed
        if let Some(parent) = p.parent() {
            let _ = fs::create_dir_all(parent);
        }
        
        // Create empty file if it doesn't exist
        if !p.exists() {
            let _ = fs::write(&p, "{}");
        }
        
        let instance = Self {
            path: p,
            cache: RwLock::new(None),
            indexes: RwLock::new(HashMap::new()),
            opts: RwLock::new(StoreOpts::default()),
            transaction: TransactionState::new(),
        };
        
        // Recover any pending transaction
        let _ = instance.recover_transaction();
        
        instance
    }
    
    /// Get file modification time (seconds since UNIX epoch)
    pub fn mtime(&self) -> u64 {
        fs::metadata(&self.path)
            .and_then(|m| m.modified())
            .map(|t| {
                t.duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
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
        let mt = self.mtime();
        
        // Check cache validity
        {
            let cache = self.cache.read().unwrap();
            if let Some(ref cached) = *cache {
                if cached.is_valid(mt) {
                    return Ok(cached.data.clone());
                }
            }
        }
        
        // Cache miss or invalid - read from disk
        let file = File::open(&self.path)
            .map_err(|e| format!("Failed to open file: {}", e))?;
        
        let mmap = unsafe { 
            Mmap::map(&file)
                .map_err(|e| format!("Failed to mmap file: {}", e))?
        };
        
        let data: Value = serde_json::from_slice(&mmap)
            .map_err(|e| format!("Failed to parse JSON: {}", e))?;
        
        let arc_data = Arc::new(data);
        
        // Update cache
        *self.cache.write().unwrap() = Some(CachedData::new(arc_data.clone(), mt));
        
        Ok(arc_data)
    }
    
    /// Write data to file or transaction buffer
    /// 
    /// If in transaction: buffers data in memory (no disk write)
    /// If not in transaction: writes atomically to disk via flush()
    pub fn write(&self, data: &Value) -> Result<(), String> {
        if self.transaction.is_active() {
            self.transaction.update_data(Arc::new(data.clone()));
            return Ok(());
        }
        
        self.flush(data)
    }
    
    /// Flush data to disk atomically with exclusive lock
    pub fn flush(&self, data: &Value) -> Result<(), String> {
        // Acquire exclusive lock for writing
        let _lock = LockGuard::write(&self.path)?;
        
        self.flush_without_lock(data)
    }

    /// Internal flush without acquiring lock (assumes caller holds lock)
    fn flush_without_lock(&self, data: &Value) -> Result<(), String> {
        let opts = self.opts.read().unwrap();
        
        // Serialize JSON
        let json_str = if opts.pretty {
            serde_json::to_string_pretty(data)
        } else {
            serde_json::to_string(data)
        }
        .map_err(|e| format!("JSON serialization failed: {}", e))?;
        
        // Write to temporary file
        let tmp_path = self.path.with_extension("tmp");
        {
            let mut file = File::create(&tmp_path)
                .map_err(|e| format!("Failed to create temp file: {}", e))?;
            
            file.write_all(json_str.as_bytes())
                .map_err(|e| format!("Failed to write data: {}", e))?;
            
            // Force flush to disk if fsync enabled
            if opts.fsync {
                file.sync_all()
                    .map_err(|e| format!("fsync failed: {}", e))?;
            }
        }
        
        // Atomic rename
        fs::rename(&tmp_path, &self.path)
            .map_err(|e| format!("Failed to rename temp file: {}", e))?;
        
        // Update cache with new data
        *self.cache.write().unwrap() = Some(CachedData::new(
            Arc::new(data.clone()),
            self.mtime()
        ));

        // Invalidate all indexes as file content (and mtime) has changed
        self.indexes.write().unwrap().clear();
        
        Ok(())
    }
    
    /// Mutate data functionally with optimized locking
    ///
    /// Reads current data, applies mutation function, writes result.
    /// Uses a single write lock for the entire operation to prevent
    /// race conditions.
    pub fn mutate<F>(&self, f: F) -> Result<(), String> 
    where 
        F: FnOnce(&mut Value)
    {
        // Acquire exclusive (write) lock for the entire read-modify-write operation
        let _lock = LockGuard::write(&self.path)?;
        
        // Read current data (directly, bypasses the lock acquisition in read())
        let arc_data = if self.transaction.is_active() {
            self.transaction.get_data()
                .ok_or("No transaction data")?
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
            self.flush_without_lock(&data)
        }
    }
    
    /// Get storage options
    pub fn get_opts(&self) -> StoreOpts {
        self.opts.read().unwrap().clone()
    }
    
    /// Set storage options
    pub fn set_opts(&self, opts: StoreOpts) {
        *self.opts.write().unwrap() = opts;
    }
    
    /// Begin a transaction
    pub fn begin_transaction(&self) -> Result<(), String> {
        if self.transaction.is_active() {
            return Err("Transaction already active".to_string());
        }

        // Acquire write lock to ensure we have exclusive access during TX initialization
        let _lock = LockGuard::write(&self.path)?;
        
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
        
        fs::write(&tx_file, serde_json::to_vec(&tx_data_json).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
        
        self.transaction.begin(current_data)
    }
    
    /// Commit current transaction
    pub fn commit(&self) -> Result<(), String> {
        let data = self.transaction.commit()?;
        
        // Acquire write lock for the commit process
        let _lock = LockGuard::write(&self.path)?;
        
        // Flush data to disk without re-acquiring lock
        self.flush_without_lock(&data)?;

        // Remove transaction file
        let tx_file = self.path.with_extension("tx");
        let _ = fs::remove_file(&tx_file);
        Ok(())
    }
    
    /// Rollback current transaction
    pub fn rollback(&self) -> Result<(), String> {
        self.transaction.rollback()?;
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
            let _ = fs::remove_file(&tx_file);
        }
        Ok(())
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
        let arr = match crate::path::read_path(&cd, coll) { Some(Value::Array(a)) => a, _ => return Err(format!("'{}' not array", coll)) };
        let mut idx: HashMap<String, Vec<usize>> = HashMap::new();
        for (i, item) in arr.iter().enumerate() { idx.entry(crate::utils::value_key(crate::path::read_nested(item, field))).or_default().push(i); }
        let mut indexes = self.indexes.write().unwrap();
        let store = indexes.entry(coll.to_string()).or_insert_with(IndexStore::new);
        store.single.insert(field.to_string(), idx); store.built_at = mt;
        drop(indexes); // Drop lock before persistence
        self.persist_index(coll)?;
        Ok(())
    }
    
    pub fn build_compound(&self, coll: &str, fields: &[String]) -> Result<(), String> {
        let cd = self.read()?; // This handles its own locking
        let arr = match crate::path::read_path(&cd, coll) { Some(Value::Array(a)) => a, _ => return Err(format!("'{}' not array", coll)) };
        let mut idx: HashMap<String, Vec<usize>> = HashMap::new();
        for (i, item) in arr.iter().enumerate() {
            let k: String = fields.iter().map(|f| crate::utils::value_key(crate::path::read_nested(item, f))).collect::<Vec<_>>().join("|");
            idx.entry(k).or_default().push(i);
        }
        let mut indexes = self.indexes.write().unwrap();
        let store = indexes.entry(coll.to_string()).or_insert_with(IndexStore::new);
        store.compound.insert(fields.to_vec(), idx); store.built_at = self.mtime();
        drop(indexes); // Drop lock before persistence
        self.persist_index(coll)?;
        Ok(())
    }
    
    pub fn idx_lookup(&self, coll: &str, field: &str, value: &Value) -> Option<Vec<usize>> {
        // Try to load index first
        let _ = self.ensure_index_loaded(coll, field);

        let indexes = self.indexes.read().unwrap();
        let store = indexes.get(coll)?;
        
        // Index validity vs current file mtime
        if store.built_at < self.mtime() { 
            return None; 
        }
        store.single.get(field)?.get(&crate::utils::value_key(Some(value))).cloned()
    }

    // ══════════ PERSISTENCE ══════════

    fn index_file_path(&self, collection: &str, field: &str) -> PathBuf {
        let hash = format!("{:x}", md5::compute(field));
        self.path.with_extension(format!("{}.{}.idx", collection, hash))
    }

    fn persist_index(&self, collection: &str) -> Result<(), String> {
        let indexes = self.indexes.read().unwrap();
        
        if let Some(store) = indexes.get(collection) {
            // Persist single indexes
            for (field, idx_map) in &store.single {
                let path = self.index_file_path(collection, field);
                let data = bincode::serialize(&(store.built_at, idx_map))
                    .map_err(|e| e.to_string())?;
                fs::write(&path, data).map_err(|e| e.to_string())?;
            }
        }
        Ok(())
    }

    fn ensure_index_loaded(&self, collection: &str, field: &str) -> bool {
        {
            let indexes = self.indexes.read().unwrap();
            if let Some(store) = indexes.get(collection) {
               if store.single.contains_key(field) {
                   if store.built_at >= self.mtime() {
                       return true; 
                   }
               }
            }
        }
        
        self.load_index_from_disk(collection, field).is_ok()
    }

    fn load_index_from_disk(&self, collection: &str, field: &str) -> Result<(), String> {
        let path = self.index_file_path(collection, field);
        if !path.exists() { return Err("Index file not found".to_string()); }

        let data = fs::read(&path).map_err(|e| e.to_string())?;
        let (built_at, idx_map): (u64, HashMap<String, Vec<usize>>) = 
            bincode::deserialize(&data).map_err(|e| e.to_string())?;

        let current_mtime = self.mtime();
        if built_at < current_mtime {
            let _ = fs::remove_file(&path);
            return Err("Index is stale".to_string());
        }

        let mut indexes = self.indexes.write().unwrap();
        let store = indexes.entry(collection.to_string())
            .or_insert_with(IndexStore::new);
        
        store.single.insert(field.to_string(), idx_map);
        store.built_at = built_at;
        
        Ok(())
    }
}
