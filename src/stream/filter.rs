use serde_json::Value;
use crate::query::matches;
use crate::error::Result;
use crate::stream::reader::StreamReader;

/// Filters applicable on a stream of items
#[derive(Clone)]
pub struct StreamFilter {
    conditions: Option<Value>,
    select_fields: Option<Vec<String>>,
    limit: Option<usize>,
    skip: Option<usize>,
}

impl StreamFilter {
    pub fn new() -> Self {
        Self {
            conditions: None,
            select_fields: None,
            limit: None,
            skip: None,
        }
    }

    pub fn with_conditions(mut self, conditions: Value) -> Self {
        if !conditions.as_object().map_or(true, |o| o.is_empty()) {
            self.conditions = Some(conditions);
        }
        self
    }

    pub fn with_select(mut self, fields: Vec<String>) -> Self {
        if !fields.is_empty() {
            self.select_fields = Some(fields);
        }
        self
    }

    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn with_skip(mut self, skip: usize) -> Self {
        self.skip = Some(skip);
        self
    }

    /// Check if item matches filter conditions
    pub fn matches(&self, item: &Value) -> bool {
        if let Some(cond) = &self.conditions {
            return matches(item, cond);
        }
        true
    }

    /// Apply selection (projection) to an item
    pub fn project(&self, value: Value) -> Value {
        if let Some(fields) = &self.select_fields {
            if let Value::Object(map) = value {
                let mut new_map = serde_json::Map::new();
                for field in fields {
                    if let Some(v) = map.get(field) {
                        new_map.insert(field.clone(), v.clone());
                    }
                }
                return Value::Object(new_map);
            }
        }
        value
    }

    /// Apply filter: check conditions and apply projection.
    /// Returns Some(projected_item) if item passes, None if filtered out.
    pub fn apply(&self, item: Value) -> Option<Value> {
        if !self.matches(&item) {
            return None;
        }
        Some(self.project(item))
    }
}

/// Iterator that wraps a StreamReader and applies filters
pub struct FilteredStream {
    inner: StreamReader,
    filter: StreamFilter,
    yielded: usize,
    skipped: usize,
}

impl FilteredStream {
    pub fn new(inner: StreamReader, filter: StreamFilter) -> Self {
        Self {
            inner,
            filter,
            yielded: 0,
            skipped: 0,
        }
    }
}

impl Iterator for FilteredStream {
    type Item = Result<Value>;

    fn next(&mut self) -> Option<Self::Item> {
        // Check limit
        if let Some(limit) = self.filter.limit {
            if self.yielded >= limit {
                return None;
            }
        }

        while let Some(result) = self.inner.next() {
            match result {
                Ok(item) => {
                    // Check conditions
                    if !self.filter.matches(&item) {
                        continue;
                    }

                    // Handle Skip
                    if let Some(skip) = self.filter.skip {
                        if self.skipped < skip {
                            self.skipped += 1;
                            continue;
                        }
                    }

                    // Apply projection
                    let projected = self.filter.project(item);
                    
                    self.yielded += 1;
                    return Some(Ok(projected));
                }
                Err(e) => return Some(Err(e)),
            }
        }
        None
    }
}
