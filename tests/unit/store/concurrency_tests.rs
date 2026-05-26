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
    let store = StoreInner::new(path_str).unwrap();
    (store, temp_dir)
}

#[test]
fn test_concurrent_reads_allowed() {
    let (store, _temp) = temp_store();
    let store = Arc::new(store);

    // Initialize with test data
    store.write(Arc::new(json!({"counter": 0}))).unwrap();

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

    store.write(Arc::new(json!({"counter": 0}))).unwrap();

    let barrier = Arc::new(Barrier::new(5));
    let mut handles = vec![];

    // Spawn 5 concurrent writers
    for _ in 0..5 {
        let store_clone = Arc::clone(&store);
        let barrier_clone = Arc::clone(&barrier);

        let handle = thread::spawn(move || {
            barrier_clone.wait();

            // Each thread increments the counter
            store_clone
                .mutate(|data| {
                    let current = data["counter"].as_i64().unwrap_or(0);
                    data["counter"] = json!(current + 1);

                    // Add small delay to increase chance of race if locks don't work
                    thread::sleep(Duration::from_millis(10));
                })
                .unwrap();
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // If locks work correctly, counter should be exactly 5
    let final_data = store.read().unwrap();
    assert_eq!(
        final_data["counter"], 5,
        "Counter should be 5 if locks prevent race conditions"
    );
}

#[test]
fn test_concurrent_read_modify_write() {
    let (store, _temp) = temp_store();
    let store = Arc::new(store);

    store.write(Arc::new(json!({"value": 0}))).unwrap();

    let iterations = 20;
    let mut handles = vec![];

    for _ in 0..iterations {
        let store_clone = Arc::clone(&store);

        let handle = thread::spawn(move || {
            store_clone
                .mutate(|data| {
                    let current = data["value"].as_i64().unwrap_or(0);
                    data["value"] = json!(current + 1);
                })
                .unwrap();
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let final_data = store.read().unwrap();
    assert_eq!(
        final_data["value"], iterations as i64,
        "All increments should be atomic"
    );
}

#[test]
fn test_write_invalidates_other_process_cache() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("shared.json");
    let path_str = path.to_str().unwrap().to_string();

    // Create two separate store instances (simulating different processes)
    let store1 = Arc::new(StoreInner::new(path_str.clone()).unwrap());
    let store2 = Arc::new(StoreInner::new(path_str.clone()).unwrap());

    // Store1 writes initial data
    store1.write(Arc::new(json!({"version": 1}))).unwrap();

    // Store2 reads and caches it
    let data2_cached = store2.read().unwrap();
    assert_eq!(data2_cached["version"], 1);

    // Ensure mtime will be different (at least 1s on many systems)
    thread::sleep(Duration::from_secs(1));

    // Store1 updates data
    store1.write(Arc::new(json!({"version": 2}))).unwrap();

    // Store2 should detect the change via mtime and re-read
    let data2_fresh = store2.read().unwrap();
    assert_eq!(data2_fresh["version"], 2, "Cache should be invalidated");
}

#[test]
fn test_reader_writer_exclusion() {
    let (store, _temp) = temp_store();
    let store = Arc::new(store);

    store.write(Arc::new(json!({"data": "initial"}))).unwrap();

    let store_clone = Arc::clone(&store);

    // Start a long-running write in background
    let writer = thread::spawn(move || {
        store_clone
            .mutate(|data| {
                data["data"] = json!("being written");
                // Simulate slow write
                thread::sleep(Duration::from_millis(150));
                data["data"] = json!("written");
            })
            .unwrap();
    });

    // Give writer time to acquire lock
    thread::sleep(Duration::from_millis(50));

    // Try to read - should block until write completes
    let start = std::time::Instant::now();
    let data = store.read().unwrap();
    let elapsed = start.elapsed();

    // Read should have been blocked
    assert!(
        elapsed >= Duration::from_millis(50),
        "Read should block during write"
    );

    // Should see the final written value
    assert_eq!(data["data"], "written");

    writer.join().unwrap();
}

#[test]
fn test_transaction_blocks_concurrent_writes() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("concurrent_tx_write.json");
    let path_str = path.to_str().unwrap().to_string();

    let store1 = Arc::new(StoreInner::new(path_str.clone()).unwrap());
    let store2 = Arc::new(StoreInner::new(path_str.clone()).unwrap());

    store1.write(Arc::new(json!({"value": 0}))).unwrap();

    // Start a transaction in store1 (acquires lock)
    store1.begin_transaction().unwrap();
    store1.mutate(|data| {
        data["value"] = json!(1);
    }).unwrap();

    let store2_clone = Arc::clone(&store2);
    let writer = thread::spawn(move || {
        let start = std::time::Instant::now();
        // This mutation in store2 should block until store1 commits
        store2_clone.mutate(|data| {
            data["value"] = json!(2);
        }).unwrap();
        start.elapsed()
    });

    // Give writer thread a moment to start and try to acquire the lock
    thread::sleep(Duration::from_millis(50));

    // Commit the transaction in store1
    store1.commit().unwrap();

    let elapsed = writer.join().unwrap();

    // Writer should have been blocked for at least ~30ms
    assert!(
        elapsed >= Duration::from_millis(30),
        "Writer should block during active transaction, took {:?}", elapsed
    );

    let file = std::fs::File::open(&path).unwrap();
    let final_data: serde_json::Value = serde_json::from_reader(file).unwrap();
    assert_eq!(final_data["value"], 2, "Final value should be written by store2");
}

#[test]
fn test_transaction_blocks_other_transactions() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("concurrent_tx_tx.json");
    let path_str = path.to_str().unwrap().to_string();

    let store1 = Arc::new(StoreInner::new(path_str.clone()).unwrap());
    let store2 = Arc::new(StoreInner::new(path_str.clone()).unwrap());

    store1.write(Arc::new(json!({"value": 0}))).unwrap();

    // Start transaction in store1
    store1.begin_transaction().unwrap();

    let store2_clone = Arc::clone(&store2);
    let writer = thread::spawn(move || {
        let start = std::time::Instant::now();
        // This transaction in store2 should block until store1 commits
        store2_clone.begin_transaction().unwrap();
        store2_clone.mutate(|data| {
            data["value"] = json!(2);
        }).unwrap();
        store2_clone.commit().unwrap();
        start.elapsed()
    });

    thread::sleep(Duration::from_millis(50));

    // Commit transaction in store1
    store1.commit().unwrap();

    let elapsed = writer.join().unwrap();

    assert!(
        elapsed >= Duration::from_millis(30),
        "Second transaction should block during active transaction, took {:?}", elapsed
    );

    let file = std::fs::File::open(&path).unwrap();
    let final_data: serde_json::Value = serde_json::from_reader(file).unwrap();
    assert_eq!(final_data["value"], 2);
}

#[test]
fn test_safe_transaction_recovery() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("safe_recovery.json");
    let path_str = path.to_str().unwrap().to_string();

    let store1 = Arc::new(StoreInner::new(path_str.clone()).unwrap());
    store1.write(Arc::new(json!({"status": "ok"}))).unwrap();

    // 1. Start transaction in store1 (creates safe_recovery.tx and locks path)
    store1.begin_transaction().unwrap();

    let tx_file = path.with_extension("tx");
    assert!(tx_file.exists(), "Transaction snapshot file should exist");

    // 2. Initialize a separate store2 instance (simulating another process startup)
    let _store2 = StoreInner::new(path_str.clone()).unwrap();

    // Verification: The transaction snapshot file must NOT have been deleted because store1 is active
    assert!(tx_file.exists(), "Active transaction snapshot file must NOT be deleted during recovery by another process");

    // 3. Rollback store1 transaction to release lock
    store1.rollback().unwrap();
    assert!(!tx_file.exists(), "Snapshot file should be deleted on rollback");

    // 4. Manually create an orphaned .tx file to simulate a crashed process
    std::fs::write(&tx_file, "{}").unwrap();
    assert!(tx_file.exists());

    // 5. Initialize store3. Since no lock is held, it should recover (delete) the orphaned .tx file.
    let _store3 = StoreInner::new(path_str.clone()).unwrap();
    assert!(!tx_file.exists(), "Orphaned transaction snapshot file should be deleted during recovery");
}
