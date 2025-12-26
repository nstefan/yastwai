/*!
 * Prompt templates for subtitle translation.
 *
 * These templates are designed to produce high-quality, context-aware
 * translations with structured JSON output.
 */

use serde::{Deserialize, Serialize};

use crate::translation::document::{DocumentEntry, Glossary};

/// System prompt template for subtitle translation.
#[derive(Debug, Clone)]
pub struct PromptTemplate {
    /// The template string with placeholders
    template: String,
}

impl PromptTemplate {
    /// The default system prompt for subtitle translation.
    pub const SUBTITLE_TRANSLATOR: &'static str = r#"You are an expert subtitle translator specializing in {source_language} to {target_language} translation.

## Your Role
- Translate dialogue naturally while preserving meaning and emotion
- Maintain consistency with provided terminology/glossary
- Preserve formatting tags, sound effects, and speaker indicators
- Keep translations concise (subtitles have limited display time)

## Context Understanding
- Review the history summary to understand the narrative flow
- Reference recent translations for style and terminology consistency
- Use lookahead entries to anticipate context when helpful
- Follow the glossary strictly for names and key terms

## Output Requirements
- Return ONLY valid JSON matching the requested schema
- Include a confidence score (0.0-1.0) for each translation
- Do not include any text outside the JSON structure

## Quality Standards
- Natural, idiomatic {target_language}
- Appropriate register (formal/informal) based on dialogue context
- Length should be similar to original (within 120% where possible)
- Preserve [sound effects] and (parentheticals) exactly as formatted
- Never translate character names unless specifically instructed"#;

    /// Create a new prompt template.
    pub fn new(template: &str) -> Self {
        Self {
            template: template.to_string(),
        }
    }

    /// Create the default subtitle translator template.
    pub fn subtitle_translator() -> Self {
        Self::new(Self::SUBTITLE_TRANSLATOR)
    }

    /// Render the template with the given variables.
    pub fn render(&self, source_language: &str, target_language: &str) -> String {
        self.template
            .replace("{source_language}", source_language)
            .replace("{target_language}", target_language)
    }
}

impl Default for PromptTemplate {
    fn default() -> Self {
        Self::subtitle_translator()
    }
}

/// Builder for constructing translation prompts with context.
#[derive(Debug, Clone)]
pub struct TranslationPromptBuilder {
    source_language: String,
    target_language: String,
    history_summary: Option<String>,
    recent_translations: Vec<TranslatedEntryContext>,
    entries_to_translate: Vec<EntryToTranslate>,
    lookahead_entries: Vec<LookaheadEntry>,
    glossary: Option<Glossary>,
    custom_instructions: Option<String>,
}

impl TranslationPromptBuilder {
    /// Create a new prompt builder.
    pub fn new(source_language: &str, target_language: &str) -> Self {
        Self {
            source_language: source_language.to_string(),
            target_language: target_language.to_string(),
            history_summary: None,
            recent_translations: Vec::new(),
            entries_to_translate: Vec::new(),
            lookahead_entries: Vec::new(),
            glossary: None,
            custom_instructions: None,
        }
    }

    /// Set the history summary for context.
    pub fn with_history_summary(mut self, summary: &str) -> Self {
        self.history_summary = Some(summary.to_string());
        self
    }

    /// Add recent translations for consistency.
    pub fn with_recent_translations(mut self, entries: Vec<TranslatedEntryContext>) -> Self {
        self.recent_translations = entries;
        self
    }

    /// Set the entries to translate.
    pub fn with_entries_to_translate(mut self, entries: &[DocumentEntry]) -> Self {
        self.entries_to_translate = entries
            .iter()
            .map(|e| EntryToTranslate {
                id: e.id,
                text: e.original_text.clone(),
                timecode: e.timecode.format_srt(),
            })
            .collect();
        self
    }

    /// Set lookahead entries for forward context.
    pub fn with_lookahead(mut self, entries: &[DocumentEntry]) -> Self {
        self.lookahead_entries = entries
            .iter()
            .map(|e| LookaheadEntry {
                id: e.id,
                text: e.original_text.clone(),
            })
            .collect();
        self
    }

    /// Set the glossary for terminology consistency.
    pub fn with_glossary(mut self, glossary: &Glossary) -> Self {
        self.glossary = Some(glossary.clone());
        self
    }

    /// Set custom instructions.
    pub fn with_custom_instructions(mut self, instructions: &str) -> Self {
        self.custom_instructions = Some(instructions.to_string());
        self
    }

    /// Build the system prompt.
    pub fn build_system_prompt(&self) -> String {
        PromptTemplate::subtitle_translator().render(&self.source_language, &self.target_language)
    }

    /// Build the user prompt as a JSON request.
    pub fn build_user_prompt(&self) -> String {
        let request = TranslationRequest {
            task: "translate_subtitles".to_string(),
            source_language: self.source_language.clone(),
            target_language: self.target_language.clone(),
            context: ContextData {
                history_summary: self.history_summary.clone(),
                recent_translations: if self.recent_translations.is_empty() {
                    None
                } else {
                    Some(self.recent_translations.clone())
                },
                lookahead: if self.lookahead_entries.is_empty() {
                    None
                } else {
                    Some(self.lookahead_entries.clone())
                },
                glossary: self.glossary.as_ref().and_then(|g| {
                    if g.is_empty() {
                        None
                    } else {
                        Some(GlossaryContext {
                            character_names: g.character_names.iter().cloned().collect(),
                            terms: g
                                .terms
                                .iter()
                                .map(|(k, v)| (k.clone(), v.target.clone()))
                                .collect(),
                        })
                    }
                }),
            },
            entries_to_translate: self.entries_to_translate.clone(),
            instructions: TranslationInstructions {
                preserve_formatting: true,
                preserve_sound_effects: true,
                max_length_ratio: 1.2,
                custom: self.custom_instructions.clone(),
            },
        };

        serde_json::to_string_pretty(&request).unwrap_or_else(|_| "{}".to_string())
    }

    /// Build both system and user prompts.
    pub fn build(&self) -> (String, String) {
        (self.build_system_prompt(), self.build_user_prompt())
    }
}

/// Translation request structure for JSON communication with LLM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationRequest {
    /// Task identifier
    pub task: String,

    /// Source language
    pub source_language: String,

    /// Target language
    pub target_language: String,

    /// Context information
    pub context: ContextData,

    /// Entries to translate
    pub entries_to_translate: Vec<EntryToTranslate>,

    /// Translation instructions
    pub instructions: TranslationInstructions,
}

/// Context data for translation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextData {
    /// Summary of content before the current window
    #[serde(skip_serializing_if = "Option::is_none")]
    pub history_summary: Option<String>,

    /// Recently translated entries
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recent_translations: Option<Vec<TranslatedEntryContext>>,

    /// Upcoming entries for forward context
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lookahead: Option<Vec<LookaheadEntry>>,

    /// Glossary for terminology consistency
    #[serde(skip_serializing_if = "Option::is_none")]
    pub glossary: Option<GlossaryContext>,
}

/// A recently translated entry for context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslatedEntryContext {
    /// Entry ID
    pub id: usize,

    /// Original text
    pub original: String,

    /// Translated text
    pub translated: String,
}

/// An entry to translate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryToTranslate {
    /// Entry ID
    pub id: usize,

    /// Text to translate
    pub text: String,

    /// Timecode (for reference, not to be modified)
    pub timecode: String,
}

/// A lookahead entry for forward context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LookaheadEntry {
    /// Entry ID
    pub id: usize,

    /// Original text
    pub text: String,
}

/// Glossary context for the prompt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlossaryContext {
    /// Character names (should not be translated)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub character_names: Vec<String>,

    /// Terms with their translations
    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub terms: std::collections::HashMap<String, String>,
}

/// Translation instructions for the LLM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationInstructions {
    /// Whether to preserve formatting tags
    pub preserve_formatting: bool,

    /// Whether to preserve sound effects in brackets
    pub preserve_sound_effects: bool,

    /// Maximum length ratio compared to original
    pub max_length_ratio: f32,

    /// Custom instructions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom: Option<String>,
}

/// Expected response structure from LLM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationResponse {
    /// Translated entries
    pub translations: Vec<TranslatedEntry>,

    /// Optional notes from the translator
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<TranslationNotes>,
}

/// A single translated entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslatedEntry {
    /// Entry ID (must match request)
    pub id: usize,

    /// Translated text
    pub translated: String,

    /// Confidence score (0.0-1.0)
    #[serde(default)]
    pub confidence: Option<f32>,
}

/// Optional notes from the translation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationNotes {
    /// Suggested glossary updates
    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub glossary_updates: std::collections::HashMap<String, String>,

    /// Scene context notes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scene_context: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_promptTemplate_render_shouldReplaceVariables() {
        let template = PromptTemplate::subtitle_translator();
        let rendered = template.render("English", "French");

        assert!(rendered.contains("English to French"));
        assert!(rendered.contains("idiomatic French"));
        assert!(!rendered.contains("{source_language}"));
        assert!(!rendered.contains("{target_language}"));
    }

    #[test]
    fn test_translationPromptBuilder_buildSystemPrompt_shouldBeValid() {
        let builder = TranslationPromptBuilder::new("en", "fr");
        let prompt = builder.build_system_prompt();

        assert!(prompt.contains("en to fr"));
    }

    #[test]
    fn test_translationPromptBuilder_buildUserPrompt_shouldBeValidJson() {
        let builder = TranslationPromptBuilder::new("English", "French")
            .with_history_summary("The story begins...");

        let user_prompt = builder.build_user_prompt();

        // Should be valid JSON
        let parsed: serde_json::Result<TranslationRequest> = serde_json::from_str(&user_prompt);
        assert!(parsed.is_ok());

        let request = parsed.unwrap();
        assert_eq!(request.task, "translate_subtitles");
        assert_eq!(request.source_language, "English");
        assert_eq!(request.target_language, "French");
    }

    #[test]
    fn test_translationResponse_deserialize_shouldParseValidJson() {
        let json = r#"{
            "translations": [
                {"id": 1, "translated": "Bonjour", "confidence": 0.95},
                {"id": 2, "translated": "Au revoir", "confidence": 0.92}
            ],
            "notes": {
                "glossary_updates": {"hello": "bonjour"},
                "scene_context": "Greeting scene"
            }
        }"#;

        let response: TranslationResponse = serde_json::from_str(json).unwrap();

        assert_eq!(response.translations.len(), 2);
        assert_eq!(response.translations[0].id, 1);
        assert_eq!(response.translations[0].translated, "Bonjour");
        assert_eq!(response.translations[0].confidence, Some(0.95));
    }
}

