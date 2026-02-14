//! Tests for StoreInner
//!
//! These are integration-style tests that require actual file system access

use jsonq::store::{StoreInner, StoreOpts};
use serde_json::json;
use std::fs;
use tempfile::TempDir;

fn temp_store() -> (StoreInner, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("test.json");
    let store = StoreInner::new(path.to_str().unwrap().to_string()).unwrap();
    (store, temp_dir)
}

#[test]
fn test_store_creation() {
    let (store, _temp) = temp_store();
    
    // File should exist after creation
    assert!(store.path().exists());
    
    // Should contain empty JSON object
    let content = fs::read_to_string(store.path()).unwrap();
    assert_eq!(content.trim(), "{}");
}

#[test]
fn test_store_creates_parent_directories() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("nested/deep/data.json");
    
    let store = StoreInner::new(path.to_str().unwrap().to_string()).unwrap();
    
    assert!(store.path().exists());
    assert!(store.path().parent().unwrap().exists());
}

#[test]
fn test_read_initial_data() {
    let (store, _temp) = temp_store();
    
    let data = store.read().unwrap();
    assert_eq!(*data, json!({}));
}

#[test]
fn test_write_and_read() {
    let (store, _temp) = temp_store();
    
    let test_data = json!({"key": "value", "number": 42});
    store.write(&test_data).unwrap();
    
    let read_data = store.read().unwrap();
    assert_eq!(*read_data, test_data);
}

#[test]
fn test_write_persists_to_disk() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("persist.json");
    let path_str = path.to_str().unwrap().to_string();
    
    {
        let store = StoreInner::new(path_str.clone()).unwrap();
        store.write(&json!({"persisted": true})).unwrap();
    }
    
    // Create new store instance
    let store2 = StoreInner::new(path_str).unwrap();
    let data = store2.read().unwrap();
    assert_eq!(data["persisted"], true);
}

#[test]
fn test_cache_validity() {
    let (store, _temp) = temp_store();
    
    store.write(&json!({"cached": true})).unwrap();
    
    // First read (cache miss)
    let data1 = store.read().unwrap();
    
    // Second read (should hit cache)
    let data2 = store.read().unwrap();
    
    // Should be same Arc instance
    assert!(std::sync::Arc::ptr_eq(&data1, &data2));
}

#[test]
fn test_mutate() {
    let (store, _temp) = temp_store();
    
    store.write(&json!({"counter": 0})).unwrap();
    
    store.mutate(|data| {
        data["counter"] = json!(42);
    }).unwrap();
    
    let result = store.read().unwrap();
    assert_eq!(result["counter"], 42);
}

#[test]
fn test_options_pretty_print() {
    let (store, _temp) = temp_store();
    
    store.set_opts(StoreOpts {
        pretty: true,
        fsync: false,
    });
    
    store.write(&json!({"key": "value"})).unwrap();
    
    let content = fs::read_to_string(store.path()).unwrap();
    assert!(content.contains("\n"), "Pretty print should have newlines");
}

#[test]
fn test_options_compact() {
    let (store, _temp) = temp_store();
    
    store.set_opts(StoreOpts {
        pretty: false,
        fsync: false,
    });
    
    store.write(&json!({"key": "value"})).unwrap();
    
    let content = fs::read_to_string(store.path()).unwrap();
    assert!(!content.contains("\n"), "Compact should have no newlines");
}

#[test]
fn test_transaction_write_buffering() {
    let (store, _temp) = temp_store();
    
    store.write(&json!({"version": 1})).unwrap();
    
    // Begin transaction
    store.begin_transaction().unwrap();
    assert!(store.in_transaction());
    
    // Write in transaction (buffered, not on disk)
    store.write(&json!({"version": 2})).unwrap();
    
    // Data in transaction buffer
    let tx_data = store.read().unwrap();
    assert_eq!(tx_data["version"], 2);
    
    // But disk still has old data
    let disk_data: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(store.path()).unwrap()
    ).unwrap();
    assert_eq!(disk_data["version"], 1);
}

#[test]
fn test_transaction_commit() {
    let (store, _temp) = temp_store();
    
    store.write(&json!({"committed": false})).unwrap();
    
    store.begin_transaction().unwrap();
    store.write(&json!({"committed": true})).unwrap();
    store.commit().unwrap();
    
    assert!(!store.in_transaction());
    
    let data = store.read().unwrap();
    assert_eq!(data["committed"], true);
}

#[test]
fn test_transaction_rollback() {
    let (store, _temp) = temp_store();
    
    store.write(&json!({"original": true})).unwrap();
    
    store.begin_transaction().unwrap();
    store.write(&json!({"modified": true})).unwrap();
    store.rollback().unwrap();
    
    assert!(!store.in_transaction());
    
    // Should have original data
    let data = store.read().unwrap();
    assert_eq!(data["original"], true);
    assert!(data.get("modified").is_none());
}

#[test]
fn test_mtime_tracking() {
    let (store, _temp) = temp_store();
    
    let mtime1 = store.mtime();
    
    std::thread::sleep(std::time::Duration::from_millis(10));
    
    store.write(&json!({"updated": true})).unwrap();
    
    let mtime2 = store.mtime();
    
    // mtime should change after write
    assert!(mtime2 >= mtime1);
}

#[test]
fn test_atomic_write_safety() {
    let (store, _temp) = temp_store();
    
    store.write(&json!({"safe": true})).unwrap();
    
    // Temp file should be cleaned up
    let tmp_path = store.path().with_extension("tmp");
    assert!(!tmp_path.exists());
}

#[test]
fn test_large_data_write_and_read() {
    let (store, _temp) = temp_store();
    
    let large_data = json!({
        "users": (0..1000).map(|i| json!({
            "id": i,
            "name": format!("User {}", i),
            "active": i % 2 == 0
        })).collect::<Vec<_>>()
    });
    
    store.write(&large_data).unwrap();
    let read = store.read().unwrap();
    
    assert_eq!(read["users"].as_array().unwrap().len(), 1000);
}
