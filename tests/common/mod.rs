/*!
 * Common test utilities for the yastwai test suite
 */

use std::path::PathBuf;
use std::fs;
use anyhow::Result;
use tempfile::TempDir;

// Re-export the mock providers module
pub mod mock_providers;

/// Creates a temporary directory for test files
pub fn create_temp_dir() -> Result<TempDir> {
    Ok(TempDir::new()?)
}

/// Creates a test file with the given content in the specified directory
pub fn create_test_file(dir: &PathBuf, filename: &str, content: &str) -> Result<PathBuf> {
    let file_path = dir.join(filename);
    fs::write(&file_path, content)?;
    Ok(file_path)
}

/// Creates a sample subtitle file for testing
pub fn create_test_subtitle(dir: &PathBuf, filename: &str) -> Result<PathBuf> {
    let content = r#"1
00:00:01,000 --> 00:00:04,000
This is a test subtitle.

2
00:00:05,000 --> 00:00:09,000
It contains multiple entries.

3
00:00:10,000 --> 00:00:14,000
For testing purposes.
"#;
    create_test_file(dir, filename, content)
}

/// Helper to get the absolute path to a test resource
pub fn test_resource_path(relative_path: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("resources");
    path.push(relative_path);
    path
} 