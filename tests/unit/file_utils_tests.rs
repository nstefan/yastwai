/*!
 * Tests for file utility functions
 */

use std::fs;
use std::path::Path;
use anyhow::Result;
use yastwai::file_utils::FileManager;
use crate::common;

/// Test that file_exists returns true for existing files
#[test]
fn test_file_exists_withExistingFile_shouldReturnTrue() -> Result<()> {
    // Create a temporary test file
    let temp_dir = common::create_temp_dir()?;
    let test_file = common::create_test_file(&temp_dir.path().to_path_buf(), "test_file_exists.tmp", "test content")?;
    
    // Test that file_exists works correctly
    assert!(FileManager::file_exists(test_file.to_str().unwrap()));
    
    Ok(())
}

/// Test that file_exists returns false for non-existent files
#[test]
fn test_file_exists_withNonExistentFile_shouldReturnFalse() {
    assert!(!FileManager::file_exists("non_existent_file.tmp"));
}

/// Test that generate_output_path creates the correct path
#[test]
fn test_generate_output_path_withValidInputs_shouldCreateCorrectPath() {
    let input_file = Path::new("/tmp/input/video.mkv");
    let output_dir = Path::new("/tmp/output");
    let target_language = "fr";
    let extension = "srt";
    
    let output_path = FileManager::generate_output_path(input_file, output_dir, target_language, extension);
    
    assert_eq!(output_path, Path::new("/tmp/output/video.fr.srt"));
}

/// Test that dir_exists returns true for existing directories
#[test]
fn test_dir_exists_withExistingDir_shouldReturnTrue() -> Result<()> {
    // Use the current directory which definitely exists
    let current_dir = ".";
    
    // Test that dir_exists works correctly
    assert!(FileManager::dir_exists(current_dir));
    
    Ok(())
}

/// Test that dir_exists returns false for non-existent directories
#[test]
fn test_dir_exists_withNonExistentDir_shouldReturnFalse() {
    assert!(!FileManager::dir_exists("./non_existent_directory_12345"));
}

/// Test that ensure_dir creates directories as needed
#[test]
fn test_ensure_dir_withNonExistentDir_shouldCreateDirectory() -> Result<()> {
    // Create a temporary directory for testing
    let temp_dir = common::create_temp_dir()?;
    let test_subdir = temp_dir.path().join("test_subdir");
    
    // Ensure the subdirectory exists (should create it)
    FileManager::ensure_dir(test_subdir.to_str().unwrap())?;
    
    // Verify the directory was created
    assert!(test_subdir.exists());
    assert!(test_subdir.is_dir());
    
    Ok(())
}

/// Test that read_to_string returns file content correctly
#[test]
fn test_read_to_string_withValidFile_shouldReturnContent() -> Result<()> {
    // Create a temporary test file
    let temp_dir = common::create_temp_dir()?;
    let content = "Hello, World!";
    let test_file = common::create_test_file(&temp_dir.path().to_path_buf(), "test_read_file.tmp", content)?;
    
    // Test read_to_string
    let read_content = FileManager::read_to_string(test_file.to_str().unwrap())?;
    assert_eq!(read_content, content);
    
    Ok(())
}

/// Test that write_to_file creates file with content correctly
#[test]
fn test_write_to_file_withValidInput_shouldCreateFileWithContent() -> Result<()> {
    // Create a temporary directory for testing
    let temp_dir = common::create_temp_dir()?;
    let test_file = temp_dir.path().join("test_write_file.tmp");
    let content = "Test write content";
    
    // Test write_to_file
    FileManager::write_to_file(test_file.to_str().unwrap(), content)?;
    
    // Verify file was created with correct content
    assert!(test_file.exists());
    let read_content = fs::read_to_string(&test_file)?;
    assert_eq!(read_content, content);
    
    Ok(())
}

/// Test that copy_file copies file correctly
#[test]
fn test_copy_file_withValidInput_shouldCopyFileCorrectly() -> Result<()> {
    // Create a temporary directory and test file
    let temp_dir = common::create_temp_dir()?;
    let content = "Test copy content";
    let source_file = common::create_test_file(&temp_dir.path().to_path_buf(), "source.txt", content)?;
    let dest_file = temp_dir.path().join("dest.txt");
    
    // Test copy_file
    FileManager::copy_file(source_file.to_str().unwrap(), dest_file.to_str().unwrap())?;
    
    // Verify destination file was created with correct content
    assert!(dest_file.exists());
    let dest_content = fs::read_to_string(&dest_file)?;
    assert_eq!(dest_content, content);
    
    Ok(())
} 