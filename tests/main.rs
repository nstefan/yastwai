/*!
 * Test runner for YASTwAI
 * 
 * This file configures the test environment and imports all test modules.
 */

// Configure logging for tests
use std::sync::Once;

// Initialize logging once
static INIT: Once = Once::new();

// Setup function that runs before tests
fn setup() {
    INIT.call_once(|| {
        // Enable logging based on environment variable
        if std::env::var("YASTWAI_TEST_LOG").is_ok() {
            env_logger::init();
        }
    });
}

// Common test utilities
mod common;

// Unit tests
mod unit {
    // Core module tests
    pub mod app_config_tests;
    pub mod file_utils_tests;
    pub mod subtitle_processor_tests;
    pub mod translation_service_tests;
    pub mod app_controller_tests;
    pub mod language_utils_tests;
    pub mod progress_bar_tests;
    
    // Provider tests
    pub mod providers_tests;
    pub mod providers;
}

// Integration tests
mod integration {
    // Integration test modules
    pub mod subtitle_workflow_tests;
    pub mod provider_api_tests;
    pub mod app_lifecycle_tests;
}

// Main test setup
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_setup() {
        // Initialize the test environment
        setup();
        assert!(true, "Test environment setup completed");
    }
} 