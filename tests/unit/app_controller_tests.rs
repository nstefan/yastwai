/*!
 * Tests for application controller functionality
 */

use std::fs;
use std::path::PathBuf;
use anyhow::Result;
use yastwai::app_config::Config;
use yastwai::app_controller::Controller;
use yastwai::file_utils::FileManager;
use yastwai::translation::core::LogEntry;
use std::time::Duration;
use tempfile::TempDir;

/// Test creating a controller with the default configuration
#[test]
fn test_new_with_default_config_shouldSucceed() -> Result<()> {
    let controller = Controller::new_for_test()?;
    // Test that the controller was created successfully
    assert!(controller.is_initialized());
    Ok(())
}

/// Test creating a controller with a specific configuration
#[test]
fn test_with_config_withValidConfig_shouldCreateController() -> Result<()> {
    let config = Config::default();
    let controller = Controller::with_config(config)?;
    // Test that the controller was created successfully
    assert!(controller.is_initialized());
    Ok(())
}

/// Test creating a controller for testing
#[test]
fn test_new_for_test_shouldCreateController() -> Result<()> {
    let controller = Controller::new_for_test()?;
    assert!(controller.is_initialized());
    Ok(())
}

/// Tests for log writing functionality
#[test]
fn test_write_logs_to_file_withValidLogs_shouldWriteFormattedLogs() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let test_log_file = temp_dir.path().join("test_logs.log");
    
    // Create a controller
    let controller = Controller::new_for_test()?;
    
    // Create some test logs
    let logs = vec![
        LogEntry { level: "ERROR".to_string(), message: "Test error message".to_string() },
        LogEntry { level: "WARN".to_string(), message: "Test warning message".to_string() },
        LogEntry { level: "INFO".to_string(), message: "Test info message".to_string() },
    ];
    
    // Use the public write_translation_logs method
    controller.write_translation_logs(&logs, test_log_file.to_str().unwrap(), "Test Context")?;
    
    // Verify file was created
    assert!(test_log_file.exists());
    
    // Verify file content contains our log messages
    let content = fs::read_to_string(test_log_file)?;
    assert!(content.contains("Test error message"));
    assert!(content.contains("Test warning message"));
    assert!(content.contains("Test info message"));
    
    Ok(())
}

/// Test that log writing works correctly
#[test]
fn test_controller_format_duration() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let test_log_file = temp_dir.path().join("test_logs.log");
    
    let controller = Controller::new_for_test()?;
    
    // Create a log with a specific message
    let logs = vec![
        LogEntry { level: "INFO".to_string(), message: "Test message".to_string() },
    ];
    
    // Write the logs
    controller.write_translation_logs(&logs, test_log_file.to_str().unwrap(), "Test Context")?;
    
    // Check the file content contains the expected message
    let content = fs::read_to_string(test_log_file)?;
    assert!(content.contains("Test message"));
    assert!(content.contains("Context: Test Context"));
    
    Ok(())
} 