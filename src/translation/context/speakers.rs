/*!
 * Speaker tracking for dialogue translation.
 *
 * This module provides speaker detection and tracking for dialogue-heavy subtitles:
 * - Pattern-based speaker detection (e.g., "NAME: dialogue")
 * - Speaker continuity tracking across entries
 * - Character name extraction
 */

use std::collections::HashMap;
use std::sync::LazyLock;

use regex::Regex;

use crate::translation::document::DocumentEntry;

/// Pattern for detecting speaker labels at the start of text.
/// Matches patterns like "JOHN:", "Mary:", "DR. SMITH:", "[NARRATOR]:"
static SPEAKER_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(?:\[)?([A-Z][A-Za-z\s\.]+?)(?:\])?:\s*").expect("Invalid speaker regex")
});

/// Pattern for detecting stage directions/sound effects.
static SOUND_EFFECT_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\[.+\]$|^\(.+\)$").expect("Invalid sound effect regex")
});

/// Configuration for speaker tracking.
#[derive(Debug, Clone)]
pub struct SpeakerConfig {
    /// Minimum times a name must appear to be considered a speaker
    pub min_occurrences: usize,

    /// Whether to detect implicit speaker changes (no label)
    pub detect_implicit_changes: bool,

    /// Maximum gap (in entries) to maintain speaker continuity
    pub continuity_gap: usize,
}

impl Default for SpeakerConfig {
    fn default() -> Self {
        Self {
            min_occurrences: 2,
            detect_implicit_changes: false,
            continuity_gap: 3,
        }
    }
}

impl SpeakerConfig {
    /// Create a strict config that requires more evidence.
    pub fn strict() -> Self {
        Self {
            min_occurrences: 3,
            detect_implicit_changes: false,
            continuity_gap: 1,
        }
    }

    /// Create a lenient config that detects more speakers.
    pub fn lenient() -> Self {
        Self {
            min_occurrences: 1,
            detect_implicit_changes: true,
            continuity_gap: 5,
        }
    }
}

/// Detected speaker information.
#[derive(Debug, Clone)]
pub struct DetectedSpeaker {
    /// Speaker name/label
    pub name: String,

    /// Entry IDs where this speaker appears
    pub entry_ids: Vec<usize>,

    /// Number of times this speaker appears
    pub occurrence_count: usize,
}

/// Speaker tracking statistics.
#[derive(Debug, Clone, Default)]
pub struct SpeakerStats {
    /// Total entries analyzed
    pub entries_analyzed: usize,

    /// Entries with detected speakers
    pub entries_with_speakers: usize,

    /// Number of unique speakers detected
    pub unique_speakers: usize,

    /// Sound effects/stage directions found
    pub sound_effects_found: usize,
}

/// Speaker tracker for dialogue-aware translation.
#[derive(Debug)]
pub struct SpeakerTracker {
    config: SpeakerConfig,
}

impl SpeakerTracker {
    /// Create a new speaker tracker with the given configuration.
    pub fn new(config: SpeakerConfig) -> Self {
        Self { config }
    }

    /// Create with default configuration.
    pub fn with_defaults() -> Self {
        Self::new(SpeakerConfig::default())
    }

    /// Detect speakers in a collection of entries.
    ///
    /// This method analyzes entries and updates their speaker field
    /// based on detected patterns.
    pub fn detect_speakers(&self, entries: &mut [DocumentEntry]) -> SpeakerStats {
        let mut stats = SpeakerStats {
            entries_analyzed: entries.len(),
            ..Default::default()
        };

        // First pass: detect explicit speaker labels
        let mut speaker_counts: HashMap<String, Vec<usize>> = HashMap::new();

        for entry in entries.iter() {
            if is_sound_effect(&entry.original_text) {
                stats.sound_effects_found += 1;
                continue;
            }

            if let Some(speaker) = extract_speaker(&entry.original_text) {
                speaker_counts
                    .entry(speaker.clone())
                    .or_default()
                    .push(entry.id);
            }
        }

        // Filter to speakers that meet minimum occurrence threshold
        let valid_speakers: HashMap<String, Vec<usize>> = speaker_counts
            .into_iter()
            .filter(|(_, ids)| ids.len() >= self.config.min_occurrences)
            .collect();

        stats.unique_speakers = valid_speakers.len();

        // Second pass: assign speakers to entries
        for entry in entries.iter_mut() {
            if is_sound_effect(&entry.original_text) {
                continue;
            }

            if let Some(speaker) = extract_speaker(&entry.original_text) {
                if valid_speakers.contains_key(&speaker) {
                    entry.speaker = Some(speaker);
                    stats.entries_with_speakers += 1;
                }
            }
        }

        // Third pass (optional): propagate speakers to unlabeled entries
        if self.config.detect_implicit_changes {
            self.propagate_speakers(entries, &mut stats);
        }

        stats
    }

    /// Propagate speaker labels to nearby unlabeled entries.
    fn propagate_speakers(&self, entries: &mut [DocumentEntry], stats: &mut SpeakerStats) {
        let mut last_speaker: Option<String> = None;
        let mut gap_count = 0;

        for entry in entries.iter_mut() {
            if entry.speaker.is_some() {
                last_speaker = entry.speaker.clone();
                gap_count = 0;
            } else if last_speaker.is_some() && gap_count < self.config.continuity_gap {
                // Only propagate if entry looks like dialogue (no special markers)
                if !is_sound_effect(&entry.original_text) && looks_like_dialogue(&entry.original_text) {
                    entry.speaker = last_speaker.clone();
                    stats.entries_with_speakers += 1;
                }
                gap_count += 1;
            } else {
                // Reset continuity
                last_speaker = None;
                gap_count = 0;
            }
        }
    }

    /// Get a list of all detected speakers from entries.
    pub fn get_speakers(&self, entries: &[DocumentEntry]) -> Vec<DetectedSpeaker> {
        let mut speaker_map: HashMap<String, Vec<usize>> = HashMap::new();

        for entry in entries {
            if let Some(ref speaker) = entry.speaker {
                speaker_map
                    .entry(speaker.clone())
                    .or_default()
                    .push(entry.id);
            }
        }

        speaker_map
            .into_iter()
            .map(|(name, entry_ids)| DetectedSpeaker {
                name,
                occurrence_count: entry_ids.len(),
                entry_ids,
            })
            .collect()
    }

    /// Extract speaker names from entries (without modifying them).
    pub fn extract_speaker_names(&self, entries: &[DocumentEntry]) -> Vec<String> {
        let mut names: HashMap<String, usize> = HashMap::new();

        for entry in entries {
            if let Some(speaker) = extract_speaker(&entry.original_text) {
                *names.entry(speaker).or_insert(0) += 1;
            }
        }

        names
            .into_iter()
            .filter(|(_, count)| *count >= self.config.min_occurrences)
            .map(|(name, _)| name)
            .collect()
    }

    /// Get configuration.
    pub fn config(&self) -> &SpeakerConfig {
        &self.config
    }
}

impl Default for SpeakerTracker {
    fn default() -> Self {
        Self::with_defaults()
    }
}

/// Extract speaker name from text if present.
fn extract_speaker(text: &str) -> Option<String> {
    SPEAKER_PATTERN
        .captures(text)
        .and_then(|caps| caps.get(1).map(|m| m.as_str().trim().to_string()))
}

/// Check if text appears to be a sound effect or stage direction.
fn is_sound_effect(text: &str) -> bool {
    SOUND_EFFECT_PATTERN.is_match(text.trim())
}

/// Check if text looks like dialogue (not a sound effect or technical marker).
fn looks_like_dialogue(text: &str) -> bool {
    let trimmed = text.trim();

    // Not empty
    if trimmed.is_empty() {
        return false;
    }

    // Not a sound effect
    if is_sound_effect(trimmed) {
        return false;
    }

    // Not all caps technical text (like "SCENE 5" or "END")
    if trimmed.len() < 20 && trimmed == trimmed.to_uppercase() && !trimmed.contains(' ') {
        return false;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::translation::document::Timecode;

    fn create_test_entry(id: usize, text: &str) -> DocumentEntry {
        DocumentEntry {
            id,
            timecode: Timecode::from_milliseconds(id as u64 * 1000, (id + 1) as u64 * 1000),
            original_text: text.to_string(),
            translated_text: None,
            speaker: None,
            scene_id: None,
            confidence: None,
            formatting: Vec::new(),
        }
    }

    #[test]
    fn test_extractSpeaker_withColonFormat_shouldExtract() {
        assert_eq!(extract_speaker("JOHN: Hello there!"), Some("JOHN".to_string()));
        assert_eq!(extract_speaker("Mary: How are you?"), Some("Mary".to_string()));
        assert_eq!(extract_speaker("Dr. Smith: The results are in."), Some("Dr. Smith".to_string()));
    }

    #[test]
    fn test_extractSpeaker_withBrackets_shouldExtract() {
        assert_eq!(extract_speaker("[NARRATOR]: Once upon a time..."), Some("NARRATOR".to_string()));
    }

    #[test]
    fn test_extractSpeaker_withNoSpeaker_shouldReturnNone() {
        assert_eq!(extract_speaker("Hello there!"), None);
        assert_eq!(extract_speaker("Just some text"), None);
    }

    #[test]
    fn test_isSoundEffect_shouldDetectBrackets() {
        assert!(is_sound_effect("[Door slams]"));
        assert!(is_sound_effect("(footsteps)"));
        assert!(!is_sound_effect("Regular text"));
        assert!(!is_sound_effect("JOHN: Hello"));
    }

    #[test]
    fn test_looksLikeDialogue_shouldIdentifyDialogue() {
        assert!(looks_like_dialogue("Hello, how are you?"));
        assert!(looks_like_dialogue("I'm fine, thanks."));
        assert!(!looks_like_dialogue("[Door slams]"));
        assert!(!looks_like_dialogue(""));
    }

    #[test]
    fn test_speakerTracker_detectSpeakers_shouldFindSpeakers() {
        let tracker = SpeakerTracker::new(SpeakerConfig {
            min_occurrences: 2,
            ..Default::default()
        });

        let mut entries = vec![
            create_test_entry(1, "JOHN: Hello there!"),
            create_test_entry(2, "MARY: Hi John!"),
            create_test_entry(3, "JOHN: How are you?"),
            create_test_entry(4, "MARY: I'm great!"),
            create_test_entry(5, "Just some narration."),
        ];

        let stats = tracker.detect_speakers(&mut entries);

        assert_eq!(stats.unique_speakers, 2);
        assert_eq!(stats.entries_with_speakers, 4);
        assert_eq!(entries[0].speaker, Some("JOHN".to_string()));
        assert_eq!(entries[1].speaker, Some("MARY".to_string()));
        assert_eq!(entries[4].speaker, None);
    }

    #[test]
    fn test_speakerTracker_detectSpeakers_shouldRespectMinOccurrences() {
        let tracker = SpeakerTracker::new(SpeakerConfig {
            min_occurrences: 3,
            ..Default::default()
        });

        let mut entries = vec![
            create_test_entry(1, "JOHN: Hello!"),
            create_test_entry(2, "JOHN: More text"),
            create_test_entry(3, "MARY: Hi!"), // Only appears once
        ];

        let stats = tracker.detect_speakers(&mut entries);

        assert_eq!(stats.unique_speakers, 0); // Neither meets threshold
    }

    #[test]
    fn test_speakerTracker_getSpeakers_shouldReturnList() {
        let tracker = SpeakerTracker::new(SpeakerConfig {
            min_occurrences: 1,
            ..Default::default()
        });

        let mut entries = vec![
            create_test_entry(1, "JOHN: Hello!"),
            create_test_entry(2, "MARY: Hi!"),
            create_test_entry(3, "JOHN: Goodbye!"),
        ];

        tracker.detect_speakers(&mut entries);
        let speakers = tracker.get_speakers(&entries);

        assert_eq!(speakers.len(), 2);
        let john = speakers.iter().find(|s| s.name == "JOHN").unwrap();
        assert_eq!(john.occurrence_count, 2);
    }

    #[test]
    fn test_speakerTracker_extractSpeakerNames_shouldNotModifyEntries() {
        let tracker = SpeakerTracker::new(SpeakerConfig {
            min_occurrences: 1,
            ..Default::default()
        });

        let entries = vec![
            create_test_entry(1, "JOHN: Hello!"),
            create_test_entry(2, "MARY: Hi!"),
        ];

        let names = tracker.extract_speaker_names(&entries);
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"JOHN".to_string()));
        assert!(names.contains(&"MARY".to_string()));

        // Original entries should not have speaker set
        assert!(entries[0].speaker.is_none());
    }

    #[test]
    fn test_speakerTracker_detectSpeakers_shouldCountSoundEffects() {
        let tracker = SpeakerTracker::with_defaults();

        let mut entries = vec![
            create_test_entry(1, "[Door opens]"),
            create_test_entry(2, "(footsteps)"),
            create_test_entry(3, "JOHN: Hello!"),
        ];

        let stats = tracker.detect_speakers(&mut entries);

        assert_eq!(stats.sound_effects_found, 2);
    }

    #[test]
    fn test_speakerConfig_strict_shouldHaveHigherThreshold() {
        let strict = SpeakerConfig::strict();
        let default = SpeakerConfig::default();

        assert!(strict.min_occurrences > default.min_occurrences);
        assert!(strict.continuity_gap < default.continuity_gap);
    }

    #[test]
    fn test_speakerConfig_lenient_shouldHaveLowerThreshold() {
        let lenient = SpeakerConfig::lenient();
        let default = SpeakerConfig::default();

        assert!(lenient.min_occurrences <= default.min_occurrences);
        assert!(lenient.detect_implicit_changes);
    }
}
