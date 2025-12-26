/*!
 * Context management for subtitle translation.
 *
 * This module provides context-aware translation support:
 * - Sliding window context for maintaining narrative flow
 * - Scene detection based on timing gaps
 * - Glossary extraction for terminology consistency
 * - History summarization for long documents
 */

pub mod glossary;
pub mod scenes;
pub mod summary;
pub mod window;

// Re-export main types
pub use glossary::{ConsistencyIssue, ExtractionConfig, GlossaryEnforcer, GlossaryExtractor, GlossaryExtractionExt};
pub use scenes::{SceneDetectionConfig, SceneDetector, SceneDetectionExt};
pub use summary::{HistorySummarizer, HistorySummary, SummarizationConfig, SummarizationExt};
pub use window::{ContextWindow, ContextWindowConfig, ContextWindowExt, WindowEntry};

