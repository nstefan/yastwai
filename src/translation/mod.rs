/*!
 * Translation service for subtitle translation using AI providers.
 *
 * This module contains the core functionality for translating subtitles
 * using various AI providers.
 */

// Re-export main types
pub use self::batch::BatchTranslator;
pub use self::core::TranslationService;
pub use self::pipeline::{PipelineAdapter, PipelineMode};

// Public modules
pub mod batch;
pub mod cache;
pub mod core;
pub mod formatting;
pub mod pipeline;

// Internal modules
pub(crate) mod concurrency;
pub(crate) mod context;
pub(crate) mod document;
pub(crate) mod prompts;
pub(crate) mod quality;
pub(crate) mod speculative;
