/*!
 * Scene detection for subtitle documents.
 *
 * Scenes are detected based on timing gaps between subtitles.
 * A significant gap (e.g., > 3 seconds) typically indicates a scene change.
 * Scene boundaries help the translation maintain appropriate context.
 */

use crate::translation::document::{DocumentEntry, Scene, SubtitleDocument};

/// Configuration for scene detection.
#[derive(Debug, Clone)]
pub struct SceneDetectionConfig {
    /// Minimum gap in milliseconds to consider a scene break
    pub min_gap_ms: u64,

    /// Maximum entries per scene (force break if exceeded)
    pub max_entries_per_scene: usize,

    /// Whether to detect scene breaks on speaker changes (if available)
    pub detect_speaker_changes: bool,
}

impl Default for SceneDetectionConfig {
    fn default() -> Self {
        Self {
            min_gap_ms: 3000, // 3 seconds
            max_entries_per_scene: 50,
            detect_speaker_changes: true,
        }
    }
}

impl SceneDetectionConfig {
    /// Create a config for short-form content (fewer scene breaks).
    pub fn short_form() -> Self {
        Self {
            min_gap_ms: 5000, // 5 seconds
            max_entries_per_scene: 100,
            detect_speaker_changes: false,
        }
    }

    /// Create a config for detailed scene detection.
    pub fn detailed() -> Self {
        Self {
            min_gap_ms: 2000, // 2 seconds
            max_entries_per_scene: 30,
            detect_speaker_changes: true,
        }
    }
}

/// Scene detector for identifying scene boundaries in subtitle documents.
pub struct SceneDetector {
    config: SceneDetectionConfig,
}

impl SceneDetector {
    /// Create a new scene detector with the given configuration.
    pub fn new(config: SceneDetectionConfig) -> Self {
        Self { config }
    }

    /// Create a scene detector with default configuration.
    pub fn with_defaults() -> Self {
        Self::new(SceneDetectionConfig::default())
    }

    /// Detect scenes in a document's entries.
    pub fn detect_scenes(&self, entries: &[DocumentEntry]) -> Vec<Scene> {
        if entries.is_empty() {
            return Vec::new();
        }

        let mut scenes = Vec::new();
        let mut scene_start = 0;
        let mut scene_id = 1;

        for i in 1..entries.len() {
            let should_break = self.should_break_scene(entries, scene_start, i);

            if should_break {
                // Create scene from scene_start to i-1
                scenes.push(Scene::new(scene_id, entries[scene_start].id, entries[i - 1].id));
                scene_id += 1;
                scene_start = i;
            }
        }

        // Add the final scene
        if scene_start < entries.len() {
            scenes.push(Scene::new(
                scene_id,
                entries[scene_start].id,
                entries[entries.len() - 1].id,
            ));
        }

        scenes
    }

    /// Determine if a scene break should occur at the given position.
    fn should_break_scene(
        &self,
        entries: &[DocumentEntry],
        scene_start: usize,
        current: usize,
    ) -> bool {
        let prev = &entries[current - 1];
        let curr = &entries[current];

        // Check timing gap
        let gap_ms = curr.timecode.start_ms.saturating_sub(prev.timecode.end_ms);
        if gap_ms >= self.config.min_gap_ms {
            return true;
        }

        // Check max entries per scene
        let scene_length = current - scene_start;
        if scene_length >= self.config.max_entries_per_scene {
            return true;
        }

        // Check speaker change (if enabled and speakers are detected)
        if self.config.detect_speaker_changes {
            if let (Some(prev_speaker), Some(curr_speaker)) = (&prev.speaker, &curr.speaker) {
                if prev_speaker != curr_speaker {
                    return true;
                }
            }
        }

        false
    }

    /// Detect scenes and update the document.
    pub fn detect_and_update(&self, doc: &mut SubtitleDocument) {
        let scenes = self.detect_scenes(&doc.entries);

        // Update entries with scene IDs
        for scene in &scenes {
            for entry in &mut doc.entries {
                if entry.id >= scene.start_entry_id && entry.id <= scene.end_entry_id {
                    entry.scene_id = Some(scene.id);
                }
            }
        }

        doc.scenes = scenes;
    }

    /// Calculate the gap between two entries in milliseconds.
    pub fn gap_between(entry1: &DocumentEntry, entry2: &DocumentEntry) -> u64 {
        entry2.timecode.start_ms.saturating_sub(entry1.timecode.end_ms)
    }

    /// Find the largest timing gaps in the document.
    pub fn find_largest_gaps(&self, entries: &[DocumentEntry], count: usize) -> Vec<(usize, u64)> {
        if entries.len() < 2 {
            return Vec::new();
        }

        let mut gaps: Vec<(usize, u64)> = (1..entries.len())
            .map(|i| {
                let gap = Self::gap_between(&entries[i - 1], &entries[i]);
                (i, gap)
            })
            .collect();

        gaps.sort_by(|a, b| b.1.cmp(&a.1));
        gaps.truncate(count);
        gaps
    }
}

/// Extension trait for SubtitleDocument to add scene detection.
pub trait SceneDetectionExt {
    /// Detect scenes in the document using default configuration.
    fn detect_scenes(&mut self);

    /// Detect scenes with custom configuration.
    fn detect_scenes_with_config(&mut self, config: SceneDetectionConfig);

    /// Get the scene for a given entry ID.
    fn scene_for_entry(&self, entry_id: usize) -> Option<&Scene>;
}

impl SceneDetectionExt for SubtitleDocument {
    fn detect_scenes(&mut self) {
        let detector = SceneDetector::with_defaults();
        detector.detect_and_update(self);
    }

    fn detect_scenes_with_config(&mut self, config: SceneDetectionConfig) {
        let detector = SceneDetector::new(config);
        detector.detect_and_update(self);
    }

    fn scene_for_entry(&self, entry_id: usize) -> Option<&Scene> {
        self.scenes.iter().find(|scene| {
            entry_id >= scene.start_entry_id && entry_id <= scene.end_entry_id
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::subtitle_processor::SubtitleEntry;

    fn create_entries_with_gaps(gaps_ms: &[u64]) -> Vec<DocumentEntry> {
        let mut entries = Vec::new();
        let mut current_time = 0u64;

        for (i, &gap) in std::iter::once(&0u64).chain(gaps_ms.iter()).enumerate() {
            current_time += gap;
            let start = current_time;
            let end = current_time + 1000; // 1 second duration

            let entry = SubtitleEntry::new(i + 1, start, end, format!("Line {}", i + 1));
            entries.push(DocumentEntry::from_subtitle_entry(entry));
            current_time = end;
        }

        entries
    }

    #[test]
    fn test_sceneDetector_detectScenes_shouldDetectGaps() {
        // Create entries with a 5 second gap after entry 3
        let entries = create_entries_with_gaps(&[100, 100, 5000, 100, 100]);

        let detector = SceneDetector::new(SceneDetectionConfig {
            min_gap_ms: 3000,
            max_entries_per_scene: 100,
            detect_speaker_changes: false,
        });

        let scenes = detector.detect_scenes(&entries);

        assert_eq!(scenes.len(), 2);
        assert_eq!(scenes[0].start_entry_id, 1);
        assert_eq!(scenes[0].end_entry_id, 3);
        assert_eq!(scenes[1].start_entry_id, 4);
        assert_eq!(scenes[1].end_entry_id, 6);
    }

    #[test]
    fn test_sceneDetector_detectScenes_shouldRespectMaxEntries() {
        // Create 10 entries with no significant gaps
        let entries = create_entries_with_gaps(&[100, 100, 100, 100, 100, 100, 100, 100, 100]);

        let detector = SceneDetector::new(SceneDetectionConfig {
            min_gap_ms: 10000, // High threshold, won't trigger
            max_entries_per_scene: 5,
            detect_speaker_changes: false,
        });

        let scenes = detector.detect_scenes(&entries);

        assert_eq!(scenes.len(), 2);
        assert_eq!(scenes[0].end_entry_id - scenes[0].start_entry_id + 1, 5);
    }

    #[test]
    fn test_sceneDetector_detectScenes_withNoGaps_shouldReturnSingleScene() {
        let entries = create_entries_with_gaps(&[100, 100, 100]);

        let detector = SceneDetector::new(SceneDetectionConfig {
            min_gap_ms: 3000,
            max_entries_per_scene: 100,
            detect_speaker_changes: false,
        });

        let scenes = detector.detect_scenes(&entries);

        assert_eq!(scenes.len(), 1);
        assert_eq!(scenes[0].start_entry_id, 1);
        assert_eq!(scenes[0].end_entry_id, 4);
    }

    #[test]
    fn test_sceneDetector_detectScenes_withEmptyEntries_shouldReturnEmpty() {
        let entries: Vec<DocumentEntry> = Vec::new();
        let detector = SceneDetector::with_defaults();

        let scenes = detector.detect_scenes(&entries);

        assert!(scenes.is_empty());
    }

    #[test]
    fn test_sceneDetector_findLargestGaps_shouldFindTopGaps() {
        let entries = create_entries_with_gaps(&[100, 5000, 100, 3000, 100]);

        let detector = SceneDetector::with_defaults();
        let gaps = detector.find_largest_gaps(&entries, 2);

        assert_eq!(gaps.len(), 2);
        assert_eq!(gaps[0].1, 5000); // Largest gap
        assert_eq!(gaps[1].1, 3000); // Second largest
    }

    #[test]
    fn test_sceneDetectionExt_detectAndUpdate_shouldUpdateDocument() {
        let entries: Vec<SubtitleEntry> = vec![
            SubtitleEntry::new(1, 0, 1000, "Line 1".to_string()),
            SubtitleEntry::new(2, 1100, 2000, "Line 2".to_string()),
            SubtitleEntry::new(3, 6000, 7000, "Line 3".to_string()), // 4 second gap
            SubtitleEntry::new(4, 7100, 8000, "Line 4".to_string()),
        ];

        let mut doc = SubtitleDocument::from_entries(entries, "en");
        doc.detect_scenes();

        assert_eq!(doc.scenes.len(), 2);
        assert_eq!(doc.entries[0].scene_id, Some(1));
        assert_eq!(doc.entries[1].scene_id, Some(1));
        assert_eq!(doc.entries[2].scene_id, Some(2));
        assert_eq!(doc.entries[3].scene_id, Some(2));
    }

    #[test]
    fn test_sceneDetectionExt_sceneForEntry_shouldFindCorrectScene() {
        let entries: Vec<SubtitleEntry> = vec![
            SubtitleEntry::new(1, 0, 1000, "Line 1".to_string()),
            SubtitleEntry::new(2, 5000, 6000, "Line 2".to_string()),
        ];

        let mut doc = SubtitleDocument::from_entries(entries, "en");
        doc.detect_scenes();

        let scene1 = doc.scene_for_entry(1);
        let scene2 = doc.scene_for_entry(2);

        assert!(scene1.is_some());
        assert!(scene2.is_some());
        assert_ne!(scene1.unwrap().id, scene2.unwrap().id);
    }
}

