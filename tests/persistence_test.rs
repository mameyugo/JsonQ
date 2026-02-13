#[cfg(test)]
mod tests {
    use jsonq::store::StoreInner;
    use serde_json::json;
    use std::fs;
    use std::path::PathBuf;
    use std::thread;
    use std::time::Duration;

    fn cleanup(path: &str) {
        let p = PathBuf::from(path);
        if p.exists() {
            let _ = fs::remove_file(&p);
        }
        let tmp = p.with_extension("tmp");
        if tmp.exists() {
            let _ = fs::remove_file(&tmp);
        }
        // Remove potential index files
        if let Ok(entries) = fs::read_dir(p.parent().unwrap()) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if let Some(ext) = path.extension() {
                        if ext == "idx" {
                            let _ = fs::remove_file(path);
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn test_persistence_reloads_index() {
        let path = "tests/data/persistence_test.json";
        cleanup(path);
        
        // 1. Create store and populate data
        let store = StoreInner::new(path.to_string());
        let data = json!({
            "users": [
                {"id": 1, "role": "admin"},
                {"id": 2, "role": "user"},
                {"id": 3, "role": "admin"}
            ]
        });
        store.write(&data).unwrap();
        
        // 2. Build index (this should trigger persistence)
        store.build_index("users", "role").unwrap();
        
        // Verify index works in memory
        let results = store.idx_lookup("users", "role", &json!("admin")).unwrap();
        assert_eq!(results.len(), 2);
        
        // 3. Simulate restart by dropping store and creating new one
        drop(store);
        
        // Ensure serialization finished (fs operations are synchronous in our impl, so this is just being safe)
        
        let store2 = StoreInner::new(path.to_string());
        
        // 4. Verify index is NOT in memory yet (lazy loading)
        {
            let indexes = store2.indexes().read().unwrap();
            assert!(indexes.get("users").is_none());
        }
        
        // 5. Trigger lookup (should load from disk)
        let results2 = store2.idx_lookup("users", "role", &json!("admin")).expect("Should load index from disk");
        assert_eq!(results2.len(), 2);
        
        // 6. Verify it's now in memory
        {
            let indexes = store2.indexes().read().unwrap();
            assert!(indexes.get("users").is_some());
        }
        
        cleanup(path);
    }

    #[test]
    fn test_invalidation_on_data_change() {
        let path = "tests/data/invalidation_test.json";
        cleanup(path);
        
        let store = StoreInner::new(path.to_string());
        let data = json!({
            "products": [
                {"id": 1, "cat": "A"},
                {"id": 2, "cat": "B"}
            ]
        });
        store.write(&data).unwrap();
        store.build_index("products", "cat").unwrap();
        
        // Force file mtime update by writing new data
        thread::sleep(Duration::from_secs(1)); // Ensure mtime allows for change
        
        let new_data = json!({
            "products": [
                {"id": 1, "cat": "A"},
                {"id": 2, "cat": "B"},
                {"id": 3, "cat": "A"}
            ]
        });
        store.write(&new_data).unwrap(); // This updates file mtime
        
        // Index is now stale in memory (store instance still has old build_at, but file mtime is newer)
        // Note: Our current implementation updates built_at in memory only on build_index.
        // But StoreInner::mtime() checks the FILE mtime.
        // The index in memory has built_at < new file mtime.
        
        // Lookup should fail or reload. 
        // In our current implementation of idx_lookup:
        // if store.built_at < mt { return None; }
        // It returns None, forcing a scan (which is correct behavior for safety, though rebuilding would be better)
        // But wait, our implementation of idx_lookup returns None if stale. 
        // Let's verify that.
        
        let results = store.idx_lookup("products", "cat", &json!("A"));
        assert!(results.is_none(), "Should return None for stale index");
        
        cleanup(path);
    }

    #[test]
    fn test_invalidation_on_write() {
        let path = "tests/data/invalidation_write_test.json";
        cleanup(path);
        
        let store = StoreInner::new(path.to_string());
        store.write(&json!({"users": [{"id":1, "name":"a"}]})).unwrap();
        store.build_index("users", "name").unwrap();
        
        // Ensure index exists
        assert!(store.indexes().read().unwrap().contains_key("users"));
        
        // Write new data
        store.write(&json!({"users": [{"id":1, "name":"b"}]})).unwrap();
        
        // Index should be gone from memory
        assert!(!store.indexes().read().unwrap().contains_key("users"));
        
        cleanup(path);
    }
}
