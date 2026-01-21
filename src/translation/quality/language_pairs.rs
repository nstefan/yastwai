/*!
 * Language-pair specific thresholds for translation quality validation.
 *
 * Different language pairs have different expected length ratios due to
 * linguistic characteristics. For example:
 * - Japanese → English: typically expands 1.5-2x
 * - English → Chinese: typically contracts 0.5-0.8x
 * - English → German: typically expands 1.1-1.3x
 *
 * This module provides calibrated thresholds for common language pairs.
 */

use std::collections::HashMap;
use std::sync::LazyLock;

/// Language-pair specific thresholds for validation
#[derive(Debug, Clone)]
pub struct LanguagePairThresholds {
    /// Minimum acceptable length ratio (translated / source)
    pub min_length_ratio: f32,
    /// Maximum acceptable length ratio (translated / source)
    pub max_length_ratio: f32,
    /// Expected average ratio for this pair
    pub expected_ratio: f32,
}

impl Default for LanguagePairThresholds {
    fn default() -> Self {
        Self {
            min_length_ratio: 0.3,
            max_length_ratio: 3.0,
            expected_ratio: 1.0,
        }
    }
}

impl LanguagePairThresholds {
    /// Create thresholds with custom values
    pub fn new(min: f32, max: f32, expected: f32) -> Self {
        Self {
            min_length_ratio: min,
            max_length_ratio: max,
            expected_ratio: expected,
        }
    }

    /// Get thresholds for a specific language pair
    ///
    /// Returns calibrated thresholds based on known linguistic characteristics.
    /// Falls back to defaults for unknown pairs.
    pub fn get_defaults(source_lang: &str, target_lang: &str) -> Self {
        let key = format!("{}_{}", source_lang.to_lowercase(), target_lang.to_lowercase());

        LANGUAGE_PAIR_THRESHOLDS
            .get(key.as_str())
            .cloned()
            .unwrap_or_default()
    }

    /// Check if a length ratio is within acceptable bounds
    pub fn is_ratio_acceptable(&self, ratio: f32) -> bool {
        ratio >= self.min_length_ratio && ratio <= self.max_length_ratio
    }

    /// Calculate how far a ratio is from expected (for scoring)
    ///
    /// Returns a value from 0.0 (at expected) to 1.0 (at boundary or beyond)
    pub fn deviation_from_expected(&self, ratio: f32) -> f32 {
        if ratio < self.expected_ratio {
            // Below expected - calculate distance to min
            let range = self.expected_ratio - self.min_length_ratio;
            if range > 0.0 {
                ((self.expected_ratio - ratio) / range).min(1.0)
            } else {
                0.0
            }
        } else {
            // Above expected - calculate distance to max
            let range = self.max_length_ratio - self.expected_ratio;
            if range > 0.0 {
                ((ratio - self.expected_ratio) / range).min(1.0)
            } else {
                0.0
            }
        }
    }
}

/// Pre-configured thresholds for common language pairs
static LANGUAGE_PAIR_THRESHOLDS: LazyLock<HashMap<&'static str, LanguagePairThresholds>> =
    LazyLock::new(|| {
        let mut m = HashMap::new();

        // English source
        m.insert("en_de", LanguagePairThresholds::new(0.9, 1.4, 1.15));  // German expands ~15%
        m.insert("en_fr", LanguagePairThresholds::new(0.9, 1.3, 1.1));   // French expands ~10%
        m.insert("en_es", LanguagePairThresholds::new(0.9, 1.3, 1.1));   // Spanish expands ~10%
        m.insert("en_it", LanguagePairThresholds::new(0.9, 1.3, 1.1));   // Italian expands ~10%
        m.insert("en_pt", LanguagePairThresholds::new(0.9, 1.3, 1.1));   // Portuguese expands ~10%
        m.insert("en_nl", LanguagePairThresholds::new(0.9, 1.3, 1.05));  // Dutch similar
        m.insert("en_ru", LanguagePairThresholds::new(0.8, 1.3, 1.0));   // Russian varies
        m.insert("en_ja", LanguagePairThresholds::new(0.4, 0.9, 0.6));   // Japanese contracts
        m.insert("en_zh", LanguagePairThresholds::new(0.4, 0.8, 0.55));  // Chinese contracts more
        m.insert("en_ko", LanguagePairThresholds::new(0.5, 1.0, 0.7));   // Korean contracts
        m.insert("en_ar", LanguagePairThresholds::new(0.8, 1.4, 1.1));   // Arabic varies
        m.insert("en_hi", LanguagePairThresholds::new(0.9, 1.5, 1.2));   // Hindi expands
        m.insert("en_pl", LanguagePairThresholds::new(0.9, 1.4, 1.15));  // Polish expands
        m.insert("en_tr", LanguagePairThresholds::new(0.9, 1.4, 1.1));   // Turkish varies
        m.insert("en_vi", LanguagePairThresholds::new(0.9, 1.5, 1.2));   // Vietnamese expands

        // Japanese source
        m.insert("ja_en", LanguagePairThresholds::new(1.3, 2.5, 1.8));   // English expands from Japanese
        m.insert("ja_zh", LanguagePairThresholds::new(0.7, 1.3, 0.95));  // Chinese similar to Japanese
        m.insert("ja_ko", LanguagePairThresholds::new(0.8, 1.4, 1.1));   // Korean similar

        // Chinese source
        m.insert("zh_en", LanguagePairThresholds::new(1.5, 3.0, 2.0));   // English expands significantly
        m.insert("zh_ja", LanguagePairThresholds::new(0.9, 1.4, 1.1));   // Japanese similar

        // German source
        m.insert("de_en", LanguagePairThresholds::new(0.7, 1.1, 0.85));  // English contracts from German
        m.insert("de_fr", LanguagePairThresholds::new(0.85, 1.2, 1.0));  // French similar

        // French source
        m.insert("fr_en", LanguagePairThresholds::new(0.75, 1.1, 0.9)); // English contracts slightly

        // Spanish source
        m.insert("es_en", LanguagePairThresholds::new(0.75, 1.1, 0.9)); // English contracts slightly

        // Korean source
        m.insert("ko_en", LanguagePairThresholds::new(1.2, 2.0, 1.5));   // English expands from Korean

        m
    });

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_getDefaults_withKnownPair_shouldReturnCalibratedThresholds() {
        let thresholds = LanguagePairThresholds::get_defaults("en", "ja");
        assert!(thresholds.max_length_ratio < 1.0); // Japanese contracts
        assert!(thresholds.min_length_ratio < 0.5);
    }

    #[test]
    fn test_getDefaults_withUnknownPair_shouldReturnDefaults() {
        let thresholds = LanguagePairThresholds::get_defaults("xx", "yy");
        assert_eq!(thresholds.min_length_ratio, 0.3);
        assert_eq!(thresholds.max_length_ratio, 3.0);
    }

    #[test]
    fn test_getDefaults_isCaseInsensitive() {
        let lower = LanguagePairThresholds::get_defaults("en", "de");
        let upper = LanguagePairThresholds::get_defaults("EN", "DE");
        assert_eq!(lower.expected_ratio, upper.expected_ratio);
    }

    #[test]
    fn test_isRatioAcceptable_withinBounds_shouldReturnTrue() {
        let thresholds = LanguagePairThresholds::new(0.5, 1.5, 1.0);
        assert!(thresholds.is_ratio_acceptable(1.0));
        assert!(thresholds.is_ratio_acceptable(0.5));
        assert!(thresholds.is_ratio_acceptable(1.5));
    }

    #[test]
    fn test_isRatioAcceptable_outsideBounds_shouldReturnFalse() {
        let thresholds = LanguagePairThresholds::new(0.5, 1.5, 1.0);
        assert!(!thresholds.is_ratio_acceptable(0.4));
        assert!(!thresholds.is_ratio_acceptable(1.6));
    }

    #[test]
    fn test_deviationFromExpected_atExpected_shouldBeZero() {
        let thresholds = LanguagePairThresholds::new(0.5, 1.5, 1.0);
        let deviation = thresholds.deviation_from_expected(1.0);
        assert!(deviation < 0.01);
    }

    #[test]
    fn test_deviationFromExpected_atBoundary_shouldBeOne() {
        let thresholds = LanguagePairThresholds::new(0.5, 1.5, 1.0);
        let at_min = thresholds.deviation_from_expected(0.5);
        let at_max = thresholds.deviation_from_expected(1.5);
        assert!((at_min - 1.0).abs() < 0.01);
        assert!((at_max - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_enToJapanese_shouldHaveContractingThresholds() {
        let thresholds = LanguagePairThresholds::get_defaults("en", "ja");
        assert!(thresholds.expected_ratio < 1.0);
        assert!(thresholds.max_length_ratio < 1.0);
    }

    #[test]
    fn test_japaneseToEn_shouldHaveExpandingThresholds() {
        let thresholds = LanguagePairThresholds::get_defaults("ja", "en");
        assert!(thresholds.expected_ratio > 1.0);
        assert!(thresholds.min_length_ratio > 1.0);
    }
}
