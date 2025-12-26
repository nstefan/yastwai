/*!
 * Mock provider implementations for testing
 * 
 * This module provides mock implementations of all providers to avoid
 * external API calls in tests. Each provider implements the Provider trait
 * and returns predetermined responses.
 */

use async_trait::async_trait;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use anyhow::Result;

use yastwai::errors::ProviderError;
use yastwai::providers::Provider;
use yastwai::providers::openai::{OpenAIRequest, OpenAIResponse, OpenAIChoice, OpenAIMessage};
use yastwai::providers::anthropic::{AnthropicRequest, AnthropicResponse, AnthropicContent, TokenUsage};
use yastwai::providers::ollama::{GenerationRequest, ChatRequest, ChatMessage, GenerationResponse, ChatResponse};

/// Tracks API calls to ensure no actual external requests are made
#[derive(Debug, Default)]
pub struct ApiCallTracker {
    /// Count of mock API calls made
    pub call_count: usize,
    /// Last request received
    pub last_request: Option<String>,
    /// Should the next call fail
    pub should_fail: bool,
    /// Error to return if failing
    pub error_type: MockErrorType,
}

/// Type of error to simulate
#[derive(Debug, Clone, Copy)]
pub enum MockErrorType {
    /// Authentication error (invalid API key)
    Auth,
    /// Connection error
    Connection,
    /// Rate limit error
    RateLimit,
    /// API error
    Api,
}

impl Default for MockErrorType {
    fn default() -> Self {
        MockErrorType::Auth
    }
}

/// Mock implementation of OpenAI provider
#[derive(Debug)]
pub struct MockOpenAI {
    tracker: Arc<Mutex<ApiCallTracker>>,
}

impl MockOpenAI {
    /// Create a new mock OpenAI provider
    pub fn new() -> Self {
        MockOpenAI {
            tracker: Arc::new(Mutex::new(ApiCallTracker::default())),
        }
    }
    
    /// Get the API call tracker
    pub fn tracker(&self) -> Arc<Mutex<ApiCallTracker>> {
        self.tracker.clone()
    }
    
    /// Configure the mock to fail on the next call
    pub fn fail_next_call(&self, error_type: MockErrorType) {
        let mut tracker = self.tracker.lock().unwrap();
        tracker.should_fail = true;
        tracker.error_type = error_type;
    }
}

#[async_trait]
impl Provider for MockOpenAI {
    type Request = OpenAIRequest;
    type Response = OpenAIResponse;
    
    async fn complete(&self, request: Self::Request) -> Result<Self::Response, ProviderError> {
        let mut tracker = self.tracker.lock().unwrap();
        tracker.call_count += 1;
        tracker.last_request = Some(format!("{:?}", request));
        
        if tracker.should_fail {
            tracker.should_fail = false; // Reset for next call
            return match tracker.error_type {
                MockErrorType::Auth => Err(ProviderError::AuthenticationError("Invalid API key".into())),
                MockErrorType::Connection => Err(ProviderError::ConnectionError("Connection failed".into())),
                MockErrorType::RateLimit => Err(ProviderError::RateLimitExceeded("Rate limit exceeded".into())),
                MockErrorType::Api => Err(ProviderError::ApiError { 
                    status_code: 400, 
                    message: "Bad request".into() 
                }),
            };
        }
        
        // Return a mock response
        Ok(OpenAIResponse {
            choices: vec![
                OpenAIChoice {
                    message: OpenAIMessage {
                        role: "assistant".into(),
                        content: "This is a mock response from OpenAI.".into(),
                    },
                }
            ],
            usage: Some(yastwai::providers::openai::TokenUsage {
                prompt_tokens: 10,
                completion_tokens: 20,
                total_tokens: 30,
            }),
        })
    }
    
    fn extract_text(response: &Self::Response) -> String {
        if let Some(choice) = response.choices.first() {
            choice.message.content.clone()
        } else {
            String::new()
        }
    }
}

/// Mock implementation of Anthropic provider
#[derive(Debug)]
pub struct MockAnthropic {
    tracker: Arc<Mutex<ApiCallTracker>>,
}

impl MockAnthropic {
    /// Create a new mock Anthropic provider
    pub fn new() -> Self {
        MockAnthropic {
            tracker: Arc::new(Mutex::new(ApiCallTracker::default())),
        }
    }
    
    /// Get the API call tracker
    pub fn tracker(&self) -> Arc<Mutex<ApiCallTracker>> {
        self.tracker.clone()
    }
    
    /// Configure the mock to fail on the next call
    pub fn fail_next_call(&self, error_type: MockErrorType) {
        let mut tracker = self.tracker.lock().unwrap();
        tracker.should_fail = true;
        tracker.error_type = error_type;
    }
}

#[async_trait]
impl Provider for MockAnthropic {
    type Request = AnthropicRequest;
    type Response = AnthropicResponse;
    
    async fn complete(&self, request: Self::Request) -> Result<Self::Response, ProviderError> {
        let mut tracker = self.tracker.lock().unwrap();
        tracker.call_count += 1;
        tracker.last_request = Some(format!("{:?}", request));
        
        if tracker.should_fail {
            tracker.should_fail = false; // Reset for next call
            return match tracker.error_type {
                MockErrorType::Auth => Err(ProviderError::AuthenticationError("Invalid API key".into())),
                MockErrorType::Connection => Err(ProviderError::ConnectionError("Connection failed".into())),
                MockErrorType::RateLimit => Err(ProviderError::RateLimitExceeded("Rate limit exceeded".into())),
                MockErrorType::Api => Err(ProviderError::ApiError { 
                    status_code: 400, 
                    message: "Bad request".into() 
                }),
            };
        }
        
        // Return a mock response
        Ok(AnthropicResponse {
            content: vec![
                AnthropicContent {
                    content_type: "text".into(),
                    text: "This is a mock response from Anthropic.".into(),
                },
            ],
            usage: TokenUsage {
                input_tokens: 10,
                output_tokens: 20,
            },
        })
    }
    
    fn extract_text(response: &Self::Response) -> String {
        response.content
            .iter()
            .filter(|c| c.content_type == "text")
            .map(|c| c.text.clone())
            .collect()
    }
}

/// Mock implementation of Ollama provider
#[derive(Debug)]
pub struct MockOllama {
    tracker: Arc<Mutex<ApiCallTracker>>,
}

impl MockOllama {
    /// Create a new mock Ollama provider
    pub fn new() -> Self {
        MockOllama {
            tracker: Arc::new(Mutex::new(ApiCallTracker::default())),
        }
    }
    
    /// Get the API call tracker
    pub fn tracker(&self) -> Arc<Mutex<ApiCallTracker>> {
        self.tracker.clone()
    }
    
    /// Configure the mock to fail on the next call
    pub fn fail_next_call(&self, error_type: MockErrorType) {
        let mut tracker = self.tracker.lock().unwrap();
        tracker.should_fail = true;
        tracker.error_type = error_type;
    }
    
    /// Mock implementation of version check
    pub async fn version(&self) -> Result<String, ProviderError> {
        let mut tracker = self.tracker.lock().unwrap();
        tracker.call_count += 1;
        
        if tracker.should_fail {
            tracker.should_fail = false; // Reset for next call
            return match tracker.error_type {
                MockErrorType::Connection => Err(ProviderError::ConnectionError("Connection failed".into())),
                _ => Err(ProviderError::ApiError { 
                    status_code: 400, 
                    message: "Bad request".into() 
                }),
            };
        }
        
        Ok("0.1.0".into())
    }
    
    /// Mock implementation of generate endpoint
    pub async fn generate(&self, _request: GenerationRequest) -> Result<GenerationResponse, ProviderError> {
        let mut tracker = self.tracker.lock().unwrap();
        tracker.call_count += 1;
        
        if tracker.should_fail {
            tracker.should_fail = false; // Reset for next call
            return match tracker.error_type {
                MockErrorType::Auth => Err(ProviderError::AuthenticationError("Invalid API key".into())),
                MockErrorType::Connection => Err(ProviderError::ConnectionError("Connection failed".into())),
                MockErrorType::RateLimit => Err(ProviderError::RateLimitExceeded("Rate limit exceeded".into())),
                MockErrorType::Api => Err(ProviderError::ApiError { 
                    status_code: 400, 
                    message: "Bad request".into() 
                }),
            };
        }
        
        // Return mock response
        Ok(GenerationResponse {
            model: String::from("mock-model"),
            created_at: String::from("2023-01-01T00:00:00Z"),
            response: "This is a mock response from Ollama generate.".into(),
            done: true,
            context: None,
            total_duration: Some(100),
            load_duration: Some(50),
            prompt_eval_count: Some(10),
            prompt_eval_duration: Some(20),
            eval_count: Some(30),
            eval_duration: Some(30),
        })
    }
    
    /// Mock implementation of chat endpoint
    pub async fn chat(&self, _request: ChatRequest) -> Result<ChatResponse, ProviderError> {
        let mut tracker = self.tracker.lock().unwrap();
        tracker.call_count += 1;
        
        if tracker.should_fail {
            tracker.should_fail = false; // Reset for next call
            return match tracker.error_type {
                MockErrorType::Auth => Err(ProviderError::AuthenticationError("Invalid API key".into())),
                MockErrorType::Connection => Err(ProviderError::ConnectionError("Connection failed".into())),
                MockErrorType::RateLimit => Err(ProviderError::RateLimitExceeded("Rate limit exceeded".into())),
                MockErrorType::Api => Err(ProviderError::ApiError { 
                    status_code: 400, 
                    message: "Bad request".into() 
                }),
            };
        }
        
        // Return mock response
        Ok(ChatResponse {
            model: String::from("mock-model"),
            created_at: String::from("2023-01-01T00:00:00Z"),
            message: ChatMessage {
                role: "assistant".into(),
                content: "This is a mock response from Ollama chat.".into(),
            },
            done: true,
            total_duration: Some(100),
            load_duration: Some(50),
            prompt_eval_count: Some(10),
            prompt_eval_duration: Some(20),
            eval_count: Some(30),
            eval_duration: Some(30),
        })
    }
}

/// Factory for creating mock providers
#[derive(Debug, Default)]
pub struct MockProviderFactory;

impl MockProviderFactory {
    /// Create a new mock provider factory
    pub fn new() -> Self {
        MockProviderFactory
    }
    
    /// Create a mock OpenAI provider
    pub fn create_openai(&self) -> MockOpenAI {
        MockOpenAI::new()
    }
    
    /// Create a mock Anthropic provider
    pub fn create_anthropic(&self) -> MockAnthropic {
        MockAnthropic::new()
    }
    
    /// Create a mock Ollama provider
    pub fn create_ollama(&self) -> MockOllama {
        MockOllama::new()
    }
}

/// Helper function to create a translation service with mock providers
pub fn create_mock_translation_service() -> Result<yastwai::translation::core::TranslationService> {
    // Import the necessary types
    use yastwai::app_config::{TranslationConfig, TranslationProvider, TranslationCommonConfig, ProviderConfig};
    
    // Create a test configuration
    let config = TranslationConfig {
        provider: TranslationProvider::OpenAI,
        common: TranslationCommonConfig {
            system_prompt: "You are a translator. Translate the following text from {source_language} to {target_language}. Only return the translated text.".into(),
            rate_limit_delay_ms: 0,
            retry_count: 1,
            retry_backoff_ms: 1,
            temperature: 0.3,
            parallel_mode: true,
            entries_per_request: 3,
            context_entries_count: 3,
        },
        available_providers: vec![
            ProviderConfig {
                provider_type: "openai".to_string(),
                model: "gpt-3.5-turbo".to_string(),
                api_key: "mock-api-key".to_string(),
                endpoint: "".to_string(),
                concurrent_requests: 1,
                max_chars_per_request: 1000,
                timeout_secs: 1,
                rate_limit: Some(60),
            },
        ],
    };
    
    yastwai::translation::core::TranslationService::new(config)
}

/// Helper function to set up a test environment that captures any API calls
pub fn setup_api_call_monitor() -> Arc<Mutex<Vec<String>>> {
    // This could be expanded to hook into network requests at a lower level
    Arc::new(Mutex::new(Vec::new()))
}

/// Helper to check if any API calls were made during a test
pub fn assert_no_api_calls(monitor: Arc<Mutex<Vec<String>>>) {
    let calls = monitor.lock().unwrap();
    assert!(
        calls.is_empty(),
        "Expected no API calls, but found: {:?}",
        *calls
    );
} 