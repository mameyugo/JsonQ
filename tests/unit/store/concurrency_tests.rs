//! Concurrency tests for file locking
//!
//! These tests verify that file locks prevent race conditions
//! between multiple threads/processes.

use jsonq::store::StoreInner;
use serde_json::json;
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::Duration;
use tempfile::TempDir;

fn temp_store() -> (StoreInner, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("concurrent.json");
    // Ensure the path is absolute and reachable
    let path_str = path.to_str().unwrap().to_owned();
    let store = StoreInner::new(path_str);
    (store, temp_dir)
}

#[test]
fn test_concurrent_reads_allowed() {
    let (store, _temp) = temp_store();
    let store = Arc::new(store);
    
    // Initialize with test data
    store.write(&json!({"counter": 0})).unwrap();
    
    let barrier = Arc::new(Barrier::new(10));
    let mut handles = vec![];
    
    // Spawn 10 concurrent readers
    for _ in 0..10 {
        let store_clone = Arc::clone(&store);
        let barrier_clone = Arc::clone(&barrier);
        
        let handle = thread::spawn(move || {
            // Wait for all threads to be ready
            barrier_clone.wait();
            
            // All should be able to read concurrently
            let data = store_clone.read().unwrap();
            assert_eq!(data["counter"], 0);
        });
        
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn test_write_blocks_concurrent_writes() {
    let (store, _temp) = temp_store();
    let store = Arc::new(store);
    
    store.write(&json!({"counter": 0})).unwrap();
    
    let barrier = Arc::new(Barrier::new(5));
    let mut handles = vec![];
    
    // Spawn 5 concurrent writers
    for _ in 0..5 {
        let store_clone = Arc::clone(&store);
        let barrier_clone = Arc::clone(&barrier);
        
        let handle = thread::spawn(move || {
            barrier_clone.wait();
            
            // Each thread increments the counter
            store_clone.mutate(|data| {
                let current = data["counter"].as_i64().unwrap_or(0);
                data["counter"] = json!(current + 1);
                
                // Add small delay to increase chance of race if locks don't work
                thread::sleep(Duration::from_millis(10));
            }).unwrap();
        });
        
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    // If locks work correctly, counter should be exactly 5
    let final_data = store.read().unwrap();
    assert_eq!(
        final_data["counter"], 
        5, 
        "Counter should be 5 if locks prevent race conditions"
    );
}

#[test]
fn test_concurrent_read_modify_write() {
    let (store, _temp) = temp_store();
    let store = Arc::new(store);
    
    store.write(&json!({"value": 0})).unwrap();
    
    let iterations = 20;
    let mut handles = vec![];
    
    for _ in 0..iterations {
        let store_clone = Arc::clone(&store);
        
        let handle = thread::spawn(move || {
            store_clone.mutate(|data| {
                let current = data["value"].as_i64().unwrap_or(0);
                data["value"] = json!(current + 1);
            }).unwrap();
        });
        
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    let final_data = store.read().unwrap();
    assert_eq!(
        final_data["value"], 
        iterations as i64,
        "All increments should be atomic"
    );
}

#[test]
fn test_write_invalidates_other_process_cache() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("shared.json");
    let path_str = path.to_str().unwrap().to_string();
    
    // Create two separate store instances (simulating different processes)
    let store1 = Arc::new(StoreInner::new(path_str.clone()));
    let store2 = Arc::new(StoreInner::new(path_str.clone()));
    
    // Store1 writes initial data
    store1.write(&json!({"version": 1})).unwrap();
    
    // Store2 reads and caches it
    let data2_cached = store2.read().unwrap();
    assert_eq!(data2_cached["version"], 1);
    
    // Ensure mtime will be different (at least 1s on many systems)
    thread::sleep(Duration::from_secs(1));
    
    // Store1 updates data
    store1.write(&json!({"version": 2})).unwrap();
    
    // Store2 should detect the change via mtime and re-read
    let data2_fresh = store2.read().unwrap();
    assert_eq!(data2_fresh["version"], 2, "Cache should be invalidated");
}

#[test]
fn test_reader_writer_exclusion() {
    let (store, _temp) = temp_store();
    let store = Arc::new(store);
    
    store.write(&json!({"data": "initial"})).unwrap();
    
    let store_clone = Arc::clone(&store);
    
    // Start a long-running write in background
    let writer = thread::spawn(move || {
        store_clone.mutate(|data| {
            data["data"] = json!("being written");
            // Simulate slow write
            thread::sleep(Duration::from_millis(150));
            data["data"] = json!("written");
        }).unwrap();
    });
    
    // Give writer time to acquire lock
    thread::sleep(Duration::from_millis(50));
    
    // Try to read - should block until write completes
    let start = std::time::Instant::now();
    let data = store.read().unwrap();
    let elapsed = start.elapsed();
    
    // Read should have been blocked
    assert!(elapsed >= Duration::from_millis(50), "Read should block during write");
    
    // Should see the final written value
    assert_eq!(data["data"], "written");
    
    writer.join().unwrap();
}
