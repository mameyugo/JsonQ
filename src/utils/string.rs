//! String utilities
//!
//! Helper functions for string manipulation and comparison.

/// Calculate the Levenshtein distance between two strings.
///
/// Used for providing "Did you mean?" suggestions in error messages.
pub fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let s1_len = s1.chars().count();
    let s2_len = s2.chars().count();

    if s1_len == 0 {
        return s2_len;
    }
    if s2_len == 0 {
        return s1_len;
    }

    let mut matrix = vec![vec![0; s2_len + 1]; s1_len + 1];

    for i in 0..=s1_len {
        matrix[i][0] = i;
    }
    for j in 0..=s2_len {
        matrix[0][j] = j;
    }

    for (i, c1) in s1.chars().enumerate() {
        for (j, c2) in s2.chars().enumerate() {
            let cost = if c1 == c2 { 0 } else { 1 };
            matrix[i + 1][j + 1] = std::cmp::min(
                std::cmp::min(matrix[i][j + 1] + 1, matrix[i + 1][j] + 1),
                matrix[i][j] + cost,
            );
        }
    }

    matrix[s1_len][s2_len]
}

/// Find the best match for a given input from a list of candidates.
/// Returns Some(candidate) if the distance is within a threshold.
pub fn suggest_similar(input: &str, candidates: &[&str]) -> Option<String> {
    let threshold = 3; // Max edits allowed
    let mut best_match = None;
    let mut min_distance = usize::MAX;

    for &candidate in candidates {
        let distance = levenshtein_distance(input, candidate);
        if distance < min_distance && distance <= threshold {
            min_distance = distance;
            best_match = Some(candidate.to_string());
        }
    }

    best_match
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levenshtein() {
        assert_eq!(levenshtein_distance("kitten", "sitting"), 3);
        assert_eq!(levenshtein_distance("gumbo", "gambol"), 2);
        assert_eq!(levenshtein_distance("", "test"), 4);
    }

    #[test]
    fn test_suggestion() {
        let candidates = vec!["name", "email", "age"];
        assert_eq!(
            suggest_similar("naem", &candidates),
            Some("name".to_string())
        );
        assert_eq!(
            suggest_similar("emial", &candidates),
            Some("email".to_string())
        );
        assert_eq!(suggest_similar("xyz", &candidates), None);
    }
}
