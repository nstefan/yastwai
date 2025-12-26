/*!
 * Core document model types for subtitle translation.
 *
 * These types provide a rich, JSON-serializable representation of subtitle
 * documents that preserves timing, formatting, and translation context.
 */

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::subtitle_processor::SubtitleEntry;

/// Complete subtitle document with metadata and translation context.
///
/// This is the primary data structure for the translation pipeline,
/// containing all information needed for context-aware translation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubtitleDocument {
    /// Document metadata
    pub metadata: DocumentMetadata,

    /// Extracted characters/speakers (if detected)
    #[serde(default)]
    pub characters: Vec<String>,

    /// Terminology glossary for consistency
    #[serde(default)]
    pub glossary: Glossary,

    /// Scene/chapter divisions
    #[serde(default)]
    pub scenes: Vec<Scene>,

    /// All subtitle entries
    pub entries: Vec<DocumentEntry>,

    /// Translation context summary (generated during analysis)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_summary: Option<String>,
}

impl SubtitleDocument {
    /// Create a new document from subtitle entries.
    pub fn from_entries(entries: Vec<SubtitleEntry>, source_language: &str) -> Self {
        let document_entries: Vec<DocumentEntry> = entries
            .into_iter()
            .map(DocumentEntry::from_subtitle_entry)
            .collect();

        Self {
            metadata: DocumentMetadata {
                source_language: source_language.to_string(),
                target_language: None,
                total_entries: document_entries.len(),
                source_file: None,
            },
            characters: Vec::new(),
            glossary: Glossary::default(),
            scenes: Vec::new(),
            entries: document_entries,
            context_summary: None,
        }
    }

    /// Set the target language for translation.
    pub fn with_target_language(mut self, target_language: &str) -> Self {
        self.metadata.target_language = Some(target_language.to_string());
        self
    }

    /// Set the source file path.
    pub fn with_source_file(mut self, source_file: &str) -> Self {
        self.metadata.source_file = Some(source_file.to_string());
        self
    }

    /// Convert back to SubtitleEntry list for SRT output.
    ///
    /// Uses translated text if available, otherwise falls back to original.
    pub fn to_subtitle_entries(&self) -> Vec<SubtitleEntry> {
        self.entries
            .iter()
            .map(|entry| entry.to_subtitle_entry())
            .collect()
    }

    /// Get entries that have been translated.
    pub fn translated_entries(&self) -> Vec<&DocumentEntry> {
        self.entries
            .iter()
            .filter(|e| e.translated_text.is_some())
            .collect()
    }

    /// Get entries that still need translation.
    pub fn pending_entries(&self) -> Vec<&DocumentEntry> {
        self.entries
            .iter()
            .filter(|e| e.translated_text.is_none())
            .collect()
    }

    /// Check if all entries have been translated.
    pub fn is_fully_translated(&self) -> bool {
        self.entries.iter().all(|e| e.translated_text.is_some())
    }

    /// Get translation progress as a percentage.
    pub fn translation_progress(&self) -> f32 {
        if self.entries.is_empty() {
            return 100.0;
        }
        let translated = self.entries.iter().filter(|e| e.translated_text.is_some()).count();
        (translated as f32 / self.entries.len() as f32) * 100.0
    }
}

/// Document metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    /// Source language code (e.g., "en", "eng")
    pub source_language: String,

    /// Target language code (e.g., "fr", "fra")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_language: Option<String>,

    /// Total number of entries
    pub total_entries: usize,

    /// Original source file path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_file: Option<String>,
}

/// A single subtitle entry with translation state.
///
/// Timecodes are immutable after creation to ensure timing integrity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentEntry {
    /// Unique identifier (1-based sequence number)
    pub id: usize,

    /// Immutable timecode information
    pub timecode: Timecode,

    /// Original source text
    pub original_text: String,

    /// Translated text (None if not yet translated)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub translated_text: Option<String>,

    /// Detected speaker (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speaker: Option<String>,

    /// Scene this entry belongs to
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scene_id: Option<usize>,

    /// Formatting tags preserved from original
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub formatting: Vec<FormattingTag>,

    /// Translation confidence score (0.0-1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f32>,
}

impl DocumentEntry {
    /// Create a new document entry from a subtitle entry.
    pub fn from_subtitle_entry(entry: SubtitleEntry) -> Self {
        let formatting = Self::extract_formatting_tags(&entry.text);

        Self {
            id: entry.seq_num,
            timecode: Timecode::from_milliseconds(entry.start_time_ms, entry.end_time_ms),
            original_text: entry.text,
            translated_text: None,
            speaker: None,
            scene_id: None,
            formatting,
            confidence: None,
        }
    }

    /// Set the translated text.
    ///
    /// Note: This does NOT modify the timecode, which is immutable.
    pub fn set_translation(&mut self, text: String, confidence: Option<f32>) {
        self.translated_text = Some(text);
        self.confidence = confidence;
    }

    /// Convert back to a SubtitleEntry.
    ///
    /// Uses translated text if available, otherwise original.
    pub fn to_subtitle_entry(&self) -> SubtitleEntry {
        SubtitleEntry {
            seq_num: self.id,
            start_time_ms: self.timecode.start_ms,
            end_time_ms: self.timecode.end_ms,
            text: self.translated_text.clone().unwrap_or_else(|| self.original_text.clone()),
        }
    }

    /// Extract formatting tags from text.
    fn extract_formatting_tags(text: &str) -> Vec<FormattingTag> {
        let mut tags = Vec::new();

        // Check for italic tags
        if text.contains("<i>") || text.contains("</i>") {
            tags.push(FormattingTag::Italic);
        }

        // Check for bold tags
        if text.contains("<b>") || text.contains("</b>") {
            tags.push(FormattingTag::Bold);
        }

        // Check for underline tags
        if text.contains("<u>") || text.contains("</u>") {
            tags.push(FormattingTag::Underline);
        }

        // Check for position tags like {\an8}
        if text.contains("{\\an") {
            tags.push(FormattingTag::Position);
        }

        // Check for color tags
        if text.contains("<font") || text.contains("</font>") {
            tags.push(FormattingTag::Color);
        }

        tags
    }

    /// Check if this is a sound effect or non-dialogue text.
    pub fn is_sound_effect(&self) -> bool {
        let text = self.original_text.trim();
        // Sound effects are typically in brackets
        (text.starts_with('[') && text.ends_with(']'))
            || (text.starts_with('(') && text.ends_with(')'))
    }
}

/// Immutable timecode representation.
///
/// Once created, timecodes cannot be modified, ensuring timing integrity
/// is preserved throughout the translation process.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Timecode {
    /// Start time in milliseconds
    pub start_ms: u64,

    /// End time in milliseconds
    pub end_ms: u64,
}

impl Timecode {
    /// Create a new timecode from milliseconds.
    pub fn from_milliseconds(start_ms: u64, end_ms: u64) -> Self {
        Self { start_ms, end_ms }
    }

    /// Get duration in milliseconds.
    pub fn duration_ms(&self) -> u64 {
        self.end_ms.saturating_sub(self.start_ms)
    }

    /// Format as SRT timestamp string.
    pub fn format_srt(&self) -> String {
        format!("{} --> {}", Self::format_ms(self.start_ms), Self::format_ms(self.end_ms))
    }

    /// Format milliseconds to SRT format (HH:MM:SS,mmm).
    fn format_ms(ms: u64) -> String {
        let hours = ms / 3_600_000;
        let minutes = (ms % 3_600_000) / 60_000;
        let seconds = (ms % 60_000) / 1_000;
        let millis = ms % 1_000;
        format!("{:02}:{:02}:{:02},{:03}", hours, minutes, seconds, millis)
    }

    /// Parse SRT timestamp to milliseconds.
    pub fn parse_srt_timestamp(timestamp: &str) -> Option<u64> {
        let parts: Vec<&str> = timestamp.split(&[':', ',', '.'][..]).collect();
        if parts.len() != 4 {
            return None;
        }

        let hours: u64 = parts[0].parse().ok()?;
        let minutes: u64 = parts[1].parse().ok()?;
        let seconds: u64 = parts[2].parse().ok()?;
        let millis: u64 = parts[3].parse().ok()?;

        Some(hours * 3_600_000 + minutes * 60_000 + seconds * 1_000 + millis)
    }
}

/// Scene or chapter division in the document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scene {
    /// Unique scene identifier
    pub id: usize,

    /// First entry ID in this scene
    pub start_entry_id: usize,

    /// Last entry ID in this scene
    pub end_entry_id: usize,

    /// Optional scene description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Detected tone (e.g., "tense", "comedic", "romantic")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tone: Option<String>,
}

impl Scene {
    /// Create a new scene.
    pub fn new(id: usize, start_entry_id: usize, end_entry_id: usize) -> Self {
        Self {
            id,
            start_entry_id,
            end_entry_id,
            description: None,
            tone: None,
        }
    }

    /// Set scene description.
    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    /// Set scene tone.
    pub fn with_tone(mut self, tone: &str) -> Self {
        self.tone = Some(tone.to_string());
        self
    }
}

/// Formatting tag types found in subtitles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FormattingTag {
    /// Italic text <i>...</i>
    Italic,
    /// Bold text <b>...</b>
    Bold,
    /// Underlined text <u>...</u>
    Underline,
    /// Position tag {\anX}
    Position,
    /// Color/font tag <font>...</font>
    Color,
}

/// Terminology glossary for translation consistency.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Glossary {
    /// Key terms with their translations
    #[serde(default)]
    pub terms: HashMap<String, GlossaryTerm>,

    /// Character names (should not be translated)
    #[serde(default)]
    pub character_names: HashSet<String>,

    /// Technical terms with specific translations
    #[serde(default)]
    pub technical_terms: HashMap<String, String>,
}

impl Glossary {
    /// Create a new empty glossary.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a term to the glossary.
    pub fn add_term(&mut self, source: &str, target: &str, context: Option<&str>) {
        self.terms.insert(
            source.to_string(),
            GlossaryTerm {
                source: source.to_string(),
                target: target.to_string(),
                context: context.map(|s| s.to_string()),
            },
        );
    }

    /// Add a character name (will not be translated).
    pub fn add_character(&mut self, name: &str) {
        self.character_names.insert(name.to_string());
    }

    /// Add a technical term with its translation.
    pub fn add_technical_term(&mut self, source: &str, target: &str) {
        self.technical_terms.insert(source.to_string(), target.to_string());
    }

    /// Check if a term exists in the glossary.
    pub fn has_term(&self, term: &str) -> bool {
        self.terms.contains_key(term)
            || self.character_names.contains(term)
            || self.technical_terms.contains_key(term)
    }

    /// Get the translation for a term if it exists.
    ///
    /// Returns the target translation for the term. For character names,
    /// returns None since they should not be translated (use `has_character` instead).
    pub fn get_translation(&self, term: &str) -> Option<&str> {
        if let Some(glossary_term) = self.terms.get(term) {
            return Some(&glossary_term.target);
        }
        self.technical_terms.get(term).map(|s| s.as_str())
    }

    /// Check if a term is a character name (should not be translated).
    pub fn is_character_name(&self, name: &str) -> bool {
        self.character_names.contains(name)
    }

    /// Merge another glossary into this one.
    pub fn merge(&mut self, other: &Glossary) {
        for (key, term) in &other.terms {
            self.terms.insert(key.clone(), term.clone());
        }
        for name in &other.character_names {
            self.character_names.insert(name.clone());
        }
        for (key, value) in &other.technical_terms {
            self.technical_terms.insert(key.clone(), value.clone());
        }
    }

    /// Check if the glossary is empty.
    pub fn is_empty(&self) -> bool {
        self.terms.is_empty() && self.character_names.is_empty() && self.technical_terms.is_empty()
    }
}

/// A single glossary term with translation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlossaryTerm {
    /// Source term
    pub source: String,

    /// Target translation
    pub target: String,

    /// Optional context for when to use this translation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timecode_fromMilliseconds_shouldCreateTimecode() {
        let tc = Timecode::from_milliseconds(1000, 5000);
        assert_eq!(tc.start_ms, 1000);
        assert_eq!(tc.end_ms, 5000);
        assert_eq!(tc.duration_ms(), 4000);
    }

    #[test]
    fn test_timecode_formatSrt_shouldFormatCorrectly() {
        let tc = Timecode::from_milliseconds(3661001, 3665500);
        assert_eq!(tc.format_srt(), "01:01:01,001 --> 01:01:05,500");
    }

    #[test]
    fn test_timecode_parseSrtTimestamp_shouldParseCorrectly() {
        let ms = Timecode::parse_srt_timestamp("01:02:03,456");
        assert_eq!(ms, Some(3723456));
    }

    #[test]
    fn test_documentEntry_fromSubtitleEntry_shouldConvertCorrectly() {
        let entry = SubtitleEntry::new(1, 1000, 5000, "Hello, world!".to_string());
        let doc_entry = DocumentEntry::from_subtitle_entry(entry);

        assert_eq!(doc_entry.id, 1);
        assert_eq!(doc_entry.timecode.start_ms, 1000);
        assert_eq!(doc_entry.timecode.end_ms, 5000);
        assert_eq!(doc_entry.original_text, "Hello, world!");
        assert!(doc_entry.translated_text.is_none());
    }

    #[test]
    fn test_documentEntry_setTranslation_shouldNotModifyTimecode() {
        let entry = SubtitleEntry::new(1, 1000, 5000, "Hello".to_string());
        let mut doc_entry = DocumentEntry::from_subtitle_entry(entry);

        doc_entry.set_translation("Bonjour".to_string(), Some(0.95));

        assert_eq!(doc_entry.timecode.start_ms, 1000);
        assert_eq!(doc_entry.timecode.end_ms, 5000);
        assert_eq!(doc_entry.translated_text, Some("Bonjour".to_string()));
        assert_eq!(doc_entry.confidence, Some(0.95));
    }

    #[test]
    fn test_documentEntry_toSubtitleEntry_shouldUseTranslatedText() {
        let entry = SubtitleEntry::new(1, 1000, 5000, "Hello".to_string());
        let mut doc_entry = DocumentEntry::from_subtitle_entry(entry);
        doc_entry.set_translation("Bonjour".to_string(), None);

        let result = doc_entry.to_subtitle_entry();

        assert_eq!(result.text, "Bonjour");
        assert_eq!(result.start_time_ms, 1000);
        assert_eq!(result.end_time_ms, 5000);
    }

    #[test]
    fn test_documentEntry_extractFormattingTags_shouldDetectItalic() {
        let entry = SubtitleEntry::new(1, 0, 1000, "<i>Whispered text</i>".to_string());
        let doc_entry = DocumentEntry::from_subtitle_entry(entry);

        assert!(doc_entry.formatting.contains(&FormattingTag::Italic));
    }

    #[test]
    fn test_documentEntry_isSoundEffect_shouldDetectBrackets() {
        let entry = SubtitleEntry::new(1, 0, 1000, "[Door slams]".to_string());
        let doc_entry = DocumentEntry::from_subtitle_entry(entry);

        assert!(doc_entry.is_sound_effect());
    }

    #[test]
    fn test_subtitleDocument_fromEntries_shouldCreateDocument() {
        let entries = vec![
            SubtitleEntry::new(1, 0, 1000, "Line 1".to_string()),
            SubtitleEntry::new(2, 1000, 2000, "Line 2".to_string()),
        ];

        let doc = SubtitleDocument::from_entries(entries, "en");

        assert_eq!(doc.entries.len(), 2);
        assert_eq!(doc.metadata.source_language, "en");
        assert_eq!(doc.metadata.total_entries, 2);
    }

    #[test]
    fn test_subtitleDocument_translationProgress_shouldCalculateCorrectly() {
        let entries = vec![
            SubtitleEntry::new(1, 0, 1000, "Line 1".to_string()),
            SubtitleEntry::new(2, 1000, 2000, "Line 2".to_string()),
            SubtitleEntry::new(3, 2000, 3000, "Line 3".to_string()),
            SubtitleEntry::new(4, 3000, 4000, "Line 4".to_string()),
        ];

        let mut doc = SubtitleDocument::from_entries(entries, "en");

        assert_eq!(doc.translation_progress(), 0.0);

        doc.entries[0].set_translation("Ligne 1".to_string(), None);
        doc.entries[1].set_translation("Ligne 2".to_string(), None);

        assert_eq!(doc.translation_progress(), 50.0);
    }

    #[test]
    fn test_glossary_addTerm_shouldStoreAndRetrieve() {
        let mut glossary = Glossary::new();
        glossary.add_term("hello", "bonjour", Some("greeting"));

        assert!(glossary.has_term("hello"));
        assert_eq!(glossary.get_translation("hello"), Some("bonjour"));
    }

    #[test]
    fn test_glossary_characterNames_shouldBeDetected() {
        let mut glossary = Glossary::new();
        glossary.add_character("John");

        assert!(glossary.has_term("John"));
        assert!(glossary.is_character_name("John"));
        assert!(!glossary.is_character_name("Unknown"));
    }

    #[test]
    fn test_scene_new_shouldCreateScene() {
        let scene = Scene::new(1, 1, 10).with_description("Opening scene").with_tone("mysterious");

        assert_eq!(scene.id, 1);
        assert_eq!(scene.start_entry_id, 1);
        assert_eq!(scene.end_entry_id, 10);
        assert_eq!(scene.description, Some("Opening scene".to_string()));
        assert_eq!(scene.tone, Some("mysterious".to_string()));
    }
}

