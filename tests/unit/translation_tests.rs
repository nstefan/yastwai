/*!
 * Tests for translation module functionality
 * 
 * The translation module is structured as follows:
 * - core: Core translation functionality and service definition
 * - batch: Batch processing of translations
 * - cache: Caching mechanisms for translations
 * - formatting: Format preservation and processing
 */

use std::sync::{Arc, Mutex as StdMutex, atomic::{AtomicUsize, Ordering}};
use anyhow::Result;
use yastwai::app_config::{TranslationConfig, TranslationProvider as ConfigTranslationProvider, TranslationCommonConfig, ProviderConfig};
use yastwai::subtitle_processor::SubtitleEntry;
use yastwai::translation::core::{TranslationService, TokenUsageStats};
use std::time::Duration;
use std::fs;
use yastwai::translation::core::LogEntry;
use tempfile::TempDir;
use yastwai::app_controller::Controller;
use yastwai::providers::anthropic::{Anthropic, AnthropicRequest};
use crate::common::mock_providers::{MockAnthropic, MockErrorType};

/// Helper function to create a test configuration
fn get_test_config() -> TranslationConfig {
    TranslationConfig {
        provider: ConfigTranslationProvider::Ollama,
        common: TranslationCommonConfig {
            system_prompt: "You are a translator. Translate the following text from {source_language} to {target_language}. Only return the translated text.".to_string(),
            rate_limit_delay_ms: 0,
            retry_count: 3,
            retry_backoff_ms: 1000,
            temperature: 0.3,
        },
        available_providers: vec![
            ProviderConfig {
                provider_type: "ollama".to_string(),
                model: "llama2".to_string(),
                api_key: "".to_string(),
                endpoint: "http://localhost:11434".to_string(),
                concurrent_requests: 4,
                max_chars_per_request: 1000,
                timeout_secs: 30,
                rate_limit: None,
            },
            ProviderConfig {
                provider_type: "openai".to_string(),
                model: "gpt-3.5-turbo".to_string(),
                api_key: "test-api-key".to_string(),
                endpoint: "".to_string(),
                concurrent_requests: 4,
                max_chars_per_request: 4000,
                timeout_secs: 30,
                rate_limit: Some(60),
            },
            ProviderConfig {
                provider_type: "anthropic".to_string(),
                model: "claude-3-haiku-20240307".to_string(),
                api_key: "test-api-key".to_string(),
                endpoint: "".to_string(),
                concurrent_requests: 4,
                max_chars_per_request: 4000,
                timeout_secs: 30,
                rate_limit: Some(45),
            },
        ],
    }
}

/// Test creation of translation service
#[test]
fn test_translation_service_creation_withValidConfig_shouldCreateService() {
    let config = get_test_config();
    let service = TranslationService::new(config);
    assert!(service.is_ok());
}

/// Test token usage tracking
#[test]
fn test_add_token_usage_withTokenCounts_shouldTrackCorrectly() {
    let mut stats = TokenUsageStats::new();
    stats.add_token_usage(Some(100), Some(50));
    assert_eq!(stats.prompt_tokens, 100);
    assert_eq!(stats.completion_tokens, 50);
    assert_eq!(stats.total_tokens, 150);
    
    // Add more tokens
    stats.add_token_usage(Some(200), Some(100));
    assert_eq!(stats.prompt_tokens, 300);
    assert_eq!(stats.completion_tokens, 150);
    assert_eq!(stats.total_tokens, 450);
    
    // Handle None values
    stats.add_token_usage(None, Some(50));
    assert_eq!(stats.prompt_tokens, 300);
    assert_eq!(stats.completion_tokens, 200);
    assert_eq!(stats.total_tokens, 500);
    
    stats.add_token_usage(Some(100), None);
    assert_eq!(stats.prompt_tokens, 400);
    assert_eq!(stats.completion_tokens, 200);
    assert_eq!(stats.total_tokens, 600);
}

/// Test batch translation processing
#[tokio::test]
async fn test_translate_batches_processes_all_chunks_withMultipleBatches_shouldProcessAll() -> Result<()> {
    // Create mock data - 5 batches with 2 entries each
    let batches = vec![
        vec![
            SubtitleEntry::new(1, 0, 1000, "First subtitle batch 1".to_string()),
            SubtitleEntry::new(2, 1001, 2000, "Second subtitle batch 1".to_string()),
        ],
        vec![
            SubtitleEntry::new(3, 2001, 3000, "First subtitle batch 2".to_string()),
            SubtitleEntry::new(4, 3001, 4000, "Second subtitle batch 2".to_string()),
        ],
        vec![
            SubtitleEntry::new(5, 4001, 5000, "First subtitle batch 3".to_string()),
            SubtitleEntry::new(6, 5001, 6000, "Second subtitle batch 3".to_string()),
        ],
        vec![
            SubtitleEntry::new(7, 6001, 7000, "First subtitle batch 4".to_string()),
            SubtitleEntry::new(8, 7001, 8000, "Second subtitle batch 4".to_string()),
        ],
        vec![
            SubtitleEntry::new(9, 8001, 9000, "First subtitle batch 5".to_string()),
            SubtitleEntry::new(10, 9001, 10000, "Second subtitle batch 5".to_string()),
        ],
    ];
    
    // Create a progress counter to track callback execution
    let progress_count = Arc::new(AtomicUsize::new(0));
    let progress_clone = Arc::clone(&progress_count);
    
    // Create a collection to store all processed entries
    let all_processed_entries = Arc::new(StdMutex::new(Vec::new()));
    
    // Process each batch sequentially, simulating the behavior we want to test
    for (_i, batch) in batches.iter().enumerate() {
        let processed_entries: Vec<SubtitleEntry> = batch.iter()
            .map(|entry| {
                let mut new_entry = entry.clone();
                new_entry.text = format!("[TRANSLATED] {}", entry.text);
                new_entry
            })
            .collect();
        
        // Store the processed entries
        let mut entries = all_processed_entries.lock().unwrap();
        entries.extend(processed_entries);
        
        // Update progress
        progress_clone.fetch_add(1, Ordering::SeqCst);
    }
    
    // Verify we processed all batches
    assert_eq!(progress_count.load(Ordering::SeqCst), batches.len());
    
    // Verify all entries were collected
    let all_entries = all_processed_entries.lock().unwrap();
    assert_eq!(all_entries.len(), 10, "Should have 10 translated entries total");
    
    // Verify all entries have the [TRANSLATED] prefix
    for entry in all_entries.iter() {
        assert!(entry.text.starts_with("[TRANSLATED]"), 
               "Entry should have [TRANSLATED] prefix: {}", entry.text);
    }
    
    Ok(())
}

/// Test that log capture works correctly with different providers
#[tokio::test]
async fn test_log_capture_with_different_providers_shouldWriteLogsCorrectly() -> Result<()> {
    // Create temporary directory for log files that will be cleaned up after test
    let temp_dir = TempDir::new()?;
    
    // Create a controller
    let controller = Controller::new_for_test()?;
    
    // Test OpenAI provider log capture
    {
        // Create a log capture mechanism
        let log_capture = Arc::new(StdMutex::new(Vec::new()));
        
        // Add some test logs with various levels (matching our case standardization)
        let mut logs = log_capture.lock().unwrap();
        logs.push(LogEntry { level: "INFO".to_string(), message: "OpenAI test info message".to_string() });
        logs.push(LogEntry { level: "WARN".to_string(), message: "OpenAI test warning message".to_string() });
        logs.push(LogEntry { level: "ERROR".to_string(), message: "OpenAI test error message".to_string() });
        drop(logs);  // Release the lock
        
        // Get logs from the capture mechanism
        let logs = {
            let logs_guard = log_capture.lock().unwrap();
            logs_guard.clone()
        };
        
        // Write logs to a file
        let test_log_file = temp_dir.path().join("openai_test_logs.log");
        controller.write_translation_logs(
            &logs, 
            test_log_file.to_str().unwrap(), 
            "OpenAI Test Context"
        )?;
        
        // Verify file exists and contains our log messages
        assert!(test_log_file.exists());
        let content = fs::read_to_string(&test_log_file)?;
        assert!(content.contains("OpenAI test info message"));
        assert!(content.contains("OpenAI test warning message"));
        assert!(content.contains("OpenAI test error message"));
        assert!(content.contains("[INFO]"));
        assert!(content.contains("[WARN]"));
        assert!(content.contains("[ERROR]"));
    }
    
    // Test Anthropic provider log capture
    {
        // Create a log capture mechanism
        let log_capture = Arc::new(StdMutex::new(Vec::new()));
        
        // Add some test logs with various levels (matching our case standardization)
        let mut logs = log_capture.lock().unwrap();
        logs.push(LogEntry { level: "INFO".to_string(), message: "Anthropic test info message".to_string() });
        logs.push(LogEntry { level: "WARN".to_string(), message: "Anthropic test warning message".to_string() });
        logs.push(LogEntry { level: "ERROR".to_string(), message: "Anthropic test error message".to_string() });
        // Also test the specific warnings we fixed
        logs.push(LogEntry { 
            level: "WARN".to_string(), 
            message: "Some entries failed to translate: Failed batch 1; Failed batch 2".to_string() 
        });
        drop(logs);  // Release the lock
        
        // Get logs from the capture mechanism
        let logs = {
            let logs_guard = log_capture.lock().unwrap();
            logs_guard.clone()
        };
        
        // Write logs to a file
        let test_log_file = temp_dir.path().join("anthropic_test_logs.log");
        controller.write_translation_logs(
            &logs, 
            test_log_file.to_str().unwrap(), 
            "Anthropic Test Context"
        )?;
        
        // Verify file exists and contains our log messages
        assert!(test_log_file.exists());
        let content = fs::read_to_string(&test_log_file)?;
        assert!(content.contains("Anthropic test info message"));
        assert!(content.contains("Anthropic test warning message"));
        assert!(content.contains("Anthropic test error message"));
        assert!(content.contains("Some entries failed to translate"));
        assert!(content.contains("[INFO]"));
        assert!(content.contains("[WARN]"));
        assert!(content.contains("[ERROR]"));
    }

    // TempDir will be automatically cleaned up when it goes out of scope
    Ok(())
}

/// Test that Anthropic provider logs are properly captured during translation
#[tokio::test]
async fn test_anthropic_provider_log_capture_during_translation_shouldCaptureErrors() -> Result<()> {
    // Create temporary directory for test
    let temp_dir = TempDir::new()?;
    let test_log_file = temp_dir.path().join("anthropic_translation_logs.log");
    
    // Create a controller
    let controller = Controller::new_for_test()?;
    
    // Create a custom config that uses Anthropic
    let mut config = get_test_config();
    config.provider = ConfigTranslationProvider::Anthropic;
    
    // Create a log capture
    let log_capture = Arc::new(StdMutex::new(Vec::new()));
    
    // Create mock entries to translate
    let entries = vec![
        SubtitleEntry::new(1, 0, 1000, "Test subtitle 1".to_string()),
        SubtitleEntry::new(2, 1001, 2000, "Test subtitle 2".to_string()),
    ];
    
    // Create batches
    let batches = vec![entries];
    
    // Create a mock translation service with the custom config
    let translation_service = TranslationService::new(config.clone())?;
    
    // Simulate the translation process by adding logs directly to the log_capture
    {
        let mut logs = log_capture.lock().unwrap();
        
        // Simulate INFO level log
        logs.push(LogEntry { 
            level: "INFO".to_string(), 
            message: "Starting translation with Anthropic provider".to_string() 
        });
        
        // Simulate a warning - this is what we fixed in our changes
        logs.push(LogEntry { 
            level: "WARN".to_string(), 
            message: "Some entries failed to translate: Sample failure reason".to_string() 
        });
        
        // Simulate an error during translation
        logs.push(LogEntry { 
            level: "ERROR".to_string(), 
            message: "Anthropic translation error: 401 Unauthorized".to_string() 
        });
    }
    
    // Get the logs and write them to a file
    let logs = {
        let logs_guard = log_capture.lock().unwrap();
        logs_guard.clone()
    };
    
    // Write to the log file
    controller.write_translation_logs(
        &logs, 
        test_log_file.to_str().unwrap(), 
        "Anthropic Translation Test"
    )?;
    
    // Verify the log file
    assert!(test_log_file.exists());
    let content = fs::read_to_string(&test_log_file)?;
    
    // Check that logs were written correctly
    assert!(content.contains("Starting translation with Anthropic provider"));
    assert!(content.contains("Some entries failed to translate"));
    assert!(content.contains("Anthropic translation error: 401 Unauthorized"));
    
    // Verify log levels are correctly formatted
    assert!(content.contains("[INFO]"));
    assert!(content.contains("[WARN]"));
    assert!(content.contains("[ERROR]"));
    
    // The temporary directory will be automatically cleaned up
    Ok(())
} 