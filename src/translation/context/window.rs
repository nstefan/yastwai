/*!
 * Sliding window context for translation.
 *
 * The context window maintains a view into the document that includes:
 * - History summary: compressed representation of earlier content
 * - Recent entries: fully translated entries for consistency
 * - Current batch: entries to translate in this request
 * - Lookahead: upcoming entries for forward context
 */

use serde::{Deserialize, Serialize};

use crate::translation::document::{DocumentEntry, Glossary, SubtitleDocument};
use crate::translation::prompts::TranslatedEntryContext;

/// Configuration for context window sizes.
#[derive(Debug, Clone)]
pub struct ContextWindowConfig {
    /// Number of recent translated entries to include for context
    pub recent_entries_count: usize,

    /// Number of entries to translate per request
    pub batch_size: usize,

    /// Number of lookahead entries for forward context
    pub lookahead_count: usize,

    /// Whether to enable history summarization for long documents
    pub enable_summarization: bool,

    /// Minimum entries before summarization kicks in
    pub summarization_threshold: usize,
}

impl Default for ContextWindowConfig {
    fn default() -> Self {
        Self {
            recent_entries_count: 10,
            batch_size: 15,
            lookahead_count: 5,
            enable_summarization: true,
            summarization_threshold: 50,
        }
    }
}

impl ContextWindowConfig {
    /// Create a minimal config for testing or simple use cases.
    pub fn minimal() -> Self {
        Self {
            recent_entries_count: 3,
            batch_size: 5,
            lookahead_count: 2,
            enable_summarization: false,
            summarization_threshold: 100,
        }
    }

    /// Create a large context config for high-quality translation.
    pub fn large_context() -> Self {
        Self {
            recent_entries_count: 20,
            batch_size: 10,
            lookahead_count: 10,
            enable_summarization: true,
            summarization_threshold: 30,
        }
    }
}

/// A window into the document for context-aware translation.
///
/// The window slides through the document, maintaining context from
/// previously translated entries and providing lookahead for upcoming content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextWindow {
    /// Source language
    pub source_language: String,

    /// Target language
    pub target_language: String,

    /// Summary of all content before the window (compressed history)
    #[serde(default)]
    pub history_summary: Option<String>,

    /// Recently translated entries (for consistency)
    pub recent_entries: Vec<TranslatedEntryContext>,

    /// Current entries to translate
    pub current_batch: Vec<WindowEntry>,

    /// Lookahead entries (for forward context)
    pub lookahead_entries: Vec<WindowEntry>,

    /// Active glossary for this window
    #[serde(default)]
    pub glossary: Glossary,

    /// Current position in the document (first entry ID in current_batch)
    pub position: usize,

    /// Total entries in the document
    pub total_entries: usize,
}

/// A simplified entry for the context window.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowEntry {
    /// Entry ID
    pub id: usize,

    /// Original text
    pub text: String,

    /// Timecode string for reference
    pub timecode: String,

    /// Whether this is a sound effect
    #[serde(default)]
    pub is_sound_effect: bool,
}

impl WindowEntry {
    /// Create from a DocumentEntry.
    pub fn from_document_entry(entry: &DocumentEntry) -> Self {
        Self {
            id: entry.id,
            text: entry.original_text.clone(),
            timecode: entry.timecode.format_srt(),
            is_sound_effect: entry.is_sound_effect(),
        }
    }
}

impl ContextWindow {
    /// Create a new context window at the given position.
    pub fn new(
        doc: &SubtitleDocument,
        position: usize,
        config: &ContextWindowConfig,
        source_language: &str,
        target_language: &str,
    ) -> Self {
        let total_entries = doc.entries.len();

        // Calculate ranges
        let recent_start = position.saturating_sub(config.recent_entries_count);
        let batch_end = (position + config.batch_size).min(total_entries);
        let lookahead_end = (batch_end + config.lookahead_count).min(total_entries);

        // Build recent entries (must be translated)
        let recent_entries: Vec<TranslatedEntryContext> = doc.entries[recent_start..position]
            .iter()
            .filter_map(|entry| {
                entry.translated_text.as_ref().map(|translated| TranslatedEntryContext {
                    id: entry.id,
                    original: entry.original_text.clone(),
                    translated: translated.clone(),
                })
            })
            .collect();

        // Build current batch
        let current_batch: Vec<WindowEntry> = doc.entries[position..batch_end]
            .iter()
            .map(WindowEntry::from_document_entry)
            .collect();

        // Build lookahead
        let lookahead_entries: Vec<WindowEntry> = doc.entries[batch_end..lookahead_end]
            .iter()
            .map(WindowEntry::from_document_entry)
            .collect();

        Self {
            source_language: source_language.to_string(),
            target_language: target_language.to_string(),
            history_summary: doc.context_summary.clone(),
            recent_entries,
            current_batch,
            lookahead_entries,
            glossary: doc.glossary.clone(),
            position,
            total_entries,
        }
    }

    /// Check if the window has reached the end of the document.
    pub fn is_at_end(&self) -> bool {
        self.current_batch.is_empty()
    }

    /// Get the IDs of entries in the current batch.
    pub fn batch_ids(&self) -> Vec<usize> {
        self.current_batch.iter().map(|e| e.id).collect()
    }

    /// Calculate progress as a percentage.
    pub fn progress_percent(&self) -> f32 {
        if self.total_entries == 0 {
            return 100.0;
        }
        (self.position as f32 / self.total_entries as f32) * 100.0
    }

    /// Get the number of entries remaining to translate.
    pub fn remaining_entries(&self) -> usize {
        self.total_entries.saturating_sub(self.position)
    }

    /// Check if this window needs history summarization.
    pub fn needs_summarization(&self, config: &ContextWindowConfig) -> bool {
        config.enable_summarization
            && self.history_summary.is_none()
            && self.position >= config.summarization_threshold
    }

    /// Set the history summary.
    pub fn with_history_summary(mut self, summary: String) -> Self {
        self.history_summary = Some(summary);
        self
    }

    /// Update the glossary with new terms.
    pub fn update_glossary(&mut self, new_terms: &Glossary) {
        self.glossary.merge(new_terms);
    }
}

/// Iterator that yields context windows for a document.
pub struct ContextWindowIterator<'a> {
    doc: &'a SubtitleDocument,
    config: ContextWindowConfig,
    source_language: String,
    target_language: String,
    current_position: usize,
}

impl<'a> ContextWindowIterator<'a> {
    /// Create a new iterator over context windows.
    pub fn new(
        doc: &'a SubtitleDocument,
        config: ContextWindowConfig,
        source_language: &str,
        target_language: &str,
    ) -> Self {
        Self {
            doc,
            config,
            source_language: source_language.to_string(),
            target_language: target_language.to_string(),
            current_position: 0,
        }
    }
}

impl<'a> Iterator for ContextWindowIterator<'a> {
    type Item = ContextWindow;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_position >= self.doc.entries.len() {
            return None;
        }

        let window = ContextWindow::new(
            self.doc,
            self.current_position,
            &self.config,
            &self.source_language,
            &self.target_language,
        );

        // Advance position by batch size
        self.current_position += self.config.batch_size;

        Some(window)
    }
}

/// Extension trait for SubtitleDocument to create context windows.
pub trait ContextWindowExt {
    /// Create an iterator over context windows for this document.
    fn context_windows(
        &self,
        config: ContextWindowConfig,
        source_language: &str,
        target_language: &str,
    ) -> ContextWindowIterator<'_>;

    /// Create a single context window at the given position.
    fn window_at(
        &self,
        position: usize,
        config: &ContextWindowConfig,
        source_language: &str,
        target_language: &str,
    ) -> ContextWindow;
}

impl ContextWindowExt for SubtitleDocument {
    fn context_windows(
        &self,
        config: ContextWindowConfig,
        source_language: &str,
        target_language: &str,
    ) -> ContextWindowIterator<'_> {
        ContextWindowIterator::new(self, config, source_language, target_language)
    }

    fn window_at(
        &self,
        position: usize,
        config: &ContextWindowConfig,
        source_language: &str,
        target_language: &str,
    ) -> ContextWindow {
        ContextWindow::new(self, position, config, source_language, target_language)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::subtitle_processor::SubtitleEntry;

    fn create_test_document(count: usize) -> SubtitleDocument {
        let entries: Vec<SubtitleEntry> = (1..=count)
            .map(|i| {
                SubtitleEntry::new(
                    i,
                    (i as u64 - 1) * 2000,
                    i as u64 * 2000,
                    format!("Line {}", i),
                )
            })
            .collect();

        SubtitleDocument::from_entries(entries, "en")
    }

    #[test]
    fn test_contextWindow_new_shouldCreateWindowAtPosition() {
        let doc = create_test_document(20);
        let config = ContextWindowConfig {
            recent_entries_count: 3,
            batch_size: 5,
            lookahead_count: 2,
            ..Default::default()
        };

        let window = ContextWindow::new(&doc, 5, &config, "en", "fr");

        assert_eq!(window.position, 5);
        assert_eq!(window.total_entries, 20);
        assert_eq!(window.current_batch.len(), 5);
        assert_eq!(window.lookahead_entries.len(), 2);
    }

    #[test]
    fn test_contextWindow_atStart_shouldHaveNoRecentEntries() {
        let doc = create_test_document(20);
        let config = ContextWindowConfig::default();

        let window = ContextWindow::new(&doc, 0, &config, "en", "fr");

        assert!(window.recent_entries.is_empty());
        assert!(!window.current_batch.is_empty());
    }

    #[test]
    fn test_contextWindow_atEnd_shouldHaveNoLookahead() {
        let doc = create_test_document(10);
        let config = ContextWindowConfig {
            batch_size: 5,
            lookahead_count: 5,
            ..Default::default()
        };

        let window = ContextWindow::new(&doc, 5, &config, "en", "fr");

        assert!(window.lookahead_entries.is_empty());
        assert_eq!(window.current_batch.len(), 5);
    }

    #[test]
    fn test_contextWindowIterator_shouldIterateOverDocument() {
        let doc = create_test_document(25);
        let config = ContextWindowConfig {
            batch_size: 10,
            ..Default::default()
        };

        let windows: Vec<ContextWindow> = doc
            .context_windows(config, "en", "fr")
            .collect();

        assert_eq!(windows.len(), 3); // 0-9, 10-19, 20-24
        assert_eq!(windows[0].position, 0);
        assert_eq!(windows[1].position, 10);
        assert_eq!(windows[2].position, 20);
    }

    #[test]
    fn test_contextWindow_progressPercent_shouldCalculateCorrectly() {
        let doc = create_test_document(100);
        let config = ContextWindowConfig::default();

        let window = ContextWindow::new(&doc, 50, &config, "en", "fr");

        assert_eq!(window.progress_percent(), 50.0);
    }

    #[test]
    fn test_contextWindow_needsSummarization_shouldDetectThreshold() {
        let doc = create_test_document(100);
        let config = ContextWindowConfig {
            enable_summarization: true,
            summarization_threshold: 30,
            ..Default::default()
        };

        let window_early = ContextWindow::new(&doc, 20, &config, "en", "fr");
        let window_late = ContextWindow::new(&doc, 50, &config, "en", "fr");

        assert!(!window_early.needs_summarization(&config));
        assert!(window_late.needs_summarization(&config));
    }

    #[test]
    fn test_windowEntry_fromDocumentEntry_shouldPreserveData() {
        let entry = SubtitleEntry::new(1, 0, 1000, "[Door slams]".to_string());
        let doc_entry = crate::translation::document::DocumentEntry::from_subtitle_entry(entry);

        let window_entry = WindowEntry::from_document_entry(&doc_entry);

        assert_eq!(window_entry.id, 1);
        assert_eq!(window_entry.text, "[Door slams]");
        assert!(window_entry.is_sound_effect);
    }
}

