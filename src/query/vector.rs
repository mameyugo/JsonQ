//! Math functions for vector similarity and distance calculations.

/// Calculate the cosine similarity between two vectors.
///
/// Returns a value between -1.0 and 1.0. Higher is more similar.
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let mut dot_product = 0.0;
    let mut norm_a = 0.0;
    let mut norm_b = 0.0;
    for i in 0..a.len() {
        let av = a[i];
        let bv = b[i];
        dot_product += av * bv;
        norm_a += av * av;
        norm_b += bv * bv;
    }
    if norm_a <= 0.0 || norm_b <= 0.0 {
        return 0.0;
    }
    dot_product / (norm_a.sqrt() * norm_b.sqrt())
}

/// Calculate the Euclidean (L2) distance between two vectors.
///
/// Lower values are closer.
pub fn euclidean_distance(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return f32::INFINITY;
    }
    let mut sum = 0.0;
    for i in 0..a.len() {
        let diff = a[i] - b[i];
        sum += diff * diff;
    }
    sum.sqrt()
}

/// Calculate the dot product between two vectors.
///
/// Higher values are closer.
pub fn dot_product(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let mut sum = 0.0;
    for i in 0..a.len() {
        sum += a[i] * b[i];
    }
    sum
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 1e-6);

        let c = vec![0.0, 1.0, 0.0];
        assert!((cosine_similarity(&a, &c) - 0.0).abs() < 1e-6);

        let d = vec![-1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &d) + 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_euclidean_distance() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![4.0, 6.0, 3.0];
        // sqrt((1-4)^2 + (2-6)^2 + (3-3)^2) = sqrt(9 + 16 + 0) = 5
        assert!((euclidean_distance(&a, &b) - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_dot_product() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![4.0, 5.0, 6.0];
        // 1*4 + 2*5 + 3*6 = 4 + 10 + 18 = 32
        assert!((dot_product(&a, &b) - 32.0).abs() < 1e-6);
    }

    #[test]
    fn test_vector_search_cosine() {
        let collection = vec![
            json!({"name": "Item A", "vector": [1.0, 0.0, 0.0]}),
            json!({"name": "Item B", "vector": [0.0, 1.0, 0.0]}),
            json!({"name": "Item C", "vector": [0.707, 0.707, 0.0]}),
        ];

        let query = vec![1.0, 0.0, 0.0];
        let results = execute_vector_search(
            &collection,
            "vector",
            &query,
            2,
            "cosine",
            None,
        ).unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].item["name"], "Item A");
        assert!((results[0].score - 1.0).abs() < 1e-3);
        assert_eq!(results[1].item["name"], "Item C");
        assert!((results[1].score - 0.707).abs() < 1e-3);
    }

    #[test]
    fn test_vector_search_l2() {
        let collection = vec![
            json!({"name": "Item A", "vector": [1.0, 2.0, 3.0]}),
            json!({"name": "Item B", "vector": [4.0, 6.0, 3.0]}),
            json!({"name": "Item C", "vector": [1.0, 5.0, 3.0]}),
        ];

        let query = vec![1.0, 2.0, 3.0];
        let results = execute_vector_search(
            &collection,
            "vector",
            &query,
            3,
            "l2",
            None,
        ).unwrap();

        assert_eq!(results.len(), 3);
        assert_eq!(results[0].item["name"], "Item A");
        assert_eq!(results[0].score, 0.0);
        assert_eq!(results[1].item["name"], "Item C");
        assert_eq!(results[1].score, 3.0);
        assert_eq!(results[2].item["name"], "Item B");
        assert_eq!(results[2].score, 5.0);
    }

    #[test]
    fn test_vector_search_with_index() {
        use crate::index::IndexBuilder;

        let collection = vec![
            json!({"name": "Item A", "vector": [1.0, 0.0, 0.0]}),
            json!({"name": "Item B", "vector": [0.0, 1.0, 0.0]}),
        ];

        let builder = IndexBuilder::new();
        let vidx = builder.build_vector(&collection, "vector", Some(3), "cosine", 100).unwrap();

        let query = vec![0.0, 1.0, 0.0];
        let results = execute_vector_search(
            &collection,
            "vector",
            &query,
            1,
            "cosine",
            Some(&vidx),
        ).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].item["name"], "Item B");
        assert!((results[0].score - 1.0).abs() < 1e-3);
    }
}

use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Serialize, Clone)]
pub struct VectorSearchResult {
    pub score: f32,
    pub item: Value,
}

/// Executes similarity search over a collection of values using a query vector.
///
/// If a pre-built VectorIndex is provided, it uses the cached vectors.
/// Otherwise, it performs a flat scan and parses the vectors on the fly.
pub fn execute_vector_search(
    collection: &[Value],
    field: &str,
    query_vector: &[f32],
    limit: usize,
    metric: &str,
    vidx: Option<&crate::store::index_store::VectorIndex>,
) -> Result<Vec<VectorSearchResult>, String> {
    if query_vector.is_empty() {
        return Err("Query vector cannot be empty".to_string());
    }

    let mut results: Vec<(f32, &Value)> = Vec::new();

    if let Some(index) = vidx {
        for entry in &index.entries {
            if entry.index < collection.len() {
                let score = match metric {
                    "l2" | "euclidean" => euclidean_distance(&entry.vector, query_vector),
                    "dot" | "inner_product" => dot_product(&entry.vector, query_vector),
                    _ => cosine_similarity(&entry.vector, query_vector),
                };
                results.push((score, &collection[entry.index]));
            }
        }
    } else {
        for item in collection {
            if let Some(val) = crate::path::read_nested(item, field) {
                if let Some(arr) = val.as_array() {
                    let mut vec = Vec::with_capacity(arr.len());
                    for num in arr {
                        if let Some(f) = num.as_f64() {
                            vec.push(f as f32);
                        } else {
                            vec.clear();
                            break;
                        }
                    }
                    if !vec.is_empty() {
                        let score = match metric {
                            "l2" | "euclidean" => euclidean_distance(&vec, query_vector),
                            "dot" | "inner_product" => dot_product(&vec, query_vector),
                            _ => cosine_similarity(&vec, query_vector),
                        };
                        results.push((score, item));
                    }
                }
            }
        }
    }

    // Sort results based on metric
    match metric {
        "l2" | "euclidean" => {
            results.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        }
        _ => {
            results.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        }
    }

    let limited = results
        .into_iter()
        .take(limit)
        .map(|(score, item)| VectorSearchResult {
            score,
            item: item.clone(),
        })
        .collect();

    Ok(limited)
}
