/*!
 * Prompt engineering for subtitle translation.
 *
 * This module provides:
 * - System prompt templates for different translation scenarios
 * - Dynamic prompt construction based on context
 * - Provider-specific prompt adaptations
 */

pub mod templates;

// Re-export main types
pub use templates::{PromptTemplate, TranslatedEntryContext, TranslationPromptBuilder};

