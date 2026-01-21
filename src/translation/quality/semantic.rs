/*!
 * Semantic validation for translation quality.
 *
 * This module provides AI-powered semantic validation to verify that
 * translations preserve the meaning of the original text. It uses
 * an LLM to check for semantic equivalence.
 */

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::translation::core::TranslationService;

/// Result of semantic validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticValidationResult {
    /// Whether the translation is semantically equivalent
    pub is_equivalent: bool,

    /// Confidence score (0.0-1.0)
    pub confidence: f32,

    /// Identified semantic issues
    pub issues: Vec<SemanticIssue>,

    /// Overall summary
    pub summary: String,
}

impl SemanticValidationResult {
    /// Create a result indicating semantic equivalence
    pub fn equivalent(confidence: f32) -> Self {
        Self {
            is_equivalent: true,
            confidence,
            issues: Vec::new(),
            summary: "Translation preserves the original meaning".to_string(),
        }
    }

    /// Create a result indicating semantic divergence
    pub fn divergent(confidence: f32, issues: Vec<SemanticIssue>) -> Self {
        let summary = if issues.is_empty() {
            "Translation may not preserve the original meaning".to_string()
        } else {
            format!("Found {} semantic issues", issues.len())
        };

        Self {
            is_equivalent: false,
            confidence,
            issues,
            summary,
        }
    }

    /// Check if this result passes validation
    pub fn passed(&self) -> bool {
        self.is_equivalent && self.confidence >= 0.7
    }
}

/// Types of semantic issues
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SemanticIssue {
    /// Meaning was changed or distorted
    MeaningChanged {
        original_meaning: String,
        translated_meaning: String,
    },

    /// Information was added that wasn't in the original
    InformationAdded {
        added_content: String,
    },

    /// Information was omitted from the original
    InformationOmitted {
        omitted_content: String,
    },

    /// Tone or register was changed significantly
    ToneChanged {
        original_tone: String,
        translated_tone: String,
    },

    /// Named entity was incorrectly handled
    EntityError {
        entity: String,
        issue: String,
    },
}

impl SemanticIssue {
    /// Get a human-readable description of the issue
    pub fn description(&self) -> String {
        match self {
            SemanticIssue::MeaningChanged { original_meaning, translated_meaning } => {
                format!(
                    "Meaning changed: '{}' became '{}'",
                    original_meaning, translated_meaning
                )
            }
            SemanticIssue::InformationAdded { added_content } => {
                format!("Added information not in original: '{}'", added_content)
            }
            SemanticIssue::InformationOmitted { omitted_content } => {
                format!("Omitted information from original: '{}'", omitted_content)
            }
            SemanticIssue::ToneChanged { original_tone, translated_tone } => {
                format!(
                    "Tone changed from '{}' to '{}'",
                    original_tone, translated_tone
                )
            }
            SemanticIssue::EntityError { entity, issue } => {
                format!("Entity '{}': {}", entity, issue)
            }
        }
    }

    /// Get the severity of this issue (0.0-1.0)
    pub fn severity(&self) -> f32 {
        match self {
            SemanticIssue::MeaningChanged { .. } => 1.0,
            SemanticIssue::InformationOmitted { .. } => 0.8,
            SemanticIssue::InformationAdded { .. } => 0.6,
            SemanticIssue::ToneChanged { .. } => 0.4,
            SemanticIssue::EntityError { .. } => 0.7,
        }
    }
}

/// Configuration for semantic validation
#[derive(Debug, Clone)]
pub struct SemanticValidationConfig {
    /// Minimum confidence threshold for passing
    pub min_confidence: f32,

    /// Whether to check for tone/register changes
    pub check_tone: bool,

    /// Whether to check for added information
    pub check_additions: bool,

    /// Whether to check for omitted information
    pub check_omissions: bool,
}

impl Default for SemanticValidationConfig {
    fn default() -> Self {
        Self {
            min_confidence: 0.7,
            check_tone: true,
            check_additions: true,
            check_omissions: true,
        }
    }
}

impl SemanticValidationConfig {
    /// Create a strict configuration
    pub fn strict() -> Self {
        Self {
            min_confidence: 0.85,
            check_tone: true,
            check_additions: true,
            check_omissions: true,
        }
    }

    /// Create a lenient configuration
    pub fn lenient() -> Self {
        Self {
            min_confidence: 0.5,
            check_tone: false,
            check_additions: false,
            check_omissions: true,
        }
    }
}

/// Semantic validator using LLM
pub struct SemanticValidator {
    config: SemanticValidationConfig,
}

impl SemanticValidator {
    /// Create a new semantic validator with the given configuration
    pub fn new(config: SemanticValidationConfig) -> Self {
        Self { config }
    }

    /// Create a validator with default configuration
    pub fn with_defaults() -> Self {
        Self::new(SemanticValidationConfig::default())
    }

    /// Validate that a translation preserves the meaning of the original
    pub async fn validate(
        &self,
        service: &TranslationService,
        original: &str,
        translated: &str,
        source_lang: &str,
        target_lang: &str,
    ) -> Result<SemanticValidationResult> {
        // Build the validation prompt
        let prompt = self.build_validation_prompt(original, translated, source_lang, target_lang);

        // Call the LLM for semantic analysis
        let response = service.translate_text(&prompt, "validation", "semantic_check").await?;

        // Parse the response
        self.parse_validation_response(&response, original, translated)
    }

    /// Build the prompt for semantic validation
    fn build_validation_prompt(
        &self,
        original: &str,
        translated: &str,
        source_lang: &str,
        target_lang: &str,
    ) -> String {
        let mut checks = vec!["meaning preservation"];
        if self.config.check_tone {
            checks.push("tone/register");
        }
        if self.config.check_additions {
            checks.push("added information");
        }
        if self.config.check_omissions {
            checks.push("omitted information");
        }

        format!(
            r#"Analyze the semantic equivalence between these texts.

Original ({source_lang}): "{original}"
Translation ({target_lang}): "{translated}"

Check for: {checks}

Respond with JSON:
{{
    "is_equivalent": true/false,
    "confidence": 0.0-1.0,
    "issues": [
        {{"type": "meaning_changed|info_added|info_omitted|tone_changed|entity_error", "description": "..."}}
    ],
    "summary": "brief summary"
}}"#,
            source_lang = source_lang,
            target_lang = target_lang,
            original = original,
            translated = translated,
            checks = checks.join(", ")
        )
    }

    /// Parse the LLM response into a validation result
    fn parse_validation_response(
        &self,
        response: &str,
        original: &str,
        translated: &str,
    ) -> Result<SemanticValidationResult> {
        // Try to parse as JSON
        if let Ok(parsed) = self.try_parse_json(response) {
            return Ok(parsed);
        }

        // Fallback: Use heuristics based on the response text
        self.heuristic_validation(response, original, translated)
    }

    /// Try to parse response as JSON
    fn try_parse_json(&self, response: &str) -> Result<SemanticValidationResult> {
        // Extract JSON from response
        let json_str = self.extract_json(response)?;

        #[derive(Deserialize)]
        struct ParsedResponse {
            is_equivalent: bool,
            confidence: f32,
            issues: Option<Vec<ParsedIssue>>,
            summary: Option<String>,
        }

        #[derive(Deserialize)]
        struct ParsedIssue {
            #[serde(rename = "type")]
            issue_type: String,
            description: String,
        }

        let parsed: ParsedResponse = serde_json::from_str(&json_str)?;

        let issues = parsed
            .issues
            .unwrap_or_default()
            .into_iter()
            .filter_map(|i| self.convert_issue(&i.issue_type, &i.description))
            .collect();

        Ok(SemanticValidationResult {
            is_equivalent: parsed.is_equivalent,
            confidence: parsed.confidence.clamp(0.0, 1.0),
            issues,
            summary: parsed.summary.unwrap_or_else(|| "Validation complete".to_string()),
        })
    }

    /// Extract JSON from response text
    fn extract_json(&self, response: &str) -> Result<String> {
        let trimmed = response.trim();

        // If it starts with {, use as-is
        if trimmed.starts_with('{') {
            if let Some(end) = trimmed.rfind('}') {
                return Ok(trimmed[..=end].to_string());
            }
        }

        // Look for JSON in code block
        if let Some(start) = trimmed.find("```json") {
            if let Some(end) = trimmed[start + 7..].find("```") {
                return Ok(trimmed[start + 7..start + 7 + end].trim().to_string());
            }
        }

        // Look for first { to last }
        if let (Some(start), Some(end)) = (trimmed.find('{'), trimmed.rfind('}')) {
            if end > start {
                return Ok(trimmed[start..=end].to_string());
            }
        }

        anyhow::bail!("Could not extract JSON from response")
    }

    /// Convert parsed issue type to SemanticIssue
    fn convert_issue(&self, issue_type: &str, description: &str) -> Option<SemanticIssue> {
        match issue_type.to_lowercase().as_str() {
            "meaning_changed" => Some(SemanticIssue::MeaningChanged {
                original_meaning: description.to_string(),
                translated_meaning: String::new(),
            }),
            "info_added" | "information_added" => Some(SemanticIssue::InformationAdded {
                added_content: description.to_string(),
            }),
            "info_omitted" | "information_omitted" => Some(SemanticIssue::InformationOmitted {
                omitted_content: description.to_string(),
            }),
            "tone_changed" => Some(SemanticIssue::ToneChanged {
                original_tone: description.to_string(),
                translated_tone: String::new(),
            }),
            "entity_error" => Some(SemanticIssue::EntityError {
                entity: String::new(),
                issue: description.to_string(),
            }),
            _ => None,
        }
    }

    /// Heuristic-based validation when JSON parsing fails
    fn heuristic_validation(
        &self,
        response: &str,
        _original: &str,
        _translated: &str,
    ) -> Result<SemanticValidationResult> {
        let response_lower = response.to_lowercase();

        // Look for positive indicators
        let positive_indicators = [
            "equivalent",
            "preserves",
            "accurate",
            "correct",
            "faithful",
            "same meaning",
        ];
        let negative_indicators = [
            "different",
            "changed",
            "lost",
            "missing",
            "incorrect",
            "wrong",
            "distorted",
        ];

        let positive_count = positive_indicators
            .iter()
            .filter(|&ind| response_lower.contains(ind))
            .count();
        let negative_count = negative_indicators
            .iter()
            .filter(|&ind| response_lower.contains(ind))
            .count();

        if positive_count > negative_count {
            Ok(SemanticValidationResult::equivalent(0.7))
        } else if negative_count > positive_count {
            Ok(SemanticValidationResult::divergent(0.6, Vec::new()))
        } else {
            // Uncertain - return neutral result
            Ok(SemanticValidationResult {
                is_equivalent: true,
                confidence: 0.5,
                issues: Vec::new(),
                summary: "Could not determine semantic equivalence with certainty".to_string(),
            })
        }
    }
}

impl Default for SemanticValidator {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semanticValidationResult_equivalent_shouldPassValidation() {
        let result = SemanticValidationResult::equivalent(0.9);

        assert!(result.is_equivalent);
        assert!(result.passed());
        assert!(result.issues.is_empty());
    }

    #[test]
    fn test_semanticValidationResult_divergent_shouldFailValidation() {
        let issues = vec![SemanticIssue::MeaningChanged {
            original_meaning: "Hello".to_string(),
            translated_meaning: "Goodbye".to_string(),
        }];

        let result = SemanticValidationResult::divergent(0.8, issues);

        assert!(!result.is_equivalent);
        assert!(!result.passed());
        assert_eq!(result.issues.len(), 1);
    }

    #[test]
    fn test_semanticValidationResult_lowConfidence_shouldNotPass() {
        let result = SemanticValidationResult::equivalent(0.5);

        assert!(result.is_equivalent);
        assert!(!result.passed()); // Low confidence fails
    }

    #[test]
    fn test_semanticIssue_severity_shouldRankCorrectly() {
        let meaning_changed = SemanticIssue::MeaningChanged {
            original_meaning: "test".to_string(),
            translated_meaning: "wrong".to_string(),
        };
        let tone_changed = SemanticIssue::ToneChanged {
            original_tone: "formal".to_string(),
            translated_tone: "casual".to_string(),
        };

        assert!(meaning_changed.severity() > tone_changed.severity());
    }

    #[test]
    fn test_semanticIssue_description_shouldBeReadable() {
        let issue = SemanticIssue::InformationOmitted {
            omitted_content: "important detail".to_string(),
        };

        let desc = issue.description();
        assert!(desc.contains("Omitted"));
        assert!(desc.contains("important detail"));
    }

    #[test]
    fn test_semanticValidator_buildPrompt_shouldIncludeAllParts() {
        let validator = SemanticValidator::with_defaults();
        let prompt = validator.build_validation_prompt(
            "Hello world",
            "Bonjour le monde",
            "en",
            "fr",
        );

        assert!(prompt.contains("Hello world"));
        assert!(prompt.contains("Bonjour le monde"));
        assert!(prompt.contains("en"));
        assert!(prompt.contains("fr"));
        assert!(prompt.contains("meaning preservation"));
    }

    #[test]
    fn test_semanticValidator_extractJson_shouldHandleWrappedJson() {
        let validator = SemanticValidator::with_defaults();

        let response = r#"Here is my analysis:
```json
{"is_equivalent": true, "confidence": 0.9}
```
"#;

        let json = validator.extract_json(response).unwrap();
        assert!(json.contains("is_equivalent"));
    }

    #[test]
    fn test_semanticValidator_heuristicValidation_positive_shouldReturnEquivalent() {
        let validator = SemanticValidator::with_defaults();

        let response = "The translation is accurate and preserves the original meaning.";
        let result = validator.heuristic_validation(response, "test", "test").unwrap();

        assert!(result.is_equivalent);
    }

    #[test]
    fn test_semanticValidator_heuristicValidation_negative_shouldReturnDivergent() {
        let validator = SemanticValidator::with_defaults();

        let response = "The meaning has changed significantly and important details are missing.";
        let result = validator.heuristic_validation(response, "test", "test").unwrap();

        assert!(!result.is_equivalent);
    }

    #[test]
    fn test_semanticValidationConfig_strict_shouldHaveHighThreshold() {
        let config = SemanticValidationConfig::strict();
        assert!(config.min_confidence > 0.8);
    }

    #[test]
    fn test_semanticValidationConfig_lenient_shouldHaveLowThreshold() {
        let config = SemanticValidationConfig::lenient();
        assert!(config.min_confidence < 0.6);
    }
}
