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
pub mod metrics;
pub mod repair;

// Re-export main types
pub use consistency::{ConsistencyChecker, ConsistencyConfig, ConsistencyReport, StyleIssue};
pub use errors::{ErrorRecovery, RecoveryAction, RecoveryStrategy, TranslationError, TranslationErrorKind};
pub use metrics::{QualityMetrics, QualityScore, QualityThresholds};
pub use repair::{RepairEngine, RepairStrategy, SmartRepair};
