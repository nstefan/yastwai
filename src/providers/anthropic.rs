use std::time::Duration;
use serde::{Serialize, Deserialize};
use anyhow::{Result, anyhow, Context};
use reqwest::{Client, header};
use log::error;

/// Anthropic client for interacting with Anthropic API
pub struct Anthropic {
    /// HTTP client for API requests
    client: Client,
    /// API key for authentication
    api_key: String,
    /// API endpoint URL (optional, defaults to public API)
    endpoint: String,
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
    /// Create a new Anthropic client
    pub fn new(api_key: impl Into<String>, endpoint: impl Into<String>) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(120))
                .build()
                .unwrap_or_default(),
            api_key: api_key.into(),
            endpoint: endpoint.into(),
        }
    }
    
    /// Complete a messages request
    pub async fn complete(&self, request: AnthropicRequest) -> Result<AnthropicResponse> {
        let api_url = if self.endpoint.is_empty() {
            "https://api.anthropic.com/v1/messages".to_string()
        } else {
            format!("{}/v1/messages", self.endpoint.trim_end_matches('/'))
        };
        
        let response = self.client.post(&api_url)
            .header("Content-Type", "application/json")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&request)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send request to Anthropic API: {}", e))?;
        
        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await
                .unwrap_or_else(|_| "Failed to get error response text".to_string());
            error!("Anthropic API error ({}): {}", status, error_text);
            return Err(anyhow!("Anthropic API error ({}): {}", status, error_text));
        }
        
        let anthropic_response = response.json::<AnthropicResponse>().await
            .map_err(|e| anyhow!("Failed to parse Anthropic API response: {}", e))?;
            
        Ok(anthropic_response)
    }
    
    /// Test the connection to the Anthropic API
    pub async fn test_connection(&self, model: &str) -> Result<()> {
        let request = AnthropicRequest::new(model, 10)
            .add_message("user", "Hello");
        
        self.complete(request).await?;
        Ok(())
    }
    
    /// Extract text from Anthropic response
    pub fn extract_text_from_response(response: &AnthropicResponse) -> String {
        response.content.iter()
            .filter(|c| c.content_type == "text")
            .map(|c| c.text.clone())
            .collect()
    }
} 