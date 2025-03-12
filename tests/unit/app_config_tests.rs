/*!
 * Tests for application configuration functionality
 */

use anyhow::Result;
use yastwai::app_config::{Config, TranslationProvider, LogLevel};

/// Test default configuration values
#[test]
fn test_default_config_withNoParameters_shouldHaveCorrectDefaults() {
    let config = Config::default();
    
    // Test default values
    assert_eq!(config.source_language, "en");
    assert_eq!(config.target_language, "fr");
    assert_eq!(config.translation.provider, TranslationProvider::Ollama);
    
    // Test provider config values
    let ollama_config = config.translation.get_provider_config(&TranslationProvider::Ollama)
        .expect("Ollama provider config should exist");
    
    // Check default values using the same functions used in the Config implementation
    // These are internal functions in the app_config module
    assert_eq!(ollama_config.concurrent_requests, 4); // default_concurrent_requests()
    assert_eq!(ollama_config.max_chars_per_request, 1000); // default_max_chars_per_request()
    assert_eq!(ollama_config.timeout_secs, 30); // default_timeout_secs()
    assert_eq!(ollama_config.model, "llama2"); // default_ollama_model()
    
    assert_eq!(config.log_level, LogLevel::Info);
}

/// Test configuration validation
#[test]
fn test_config_validation_withVariousConfigs_shouldValidateCorrectly() {
    // Start with a valid config
    let mut config = Config::default();
    assert!(config.validate().is_ok());
    
    // Invalid source language
    config.source_language = "xyz".to_string();
    assert!(config.validate().is_err());
    config.source_language = "en".to_string();
    
    // Invalid target language
    config.target_language = "".to_string();
    assert!(config.validate().is_err());
    config.target_language = "fr".to_string();
    
    // For OpenAI provider that requires an API key
    config.translation.provider = TranslationProvider::OpenAI;
    
    // Make sure available_providers has entries
    if config.translation.available_providers.is_empty() {
        // Initialize default providers if empty
        config.translation.available_providers = vec![
            yastwai::app_config::ProviderConfig::new(TranslationProvider::Ollama),
            yastwai::app_config::ProviderConfig::new(TranslationProvider::OpenAI),
            yastwai::app_config::ProviderConfig::new(TranslationProvider::Anthropic),
        ];
    }
    
    // First update the API key in available_providers 
    if let Some(provider) = config.translation
        .available_providers
        .iter_mut()
        .find(|p| p.provider_type == "openai") {
        provider.api_key = "".to_string();
    }
    
    // OpenAI with empty API key should fail validation
    assert!(config.validate().is_err());
    
    // Set a valid API key in available_providers
    if let Some(provider) = config.translation
        .available_providers
        .iter_mut()
        .find(|p| p.provider_type == "openai") {
        provider.api_key = "sk-1234567890".to_string();
    }
    
    // Valid with API key
    assert!(config.validate().is_ok());
    
    // Ollama doesn't require API key
    config.translation.provider = TranslationProvider::Ollama;
    assert!(config.validate().is_ok());
} 