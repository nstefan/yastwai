use std::fs;
use std::fs::File;
use std::fmt;
use regex::Regex;
use once_cell::sync::Lazy;
use anyhow::{Result, Context, anyhow};
use std::io::Write;
use std::path::{Path, PathBuf};
use log::{error, warn, info, debug};
use std::process::Command;
use serde_json::{Value, from_str};
use crate::app_config::SubtitleInfo;
use crate::language_utils;
use serde::{Deserialize, Serialize};
use tempfile::NamedTempFile;
use std::collections::HashMap;

// @module: Subtitle processing and manipulation

// @const: SRT timestamp regex
static TIMESTAMP_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(\d{2}):(\d{2}):(\d{2}),(\d{3}) --> (\d{2}):(\d{2}):(\d{2}),(\d{3})").unwrap()
});

// @struct: Single subtitle entry
#[derive(Debug, Clone)]
pub struct SubtitleEntry {
    // @field: Sequence number
    pub seq_num: usize,
    
    // @field: Start time in ms
    pub start_time_ms: u64,
    
    // @field: End time in ms
    pub end_time_ms: u64,
    
    // @field: Subtitle text
    pub text: String,
}

impl SubtitleEntry {
    // @creates: New subtitle entry
    pub fn new(seq_num: usize, start_time_ms: u64, end_time_ms: u64, text: String) -> Self {
        SubtitleEntry {
            seq_num,
            start_time_ms,
            end_time_ms,
            text,
        }
    }
    
    // @creates: Validated subtitle entry
    // @validates: Time range and non-empty text
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
    
    // @returns: Duration in milliseconds
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
                warn!("Language code issue: {}", e);
                source_language.to_string()
            }
        };
        
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
        let _file_name = path.file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| String::from("Unknown file"));
            
        // Create parent directory if needed
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }
        
        // Write to file
        let mut file = File::create(path)
            .with_context(|| format!("Failed to create subtitle file: {}", path.display()))?;
        
        // Write each entry to the file
        for entry in &self.entries {
            write!(file, "{}", entry)?;
        }
        
        Ok(())
    }
    
    /// Split subtitles into chunks for translation
    /// 
    /// This method divides the subtitle entries into chunks that don't exceed the specified 
    /// maximum character count, ensuring that each chunk contains a coherent set of subtitle entries.
    /// The chunks are optimized to maximize batch size while respecting the character limit.
    pub fn split_into_chunks(&self, max_chars_per_chunk: usize) -> Vec<Vec<SubtitleEntry>> {
        if self.entries.is_empty() {
            warn!("No subtitle entries to split into chunks");
            return Vec::new();
        }
        
        // Protect against accidental loss of subtitles - count at the beginning
        let total_entries = self.entries.len();
        
        // Handle unreasonably small max_chars by enforcing a minimum
        let effective_max_chars = max_chars_per_chunk.max(100);
        
        // For Anthropic provider, consider using smaller chunks to improve reliability
        // We can infer this is likely an Anthropic request if the max_chars is very large (>8000)
        let is_likely_anthropic = max_chars_per_chunk > 8000;
        
        // If this appears to be an Anthropic request, use a more conservative size limit
        // This helps prevent truncated responses by keeping chunks smaller
        let actual_max_chars = if is_likely_anthropic {
            // Use a smaller effective size for Anthropic to improve reliability
            // For Claude-3-Haiku, keep chunks especially small to avoid max_tokens errors
            (effective_max_chars / 3).min(2500)
        } else {
            effective_max_chars
        };
        
        let mut chunks = Vec::new();
        let mut current_chunk = Vec::with_capacity(20); // Pre-allocate with reasonable capacity
        let mut current_size = 0;
        
        for entry in &self.entries {
            let entry_size = entry.text.len();
            
            // If a single entry exceeds the limit, it needs its own chunk
            if entry_size > actual_max_chars {
                // If we have entries in the current chunk, finalize it first
                if !current_chunk.is_empty() {
                    chunks.push(current_chunk);
                    current_chunk = Vec::with_capacity(1);
                    current_size = 0;
                }
                
                // Add the oversized entry as its own chunk
                debug!("Entry {} is oversized ({} chars), placing in its own chunk", 
                       entry.seq_num, entry_size);
                chunks.push(vec![entry.clone()]);
                continue;
            }
            
            // If adding this entry would exceed the limit, finalize the current chunk
            if current_size + entry_size > actual_max_chars && !current_chunk.is_empty() {
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
        
        // Verify that all entries have been included in the chunks
        let total_chunked_entries: usize = chunks.iter().map(|chunk| chunk.len()).sum();
        if total_chunked_entries != total_entries {
            error!("CRITICAL ERROR: Lost entries during chunking! Original: {}, After chunking: {}", 
                   total_entries, total_chunked_entries);
        } else {
            // Add detailed chunk information in debug mode
            if log::max_level() >= log::LevelFilter::Debug {
                for (i, chunk) in chunks.iter().enumerate() {
                    let chunk_seq_nums: Vec<usize> = chunk.iter().map(|e| e.seq_num).collect();
                    let chunk_chars: usize = chunk.iter().map(|e| e.text.len()).sum();
                    debug!("Chunk {}: {} entries (seq_nums: {:?}, total {} chars)", 
                           i+1, chunk.len(), chunk_seq_nums, chunk_chars);
                }
            }
        }
        
        chunks
    }
    
    /// List subtitle tracks in a video file
    pub fn list_subtitle_tracks<P: AsRef<Path>>(video_path: P) -> Result<Vec<SubtitleInfo>> {
        let video_path = video_path.as_ref();
        
        // Check if the file exists
        if !video_path.exists() {
            error!(" Video file not found: {:?}", video_path);
            return Err(anyhow::anyhow!("Video file not found: {:?}", video_path));
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
            // Removed: warn!("ffprobe returned empty output");
            return Ok(Vec::new());
        }
        
        let json: Value = from_str(&stdout)
            .context("Failed to parse ffprobe JSON output")?;
        
        let mut tracks = Vec::new();
        
        if let Some(streams) = json.get("streams").and_then(|s| s.as_array()) {
            // Instead of logging each stream detail, just log the total count
            // Removed: warn!("Found {} subtitle streams", streams.len());
            
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
                
                tracks.push(track);
            }
        } else {
            // Removed: warn!("No subtitle streams found in video");
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
                        return Some(track.index);
                    }
                }
                
                // Also check for language code in title
                let title_lower = title.to_lowercase();
                if title_lower.contains(&preferred_language.to_lowercase()) {
                    return Some(track.index);
                }
            }
        }
        
        // Try to find English if preferred language not found (using ISO codes)
        if !language_utils::language_codes_match(preferred_language, "en") {
            for track in tracks {
                if let Some(lang) = &track.language {
                    if language_utils::language_codes_match(lang, "en") {
                        return Some(track.index);
                    }
                }
                
                // Also check title for English mention
                if let Some(title) = &track.title {
                    if title.to_lowercase().contains("english") {
                        return Some(track.index);
                    }
                }
            }
        }
        
        // If neither preferred nor English found, use the first track
        if !tracks.is_empty() {
            let first_track = tracks.first().unwrap().index;
            return Some(first_track);
        }
        
        None
    }
    
    /// Extract subtitles from a video file with automatic track selection
    pub fn extract_with_auto_track_selection<P: AsRef<Path>>(
        video_path: P, 
        preferred_language: &str,
        output_path: Option<&Path>,
        source_language: &str
    ) -> Result<Self> {
        let video_path = video_path.as_ref();
        
        // List all subtitle tracks
        let tracks = Self::list_subtitle_tracks(video_path)?;
        
        // Exit early if no subtitle streams found
        if tracks.is_empty() {
            error!("No subtitle streams found in video");
            return Err(anyhow::anyhow!("No subtitle tracks found in the video"));
        }
        
        // Select the subtitle track
        let track_id = Self::select_subtitle_track(&tracks, preferred_language)
            .ok_or_else(|| anyhow::anyhow!("No matching subtitle track found for language: {}", preferred_language))?;
        
        // Extract the selected track
        if let Some(output_path) = output_path {
            Self::extract_from_video(video_path, track_id, source_language, output_path)
        } else {
            // Extract to a temporary file first
            let temp_filename = format!("extracted_subtitle_{}.srt", track_id);
            let temp_path = std::env::temp_dir().join(&temp_filename);
            
            let result = Self::extract_from_video(video_path, track_id, source_language, &temp_path);
            
            // Clean up temporary file
            if temp_path.exists() {
                let _ = std::fs::remove_file(&temp_path);
            }
            
            result
        }
    }

    /// Extract source language subtitle to memory
    pub fn extract_source_language_subtitle_to_memory<P: AsRef<Path>>(video_path: P, source_language: &str) -> Result<Self> {
        let video_path = video_path.as_ref();
        
        error!("Extracting {source_language} subtitles from video (in-memory)");
        
        // Avoiding additional logs by passing directly to extract_with_auto_track_selection
        Self::extract_with_auto_track_selection(video_path, source_language, None, source_language)
    }
    
    /// Fast extraction using ffmpeg subtitle copy
    #[allow(dead_code)]
    pub fn fast_extract_source_subtitles<P: AsRef<Path>>(video_path: P, source_language: &str) -> Result<Self> {
        error!("Fast extracting subtitles directly for language: {}", source_language);
        
        // Call extract_with_auto_track_selection directly
        Self::extract_with_auto_track_selection(video_path, source_language, None, source_language)
    }

    /// Save the subtitle collection to an SRT file
    pub fn save_to_file<P: AsRef<Path>>(&self, file_path: P) -> Result<()> {
        let file_path = file_path.as_ref();
        
        // Ensure the parent directory exists
        if let Some(parent) = file_path.parent() {
            crate::file_utils::FileManager::ensure_dir(parent)?;
        }
        
        let _file_name = file_path.file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| String::from("Unknown file"));
        
        // Keep this log as it's useful for progress tracking
        error!("Writing {} subtitle entries to {}", self.entries.len(), _file_name);
        
        // Convert to string
        let srt_content = self.to_string();
        
        // Write to file
        let mut file = File::create(file_path)
            .with_context(|| format!("Failed to create subtitle file: {}", _file_name))?;
        
        file.write_all(srt_content.as_bytes())
            .with_context(|| format!("Failed to write to subtitle file: {}", _file_name))?;
        
        Ok(())
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
                        error!("Skipping invalid subtitle entry {}: {}", seq_num, e);
                        false
                    }
                }
            } else {
                error!("Skipping empty subtitle entry {}", seq_num);
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
            error!("No valid subtitle entries found in content");
            return Err(anyhow::anyhow!("No valid subtitle entries were found in the SRT content"));
        }
        
        // Sort by start time to ensure correct order
        entries.sort_by_key(|entry| entry.start_time_ms);
        
        // Check for overlapping entries
        let mut overlap_count = 0;
        for i in 0..entries.len().saturating_sub(1) {
            if entries[i].end_time_ms > entries[i+1].start_time_ms {
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
        
        // Split with max 50 characters per chunk (should give 2 chunks)
        // Note that the actual limit used will be 100 due to the minimum enforced in the method
        let chunks = collection.split_into_chunks(50);
        
        // Since 100 is the minimum limit, all entries (total 113 chars) should be spread as follows:
        // First chunk: entries 1 and 2 (46 chars), Second chunk: entry 3 (67 chars)
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0].len(), 2);  // First chunk has 2 entries
        assert_eq!(chunks[1].len(), 1);  // Second chunk has 1 entry
        
        // Split with max 20 characters per chunk
        // Note that the actual limit used will be 100 due to the minimum enforced in the method
        let chunks = collection.split_into_chunks(20);
        
        // With a 100 char minimum limit, it should still split the same way as above
        assert_eq!(chunks.len(), 2);  // Still 2 chunks due to minimum size enforcement
        assert_eq!(chunks[0].len(), 2);  // First chunk has 2 entries (46 chars total)
        assert_eq!(chunks[1].len(), 1);  // Second chunk has 1 entry (67 chars)
        
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