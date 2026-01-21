/*!
 * Analysis pass for document preprocessing.
 *
 * This pass analyzes the document before translation to extract:
 * - Character names (for preservation during translation)
 * - Terminology (for consistent translation)
 * - Scene boundaries (for context segmentation)
 * - Content summary (for long document context)
 */

use crate::translation::context::{
    ExtractionConfig, GlossaryExtractor, GlossaryPreflightChecker, HistorySummarizer,
    PreflightReport, SceneDetectionConfig, SceneDetector, SummarizationConfig,
};
use crate::translation::document::{Glossary, Scene, SubtitleDocument};

/// Configuration for the analysis pass.
#[derive(Debug, Clone)]
pub struct AnalysisConfig {
    /// Configuration for glossary extraction
    pub extraction_config: ExtractionConfig,

    /// Configuration for scene detection
    pub scene_config: SceneDetectionConfig,

    /// Configuration for history summarization
    pub summarization_config: SummarizationConfig,

    /// Whether to run glossary extraction
    pub extract_glossary: bool,

    /// Whether to detect scenes
    pub detect_scenes: bool,

    /// Whether to generate a document summary
    pub generate_summary: bool,

    /// Whether to run glossary preflight check
    pub run_preflight: bool,
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            extraction_config: ExtractionConfig::default(),
            scene_config: SceneDetectionConfig::default(),
            summarization_config: SummarizationConfig::default(),
            extract_glossary: true,
            detect_scenes: true,
            generate_summary: true,
            run_preflight: false, // Controlled by experimental flag
        }
    }
}

impl AnalysisConfig {
    /// Create a minimal analysis config (faster, less thorough).
    pub fn minimal() -> Self {
        Self {
            extraction_config: ExtractionConfig::minimal(),
            scene_config: SceneDetectionConfig::default(),
            summarization_config: SummarizationConfig::default(),
            extract_glossary: true,
            detect_scenes: false,
            generate_summary: false,
            run_preflight: false,
        }
    }

    /// Create a thorough analysis config (slower, more comprehensive).
    pub fn thorough() -> Self {
        Self {
            extraction_config: ExtractionConfig::aggressive(),
            scene_config: SceneDetectionConfig::default(),
            summarization_config: SummarizationConfig::default(),
            extract_glossary: true,
            detect_scenes: true,
            generate_summary: true,
            run_preflight: true,
        }
    }

    /// Enable or disable glossary extraction.
    pub fn with_glossary_extraction(mut self, enabled: bool) -> Self {
        self.extract_glossary = enabled;
        self
    }

    /// Enable or disable scene detection.
    pub fn with_scene_detection(mut self, enabled: bool) -> Self {
        self.detect_scenes = enabled;
        self
    }

    /// Enable or disable summary generation.
    pub fn with_summary_generation(mut self, enabled: bool) -> Self {
        self.generate_summary = enabled;
        self
    }

    /// Enable or disable glossary preflight checking.
    pub fn with_preflight(mut self, enabled: bool) -> Self {
        self.run_preflight = enabled;
        self
    }
}

/// Result of the analysis pass.
#[derive(Debug, Clone)]
pub struct AnalysisResult {
    /// Extracted glossary (characters, terms)
    pub glossary: Glossary,

    /// Detected scenes
    pub scenes: Vec<Scene>,

    /// Document summary (if generated)
    pub summary: Option<String>,

    /// Number of characters detected
    pub character_count: usize,

    /// Number of terms extracted
    pub term_count: usize,

    /// Number of scenes detected
    pub scene_count: usize,

    /// Glossary preflight report (if run)
    pub preflight_report: Option<PreflightReport>,
}

impl AnalysisResult {
    /// Create an empty analysis result.
    pub fn empty() -> Self {
        Self {
            glossary: Glossary::new(),
            scenes: Vec::new(),
            summary: None,
            character_count: 0,
            term_count: 0,
            scene_count: 0,
            preflight_report: None,
        }
    }

    /// Check if any analysis was performed.
    pub fn has_data(&self) -> bool {
        !self.glossary.is_empty()
            || !self.scenes.is_empty()
            || self.summary.is_some()
            || self.preflight_report.is_some()
    }

    /// Get a summary description of the analysis.
    pub fn description(&self) -> String {
        let mut parts = Vec::new();

        if self.character_count > 0 {
            parts.push(format!("{} characters", self.character_count));
        }

        if self.term_count > 0 {
            parts.push(format!("{} terms", self.term_count));
        }

        if self.scene_count > 0 {
            parts.push(format!("{} scenes", self.scene_count));
        }

        if self.summary.is_some() {
            parts.push("summary generated".to_string());
        }

        if let Some(ref report) = self.preflight_report {
            parts.push(format!("preflight: {}", report.summary()));
        }

        if parts.is_empty() {
            "no analysis data".to_string()
        } else {
            parts.join(", ")
        }
    }
}

/// Analysis pass for preprocessing documents before translation.
pub struct AnalysisPass {
    config: AnalysisConfig,
    glossary_extractor: GlossaryExtractor,
    scene_detector: SceneDetector,
    summarizer: HistorySummarizer,
}

impl AnalysisPass {
    /// Create a new analysis pass with the given configuration.
    pub fn new(config: AnalysisConfig) -> Self {
        let glossary_extractor = GlossaryExtractor::new(config.extraction_config.clone());
        let scene_detector = SceneDetector::new(config.scene_config.clone());
        let summarizer = HistorySummarizer::new(config.summarization_config.clone());

        Self {
            config,
            glossary_extractor,
            scene_detector,
            summarizer,
        }
    }

    /// Create an analysis pass with default configuration.
    pub fn with_defaults() -> Self {
        Self::new(AnalysisConfig::default())
    }

    /// Analyze a document and return the analysis result.
    ///
    /// This method does not modify the document; use `analyze_and_update`
    /// to apply the analysis results to the document.
    pub fn analyze(&self, doc: &SubtitleDocument) -> AnalysisResult {
        let mut result = AnalysisResult::empty();

        // Extract glossary (characters and terms)
        if self.config.extract_glossary {
            result.glossary = self.glossary_extractor.extract(&doc.entries);
            result.character_count = result.glossary.character_names.len();
            result.term_count = result.glossary.terms.len() + result.glossary.technical_terms.len();
        }

        // Detect scenes
        if self.config.detect_scenes {
            result.scenes = self.scene_detector.detect_scenes(&doc.entries);
            result.scene_count = result.scenes.len();
        }

        // Generate summary
        if self.config.generate_summary && !doc.entries.is_empty() {
            let summary = self.summarizer.summarize_extractive(&doc.entries);
            if !summary.text.is_empty() {
                result.summary = Some(summary.text);
            }
        }

        // Run glossary preflight check
        if self.config.run_preflight && !result.glossary.is_empty() {
            let checker = GlossaryPreflightChecker::new(&result.glossary);
            result.preflight_report = Some(checker.check_entries(&doc.entries));
        }

        result
    }

    /// Analyze a document and apply the results to it.
    ///
    /// This modifies the document in place, updating its glossary, scenes, and summary.
    pub fn analyze_and_update(&self, doc: &mut SubtitleDocument) -> AnalysisResult {
        let result = self.analyze(doc);

        // Apply glossary
        if !result.glossary.is_empty() {
            doc.glossary.merge(&result.glossary);
        }

        // Apply scenes
        if !result.scenes.is_empty() {
            doc.scenes = result.scenes.clone();

            // Update entries with scene IDs
            for scene in &doc.scenes {
                for entry in &mut doc.entries {
                    if entry.id >= scene.start_entry_id && entry.id <= scene.end_entry_id {
                        entry.scene_id = Some(scene.id);
                    }
                }
            }
        }

        // Apply summary
        if result.summary.is_some() {
            doc.context_summary = result.summary.clone();
        }

        result
    }

    /// Extract glossary only (without full analysis).
    pub fn extract_glossary(&self, doc: &SubtitleDocument) -> Glossary {
        self.glossary_extractor.extract(&doc.entries)
    }

    /// Detect scenes only (without full analysis).
    pub fn detect_scenes(&self, doc: &SubtitleDocument) -> Vec<Scene> {
        self.scene_detector.detect_scenes(&doc.entries)
    }

    /// Generate summary only (without full analysis).
    pub fn generate_summary(&self, doc: &SubtitleDocument) -> Option<String> {
        if doc.entries.is_empty() {
            return None;
        }

        let summary = self.summarizer.summarize_extractive(&doc.entries);
        if summary.text.is_empty() {
            None
        } else {
            Some(summary.text)
        }
    }
}

impl Default for AnalysisPass {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::subtitle_processor::SubtitleEntry;

    fn create_test_document(texts: &[&str]) -> SubtitleDocument {
        let entries: Vec<SubtitleEntry> = texts
            .iter()
            .enumerate()
            .map(|(i, text)| {
                SubtitleEntry::new(
                    i + 1,
                    (i as u64) * 2000,
                    (i as u64 + 1) * 2000,
                    text.to_string(),
                )
            })
            .collect();

        SubtitleDocument::from_entries(entries, "en")
    }

    #[test]
    fn test_analysisPass_analyze_shouldExtractCharacters() {
        let doc = create_test_document(&[
            "John entered the room.",
            "Mary was waiting there.",
            "John looked at Mary.",
            "They started talking.",
        ]);

        let pass = AnalysisPass::with_defaults();
        let result = pass.analyze(&doc);

        assert!(result.character_count > 0);
        assert!(result.glossary.character_names.contains("John"));
        assert!(result.glossary.character_names.contains("Mary"));
    }

    #[test]
    fn test_analysisPass_analyze_shouldDetectScenes() {
        // Create entries with a timing gap (simulating scene change)
        let entries = vec![
            SubtitleEntry::new(1, 0, 2000, "Scene one starts.".to_string()),
            SubtitleEntry::new(2, 2000, 4000, "Scene one continues.".to_string()),
            // 10 second gap indicates scene change
            SubtitleEntry::new(3, 14000, 16000, "Scene two begins.".to_string()),
            SubtitleEntry::new(4, 16000, 18000, "Scene two continues.".to_string()),
        ];

        let doc = SubtitleDocument::from_entries(entries, "en");

        let pass = AnalysisPass::new(AnalysisConfig {
            scene_config: SceneDetectionConfig {
                min_gap_ms: 5000, // 5 second gap threshold
                ..Default::default()
            },
            ..Default::default()
        });

        let result = pass.analyze(&doc);

        // Should detect scene boundary at the 10-second gap
        assert!(result.scene_count >= 1);
    }

    #[test]
    fn test_analysisPass_analyzeAndUpdate_shouldModifyDocument() {
        let mut doc = create_test_document(&[
            "Alice walked in.",
            "Bob was sitting.",
            "Alice greeted Bob.",
        ]);

        let pass = AnalysisPass::with_defaults();
        let result = pass.analyze_and_update(&mut doc);

        // Document should be updated
        assert_eq!(doc.glossary.character_names.len(), result.character_count);
    }

    #[test]
    fn test_analysisPass_minimalConfig_shouldBeEfficient() {
        let doc = create_test_document(&["Simple test.", "Another line."]);

        let pass = AnalysisPass::new(AnalysisConfig::minimal());
        let result = pass.analyze(&doc);

        // Minimal config should skip scene detection and summary
        assert_eq!(result.scene_count, 0);
        assert!(result.summary.is_none());
    }

    #[test]
    fn test_analysisResult_description_shouldSummarize() {
        let mut result = AnalysisResult::empty();
        result.character_count = 3;
        result.term_count = 5;
        result.scene_count = 2;

        let desc = result.description();

        assert!(desc.contains("3 characters"));
        assert!(desc.contains("5 terms"));
        assert!(desc.contains("2 scenes"));
    }

    #[test]
    fn test_analysisResult_empty_shouldHaveNoData() {
        let result = AnalysisResult::empty();

        assert!(!result.has_data());
        assert_eq!(result.description(), "no analysis data");
    }
}
