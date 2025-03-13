/*!
 * Integration tests for provider API interactions
 */

use anyhow::Result;
use std::env;
use tokio_test;
use yastwai::providers::Provider;
use yastwai::providers::openai::{OpenAI, OpenAIRequest};
use yastwai::providers::anthropic::{Anthropic, AnthropicRequest};
use yastwai::providers::ollama::{Ollama, GenerationRequest, ChatRequest, ChatMessage};

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

/// Test OpenAI API integration
#[tokio::test]
#[ignore]
async fn test_openai_complete_withValidApiKey_shouldReturnResponse() {
    // This test should only run if an API key is provided
    let api_key = std::env::var("OPENAI_API_KEY").unwrap_or_default();
    if api_key.is_empty() {
        return;
    }
    
    let client = OpenAI::new(api_key, "");
    let request = OpenAIRequest::new("gpt-3.5-turbo")
        .add_message("system", "You are a helpful assistant.")
        .add_message("user", "Say hello!")
        .max_tokens(10);
    
    let response = client.complete(request).await.unwrap();
    assert!(!response.choices.is_empty());
    assert!(!response.choices[0].message.content.is_empty());
}

/// Test Anthropic API integration
#[tokio::test]
#[ignore]
async fn test_anthropic_complete_withValidApiKey_shouldReturnResponse() {
    // This test should only run if an API key is provided
    let api_key = std::env::var("ANTHROPIC_API_KEY").unwrap_or_default();
    if api_key.is_empty() {
        return;
    }
    
    let client = Anthropic::new(api_key, "");
    let request = AnthropicRequest::new("claude-3-haiku-20240307", 1024)
        .system("You are a helpful assistant.")
        .add_message("user", "Say hello!");
    
    let response = client.complete(request).await.unwrap();
    
    // Extract text from the response
    let text = if let Some(content) = response.content.first() {
        &content.text
    } else {
        ""
    };
    
    assert!(!text.is_empty());
}

/// Test Ollama generation API integration
#[tokio::test]
#[ignore]
async fn test_ollama_generate_withLocalServer_shouldReturnResponse() {
    // This test should only run if Ollama is available locally
    let client = Ollama::new("http://localhost", 11434);
    
    // Try to get the version, if it fails, skip the test
    if client.version().await.is_err() {
        println!("Skipping test because Ollama is not available");
        return;
    }
    
    let request = GenerationRequest::new("llama2", "Hello, world!")
        .system("You are a helpful assistant.")
        .temperature(0.7);
    
    let response = client.generate(request).await;
    assert!(response.is_ok());
}

/// Test Ollama chat API integration
#[tokio::test]
#[ignore]
async fn test_ollama_chat_withLocalServer_shouldReturnResponse() {
    // This test should only run if Ollama is available locally
    let client = Ollama::new("http://localhost", 11434);
    
    // Try to get the version, if it fails, skip the test
    if client.version().await.is_err() {
        println!("Skipping test because Ollama is not available");
        return;
    }
    
    let messages = vec![
        ChatMessage {
            role: "user".to_string(),
            content: "Hello, who are you?".to_string(),
        }
    ];
    
    let request = ChatRequest::new("llama2", messages)
        .system("You are a helpful assistant.")
        .temperature(0.7);
    
    let response = client.chat(request).await;
    assert!(response.is_ok());
} 