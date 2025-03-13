/*!
 * Main test entry point for yastwai test suite
 */

// Import common test utilities
pub mod common;

// Import unit tests
mod unit {
    // File and folder related tests
    pub mod file_utils_tests;
    
    // Language utilities tests
    pub mod language_utils_tests;
    
    // Subtitle processing tests
    pub mod subtitle_processor_tests;
    
    // Translation service tests
    pub mod translation_service_tests;
    
    // App configuration tests
    pub mod app_config_tests;
    
    // UI and progress bar tests
    pub mod progress_bar_tests;
    
    // Provider implementation tests
    pub mod providers_tests;
}

// Import integration tests
mod integration {
    // End-to-end subtitle processing tests
    pub mod subtitle_workflow_tests;
    
    // Provider API integration tests
    pub mod provider_api_tests;
    
    // Full app lifecycle tests
    pub mod app_lifecycle_tests;
} 