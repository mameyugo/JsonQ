use crate::query::path::PathSegment;
use serde_json::Value;

pub struct QueryExecutor;

impl QueryExecutor {
    pub fn new() -> Self {
        Self
    }

    pub fn execute_path(&self, root: &Value, segments: &[PathSegment]) -> Vec<Value> {
        let mut current_nodes = vec![root.clone()];

        for segment in segments {
            let mut next_nodes = Vec::new();
            for node in &current_nodes {
                let mut results = self.apply_segment(node, segment);
                next_nodes.append(&mut results);
            }
            if next_nodes.is_empty() {
                return vec![];
            }
            current_nodes = next_nodes;
        }

        current_nodes
    }

    pub fn apply_segment(&self, current: &Value, segment: &PathSegment) -> Vec<Value> {
        match segment {
            PathSegment::Key(key) => {
                if let Value::Object(obj) = current {
                    if let Some(v) = obj.get(key) {
                        return vec![v.clone()];
                    }
                }
                vec![]
            }
            PathSegment::Index(idx) => {
                if let Value::Array(arr) = current {
                    let i = if *idx < 0 {
                        (arr.len() as i64 + idx) as usize
                    } else {
                        *idx as usize
                    };
                    if let Some(v) = arr.get(i) {
                        return vec![v.clone()];
                    }
                }
                vec![]
            }
            PathSegment::Wildcard => match current {
                Value::Array(arr) => arr.clone(),
                Value::Object(obj) => obj.values().cloned().collect(),
                _ => vec![],
            },
            PathSegment::RecursiveDescent(key) => {
                let mut results = Vec::new();
                self.recursive_find(current, key, &mut results);
                results
            }
            PathSegment::Slice { start, end, step } => {
                if let Value::Array(arr) = current {
                    // Logic for slice is in PathSegment
                    segment.apply_slice(arr, *start, *end, *step)
                } else {
                    vec![]
                }
            }

            PathSegment::MultiKey(keys) => {
                if let Value::Object(obj) = current {
                    vec![segment.apply_multi_key(obj, keys)]
                } else {
                    vec![]
                }
            }
        }
    }

    fn recursive_find(&self, current: &Value, key: &str, results: &mut Vec<Value>) {
        if let Value::Object(obj) = current {
            if key == "*" {
                for v in obj.values() {
                    results.push(v.clone());
                }
            } else if let Some(v) = obj.get(key) {
                results.push(v.clone());
            }
            for v in obj.values() {
                self.recursive_find(v, key, results);
            }
        } else if let Value::Array(arr) = current {
            if key == "*" {
                for v in arr {
                    results.push(v.clone());
                }
            }
            for v in arr {
                self.recursive_find(v, key, results);
            }
        }
    }
}
