/*!
 * Database entity models and DTOs.
 *
 * These structures map directly to database tables and provide
 * type-safe access to persisted data.
 */

use serde::{Deserialize, Serialize};
use std::fmt;

/// Session status enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    /// Session is actively being processed
    InProgress,
    /// Session was interrupted and can be resumed
    Paused,
    /// All entries translated and validated
    Completed,
    /// Unrecoverable error occurred
    Failed,
}

impl fmt::Display for SessionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SessionStatus::InProgress => write!(f, "in_progress"),
            SessionStatus::Paused => write!(f, "paused"),
            SessionStatus::Completed => write!(f, "completed"),
            SessionStatus::Failed => write!(f, "failed"),
        }
    }
}

impl std::str::FromStr for SessionStatus {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "in_progress" => Ok(SessionStatus::InProgress),
            "paused" => Ok(SessionStatus::Paused),
            "completed" => Ok(SessionStatus::Completed),
            "failed" => Ok(SessionStatus::Failed),
            _ => Err(anyhow::anyhow!("Invalid session status: {}", s)),
        }
    }
}

/// Translation status for individual entries
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TranslationStatus {
    /// Entry awaiting translation
    Pending,
    /// Entry has been translated
    Translated,
    /// Entry translated and passed validation
    Validated,
    /// Translation failed
    Failed,
    /// Entry marked for retry
    Retry,
}

impl fmt::Display for TranslationStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TranslationStatus::Pending => write!(f, "pending"),
            TranslationStatus::Translated => write!(f, "translated"),
            TranslationStatus::Validated => write!(f, "validated"),
            TranslationStatus::Failed => write!(f, "failed"),
            TranslationStatus::Retry => write!(f, "retry"),
        }
    }
}

impl std::str::FromStr for TranslationStatus {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pending" => Ok(TranslationStatus::Pending),
            "translated" => Ok(TranslationStatus::Translated),
            "validated" => Ok(TranslationStatus::Validated),
            "failed" => Ok(TranslationStatus::Failed),
            "retry" => Ok(TranslationStatus::Retry),
            _ => Err(anyhow::anyhow!("Invalid translation status: {}", s)),
        }
    }
}

/// Validation type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationType {
    /// Marker validation (<<ENTRY_X>>)
    MarkerCheck,
    /// Timecode validation
    TimecodeCheck,
    /// Format preservation validation
    FormatCheck,
    /// Length ratio validation
    LengthCheck,
}

impl fmt::Display for ValidationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationType::MarkerCheck => write!(f, "marker_check"),
            ValidationType::TimecodeCheck => write!(f, "timecode_check"),
            ValidationType::FormatCheck => write!(f, "format_check"),
            ValidationType::LengthCheck => write!(f, "length_check"),
        }
    }
}

impl std::str::FromStr for ValidationType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "marker_check" => Ok(ValidationType::MarkerCheck),
            "timecode_check" => Ok(ValidationType::TimecodeCheck),
            "format_check" => Ok(ValidationType::FormatCheck),
            "length_check" => Ok(ValidationType::LengthCheck),
            _ => Err(anyhow::anyhow!("Invalid validation type: {}", s)),
        }
    }
}

/// Validation severity enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationSeverity {
    /// Warning - translation usable but may have issues
    Warning,
    /// Error - translation should be retried
    Error,
}

impl fmt::Display for ValidationSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationSeverity::Warning => write!(f, "warning"),
            ValidationSeverity::Error => write!(f, "error"),
        }
    }
}

impl std::str::FromStr for ValidationSeverity {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "warning" => Ok(ValidationSeverity::Warning),
            "error" => Ok(ValidationSeverity::Error),
            _ => Err(anyhow::anyhow!("Invalid validation severity: {}", s)),
        }
    }
}

/// Translation session record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRecord {
    /// Unique session identifier (UUID)
    pub id: String,
    /// Path to the source file
    pub source_file_path: String,
    /// SHA256 hash of the source file for change detection
    pub source_file_hash: String,
    /// Source language code
    pub source_language: String,
    /// Target language code
    pub target_language: String,
    /// Translation provider used
    pub provider: String,
    /// Model used for translation
    pub model: String,
    /// Total number of subtitle entries
    pub total_entries: i64,
    /// Number of completed entries
    pub completed_entries: i64,
    /// Current session status
    pub status: SessionStatus,
    /// Creation timestamp (ISO 8601)
    pub created_at: String,
    /// Last update timestamp (ISO 8601)
    pub updated_at: String,
    /// Completion timestamp (ISO 8601), if completed
    pub completed_at: Option<String>,
}

impl SessionRecord {
    /// Create a new session record
    pub fn new(
        id: String,
        source_file_path: String,
        source_file_hash: String,
        source_language: String,
        target_language: String,
        provider: String,
        model: String,
        total_entries: i64,
    ) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id,
            source_file_path,
            source_file_hash,
            source_language,
            target_language,
            provider,
            model,
            total_entries,
            completed_entries: 0,
            status: SessionStatus::InProgress,
            created_at: now.clone(),
            updated_at: now,
            completed_at: None,
        }
    }

    /// Check if session is resumable
    pub fn is_resumable(&self) -> bool {
        matches!(self.status, SessionStatus::InProgress | SessionStatus::Paused)
    }

    /// Calculate completion percentage
    pub fn completion_percentage(&self) -> f64 {
        if self.total_entries == 0 {
            return 0.0;
        }
        (self.completed_entries as f64 / self.total_entries as f64) * 100.0
    }
}

/// Source subtitle entry record (immutable reference)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceEntryRecord {
    /// Database ID
    pub id: i64,
    /// Session ID this entry belongs to
    pub session_id: String,
    /// Sequence number in the subtitle file
    pub seq_num: i64,
    /// Start time in milliseconds
    pub start_time_ms: i64,
    /// End time in milliseconds
    pub end_time_ms: i64,
    /// Original source text
    pub source_text: String,
}

impl SourceEntryRecord {
    /// Create a new source entry record (without database ID)
    pub fn new(
        session_id: String,
        seq_num: i64,
        start_time_ms: i64,
        end_time_ms: i64,
        source_text: String,
    ) -> Self {
        Self {
            id: 0, // Will be assigned by database
            session_id,
            seq_num,
            start_time_ms,
            end_time_ms,
            source_text,
        }
    }
}

/// Translated entry record with quality metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslatedEntryRecord {
    /// Database ID
    pub id: i64,
    /// Reference to source entry
    pub source_entry_id: i64,
    /// Translated text
    pub translated_text: String,
    /// Translation status
    pub translation_status: TranslationStatus,
    /// Quality score (0.0 to 1.0)
    pub quality_score: Option<f64>,
    /// JSON array of validation errors
    pub validation_errors: Option<String>,
    /// Number of translation attempts
    pub attempt_count: i64,
    /// Creation timestamp
    pub created_at: String,
    /// Last update timestamp
    pub updated_at: String,
}

impl TranslatedEntryRecord {
    /// Create a new translated entry record
    pub fn new(source_entry_id: i64, translated_text: String) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: 0, // Will be assigned by database
            source_entry_id,
            translated_text,
            translation_status: TranslationStatus::Translated,
            quality_score: None,
            validation_errors: None,
            attempt_count: 1,
            created_at: now.clone(),
            updated_at: now,
        }
    }
}

/// Translation cache record for cross-session deduplication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheRecord {
    /// Database ID
    pub id: i64,
    /// SHA256 hash of source text
    pub source_text_hash: String,
    /// Original source text
    pub source_text: String,
    /// Source language code
    pub source_language: String,
    /// Target language code
    pub target_language: String,
    /// Translated text
    pub translated_text: String,
    /// Provider used for translation
    pub provider: String,
    /// Model used for translation
    pub model: String,
    /// Creation timestamp
    pub created_at: String,
    /// Number of cache hits
    pub hit_count: i64,
}

impl CacheRecord {
    /// Create a new cache record
    pub fn new(
        source_text_hash: String,
        source_text: String,
        source_language: String,
        target_language: String,
        translated_text: String,
        provider: String,
        model: String,
    ) -> Self {
        Self {
            id: 0, // Will be assigned by database
            source_text_hash,
            source_text,
            source_language,
            target_language,
            translated_text,
            provider,
            model,
            created_at: chrono::Utc::now().to_rfc3339(),
            hit_count: 1,
        }
    }
}

/// Validation result record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResultRecord {
    /// Database ID
    pub id: i64,
    /// Reference to translated entry
    pub translated_entry_id: i64,
    /// Type of validation performed
    pub validation_type: ValidationType,
    /// Whether validation passed
    pub passed: bool,
    /// Severity of the issue (if failed)
    pub severity: Option<ValidationSeverity>,
    /// Detailed message
    pub message: Option<String>,
    /// Creation timestamp
    pub created_at: String,
}

impl ValidationResultRecord {
    /// Create a new validation result record
    pub fn new(
        translated_entry_id: i64,
        validation_type: ValidationType,
        passed: bool,
        severity: Option<ValidationSeverity>,
        message: Option<String>,
    ) -> Self {
        Self {
            id: 0, // Will be assigned by database
            translated_entry_id,
            validation_type,
            passed,
            severity,
            message,
            created_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Create a passing validation result
    pub fn passed(translated_entry_id: i64, validation_type: ValidationType) -> Self {
        Self::new(translated_entry_id, validation_type, true, None, None)
    }

    /// Create a failing validation result with warning severity
    pub fn warning(
        translated_entry_id: i64,
        validation_type: ValidationType,
        message: String,
    ) -> Self {
        Self::new(
            translated_entry_id,
            validation_type,
            false,
            Some(ValidationSeverity::Warning),
            Some(message),
        )
    }

    /// Create a failing validation result with error severity
    pub fn error(
        translated_entry_id: i64,
        validation_type: ValidationType,
        message: String,
    ) -> Self {
        Self::new(
            translated_entry_id,
            validation_type,
            false,
            Some(ValidationSeverity::Error),
            Some(message),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sessionStatus_display_shouldReturnSnakeCase() {
        assert_eq!(SessionStatus::InProgress.to_string(), "in_progress");
        assert_eq!(SessionStatus::Paused.to_string(), "paused");
        assert_eq!(SessionStatus::Completed.to_string(), "completed");
        assert_eq!(SessionStatus::Failed.to_string(), "failed");
    }

    #[test]
    fn test_sessionStatus_fromStr_shouldParseValidStrings() {
        assert_eq!(
            "in_progress".parse::<SessionStatus>().unwrap(),
            SessionStatus::InProgress
        );
        assert_eq!(
            "paused".parse::<SessionStatus>().unwrap(),
            SessionStatus::Paused
        );
    }

    #[test]
    fn test_sessionRecord_isResumable_shouldReturnTrueForInProgressOrPaused() {
        let mut session = SessionRecord::new(
            "test-id".to_string(),
            "/path/to/file.mkv".to_string(),
            "hash123".to_string(),
            "en".to_string(),
            "fr".to_string(),
            "ollama".to_string(),
            "llama2".to_string(),
            100,
        );

        assert!(session.is_resumable());

        session.status = SessionStatus::Paused;
        assert!(session.is_resumable());

        session.status = SessionStatus::Completed;
        assert!(!session.is_resumable());

        session.status = SessionStatus::Failed;
        assert!(!session.is_resumable());
    }

    #[test]
    fn test_sessionRecord_completionPercentage_shouldCalculateCorrectly() {
        let mut session = SessionRecord::new(
            "test-id".to_string(),
            "/path/to/file.mkv".to_string(),
            "hash123".to_string(),
            "en".to_string(),
            "fr".to_string(),
            "ollama".to_string(),
            "llama2".to_string(),
            100,
        );

        assert_eq!(session.completion_percentage(), 0.0);

        session.completed_entries = 50;
        assert_eq!(session.completion_percentage(), 50.0);

        session.completed_entries = 100;
        assert_eq!(session.completion_percentage(), 100.0);
    }

    #[test]
    fn test_translationStatus_display_shouldReturnCorrectString() {
        assert_eq!(TranslationStatus::Pending.to_string(), "pending");
        assert_eq!(TranslationStatus::Translated.to_string(), "translated");
        assert_eq!(TranslationStatus::Validated.to_string(), "validated");
    }

    #[test]
    fn test_validationResultRecord_passed_shouldCreatePassingResult() {
        let result = ValidationResultRecord::passed(1, ValidationType::MarkerCheck);
        assert!(result.passed);
        assert!(result.severity.is_none());
        assert!(result.message.is_none());
    }

    #[test]
    fn test_validationResultRecord_error_shouldCreateErrorResult() {
        let result = ValidationResultRecord::error(
            1,
            ValidationType::MarkerCheck,
            "Missing marker".to_string(),
        );
        assert!(!result.passed);
        assert_eq!(result.severity, Some(ValidationSeverity::Error));
        assert_eq!(result.message, Some("Missing marker".to_string()));
    }
}
