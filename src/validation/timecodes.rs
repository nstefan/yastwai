/*!
 * Timecode validation for subtitle entries.
 *
 * This module validates that timecodes are:
 * - Properly formatted and parseable
 * - Logically consistent (start < end)
 * - Non-overlapping between entries
 * - Within reasonable reading speed limits
 */

use log::debug;

use crate::subtitle_processor::SubtitleEntry;

/// Maximum characters per second for readable subtitles
/// Research suggests 15-25 CPS is readable
const DEFAULT_MAX_CPS: f64 = 25.0;

/// Minimum duration for a subtitle in milliseconds
const MIN_SUBTITLE_DURATION_MS: u64 = 500;

/// Maximum duration for a single subtitle in milliseconds (30 seconds)
const MAX_SUBTITLE_DURATION_MS: u64 = 30_000;

/// Result of timecode validation for a single entry
#[derive(Debug, Clone)]
pub struct TimecodeEntryResult {
    /// Sequence number of the entry
    pub seq_num: usize,
    /// Whether the entry passed validation
    pub passed: bool,
    /// Issues found
    pub issues: Vec<TimecodeIssue>,
}

impl TimecodeEntryResult {
    /// Create a passing result
    pub fn passed(seq_num: usize) -> Self {
        Self {
            seq_num,
            passed: true,
            issues: vec![],
        }
    }

    /// Create a failing result
    pub fn failed(seq_num: usize, issues: Vec<TimecodeIssue>) -> Self {
        Self {
            seq_num,
            passed: false,
            issues,
        }
    }
}

/// Types of timecode issues
#[derive(Debug, Clone, PartialEq)]
pub enum TimecodeIssue {
    /// Start time is after end time
    InvalidTimeRange {
        start_ms: u64,
        end_ms: u64,
    },
    /// Duration is too short
    DurationTooShort {
        duration_ms: u64,
        min_duration_ms: u64,
    },
    /// Duration is too long
    DurationTooLong {
        duration_ms: u64,
        max_duration_ms: u64,
    },
    /// Reading speed exceeds limit
    ReadingSpeedTooHigh {
        cps: f64,
        max_cps: f64,
    },
    /// Overlaps with another entry
    OverlapsWithEntry {
        other_seq_num: usize,
        overlap_ms: u64,
    },
    /// Gap too large between entries
    LargeGap {
        prev_seq_num: usize,
        gap_ms: u64,
    },
}

impl std::fmt::Display for TimecodeIssue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimecodeIssue::InvalidTimeRange { start_ms, end_ms } => {
                write!(f, "Invalid time range: start {}ms > end {}ms", start_ms, end_ms)
            }
            TimecodeIssue::DurationTooShort { duration_ms, min_duration_ms } => {
                write!(
                    f,
                    "Duration too short: {}ms (min: {}ms)",
                    duration_ms, min_duration_ms
                )
            }
            TimecodeIssue::DurationTooLong { duration_ms, max_duration_ms } => {
                write!(
                    f,
                    "Duration too long: {}ms (max: {}ms)",
                    duration_ms, max_duration_ms
                )
            }
            TimecodeIssue::ReadingSpeedTooHigh { cps, max_cps } => {
                write!(f, "Reading speed too high: {:.1} CPS (max: {:.1})", cps, max_cps)
            }
            TimecodeIssue::OverlapsWithEntry { other_seq_num, overlap_ms } => {
                write!(
                    f,
                    "Overlaps with entry {} by {}ms",
                    other_seq_num, overlap_ms
                )
            }
            TimecodeIssue::LargeGap { prev_seq_num, gap_ms } => {
                write!(
                    f,
                    "Large gap of {}ms after entry {}",
                    gap_ms, prev_seq_num
                )
            }
        }
    }
}

/// Result of validating all timecodes in a collection
#[derive(Debug, Clone)]
pub struct TimecodeValidationResult {
    /// Overall pass/fail status
    pub passed: bool,
    /// Results for each entry
    pub entry_results: Vec<TimecodeEntryResult>,
    /// Total number of issues
    pub total_issues: usize,
    /// Number of overlapping entries
    pub overlap_count: usize,
}

impl TimecodeValidationResult {
    /// Get all failed entries
    pub fn failed_entries(&self) -> Vec<&TimecodeEntryResult> {
        self.entry_results.iter().filter(|r| !r.passed).collect()
    }
}

/// Configuration for timecode validation
#[derive(Debug, Clone)]
pub struct TimecodeValidatorConfig {
    /// Maximum characters per second
    pub max_cps: f64,
    /// Minimum subtitle duration in ms
    pub min_duration_ms: u64,
    /// Maximum subtitle duration in ms
    pub max_duration_ms: u64,
    /// Whether to check for overlaps
    pub check_overlaps: bool,
    /// Maximum gap in ms before warning (0 = disable)
    pub max_gap_warning_ms: u64,
}

impl Default for TimecodeValidatorConfig {
    fn default() -> Self {
        Self {
            max_cps: DEFAULT_MAX_CPS,
            min_duration_ms: MIN_SUBTITLE_DURATION_MS,
            max_duration_ms: MAX_SUBTITLE_DURATION_MS,
            check_overlaps: true,
            max_gap_warning_ms: 0, // Disabled by default
        }
    }
}

/// Timecode validator for subtitle entries
pub struct TimecodeValidator {
    config: TimecodeValidatorConfig,
}

impl TimecodeValidator {
    /// Create a new validator with default configuration
    pub fn new() -> Self {
        Self {
            config: TimecodeValidatorConfig::default(),
        }
    }

    /// Create a new validator with custom configuration
    pub fn with_config(config: TimecodeValidatorConfig) -> Self {
        Self { config }
    }

    /// Validate a single subtitle entry
    pub fn validate_entry(&self, entry: &SubtitleEntry) -> TimecodeEntryResult {
        let mut issues = Vec::new();

        // Check time range validity
        if entry.end_time_ms <= entry.start_time_ms {
            issues.push(TimecodeIssue::InvalidTimeRange {
                start_ms: entry.start_time_ms,
                end_ms: entry.end_time_ms,
            });
            // Can't do further validation with invalid times
            return TimecodeEntryResult::failed(entry.seq_num, issues);
        }

        let duration_ms = entry.end_time_ms - entry.start_time_ms;

        // Check duration limits
        if duration_ms < self.config.min_duration_ms {
            issues.push(TimecodeIssue::DurationTooShort {
                duration_ms,
                min_duration_ms: self.config.min_duration_ms,
            });
        }

        if duration_ms > self.config.max_duration_ms {
            issues.push(TimecodeIssue::DurationTooLong {
                duration_ms,
                max_duration_ms: self.config.max_duration_ms,
            });
        }

        // Calculate reading speed (characters per second)
        let char_count = entry.text.chars().count() as f64;
        let duration_secs = duration_ms as f64 / 1000.0;
        let cps = if duration_secs > 0.0 {
            char_count / duration_secs
        } else {
            f64::INFINITY
        };

        if cps > self.config.max_cps {
            issues.push(TimecodeIssue::ReadingSpeedTooHigh {
                cps,
                max_cps: self.config.max_cps,
            });
        }

        if issues.is_empty() {
            TimecodeEntryResult::passed(entry.seq_num)
        } else {
            TimecodeEntryResult::failed(entry.seq_num, issues)
        }
    }

    /// Validate a collection of subtitle entries
    pub fn validate_collection(&self, entries: &[SubtitleEntry]) -> TimecodeValidationResult {
        if entries.is_empty() {
            return TimecodeValidationResult {
                passed: true,
                entry_results: vec![],
                total_issues: 0,
                overlap_count: 0,
            };
        }

        let mut entry_results: Vec<TimecodeEntryResult> = entries
            .iter()
            .map(|e| self.validate_entry(e))
            .collect();

        let mut overlap_count = 0;

        // Check for overlaps if enabled
        if self.config.check_overlaps && entries.len() > 1 {
            // Sort entries by start time for overlap detection
            let mut sorted_indices: Vec<usize> = (0..entries.len()).collect();
            sorted_indices.sort_by_key(|&i| entries[i].start_time_ms);

            for i in 0..sorted_indices.len() - 1 {
                let current_idx = sorted_indices[i];
                let next_idx = sorted_indices[i + 1];

                let current = &entries[current_idx];
                let next = &entries[next_idx];

                // Check for overlap
                if current.end_time_ms > next.start_time_ms {
                    let overlap_ms = current.end_time_ms - next.start_time_ms;
                    overlap_count += 1;

                    // Add issue to the later entry
                    if let Some(result) = entry_results.get_mut(next_idx) {
                        result.issues.push(TimecodeIssue::OverlapsWithEntry {
                            other_seq_num: current.seq_num,
                            overlap_ms,
                        });
                        result.passed = false;
                    }
                }

                // Check for large gaps if configured
                if self.config.max_gap_warning_ms > 0 && next.start_time_ms > current.end_time_ms {
                    let gap_ms = next.start_time_ms - current.end_time_ms;
                    if gap_ms > self.config.max_gap_warning_ms {
                        if let Some(result) = entry_results.get_mut(next_idx) {
                            result.issues.push(TimecodeIssue::LargeGap {
                                prev_seq_num: current.seq_num,
                                gap_ms,
                            });
                            // Large gaps are warnings, not failures
                        }
                    }
                }
            }
        }

        let total_issues: usize = entry_results.iter().map(|r| r.issues.len()).sum();
        let passed = entry_results.iter().all(|r| r.passed);

        debug!(
            "Timecode validation: {} entries, {} issues, {} overlaps",
            entries.len(),
            total_issues,
            overlap_count
        );

        TimecodeValidationResult {
            passed,
            entry_results,
            total_issues,
            overlap_count,
        }
    }

    /// Calculate reading speed (characters per second) for an entry
    pub fn calculate_cps(entry: &SubtitleEntry) -> f64 {
        if entry.end_time_ms <= entry.start_time_ms {
            return f64::INFINITY;
        }

        let char_count = entry.text.chars().count() as f64;
        let duration_secs = (entry.end_time_ms - entry.start_time_ms) as f64 / 1000.0;

        if duration_secs > 0.0 {
            char_count / duration_secs
        } else {
            f64::INFINITY
        }
    }
}

impl Default for TimecodeValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_entry(seq: usize, start: u64, end: u64, text: &str) -> SubtitleEntry {
        SubtitleEntry::new(seq, start, end, text.to_string())
    }

    #[test]
    fn test_validateEntry_withValidEntry_shouldPass() {
        let validator = TimecodeValidator::new();
        let entry = create_entry(1, 0, 2000, "Hello World");

        let result = validator.validate_entry(&entry);

        assert!(result.passed);
        assert!(result.issues.is_empty());
    }

    #[test]
    fn test_validateEntry_withInvalidTimeRange_shouldFail() {
        let validator = TimecodeValidator::new();
        let entry = create_entry(1, 2000, 1000, "Hello"); // End before start

        let result = validator.validate_entry(&entry);

        assert!(!result.passed);
        assert!(matches!(
            result.issues[0],
            TimecodeIssue::InvalidTimeRange { .. }
        ));
    }

    #[test]
    fn test_validateEntry_withShortDuration_shouldFail() {
        let validator = TimecodeValidator::new();
        let entry = create_entry(1, 0, 200, "Hi"); // 200ms duration

        let result = validator.validate_entry(&entry);

        assert!(!result.passed);
        assert!(matches!(
            result.issues[0],
            TimecodeIssue::DurationTooShort { .. }
        ));
    }

    #[test]
    fn test_validateEntry_withLongDuration_shouldFail() {
        let validator = TimecodeValidator::new();
        let entry = create_entry(1, 0, 60_000, "Very long subtitle"); // 60 seconds

        let result = validator.validate_entry(&entry);

        assert!(!result.passed);
        assert!(matches!(
            result.issues[0],
            TimecodeIssue::DurationTooLong { .. }
        ));
    }

    #[test]
    fn test_validateEntry_withHighReadingSpeed_shouldFail() {
        let validator = TimecodeValidator::new();
        // 50 characters in 1 second = 50 CPS (way above 25 limit)
        let entry = create_entry(
            1,
            0,
            1000,
            "This is a very long subtitle text that has many chars",
        );

        let result = validator.validate_entry(&entry);

        assert!(!result.passed);
        assert!(matches!(
            result.issues[0],
            TimecodeIssue::ReadingSpeedTooHigh { .. }
        ));
    }

    #[test]
    fn test_validateCollection_withOverlaps_shouldDetect() {
        let validator = TimecodeValidator::new();
        let entries = vec![
            create_entry(1, 0, 2000, "First"),
            create_entry(2, 1500, 3000, "Second"), // Overlaps with first
        ];

        let result = validator.validate_collection(&entries);

        assert!(!result.passed);
        assert_eq!(result.overlap_count, 1);
    }

    #[test]
    fn test_validateCollection_withNoOverlaps_shouldPass() {
        let validator = TimecodeValidator::new();
        let entries = vec![
            create_entry(1, 0, 2000, "First"),
            create_entry(2, 2000, 4000, "Second"),
            create_entry(3, 4000, 6000, "Third"),
        ];

        let result = validator.validate_collection(&entries);

        assert!(result.passed);
        assert_eq!(result.overlap_count, 0);
    }

    #[test]
    fn test_calculateCps_shouldCalculateCorrectly() {
        let entry = create_entry(1, 0, 2000, "Hello World"); // 11 chars in 2 secs = 5.5 CPS

        let cps = TimecodeValidator::calculate_cps(&entry);

        assert!((cps - 5.5).abs() < 0.01);
    }

    #[test]
    fn test_customConfig_shouldBeRespected() {
        let config = TimecodeValidatorConfig {
            max_cps: 50.0, // More lenient
            min_duration_ms: 100,
            max_duration_ms: 60_000,
            ..Default::default()
        };
        let validator = TimecodeValidator::with_config(config);

        // This would fail with default config due to high CPS
        let entry = create_entry(
            1,
            0,
            1000,
            "This is a very long subtitle text that has many chars",
        );

        let result = validator.validate_entry(&entry);

        // With max_cps=50, this should pass (53 chars / 1 sec = 53 CPS > 50)
        // Actually still fails, let's use a shorter text
        let entry2 = create_entry(1, 0, 1000, "This is a shorter text with less chars");
        let result2 = validator.validate_entry(&entry2);

        assert!(result2.passed);
    }
}
