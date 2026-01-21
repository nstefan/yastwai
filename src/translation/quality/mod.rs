/*!
 * Quality assurance module for translation reliability.
 *
 * This module provides comprehensive quality assurance for translations:
 * - **Metrics**: Quantitative quality scoring and measurement
 * - **Consistency**: Style, terminology, and tone consistency checking
 * - **Repair**: Intelligent auto-repair strategies for common issues
 * - **Errors**: Comprehensive error handling with recovery strategies
 *
 * The quality module integrates with the validation module and pipeline
 * to provide end-to-end quality assurance.
 */

pub mod consistency;
pub mod errors;
pub mod language_pairs;
pub mod metrics;
pub mod repair;
pub mod semantic;

// Re-export main types
pub use consistency::{ConsistencyChecker, ConsistencyConfig, ConsistencyReport, StyleIssue};
pub use errors::{ErrorRecovery, RecoveryAction, RecoveryStrategy, TranslationError, TranslationErrorKind};
pub use language_pairs::LanguagePairThresholds;
pub use metrics::{QualityMetrics, QualityScore, QualityThresholds};
pub use repair::{RepairEngine, RepairStrategy, SmartRepair};
pub use semantic::{SemanticIssue, SemanticValidationConfig, SemanticValidationResult, SemanticValidator};
