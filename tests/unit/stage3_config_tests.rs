/*!
 * Tests for Stage 3 audit requirements - config-driven concurrency, retries, and rate limits
 */

use yastwai::app_config::{Config, TranslationProvider, ProviderConfig, TranslationCommonConfig};
use yastwai::translation::core::TranslationService;

#[test]
fn test_concurrency_withConfigValue_shouldUseConfigNotDefault() {
    // Create a config with specific concurrency setting
    let mut config = Config::default();
    
    // Set up provider config with custom concurrency
    let mut provider_config = ProviderConfig::new(TranslationProvider::Ollama);
    provider_config.concurrent_requests = 8; // Custom value different from default (4)
    
    config.translation.available_providers = vec![provider_config];
    config.translation.provider = TranslationProvider::Ollama;
    
    // Create translation service
    let service = TranslationService::new(config.translation.clone()).unwrap();
    
    // Verify that the service uses the configured concurrency value
    assert_eq!(service.options.max_concurrent_requests, 8);
    assert_eq!(config.translation.optimal_concurrent_requests(), 8);
}

#[test]
fn test_retryConfig_withCustomValues_shouldUseConfigValues() {
    let mut config = Config::default();
    
    // Set custom retry configuration
    config.translation.common.retry_count = 5;
    config.translation.common.retry_backoff_ms = 2000;
    
    // Set up Anthropic provider with API key
    let mut anthropic_config = ProviderConfig::new(TranslationProvider::Anthropic);
    anthropic_config.api_key = "test-key".to_string();
    
    config.translation.available_providers = vec![anthropic_config];
    config.translation.provider = TranslationProvider::Anthropic;
    
    // Create translation service - this should use the custom retry values
    // The fact that it succeeds without error indicates configuration was applied correctly
    let service = TranslationService::new(config.translation).unwrap();
    
    // Verify the service was created with our custom config
    assert_eq!(service.config.common.retry_count, 5);
    assert_eq!(service.config.common.retry_backoff_ms, 2000);
}

#[test]
fn test_rateLimit_withProviderSettings_shouldRespectRateLimits() {
    let mut config = Config::default();
    
    // Test OpenAI rate limiting
    let mut openai_config = ProviderConfig::new(TranslationProvider::OpenAI);
    openai_config.api_key = "test-key".to_string();
    openai_config.rate_limit = Some(30); // 30 requests per minute
    
    config.translation.available_providers = vec![openai_config];
    config.translation.provider = TranslationProvider::OpenAI;
    
    // Verify config retrieval
    assert_eq!(config.translation.get_rate_limit(), Some(30));
    
    // Service should be created successfully with rate limit configuration
    let service = TranslationService::new(config.translation).unwrap();
    
    // Verify the service was created with our rate limit config
    assert_eq!(service.config.get_rate_limit(), Some(30));
}

#[test]
fn test_ollamaRateLimit_withConfiguration_shouldAllowOptionalThrottling() {
    let mut config = Config::default();
    
    // Test Ollama with optional rate limiting
    let mut ollama_config = ProviderConfig::new(TranslationProvider::Ollama);
    ollama_config.rate_limit = Some(10); // Optional throttling for Ollama
    
    config.translation.available_providers = vec![ollama_config];
    config.translation.provider = TranslationProvider::Ollama;
    
    // Verify config retrieval
    assert_eq!(config.translation.get_rate_limit(), Some(10));
    
    // Service should be created successfully with rate limit configuration
    let service = TranslationService::new(config.translation).unwrap();
    
    // Verify the service was created with our rate limit config
    assert_eq!(service.config.get_rate_limit(), Some(10));
}

#[test]
fn test_defaultConcurrency_withoutProviderConfig_shouldUseFallback() {
    let mut config = Config::default();
    
    // Don't add any provider configs, so it should fall back to defaults
    config.translation.available_providers = vec![];
    config.translation.provider = TranslationProvider::Ollama;
    
    // Create translation service
    let service = TranslationService::new(config.translation.clone()).unwrap();
    
    // Should use default concurrency (4) when no provider config exists
    assert_eq!(service.options.max_concurrent_requests, 4);
    assert_eq!(config.translation.optimal_concurrent_requests(), 4);
}

#[test]
fn test_commonConfigDefaults_shouldProvideReasonableValues() {
    let common_config = TranslationCommonConfig::default();
    
    // Verify reasonable default values for retry configuration
    assert_eq!(common_config.retry_count, 3);
    assert_eq!(common_config.retry_backoff_ms, 1000);
    assert!(common_config.rate_limit_delay_ms > 0);
    assert!(common_config.temperature >= 0.0 && common_config.temperature <= 1.0);
}

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
}
