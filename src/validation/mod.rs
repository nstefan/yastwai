/*!
 * Validation module for translation quality assurance.
 *
 * This module provides comprehensive validation for translated subtitles:
 * - Marker validation (<<ENTRY_X>> markers in batch translations)
 * - Timecode validation (timing integrity)
 * - Format preservation validation (tags, styles)
 * - Length validation (reasonable translation length ratios)
 *
 * # Architecture
 *
 * - `markers`: Validates entry markers in batch responses
 * - `timecodes`: Validates timing data integrity
 * - `formatting`: Validates format tag preservation
 * - `length`: Validates translation length ratios
 * - `service`: Orchestrates all validators
 */

pub mod markers;
pub mod timecodes;
pub mod formatting;
pub mod length;
pub mod service;

// Re-export main types
pub use service::{ValidationService, ValidationConfig, ValidationResult, ValidationReport};
pub use markers::MarkerValidator;
pub use timecodes::TimecodeValidator;
pub use formatting::FormatValidator;
pub use length::LengthValidator;
