/*!
 * Integration tests for subtitle processing workflow
 */

use std::path::PathBuf;
use anyhow::Result;
use tokio_test;

use yastwai::subtitle_processor::{SubtitleCollection, SubtitleEntry};
use yastwai::file_utils::FileManager;
use crate::common;

/// Test that we can load, modify, and save subtitles in a full workflow
#[test]
fn test_subtitle_workflow_withFullProcess_shouldSucceed() -> Result<()> {
    // Create a temporary directory for testing
    let temp_dir = common::create_temp_dir()?;
    
    // Create a subtitle collection manually
    let mut collection = SubtitleCollection::new(
        temp_dir.path().join("source.mkv"),
        "en".to_string()
    );
    
    // Add some entries
    collection.entries.push(SubtitleEntry::new(
        1, 0, 5000, "First subtitle".to_string()
    ));
    collection.entries.push(SubtitleEntry::new(
        2, 5500, 10000, "Second subtitle".to_string()
    ));
    collection.entries.push(SubtitleEntry::new(
        3, 10500, 15000, "Third subtitle".to_string()
    ));
    
    // Save to a file
    let subtitle_path = temp_dir.path().join("test.srt");
    collection.write_to_srt(&subtitle_path)?;
    
    // 1. Load the subtitle file content
    let content = FileManager::read_to_string(&subtitle_path)?;
    
    // 2. Parse the subtitle content
    let entries = SubtitleCollection::parse_srt_string(&content)?;
    
    // Verify that we have entries
    assert!(!entries.is_empty(), "Should have parsed subtitle entries");
    assert_eq!(entries.len(), 3, "Should have 3 subtitle entries");
    
    // 3. Create a subtitle collection
    let mut translated_collection = SubtitleCollection {
        source_file: subtitle_path.clone(),
        entries,
        source_language: "en".to_string(),
    };
    
    // 4. Modify the subtitles (simulate translation)
    for entry in translated_collection.entries.iter_mut() {
        let new_text = format!("Translated: {}", entry.text);
        entry.text = new_text;
    }
    
    // 5. Save to a new file
    let output_path = temp_dir.path().join("test.translated.srt");
    translated_collection.write_to_srt(&output_path)?;
    
    // 6. Verify the new file exists and has the expected content
    assert!(output_path.exists(), "Output file should exist");
    
    // 7. Load the translated file and verify content
    let translated_content = FileManager::read_to_string(&output_path)?;
    
    // Print the content for debugging
    println!("Translated content: {}", translated_content);
    
    // 8. Parse the translated content
    let translated_entries = SubtitleCollection::parse_srt_string(&translated_content)?;
    
    // Verify that we have entries after parsing the translated content
    assert!(!translated_entries.is_empty(), "Should have parsed translated subtitle entries");
    assert_eq!(translated_entries.len(), 3, "Should have 3 translated subtitle entries");
    
    // 9. Create a new collection from the translated entries
    let final_collection = SubtitleCollection {
        source_file: output_path,
        entries: translated_entries,
        source_language: "en".to_string(),
    };
    
    // 10. Verify the translated entries
    assert_eq!(final_collection.entries.len(), translated_collection.entries.len(), 
               "Translated collection should have the same number of entries as original");
    
    // Verify the first entry has the expected translated text
    let first_entry = &final_collection.entries[0];
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