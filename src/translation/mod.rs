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
 * - `pipeline`: Multi-pass translation pipeline (analysis, translation, validation)
 * - `quality`: Quality assurance (metrics, consistency, repair, error handling)
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

// Re-export pipeline types
pub use self::pipeline::{
    AnalysisPass, AnalysisResult, PipelineConfig, PipelineProgress, RepairAction, RepairResult,
    TranslationPass, TranslationPassConfig, TranslationPipeline, ValidationIssue, ValidationPass,
    ValidationReport,
};

// Re-export quality types
pub use self::quality::{
    ConsistencyChecker, ConsistencyConfig, ConsistencyReport, ErrorRecovery, QualityMetrics,
    QualityScore, QualityThresholds, RecoveryAction, RecoveryStrategy, RepairEngine, RepairStrategy,
    SmartRepair, StyleIssue, TranslationError, TranslationErrorKind,
};

// Submodules
pub mod batch;
pub mod cache;
pub mod context;
pub mod core;
pub mod document;
pub mod formatting;
pub mod pipeline;
pub mod prompts;
pub mod quality; 