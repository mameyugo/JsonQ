//! Unit tests for advanced query features

#[cfg(test)]
mod tests {
    use jsonq::query::path::PathSegment;
    use jsonq::query::executor::QueryExecutor;
    use jsonq::query::regex_safe::is_match;
    use serde_json::json;

    #[test]
    fn test_recursive_descent() {
        let data = json!({
            "users": [
                {
                    "name": "Alice",
                    "profile": { "bio": "Rustacean" }
                },
                {
                    "name": "Bob",
                    "profile": { "bio": "PHPer" }
                }
            ],
            "settings": {
                "theme": "dark"
            }
        });

        let executor = QueryExecutor::new();

        // Deep find 'bio'
        let segments = PathSegment::parse_json_path("..bio").unwrap();
        let results = executor.execute_path(&data, &segments);
        assert_eq!(results.len(), 2);
        assert!(results.contains(&json!("Rustacean")));
        assert!(results.contains(&json!("PHPer")));

        // Deep find 'theme'
        let segments = PathSegment::parse_json_path("..theme").unwrap();
        let results = executor.execute_path(&data, &segments);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], json!("dark"));
    }

    #[test]
    fn test_wildcard_navigation() {
        let data = json!({
            "inventory": {
                "item1": { "price": 8.95 },
                "item2": { "price": 12.99 },
                "item3": { "price": 19.95 }
            }
        });

        let executor = QueryExecutor::new();

        // Get all prices in inventory.*.price
        let segments = PathSegment::parse_json_path("inventory.*.price").unwrap();
        let results = executor.execute_path(&data, &segments);
        assert_eq!(results.len(), 3);
        assert!(results.contains(&json!(8.95)));
        assert!(results.contains(&json!(12.99)));
        assert!(results.contains(&json!(19.95)));
    }

    #[test]
    fn test_regex_safety_backtracking() {
        let pattern = "^(a+)+$";
        let input = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa!"; 
        
        let result = is_match(input, pattern);
        assert_eq!(result, false);
    }

    #[test]
    fn test_visual_error_context() {
        let invalid_path = "users[0:10:"; 
        let error = PathSegment::parse_json_path(invalid_path).unwrap_err();
        
        let error_msg = format!("{}", error);
        assert!(error_msg.contains("users[0:10:"));
        assert!(error_msg.contains("^"));
    }

    #[test]
    fn test_utf8_simd_validation() {
        use jsonq::validation::utf8::validate_utf8;

        // Valid UTF-8
        let valid = "Hello, 🦀!".as_bytes();
        assert!(validate_utf8(valid));

        // Invalid UTF-8 (broken crab emoji)
        let invalid = vec![0xf0, 0x9f, 0x90]; // Missing last byte
        assert!(!validate_utf8(&invalid));
    }
}
