//! Core storage engine implementation

use super::{StoreOpts, CachedData, IndexStore};
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
    ///
    /// # Examples
    ///
    /// ```rust
    /// use jsonq::store::StoreInner;
    ///
    /// let store = StoreInner::new("/tmp/data.json".to_string());
    /// ```
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
        
        Self {
            path: p,
            cache: RwLock::new(None),
            indexes: RwLock::new(HashMap::new()),
            opts: RwLock::new(StoreOpts::default()),
            transaction: TransactionState::new(),
        }
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
    /// During a transaction, returns transaction buffer instead of file.
    pub fn read(&self) -> Result<Arc<Value>, String> {
        // If in transaction, return transaction data
        if self.transaction.is_active() {
            if let Some(data) = self.transaction.get_data() {
                return Ok(data);
            }
        }
        
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
    
    /// Flush data to disk atomically
    ///
    /// Uses atomic write pattern:
    /// 1. Write to temporary file (.tmp)
    /// 2. Optionally fsync (if enabled)
    /// 3. Rename (atomic operation)
    ///
    /// This ensures crash safety - either old or new data, never partial.
    pub fn flush(&self, data: &Value) -> Result<(), String> {
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
        
        Ok(())
    }
    
    /// Mutate data functionally
    ///
    /// Reads current data, applies mutation function, writes result.
    /// Atomic operation (read-modify-write).
    ///
    /// # Examples
    ///
    /// ```rust
    /// store.mutate(|data| {
    ///     data.as_object_mut().unwrap().insert(
    ///         "key".to_string(),
    ///         json!("value")
    ///     );
    /// });
    /// ```
    pub fn mutate<F>(&self, f: F) -> Result<(), String> 
    where 
        F: FnOnce(&mut Value)
    {
        let arc_data = self.read()?;
        let mut data = (*arc_data).clone();
        f(&mut data);
        self.write(&data)
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
        let data = self.read()?;
        self.transaction.begin(data)
    }
    
    /// Commit current transaction
    pub fn commit(&self) -> Result<(), String> {
        let data = self.transaction.commit()?;
        self.flush(&data)
    }
    
    /// Rollback current transaction
    pub fn rollback(&self) -> Result<(), String> {
        self.transaction.rollback()
    }
    
    /// Check if transaction is active
    pub fn in_transaction(&self) -> bool {
        self.transaction.is_active()
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
        let cd = self.read()?;
        let arr = match crate::path::read_path(&cd, coll) { Some(Value::Array(a)) => a, _ => return Err(format!("'{}' not array", coll)) };
        let mut idx: HashMap<String, Vec<usize>> = HashMap::new();
        for (i, item) in arr.iter().enumerate() { idx.entry(crate::utils::value_key(crate::path::read_nested(item, field))).or_default().push(i); }
        let mut indexes = self.indexes.write().unwrap();
        let store = indexes.entry(coll.to_string()).or_insert_with(IndexStore::new);
        store.single.insert(field.to_string(), idx); store.built_at = mt;
        Ok(())
    }
    
    pub fn build_compound(&self, coll: &str, fields: &[String]) -> Result<(), String> {
        let cd = self.read()?;
        let arr = match crate::path::read_path(&cd, coll) { Some(Value::Array(a)) => a, _ => return Err(format!("'{}' not array", coll)) };
        let mut idx: HashMap<String, Vec<usize>> = HashMap::new();
        for (i, item) in arr.iter().enumerate() {
            let k: String = fields.iter().map(|f| crate::utils::value_key(crate::path::read_nested(item, f))).collect::<Vec<_>>().join("|");
            idx.entry(k).or_default().push(i);
        }
        let mut indexes = self.indexes.write().unwrap();
        let store = indexes.entry(coll.to_string()).or_insert_with(IndexStore::new);
        store.compound.insert(fields.to_vec(), idx); store.built_at = self.mtime();
        Ok(())
    }
    
    pub fn idx_lookup(&self, coll: &str, field: &str, value: &Value) -> Option<Vec<usize>> {
        let mt = self.mtime();
        let indexes = self.indexes.read().unwrap();
        let store = indexes.get(coll)?;
        if store.built_at < mt { return None; }
        store.single.get(field)?.get(&crate::utils::value_key(Some(value))).cloned()
    }
}

// Store engine components moved to mod store
