/*!
 * Tests to verify no external network calls are made in tests
 */

use anyhow::Result;
use yastwai::providers::Provider;

// Testing that our mock OpenAI provider correctly implements the Provider trait
#[tokio::test]
async fn test_mock_openai_provider_correctly_implements_provider_trait() -> Result<()> {
    use crate::common::mock_providers::MockOpenAI;
    use yastwai::providers::openai::OpenAIRequest;
    
    // Create a mock OpenAI provider
    let provider = MockOpenAI::new();
    
    // Create a test request
    let request = OpenAIRequest::new("gpt-3.5-turbo")
        .add_message("system", "You are a helpful assistant.")
        .add_message("user", "Say hello!");
    
    // Test the complete method
    let response = provider.complete(request).await?;
    
    // Test extract_text
    let text = MockOpenAI::extract_text(&response);
    assert!(!text.is_empty());
    
    Ok(())
}

// Testing that our mock Anthropic provider correctly implements the Provider trait
#[tokio::test]
async fn test_mock_anthropic_provider_correctly_implements_provider_trait() -> Result<()> {
    use crate::common::mock_providers::MockAnthropic;
    use yastwai::providers::anthropic::AnthropicRequest;
    
    // Create a mock Anthropic provider
    let provider = MockAnthropic::new();
    
    // Create a test request
    let request = AnthropicRequest::new("claude-3-haiku-20240307", 10)
        .system("You are a helpful assistant.")
        .add_message("user", "Hello");
    
    // Test the complete method
    let response = provider.complete(request).await?;
    
    // Test extract_text
    let text = MockAnthropic::extract_text(&response);
    assert!(!text.is_empty());
    
    Ok(())
}

// Testing that we can create and use the mock provider factory
#[test]
fn test_mock_provider_factory() {
    use crate::common::mock_providers::MockProviderFactory;
    
    let factory = MockProviderFactory::new();
    
    // Create each provider type
    let openai = factory.create_openai();
    let anthropic = factory.create_anthropic();
    let ollama = factory.create_ollama();
    
    // Verify they can be configured
    openai.fail_next_call(crate::common::mock_providers::MockErrorType::Auth);
    anthropic.fail_next_call(crate::common::mock_providers::MockErrorType::RateLimit);
    ollama.fail_next_call(crate::common::mock_providers::MockErrorType::Connection);
    
    // Just checking that the above compiles and runs without errors
    assert!(true);
}

// Creating a mock translation service
#[test]
fn test_create_mock_translation_service() {
    use crate::common::mock_providers::create_mock_translation_service;
    
    let service_result = create_mock_translation_service();
    assert!(service_result.is_ok(), "Failed to create mock translation service");
}

// Test that API call monitoring works
#[test]
fn test_api_call_monitoring() {
    use crate::common::mock_providers::{setup_api_call_monitor, assert_no_api_calls};
    
    let monitor = setup_api_call_monitor();
    
    // No calls should have been made
    assert_no_api_calls(monitor.clone());
    
    // Add a call to the monitor for testing
    {
        let mut calls = monitor.lock().unwrap();
        calls.push("test call".to_string());
    }
    
    // This should fail because there's a call
    let result = std::panic::catch_unwind(|| {
        assert_no_api_calls(monitor.clone());
    });
    
    // The assertion should have panicked
    assert!(result.is_err(), "Expected assert_no_api_calls to panic");
} 