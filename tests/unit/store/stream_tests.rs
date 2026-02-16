#[cfg(test)]
mod tests {
    use jsonq::store::StoreInner;
    use serde_json::json;
    use std::fs;
    use std::io::Write;
    use std::sync::Arc;
    use tempfile::NamedTempFile;

    #[test]
    fn test_append_jsonl() {
        let tmp = tempfile::Builder::new()
            .suffix(".jsonl")
            .tempfile()
            .unwrap();
        let path = tmp.path().to_str().unwrap().to_string();
        let store = StoreInner::new(path.clone()).unwrap();

        let record1 = json!({"id": 1, "val": "a"});
        let record2 = json!({"id": 2, "val": "b"});

        store.append_jsonl(&record1).unwrap();
        store.append_jsonl(&record2).unwrap();

        // Verify file content
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains(r#"{"id":1,"val":"a"}"#));
        assert!(content.contains(r#"{"id":2,"val":"b"}"#));
        assert_eq!(content.lines().count(), 2);
    }

    #[test]
    fn test_read_jsonl_iter() {
        let mut tmp = tempfile::Builder::new()
            .suffix(".jsonl")
            .tempfile()
            .unwrap();
        writeln!(tmp, "{{\"id\": 1}}").unwrap();
        writeln!(tmp, "{{\"id\": 2}}").unwrap();
        writeln!(tmp, "").unwrap(); // Empty line should be skipped
        writeln!(tmp, "{{\"id\": 3}}").unwrap();

        let path = tmp.path().to_str().unwrap().to_string();
        let store = StoreInner::new(path).unwrap();

        let records: Vec<_> = store.read_jsonl_iter().unwrap().collect();
        assert_eq!(records.len(), 3);
        assert_eq!(records[0]["id"], 1);
        assert_eq!(records[1]["id"], 2);
        assert_eq!(records[2]["id"], 3);
    }

    #[test]
    fn test_write_to_stream() {
        let tmp_src = tempfile::Builder::new()
            .suffix(".json")
            .tempfile()
            .unwrap();
        let path_src = tmp_src.path().to_str().unwrap().to_string();
        let store = StoreInner::new(path_src).unwrap();
        
        let data = json!({"users": [{"id": 1}, {"id": 2}]});
        store.write(Arc::new(data)).unwrap();

        let mut buffer = Vec::new();
        store.write_to_stream(&mut buffer).unwrap();

        let output: serde_json::Value = serde_json::from_slice(&buffer).unwrap();
        assert_eq!(output["users"].as_array().unwrap().len(), 2);
    }
}
