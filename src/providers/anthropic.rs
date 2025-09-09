use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};
use anyhow::Result;
use reqwest::Client;
use async_trait::async_trait;
use tokio::time::sleep;
use tokio::sync::Mutex;

use crate::errors::ProviderError;
use super::Provider;


/// Token bucket rate limiter implementation
///
/// This rate limiter implements the token bucket algorithm:
/// - A bucket holds tokens up to a maximum capacity
/// - Tokens are consumed when API requests are made
/// - Tokens are refilled at a steady rate over time
/// - If the bucket is empty, requests wait until tokens are available
///
/// This helps prevent rate limit errors from the Anthropic API, which has a
/// limit of 50 requests per minute for most accounts.
#[derive(Debug)]
struct TokenBucketRateLimiter {
    /// Maximum number of tokens in the bucket
    capacity: u32,
    
    /// Current number of tokens in the bucket
    tokens: u32,
    
    /// Time of last token refill
    last_refill: Instant,
    
    /// Refill rate in tokens per second
    refill_rate: f64,
}

impl TokenBucketRateLimiter {
    /// Create a new token bucket rate limiter
    fn new(requests_per_minute: u32) -> Self {
        // Calculate tokens per second from requests per minute
        let refill_rate = requests_per_minute as f64 / 60.0;
        
        Self {
            capacity: requests_per_minute,
            tokens: requests_per_minute, // Start with a full bucket
            last_refill: Instant::now(),
            refill_rate,
        }
    }
    
    /// Refill the token bucket based on elapsed time
    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill);
        let elapsed_secs = elapsed.as_secs_f64();
        
        // Calculate how many tokens to add based on elapsed time and refill rate
        let new_tokens = (elapsed_secs * self.refill_rate).floor() as u32;
        
        if new_tokens > 0 {
            // Add tokens up to capacity
            self.tokens = (self.tokens + new_tokens).min(self.capacity);
            self.last_refill = now;
        }
    }
    
    /// Try to consume a token from the bucket
    async fn consume(&mut self) -> bool {
        self.refill();
        
        if self.tokens > 0 {
            self.tokens -= 1;
            true
        } else {
            false
        }
    }
    
    /// Wait until a token is available
    async fn wait_for_token(&mut self) {
        while !self.consume().await {
            // If no tokens are available, sleep for a short duration
            // Calculate time until next token is available
            let time_to_next_token_secs = 1.0 / self.refill_rate;
            let wait_ms = (time_to_next_token_secs * 1000.0).ceil() as u64;
            
            // Add small buffer to ensure token is ready
            sleep(Duration::from_millis(wait_ms + 10)).await;
            
            // Refill bucket after waiting
            self.refill();
        }
    }
}

/// Anthropic client for interacting with Anthropic API
#[derive(Debug)]
pub struct Anthropic {
    /// HTTP client for API requests
    client: Client,
    /// API key for authentication
    api_key: String,
    /// API endpoint URL (optional, defaults to public API)
    endpoint: String,
    /// Maximum number of retries for transient errors
    max_retries: u32,
    /// Initial backoff duration for retry in milliseconds
    initial_backoff_ms: u64,
    /// Rate limiter (optional)
    rate_limiter: Option<Mutex<TokenBucketRateLimiter>>,
}

/// Anthropic message request
#[derive(Debug, Serialize)]
pub struct AnthropicRequest {
    /// The model to use
    model: String,
    
    /// The messages for the conversation
    messages: Vec<AnthropicMessage>,
    
    /// System prompt to guide the AI
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    
    /// Temperature for generation
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    
    /// Maximum number of tokens to generate
    max_tokens: u32,
    
    /// Top probability mass to consider (nucleus sampling)
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    
    /// Top k tokens to consider
    #[serde(skip_serializing_if = "Option::is_none")]
    top_k: Option<u32>,
}

/// Anthropic message format
#[derive(Debug, Serialize, Deserialize)]
pub struct AnthropicMessage {
    /// Role of the message sender (user, assistant)
    pub role: String,
    
    /// Content of the message
    pub content: String,
}

/// Token usage information
#[derive(Debug, Deserialize)]
pub struct TokenUsage {
    /// Number of input tokens
    pub input_tokens: u32,
    /// Number of output tokens
    pub output_tokens: u32,
}

/// Anthropic response
#[derive(Debug, Deserialize)]
pub struct AnthropicResponse {
    /// The content of the response
    pub content: Vec<AnthropicContent>,
    /// Token usage information
    pub usage: TokenUsage,
}

/// Individual content block in an Anthropic response
#[derive(Debug, Deserialize)]
pub struct AnthropicContent {
    /// The type of content
    #[serde(rename = "type")]
    pub content_type: String,
    
    /// The actual text content
    pub text: String,
}

impl Default for AnthropicRequest {
    fn default() -> Self {
        Self {
            model: String::new(),
            messages: Vec::new(),
            system: None,
            temperature: Some(0.7),
            max_tokens: 4096,
            top_p: None,
            top_k: None,
        }
    }
}

impl AnthropicRequest {
    /// Create a new Anthropic request
    pub fn new(model: impl Into<String>, max_tokens: u32) -> Self {
        Self {
            model: model.into(),
            max_tokens,
            ..Default::default()
        }
    }
    
    /// Add a message to the request
    pub fn add_message(mut self, role: impl Into<String>, content: impl Into<String>) -> Self {
        self.messages.push(AnthropicMessage {
            role: role.into(),
            content: content.into(),
        });
        self
    }
    
    /// Set the system prompt
    pub fn system(mut self, system: impl Into<String>) -> Self {
        self.system = Some(system.into());
        self
    }
    
    /// Set the temperature
    pub fn temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }
    
    /// Set the top_p (nucleus sampling)
    pub fn top_p(mut self, top_p: f32) -> Self {
        self.top_p = Some(top_p);
        self
    }
    
    /// Set the top_k
    pub fn top_k(mut self, top_k: u32) -> Self {
        self.top_k = Some(top_k);
        self
    }
}

impl Anthropic {
    /// Create a new Anthropic client with simple configuration
    pub fn new(api_key: impl Into<String>, endpoint: impl Into<String>) -> Self {
        Self::new_with_config(api_key, endpoint, 3, 1000, None)
    }
    
    /// Create a new Anthropic client with rate limiting
    pub fn new_with_rate_limit(
        api_key: impl Into<String>,
        endpoint: impl Into<String>,
        requests_per_minute: u32,
    ) -> Self {
        Self::new_with_config(api_key, endpoint, 3, 1000, Some(requests_per_minute))
    }
    
    /// Create a new Anthropic client with combined configuration
    pub fn new_with_config(
        api_key: impl Into<String>,
        endpoint: impl Into<String>,
        max_retries: u32,
        initial_backoff_ms: u64,
        requests_per_minute: Option<u32>,
    ) -> Self {
        let rate_limiter = requests_per_minute
            .filter(|&rpm| rpm > 0)
            .map(|rpm| Mutex::new(TokenBucketRateLimiter::new(rpm)));
        
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(120))
                .build()
                .unwrap_or_default(),
            api_key: api_key.into(),
            endpoint: endpoint.into(),
            max_retries,
            initial_backoff_ms,
            rate_limiter,
        }
    }
    
    /// Generate API URL based on configured endpoint
    fn api_url(&self) -> String {
        if self.endpoint.is_empty() {
            "https://api.anthropic.com/v1/messages".to_string()
        } else {
            format!("{}/v1/messages", self.endpoint.trim_end_matches('/'))
        }
    }
    
    /// Send a request to the Anthropic API with retry logic
    async fn send_request_with_retry(&self, request: &AnthropicRequest) -> Result<AnthropicResponse, ProviderError> {
        let api_url = self.api_url();
        let mut attempts = 0;
        let mut last_error = None;
        
        while attempts <= self.max_retries {
            if attempts > 0 {
                let backoff_ms = self.initial_backoff_ms * 2u64.pow(attempts - 1);
                sleep(Duration::from_millis(backoff_ms)).await;
            }
            
            // Apply rate limiting if configured
            if let Some(rate_limiter) = &self.rate_limiter {
                rate_limiter.lock().await.wait_for_token().await;
            }
            
            attempts += 1;
            
            match self.send_request(&api_url, request).await {
                Ok(response) => return Ok(response),
                Err(err) => {
                    // Only retry on connection errors and rate limit errors
                    match &err {
                        ProviderError::ConnectionError(_) => {
                            last_error = Some(err);
                        },
                        ProviderError::RateLimitExceeded(_) => {
                            // For rate limit errors, apply a longer backoff
                            let rate_limit_backoff_ms = self.initial_backoff_ms * 5 * 2u64.pow(attempts - 1);
                            sleep(Duration::from_millis(rate_limit_backoff_ms)).await;
                            last_error = Some(err);
                        },
                        ProviderError::ApiError { status_code, .. } => {
                            // Retry on rate limiting (429) and server errors (5xx)
                            if *status_code == 429 || *status_code >= 500 {
                                last_error = Some(err);
                            } else {
                                // Don't retry on client errors (4xx) except rate limiting
                                return Err(err);
                            }
                        },
                        _ => return Err(err), // Don't retry on other errors
                    }
                }
            }
        }
        
        // If we get here, all retries failed
        Err(last_error.unwrap_or_else(|| 
            ProviderError::ConnectionError("All retry attempts failed".to_string())))
    }
    
    /// Send a single request to the Anthropic API
    async fn send_request(&self, api_url: &str, request: &AnthropicRequest) -> Result<AnthropicResponse, ProviderError> {
        // Add timeout to prevent hanging HTTP requests
        let request_future = self.client.post(api_url)
            .header("Content-Type", "application/json")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(request)
            .send();
        
        let timeout_duration = std::time::Duration::from_secs(60); // 1 minute timeout
        let response = tokio::select! {
            result = request_future => {
                result.map_err(|e| {
                    if e.is_timeout() {
                        ProviderError::ConnectionError(format!("Request timed out: {}", e))
                    } else if e.is_connect() {
                        ProviderError::ConnectionError(format!("Connection failed: {}", e))
                    } else {
                        ProviderError::RequestFailed(e.to_string())
                    }
                })?
            },
            _ = tokio::time::sleep(timeout_duration) => {
                return Err(ProviderError::ConnectionError("Anthropic API request timed out after 60 seconds".to_string()));
            }
        };
        
        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await
                .unwrap_or_else(|_| "Failed to get error response text".to_string());
            
            return match status.as_u16() {
                429 => Err(ProviderError::RateLimitExceeded(error_text)),
                401 | 403 => Err(ProviderError::AuthenticationError(error_text)),
                _ => Err(ProviderError::ApiError { 
                    status_code: status.as_u16(), 
                    message: error_text
                })
            };
        }
        
        response.json::<AnthropicResponse>().await
            .map_err(|e| ProviderError::ParseError(e.to_string()))
    }
}

#[async_trait]
impl Provider for Anthropic {
    type Request = AnthropicRequest;
    type Response = AnthropicResponse;
    
    /// Complete a messages request
    async fn complete(&self, request: Self::Request) -> Result<Self::Response, ProviderError> {
        self.send_request_with_retry(&request).await
    }
    
    /// Test the connection to the Anthropic API
    async fn test_connection(&self) -> Result<(), ProviderError> {
        let request = AnthropicRequest::new("claude-3-haiku-20240307", 10)
            .add_message("user", "Hello");
        
        self.complete(request).await?;
        Ok(())
    }
    
    /// Extract text from Anthropic response
    fn extract_text(response: &Self::Response) -> String {
        response.content.iter()
            .filter(|c| c.content_type == "text")
            .map(|c| c.text.clone())
            .collect()
    }
} 