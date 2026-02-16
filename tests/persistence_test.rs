#[cfg(test)]
mod tests {
    use jsonq::store::StoreInner;
    use serde_json::json;
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;
    use tempfile::TempDir;

    #[test]
    fn test_persistence_reloads_index() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("persistence_test.json");
        let path_str = path.to_str().unwrap().to_string();

        // 1. Create store and populate data
        let store = StoreInner::new(path_str.clone()).unwrap();
        let data = json!({
            "users": [
                {"id": 1, "role": "admin"},
                {"id": 2, "role": "user"},
                {"id": 3, "role": "admin"}
            ]
        });
        store.write(Arc::new(data)).unwrap();

        // 2. Build index (this should trigger persistence)
        store.build_index("users", "role").unwrap();

        // Verify index works in memory
        let results = store.idx_lookup("users", "role", &json!("admin")).unwrap();
        assert_eq!(results.len(), 2);

        // 3. Simulate restart by dropping store and creating new one
        drop(store);

        let store2 = StoreInner::new(path_str).unwrap();

        // 4. Verify index is NOT in memory yet (lazy loading)
        {
            let indexes = store2.indexes().read().unwrap();
            assert!(indexes.get("users").is_none());
        }

        // 5. Trigger lookup (should load from disk)
        let results2 = store2
            .idx_lookup("users", "role", &json!("admin"))
            .expect("Should load index from disk");
        assert_eq!(results2.len(), 2);

        // 6. Verify it's now in memory
        {
            let indexes = store2.indexes().read().unwrap();
            assert!(indexes.get("users").is_some());
        }
    }

    #[test]
    fn test_invalidation_on_data_change() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("invalidation_test.json");
        let path_str = path.to_str().unwrap().to_string();

        let store = StoreInner::new(path_str).unwrap();
        let data = json!({
            "products": [
                {"id": 1, "cat": "A"},
                {"id": 2, "cat": "B"}
            ]
        });
        store.write(Arc::new(data)).unwrap();
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
        store.write(Arc::new(new_data)).unwrap(); // This updates file mtime

        let results = store.idx_lookup("products", "cat", &json!("A"));
        assert!(results.is_none(), "Should return None for stale index");
    }

    #[test]
    fn test_invalidation_on_write() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("invalidation_write_test.json");
        let path_str = path.to_str().unwrap().to_string();

        let store = StoreInner::new(path_str).unwrap();
        store
            .write(Arc::new(json!({"users": [{"id":1, "name":"a"}]})))
            .unwrap();
        store.build_index("users", "name").unwrap();

        // Ensure index exists
        assert!(store.indexes().read().unwrap().contains_key("users"));

        // Write new data
        store
            .write(Arc::new(json!({"users": [{"id":1, "name":"b"}]})))
            .unwrap();

        // Index should be gone from memory
        assert!(!store.indexes().read().unwrap().contains_key("users"));
    }
}
