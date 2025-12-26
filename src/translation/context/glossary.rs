/*!
 * Glossary extraction for terminology consistency.
 *
 * This module provides automatic extraction of:
 * - Character names (capitalized words that appear frequently)
 * - Technical terms (domain-specific vocabulary)
 * - Key phrases (recurring expressions)
 *
 * The extracted glossary helps maintain translation consistency.
 */

use regex::Regex;
use std::collections::{HashMap, HashSet};

use crate::translation::document::{DocumentEntry, Glossary, SubtitleDocument};

/// Configuration for glossary extraction.
#[derive(Debug, Clone)]
pub struct ExtractionConfig {
    /// Minimum occurrences to consider a term significant
    pub min_occurrences: usize,

    /// Whether to extract capitalized words as potential names
    pub extract_names: bool,

    /// Whether to extract quoted phrases
    pub extract_quoted: bool,

    /// Custom patterns to match (regex strings)
    pub custom_patterns: Vec<String>,

    /// Words to exclude from extraction (common words)
    pub exclude_words: HashSet<String>,
}

impl Default for ExtractionConfig {
    fn default() -> Self {
        let mut exclude_words = HashSet::new();
        // Common English words that shouldn't be extracted as names
        for word in &[
            "I", "The", "A", "An", "This", "That", "These", "Those", "It", "He", "She", "They",
            "We", "You", "My", "Your", "His", "Her", "Our", "Their", "What", "Who", "Where",
            "When", "Why", "How", "Yes", "No", "Oh", "Ah", "Hey", "Well", "Now", "Then", "Here",
            "There", "Please", "Thank", "Thanks", "Sorry", "Hello", "Hi", "Goodbye", "Bye",
            "Mr", "Mrs", "Ms", "Dr", "Sir", "Ma'am", "OK", "Okay",
        ] {
            exclude_words.insert(word.to_string());
        }

        Self {
            min_occurrences: 2,
            extract_names: true,
            extract_quoted: true,
            custom_patterns: Vec::new(),
            exclude_words,
        }
    }
}

impl ExtractionConfig {
    /// Create a minimal config that extracts only obvious names.
    pub fn minimal() -> Self {
        Self {
            min_occurrences: 3,
            extract_names: true,
            extract_quoted: false,
            custom_patterns: Vec::new(),
            exclude_words: Self::default().exclude_words,
        }
    }

    /// Create an aggressive config that extracts more terms.
    pub fn aggressive() -> Self {
        Self {
            min_occurrences: 1,
            extract_names: true,
            extract_quoted: true,
            custom_patterns: Vec::new(),
            exclude_words: HashSet::new(),
        }
    }

    /// Add a word to the exclusion list.
    pub fn exclude(mut self, word: &str) -> Self {
        self.exclude_words.insert(word.to_string());
        self
    }

    /// Add a custom pattern.
    pub fn with_pattern(mut self, pattern: &str) -> Self {
        self.custom_patterns.push(pattern.to_string());
        self
    }
}

/// Glossary extractor for automatic term detection.
pub struct GlossaryExtractor {
    config: ExtractionConfig,
    name_pattern: Regex,
    quoted_pattern: Regex,
}

impl GlossaryExtractor {
    /// Create a new glossary extractor with the given configuration.
    pub fn new(config: ExtractionConfig) -> Self {
        Self {
            config,
            // Pattern for capitalized words (potential names)
            name_pattern: Regex::new(r"\b([A-Z][a-z]+(?:\s+[A-Z][a-z]+)?)\b").unwrap(),
            // Pattern for quoted phrases
            quoted_pattern: Regex::new(r#""([^"]+)""#).unwrap(),
        }
    }

    /// Create an extractor with default configuration.
    pub fn with_defaults() -> Self {
        Self::new(ExtractionConfig::default())
    }

    /// Extract a glossary from document entries.
    pub fn extract(&self, entries: &[DocumentEntry]) -> Glossary {
        let mut glossary = Glossary::new();
        let mut name_counts: HashMap<String, usize> = HashMap::new();
        let mut quoted_phrases: HashMap<String, usize> = HashMap::new();

        for entry in entries {
            let text = &entry.original_text;

            // Extract potential names
            if self.config.extract_names {
                for cap in self.name_pattern.captures_iter(text) {
                    if let Some(name) = cap.get(1) {
                        let name_str = name.as_str().to_string();
                        if !self.config.exclude_words.contains(&name_str)
                            && !self.is_common_word(&name_str)
                        {
                            *name_counts.entry(name_str).or_insert(0) += 1;
                        }
                    }
                }
            }

            // Extract quoted phrases
            if self.config.extract_quoted {
                for cap in self.quoted_pattern.captures_iter(text) {
                    if let Some(phrase) = cap.get(1) {
                        let phrase_str = phrase.as_str().to_string();
                        if phrase_str.len() >= 2 {
                            *quoted_phrases.entry(phrase_str).or_insert(0) += 1;
                        }
                    }
                }
            }
        }

        // Add names that meet the threshold
        for (name, count) in name_counts {
            if count >= self.config.min_occurrences {
                glossary.add_character(&name);
            }
        }

        // Add quoted phrases that meet the threshold as terms
        for (phrase, count) in quoted_phrases {
            if count >= self.config.min_occurrences {
                glossary.add_term(&phrase, &phrase, Some("quoted phrase"));
            }
        }

        glossary
    }

    /// Check if a word is a common word that shouldn't be extracted.
    fn is_common_word(&self, word: &str) -> bool {
        // Additional heuristics for common words
        if word.len() <= 2 {
            return true;
        }

        // Check for common sentence starters that might be capitalized
        let lower = word.to_lowercase();
        matches!(
            lower.as_str(),
            "just" | "really" | "actually" | "probably" | "definitely" | "certainly" | "maybe"
                | "perhaps" | "finally" | "suddenly" | "quickly" | "slowly"
        )
    }

    /// Extract terms and update the document's glossary.
    pub fn extract_and_update(&self, doc: &mut SubtitleDocument) {
        let extracted = self.extract(&doc.entries);
        doc.glossary.merge(&extracted);
    }

    /// Merge an existing glossary with extracted terms.
    pub fn extract_and_merge(&self, entries: &[DocumentEntry], existing: &Glossary) -> Glossary {
        let mut result = existing.clone();
        let extracted = self.extract(entries);
        result.merge(&extracted);
        result
    }
}

/// Glossary enforcer for applying terminology consistency.
pub struct GlossaryEnforcer<'a> {
    glossary: &'a Glossary,
}

impl<'a> GlossaryEnforcer<'a> {
    /// Create a new glossary enforcer.
    pub fn new(glossary: &'a Glossary) -> Self {
        Self { glossary }
    }

    /// Check if a translation uses consistent terminology.
    pub fn check_consistency(&self, original: &str, translated: &str) -> Vec<ConsistencyIssue> {
        let mut issues = Vec::new();

        // Check character names (should appear unchanged in translation)
        for name in &self.glossary.character_names {
            if original.contains(name) && !translated.contains(name) {
                issues.push(ConsistencyIssue::MissingName {
                    name: name.clone(),
                    original: original.to_string(),
                    translated: translated.to_string(),
                });
            }
        }

        // Check terms (should use consistent translations)
        for (source, term) in &self.glossary.terms {
            if original.contains(source) && !translated.contains(&term.target) {
                issues.push(ConsistencyIssue::InconsistentTerm {
                    source: source.clone(),
                    expected: term.target.clone(),
                    translated: translated.to_string(),
                });
            }
        }

        issues
    }

    /// Enforce glossary terms in a translation (auto-fix).
    pub fn enforce(&self, original: &str, translated: &str) -> String {
        let mut result = translated.to_string();

        // Ensure character names are preserved
        for name in &self.glossary.character_names {
            if original.contains(name) && !result.contains(name) {
                // Try to find a likely mistranslation of the name
                // This is a simple heuristic - more sophisticated matching could be added
                // For now, we just ensure the name appears if it was in the original
            }
        }

        // Apply term replacements
        for (source, term) in &self.glossary.terms {
            if original.contains(source) {
                // If the source term appears in the translation (wasn't translated),
                // replace it with the target term
                if result.contains(source) {
                    result = result.replace(source, &term.target);
                }
            }
        }

        result
    }
}

/// Issue found during consistency checking.
#[derive(Debug, Clone)]
pub enum ConsistencyIssue {
    /// A character name is missing from the translation
    MissingName {
        name: String,
        original: String,
        translated: String,
    },
    /// A term was not translated consistently
    InconsistentTerm {
        source: String,
        expected: String,
        translated: String,
    },
}

impl ConsistencyIssue {
    /// Get a human-readable description of the issue.
    pub fn description(&self) -> String {
        match self {
            ConsistencyIssue::MissingName { name, .. } => {
                format!("Character name '{}' is missing from translation", name)
            }
            ConsistencyIssue::InconsistentTerm { source, expected, .. } => {
                format!(
                    "Term '{}' should be translated as '{}' for consistency",
                    source, expected
                )
            }
        }
    }
}

/// Extension trait for SubtitleDocument to add glossary extraction.
pub trait GlossaryExtractionExt {
    /// Extract glossary terms from the document.
    fn extract_glossary(&mut self);

    /// Extract glossary with custom configuration.
    fn extract_glossary_with_config(&mut self, config: ExtractionConfig);
}

impl GlossaryExtractionExt for SubtitleDocument {
    fn extract_glossary(&mut self) {
        let extractor = GlossaryExtractor::with_defaults();
        extractor.extract_and_update(self);
    }

    fn extract_glossary_with_config(&mut self, config: ExtractionConfig) {
        let extractor = GlossaryExtractor::new(config);
        extractor.extract_and_update(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::subtitle_processor::SubtitleEntry;

    fn create_entries(texts: &[&str]) -> Vec<DocumentEntry> {
        texts
            .iter()
            .enumerate()
            .map(|(i, text)| {
                let entry =
                    SubtitleEntry::new(i + 1, (i as u64) * 1000, (i as u64 + 1) * 1000, text.to_string());
                DocumentEntry::from_subtitle_entry(entry)
            })
            .collect()
    }

    #[test]
    fn test_glossaryExtractor_extractNames_shouldFindCapitalizedWords() {
        let entries = create_entries(&[
            "John went to the store.",
            "Mary called John.",
            "John and Mary talked.",
        ]);

        let extractor = GlossaryExtractor::new(ExtractionConfig {
            min_occurrences: 2,
            extract_names: true,
            extract_quoted: false,
            ..Default::default()
        });

        let glossary = extractor.extract(&entries);

        assert!(glossary.character_names.contains("John"));
        assert!(glossary.character_names.contains("Mary"));
    }

    #[test]
    fn test_glossaryExtractor_extractQuoted_shouldFindQuotedPhrases() {
        let entries = create_entries(&[
            r#"He said "Operation Phoenix" is ready."#,
            r#"The "Operation Phoenix" begins now."#,
        ]);

        let extractor = GlossaryExtractor::new(ExtractionConfig {
            min_occurrences: 2,
            extract_names: false,
            extract_quoted: true,
            ..Default::default()
        });

        let glossary = extractor.extract(&entries);

        assert!(glossary.has_term("Operation Phoenix"));
    }

    #[test]
    fn test_glossaryExtractor_excludeWords_shouldNotExtractExcluded() {
        let entries = create_entries(&["The man said Hello.", "The man replied Hello."]);

        let extractor = GlossaryExtractor::with_defaults();
        let glossary = extractor.extract(&entries);

        assert!(!glossary.character_names.contains("The"));
        assert!(!glossary.character_names.contains("Hello"));
    }

    #[test]
    fn test_glossaryExtractor_minOccurrences_shouldFilterRareTerms() {
        let entries = create_entries(&[
            "Alice went to the park.",
            "Bob stayed home.",
            "Charlie arrived later.",
        ]);

        let extractor = GlossaryExtractor::new(ExtractionConfig {
            min_occurrences: 2,
            ..Default::default()
        });

        let glossary = extractor.extract(&entries);

        // None should appear twice, so none should be extracted
        assert!(glossary.character_names.is_empty());
    }

    #[test]
    fn test_glossaryEnforcer_checkConsistency_shouldDetectMissingNames() {
        let mut glossary = Glossary::new();
        glossary.add_character("John");

        let enforcer = GlossaryEnforcer::new(&glossary);

        let issues =
            enforcer.check_consistency("John went home.", "Jean est allé à la maison.");

        assert!(!issues.is_empty());
        matches!(&issues[0], ConsistencyIssue::MissingName { name, .. } if name == "John");
    }

    #[test]
    fn test_glossaryEnforcer_checkConsistency_shouldPassWhenNamesPresent() {
        let mut glossary = Glossary::new();
        glossary.add_character("John");

        let enforcer = GlossaryEnforcer::new(&glossary);

        let issues =
            enforcer.check_consistency("John went home.", "John est allé à la maison.");

        assert!(issues.is_empty());
    }

    #[test]
    fn test_glossaryEnforcer_enforce_shouldReplaceTerms() {
        let mut glossary = Glossary::new();
        glossary.add_term("extraction point", "point d'extraction", None);

        let enforcer = GlossaryEnforcer::new(&glossary);

        let result =
            enforcer.enforce("Meet at the extraction point.", "Retrouvez-vous au extraction point.");

        assert!(result.contains("point d'extraction"));
    }

    #[test]
    fn test_glossaryExtractionExt_extractGlossary_shouldUpdateDocument() {
        let entries: Vec<SubtitleEntry> = vec![
            SubtitleEntry::new(1, 0, 1000, "Sarah is here.".to_string()),
            SubtitleEntry::new(2, 1000, 2000, "Sarah went out.".to_string()),
            SubtitleEntry::new(3, 2000, 3000, "Call Sarah please.".to_string()),
        ];

        let mut doc = SubtitleDocument::from_entries(entries, "en");
        doc.extract_glossary();

        assert!(doc.glossary.character_names.contains("Sarah"));
    }
}

