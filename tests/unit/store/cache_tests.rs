//! Tests for CachedData

use jsonq::store::CachedData;
use serde_json::json;
use std::sync::Arc;

#[test]
fn test_cache_creation() {
    let data = Arc::new(json!({"key": "value"}));
    let cache = CachedData::new(data.clone(), 1234567890);
    
    assert_eq!(cache.mtime, 1234567890);
    assert_eq!(*cache.data, json!({"key": "value"}));
}

#[test]
fn test_cache_is_valid_same_mtime() {
    let data = Arc::new(json!({}));
    let cache = CachedData::new(data, 1000);
    
    assert!(cache.is_valid(1000));
}

#[test]
fn test_cache_is_invalid_different_mtime() {
    let data = Arc::new(json!({}));
    let cache = CachedData::new(data, 1000);
    
    assert!(!cache.is_valid(2000));
    assert!(!cache.is_valid(999));
}

#[test]
fn test_cache_clone() {
    let data = Arc::new(json!({"test": 123}));
    let cache1 = CachedData::new(data.clone(), 5000);
    let cache2 = cache1.clone();
    
    assert_eq!(cache1.mtime, cache2.mtime);
    assert_eq!(*cache1.data, *cache2.data);
}

#[test]
fn test_cache_arc_sharing() {
    let data = Arc::new(json!({"shared": true}));
    let cache1 = CachedData::new(data.clone(), 100);
    let cache2 = CachedData::new(data.clone(), 100);
    
    // Both caches share the same Arc
    assert!(Arc::ptr_eq(&cache1.data, &cache2.data));
}

#[test]
fn test_cache_with_complex_data() {
    let data = Arc::new(json!({
        "users": [
            {"id": 1, "name": "Alice"},
            {"id": 2, "name": "Bob"}
        ],
        "meta": {
            "total": 2,
            "version": "1.0"
        }
    }));
    
    let cache = CachedData::new(data, 999);
    assert!(cache.is_valid(999));
    assert_eq!(cache.data["users"][0]["name"], "Alice");
}
