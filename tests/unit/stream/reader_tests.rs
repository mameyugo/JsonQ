#[cfg(test)]
mod tests {
    use jsonq::stream::StreamReader;
    use serde_json::json;
    use std::io::Write;

    fn write_temp(content: &str) -> tempfile::NamedTempFile {
        let mut f = tempfile::Builder::new().suffix(".json").tempfile().unwrap();
        write!(f, "{}", content).unwrap();
        f
    }

    #[test]
    fn test_stream_root_array() {
        let f = write_temp(r#"[{"id":1},{"id":2},{"id":3}]"#);
        let reader = StreamReader::new(f.path().to_str().unwrap(), "").unwrap();
        let items: Vec<_> = reader.collect::<Result<Vec<_>, _>>().unwrap();
        assert_eq!(3, items.len());
        assert_eq!(json!(1), items[0]["id"]);
    }

    #[test]
    fn test_stream_nested_array() {
        let f = write_temp(r#"{"users":[{"id":1,"name":"Alice"},{"id":2,"name":"Bob"}]}"#);
        let reader = StreamReader::new(f.path().to_str().unwrap(), "/users").unwrap();
        let items: Vec<_> = reader.collect::<Result<Vec<_>, _>>().unwrap();
        assert_eq!(2, items.len());
        assert_eq!("Alice", items[0]["name"].as_str().unwrap());
    }

    #[test]
    fn test_stream_empty_array() {
        let f = write_temp(r#"{"items":[]}"#);
        let reader = StreamReader::new(f.path().to_str().unwrap(), "/items").unwrap();
        let items: Vec<_> = reader.collect::<Result<Vec<_>, _>>().unwrap();
        assert_eq!(0, items.len());
    }

    #[test]
    fn test_stream_deeply_nested() {
        let f = write_temp(r#"{"a":{"b":{"c":[{"id":1},{"id":2}]}}}"#);
        let reader = StreamReader::new(f.path().to_str().unwrap(), "/a/b/c").unwrap();
        let items: Vec<_> = reader.collect::<Result<Vec<_>, _>>().unwrap();
        assert_eq!(2, items.len());
    }

    #[test]
    fn test_stream_preserves_order() {
        let data: String = format!(
            r#"{{"data":[{}]}}"#,
            (1..=50).map(|i| format!(r#"{{"id":{}}}"#, i)).collect::<Vec<_>>().join(",")
        );
        let f = write_temp(&data);
        let reader = StreamReader::new(f.path().to_str().unwrap(), "/data").unwrap();
        let items: Vec<_> = reader.collect::<Result<Vec<_>, _>>().unwrap();
        assert_eq!(50, items.len());
        for (i, item) in items.iter().enumerate() {
            assert_eq!((i + 1) as i64, item["id"].as_i64().unwrap());
        }
    }

    #[test]
    fn test_stream_by_numeric_index() {
        let f = write_temp(r#"{"items":[[1,2,3],[4,5,6]]}"#);
        let reader = StreamReader::new(f.path().to_str().unwrap(), "/items/1").unwrap();
        let items: Vec<_> = reader.collect::<Result<Vec<_>, _>>().unwrap();
        assert_eq!(3, items.len());
        assert_eq!(json!(4), items[0]);
    }

    #[test]
    fn test_stream_nonexistent_key_returns_error() {
        let f = write_temp(r#"{"users":[]}"#);
        // Either new() fails, or first next() is Err
        match StreamReader::new(f.path().to_str().unwrap(), "/nonexistent") {
            Err(_) => {} // ok
            Ok(mut r) => {
                // The error may surface on first call
                let first: Option<jsonq::error::Result<serde_json::Value>> = r.next();
                assert!(first.map_or(true, |res| res.is_err()),
                    "Expected error for nonexistent key");
            }
        }
    }

    #[test]
    fn test_stream_invalid_json_returns_error() {
        let f = write_temp("not valid json {{{");
        match StreamReader::new(f.path().to_str().unwrap(), "/users") {
            Err(_) => {}
            Ok(mut r) => {
                let first: Option<jsonq::error::Result<serde_json::Value>> = r.next();
                assert!(first.map_or(false, |res| res.is_err()),
                    "Expected parse error");
            }
        }
    }
}
