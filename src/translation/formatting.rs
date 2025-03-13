/*!
 * Format preservation for translated text.
 * 
 * This module provides functionality to preserve formatting elements
 * like line breaks, italics, bold, and other styling when translating text.
 */

use regex::Regex;
use std::collections::HashMap;
use once_cell::sync::Lazy;

/// Static regex patterns for formatting detection
static FORMATTING_PATTERNS: Lazy<Vec<(Regex, &'static str)>> = Lazy::new(|| {
    vec![
        // Italic formatting with asterisks
        (Regex::new(r"\*([^*]+)\*").unwrap(), "*$1*"),
        
        // Bold formatting with double asterisks
        (Regex::new(r"\*\*([^*]+)\*\*").unwrap(), "**$1**"),
        
        // Underline with underscores
        (Regex::new(r"_([^_]+)_").unwrap(), "_$1_"),
        
        // SSA/ASS italic tags
        (Regex::new(r"\\i1([^\\]+)\\i0").unwrap(), "\\i1$1\\i0"),
        
        // SSA/ASS bold tags
        (Regex::new(r"\\b1([^\\]+)\\b0").unwrap(), "\\b1$1\\b0"),
        
        // SSA/ASS underline tags
        (Regex::new(r"\\u1([^\\]+)\\u0").unwrap(), "\\u1$1\\u0"),
        
        // SRT italic tags
        (Regex::new(r"<i>([^<]+)</i>").unwrap(), "<i>$1</i>"),
        
        // SRT bold tags
        (Regex::new(r"<b>([^<]+)</b>").unwrap(), "<b>$1</b>"),
        
        // SRT underline tags
        (Regex::new(r"<u>([^<]+)</u>").unwrap(), "<u>$1</u>"),
        
        // SRT font tags
        (Regex::new(r"<font([^>]*)>([^<]+)</font>").unwrap(), "<font$1>$2</font>"),
    ]
});

/// Positional tag regex ({\an8} etc.)
static POSITION_TAG_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(\{\\an\d\})").unwrap()
});

/// Language indicator regex ([IN SPANISH], [EN FRANÇAIS], etc.)
static LANGUAGE_INDICATOR_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\[([^]]*?)(IN|EN|À|AU|AUX|DE)\s+([^]]*?)\]").unwrap()
});

/// Format preserver for maintaining text formatting during translation
pub struct FormatPreserver;

impl FormatPreserver {
    /// Preserve formatting from original text in translated text
    pub fn preserve_formatting(original: &str, translated: &str) -> String {
        // If either string is empty, return the translated text as is
        if original.is_empty() || translated.is_empty() {
            return translated.to_string();
        }
        
        // First, preserve position tags ({\an8})
        let mut result = Self::preserve_position_tags(original, translated);
        
        // Then, preserve language indicators
        result = Self::preserve_language_indicators(original, &result);
        
        // Next, try to preserve line breaks
        result = Self::preserve_line_breaks(original, &result);
        
        // Then, try to preserve formatting tags (and fix doubled tags)
        result = Self::preserve_formatting_tags(&result);
        
        // Fix doubled formatting tags
        result = Self::fix_doubled_formatting_tags(&result);
        
        result
    }
    
    /// Preserve position tags like {\an8} from original text
    fn preserve_position_tags(original: &str, translated: &str) -> String {
        // Find position tags in the original text
        let position_tags: Vec<_> = POSITION_TAG_REGEX.find_iter(original).collect();
        
        if position_tags.is_empty() {
            return translated.to_string();
        }
        
        let mut result = translated.to_string();
        
        // Add each position tag at the start of each line in the translated text
        for tag_match in position_tags {
            let tag = &original[tag_match.start()..tag_match.end()];
            
            // Check if the tag is already in the translated text
            if !result.contains(tag) {
                // Split by lines
                let lines: Vec<&str> = result.split('\n').collect();
                
                if !lines.is_empty() {
                    // Add the tag to the first line if it starts with a letter
                    // (to avoid adding it to an existing tag)
                    let first_line = lines[0];
                    let mut new_result = String::new();
                    
                    if !first_line.starts_with('{') {
                        new_result.push_str(tag);
                        new_result.push_str(first_line);
                    } else {
                        new_result.push_str(first_line);
                    }
                    
                    // Add the rest of the lines
                    for line in &lines[1..] {
                        new_result.push('\n');
                        new_result.push_str(line);
                    }
                    
                    result = new_result;
                }
            }
        }
        
        result
    }
    
    /// Preserve language indicators like [IN SPANISH] from original text
    fn preserve_language_indicators(original: &str, translated: &str) -> String {
        // Find language indicators in the original text
        let language_indicators = LANGUAGE_INDICATOR_REGEX.captures_iter(original);
        
        let mut result = translated.to_string();
        
        // Create a fallback regex outside the loop to avoid creating it inside the loop
        let fallback_regex = Regex::new(r"\[\]").unwrap();
        
        for cap in language_indicators {
            if cap.len() >= 4 {
                let full_match = cap.get(0).unwrap().as_str();
                let prefix = cap.get(1).map_or("", |m| m.as_str());
                let indicator = cap.get(2).unwrap().as_str();
                let _language = cap.get(3).unwrap().as_str();
                
                // Preserve the exact original language indicator
                if result.contains(full_match) {
                    continue;
                }
                
                // Look for a translated version of the language indicator
                let translated_indicator = match indicator {
                    "IN" => "EN",
                    "EN" => "EN",
                    "À" => "À",
                    "AU" => "AU",
                    "AUX" => "AUX",
                    "DE" => "DE",
                    _ => indicator,
                };
                
                // Create regex to find the translated language indicator
                let pattern = format!(r"\[{prefix}?{translated_indicator}\s+[^]]*?\]");
                let translated_regex = Regex::new(&pattern).unwrap_or_else(|_| fallback_regex.clone());
                
                if let Some(m) = translated_regex.find(&result) {
                    // Replace the translated language indicator with the original one
                    let before = &result[..m.start()];
                    let after = &result[m.end()..];
                    result = format!("{}{}{}", before, full_match, after);
                }
            }
        }
        
        result
    }
    
    /// Preserve line breaks from original text in translated text
    fn preserve_line_breaks(original: &str, translated: &str) -> String {
        let original_lines: Vec<&str> = original.split('\n').collect();
        let translated_lines: Vec<&str> = translated.split('\n').collect();
        
        // If the number of lines matches, we can just return the translated text
        if original_lines.len() == translated_lines.len() {
            return translated.to_string();
        }
        
        // If the original has multiple lines but the translation doesn't,
        // try to split the translation to match the original line count
        if original_lines.len() > 1 && translated_lines.len() == 1 {
            return Self::split_translation_to_match_lines(original, translated);
        }
        
        // If there are extra lines in the translation that don't exist in the original,
        // try to merge them
        if translated_lines.len() > original_lines.len() {
            let mut result = String::new();
            let mut i = 0;
            
            while i < translated_lines.len() {
                if i < original_lines.len() {
                    if !result.is_empty() {
                        result.push('\n');
                    }
                    result.push_str(translated_lines[i]);
                } else {
                    // For extra lines, append to the last valid line
                    result.push(' ');
                    result.push_str(translated_lines[i]);
                }
                i += 1;
            }
            
            return result;
        }
        
        // Otherwise, just return the translated text as is
        translated.to_string()
    }
    
    /// Split a single-line translation to match the line count of the original
    fn split_translation_to_match_lines(original: &str, translated: &str) -> String {
        let original_lines: Vec<&str> = original.split('\n').collect();
        
        // If the original has only one line, return the translated text as is
        if original_lines.len() <= 1 {
            return translated.to_string();
        }
        
        // Calculate the average line length in the original
        let original_chars: Vec<usize> = original_lines.iter().map(|line| line.chars().count()).collect();
        let total_original_chars: usize = original_chars.iter().sum();
        
        // Create a vector to store the split points
        let mut split_points = Vec::new();
        
        // Calculate split points based on the proportion of characters in each original line
        let mut current_pos = 0;
        for (_i, &char_count) in original_chars.iter().enumerate().take(original_lines.len() - 1) {
            let proportion = char_count as f64 / total_original_chars as f64;
            let chars_in_translated = translated.chars().count();
            let split_point = (proportion * chars_in_translated as f64).round() as usize;
            
            current_pos += split_point;
            if current_pos < chars_in_translated {
                split_points.push(current_pos);
            }
        }
        
        // Split the translated text at the calculated points
        let mut result = String::new();
        let mut last_pos = 0;
        let translated_chars: Vec<char> = translated.chars().collect();
        
        for pos in split_points {
            if pos > last_pos && pos < translated_chars.len() {
                result.push_str(&translated_chars[last_pos..pos].iter().collect::<String>());
                result.push('\n');
                last_pos = pos;
            }
        }
        
        // Add the remaining text
        if last_pos < translated_chars.len() {
            result.push_str(&translated_chars[last_pos..].iter().collect::<String>());
        }
        
        result
    }
    
    /// Preserve formatting tags from original text in translated text
    fn preserve_formatting_tags(text: &str) -> String {
        let mut result = text.to_string();
        
        // Apply each formatting pattern
        for (pattern, replacement) in FORMATTING_PATTERNS.iter() {
            // Find all matches in the original text
            let matches: Vec<_> = pattern.find_iter(text).collect();
            
            // If there are no matches, continue to the next pattern
            if matches.is_empty() {
                continue;
            }
            
            // Create a map of positions to replacements
            let mut replacements = HashMap::new();
            
            for m in matches {
                let start = m.start();
                let end = m.end();
                let matched_text = &text[start..end];
                
                // Extract the content inside the formatting tags
                let content = pattern.captures(matched_text).unwrap().get(1).unwrap().as_str();
                
                // Find the content in the result text
                if let Some(content_pos) = result.find(content) {
                    // Create the replacement with the same formatting
                    let replacement_text = pattern.replace(matched_text, *replacement).to_string();
                    
                    // Store the replacement
                    replacements.insert(content_pos, (content.len(), replacement_text));
                }
            }
            
            // Apply the replacements in reverse order to avoid position shifts
            let mut positions: Vec<_> = replacements.keys().cloned().collect();
            positions.sort_by(|a, b| b.cmp(a));
            
            for pos in positions {
                let (len, replacement) = replacements.get(&pos).unwrap();
                result.replace_range(pos..pos + len, replacement);
            }
        }
        
        result
    }
    
    /// Fix doubled formatting tags like <i><i>...</i></i>
    pub fn fix_doubled_formatting_tags(text: &str) -> String {
        let mut result = text.to_string();
        
        // Fix doubled italic tags
        let doubled_italic_regex = Regex::new(r"<i><i>([^<]*)</i></i>").unwrap();
        result = doubled_italic_regex.replace_all(&result, "<i>$1</i>").to_string();
        
        // Fix doubled bold tags
        let doubled_bold_regex = Regex::new(r"<b><b>([^<]*)</b></b>").unwrap();
        result = doubled_bold_regex.replace_all(&result, "<b>$1</b>").to_string();
        
        // Fix doubled underline tags
        let doubled_underline_regex = Regex::new(r"<u><u>([^<]*)</u></u>").unwrap();
        result = doubled_underline_regex.replace_all(&result, "<u>$1</u>").to_string();
        
        result
    }
} 