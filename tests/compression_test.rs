#[cfg(test)]
mod tests {
    use jsonq::store::StoreInner;
    use jsonq::store::options::{StoreOpts, CompressionMethod};
    use serde_json::json;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_compression_writing_and_reading() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("compression_test.json");
        let path_str = path.to_str().unwrap().to_string();
        
        let store = StoreInner::new(path_str.clone()).unwrap();
        
        // 1. Write with Zstd
        let mut opts = StoreOpts::default();
        opts.compression = CompressionMethod::Zstd;
        store.set_opts(opts);
        
        let data = json!({"test": "value".repeat(100)});
        store.write(&data).unwrap();
        
        // Verify it's actually compressed (check magic header)
        let bytes = fs::read(&path).unwrap();
        assert_eq!(&bytes[0..4], &[0x28, 0xB5, 0x2F, 0xFD]);
        
        // 2. Read back
        let read_data = store.read().unwrap();
        assert_eq!(read_data.as_ref(), &data);
        
        // 3. Switch to Gzip
        let mut opts = StoreOpts::default();
        opts.compression = CompressionMethod::Gzip;
        store.set_opts(opts);
        
        store.write(&data).unwrap();
        let bytes = fs::read(&path).unwrap();
        assert_eq!(&bytes[0..2], &[0x1F, 0x8B]);
        
        let read_data_gz = store.read().unwrap();
        assert_eq!(read_data_gz.as_ref(), &data);
    }
    
    #[test]
    fn test_transparent_decompression() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("transparent_test.json");
        let path_str = path.to_str().unwrap().to_string();
        
        let store = StoreInner::new(path_str.clone()).unwrap();
        let data = json!({"hello": "world"});
        
        // Write compressed
        let mut opts = StoreOpts::default();
        opts.compression = CompressionMethod::Zstd;
        store.set_opts(opts);
        store.write(&data).unwrap();
        
        // Create a new store instance with NO compression set (it should auto-detect)
        let store2 = StoreInner::new(path_str).unwrap();
        let read_data = store2.read().unwrap();
        assert_eq!(read_data.as_ref(), &data);
    }
}
