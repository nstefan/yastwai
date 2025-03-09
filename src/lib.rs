/*!
 * # YASTwAI - Yet Another Subtitle Translator with AI
 * 
 * A Rust library for automatic translation of video subtitles using AI.
 * 
 * ## Features
 * 
 * - Extract subtitles from video files
 * - Translate subtitles using various AI providers:
 *   - Ollama (local LLM)
 *   - OpenAI API
 *   - Anthropic API
 * - Preserve subtitle formatting and timing
 * - Configurable translation parameters
 * - Batch processing for efficient translation
 * - ISO 639-1 and ISO 639-2 language code support
 * 
 * ## Architecture
 * 
 * The library is organized in these main modules:
 * - `app_config`: Configuration management
 * - `subtitle_processor`: Subtitle file handling and processing
 * - `translation_service`: AI-powered translation services
 * - `file_utils`: File system operations
 * - `app_controller`: Main application controller
 * - `language_utils`: ISO language code utilities
 * - `providers`: Client implementations for various LLM providers:
 *   - `providers::ollama`: Ollama API client
 *   - `providers::openai`: OpenAI API client
 *   - `providers::anthropic`: Anthropic API client
 * 
 * ## License
 * 
 * This project is licensed under the MIT License
 */

// Public modules
pub mod app_config;
pub mod file_utils;
pub mod subtitle_processor;
pub mod translation_service;
pub mod app_controller;
pub mod language_utils;
pub mod providers;

// Re-export main types for easier usage
pub use app_config::Config;
pub use subtitle_processor::{SubtitleCollection, SubtitleEntry};
pub use translation_service::TranslationService;
pub use language_utils::{language_codes_match, normalize_to_part2t, get_language_name, validate_language_code}; 