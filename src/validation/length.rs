/*!
 * Length validation for translated subtitles.
 *
 * This module validates that translation lengths are reasonable:
 * - Length ratio between source and translated text
 * - Absolute length limits
 * - Empty translation detection
 */

use log::debug;

/// Default minimum length ratio (translation / source)
const DEFAULT_MIN_LENGTH_RATIO: f64 = 0.3;

/// Default maximum length ratio (translation / source)
const DEFAULT_MAX_LENGTH_RATIO: f64 = 3.0;

/// Result of length validation for a single entry
#[derive(Debug, Clone)]
pub struct LengthEntryResult {
    /// Sequence number
    pub seq_num: usize,
    /// Whether validation passed
    pub passed: bool,
    /// Issues found
    pub issues: Vec<LengthIssue>,
    /// Calculated length ratio
    pub length_ratio: f64,
}

impl LengthEntryResult {
    /// Create a passing result
    pub fn passed(seq_num: usize, length_ratio: f64) -> Self {
        Self {
            seq_num,
            passed: true,
            issues: vec![],
            length_ratio,
        }
    }

    /// Create a failing result
    pub fn failed(seq_num: usize, length_ratio: f64, issues: Vec<LengthIssue>) -> Self {
        Self {
            seq_num,
            passed: false,
            issues,
            length_ratio,
        }
    }
}

/// Types of length issues
#[derive(Debug, Clone, PartialEq)]
pub enum LengthIssue {
    /// Translation is empty
    EmptyTranslation,
    /// Translation is too short relative to source
    TranslationTooShort {
        ratio: f64,
        min_ratio: f64,
        source_len: usize,
        translated_len: usize,
    },
    /// Translation is too long relative to source
    TranslationTooLong {
        ratio: f64,
        max_ratio: f64,
        source_len: usize,
        translated_len: usize,
    },
    /// Source is empty but translation is not
    UnexpectedTranslation {
        translated_len: usize,
    },
}

impl std::fmt::Display for LengthIssue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LengthIssue::EmptyTranslation => {
                write!(f, "Translation is empty")
            }
            LengthIssue::TranslationTooShort {
                ratio,
                min_ratio,
                source_len,
                translated_len,
            } => {
                write!(
                    f,
                    "Translation too short: ratio {:.2} < {:.2} ({} -> {} chars)",
                    ratio, min_ratio, source_len, translated_len
                )
            }
            LengthIssue::TranslationTooLong {
                ratio,
                max_ratio,
                source_len,
                translated_len,
            } => {
                write!(
                    f,
                    "Translation too long: ratio {:.2} > {:.2} ({} -> {} chars)",
                    ratio, max_ratio, source_len, translated_len
                )
            }
            LengthIssue::UnexpectedTranslation { translated_len } => {
                write!(
                    f,
                    "Unexpected translation of {} chars for empty source",
                    translated_len
                )
            }
        }
    }
}

/// Result of validating lengths for a collection
#[derive(Debug, Clone)]
pub struct LengthValidationResult {
    /// Overall pass/fail status
    pub passed: bool,
    /// Results for each entry
    pub entry_results: Vec<LengthEntryResult>,
    /// Total number of issues
    pub total_issues: usize,
    /// Average length ratio across all entries
    pub average_ratio: f64,
}

impl LengthValidationResult {
    /// Get all failed entries
    pub fn failed_entries(&self) -> Vec<&LengthEntryResult> {
        self.entry_results.iter().filter(|r| !r.passed).collect()
    }
}

/// Configuration for length validation
#[derive(Debug, Clone)]
pub struct LengthValidatorConfig {
    /// Minimum acceptable length ratio (translated / source)
    pub min_ratio: f64,
    /// Maximum acceptable length ratio (translated / source)
    pub max_ratio: f64,
    /// Whether to fail on empty translations
    pub fail_on_empty: bool,
    /// Minimum source length to apply ratio checks (shorter texts get lenient treatment)
    pub min_source_length_for_ratio: usize,
}

impl Default for LengthValidatorConfig {
    fn default() -> Self {
        Self {
            min_ratio: DEFAULT_MIN_LENGTH_RATIO,
            max_ratio: DEFAULT_MAX_LENGTH_RATIO,
            fail_on_empty: true,
            min_source_length_for_ratio: 10, // Don't ratio-check very short texts
        }
    }
}

/// Length validator for subtitle translations
pub struct LengthValidator {
    config: LengthValidatorConfig,
}

impl LengthValidator {
    /// Create a new validator with default configuration
    pub fn new() -> Self {
        Self {
            config: LengthValidatorConfig::default(),
        }
    }

    /// Create a new validator with custom configuration
    pub fn with_config(config: LengthValidatorConfig) -> Self {
        Self { config }
    }

    /// Calculate length ratio between translated and source text
    pub fn calculate_ratio(source: &str, translated: &str) -> f64 {
        let source_len = source.chars().count();
        let translated_len = translated.chars().count();

        if source_len == 0 {
            if translated_len == 0 {
                1.0 // Both empty = ratio of 1
            } else {
                f64::INFINITY
            }
        } else {
            translated_len as f64 / source_len as f64
        }
    }

    /// Validate a single translation pair
    pub fn validate_entry(
        &self,
        seq_num: usize,
        source_text: &str,
        translated_text: &str,
    ) -> LengthEntryResult {
        let source_trimmed = source_text.trim();
        let translated_trimmed = translated_text.trim();

        let source_len = source_trimmed.chars().count();
        let translated_len = translated_trimmed.chars().count();

        let ratio = Self::calculate_ratio(source_trimmed, translated_trimmed);
        let mut issues = Vec::new();

        // Check for empty translation of non-empty source
        if source_len > 0 && translated_len == 0 {
            if self.config.fail_on_empty {
                issues.push(LengthIssue::EmptyTranslation);
            }
            return LengthEntryResult::failed(seq_num, ratio, issues);
        }

        // Check for translation of empty source
        if source_len == 0 && translated_len > 0 {
            issues.push(LengthIssue::UnexpectedTranslation { translated_len });
            return LengthEntryResult::failed(seq_num, ratio, issues);
        }

        // Both empty is valid
        if source_len == 0 && translated_len == 0 {
            return LengthEntryResult::passed(seq_num, 1.0);
        }

        // Only check ratio if source is long enough
        if source_len >= self.config.min_source_length_for_ratio {
            if ratio < self.config.min_ratio {
                issues.push(LengthIssue::TranslationTooShort {
                    ratio,
                    min_ratio: self.config.min_ratio,
                    source_len,
                    translated_len,
                });
            }

            if ratio > self.config.max_ratio {
                issues.push(LengthIssue::TranslationTooLong {
                    ratio,
                    max_ratio: self.config.max_ratio,
                    source_len,
                    translated_len,
                });
            }
        }

        if issues.is_empty() {
            LengthEntryResult::passed(seq_num, ratio)
        } else {
            LengthEntryResult::failed(seq_num, ratio, issues)
        }
    }

    /// Validate a collection of translation pairs
    pub fn validate_collection(
        &self,
        pairs: &[(usize, &str, &str)], // (seq_num, source, translated)
    ) -> LengthValidationResult {
        if pairs.is_empty() {
            return LengthValidationResult {
                passed: true,
                entry_results: vec![],
                total_issues: 0,
                average_ratio: 1.0,
            };
        }

        let entry_results: Vec<LengthEntryResult> = pairs
            .iter()
            .map(|(seq_num, source, translated)| {
                self.validate_entry(*seq_num, source, translated)
            })
            .collect();

        let total_issues: usize = entry_results.iter().map(|r| r.issues.len()).sum();
        let passed = entry_results.iter().all(|r| r.passed);

        // Calculate average ratio (excluding infinite values)
        let valid_ratios: Vec<f64> = entry_results
            .iter()
            .filter(|r| r.length_ratio.is_finite())
            .map(|r| r.length_ratio)
            .collect();

        let average_ratio = if valid_ratios.is_empty() {
            1.0
        } else {
            valid_ratios.iter().sum::<f64>() / valid_ratios.len() as f64
        };

        debug!(
            "Length validation: {} entries, {} issues, avg ratio: {:.2}",
            pairs.len(),
            total_issues,
            average_ratio
        );

        LengthValidationResult {
            passed,
            entry_results,
            total_issues,
            average_ratio,
        }
    }
}

impl Default for LengthValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculateRatio_shouldCalculateCorrectly() {
        assert!((LengthValidator::calculate_ratio("hello", "bonjour") - 1.4).abs() < 0.01);
        assert!((LengthValidator::calculate_ratio("test", "test") - 1.0).abs() < 0.01);
        assert!((LengthValidator::calculate_ratio("abcdefghij", "ab") - 0.2).abs() < 0.01);
    }

    #[test]
    fn test_validateEntry_withReasonableRatio_shouldPass() {
        let validator = LengthValidator::new();

        // "Hello World" -> "Bonjour le Monde" is a reasonable ratio
        let result = validator.validate_entry(1, "Hello World", "Bonjour le Monde");

        assert!(result.passed);
        assert!(result.length_ratio > 0.5 && result.length_ratio < 2.0);
    }

    #[test]
    fn test_validateEntry_withEmptyTranslation_shouldFail() {
        let validator = LengthValidator::new();

        let result = validator.validate_entry(1, "Hello World", "");

        assert!(!result.passed);
        assert!(matches!(result.issues[0], LengthIssue::EmptyTranslation));
    }

    #[test]
    fn test_validateEntry_withTooShortTranslation_shouldFail() {
        let validator = LengthValidator::new();

        // A translation that's way too short
        let result = validator.validate_entry(
            1,
            "This is a very long source text that should have a substantial translation",
            "Hi",
        );

        assert!(!result.passed);
        assert!(matches!(
            result.issues[0],
            LengthIssue::TranslationTooShort { .. }
        ));
    }

    #[test]
    fn test_validateEntry_withTooLongTranslation_shouldFail() {
        let validator = LengthValidator::new();

        // A translation that's way too long
        let result = validator.validate_entry(
            1,
            "Short text here",
            "This is an extremely long translation that is way too verbose and should fail the length check because it is much longer than the original",
        );

        assert!(!result.passed);
        assert!(matches!(
            result.issues[0],
            LengthIssue::TranslationTooLong { .. }
        ));
    }

    #[test]
    fn test_validateEntry_withShortSource_shouldBeLenient() {
        let validator = LengthValidator::new();

        // Short source texts get lenient treatment
        let result = validator.validate_entry(1, "Hi", "Salut mon ami");

        // Should pass despite high ratio because source is short
        assert!(result.passed);
    }

    #[test]
    fn test_validateEntry_withBothEmpty_shouldPass() {
        let validator = LengthValidator::new();

        let result = validator.validate_entry(1, "", "");

        assert!(result.passed);
        assert_eq!(result.length_ratio, 1.0);
    }

    #[test]
    fn test_validateEntry_withUnexpectedTranslation_shouldFail() {
        let validator = LengthValidator::new();

        let result = validator.validate_entry(1, "", "Unexpected content");

        assert!(!result.passed);
        assert!(matches!(
            result.issues[0],
            LengthIssue::UnexpectedTranslation { .. }
        ));
    }

    #[test]
    fn test_validateCollection_shouldCalculateAverageRatio() {
        let validator = LengthValidator::new();

        let pairs = vec![
            (1, "Hello", "Bonjour"),    // ~1.4 ratio
            (2, "World", "Monde"),      // 1.0 ratio
            (3, "Test text", "Essai"),  // 0.5 ratio
        ];

        let result = validator.validate_collection(&pairs);

        assert!(result.average_ratio > 0.5);
        assert!(result.average_ratio < 2.0);
    }

    #[test]
    fn test_customConfig_shouldBeRespected() {
        let config = LengthValidatorConfig {
            min_ratio: 0.1, // Very lenient
            max_ratio: 10.0,
            fail_on_empty: false,
            min_source_length_for_ratio: 5,
        };
        let validator = LengthValidator::with_config(config);

        // With lenient config, this should pass
        let result = validator.validate_entry(
            1,
            "Very long source text here",
            "Short",
        );

        assert!(result.passed);
    }
}
