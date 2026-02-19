use std::fmt;

/// Parsed JSON Pointer following RFC 6901
/// https://datatracker.ietf.org/doc/html/rfc6901
///
/// Examples:
/// - ""         → root document
/// - "/users"   → key "users" at root
/// - "/users/0" → first element of "users" array
/// - "/a~1b"    → key "a/b" (~ escaping)
/// - "/a~0b"    → key "a~b"
#[derive(Debug, Clone, PartialEq)]
pub struct JsonPointer {
    pub tokens: Vec<String>,
}

impl JsonPointer {
    /// Parse a JSON Pointer string
    /// Returns Err if pointer doesn't start with "/" (unless empty)
    pub fn parse(input: &str) -> Result<Self, String> {
        if input.is_empty() {
            return Ok(JsonPointer { tokens: vec![] });
        }
        if !input.starts_with('/') {
            return Err(format!(
                "JSON Pointer must start with '/' or be empty, got: {:?}",
                input
            ));
        }
        let tokens = input[1..]
            .split('/')
            .map(|t| t.replace("~1", "/").replace("~0", "~"))
            .collect();
        Ok(JsonPointer { tokens })
    }

    pub fn is_root(&self) -> bool {
        self.tokens.is_empty()
    }

    pub fn depth(&self) -> usize {
        self.tokens.len()
    }
}

impl fmt::Display for JsonPointer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_root() {
            return write!(f, "");
        }
        for token in &self.tokens {
            write!(f, "/{}", token.replace("~", "~0").replace("/", "~1"))?;
        }
        Ok(())
    }
}
