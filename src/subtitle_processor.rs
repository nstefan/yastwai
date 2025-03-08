use std::fs;
use std::fs::File;
use std::fmt;
use regex::Regex;
use anyhow::{Result, Context};
use std::io::Write;
use std::path::{Path, PathBuf};
use log::{info, warn, error};
use once_cell::sync::Lazy;
use std::process::Command;
use serde_json::{Value, from_str};
use crate::app_config::SubtitleInfo;
use crate::language_utils;
use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use tempfile::NamedTempFile;

/// Subtitle processor module
///
/// This module handles subtitle file processing, parsing, and manipulation
/// including extraction from video files, formatting and splitting into chunks.
/// Regex for parsing SRT timestamps
static TIMESTAMP_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(\d{2}):(\d{2}):(\d{2}),(\d{3}) --> (\d{2}):(\d{2}):(\d{2}),(\d{3})").unwrap()
});

/// Represents a single subtitle entry
#[derive(Debug, Clone)]
pub struct SubtitleEntry {
    /// Sequence number
    pub seq_num: usize,
    
    /// Start time in milliseconds
    pub start_time_ms: u64,
    
    /// End time in milliseconds
    pub end_time_ms: u64,
    
    /// Subtitle text content
    pub text: String,
}

impl SubtitleEntry {
    /// Create a new subtitle entry
    pub fn new(seq_num: usize, start_time_ms: u64, end_time_ms: u64, text: String) -> Self {
        SubtitleEntry {
            seq_num,
            start_time_ms,
            end_time_ms,
            text,
        }
    }
    
    /// Create a new subtitle entry with validation
    pub fn new_validated(seq_num: usize, start_time_ms: u64, end_time_ms: u64, text: String) -> Result<Self> {
        // Validate time range
        if end_time_ms <= start_time_ms {
            return Err(anyhow!(
                "Invalid time range: end time {} <= start time {}",
                end_time_ms, start_time_ms
            ));
        }

        // Validate text is not empty (after trimming)
        let trimmed_text = text.trim();
        if trimmed_text.is_empty() {
            return Err(anyhow!("Empty subtitle text for entry {}", seq_num));
        }

        Ok(SubtitleEntry {
            seq_num,
            start_time_ms,
            end_time_ms,
            text: trimmed_text.to_string(),
        })
    }
    
    /// Get the duration of this subtitle entry in milliseconds
    pub fn duration_ms(&self) -> u64 {
        self.end_time_ms.saturating_sub(self.start_time_ms)
    }
    
    /// Check if this subtitle entry overlaps with another
    pub fn overlaps_with(&self, other: &Self) -> bool {
        // Check if either entry's start time falls within the other's time range
        (self.start_time_ms >= other.start_time_ms && self.start_time_ms < other.end_time_ms) ||
        (other.start_time_ms >= self.start_time_ms && other.start_time_ms < self.end_time_ms)
    }
    
    /// Convert start time to formatted SRT timestamp
    pub fn format_start_time(&self) -> String {
        Self::format_timestamp(self.start_time_ms)
    }
    
    /// Convert end time to formatted SRT timestamp
    pub fn format_end_time(&self) -> String {
        Self::format_timestamp(self.end_time_ms)
    }
    
    /// Format a timestamp in milliseconds to SRT format (HH:MM:SS,mmm)
    pub fn format_timestamp(ms: u64) -> String {
        let hours = ms / 3_600_000;
        let minutes = (ms % 3_600_000) / 60_000;
        let seconds = (ms % 60_000) / 1_000;
        let millis = ms % 1_000;
        
        format!("{:02}:{:02}:{:02},{:03}", hours, minutes, seconds, millis)
    }
    
    /// Parse an SRT timestamp to milliseconds
    pub fn parse_timestamp(timestamp: &str) -> Result<u64> {
        // Parse HH:MM:SS,mmm format
        let parts: Vec<&str> = timestamp.split(|c| c == ':' || c == ',' || c == '.').collect();
        
        if parts.len() != 4 {
            return Err(anyhow!("Invalid timestamp format: {}", timestamp));
        }
        
        let hours: u64 = parts[0].parse().context("Failed to parse hours")?;
        let minutes: u64 = parts[1].parse().context("Failed to parse minutes")?;
        let seconds: u64 = parts[2].parse().context("Failed to parse seconds")?;
        let millis: u64 = parts[3].parse().context("Failed to parse milliseconds")?;
        
        // Validate time components
        if minutes >= 60 || seconds >= 60 || millis >= 1000 {
            return Err(anyhow!("Invalid time components in timestamp: {}", timestamp));
        }
        
        Ok(hours * 3_600_000 + minutes * 60_000 + seconds * 1_000 + millis)
    }
    
    /// Escape special characters that might interfere with API communication
    pub fn escape_text(&self) -> String {
        self.text.replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t")
            .replace('\u{0008}', "\\b")  // backspace
            .replace('\u{000C}', "\\f")  // form feed
    }
    
    /// Unescape special characters after receiving translated text
    pub fn unescape_text(text: &str) -> String {
        // Create a mutable buffer for the result with a capacity that's a reasonable estimate
        let mut result = String::with_capacity(text.len());
        let mut chars = text.chars().peekable();
        
        while let Some(ch) = chars.next() {
            if ch == '\\' && chars.peek().is_some() {
                match chars.next().unwrap() {
                    'n' => result.push('\n'),
                    'r' => result.push('\r'),
                    't' => result.push('\t'),
                    'b' => result.push('\u{0008}'),  // backspace
                    'f' => result.push('\u{000C}'),  // form feed
                    '\\' => result.push('\\'),
                    '"' => result.push('"'),
                    // For any other escaped character, just add the character itself
                    c => result.push(c),
                }
            } else {
                result.push(ch);
            }
        }
        
        result
    }
    
    /// Check if this subtitle entry has valid content
    pub fn is_valid(&self) -> bool {
        !self.text.trim().is_empty() && self.end_time_ms > self.start_time_ms
    }
    
    /// Word count in the subtitle text
    pub fn word_count(&self) -> usize {
        self.text.split_whitespace().count()
    }
    
    /// Character count in the subtitle text
    pub fn char_count(&self) -> usize {
        self.text.chars().count()
    }
}

impl fmt::Display for SubtitleEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{}", self.seq_num)?;
        writeln!(f, "{} --> {}", self.format_start_time(), self.format_end_time())?;
        writeln!(f, "{}", self.text)?;
        writeln!(f)
    }
}

/// Collection of subtitle entries with metadata
#[derive(Debug)]
pub struct SubtitleCollection {
    /// Source filename
    pub source_file: PathBuf,
    
    /// List of subtitle entries
    pub entries: Vec<SubtitleEntry>,
    
    /// Source language
    pub source_language: String,
}

impl SubtitleCollection {
    /// Create a new subtitle collection
    pub fn new(source_file: PathBuf, source_language: String) -> Self {
        SubtitleCollection {
            source_file,
            entries: Vec::new(),
            source_language,
        }
    }
    
    /// Extract subtitles from a video file
    pub fn extract_from_video<P: AsRef<Path>>(video_path: P, track_id: usize, source_language: &str, output_path: P) -> Result<Self> {
        let video_path = video_path.as_ref();
        let output_path = output_path.as_ref();
        
        if !video_path.exists() {
            return Err(anyhow!("Video file does not exist: {:?}", video_path));
        }
        
        // Normalize language code if possible, but continue if not
        let normalized_language = match language_utils::normalize_to_part1_or_part2t(source_language) {
            Ok(lang) => lang,
            Err(e) => {
                warn!("⚠️ Language code issue: {}", e);
                source_language.to_string()
            }
        };
        
        info!("Extracting track {} to: {}", track_id, output_path.display());
        
        // Use ffmpeg to extract the subtitle directly to SRT file
        let result = Command::new("ffmpeg")
            .args([
                "-y",                       // Overwrite existing file
                "-i", video_path.to_str().unwrap_or_default(),
                "-map", &format!("0:{}", track_id),
                "-c:s", "srt",              // SRT output format
                output_path.to_str().unwrap_or_default()
            ])
            .output()
            .context("Failed to execute ffmpeg command for subtitle extraction")?;

        if !result.status.success() {
            let stderr = String::from_utf8_lossy(&result.stderr);
            error!(" Subtitle extraction failed: {}", stderr);
            return Err(anyhow!("ffmpeg command failed: {}", stderr));
        }
        
        // If no errors occurred, parse the output file
        let file_size = std::fs::metadata(output_path)?.len();
        if file_size == 0 {
            error!(" No subtitles found");
            return Err(anyhow!("Extracted file is empty - no subtitles found"));
        }
        
        let entries = Self::parse_srt_file(output_path)?;
        if entries.is_empty() {
            error!(" No valid subtitle entries");
            return Err(anyhow!("Failed to parse any subtitle entries from the extracted file"));
        }
                
        Ok(SubtitleCollection {
            source_file: output_path.to_path_buf(),
            entries,
            source_language: normalized_language,
        })
    }
    
    /// Write subtitles to an SRT file
    pub fn write_to_srt<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();
        let file_name = path.file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| String::from("Unknown file"));
        info!("Writing {} subtitle entries to {}", self.entries.len(), file_name);
        
        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }
        
        let mut file = File::create(path)?;
        for entry in &self.entries {
            write!(file, "{}", entry)?;
        }
        
        info!("Subtitles saved");
        Ok(())
    }
    
    /// Split subtitles into chunks for translation
    /// 
    /// This method divides the subtitle entries into chunks that don't exceed the specified 
    /// maximum character count, ensuring that each chunk contains a coherent set of subtitle entries.
    /// The chunks are optimized to maximize batch size while respecting the character limit.
    pub fn split_into_chunks(&self, max_chars: usize) -> Vec<Vec<SubtitleEntry>> {
        // Early return for empty collections
        if self.entries.is_empty() {
            info!("No subtitle entries to split into chunks");
            return Vec::new();
        }
        
        // Handle unreasonably small max_chars by enforcing a minimum
        let effective_max_chars = max_chars.max(10);
        
        let mut chunks = Vec::new();
        let mut current_chunk = Vec::with_capacity(20); // Pre-allocate with reasonable capacity
        let mut current_size = 0;
        
        for entry in &self.entries {
            let entry_size = entry.text.len();
            
            // If a single entry exceeds the limit, it needs its own chunk
            if entry_size > effective_max_chars {
                // If we have entries in the current chunk, finalize it first
                if !current_chunk.is_empty() {
                    chunks.push(current_chunk);
                    current_chunk = Vec::with_capacity(1);
                    current_size = 0;
                }
                
                // Add the oversized entry as its own chunk
                chunks.push(vec![entry.clone()]);
                continue;
            }
            
            // If adding this entry would exceed the limit, finalize the current chunk
            if current_size + entry_size > effective_max_chars && !current_chunk.is_empty() {
                chunks.push(current_chunk);
                current_chunk = Vec::with_capacity(20);
                current_size = 0;
            }
            
            // Add the entry to the current chunk
            current_chunk.push(entry.clone());
            current_size += entry_size;
        }
        
        // Add the last chunk if it's not empty
        if !current_chunk.is_empty() {
            chunks.push(current_chunk);
        }
        
        chunks
    }
    
    /// List subtitle tracks in a video file
    pub fn list_subtitle_tracks<P: AsRef<Path>>(video_path: P) -> Result<Vec<SubtitleInfo>> {
        let video_path = video_path.as_ref();
        
        if !video_path.exists() {
            error!(" Video file not found: {:?}", video_path);
            return Err(anyhow!("Video file does not exist: {:?}", video_path));
        }
        
        // Remove verbose stack trace and unnecessary ffprobe details logs
        let output = Command::new("ffprobe")
            .args([
                "-v", "quiet",
                "-print_format", "json",
                "-show_streams",
                "-select_streams", "s",
                video_path.to_str().unwrap_or("")
            ])
            .output()
            .context("Failed to execute ffprobe command")?;
            
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("ffprobe failed: {}", stderr);
            return Err(anyhow!("ffprobe command failed: {}", stderr));
        }
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        if stdout.trim().is_empty() {
            warn!("ffprobe returned empty output");
            return Ok(Vec::new());
        }
        
        let json: Value = from_str(&stdout)
            .context("Failed to parse ffprobe JSON output")?;
        
        let mut tracks = Vec::new();
        
        if let Some(streams) = json.get("streams").and_then(|s| s.as_array()) {
            // Instead of logging each stream detail, just log the total count
            info!("Found {} subtitle streams", streams.len());
            
            for stream in streams.iter() {
                let index = stream.get("index")
                    .and_then(|v| v.as_u64())
                    .map(|v| v as usize)
                    .unwrap_or(0);
                
                let codec_name = stream.get("codec_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                
                let language = stream.get("tags")
                    .and_then(|t| t.get("language"))
                    .and_then(|l| l.as_str())
                    .map(|s| s.to_string());
                
                let title = stream.get("tags")
                    .and_then(|t| t.get("title"))
                    .and_then(|l| l.as_str())
                    .map(|s| s.to_string());
                
                let track = SubtitleInfo {
                    index,
                    codec_name: codec_name.to_string(),
                    language,
                    title,
                };
                
                // Only log meaningful language information, not internal track details
                if let Some(lang) = &track.language {
                    let lang_name = language_utils::get_language_name(lang).unwrap_or_else(|_| lang.clone());
                    let title_info = track.title.as_deref().map_or(String::new(), |t| format!(" ({})", t));
                    info!("Track {}: {} {}{}", track.index, lang, lang_name, title_info);
                } else if let Some(title) = &track.title {
                    info!("Track {}: {} (unknown language)", track.index, title);
                } else {
                    info!("Track {}: Unknown language", track.index);
                }
                
                tracks.push(track);
            }
        } else {
            info!("No subtitle streams found in video");
        }
        
        Ok(tracks)
    }
    
    /// Select a subtitle track based on preferred language
    pub fn select_subtitle_track(tracks: &[SubtitleInfo], preferred_language: &str) -> Option<usize> {
        if tracks.is_empty() {
            return None;
        }
        
        // Try to find the preferred language - first check for ISO language code match
        for track in tracks {
            if let Some(track_lang) = &track.language {
                // Use language_utils to compare ISO codes (handles both 639-1 and 639-2 codes)
                if language_utils::language_codes_match(track_lang, preferred_language) {             
                    return Some(track.index);
                }
            }
            
            // Also check title for language mention
            if let Some(title) = &track.title {
                // Try to normalize preferred language to get the name
                if let Ok(pref_name) = language_utils::get_language_name(preferred_language) {
                    let title_lower = title.to_lowercase();
                    let name_lower = pref_name.to_lowercase();
                    
                    // Check if the language name is in the title
                    if title_lower.contains(&name_lower) {
                        info!("Found language name '{}' in title: {} (track {})", 
                               pref_name, title, track.index);
                        return Some(track.index);
                    }
                }
                
                // Also check for language code in title
                let title_lower = title.to_lowercase();
                if title_lower.contains(&preferred_language.to_lowercase()) {
                    info!("Found language code in title: {} (track {})", title, track.index);
                    return Some(track.index);
                }
            }
        }
        
        // Try to find English if preferred language not found (using ISO codes)
        if !language_utils::language_codes_match(preferred_language, "en") {
            for track in tracks {
                if let Some(lang) = &track.language {
                    if language_utils::language_codes_match(lang, "en") {
                        info!("Found English fallback: {} (track {})", lang, track.index);
                        return Some(track.index);
                    }
                }
                
                // Also check title for English mention
                if let Some(title) = &track.title {
                    if title.to_lowercase().contains("english") {
                        info!("Found English in title: {} (track {})", title, track.index);
                        return Some(track.index);
                    }
                }
            }
        }
        
        // If neither preferred nor English found, use the first track
        if !tracks.is_empty() {
            let first_track = tracks.first().unwrap().index;
            warn!("No language match found, using first track: {}", first_track);
            info!("Selected track {} for extraction", first_track);
            return Some(first_track);
        }
        
        None
    }
    
    /// Extract subtitles with automatic track selection
    pub fn extract_with_auto_track_selection<P: AsRef<Path>>(
        video_path: P, 
        preferred_language: &str,
        output_path: Option<&Path>,
        source_language: &str
    ) -> Result<Self> {
        let video_path = video_path.as_ref();
        
        // Normalize preferred language code
        let normalized_preferred = match language_utils::normalize_to_part1_or_part2t(preferred_language) {
            Ok(lang) => lang,
            Err(e) => {
                warn!("Could not normalize preferred language code '{}': {}", preferred_language, e);
                preferred_language.to_string()
            }
        };
        
        info!("Starting auto track selection for: {} (preferred: {})", 
               video_path.file_name().unwrap_or_default().to_string_lossy(), normalized_preferred);
        
        // List available subtitle tracks
        let tracks = Self::list_subtitle_tracks(video_path)?;
        
        if tracks.is_empty() {
            error!(" No subtitles found in the video");
            return Err(anyhow::anyhow!("No subtitle tracks found in the video"));
        }
        
        // Select the best track
        let track_id = match Self::select_subtitle_track(&tracks, &normalized_preferred) {
            Some(id) => {
                info!("Selected track {} for extraction", id);
                id
            },
            None => {
                // Use first track if no language match found
                let first_track = tracks.first().unwrap().index;
                warn!("No language match found, using first track: {}", first_track);
                info!("Selected track {} for extraction", first_track);
                first_track
            }
        };
        
        // Extract the subtitles
        if let Some(output_path) = output_path {
            info!("Extracting to file: {:?}", output_path);
            Self::extract_from_video(video_path, track_id, source_language, output_path)
        } else {
            // Extract to a temporary file first
            let temp_filename = format!("extracted_subtitle_{}.srt", track_id);
            let temp_path = std::env::temp_dir().join(&temp_filename);
            
            let result = Self::extract_from_video(video_path, track_id, source_language, &temp_path);
            
            // Clean up temporary file
            if temp_path.exists() {
                if let Err(e) = std::fs::remove_file(&temp_path) {
                    warn!("Failed to remove temporary file: {}", e);
                }
            }
            
            result
        }
    }
    
    /// Extract source language subtitle to memory
    pub fn extract_source_language_subtitle_to_memory<P: AsRef<Path>>(video_path: P, source_language: &str) -> Result<Self> {
        let video_path = video_path.as_ref();
        
        info!("Extracting {source_language} subtitles from video (in-memory)");
        // Avoiding additional logs by passing directly to extract_with_auto_track_selection
        Self::extract_with_auto_track_selection(video_path, source_language, None, source_language)
    }
    
    /// Parse SRT file content to subtitle entries
    fn parse_srt_file(path: &Path) -> Result<Vec<SubtitleEntry>> {
        let content = fs::read_to_string(path)?;
        Self::parse_srt_string(&content)
    }
    
    /// Parse SRT file content to subtitle entries
    fn parse_srt_string(content: &str) -> Result<Vec<SubtitleEntry>> {
        let mut entries = Vec::new();
        let mut lines = content.lines().peekable();
        
        // State variables for parsing
        let mut current_seq_num: Option<usize> = None;
        let mut current_start_time_ms: Option<u64> = None;
        let mut current_end_time_ms: Option<u64> = None;
        let mut current_text = String::new();
        let mut line_count = 0;
        
        // Helper function to add the current entry if complete
        let mut add_current_entry = |seq_num: usize, start_ms: u64, end_ms: u64, text: &str| {
            if !text.trim().is_empty() {
                match SubtitleEntry::new_validated(seq_num, start_ms, end_ms, text.trim().to_string()) {
                    Ok(entry) => {
                        entries.push(entry);
                        true
                    },
                    Err(e) => {
                        warn!("Skipping invalid subtitle entry {}: {}", seq_num, e);
                        false
                    }
                }
            } else {
                warn!("Skipping empty subtitle entry {}", seq_num);
                false
            }
        };
        
        while let Some(line) = lines.next() {
            line_count += 1;
            let trimmed = line.trim();
            
            // Skip empty lines, but check if we need to finalize the current entry
            if trimmed.is_empty() {
                if let (Some(seq_num), Some(start_ms), Some(end_ms)) = (current_seq_num, current_start_time_ms, current_end_time_ms) {
                    if !current_text.is_empty() {
                        add_current_entry(seq_num, start_ms, end_ms, &current_text);
                        
                        // Reset state for next entry
                        current_seq_num = None;
                        current_start_time_ms = None;
                        current_end_time_ms = None;
                        current_text.clear();
                    }
                }
                continue;
            }
            
            // Try to parse as sequence number (only if we're starting a new entry)
            if current_seq_num.is_none() && current_text.is_empty() {
                if let Ok(num) = trimmed.parse::<usize>() {
                    current_seq_num = Some(num);
                    continue;
                }
            }
            
            // Try to parse as timestamp
            if current_seq_num.is_some() && current_start_time_ms.is_none() && current_end_time_ms.is_none() {
                if let Some(caps) = TIMESTAMP_REGEX.captures(trimmed) {
                    match (Self::parse_timestamp_to_ms(&caps, 1), Self::parse_timestamp_to_ms(&caps, 5)) {
                        (Ok(start_ms), Ok(end_ms)) => {
                            current_start_time_ms = Some(start_ms);
                            current_end_time_ms = Some(end_ms);
                            continue;
                        },
                        _ => {
                            // Invalid timestamp format, but we'll treat it as text
                            warn!("Invalid timestamp format at line {}: {}", line_count, trimmed);
                        }
                    }
                }
            }
            
            // If we have a sequence number and timestamps, this must be subtitle text
            if current_seq_num.is_some() && current_start_time_ms.is_some() && current_end_time_ms.is_some() {
                if !current_text.is_empty() {
                    current_text.push('\n');
                }
                current_text.push_str(trimmed);
            } else {
                // We have text but no sequence number or timestamps yet
                // This is likely malformed SRT, but we'll try to recover
                warn!("Unexpected text at line {} before sequence number or timestamp: {}", line_count, trimmed);
            }
        }
        
        // Add the last entry if there is one
        if let (Some(seq_num), Some(start_ms), Some(end_ms)) = (current_seq_num, current_start_time_ms, current_end_time_ms) {
            if !current_text.is_empty() {
                add_current_entry(seq_num, start_ms, end_ms, &current_text);
            }
        }
        
        // Validate and sort entries
        if entries.is_empty() {
            warn!("No valid subtitle entries found in content");
        } else {
            // Sort by start time to ensure correct order
            entries.sort_by_key(|entry| entry.start_time_ms);
            
            // Check for overlapping entries
            let mut overlap_count = 0;
            for i in 0..entries.len().saturating_sub(1) {
                if entries[i].overlaps_with(&entries[i + 1]) {
                    overlap_count += 1;
                }
            }
            
            if overlap_count > 0 {
                warn!("Found {} overlapping subtitle entries", overlap_count);
            }
            
            // Renumber entries to ensure sequential order
            for (i, entry) in entries.iter_mut().enumerate() {
                entry.seq_num = i + 1;
            }
        }
        
        Ok(entries)
    }
    
    /// Parse timestamp to milliseconds
    fn parse_timestamp_to_ms(caps: &regex::Captures, start_idx: usize) -> Result<u64> {
        let hours: u64 = caps.get(start_idx)
            .map_or(0, |m| m.as_str().parse().unwrap_or(0));
        let minutes: u64 = caps.get(start_idx + 1)
            .map_or(0, |m| m.as_str().parse().unwrap_or(0));
        let seconds: u64 = caps.get(start_idx + 2)
            .map_or(0, |m| m.as_str().parse().unwrap_or(0));
        let millis: u64 = caps.get(start_idx + 3)
            .map_or(0, |m| m.as_str().parse().unwrap_or(0));
            
        Ok((hours * 3600 + minutes * 60 + seconds) * 1000 + millis)
    }
    
    /// Fast extract source subtitles
    pub fn fast_extract_source_subtitles<P: AsRef<Path>>(video_path: P, source_language: &str) -> Result<Self> {
        info!("Fast extracting subtitles directly for language: {}", source_language);
        // Call extract_with_auto_track_selection directly instead of going through another method
        Self::extract_with_auto_track_selection(video_path, source_language, None, source_language)
    }
}

impl fmt::Display for SubtitleCollection {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Subtitle Collection")?;
        writeln!(f, "Source: {:?}", self.source_file)?;
        writeln!(f, "Language: {}", self.source_language)?;
        writeln!(f, "Entries: {}", self.entries.len())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use std::path::PathBuf;
    
    #[test]
    fn test_timestamp_parsing() {
        // Helper test function for parsing timestamps
        fn parse_timestamp(timestamp: &str) -> Result<u64> {
            let parts: Vec<&str> = timestamp.split([',', ':']).collect();
            if parts.len() != 4 {
                return Err(anyhow::anyhow!("Invalid timestamp format: {}", timestamp));
            }
            
            let hours: u64 = parts[0].parse().with_context(|| "Invalid hours in timestamp")?;
            let minutes: u64 = parts[1].parse().with_context(|| "Invalid minutes in timestamp")?;
            let seconds: u64 = parts[2].parse().with_context(|| "Invalid seconds in timestamp")?;
            let millis: u64 = parts[3].parse().with_context(|| "Invalid milliseconds in timestamp")?;
            
            Ok(hours * 3_600_000 + minutes * 60_000 + seconds * 1_000 + millis)
        }
        
        let ts = "01:23:45,678";
        let ms = parse_timestamp(ts).unwrap();
        assert_eq!(ms, 5025678);
        
        let formatted = SubtitleEntry::format_timestamp(ms);
        assert_eq!(formatted, ts);
    }

    #[test]
    fn test_subtitle_entry_display() {
        let entry = SubtitleEntry::new(1, 5000, 10000, "Test subtitle".to_string());
        let display = format!("{}", entry);
        
        assert!(display.contains("1"));
        assert!(display.contains("00:00:05,000"));
        assert!(display.contains("00:00:10,000"));
        assert!(display.contains("Test subtitle"));
    }

    #[test]
    fn test_subtitle_entry_properties() {
        let entry = SubtitleEntry::new(
            42,
            61234,
            65432,
            "Hello\nWorld".to_string()
        );
        
        // Check properties
        assert_eq!(entry.seq_num, 42);
        assert_eq!(entry.start_time_ms, 61234);
        assert_eq!(entry.end_time_ms, 65432);
        assert_eq!(entry.text, "Hello\nWorld");
        
        // Check formatting
        assert_eq!(entry.format_start_time(), "00:01:01,234");
        assert_eq!(entry.format_end_time(), "00:01:05,432");
    }

    #[test]
    fn test_in_memory_subtitle_collection() {
        // Create a collection
        let source_file = PathBuf::from("test.mkv");
        let mut collection = SubtitleCollection::new(source_file.clone(), "en".to_string());
        
        // Add some entries
        collection.entries.push(SubtitleEntry::new(
            1, 0, 5000, "First subtitle".to_string()
        ));
        collection.entries.push(SubtitleEntry::new(
            2, 5500, 10000, "Second subtitle".to_string()
        ));
        
        // Check properties
        assert_eq!(collection.source_file, source_file);
        assert_eq!(collection.source_language, "en");
        assert_eq!(collection.entries.len(), 2);
        
        // Check entries
        assert_eq!(collection.entries[0].seq_num, 1);
        assert_eq!(collection.entries[0].text, "First subtitle");
        assert_eq!(collection.entries[1].seq_num, 2);
        assert_eq!(collection.entries[1].text, "Second subtitle");
    }

    #[test]
    fn test_split_into_chunks() -> Result<()> {
        // Create a collection with entries of varying lengths
        let mut collection = SubtitleCollection::new(PathBuf::from("test.mkv"), "en".to_string());
        
        // Add entries with different character counts
        collection.entries.push(SubtitleEntry::new(
            1, 0, 5000, "Short entry".to_string()  // 11 chars
        ));
        collection.entries.push(SubtitleEntry::new(
            2, 5500, 10000, "Medium length entry with some text".to_string()  // 35 chars
        ));
        collection.entries.push(SubtitleEntry::new(
            3, 10500, 15000, "A longer entry that should take more space in the chunk calculation".to_string()  // 67 chars
        ));
        
        // Split with max 50 characters per chunk (should give 3 chunks)
        let chunks = collection.split_into_chunks(50);
        
        // First two entries fit in first chunk (46 chars total), third entry gets its own chunk
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0].len(), 2);  // First chunk has 2 entries
        assert_eq!(chunks[1].len(), 1);  // Second chunk has 1 entry
        
        // Split with max 20 characters per chunk
        let chunks = collection.split_into_chunks(20);
        
        // Each entry should be in its own chunk
        assert_eq!(chunks.len(), 3);
        
        Ok(())
    }

    #[test]
    fn test_extract_source_language_subtitle_to_memory() -> Result<()> {
        // This test requires a real video file with subtitles
        // We're only testing the function structure here
        
        // Mock the video path - this won't actually run in automated tests
        let video_path = PathBuf::from("test_data/sample.mp4");
        if !video_path.exists() {
            // Skip test if test file doesn't exist
            return Ok(());
        }
        
        let source_language = "en";
        let collection = super::SubtitleCollection::extract_source_language_subtitle_to_memory(
            &video_path, 
            source_language
        )?;
        
        assert_eq!(collection.source_language, source_language);
        assert!(!collection.entries.is_empty(), "Should have extracted some subtitle entries");
        
        Ok(())
    }
    
    #[test]
    fn test_parse_srt_string() -> Result<()> {
        let srt_content = "1\n00:00:01,000 --> 00:00:04,000\nHello world\n\n2\n00:00:05,000 --> 00:00:08,000\nTest subtitle\nSecond line\n\n";
        
        let entries = super::SubtitleCollection::parse_srt_string(srt_content)?;
        
        assert_eq!(entries.len(), 2);
        
        assert_eq!(entries[0].seq_num, 1);
        assert_eq!(entries[0].start_time_ms, 1000);
        assert_eq!(entries[0].end_time_ms, 4000);
        assert_eq!(entries[0].text, "Hello world");
        
        assert_eq!(entries[1].seq_num, 2);
        assert_eq!(entries[1].start_time_ms, 5000);
        assert_eq!(entries[1].end_time_ms, 8000);
        assert_eq!(entries[1].text, "Test subtitle\nSecond line");
        
        Ok(())
    }

    #[test]
    fn test_fast_extract_source_subtitles() -> Result<()> {
        // This test demonstrates how to use the fast extraction utility
        // It requires a real video file with subtitles
        
        // Mock the video path - this won't actually run in automated tests
        let video_path = PathBuf::from("test_data/sample.mp4");
        if !video_path.exists() {
            // Skip test if test file doesn't exist
            return Ok(());
        }
        
        let source_language = "en";
        let collection = super::SubtitleCollection::fast_extract_source_subtitles(
            &video_path, 
            source_language
        )?;
        
        assert_eq!(collection.source_language, source_language);
        assert!(!collection.entries.is_empty(), "Should have extracted some subtitle entries");
        
        Ok(())
    }
} 