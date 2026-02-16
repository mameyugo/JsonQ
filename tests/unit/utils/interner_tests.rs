#[cfg(test)]
mod tests {
    use jsonq::utils::interner::KeyInterner;
    use std::sync::Arc;

    #[test]
    fn test_interner_deduplication() {
        let mut interner = KeyInterner::new();
        
        // Intern two identical strings
        let s1 = interner.intern("name");
        let s2 = interner.intern("name");
        
        // They should be the exact same reference
        assert!(Arc::ptr_eq(&s1, &s2));
        assert_eq!(Arc::strong_count(&s1), 3); // 1 in s1, 1 in s2, 1 in interner cache
        
        // Intern a different string
        let s3 = interner.intern("age");
        assert!(!Arc::ptr_eq(&s1, &s3));
    }

    #[test]
    fn test_interner_stats() {
        let mut interner = KeyInterner::new();
        
        interner.intern("a");
        interner.intern("a");
        interner.intern("b");
        
        let stats = interner.stats();
        assert_eq!(stats.unique_keys, 2);
        assert_eq!(stats.total_references, 3); // "a" (2 refs) + "b" (1 ref)
    }

    #[test]
    fn test_interner_clear() {
        let mut interner = KeyInterner::new();
        interner.intern("a");
        assert_eq!(interner.stats().unique_keys, 1);
        
        interner.clear();
        assert_eq!(interner.stats().unique_keys, 0);
    }
}
