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
pub use glossary::{ConsistencyIssue, ExtractionConfig, GlossaryEnforcer, GlossaryExtractor};
pub use scenes::{SceneDetectionConfig, SceneDetector};
pub use summary::{HistorySummarizer, HistorySummary, SummarizationConfig};
pub use window::{ContextWindow, ContextWindowConfig, ContextWindowExt, WindowEntry};

// Re-export extension traits
pub use glossary::GlossaryExtractionExt;
pub use scenes::SceneDetectionExt;
pub use summary::SummarizationExt;

