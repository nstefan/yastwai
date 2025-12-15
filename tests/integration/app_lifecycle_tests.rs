/*!
 * Integration tests for application lifecycle
 */

use anyhow::Result;
use tokio_test;
use yastwai::app_controller::Controller;
use yastwai::app_config::Config;
use crate::common;

/// Test the controller initialization with default config
#[test]
fn test_controller_initialization_withDefaultConfig_shouldSucceed() -> Result<()> {
    // Create a controller with test configuration - should succeed without errors
    let controller = Controller::new_for_test()?;
    
    // If we got here, the controller was successfully initialized
    assert!(true);
    
    Ok(())
}

/// Test the controller with custom configuration
#[test]
fn test_controller_with_custom_config_shouldInitializeWithoutErrors() -> Result<()> {
    // Create a custom configuration with non-default languages
    let mut config = Config::default();
    config.source_language = "es".to_string();
    config.target_language = "de".to_string();
    
    // Create a controller with the custom configuration - should succeed
    let controller = Controller::with_config(config.clone())?;
    
    // If we got here, the controller was successfully initialized with custom config
    assert!(true);
    
    Ok(())
}

/// Test dry run functionality
#[test]
fn test_dry_run_withTestData_shouldNotProduceOutput() -> Result<()> {
    // Create a controller with test configuration
    let controller = Controller::new_for_test()?;
    
    // Set up test environment
    let temp_dir = common::create_temp_dir()?;
    let subtitle_path = common::create_test_subtitle(&temp_dir.path().to_path_buf(), "test.srt")?;
    
    // Execute a test run with dry run flag
    let result = tokio_test::block_on(async {
        controller.test_run(
            subtitle_path.clone(), 
            temp_dir.path().to_path_buf(), 
            true // Set dry run flag
        ).await
    });
    
    // Verify the dry run completes successfully
    assert!(result.is_ok(), "Dry run should complete without errors");
    
    // Get the default target language from a default config (normally "fr")
    let default_config = Config::default();
    
    // In a dry run, no output file should be created
    let expected_output = temp_dir.path().join(
        format!("test.{}.srt", default_config.target_language)
    );
    assert!(!expected_output.exists(), "Dry run should not create output file");
    
    Ok(())
} 