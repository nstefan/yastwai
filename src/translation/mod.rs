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
 */

// Re-export main types for easier usage
pub use self::core::TranslationService;
// pub use self::core::TranslationOptions;
pub use self::batch::BatchTranslator;
// pub use self::cache::TranslationCache;
// pub use self::formatting::FormatPreserver;

// Submodules
pub mod core;
pub mod batch;
pub mod cache;
pub mod formatting; 