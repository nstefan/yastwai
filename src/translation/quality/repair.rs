/*!
 * Auto-repair strategies for translation issues.
 *
 * Provides intelligent repair strategies for common translation problems:
 * - Formatting tag restoration
 * - Terminology correction
 * - Length adjustment
 * - Punctuation normalization
 */

use std::collections::HashMap;

use crate::translation::document::{DocumentEntry, FormattingTag, Glossary};

/// Types of repairs that can be applied.
#[derive(Debug, Clone, PartialEq)]
pub enum RepairStrategy {
    /// Restore missing formatting tags
    RestoreFormatting,

    /// Apply glossary term correction
    ApplyGlossary,

    /// Normalize punctuation
    NormalizePunctuation,

    /// Preserve character name
    PreserveName,

    /// Truncate overly long translation
    Truncate { max_ratio: f32 },

    /// Request retranslation
    Retranslate,

    /// No repair possible
    NoRepair,
}

impl RepairStrategy {
    /// Check if this strategy can be applied automatically.
    pub fn is_automatic(&self) -> bool {
        !matches!(self, RepairStrategy::Retranslate | RepairStrategy::NoRepair)
    }

    /// Get priority (higher = try first).
    pub fn priority(&self) -> u8 {
        match self {
            RepairStrategy::PreserveName => 100,
            RepairStrategy::RestoreFormatting => 90,
            RepairStrategy::ApplyGlossary => 80,
            RepairStrategy::NormalizePunctuation => 70,
            RepairStrategy::Truncate { .. } => 60,
            RepairStrategy::Retranslate => 10,
            RepairStrategy::NoRepair => 0,
        }
    }
}

/// Result of a repair attempt.
#[derive(Debug, Clone)]
pub struct SmartRepair {
    /// The strategy used
    pub strategy: RepairStrategy,

    /// Whether repair was successful
    pub success: bool,

    /// Original text
    pub original: String,

    /// Repaired text (if successful)
    pub repaired: Option<String>,

    /// Description of what was done
    pub description: String,
}

impl SmartRepair {
    /// Create a successful repair.
    pub fn success(strategy: RepairStrategy, original: &str, repaired: &str, description: &str) -> Self {
        Self {
            strategy,
            success: true,
            original: original.to_string(),
            repaired: Some(repaired.to_string()),
            description: description.to_string(),
        }
    }

    /// Create a failed repair.
    pub fn failure(strategy: RepairStrategy, original: &str, reason: &str) -> Self {
        Self {
            strategy,
            success: false,
            original: original.to_string(),
            repaired: None,
            description: reason.to_string(),
        }
    }

    /// Create a no-op repair (nothing to fix).
    pub fn noop(original: &str) -> Self {
        Self {
            strategy: RepairStrategy::NoRepair,
            success: true,
            original: original.to_string(),
            repaired: Some(original.to_string()),
            description: "No repair needed".to_string(),
        }
    }
}

/// Configuration for the repair engine.
#[derive(Debug, Clone)]
pub struct RepairConfig {
    /// Enable formatting restoration
    pub restore_formatting: bool,

    /// Enable glossary correction
    pub apply_glossary: bool,

    /// Enable punctuation normalization
    pub normalize_punctuation: bool,

    /// Enable name preservation
    pub preserve_names: bool,

    /// Enable length truncation
    pub truncate_long: bool,

    /// Maximum length ratio before truncation
    pub max_length_ratio: f32,

    /// Target quote style (if normalizing)
    pub target_quote_style: QuoteStyle,
}

impl Default for RepairConfig {
    fn default() -> Self {
        Self {
            restore_formatting: true,
            apply_glossary: true,
            normalize_punctuation: true,
            preserve_names: true,
            truncate_long: false, // Risky, disabled by default
            max_length_ratio: 1.5,
            target_quote_style: QuoteStyle::Double,
        }
    }
}

impl RepairConfig {
    /// Create an aggressive config that repairs everything.
    pub fn aggressive() -> Self {
        Self {
            restore_formatting: true,
            apply_glossary: true,
            normalize_punctuation: true,
            preserve_names: true,
            truncate_long: true,
            max_length_ratio: 1.5,
            target_quote_style: QuoteStyle::Double,
        }
    }

    /// Create a conservative config.
    pub fn conservative() -> Self {
        Self {
            restore_formatting: true,
            apply_glossary: false,
            normalize_punctuation: false,
            preserve_names: true,
            truncate_long: false,
            max_length_ratio: 2.0,
            target_quote_style: QuoteStyle::Double,
        }
    }
}

/// Quote styles for normalization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuoteStyle {
    /// "text"
    Double,
    /// 'text'
    Single,
    /// «text»
    Guillemet,
    /// "text"
    SmartDouble,
}

/// Repair engine for automatic issue correction.
pub struct RepairEngine {
    config: RepairConfig,
}

impl RepairEngine {
    /// Create a new repair engine with default config.
    pub fn new() -> Self {
        Self {
            config: RepairConfig::default(),
        }
    }

    /// Create with custom config.
    pub fn with_config(config: RepairConfig) -> Self {
        Self { config }
    }

    /// Repair an entry with all applicable strategies.
    pub fn repair_entry(
        &self,
        entry: &DocumentEntry,
        glossary: &Glossary,
    ) -> Vec<SmartRepair> {
        let mut repairs = Vec::new();

        let translated = match &entry.translated_text {
            Some(t) => t.clone(),
            None => return repairs,
        };

        let mut current = translated.clone();

        // Apply repairs in priority order
        if self.config.preserve_names {
            if let Some(repair) = self.repair_names(entry, &current, glossary) {
                if repair.success {
                    current = repair.repaired.clone().unwrap_or(current);
                }
                repairs.push(repair);
            }
        }

        if self.config.restore_formatting {
            if let Some(repair) = self.repair_formatting(entry, &current) {
                if repair.success {
                    current = repair.repaired.clone().unwrap_or(current);
                }
                repairs.push(repair);
            }
        }

        if self.config.apply_glossary {
            if let Some(repair) = self.repair_glossary(entry, &current, glossary) {
                if repair.success {
                    current = repair.repaired.clone().unwrap_or(current);
                }
                repairs.push(repair);
            }
        }

        if self.config.normalize_punctuation {
            if let Some(repair) = self.repair_punctuation(&current) {
                if repair.success {
                    current = repair.repaired.clone().unwrap_or(current);
                }
                repairs.push(repair);
            }
        }

        if self.config.truncate_long {
            if let Some(repair) = self.repair_length(entry, &current) {
                repairs.push(repair);
            }
        }

        repairs
    }

    /// Repair missing character names.
    fn repair_names(
        &self,
        entry: &DocumentEntry,
        translated: &str,
        glossary: &Glossary,
    ) -> Option<SmartRepair> {
        let original = &entry.original_text;
        let mut repaired = translated.to_string();
        let mut fixed = false;

        for name in &glossary.character_names {
            if original.contains(name) && !repaired.contains(name) {
                // Try to find and replace a mistranslated name
                // Simple heuristic: look for similar-length capitalized words
                if let Some(replacement_pos) = self.find_name_replacement_position(&repaired, name) {
                    let (start, end) = replacement_pos;
                    repaired = format!("{}{}{}", &repaired[..start], name, &repaired[end..]);
                    fixed = true;
                }
            }
        }

        if fixed {
            Some(SmartRepair::success(
                RepairStrategy::PreserveName,
                translated,
                &repaired,
                "Restored character name(s)",
            ))
        } else {
            None
        }
    }

    /// Find position to replace a mistranslated name.
    fn find_name_replacement_position(&self, text: &str, name: &str) -> Option<(usize, usize)> {
        let name_len = name.len();
        let tolerance = (name_len / 3).max(1);

        let mut current_pos = 0;
        for word in text.split_whitespace() {
            let word_start = text[current_pos..].find(word).map(|p| p + current_pos)?;
            let word_end = word_start + word.len();
            current_pos = word_end;

            let clean_word = word.trim_matches(|c: char| !c.is_alphabetic());
            if clean_word.is_empty() {
                continue;
            }

            // Check if it's a capitalized word of similar length
            if clean_word.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
                let len_diff = (clean_word.len() as i32 - name_len as i32).unsigned_abs() as usize;
                if len_diff <= tolerance && clean_word != name {
                    // Found a candidate
                    let clean_start = text[word_start..].find(clean_word).map(|p| p + word_start)?;
                    let clean_end = clean_start + clean_word.len();
                    return Some((clean_start, clean_end));
                }
            }
        }

        None
    }

    /// Repair missing formatting tags.
    fn repair_formatting(&self, entry: &DocumentEntry, translated: &str) -> Option<SmartRepair> {
        let original = &entry.original_text;
        let mut repaired = translated.to_string();
        let mut fixed = false;

        // Check and restore various formatting tags
        for tag in &entry.formatting {
            let (open, close) = match tag {
                FormattingTag::Italic => ("<i>", "</i>"),
                FormattingTag::Bold => ("<b>", "</b>"),
                FormattingTag::Underline => ("<u>", "</u>"),
                FormattingTag::Color => continue, // Complex, skip for now
                FormattingTag::Position => {
                    // Handle position tags specially
                    if let Some(pos_tag) = self.extract_position_tag(original) {
                        if !repaired.contains(&pos_tag) {
                            repaired = format!("{}{}", pos_tag, repaired);
                            fixed = true;
                        }
                    }
                    continue;
                }
            };

            let has_open = original.contains(open);
            let has_close = original.contains(close);
            let trans_has_open = repaired.contains(open);
            let trans_has_close = repaired.contains(close);

            // If original is fully wrapped, wrap translation
            if has_open && has_close && original.starts_with(open) && original.ends_with(close) {
                if !trans_has_open || !trans_has_close {
                    repaired = format!("{}{}{}", open, repaired.trim(), close);
                    fixed = true;
                }
            }
        }

        if fixed {
            Some(SmartRepair::success(
                RepairStrategy::RestoreFormatting,
                translated,
                &repaired,
                "Restored formatting tags",
            ))
        } else {
            None
        }
    }

    /// Extract position tag from text.
    fn extract_position_tag(&self, text: &str) -> Option<String> {
        if let Some(start) = text.find("{\\an") {
            if let Some(end) = text[start..].find('}') {
                return Some(text[start..start + end + 1].to_string());
            }
        }
        None
    }

    /// Repair glossary term usage.
    fn repair_glossary(
        &self,
        entry: &DocumentEntry,
        translated: &str,
        glossary: &Glossary,
    ) -> Option<SmartRepair> {
        let original = &entry.original_text;
        let mut repaired = translated.to_string();
        let mut fixed = false;

        for (source_term, term_info) in &glossary.terms {
            if original.contains(source_term) {
                // If source term appears untranslated in output, replace with target
                if repaired.contains(source_term) {
                    repaired = repaired.replace(source_term, &term_info.target);
                    fixed = true;
                }
            }
        }

        if fixed {
            Some(SmartRepair::success(
                RepairStrategy::ApplyGlossary,
                translated,
                &repaired,
                "Applied glossary corrections",
            ))
        } else {
            None
        }
    }

    /// Repair punctuation inconsistencies.
    fn repair_punctuation(&self, translated: &str) -> Option<SmartRepair> {
        let mut repaired = translated.to_string();
        let mut fixed = false;

        // Normalize quotes to target style
        let (open_quote, close_quote) = match self.config.target_quote_style {
            QuoteStyle::Double => ('"', '"'),
            QuoteStyle::Single => ('\'', '\''),
            QuoteStyle::Guillemet => ('«', '»'),
            QuoteStyle::SmartDouble => ('"', '"'),
        };

        // Replace various quote styles
        let quote_replacements = [
            ('"', open_quote),
            ('"', close_quote),
            ('„', open_quote),
            ('‟', close_quote),
            ('«', open_quote),
            ('»', close_quote),
        ];

        for (from, to) in quote_replacements {
            if repaired.contains(from) && from != to {
                repaired = repaired.replace(from, &to.to_string());
                fixed = true;
            }
        }

        // Normalize ellipsis
        if repaired.contains("...") {
            repaired = repaired.replace("...", "…");
            fixed = true;
        }

        if fixed {
            Some(SmartRepair::success(
                RepairStrategy::NormalizePunctuation,
                translated,
                &repaired,
                "Normalized punctuation",
            ))
        } else {
            None
        }
    }

    /// Repair overly long translations.
    fn repair_length(&self, entry: &DocumentEntry, translated: &str) -> Option<SmartRepair> {
        let original_len = entry.original_text.chars().count();
        let translated_len = translated.chars().count();

        if original_len == 0 {
            return None;
        }

        let ratio = translated_len as f32 / original_len as f32;

        if ratio <= self.config.max_length_ratio {
            return None;
        }

        // Calculate target length
        let target_len = (original_len as f32 * self.config.max_length_ratio) as usize;

        // Truncate at word boundary
        let truncated = self.truncate_at_word_boundary(translated, target_len);

        Some(SmartRepair::success(
            RepairStrategy::Truncate {
                max_ratio: self.config.max_length_ratio,
            },
            translated,
            &truncated,
            &format!("Truncated from {} to {} chars", translated_len, truncated.chars().count()),
        ))
    }

    /// Truncate text at a word boundary.
    fn truncate_at_word_boundary(&self, text: &str, max_len: usize) -> String {
        if text.chars().count() <= max_len {
            return text.to_string();
        }

        let mut result = String::new();
        let mut current_len = 0;

        for word in text.split_whitespace() {
            let word_len = word.chars().count();
            if current_len + word_len + 1 > max_len {
                break;
            }
            if !result.is_empty() {
                result.push(' ');
                current_len += 1;
            }
            result.push_str(word);
            current_len += word_len;
        }

        if result.is_empty() {
            // Fall back to character truncation
            text.chars().take(max_len.saturating_sub(3)).collect::<String>() + "..."
        } else {
            result + "..."
        }
    }

    /// Get the final repaired text from a series of repairs.
    pub fn get_final_text(&self, repairs: &[SmartRepair]) -> Option<String> {
        repairs
            .iter()
            .rev()
            .find(|r| r.success && r.repaired.is_some())
            .and_then(|r| r.repaired.clone())
    }
}

impl Default for RepairEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::subtitle_processor::SubtitleEntry;

    fn create_entry(id: usize, original: &str, translated: Option<&str>) -> DocumentEntry {
        let subtitle = SubtitleEntry::new(id, 0, 2000, original.to_string());
        let mut entry = crate::translation::document::DocumentEntry::from_subtitle_entry(subtitle);
        if let Some(t) = translated {
            entry.set_translation(t.to_string(), None);
        }
        entry
    }

    #[test]
    fn test_repairEngine_repairFormatting_shouldRestoreItalics() {
        let mut entry = create_entry(1, "<i>Whispered text</i>", Some("Texte chuchoté"));
        entry.formatting.push(FormattingTag::Italic);

        let engine = RepairEngine::new();
        let repair = engine.repair_formatting(&entry, "Texte chuchoté");

        assert!(repair.is_some());
        let repair = repair.unwrap();
        assert!(repair.success);
        assert!(repair.repaired.as_ref().unwrap().contains("<i>"));
        assert!(repair.repaired.as_ref().unwrap().contains("</i>"));
    }

    #[test]
    fn test_repairEngine_repairPunctuation_shouldNormalizeQuotes() {
        let engine = RepairEngine::new();
        let repair = engine.repair_punctuation("He said «hello»");

        assert!(repair.is_some());
        let repair = repair.unwrap();
        assert!(repair.success);
        assert!(repair.repaired.as_ref().unwrap().contains('"'));
    }

    #[test]
    fn test_repairEngine_repairLength_shouldTruncate() {
        let entry = create_entry(1, "Short", Some("This is a very long translation that should be truncated"));

        let config = RepairConfig {
            truncate_long: true,
            max_length_ratio: 2.0,
            ..Default::default()
        };
        let engine = RepairEngine::with_config(config);

        let repair = engine.repair_length(&entry, "This is a very long translation that should be truncated");

        assert!(repair.is_some());
        let repair = repair.unwrap();
        assert!(repair.success);
        assert!(repair.repaired.as_ref().unwrap().len() < 50);
    }

    #[test]
    fn test_repairEngine_repairGlossary_shouldApplyTerms() {
        let entry = create_entry(1, "Go to the extraction point.", Some("Allez au extraction point."));

        let mut glossary = Glossary::new();
        glossary.add_term("extraction point", "point d'extraction", None);

        let engine = RepairEngine::new();
        let repair = engine.repair_glossary(&entry, "Allez au extraction point.", &glossary);

        assert!(repair.is_some());
        let repair = repair.unwrap();
        assert!(repair.success);
        assert!(repair.repaired.as_ref().unwrap().contains("point d'extraction"));
    }

    #[test]
    fn test_repairStrategy_priority_shouldOrderCorrectly() {
        assert!(RepairStrategy::PreserveName.priority() > RepairStrategy::RestoreFormatting.priority());
        assert!(RepairStrategy::RestoreFormatting.priority() > RepairStrategy::Retranslate.priority());
    }

    #[test]
    fn test_truncateAtWordBoundary_shouldRespectWords() {
        let engine = RepairEngine::new();

        let result = engine.truncate_at_word_boundary("This is a test of the truncation function", 20);

        assert!(result.len() <= 23); // 20 + "..."
        assert!(result.ends_with("..."));
        assert!(!result.contains("truncation")); // Should be cut before this
    }
}
