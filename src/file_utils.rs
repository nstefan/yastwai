use anyhow::{Result, Context};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use std::fs::OpenOptions;
use std::io::Write;
use chrono::Local;

// @module: File and directory utilities

// @struct: File operations utility
pub struct FileManager;

impl FileManager {
    // @checks: File existence
    pub fn file_exists<P: AsRef<Path>>(path: P) -> bool {
        path.as_ref().exists() && path.as_ref().is_file()
    }
    
    // @checks: Directory existence
    pub fn dir_exists<P: AsRef<Path>>(path: P) -> bool {
        path.as_ref().exists() && path.as_ref().is_dir()
    }
    
    // @creates: Directory and parents if needed
    pub fn ensure_dir<P: AsRef<Path>>(path: P) -> Result<()> {
        let path = path.as_ref();
        if !path.exists() {
            fs::create_dir_all(path)?;
        }
        Ok(())
    }
    
    // @generates: Output path for translated subtitle
    // @params: input_file, output_dir, target_language, extension
    pub fn generate_output_path<P1: AsRef<Path>, P2: AsRef<Path>>(
        input_file: P1,
        output_dir: P2,
        target_language: &str,
        extension: &str,
    ) -> PathBuf {
        let input_file = input_file.as_ref();
        let output_dir = output_dir.as_ref();
        
        // Get the file stem (filename without extension)
        let stem = input_file.file_stem().unwrap_or_default();
        
        // Create the output filename with language code and extension
        let mut output_filename = stem.to_string_lossy().to_string();
        output_filename.push('.');
        output_filename.push_str(target_language);
        output_filename.push('.');
        output_filename.push_str(extension);
        
        // Join with the output directory
        output_dir.join(output_filename)
    }
    
    /// Find files with a specific extension in a directory
    pub fn find_files<P: AsRef<Path>>(dir: P, extension: &str) -> Result<Vec<PathBuf>> {
        let mut result = Vec::new();
        let normalized_ext = if extension.starts_with('.') {
            extension.to_string()
        } else {
            format!(".{}", extension)
        };
        
        for entry in WalkDir::new(dir.as_ref()).follow_links(true) {
            let entry = entry.context("Failed to read directory entry")?;
            let path = entry.path();
            
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext.to_string_lossy().eq_ignore_ascii_case(&normalized_ext[1..]) {
                        result.push(path.to_path_buf());
                    }
                }
            }
        }
        
        Ok(result)
    }
    
    /// Read a file to a string
    pub fn read_to_string<P: AsRef<Path>>(path: P) -> Result<String> {
        fs::read_to_string(&path)
            .with_context(|| format!("Failed to read file: {:?}", path.as_ref()))
    }
    
    /// Write a string to a file
    pub fn write_to_file<P: AsRef<Path>>(path: P, content: &str) -> Result<()> {
        // Ensure the parent directory exists
        if let Some(parent) = path.as_ref().parent() {
            Self::ensure_dir(parent)?;
        }
        
        fs::write(&path, content)
            .with_context(|| format!("Failed to write to file: {:?}", path.as_ref()))?;
        
        // No need to log every file write operation
        Ok(())
    }
    
    /// Copy a file from one location to another, ensuring the target directory exists
    pub fn copy_file<P1: AsRef<Path>, P2: AsRef<Path>>(from: P1, to: P2) -> Result<()> {
        let from = from.as_ref();
        let to = to.as_ref();
        
        if !from.exists() {
            return Err(anyhow::anyhow!("Source file does not exist: {:?}", from));
        }
        
        // Ensure the target directory exists
        if let Some(parent) = to.parent() {
            Self::ensure_dir(parent)?;
        }
        
        // Perform the copy
        fs::copy(from, to)?;
        
        Ok(())
    }
    
    /// Append content to a log file with timestamp
    pub fn append_to_log_file<P: AsRef<Path>>(path: P, content: &str) -> Result<()> {
        // Get current timestamp
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        
        // Ensure the parent directory exists
        if let Some(parent) = path.as_ref().parent() {
            Self::ensure_dir(parent)?;
        }
        
        // Open file in append mode, create if it doesn't exist
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .with_context(|| format!("Failed to open log file: {:?}", path.as_ref()))?;
        
        // Write content with timestamp
        writeln!(file, "[{}] {}", timestamp, content)
            .with_context(|| format!("Failed to write to log file: {:?}", path.as_ref()))?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use anyhow::Result;

    /// Test file existence check
    #[test]
    fn test_file_exists_with_existing_file_should_return_true() -> Result<()> {
        // Create a temporary test file
        let test_file = "test_file_exists.tmp";
        fs::write(test_file, "test content")?;
        
        // Test that file_exists works correctly
        assert!(FileManager::file_exists(test_file));
        
        // Clean up
        fs::remove_file(test_file)?;
        Ok(())
    }

    /// Test file existence check for non-existent file
    #[test]
    fn test_file_exists_with_non_existent_file_should_return_false() {
        assert!(!FileManager::file_exists("non_existent_file.tmp"));
    }

    /// Test generation of output path
    #[test]
    fn test_generate_output_path_with_valid_inputs_should_create_correct_path() {
        let input_file = Path::new("/tmp/input/video.mkv");
        let output_dir = Path::new("/tmp/output");
        let target_language = "fr";
        let extension = "srt";
        
        let output_path = FileManager::generate_output_path(input_file, output_dir, target_language, extension);
        
        assert_eq!(output_path, Path::new("/tmp/output/video.fr.srt"));
    }

    /// Test directory existence check
    #[test]
    fn test_dir_exists_with_existing_dir_should_return_true() -> Result<()> {
        // Use the current directory which definitely exists
        let current_dir = ".";
        
        // Test that dir_exists works correctly
        assert!(FileManager::dir_exists(current_dir));
        
        Ok(())
    }

    /// Test directory existence check for non-existent directory
    #[test]
    fn test_dir_exists_with_non_existent_dir_should_return_false() {
        assert!(!FileManager::dir_exists("./non_existent_directory_12345"));
    }

    /// Test ensure directory exists
    #[test]
    fn test_ensure_dir_with_non_existent_dir_should_create_it() -> Result<()> {
        // Define a test directory path
        let test_dir = "./test_ensure_dir_tmp";
        
        // Make sure it doesn't exist initially
        if Path::new(test_dir).exists() {
            fs::remove_dir_all(test_dir)?;
        }
        
        // Test ensure_dir creates the directory
        FileManager::ensure_dir(test_dir)?;
        assert!(Path::new(test_dir).exists());
        assert!(Path::new(test_dir).is_dir());
        
        // Clean up
        fs::remove_dir_all(test_dir)?;
        Ok(())
    }

    /// Test finding files with specific extension
    #[test]
    fn test_find_files_with_existing_files_should_return_them() -> Result<()> {
        // Create a temporary test directory
        let test_dir = "./test_find_files_tmp";
        if Path::new(test_dir).exists() {
            fs::remove_dir_all(test_dir)?;
        }
        fs::create_dir_all(test_dir)?;
        
        // Create test files with different extensions
        fs::write(format!("{}/file1.txt", test_dir), "test content")?;
        fs::write(format!("{}/file2.txt", test_dir), "test content")?;
        fs::write(format!("{}/file3.log", test_dir), "test content")?;
        
        // Test find_files
        let txt_files = FileManager::find_files(test_dir, "txt")?;
        assert_eq!(txt_files.len(), 2);
        
        let log_files = FileManager::find_files(test_dir, "log")?;
        assert_eq!(log_files.len(), 1);
        
        // Clean up
        fs::remove_dir_all(test_dir)?;
        Ok(())
    }

    /// Test reading file to string
    #[test]
    fn test_read_to_string_with_valid_file_should_return_content() -> Result<()> {
        // Create a temporary test file
        let test_file = "test_read_file.tmp";
        let content = "Hello, World!";
        fs::write(test_file, content)?;
        
        // Test read_to_string
        let read_content = FileManager::read_to_string(test_file)?;
        assert_eq!(read_content, content);
        
        // Clean up
        fs::remove_file(test_file)?;
        Ok(())
    }

    /// Test writing content to file
    #[test]
    fn test_write_to_file_should_create_file_with_content() -> Result<()> {
        // Define test file
        let test_file = "test_write_file.tmp";
        let content = "Test write content";
        
        // Remove file if it exists
        if Path::new(test_file).exists() {
            fs::remove_file(test_file)?;
        }
        
        // Test write_to_file
        FileManager::write_to_file(test_file, content)?;
        
        // Verify file was created with correct content
        assert!(Path::new(test_file).exists());
        let read_content = fs::read_to_string(test_file)?;
        assert_eq!(read_content, content);
        
        // Clean up
        fs::remove_file(test_file)?;
        Ok(())
    }

    /// Test copying a file
    #[test]
    fn test_copy_file_should_copy_content_correctly() -> Result<()> {
        // Setup source file
        let source_file = "test_copy_source.tmp";
        let dest_file = "test_copy_dest.tmp";
        let content = "Test file content for copying";
        
        // Create source file
        fs::write(source_file, content)?;
        
        // Remove destination if it exists
        if Path::new(dest_file).exists() {
            fs::remove_file(dest_file)?;
        }
        
        // Test copy_file
        FileManager::copy_file(source_file, dest_file)?;
        
        // Verify copied file has correct content
        assert!(Path::new(dest_file).exists());
        let copied_content = fs::read_to_string(dest_file)?;
        assert_eq!(copied_content, content);
        
        // Clean up
        fs::remove_file(source_file)?;
        fs::remove_file(dest_file)?;
        Ok(())
    }

    /// Test appending to log file
    #[test]
    fn test_append_to_log_file_should_append_content_with_timestamp() -> Result<()> {
        // Define test file
        let test_log_file = "test_log_issues.log";
        
        // Ensure the file doesn't exist initially
        if Path::new(test_log_file).exists() {
            fs::remove_file(test_log_file)?;
        }
        
        // Append two log entries
        FileManager::append_to_log_file(test_log_file, "Test log entry 1")?;
        FileManager::append_to_log_file(test_log_file, "Test log entry 2")?;
        
        // Read the file content
        let content = fs::read_to_string(test_log_file)?;
        
        // Verify that it contains both entries and timestamps
        assert!(content.contains("Test log entry 1"));
        assert!(content.contains("Test log entry 2"));
        
        // Should have timestamps in format [YYYY-MM-DD HH:MM:SS]
        assert!(content.contains("] Test log entry 1"));
        assert!(content.contains("] Test log entry 2"));
        
        // Should have two lines
        assert_eq!(content.lines().count(), 2);
        
        // Clean up
        fs::remove_file(test_log_file)?;
        Ok(())
    }
} 