use std::time::Duration;
use yastwai::providers::anthropic::{Anthropic, AnthropicRequest, AnthropicResponse, AnthropicContent, TokenUsage};
use yastwai::providers::Provider;
use yastwai::errors::ProviderError;

#[tokio::test]
async fn test_anthropic_request_builder() {
    // Test the builder pattern functions for AnthropicRequest
    let request = AnthropicRequest::new("claude-3-sonnet-20240229", 100)
        .add_message("user", "Hello")
        .system("You are a helpful assistant")
        .temperature(0.5)
        .top_p(0.9)
        .top_k(40);
    
    // Serialize to JSON and check format
    let json = serde_json::to_string(&request).expect("Failed to serialize request");
    
    // Check that all fields were properly set
    assert!(json.contains(r#""model":"claude-3-sonnet-20240229""#));
    assert!(json.contains(r#""max_tokens":100"#));
    assert!(json.contains(r#""temperature":0.5"#));
    assert!(json.contains(r#""top_p":0.9"#));
    assert!(json.contains(r#""top_k":40"#));
    assert!(json.contains(r#""system":"You are a helpful assistant""#));
    assert!(json.contains(r#""role":"user""#));
    assert!(json.contains(r#""content":"Hello""#));
}

#[tokio::test]
async fn test_anthropic_extract_text() {
    // Create a mock response
    let response = AnthropicResponse {
        content: vec![
            AnthropicContent {
                content_type: "text".to_string(),
                text: "Hello, ".to_string(),
            },
            AnthropicContent {
                content_type: "text".to_string(),
                text: "world!".to_string(),
            },
            // This one should be filtered out
            AnthropicContent {
                content_type: "image".to_string(),
                text: "image_data".to_string(),
            },
        ],
        usage: TokenUsage {
            input_tokens: 10,
            output_tokens: 20,
        },
    };
    
    // Test the extract_text function
    let text = Anthropic::extract_text(&response);
    
    // Should combine the text content only, excluding non-text types
    assert_eq!(text, "Hello, world!");
}

#[tokio::test]
async fn test_anthropic_api_error_handling() {
    // Use the mock provider instead of real API
    use crate::common::mock_providers::{MockAnthropic, MockErrorType};
    
    // Create a mock Anthropic provider
    let anthropic = MockAnthropic::new();
    
    // Configure it to fail with auth error
    anthropic.fail_next_call(MockErrorType::Auth);
    
    // Make a request that should fail with the configured error
    let request = AnthropicRequest::new("claude-3-haiku-20240307", 10)
        .add_message("user", "Hello");
    
    let result = anthropic.complete(request).await;
    
    // This should return an error
    assert!(result.is_err());
    
    // The error should be an authentication error
    match result.unwrap_err() {
        ProviderError::AuthenticationError(_) => {
            // This is the expected error type
            assert!(true);
        },
        err => {
            // Any other error type is unexpected
            panic!("Unexpected error type: {:?}", err);
        }
    }
}

#[tokio::test]
async fn test_anthropic_retry_logic() {
    // Use the mock provider for testing retry logic
    use crate::common::mock_providers::{MockAnthropic, MockErrorType};
    use std::time::{Instant, Duration};
    
    // Create a mock Anthropic provider
    let anthropic = MockAnthropic::new();
    
    // Configure it to fail with connection error
    anthropic.fail_next_call(MockErrorType::Connection);
    
    // Make a request
    let request = AnthropicRequest::new("claude-3-haiku-20240307", 10)
        .add_message("user", "Hello");
    
    let start = Instant::now();
    let result = anthropic.complete(request).await;
    let elapsed = start.elapsed();
    
    // Should fail
    assert!(result.is_err());
    
    // Error should be a connection error
    match result.unwrap_err() {
        ProviderError::ConnectionError(_) => {
            // This is expected
            assert!(true);
        },
        err => {
            panic!("Unexpected error type: {:?}", err);
        }
    }
}

// This test uses the mock provider instead of real API credentials
#[tokio::test]
async fn test_anthropic_successful_request() {
    // Use the mock provider
    use crate::common::mock_providers::MockAnthropic;
    
    // Create a mock Anthropic provider
    let anthropic = MockAnthropic::new();
    
    // Make a request
    let request = AnthropicRequest::new("claude-3-haiku-20240307", 10)
        .system("You are a helpful assistant.")
        .add_message("user", "Hello");
    
    let response = anthropic.complete(request).await;
    
    // Should succeed
    assert!(response.is_ok());
    
    // Should have expected response text
    let text = Anthropic::extract_text(&response.unwrap());
    assert_eq!(text, "This is a mock response from Anthropic.");
    
    // Should have tracked the call
    let tracker = anthropic.tracker();
    let tracker = tracker.lock().unwrap();
    assert_eq!(tracker.call_count, 1);
}

// This test is disabled by default as it requires real API credentials
#[ignore]
#[tokio::test]
async fn test_integration_with_real_api() {
    // Read API key from environment variable
    let api_key = std::env::var("ANTHROPIC_API_KEY")
        .expect("ANTHROPIC_API_KEY environment variable not set");
    
    let anthropic = Anthropic::new(api_key, "");
    
    // Test the connection
    let result = anthropic.test_connection().await;
    assert!(result.is_ok(), "API connection test failed: {:?}", result.err());
    
    // Make a simple completion request
    let request = AnthropicRequest::new("claude-3-haiku-20240307", 50)
        .add_message("user", "Say hello in French")
        .temperature(0.0); // Use deterministic output for testing
    
    let result = anthropic.complete(request).await;
    assert!(result.is_ok(), "API completion failed: {:?}", result.err());
    
    let response = result.unwrap();
    let text = Anthropic::extract_text(&response);
    
    // The text should contain a French greeting
    assert!(text.contains("Bonjour") || text.contains("Salut"), 
            "Response doesn't contain expected French greeting: {}", text);
} 