use crate::query::error::QueryError;
use crate::utils::string::suggest_similar;
use serde_json::Value;

fn get_context(input: &str, position: usize) -> String {
    let start = position.saturating_sub(10);
    let end = (position + 10).min(input.len());
    let snippet = &input[start..end];
    let padding = " ".repeat(position - start);
    format!("\n    {}\n    {}^", snippet, padding)
}

#[derive(Debug, PartialEq, Clone)]
pub enum PathSegment {
    Key(String),
    Index(i64),
    Wildcard,
    RecursiveDescent(String),

    // Advanced Slice
    Slice {
        start: Option<i64>,
        end: Option<i64>,
        step: Option<i64>,
    },

    // Multi-key selection
    MultiKey(Vec<String>),
    // Placeholder for filter (Phase 3.3)
    // Filter(FilterExpr),
}

impl PathSegment {
    /// Parse slice notation: [start:end:step]
    /// Examples: [0:10:2], [::3], [5:], [:10], [:-1]
    /// Parse slice notation: [start:end:step]
    /// Examples: [0:10:2], [::3], [5:], [:10], [:-1]
    pub fn parse_slice(input: &str) -> Result<Self, QueryError> {
        let parts: Vec<&str> = input.split(':').collect();

        if parts.len() > 3 {
            return Err(QueryError::new("Invalid slice: too many colons", 0)
                .with_context(format!("Input: {}", input))
                .with_suggestion("Slice format is [start:end:step]"));
        }

        let parse_int = |s: &str| -> Option<i64> {
            if s.trim().is_empty() {
                None
            } else {
                s.trim().parse().ok()
            }
        };

        let start = parts.get(0).and_then(|s| parse_int(s));
        let end = parts.get(1).and_then(|s| parse_int(s));
        let step = parts.get(2).and_then(|s| parse_int(s));

        // Validation: step cannot be 0
        if let Some(0) = step {
            return Err(QueryError::new("Slice step cannot be zero", 0));
        }

        Ok(PathSegment::Slice { start, end, step })
    }

    /// Apply slice to an array
    pub fn apply_slice(
        &self,
        array: &[Value],
        start: Option<i64>,
        end: Option<i64>,
        step: Option<i64>,
    ) -> Vec<Value> {
        let len = array.len() as i64;
        let step = step.unwrap_or(1);

        if step == 0 {
            return vec![];
        }

        let normalize = |idx: i64| -> i64 {
            if idx < 0 {
                (len + idx).max(0)
            } else {
                idx.min(len)
            }
        };

        if step > 0 {
            let start_idx = start.map(normalize).unwrap_or(0) as usize;
            let end_idx = end.map(normalize).unwrap_or(len) as usize;

            if start_idx >= len as usize || start_idx >= end_idx {
                return vec![];
            }

            array[start_idx..end_idx.min(len as usize)]
                .iter()
                .step_by(step as usize)
                .cloned()
                .collect()
        } else {
            // Negative step logic (Python-style)
            let start_idx = start.map(|idx| {
                if idx < 0 { (len + idx).max(0) } else { idx.min(len - 1) }
            }).unwrap_or(len - 1);

            let end_idx = end.map(|idx| {
                if idx < 0 { (len + idx).max(-1) } else { idx.min(len) }
            }).unwrap_or(-1);

            let mut res = Vec::new();
            let mut curr = start_idx;

            while curr > end_idx && curr >= 0 && curr < len {
                if let Some(v) = array.get(curr as usize) {
                    res.push(v.clone());
                }
                curr += step;
            }
            res
        }
    }

    /// Parse multi-key selector: ["key1", "key2"]
    pub fn parse_multi_key(input: &str) -> Result<Self, QueryError> {
        let cleaned: Vec<String> = input
            .trim_matches(|c| c == '[' || c == ']')
            .split(',')
            .map(|s| s.trim().trim_matches('\'').trim_matches('"').to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if cleaned.is_empty() {
            return Err(QueryError::new("Multi-key selector cannot be empty", 0));
        }

        Ok(PathSegment::MultiKey(cleaned))
    }

    /// Apply multi-key to an object
    pub fn apply_multi_key(&self, obj: &serde_json::Map<String, Value>, keys: &[String]) -> Value {
        let mut result = serde_json::Map::new();

        for key in keys {
            if let Some(value) = obj.get(key) {
                result.insert(key.clone(), value.clone());
            }
        }

        Value::Object(result)
    }

    /// Parse a full JSONPath string into segments
    /// Examples:
    /// "users.0.name"
    /// "users[0:10].name"
    /// "items[\"key1\",\"key2\"]"
    /// Parse a full JSONPath string into segments
    /// Examples:
    /// "users.0.name"
    /// "users[0:10].name"
    /// "items[\"key1\",\"key2\"]"
    pub fn parse_json_path(input: &str) -> Result<Vec<PathSegment>, QueryError> {
        let mut segments = Vec::new();
        let mut current_pos = 0;
        let chars: Vec<char> = input.chars().collect();
        let len = chars.len();

        while current_pos < len {
            match chars[current_pos] {
                '.' => {
                    current_pos += 1; // Skip dot
                    if current_pos < len && chars[current_pos] == '.' {
                        // Recursive descent ..
                        current_pos += 1;
                        let start = current_pos;
                        while current_pos < len
                            && chars[current_pos] != '.'
                            && chars[current_pos] != '['
                        {
                            current_pos += 1;
                        }
                        let key: String = if start < current_pos {
                            chars[start..current_pos].iter().collect()
                        } else {
                            // If just .., handle next segment
                            "".to_string()
                        };
                        segments.push(PathSegment::RecursiveDescent(key));
                    } else {
                        // Read key until next . or [
                        let start = current_pos;
                        while current_pos < len
                            && chars[current_pos] != '.'
                            && chars[current_pos] != '['
                        {
                            current_pos += 1;
                        }
                        if start < current_pos {
                            let key: String = chars[start..current_pos].iter().collect();
                            if key == "*" {
                                segments.push(PathSegment::Wildcard);
                            } else {
                                segments.push(PathSegment::Key(key));
                            }
                        }
                    }
                }
                '[' => {
                    let bracket_start = current_pos;
                    current_pos += 1; // Skip [
                    let start = current_pos;
                    while current_pos < len && chars[current_pos] != ']' {
                        current_pos += 1;
                    }
                    if current_pos >= len {
                        return Err(QueryError::new("Unclosed bracket", bracket_start)
                            .with_context(get_context(input, bracket_start))
                            .with_suggestion("Add ']' to close the index/slice selector"));
                    }
                    let content: String = chars[start..current_pos].iter().collect();
                    current_pos += 1; // Skip ]

                    // Determine type: Index, Slice, MultiKey, or Wildcard
                    if content == "*" {
                        segments.push(PathSegment::Wildcard);
                    } else if content.contains(':') {
                        segments.push(PathSegment::parse_slice(&content).map_err(|e| {
                            QueryError::new(e.message, bracket_start)
                                .with_context(get_context(input, bracket_start))
                                .with_suggestion("Check slice syntax [start:end:step]")
                        })?);
                    } else if content.contains(',')
                        || content.contains('"')
                        || content.contains('\'')
                    {
                        segments.push(PathSegment::parse_multi_key(&content).map_err(|e| {
                            QueryError::new(e.message, bracket_start)
                                .with_context(get_context(input, bracket_start))
                        })?);
                    } else {
                        // Index or simple key inside brackets ["key"]
                        if let Ok(idx) = content.trim().parse::<i64>() {
                            segments.push(PathSegment::Index(idx));
                        } else {
                            // Treat as key if wrapped in quotes, or if it looks like a string
                            let key = content
                                .trim()
                                .trim_matches('\'')
                                .trim_matches('"')
                                .to_string();
                            if key == "*" {
                                segments.push(PathSegment::Wildcard);
                            } else {
                                segments.push(PathSegment::Key(key));
                            }
                        }
                    }
                }
                '*' => {
                    segments.push(PathSegment::Wildcard);
                    current_pos += 1;
                }
                _ => {
                    // Start of path (no leading dot/bracket usually, or just key)
                    let start = current_pos;
                    while current_pos < len
                        && chars[current_pos] != '.'
                        && chars[current_pos] != '['
                    {
                        current_pos += 1;
                    }
                    if start < current_pos {
                        let key: String = chars[start..current_pos].iter().collect();
                        // Special case: $ is root
                        if key == "$" {
                            // Skip root marker
                        } else if key == "*" {
                            segments.push(PathSegment::Wildcard);
                        } else {
                            segments.push(PathSegment::Key(key));
                        }
                    } else {
                        // Stuck?
                        current_pos += 1;
                    }
                }
            }
        }

        Ok(segments)
    }
}
