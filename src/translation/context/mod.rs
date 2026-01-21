/*!
 * Context management for subtitle translation.
 *
 * This module provides context-aware translation support:
 * - Sliding window context for maintaining narrative flow
 * - Scene detection based on timing gaps
 * - Glossary extraction for terminology consistency
 * - History summarization for long documents
 */

pub mod dynamic;
pub mod fuzzy;
pub mod glossary;
pub mod scenes;
pub mod speakers;
pub mod summary;
pub mod window;

// Re-export main types
pub use dynamic::{DynamicWindowConfig, DynamicWindowSizer};
pub use fuzzy::FuzzyMatcher;
pub use glossary::{ConsistencyIssue, ExtractionConfig, GlossaryEnforcer, GlossaryExtractor, GlossaryExtractionExt, GlossaryPreflightChecker, PreflightReport};
pub use scenes::{SceneDetectionConfig, SceneDetector, SceneDetectionExt};
pub use speakers::{DetectedSpeaker, SpeakerConfig, SpeakerStats, SpeakerTracker};
pub use summary::{HistorySummarizer, HistorySummary, SummarizationConfig, SummarizationExt};
pub use window::{ContextWindow, ContextWindowConfig, ContextWindowExt, WindowEntry};

