/*!
 * Integration tests for provider API interactions
 */

use anyhow::Result;
use std::env;
use yastwai::providers::Provider;
use yastwai::providers::openai::OpenAIRequest;
use yastwai::providers::anthropic::{Anthropic, AnthropicRequest};
use yastwai::providers::ollama::{GenerationRequest, ChatRequest, ChatMessage};
use crate::common::mock_providers::{MockOpenAI, MockAnthropic, MockOllama};

/// Test that we can handle missing API keys gracefully
#[test]
fn test_missing_api_key_withEmptyKey_shouldReturnError() -> Result<()> {
    // This test doesn't actually make API calls but simulates the behavior
    
    // Test a fake API key handling
    let api_key = env::var("FAKE_API_KEY").unwrap_or_default();
    assert!(api_key.is_empty(), "Expected empty API key for test");
    
    // This simulates what would happen when a client tries to use an empty API key
    let result = if api_key.is_empty() {
        Err(anyhow::anyhow!("API key is missing or empty"))
    } else {
        Ok(())
    };
    
    // Verify proper error handling
    assert!(result.is_err(), "Empty API key should return error");
    if let Err(e) = result {
        assert!(e.to_string().contains("API key"), 
                "Error message should mention API key but was: {}", e);
    }
    
    Ok(())
}

/// Test that we can mock the provider interface for testing
#[test]
fn test_mock_provider_withMockedResponse_shouldReturnExpectedResult() -> Result<()> {
    // Create a simple mock provider that just returns a preset response
    struct MockProvider;
    
    impl MockProvider {
        fn new() -> Self {
            MockProvider
        }
        
        fn translate(&self, text: &str, _source_lang: &str, target_lang: &str) -> Result<String> {
            Ok(format!("[{}] {}", target_lang, text))
        }
    }
    
    // Create the mock provider
    let provider = MockProvider::new();
    
    // Test translation
    let input = "Hello, world!";
    let result = provider.translate(input, "en", "fr")?;
    
    // Verify the result
    assert_eq!(result, "[fr] Hello, world!");
    
    Ok(())
}

/// Test OpenAI API integration with mock provider
#[tokio::test]
async fn test_openai_complete_withMockProvider_shouldReturnResponse() {
    // Create a mock OpenAI provider
    let client = MockOpenAI::new();
    let request = OpenAIRequest::new("gpt-3.5-turbo")
        .add_message("system", "You are a helpful assistant.")
        .add_message("user", "Say hello!")
        .max_tokens(10);
    
    let response = client.complete(request).await.unwrap();
    assert!(!response.choices.is_empty());
    assert!(!response.choices[0].message.content.is_empty());
    
    // Verify the mock was called
    let tracker = client.tracker();
    let tracker = tracker.lock().unwrap();
    assert_eq!(tracker.call_count, 1);
}

/// Test Anthropic API integration with mock provider
#[tokio::test]
async fn test_anthropic_complete_withMockProvider_shouldReturnResponse() {
    // Create a mock Anthropic provider
    let client = MockAnthropic::new();
    let request = AnthropicRequest::new("claude-3-haiku-20240307", 1024)
        .system("You are a helpful assistant.")
        .add_message("user", "Say hello!");
    
    let response = client.complete(request).await.unwrap();
    
    // Extract text from the response
    let text = Anthropic::extract_text(&response);
    assert!(!text.is_empty());
    
    // Verify the mock was called
    let tracker = client.tracker();
    let tracker = tracker.lock().unwrap();
    assert_eq!(tracker.call_count, 1);
}

/// Test Ollama generation API integration with mock provider
#[tokio::test]
async fn test_ollama_generate_withMockProvider_shouldReturnResponse() {
    // Create a mock Ollama provider
    let client = MockOllama::new();
    
    let request = GenerationRequest::new("llama2", "Hello, world!")
        .system("You are a helpful assistant.")
        .temperature(0.7);
    
    let response = client.generate(request).await;
    assert!(response.is_ok());
    
    // Verify the mock was called
    let tracker = client.tracker();
    let tracker = tracker.lock().unwrap();
    assert_eq!(tracker.call_count, 1);
}

/// Test Ollama chat API integration with mock provider
#[tokio::test]
async fn test_ollama_chat_withMockProvider_shouldReturnResponse() {
    // Create a mock Ollama provider
    let client = MockOllama::new();
    
    let messages = vec![
        ChatMessage {
            role: "system".to_string(),
            content: "You are a helpful assistant.".to_string(),
        },
        ChatMessage {
            role: "user".to_string(),
            content: "Say hello!".to_string(),
        }
    ];
    
    let request = ChatRequest::new("llama2", messages)
        .temperature(0.7);
    
    let response = client.chat(request).await;
    assert!(response.is_ok());
    
    // Verify the mock was called
    let tracker = client.tracker();
    let tracker = tracker.lock().unwrap();
    assert_eq!(tracker.call_count, 1);
} 