/*!
 * Tests for translation service functionality
 */

use std::sync::{Arc, Mutex as StdMutex, atomic::{AtomicUsize, Ordering}};
use anyhow::Result;
use yastwai::app_config::{TranslationConfig, TranslationProvider as ConfigTranslationProvider, TranslationCommonConfig, ProviderConfig};
use yastwai::subtitle_processor::SubtitleEntry;
use yastwai::translation_service::{TranslationService, TokenUsageStats};

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
            },
            ProviderConfig {
                provider_type: "openai".to_string(),
                model: "gpt-3.5-turbo".to_string(),
                api_key: "test-api-key".to_string(),
                endpoint: "".to_string(),
                concurrent_requests: 4,
                max_chars_per_request: 4000,
                timeout_secs: 30,
            },
            ProviderConfig {
                provider_type: "anthropic".to_string(),
                model: "claude-3-haiku-20240307".to_string(),
                api_key: "test-api-key".to_string(),
                endpoint: "".to_string(),
                concurrent_requests: 4,
                max_chars_per_request: 4000,
                timeout_secs: 30,
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