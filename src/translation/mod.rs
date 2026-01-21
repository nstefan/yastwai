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

// These re-exports are part of the public API for external consumers
#![allow(unused_imports)]

// Re-export main types for easier usage
pub use self::batch::{AdaptiveBatchSizer, BatchTranslator};
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
    ConsistencyIssue, ContextWindow, ContextWindowConfig, ContextWindowExt, DetectedSpeaker,
    DynamicWindowConfig, DynamicWindowSizer, ExtractionConfig, FuzzyMatcher, GlossaryEnforcer,
    GlossaryExtractor, GlossaryExtractionExt, GlossaryPreflightChecker, HistorySummarizer,
    HistorySummary, PreflightReport, SceneDetectionConfig, SceneDetectionExt, SceneDetector,
    SpeakerConfig, SpeakerStats, SpeakerTracker, SummarizationConfig, SummarizationExt, WindowEntry,
};

// Re-export pipeline types
pub use self::pipeline::{
    AnalysisPass, AnalysisResult, FailureReason, PipelineAdapter, PipelineConfig, PipelineMode,
    PipelinePhase, PipelineProgress, RepairAction, RepairResult, TranslationPass,
    TranslationPassConfig, TranslationPipeline, ValidationIssue, ValidationPass, ValidationReport,
};

// Re-export quality types
pub use self::quality::{
    ConsistencyChecker, ConsistencyConfig, ConsistencyReport, ErrorRecovery, LanguagePairThresholds,
    QualityMetrics, QualityScore, QualityThresholds, RecoveryAction, RecoveryStrategy, RepairEngine,
    RepairStrategy, SemanticIssue, SemanticValidationConfig, SemanticValidationResult,
    SemanticValidator, SmartRepair, StyleIssue, TranslationError, TranslationErrorKind,
};

// Re-export concurrency types
pub use self::concurrency::ProviderProfile;

// Re-export speculative batching types
pub use self::speculative::SpeculativeBatcher;

// Submodules
pub mod batch;
pub mod cache;
pub mod concurrency;
pub mod context;
pub mod core;
pub mod document;
pub mod formatting;
pub mod pipeline;
pub mod prompts;
pub mod quality;
pub mod speculative;
