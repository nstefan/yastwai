/*!
 * Tests for application configuration functionality
 */

use yastwai::app_config::{Config, TranslationProvider, LogLevel, ProviderConfig, TranslationCommonConfig, ExperimentalFeatures};

/// Helper function to find provider config by type
fn find_provider_config<'a>(config: &'a Config, provider: &TranslationProvider) -> Option<&'a ProviderConfig> {
    let provider_str = match provider {
        TranslationProvider::Ollama => "ollama",
        TranslationProvider::OpenAI => "openai",
        TranslationProvider::Anthropic => "anthropic",
        TranslationProvider::LMStudio => "lmstudio",
    };
    config.translation.available_providers.iter()
        .find(|p| p.provider_type == provider_str)
}

/// Test default configuration values
#[test]
fn test_default_config_withNoParameters_shouldHaveCorrectDefaults() {
    let config = Config::default();
    
    // Test default values
    assert_eq!(config.source_language, "en");
    assert_eq!(config.target_language, "fr");
    assert_eq!(config.translation.provider, TranslationProvider::Ollama);
    
    // Test provider config values
    let ollama_config = find_provider_config(&config, &TranslationProvider::Ollama)
        .expect("Ollama provider config should exist");
    
    // Check default values using the same functions used in the Config implementation
    assert_eq!(ollama_config.concurrent_requests, 4); // default_concurrent_requests()
    assert_eq!(ollama_config.max_chars_per_request, 1000); // default_max_chars_per_request()
    assert_eq!(ollama_config.timeout_secs, 30); // default_timeout_secs()
    assert_eq!(ollama_config.model, "llama2"); // default_ollama_model()
    
    assert_eq!(config.log_level, LogLevel::Info);

    // Ensure LM Studio provider config exists by default
    let lmstudio_config = find_provider_config(&config, &TranslationProvider::LMStudio)
        .expect("LMStudio provider config should exist");
    assert_eq!(lmstudio_config.endpoint, "http://localhost:1234/v1");
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

    // LM Studio shouldn't require API key
    config.translation.provider = TranslationProvider::LMStudio;
    // Ensure available_providers has lmstudio
    if find_provider_config(&config, &TranslationProvider::LMStudio).is_none() {
        config.translation.available_providers.push(
            yastwai::app_config::ProviderConfig::new(TranslationProvider::LMStudio)
        );
    }
    assert!(config.validate().is_ok());
}

/// Test that common configuration provides reasonable default values
#[test]
fn test_commonConfigDefaults_shouldProvideReasonableValues() {
    let common_config = TranslationCommonConfig::default();
    
    // Verify reasonable default values for retry configuration
    assert_eq!(common_config.retry_count, 3);
    assert_eq!(common_config.retry_backoff_ms, 1000);
    assert!(common_config.rate_limit_delay_ms > 0);
    assert!(common_config.temperature >= 0.0 && common_config.temperature <= 1.0);
}

/// Test that each provider has appropriate default rate limits
#[test]
fn test_providerSpecificDefaults_shouldHaveCorrectRateLimits() {
    // Test that each provider has appropriate default rate limits
    
    // Ollama (local) should have no rate limit by default
    let ollama_config = ProviderConfig::new(TranslationProvider::Ollama);
    assert_eq!(ollama_config.rate_limit, None);
    
    // OpenAI should have a reasonable rate limit
    let openai_config = ProviderConfig::new(TranslationProvider::OpenAI);
    assert_eq!(openai_config.rate_limit, Some(60));
    
    // Anthropic should have a conservative rate limit (45 < 50 limit)
    let anthropic_config = ProviderConfig::new(TranslationProvider::Anthropic);
    assert_eq!(anthropic_config.rate_limit, Some(45));

    // LM Studio (local) should have no rate limit by default
    let lmstudio_config = ProviderConfig::new(TranslationProvider::LMStudio);
    assert_eq!(lmstudio_config.rate_limit, None);
}

/// Test that ExperimentalFeatures defaults all flags to false
#[test]
fn test_experimentalFeaturesDefaults_shouldAllBeFalse() {
    let features = ExperimentalFeatures::default();

    assert!(!features.enable_auto_tune_concurrency);
    assert!(!features.enable_adaptive_batch_sizing);
    assert!(!features.enable_cache_warming);
    assert!(!features.enable_speculative_batching);
    assert!(!features.enable_language_pair_thresholds);
    assert!(!features.enable_glossary_preflight);
    assert!(!features.enable_fuzzy_glossary_matching);
    assert!(!features.enable_feedback_retry);
    assert!(!features.enable_semantic_validation);
    assert!(!features.enable_dynamic_context_window);
    assert!(!features.enable_scene_aware_batching);
    assert!(!features.enable_speaker_tracking);
}

/// Test that Config includes ExperimentalFeatures with all defaults false
#[test]
fn test_configDefault_shouldIncludeExperimentalFeaturesAllFalse() {
    let config = Config::default();

    assert!(!config.experimental.enable_auto_tune_concurrency);
    assert!(!config.experimental.enable_adaptive_batch_sizing);
    assert!(!config.experimental.enable_cache_warming);
    assert!(!config.experimental.enable_speculative_batching);
    assert!(!config.experimental.enable_language_pair_thresholds);
    assert!(!config.experimental.enable_glossary_preflight);
    assert!(!config.experimental.enable_fuzzy_glossary_matching);
    assert!(!config.experimental.enable_feedback_retry);
    assert!(!config.experimental.enable_semantic_validation);
    assert!(!config.experimental.enable_dynamic_context_window);
    assert!(!config.experimental.enable_scene_aware_batching);
    assert!(!config.experimental.enable_speaker_tracking);
}

/// Test backward compatibility: old config without experimental field deserializes correctly
#[test]
fn test_configDeserialization_withoutExperimentalField_shouldUseDefaults() {
    let json = r#"{
        "source_language": "en",
        "target_language": "de",
        "translation": {
            "provider": "ollama",
            "available_providers": [],
            "common": {}
        }
    }"#;

    let config: Config = serde_json::from_str(json).expect("Should deserialize config without experimental field");

    // Verify basic fields
    assert_eq!(config.source_language, "en");
    assert_eq!(config.target_language, "de");

    // Verify experimental features default to false
    assert!(!config.experimental.enable_auto_tune_concurrency);
    assert!(!config.experimental.enable_adaptive_batch_sizing);
    assert!(!config.experimental.enable_cache_warming);
    assert!(!config.experimental.enable_speculative_batching);
}

/// Test that individual experimental flags can be enabled
#[test]
fn test_configDeserialization_withPartialExperimentalFlags_shouldMergeWithDefaults() {
    let json = r#"{
        "source_language": "en",
        "target_language": "de",
        "translation": {
            "provider": "ollama",
            "available_providers": [],
            "common": {}
        },
        "experimental": {
            "enable_cache_warming": true,
            "enable_feedback_retry": true
        }
    }"#;

    let config: Config = serde_json::from_str(json).expect("Should deserialize config with partial experimental flags");

    // Explicitly enabled flags should be true
    assert!(config.experimental.enable_cache_warming);
    assert!(config.experimental.enable_feedback_retry);

    // Unspecified flags should default to false
    assert!(!config.experimental.enable_auto_tune_concurrency);
    assert!(!config.experimental.enable_adaptive_batch_sizing);
    assert!(!config.experimental.enable_speculative_batching);
    assert!(!config.experimental.enable_language_pair_thresholds);
    assert!(!config.experimental.enable_glossary_preflight);
    assert!(!config.experimental.enable_fuzzy_glossary_matching);
    assert!(!config.experimental.enable_semantic_validation);
    assert!(!config.experimental.enable_dynamic_context_window);
    assert!(!config.experimental.enable_scene_aware_batching);
    assert!(!config.experimental.enable_speaker_tracking);
}
