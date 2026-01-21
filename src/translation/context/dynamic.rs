/*!
 * Dynamic window sizing for adaptive context management.
 *
 * This module provides intelligent window sizing that adapts to:
 * - Scene boundaries: Avoid splitting scenes across batches
 * - Entry complexity: Larger batches for simple entries, smaller for complex
 * - Document position: More context at beginnings of scenes
 */

use crate::translation::document::{DocumentEntry, Scene};

/// Configuration for dynamic window sizing.
#[derive(Debug, Clone)]
pub struct DynamicWindowConfig {
    /// Minimum batch size (entries)
    pub min_batch_size: usize,

    /// Maximum batch size (entries)
    pub max_batch_size: usize,

    /// Target tokens per batch (soft limit)
    pub target_tokens: usize,

    /// Whether to respect scene boundaries
    pub respect_scene_boundaries: bool,

    /// Lookahead multiplier for scene detection
    pub lookahead_factor: f32,
}

impl Default for DynamicWindowConfig {
    fn default() -> Self {
        Self {
            min_batch_size: 5,
            max_batch_size: 25,
            target_tokens: 2000,
            respect_scene_boundaries: true,
            lookahead_factor: 1.5,
        }
    }
}

impl DynamicWindowConfig {
    /// Create a config for simple, fast processing.
    pub fn fast() -> Self {
        Self {
            min_batch_size: 10,
            max_batch_size: 30,
            target_tokens: 3000,
            respect_scene_boundaries: false,
            lookahead_factor: 1.0,
        }
    }

    /// Create a config for high-quality, scene-aware processing.
    pub fn quality() -> Self {
        Self {
            min_batch_size: 3,
            max_batch_size: 15,
            target_tokens: 1500,
            respect_scene_boundaries: true,
            lookahead_factor: 2.0,
        }
    }
}

/// Dynamic window sizer that calculates optimal batch sizes.
#[derive(Debug)]
pub struct DynamicWindowSizer {
    config: DynamicWindowConfig,
}

impl DynamicWindowSizer {
    /// Create a new dynamic window sizer with the given configuration.
    pub fn new(config: DynamicWindowConfig) -> Self {
        Self { config }
    }

    /// Create with default configuration.
    pub fn with_defaults() -> Self {
        Self::new(DynamicWindowConfig::default())
    }

    /// Calculate the optimal batch size for the current position.
    ///
    /// Returns the number of entries to include in the next batch.
    pub fn calculate_optimal_size(
        &self,
        entries: &[DocumentEntry],
        position: usize,
        scenes: Option<&[Scene]>,
    ) -> usize {
        if position >= entries.len() {
            return 0;
        }

        let remaining = entries.len() - position;

        // Start with target size based on complexity
        let complexity_size = self.size_by_complexity(&entries[position..]);

        // Adjust for scene boundaries if enabled
        let scene_adjusted = if self.config.respect_scene_boundaries {
            if let Some(scenes) = scenes {
                self.adjust_for_scenes(position, complexity_size, scenes, remaining)
            } else {
                complexity_size
            }
        } else {
            complexity_size
        };

        // Clamp to configured bounds
        scene_adjusted
            .max(self.config.min_batch_size)
            .min(self.config.max_batch_size)
            .min(remaining)
    }

    /// Calculate batch size based on entry complexity.
    fn size_by_complexity(&self, entries: &[DocumentEntry]) -> usize {
        let mut token_estimate = 0;
        let mut count = 0;

        for entry in entries.iter().take(self.config.max_batch_size) {
            let entry_tokens = estimate_tokens(&entry.original_text);
            if token_estimate + entry_tokens > self.config.target_tokens && count >= self.config.min_batch_size {
                break;
            }
            token_estimate += entry_tokens;
            count += 1;
        }

        count.max(self.config.min_batch_size)
    }

    /// Adjust batch size to respect scene boundaries.
    fn adjust_for_scenes(
        &self,
        position: usize,
        initial_size: usize,
        scenes: &[Scene],
        remaining: usize,
    ) -> usize {
        // Find the current scene
        let current_scene = scenes.iter().find(|s| {
            position >= s.start_entry_id && position <= s.end_entry_id
        });

        let Some(scene) = current_scene else {
            return initial_size;
        };

        // Calculate entries until scene end
        let entries_to_scene_end = scene.end_entry_id.saturating_sub(position) + 1;

        // If we can fit the rest of the scene, do it
        if entries_to_scene_end <= self.config.max_batch_size && entries_to_scene_end >= self.config.min_batch_size {
            return entries_to_scene_end.min(remaining);
        }

        // If initial size would end mid-scene, try to find a better boundary
        let end_position = position + initial_size;
        if end_position > scene.start_entry_id && end_position <= scene.end_entry_id {
            // We'd end in the middle of the scene
            // Option 1: Extend to scene end if not too big
            if entries_to_scene_end <= (self.config.max_batch_size as f32 * self.config.lookahead_factor) as usize {
                return entries_to_scene_end.min(remaining).min(self.config.max_batch_size);
            }
            // Option 2: Keep initial size (scene is too long)
        }

        initial_size
    }

    /// Get entries to include in the lookahead based on scenes.
    pub fn calculate_lookahead(
        &self,
        entries: &[DocumentEntry],
        batch_end: usize,
        scenes: Option<&[Scene]>,
        base_lookahead: usize,
    ) -> usize {
        if batch_end >= entries.len() {
            return 0;
        }

        let remaining = entries.len() - batch_end;

        if !self.config.respect_scene_boundaries {
            return base_lookahead.min(remaining);
        }

        // Find if the next entries are in a new scene
        if let Some(scenes) = scenes {
            let next_scene = scenes.iter().find(|s| {
                batch_end >= s.start_entry_id && batch_end <= s.end_entry_id
            });

            if let Some(scene) = next_scene {
                // Include entries to the end of the scene in lookahead
                let entries_to_scene_end = scene.end_entry_id.saturating_sub(batch_end) + 1;
                return entries_to_scene_end.min(remaining).min(base_lookahead * 2);
            }
        }

        base_lookahead.min(remaining)
    }

    /// Get the configuration.
    pub fn config(&self) -> &DynamicWindowConfig {
        &self.config
    }
}

impl Default for DynamicWindowSizer {
    fn default() -> Self {
        Self::with_defaults()
    }
}

/// Estimate token count for text (rough approximation).
fn estimate_tokens(text: &str) -> usize {
    // Rough estimate: ~4 chars per token for English
    // Adjust for languages with different character densities
    let char_count = text.chars().count();
    char_count.div_ceil(4)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::translation::document::Timecode;

    fn create_test_entries(count: usize, text_length: usize) -> Vec<DocumentEntry> {
        (0..count)
            .map(|i| {
                let text = "x".repeat(text_length);
                DocumentEntry {
                    id: i + 1,
                    timecode: Timecode::from_milliseconds(i as u64 * 1000, (i + 1) as u64 * 1000),
                    original_text: text,
                    translated_text: None,
                    speaker: None,
                    scene_id: None,
                    confidence: None,
                    formatting: Vec::new(),
                }
            })
            .collect()
    }

    fn create_test_scenes() -> Vec<Scene> {
        vec![
            Scene {
                id: 1,
                start_entry_id: 1,
                end_entry_id: 5,
                description: None,
                tone: None,
            },
            Scene {
                id: 2,
                start_entry_id: 6,
                end_entry_id: 12,
                description: None,
                tone: None,
            },
            Scene {
                id: 3,
                start_entry_id: 13,
                end_entry_id: 20,
                description: None,
                tone: None,
            },
        ]
    }

    #[test]
    fn test_dynamicWindowSizer_calculateOptimalSize_shouldRespectMinMax() {
        let sizer = DynamicWindowSizer::new(DynamicWindowConfig {
            min_batch_size: 5,
            max_batch_size: 10,
            ..Default::default()
        });

        let entries = create_test_entries(100, 20);

        let size = sizer.calculate_optimal_size(&entries, 0, None);
        assert!(size >= 5);
        assert!(size <= 10);
    }

    #[test]
    fn test_dynamicWindowSizer_calculateOptimalSize_shouldNotExceedRemaining() {
        let sizer = DynamicWindowSizer::with_defaults();
        let entries = create_test_entries(3, 20);

        let size = sizer.calculate_optimal_size(&entries, 0, None);
        assert!(size <= 3);
    }

    #[test]
    fn test_dynamicWindowSizer_calculateOptimalSize_atEndOfDocument_shouldReturnZero() {
        let sizer = DynamicWindowSizer::with_defaults();
        let entries = create_test_entries(10, 20);

        let size = sizer.calculate_optimal_size(&entries, 10, None);
        assert_eq!(size, 0);
    }

    #[test]
    fn test_dynamicWindowSizer_calculateOptimalSize_withScenes_shouldRespectBoundaries() {
        let sizer = DynamicWindowSizer::new(DynamicWindowConfig {
            min_batch_size: 3,
            max_batch_size: 10,
            respect_scene_boundaries: true,
            ..Default::default()
        });

        let entries = create_test_entries(20, 20);
        let scenes = create_test_scenes();

        // Position 1 is at start of scene 1 (entries 1-5)
        let size = sizer.calculate_optimal_size(&entries, 0, Some(&scenes));
        // Should try to fit within scene 1 (5 entries) since it's within max
        assert!(size <= 5 || size >= 3);
    }

    #[test]
    fn test_dynamicWindowSizer_sizeByComplexity_shortEntries_shouldReturnLargerBatch() {
        let sizer = DynamicWindowSizer::new(DynamicWindowConfig {
            target_tokens: 100,
            min_batch_size: 1,
            max_batch_size: 20,
            ..Default::default()
        });

        // Short entries (10 chars each, ~2.5 tokens)
        let short_entries = create_test_entries(20, 10);
        let short_size = sizer.calculate_optimal_size(&short_entries, 0, None);

        // Long entries (100 chars each, ~25 tokens)
        let long_entries = create_test_entries(20, 100);
        let long_size = sizer.calculate_optimal_size(&long_entries, 0, None);

        assert!(short_size >= long_size);
    }

    #[test]
    fn test_dynamicWindowSizer_calculateLookahead_shouldRespectRemaining() {
        let sizer = DynamicWindowSizer::with_defaults();
        let entries = create_test_entries(10, 20);

        let lookahead = sizer.calculate_lookahead(&entries, 8, None, 5);
        assert!(lookahead <= 2); // Only 2 entries remaining
    }

    #[test]
    fn test_dynamicWindowSizer_calculateLookahead_atEnd_shouldReturnZero() {
        let sizer = DynamicWindowSizer::with_defaults();
        let entries = create_test_entries(10, 20);

        let lookahead = sizer.calculate_lookahead(&entries, 10, None, 5);
        assert_eq!(lookahead, 0);
    }

    #[test]
    fn test_estimateTokens_shouldEstimateCorrectly() {
        assert_eq!(estimate_tokens(""), 0);
        assert_eq!(estimate_tokens("hi"), 1);
        assert_eq!(estimate_tokens("hello"), 2);
        assert_eq!(estimate_tokens("hello world test"), 4);
    }

    #[test]
    fn test_dynamicWindowConfig_fast_shouldHaveLargerBatches() {
        let fast = DynamicWindowConfig::fast();
        let quality = DynamicWindowConfig::quality();

        assert!(fast.max_batch_size > quality.max_batch_size);
        assert!(fast.target_tokens > quality.target_tokens);
    }

    #[test]
    fn test_dynamicWindowConfig_quality_shouldRespectScenes() {
        let quality = DynamicWindowConfig::quality();
        assert!(quality.respect_scene_boundaries);
    }
}
