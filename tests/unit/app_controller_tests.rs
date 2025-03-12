/*!
 * Tests for application controller functionality
 */

use std::fs;
use std::path::{Path, PathBuf};
use anyhow::Result;
use yastwai::app_config::{Config, SubtitleInfo};
use yastwai::app_controller::Controller;
use yastwai::file_utils::{FileManager, FileType};
use yastwai::language_utils;
use crate::common;

/// Test creating a controller with the default configuration
#[test]
fn test_new_with_default_config_shouldSucceed() -> Result<()> {
    let controller = Controller::new_for_test()?;
    assert!(!controller.config.source_language.is_empty());
    assert!(!controller.config.target_language.is_empty());
    Ok(())
}

/// Test creating a controller with a specific configuration
#[test]
fn test_with_config_withValidConfig_shouldCreateController() -> Result<()> {
    let config = Config::default();
    let controller = Controller::with_config(config)?;
    assert_eq!(controller.config.source_language, "en");
    assert_eq!(controller.config.target_language, "fr");
    Ok(())
}

/// Test creating a controller for testing
#[test]
fn test_new_for_test_shouldCreateController() -> Result<()> {
    let _controller = Controller::new_for_test()?;
    Ok(())
}

/// Test output filename formatting
#[test]
fn test_get_subtitle_output_filename_withVariousInputs_shouldFormatCorrectly() -> Result<()> {
    let controller = Controller::new_for_test()?;
    
    // Test with different file paths and extensions
    let test_cases = [
        // Video files - should append target language
        ("video.mp4", "fr", "video.fr.srt"),
        ("path/to/video.mkv", "es", "video.es.srt"),
        ("with spaces.mov", "de", "with spaces.de.srt"),
        ("/absolute/path/video.avi", "it", "video.it.srt"),
        
        // SRT files - should replace source language with target language
        ("video.source.en.srt", "fr", "video.source.fr.srt"),
        ("path/to/movie.en.srt", "es", "path/to/movie.es.srt"),
        ("subtitles.with.dots.en.srt", "de", "subtitles.with.dots.de.srt"),
        ("/absolute/path/video.source.en.srt", "it", "/absolute/path/video.source.it.srt"),
        
        // Edge cases for SRT files
        ("single.srt", "fr", "single.fr.srt"), // No language code to replace, should append
    ];
    
    for (input, lang, expected) in test_cases {
        let output = controller.get_subtitle_output_filename(&PathBuf::from(input), lang);
        assert_eq!(output, expected, "Failed for input: {}", input);
    }
    
    Ok(())
}

/// Test language code matching for subtitle tracks
#[test]
fn test_find_target_language_track_withVariousCodes_shouldMatchLanguageCodes() -> Result<()> {
    // Create a controller for testing
    let controller = Controller::new_for_test()?;
    
    // Mock the SubtitleCollection::list_subtitle_tracks function
    // This is a bit tricky without mocking libraries, so we'll test the language matching logic directly
    
    let mut tracks = Vec::new();
    
    // Add tracks with various language codes
    tracks.push(SubtitleInfo {
        index: 0,
        codec_name: "subrip".to_string(),
        language: Some("eng".to_string()),
        title: Some("English".to_string()),
    });
    
    tracks.push(SubtitleInfo {
        index: 1,
        codec_name: "subrip".to_string(),
        language: Some("spa".to_string()),
        title: Some("Spanish".to_string()),
    });
    
    tracks.push(SubtitleInfo {
        index: 2,
        codec_name: "subrip".to_string(),
        language: Some("fre".to_string()),
        title: Some("French".to_string()),
    });
    
    // Test with 2-letter code matching 3-letter code
    let result = language_utils::language_codes_match("es", "spa");
    assert!(result, "2-letter 'es' should match 3-letter 'spa'");
    
    let result = language_utils::language_codes_match("fr", "fre");
    assert!(result, "2-letter 'fr' should match 3-letter 'fre'");
    
    // Test with 3-letter code matching 2-letter code
    let result = language_utils::language_codes_match("spa", "es");
    assert!(result, "3-letter 'spa' should match 2-letter 'es'");
    
    // Test with equivalent 3-letter codes (ISO 639-2/B and ISO 639-2/T)
    let result = language_utils::language_codes_match("fre", "fra");
    assert!(result, "3-letter 'fre' should match 3-letter 'fra'");
    
    Ok(())
}

/// Test writing logs to file
#[test]
fn test_write_logs_to_file_withValidLogs_shouldWriteFormattedLogs() -> Result<()> {
    use yastwai::translation_service::LogEntry;
    
    // Create test controller
    let controller = Controller::new_for_test()?;
    
    // Create a temporary directory for testing
    let temp_dir = common::create_temp_dir()?;
    let test_log_file = temp_dir.path().join("test_controller_issues.log");
    
    // Create test logs
    let logs = vec![
        LogEntry { level: "ERROR".to_string(), message: "Test error message".to_string() },
        LogEntry { level: "WARN".to_string(), message: "Test warning message".to_string() },
        LogEntry { level: "INFO".to_string(), message: "Test info message".to_string() },
    ];
    
    // Write logs to file
    controller.write_logs_to_file(&logs, test_log_file.to_str().unwrap(), "Test Context")?;
    
    // Read the file content
    let content = fs::read_to_string(&test_log_file)?;
    
    // Verify that it contains the log entries
    assert!(content.contains("Test error message"));
    assert!(content.contains("Test warning message"));
    assert!(content.contains("Test info message"));
    
    // Verify format
    assert!(content.contains("=== Translation Session: Test Context ==="));
    assert!(content.contains("Summary: 1 errors, 1 warnings, 1 info messages"));
    assert!(content.contains("ERROR: Test error message"));
    assert!(content.contains("WARNING: Test warning message"));
    assert!(content.contains("INFO: Test info message"));
    assert!(content.contains("=== End of Translation Session: Test Context ==="));
    
    Ok(())
}

/// Test direct subtitle file processing
#[test]
fn test_run_with_direct_subtitle_file_shouldSkipExtraction() -> Result<()> {
    // This is a minimal test that doesn't actually run the async code,
    // but ensures the file type detection and special handling path exists
    
    // Create a test controller
    let controller = Controller::new_for_test()?;
    
    // Create a temporary test directory and files
    let temp_dir = common::create_temp_dir()?;
    let input_file = common::create_test_subtitle(&temp_dir.path().to_path_buf(), "test.srt")?;
    
    // We don't actually run the async code, just ensure the code path exists
    // and basic file operations work
    assert!(FileManager::detect_file_type(&input_file)? == FileType::Subtitle);
    
    Ok(())
} 