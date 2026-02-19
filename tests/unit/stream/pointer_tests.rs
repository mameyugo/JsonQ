#[cfg(test)]
mod tests {
    use jsonq::stream::pointer::JsonPointer;

    #[test]
    fn test_parse_empty_is_root() {
        let p = JsonPointer::parse("").unwrap();
        assert!(p.is_root());
        assert_eq!(0, p.depth());
        assert_eq!(0, p.tokens.len());
    }

    #[test]
    fn test_parse_single_key() {
        let p = JsonPointer::parse("/users").unwrap();
        assert_eq!(vec!["users"], p.tokens);
        assert_eq!(1, p.depth());
    }

    #[test]
    fn test_parse_nested_keys() {
        let p = JsonPointer::parse("/company/departments/0").unwrap();
        assert_eq!(vec!["company", "departments", "0"], p.tokens);
        assert_eq!(3, p.depth());
    }

    #[test]
    fn test_parse_tilde_1_decodes_to_slash() {
        let p = JsonPointer::parse("/a~1b").unwrap();
        assert_eq!(vec!["a/b"], p.tokens);
    }

    #[test]
    fn test_parse_tilde_0_decodes_to_tilde() {
        let p = JsonPointer::parse("/a~0b").unwrap();
        assert_eq!(vec!["a~b"], p.tokens);
    }

    #[test]
    fn test_parse_combined_tilde_escaping() {
        let p = JsonPointer::parse("/a~0~1b").unwrap();
        assert_eq!(vec!["a~/b"], p.tokens);
    }

    #[test]
    fn test_parse_missing_leading_slash_fails() {
        let result = JsonPointer::parse("users");
        assert!(result.is_err(), "Should fail without leading slash");
    }

    #[test]
    fn test_parse_root_slash_only() {
        // Single slash → one empty-string token (key "" at root)
        let p = JsonPointer::parse("/").unwrap();
        assert_eq!(vec![""], p.tokens);
    }

    #[test]
    fn test_display_roundtrip() {
        let inputs = ["/users", "/a~1b", "/a~0b", "/company/departments/0", ""];
        for input in inputs {
            let p = JsonPointer::parse(input).unwrap();
            assert_eq!(input, p.to_string());
        }
    }
}
