/*!
 * Fuzzy matching for glossary terms.
 *
 * Provides Levenshtein distance-based fuzzy matching to find glossary terms
 * even when there are minor typos or variations in the source text.
 */

/// Fuzzy matcher using Levenshtein distance
#[derive(Debug, Clone)]
pub struct FuzzyMatcher {
    /// Default threshold for fuzzy matching (0.0-1.0, higher = stricter)
    default_threshold: f32,
}

impl Default for FuzzyMatcher {
    fn default() -> Self {
        Self {
            default_threshold: 0.8,
        }
    }
}

impl FuzzyMatcher {
    /// Create a new fuzzy matcher with custom threshold
    pub fn new(threshold: f32) -> Self {
        Self {
            default_threshold: threshold.clamp(0.0, 1.0),
        }
    }

    /// Check if text fuzzy-matches a term within the threshold
    ///
    /// Returns true if the similarity is at or above the threshold.
    pub fn matches(&self, text: &str, term: &str, threshold: Option<f32>) -> bool {
        let thresh = threshold.unwrap_or(self.default_threshold);
        self.similarity(text, term) >= thresh
    }

    /// Calculate similarity between two strings (0.0-1.0)
    ///
    /// Uses normalized Levenshtein distance.
    pub fn similarity(&self, a: &str, b: &str) -> f32 {
        if a.is_empty() && b.is_empty() {
            return 1.0;
        }
        if a.is_empty() || b.is_empty() {
            return 0.0;
        }

        let a_lower = a.to_lowercase();
        let b_lower = b.to_lowercase();

        let distance = levenshtein_distance(&a_lower, &b_lower);
        let max_len = a_lower.len().max(b_lower.len());

        1.0 - (distance as f32 / max_len as f32)
    }

    /// Find the best matching term from a list
    ///
    /// Returns the term with highest similarity above threshold, if any.
    pub fn find_best_match<'a>(&self, text: &str, terms: &[&'a str], threshold: Option<f32>) -> Option<&'a str> {
        let thresh = threshold.unwrap_or(self.default_threshold);
        let mut best_match: Option<(&str, f32)> = None;

        for term in terms {
            let sim = self.similarity(text, term);
            if sim >= thresh {
                if let Some((_, best_sim)) = best_match {
                    if sim > best_sim {
                        best_match = Some((term, sim));
                    }
                } else {
                    best_match = Some((term, sim));
                }
            }
        }

        best_match.map(|(term, _)| term)
    }

    /// Find all terms that match within threshold
    pub fn find_all_matches<'a>(&self, text: &str, terms: &[&'a str], threshold: Option<f32>) -> Vec<(&'a str, f32)> {
        let thresh = threshold.unwrap_or(self.default_threshold);

        terms
            .iter()
            .filter_map(|term| {
                let sim = self.similarity(text, term);
                if sim >= thresh {
                    Some((*term, sim))
                } else {
                    None
                }
            })
            .collect()
    }
}

/// Calculate Levenshtein distance between two strings
fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let a_len = a_chars.len();
    let b_len = b_chars.len();

    if a_len == 0 {
        return b_len;
    }
    if b_len == 0 {
        return a_len;
    }

    // Use two-row optimization for space efficiency
    let mut prev_row: Vec<usize> = (0..=b_len).collect();
    let mut curr_row: Vec<usize> = vec![0; b_len + 1];

    for i in 1..=a_len {
        curr_row[0] = i;

        for j in 1..=b_len {
            let cost = if a_chars[i - 1] == b_chars[j - 1] { 0 } else { 1 };

            curr_row[j] = (prev_row[j] + 1)                  // deletion
                .min(curr_row[j - 1] + 1)                    // insertion
                .min(prev_row[j - 1] + cost);                // substitution
        }

        std::mem::swap(&mut prev_row, &mut curr_row);
    }

    prev_row[b_len]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levenshteinDistance_identical_shouldBeZero() {
        assert_eq!(levenshtein_distance("hello", "hello"), 0);
    }

    #[test]
    fn test_levenshteinDistance_oneDifferent_shouldBeOne() {
        assert_eq!(levenshtein_distance("hello", "hallo"), 1);
        assert_eq!(levenshtein_distance("cat", "hat"), 1);
    }

    #[test]
    fn test_levenshteinDistance_empty_shouldReturnLength() {
        assert_eq!(levenshtein_distance("", "hello"), 5);
        assert_eq!(levenshtein_distance("hello", ""), 5);
    }

    #[test]
    fn test_similarity_identical_shouldBeOne() {
        let matcher = FuzzyMatcher::default();
        assert!((matcher.similarity("hello", "hello") - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_similarity_completelyDifferent_shouldBeLow() {
        let matcher = FuzzyMatcher::default();
        assert!(matcher.similarity("abc", "xyz") < 0.5);
    }

    #[test]
    fn test_similarity_isCaseInsensitive() {
        let matcher = FuzzyMatcher::default();
        assert!((matcher.similarity("Hello", "hello") - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_matches_withinThreshold_shouldReturnTrue() {
        let matcher = FuzzyMatcher::new(0.8);
        assert!(matcher.matches("hello", "helo", None)); // 80% similar
    }

    #[test]
    fn test_matches_belowThreshold_shouldReturnFalse() {
        let matcher = FuzzyMatcher::new(0.9);
        assert!(!matcher.matches("hello", "hxxxx", None));
    }

    #[test]
    fn test_findBestMatch_shouldReturnBestAboveThreshold() {
        let matcher = FuzzyMatcher::new(0.6);
        let terms = vec!["cat", "car", "bat", "xyz"];

        let result = matcher.find_best_match("cat", &terms, None);
        assert_eq!(result, Some("cat"));

        // "cot" is 0.67 similar to "cat" (1 edit / 3 chars)
        let result = matcher.find_best_match("cot", &terms, None);
        assert_eq!(result, Some("cat"));
    }

    #[test]
    fn test_findBestMatch_noneAboveThreshold_shouldReturnNone() {
        let matcher = FuzzyMatcher::new(0.95);
        let terms = vec!["apple", "banana", "cherry"];

        let result = matcher.find_best_match("xyz", &terms, None);
        assert_eq!(result, None);
    }

    #[test]
    fn test_findAllMatches_shouldReturnAllAboveThreshold() {
        let matcher = FuzzyMatcher::new(0.6);
        let terms = vec!["cat", "car", "bat", "xyz"];

        let matches = matcher.find_all_matches("cat", &terms, None);
        assert!(matches.iter().any(|(t, _)| *t == "cat"));
        assert!(matches.iter().any(|(t, _)| *t == "car")); // car is 66% similar
    }
}
