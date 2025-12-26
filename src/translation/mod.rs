/*!
 * Translation service for subtitle translation using AI providers.
 *
 * This module contains the core functionality for translating subtitles
 * using various AI providers. It is split into several submodules:
 *
 * - `core`: Core translation functionality and service definition
 * - `batch`: Batch processing of translations
 * - `cache`: Caching mechanisms for translations
 * - `formatting`: Format preservation and processing
 * - `document`: Document model for structured subtitle representation
 * - `prompts`: Prompt templates and builders for translation
 * - `context`: Context management (sliding window, scenes, glossary)
 */

// Re-export main types for easier usage
pub use self::batch::BatchTranslator;
pub use self::core::TranslationService;

// Re-export document model types
pub use self::document::{
    DocumentEntry, DocumentMetadata, FormattingTag, Glossary, GlossaryTerm, Scene,
    SubtitleDocument, Timecode,
};

// Re-export prompt types
pub use self::prompts::{PromptTemplate, TranslationPromptBuilder};

// Re-export context types
pub use self::context::{
    ConsistencyIssue, ContextWindow, ContextWindowConfig, ContextWindowExt, ExtractionConfig,
    GlossaryEnforcer, GlossaryExtractor, GlossaryExtractionExt, HistorySummarizer, HistorySummary,
    SceneDetectionConfig, SceneDetectionExt, SceneDetector, SummarizationConfig, SummarizationExt,
    WindowEntry,
};

// Submodules
pub mod batch;
pub mod cache;
pub mod context;
pub mod core;
pub mod document;
pub mod formatting;
pub mod prompts; 