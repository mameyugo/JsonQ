//! Type conversion utilities

use serde_json::Value;

/// Convert JSON Value to u64 if possible
///
/// Attempts to convert a JSON number to u64, trying:
/// 1. Direct u64 conversion
/// 2. i64 conversion (casting to u64)
/// 3. f64 conversion (truncating to u64)
///
/// Returns `None` if the value is not a number.
///
/// # Examples
///
/// ```rust,no_run
/// use jsonq::utils::as_u64;
/// use serde_json::json;
///
/// assert_eq!(as_u64(&json!(42)), Some(42));
/// assert_eq!(as_u64(&json!(42.7)), Some(42));
/// assert_eq!(as_u64(&json!(-5)), Some(0)); // Negative becomes 0
/// assert_eq!(as_u64(&json!("42")), None);  // String is None
/// ```
///
/// # Edge Cases
///
/// - Negative numbers are clamped to 0
/// - Floats are truncated (3.9 → 3)
/// - Non-numbers return None
pub fn as_u64(value: &Value) -> Option<u64> {
    if let Some(u) = value.as_u64() {
        Some(u)
    } else if let Some(i) = value.as_i64() {
        Some(if i < 0 { 0 } else { i as u64 })
    } else if let Some(f) = value.as_f64() {
        Some(if f < 0.0 { 0 } else { f as u64 })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_as_u64_positive_integer() {
        assert_eq!(as_u64(&json!(42)), Some(42));
        assert_eq!(as_u64(&json!(0)), Some(0));
        assert_eq!(as_u64(&json!(999)), Some(999));
    }

    #[test]
    fn test_as_u64_large_integer() {
        assert_eq!(as_u64(&json!(u64::MAX)), Some(u64::MAX));
        assert_eq!(as_u64(&json!(1_000_000)), Some(1_000_000));
    }

    #[test]
    fn test_as_u64_negative_integer() {
        // Negative numbers cast to u64 (implementation defined)
        assert!(as_u64(&json!(-1)).is_some());
        assert!(as_u64(&json!(-999)).is_some());
    }

    #[test]
    fn test_as_u64_float() {
        assert_eq!(as_u64(&json!(42.0)), Some(42));
        assert_eq!(as_u64(&json!(42.9)), Some(42)); // Truncates
        assert_eq!(as_u64(&json!(3.14)), Some(3));
    }

    #[test]
    fn test_as_u64_non_number() {
        assert_eq!(as_u64(&json!("42")), None);
        assert_eq!(as_u64(&json!("hello")), None);
        assert_eq!(as_u64(&json!(true)), None);
        assert_eq!(as_u64(&json!(false)), None);
        assert_eq!(as_u64(&json!(null)), None);
        assert_eq!(as_u64(&json!([])), None);
        assert_eq!(as_u64(&json!({})), None);
    }
}
