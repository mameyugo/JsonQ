//! SIMD-accelerated UTF-8 validation
//!
//! Uses simdutf to validate UTF-8 strings much faster than standard library.

/// Validate that a byte slice is valid UTF-8
///
/// Returns true if valid, false otherwise.
/// Uses SIMD acceleration if available.
pub fn validate_utf8(input: &[u8]) -> bool {
    // Check for empty input
    if input.is_empty() {
        return true;
    }

    // Use simdutf for validation
    simdutf::validate_utf8(input)
}

/// Validate UTF-8 and return result suitable for Result<(), Utf8Error>
pub fn ensure_utf8(input: &[u8]) -> Result<(), std::str::Utf8Error> {
    if validate_utf8(input) {
        Ok(())
    } else {
        // Fallback to std to generate the correct error with position
        std::str::from_utf8(input).map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_utf8() {
        assert!(validate_utf8(b"Hello world"));
        assert!(validate_utf8("こんにちは".as_bytes()));
        assert!(validate_utf8("🎉".as_bytes()));
    }

    #[test]
    fn test_invalid_utf8() {
        // Invalid sequence
        assert!(!validate_utf8(&[0xFF, 0xFE]));
        assert!(!validate_utf8(&[0xC3, 0x28])); // Invalid 2-byte sequence
    }
}
