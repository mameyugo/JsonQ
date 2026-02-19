#[cfg(test)]
mod tests {
    use jsonq::stream::filter::StreamFilter;
    use serde_json::json;

    #[test]
    fn test_filter_no_conditions_passes_all() {
        let filter = StreamFilter::new();
        assert!(filter.apply(json!({"id": 1})).is_some());
        assert!(filter.apply(json!({"id": 2})).is_some());
    }

    #[test]
    fn test_filter_eq_condition() {
        let filter = StreamFilter::new()
            .with_conditions(json!({"role": {"$eq": "admin"}}));
        assert!(filter.apply(json!({"id": 1, "role": "admin"})).is_some());
        assert!(filter.apply(json!({"id": 2, "role": "viewer"})).is_none());
    }

    #[test]
    fn test_filter_gt_condition() {
        let filter = StreamFilter::new()
            .with_conditions(json!({"age": {"$gt": 30}}));
        assert!(filter.apply(json!({"age": 35})).is_some());
        assert!(filter.apply(json!({"age": 30})).is_none());
        assert!(filter.apply(json!({"age": 25})).is_none());
    }

    #[test]
    fn test_filter_in_condition() {
        let filter = StreamFilter::new()
            .with_conditions(json!({"city": {"$in": ["NYC", "LA"]}}));
        assert!(filter.apply(json!({"city": "NYC"})).is_some());
        assert!(filter.apply(json!({"city": "Chicago"})).is_none());
    }

    #[test]
    fn test_filter_select_projection() {
        let filter = StreamFilter::new()
            .with_select(vec!["id".to_string(), "name".to_string()]);
        let input = json!({"id": 1, "name": "Alice", "age": 30, "email": "a@x.com"});
        let output = filter.apply(input).unwrap();
        assert!(output.get("id").is_some());
        assert!(output.get("name").is_some());
        assert!(output.get("age").is_none());
        assert!(output.get("email").is_none());
    }

    #[test]
    fn test_filter_select_on_non_object_returns_unchanged() {
        let filter = StreamFilter::new()
            .with_select(vec!["id".to_string()]);
        // Primitive value (not an object) should return as-is
        let result = filter.apply(json!(42));
        assert_eq!(Some(json!(42)), result);
    }

    #[test]
    fn test_filter_combined_condition_and_select() {
        let filter = StreamFilter::new()
            .with_conditions(json!({"active": {"$eq": true}}))
            .with_select(vec!["id".to_string()]);

        let matching = json!({"id": 1, "active": true, "name": "Alice"});
        let result = filter.apply(matching).unwrap();
        assert!(result.get("id").is_some());
        assert!(result.get("name").is_none());

        let not_matching = json!({"id": 2, "active": false, "name": "Bob"});
        assert!(filter.apply(not_matching).is_none());
    }

    #[test]
    fn test_filter_empty_conditions_object_is_ignored() {
        // Empty conditions object {} should not filter anything
        let filter = StreamFilter::new()
            .with_conditions(json!({}));
        assert!(filter.apply(json!({"id": 1})).is_some());
    }
}
