/*!
 * Integration tests for subtitle processing workflow
 */

use std::path::PathBuf;
use anyhow::Result;
use tokio_test;

use yastwai::subtitle_processor::SubtitleCollection;
use yastwai::file_utils::FileManager;
use crate::common;

/// Test that we can load, modify, and save subtitles in a full workflow
#[test]
fn test_subtitle_workflow_withFullProcess_shouldSucceed() -> Result<()> {
    // Create a temporary directory and subtitle file for testing
    let temp_dir = common::create_temp_dir()?;
    let subtitle_path = common::create_test_subtitle(&temp_dir.path().to_path_buf(), "test.srt")?;
    
    // 1. Load the subtitle file content
    let content = FileManager::read_to_string(&subtitle_path)?;
    
    // 2. Parse the subtitle content
    let entries = SubtitleCollection::parse_srt_string(&content)?;
    
    // 3. Create a subtitle collection
    let mut collection = SubtitleCollection {
        source_file: subtitle_path.clone(),
        entries,
        source_language: "en".to_string(),
    };
    
    // 4. Modify the subtitles (simulate translation)
    for entry in collection.entries.iter_mut() {
        let new_text = format!("Translated: {}", entry.text);
        entry.text = new_text;
    }
    
    // 5. Save to a new file
    let output_path = temp_dir.path().join("test.translated.srt");
    collection.save_to_file(&output_path)?;
    
    // 6. Verify the new file exists and has the expected content
    assert!(output_path.exists(), "Output file should exist");
    
    // 7. Load the translated file and verify content
    let translated_content = FileManager::read_to_string(&output_path)?;
    let translated_entries = SubtitleCollection::parse_srt_string(&translated_content)?;
    let translated_collection = SubtitleCollection {
        source_file: output_path,
        entries: translated_entries,
        source_language: "en".to_string(),
    };
    
    assert_eq!(translated_collection.entries.len(), 3, "Translated collection should have 3 entries");
    
    // Verify the first entry has the expected translated text
    let first_entry = &translated_collection.entries[0];
    assert!(first_entry.text.starts_with("Translated:"), 
            "First entry should start with 'Translated:' but was: {}", first_entry.text);
    
    Ok(())
}

/// Test that we can handle errors correctly in the workflow
#[test]
fn test_subtitle_workflow_withInvalidInput_shouldHandleErrors() -> Result<()> {
    // Try to load a non-existent file
    let non_existent_path = PathBuf::from("non_existent_file.srt");
    let result = FileManager::read_to_string(&non_existent_path);
    
    // Verify proper error handling
    assert!(result.is_err(), "Loading non-existent file should return error");
    
    Ok(())
} 