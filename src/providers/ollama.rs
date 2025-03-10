use anyhow::{Result, anyhow, Context};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use log::{error, debug};

/// Ollama client for interacting with Ollama API
pub struct Ollama {
    /// Base URL of the Ollama API
    base_url: String,
    /// HTTP client for making requests
    client: Client,
}

/// Generate request for the Ollama API
#[derive(Debug, Serialize, Deserialize)]
pub struct GenerationRequest {
    /// Model name to use for generation
    model: String,
    /// Prompt to generate from
    prompt: String,
    /// System message to guide the model
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    /// Additional model parameters
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<GenerationOptions>,
    /// Format to return a response in
    #[serde(skip_serializing_if = "Option::is_none")]
    format: Option<String>,
    /// Context from previous generations
    #[serde(skip_serializing_if = "Option::is_none")]
    context: Option<Vec<i32>>,
    /// Whether to stream the response
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
    /// Whether to use raw prompting
    #[serde(skip_serializing_if = "Option::is_none")]
    raw: Option<bool>,
    /// How long to keep the model loaded in memory
    #[serde(skip_serializing_if = "Option::is_none")]
    keep_alive: Option<String>,
}

/// Generation options for the Ollama API
#[derive(Debug, Serialize, Deserialize)]
pub struct GenerationOptions {
    /// Temperature for generation (default: 0.8)
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    /// Top-p sampling (default: 0.9)
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    /// Top-k sampling (default: 40)
    #[serde(skip_serializing_if = "Option::is_none")]
    top_k: Option<u32>,
    /// Random seed for generation
    #[serde(skip_serializing_if = "Option::is_none")]
    seed: Option<u64>,
    /// Maximum number of tokens to generate
    #[serde(skip_serializing_if = "Option::is_none")]
    num_predict: Option<u32>,
}

/// Generation response from the Ollama API
#[derive(Debug, Serialize, Deserialize)]
pub struct GenerationResponse {
    /// Model name
    pub model: String,
    /// Creation timestamp
    pub created_at: String,
    /// Generated text
    pub response: String,
    /// Whether the generation is complete
    pub done: bool,
    /// Context for future generations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<Vec<i32>>,
    /// Total duration of the request in nanoseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_duration: Option<u64>,
    /// Duration of loading the model in nanoseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub load_duration: Option<u64>,
    /// Number of prompt tokens
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_eval_count: Option<u64>,
    /// Duration of prompt evaluation in nanoseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_eval_duration: Option<u64>,
    /// Number of generated tokens
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eval_count: Option<u64>,
    /// Duration of generation in nanoseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eval_duration: Option<u64>,
}

/// Chat message object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    /// Role of the message sender (system, user, assistant, or tool)
    pub role: String,
    /// Content of the message
    pub content: String,
}

/// Chat request for the Ollama API
#[derive(Debug, Serialize, Deserialize)]
pub struct ChatRequest {
    /// Model name to use for generation
    model: String,
    /// Messages of the conversation
    messages: Vec<ChatMessage>,
    /// System message to guide the model
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    /// Additional model parameters
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<GenerationOptions>,
    /// Format to return a response in
    #[serde(skip_serializing_if = "Option::is_none")]
    format: Option<String>,
    /// Whether to stream the response
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
    /// How long to keep the model loaded in memory
    #[serde(skip_serializing_if = "Option::is_none")]
    keep_alive: Option<String>,
}

/// Chat response from the Ollama API
#[derive(Debug, Serialize, Deserialize)]
pub struct ChatResponse {
    /// Model name
    pub model: String,
    /// Creation timestamp
    pub created_at: String,
    /// Response message
    pub message: ChatMessage,
    /// Whether the generation is complete
    pub done: bool,
    /// Total duration of the request in nanoseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_duration: Option<u64>,
    /// Duration of loading the model in nanoseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub load_duration: Option<u64>,
    /// Number of prompt tokens
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_eval_count: Option<u64>,
    /// Duration of prompt evaluation in nanoseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_eval_duration: Option<u64>,
    /// Number of generated tokens
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eval_count: Option<u64>,
    /// Duration of generation in nanoseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eval_duration: Option<u64>,
}

/// Embeddings request for the Ollama API
#[derive(Debug, Serialize, Deserialize)]
pub struct EmbeddingRequest {
    /// Model name to use for generation
    model: String,
    /// Prompt to generate embeddings for
    prompt: String,
    /// Additional model parameters
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<GenerationOptions>,
    /// How long to keep the model loaded in memory
    #[serde(skip_serializing_if = "Option::is_none")]
    keep_alive: Option<String>,
}

/// Embeddings response from the Ollama API
#[derive(Debug, Serialize, Deserialize)]
pub struct EmbeddingResponse {
    /// Embedding vector
    pub embedding: Vec<f32>,
}

impl GenerationRequest {
    /// Create a new generation request
    pub fn new(model: impl Into<String>, prompt: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            prompt: prompt.into(),
            system: None,
            options: None,
            format: None,
            context: None,
            stream: Some(false),
            raw: None,
            keep_alive: None,
        }
    }

    /// Set the system prompt
    pub fn system(mut self, system: impl Into<String>) -> Self {
        self.system = Some(system.into());
        self
    }

    /// Set the temperature
    pub fn temperature(mut self, temperature: f32) -> Self {
        if self.options.is_none() {
            self.options = Some(GenerationOptions {
                temperature: Some(temperature),
                top_p: None,
                top_k: None,
                seed: None,
                num_predict: None,
            });
        } else if let Some(options) = &mut self.options {
            options.temperature = Some(temperature);
        }
        self
    }

    /// Set the format
    pub fn format(mut self, format: impl Into<String>) -> Self {
        self.format = Some(format.into());
        self
    }

    /// Set the keep-alive duration
    pub fn keep_alive(mut self, keep_alive: impl Into<String>) -> Self {
        self.keep_alive = Some(keep_alive.into());
        self
    }

    /// Disable streaming for this request
    pub fn no_stream(mut self) -> Self {
        self.stream = Some(false);
        self
    }
}

impl ChatRequest {
    /// Create a new chat request
    pub fn new(model: impl Into<String>, messages: Vec<ChatMessage>) -> Self {
        Self {
            model: model.into(),
            messages,
            system: None,
            options: None,
            format: None,
            stream: Some(false),
            keep_alive: None,
        }
    }

    /// Set the system prompt
    pub fn system(mut self, system: impl Into<String>) -> Self {
        self.system = Some(system.into());
        self
    }

    /// Set the temperature
    pub fn temperature(mut self, temperature: f32) -> Self {
        if self.options.is_none() {
            self.options = Some(GenerationOptions {
                temperature: Some(temperature),
                top_p: None,
                top_k: None,
                seed: None,
                num_predict: None,
            });
        } else if let Some(options) = &mut self.options {
            options.temperature = Some(temperature);
        }
        self
    }

    /// Set the format
    pub fn format(mut self, format: impl Into<String>) -> Self {
        self.format = Some(format.into());
        self
    }

    /// Set the keep-alive duration
    pub fn keep_alive(mut self, keep_alive: impl Into<String>) -> Self {
        self.keep_alive = Some(keep_alive.into());
        self
    }

    /// Disable streaming
    pub fn no_stream(mut self) -> Self {
        self.stream = Some(false);
        self
    }
}

impl Ollama {
    /// Create a new Ollama client with the specified base URL
    pub fn new(host: impl Into<String>, port: u16) -> Self {
        let host = host.into();
        let base_url = format!("{}:{}", host, port);
        
        Self {
            base_url,
            client: Client::builder()
                .timeout(Duration::from_secs(60))
                .build()
                .unwrap_or_default(),
        }
    }
    
    /// Create a new Ollama client from a complete URL
    pub fn from_url(url: impl Into<String>) -> Self {
        Self {
            base_url: url.into(),
            client: Client::builder()
                .timeout(Duration::from_secs(60))
                .build()
                .unwrap_or_default(),
        }
    }
    
    /// Generate text from the Ollama API
    pub async fn generate(&self, request: GenerationRequest) -> Result<GenerationResponse> {
        let url = format!("{}/api/generate", self.base_url);
        
        let response = self.client.post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send request to Ollama API: {}", e))?;
        
        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await
                .unwrap_or_else(|_| "Failed to get error response text".to_string());
            error!("Ollama API error ({}): {}", status, error_text);
            return Err(anyhow!("Ollama API error ({}): {}", status, error_text));
        }
        
        // Get the raw response text first
        let response_text = response.text().await
            .map_err(|e| anyhow!("Failed to get response text from Ollama API: {}", e))?;
        
        // Try to parse as single JSON object first
        match serde_json::from_str::<GenerationResponse>(&response_text) {
            Ok(generated_response) => {
                Ok(generated_response)
            },
            Err(e) => {
                // Log the raw response for debugging
                error!("Failed to parse Ollama API response: {}. Raw response (first 500 chars): {}", 
                      e, if response_text.chars().count() > 500 { 
                          response_text.chars().take(500).collect::<String>() 
                      } else { 
                          response_text.clone() 
                      });
                
                // The response might be in JSONL format (streaming response)
                // Split by lines and try to parse each as a JSON object
                let lines: Vec<&str> = response_text.lines().collect();
                
                if !lines.is_empty() {
                    // Try to parse the last line, which should contain the final state
                    for line in lines.iter().rev() {
                        if line.is_empty() {
                            continue;
                        }
                        
                        if let Ok(value) = serde_json::from_str::<serde_json::Value>(line) {
                            // Check if it's a "done" message
                            if value.get("done").and_then(|v| v.as_bool()).unwrap_or(false) {
                                // Found the final response
                                let model = value.get("model").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
                                let created_at = value.get("created_at").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                
                                // For streaming responses, we need to concatenate all the pieces
                                let mut full_response = String::new();
                                for line in lines.iter() {
                                    if let Ok(obj) = serde_json::from_str::<serde_json::Value>(line) {
                                        if let Some(part) = obj.get("response").and_then(|v| v.as_str()) {
                                            full_response.push_str(part);
                                        }
                                    }
                                }
                                
                                // Extract optional numeric fields if available
                                let prompt_eval_count = value.get("prompt_eval_count").and_then(|v| v.as_u64());
                                let eval_count = value.get("eval_count").and_then(|v| v.as_u64());
                                
                                return Ok(GenerationResponse {
                                    model,
                                    created_at,
                                    response: full_response,
                                    done: true,
                                    context: None,
                                    total_duration: None,
                                    load_duration: None,
                                    prompt_eval_count,
                                    prompt_eval_duration: None,
                                    eval_count,
                                    eval_duration: None,
                                });
                            }
                        }
                    }
                    
                    // If we didn't find a "done" message, try to use the last valid JSON object
                    if let Ok(value) = serde_json::from_str::<serde_json::Value>(lines[lines.len() - 1]) {
                        let model = value.get("model").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
                        let created_at = value.get("created_at").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        let response_text = value.get("response").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        
                        // Extract optional numeric fields if available
                        let prompt_eval_count = value.get("prompt_eval_count").and_then(|v| v.as_u64());
                        let eval_count = value.get("eval_count").and_then(|v| v.as_u64());
                        
                        return Ok(GenerationResponse {
                            model,
                            created_at,
                            response: response_text,
                            done: true,
                            context: None,
                            total_duration: None,
                            load_duration: None,
                            prompt_eval_count,
                            prompt_eval_duration: None,
                            eval_count,
                            eval_duration: None,
                        });
                    }
                }
                
                // If we still can't parse the response, try our original lenient approach
                match serde_json::from_str::<serde_json::Value>(&response_text) {
                    Ok(value) => {
                        // Try to extract essential fields
                        let model = value.get("model").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
                        let response_text = value.get("response").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        let created_at = value.get("created_at").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        let done = value.get("done").and_then(|v| v.as_bool()).unwrap_or(true);
                        
                        // Extract optional numeric fields if available
                        let prompt_eval_count = value.get("prompt_eval_count").and_then(|v| v.as_u64());
                        let eval_count = value.get("eval_count").and_then(|v| v.as_u64());
                        
                        // Create a response with the extracted fields
                        Ok(GenerationResponse {
                            model,
                            created_at,
                            response: response_text,
                            done,
                            context: None,
                            total_duration: None,
                            load_duration: None,
                            prompt_eval_count,
                            prompt_eval_duration: None,
                            eval_count,
                            eval_duration: None,
                        })
                    },
                    Err(_) => {
                        // If we can't even parse as a JSON Value, return the original error
                        Err(anyhow!("Failed to parse Ollama API response: {}. Response contains invalid JSON.", e))
                    }
                }
            }
        }
    }
    
    /// Chat with the Ollama API
    pub async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
        let url = format!("{}/api/chat", self.base_url);
        
        let response = self.client.post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send chat request to Ollama API: {}", e))?;
        
        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await
                .unwrap_or_else(|_| "Failed to get error response text".to_string());
            error!("Ollama API error ({}): {}", status, error_text);
            return Err(anyhow!("Ollama API error ({}): {}", status, error_text));
        }
        
        // Get the raw response text first
        let response_text = response.text().await
            .map_err(|e| anyhow!("Failed to get response text from Ollama API: {}", e))?;
        
        // Try to parse as single JSON object first
        match serde_json::from_str::<ChatResponse>(&response_text) {
            Ok(chat_response) => {
                Ok(chat_response)
            },
            Err(e) => {
                // Log the raw response for debugging
                error!("Failed to parse Ollama API chat response: {}. Raw response (first 500 chars): {}", 
                      e, if response_text.chars().count() > 500 { 
                          response_text.chars().take(500).collect::<String>() 
                      } else { 
                          response_text.clone() 
                      });
                
                // The response might be in JSONL format (streaming response)
                // Split by lines and try to parse each as a JSON object
                let lines: Vec<&str> = response_text.lines().collect();
                
                if !lines.is_empty() {
                    // Try to parse the last line, which should contain the final state
                    for line in lines.iter().rev() {
                        if line.is_empty() {
                            continue;
                        }
                        
                        if let Ok(value) = serde_json::from_str::<serde_json::Value>(line) {
                            // Check if it's a "done" message
                            if value.get("done").and_then(|v| v.as_bool()).unwrap_or(false) {
                                // Found the final response
                                let model = value.get("model").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
                                let created_at = value.get("created_at").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                
                                // For streaming responses, we need to concatenate all the message content pieces
                                let mut full_content = String::new();
                                for line in lines.iter() {
                                    if let Ok(obj) = serde_json::from_str::<serde_json::Value>(line) {
                                        if let Some(message) = obj.get("message") {
                                            if let Some(part) = message.get("content").and_then(|v| v.as_str()) {
                                                full_content.push_str(part);
                                            }
                                        }
                                    }
                                }
                                
                                // Extract optional numeric fields if available
                                let prompt_eval_count = value.get("prompt_eval_count").and_then(|v| v.as_u64());
                                let eval_count = value.get("eval_count").and_then(|v| v.as_u64());
                                
                                return Ok(ChatResponse {
                                    model,
                                    created_at,
                                    message: ChatMessage {
                                        role: "assistant".to_string(), 
                                        content: full_content,
                                    },
                                    done: true,
                                    total_duration: None,
                                    load_duration: None,
                                    prompt_eval_count,
                                    prompt_eval_duration: None,
                                    eval_count,
                                    eval_duration: None,
                                });
                            }
                        }
                    }
                    
                    // If we didn't find a "done" message, try to use the last valid JSON object
                    if let Ok(value) = serde_json::from_str::<serde_json::Value>(lines[lines.len() - 1]) {
                        let model = value.get("model").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
                        let created_at = value.get("created_at").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        
                        // Extract message content
                        let content = if let Some(message) = value.get("message") {
                            message.get("content").and_then(|v| v.as_str()).unwrap_or("").to_string()
                        } else {
                            "".to_string()
                        };
                        
                        // Extract optional numeric fields if available
                        let prompt_eval_count = value.get("prompt_eval_count").and_then(|v| v.as_u64());
                        let eval_count = value.get("eval_count").and_then(|v| v.as_u64());
                        
                        return Ok(ChatResponse {
                            model,
                            created_at,
                            message: ChatMessage {
                                role: "assistant".to_string(),
                                content,
                            },
                            done: true,
                            total_duration: None,
                            load_duration: None,
                            prompt_eval_count,
                            prompt_eval_duration: None,
                            eval_count,
                            eval_duration: None,
                        });
                    }
                }
                
                // If we still can't parse the response, try our original lenient approach
                match serde_json::from_str::<serde_json::Value>(&response_text) {
                    Ok(value) => {
                        // Try to extract essential fields
                        let model = value.get("model").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
                        let created_at = value.get("created_at").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        let done = value.get("done").and_then(|v| v.as_bool()).unwrap_or(true);
                        
                        // Extract message content
                        let content = if let Some(message) = value.get("message") {
                            message.get("content").and_then(|v| v.as_str()).unwrap_or("").to_string()
                        } else {
                            "".to_string()
                        };
                        
                        // Extract optional numeric fields if available
                        let prompt_eval_count = value.get("prompt_eval_count").and_then(|v| v.as_u64());
                        let eval_count = value.get("eval_count").and_then(|v| v.as_u64());
                        
                        // Create a response with the extracted fields
                        Ok(ChatResponse {
                            model,
                            created_at,
                            message: ChatMessage {
                                role: "assistant".to_string(),
                                content,
                            },
                            done,
                            total_duration: None,
                            load_duration: None,
                            prompt_eval_count,
                            prompt_eval_duration: None,
                            eval_count,
                            eval_duration: None,
                        })
                    },
                    Err(_) => {
                        // If we can't even parse as a JSON Value, return the original error
                        Err(anyhow!("Failed to parse Ollama API chat response: {}. Response contains invalid JSON.", e))
                    }
                }
            }
        }
    }
    
    /// Generate embeddings from the Ollama API
    pub async fn embed(&self, model: impl Into<String>, prompt: impl Into<String>) -> Result<EmbeddingResponse> {
        let url = format!("{}/api/embeddings", self.base_url);
        
        let request = EmbeddingRequest {
            model: model.into(),
            prompt: prompt.into(),
            options: None,
            keep_alive: None,
        };
        
        let response = self.client.post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send embeddings request to Ollama API: {}", e))?;
        
        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await
                .unwrap_or_else(|_| "Failed to get error response text".to_string());
            error!("Ollama API error ({}): {}", status, error_text);
            return Err(anyhow!("Ollama API error ({}): {}", status, error_text));
        }
        
        // Get the raw response text first
        let response_text = response.text().await
            .map_err(|e| anyhow!("Failed to get response text from Ollama API: {}", e))?;
        
        // Try to parse the response
        match serde_json::from_str::<EmbeddingResponse>(&response_text) {
            Ok(embedding_response) => {
                Ok(embedding_response)
            },
            Err(e) => {
                // Log the raw response for debugging
                error!("Failed to parse Ollama API embeddings response: {}. Raw response (first 500 chars): {}", 
                      e, if response_text.chars().count() > 500 { 
                          response_text.chars().take(500).collect::<String>() 
                      } else { 
                          response_text.clone() 
                      });
                
                // Try a more lenient approach - parse as Value first
                match serde_json::from_str::<serde_json::Value>(&response_text) {
                    Ok(value) => {
                        // Try to extract embedding array
                        if let Some(embedding) = value.get("embedding").and_then(|v| v.as_array()) {
                            // Convert to vector of f32
                            let embedding_vec: Vec<f32> = embedding.iter()
                                .filter_map(|v| v.as_f64().map(|f| f as f32))
                                .collect();
                            
                            if !embedding_vec.is_empty() {
                                return Ok(EmbeddingResponse { embedding: embedding_vec });
                            }
                        }
                        
                        // If we couldn't extract the embedding, return an error
                        Err(anyhow!("Failed to extract embedding from Ollama API response"))
                    },
                    Err(_) => {
                        // If we can't even parse as a JSON Value, return the original error
                        Err(anyhow!("Failed to parse Ollama API embeddings response: {}. Response contains invalid JSON.", e))
                    }
                }
            }
        }
    }
    
    /// Get the Ollama API version
    pub async fn version(&self) -> Result<String> {
        let url = format!("{}/api/version", self.base_url);
        
        let response = self.client.get(&url)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to get Ollama API version: {}", e))?;
        
        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await
                .unwrap_or_else(|_| "Failed to get error response text".to_string());
            error!("Ollama API error ({}): {}", status, error_text);
            return Err(anyhow!("Ollama API error ({}): {}", status, error_text));
        }
        
        // Get the raw response text first
        let response_text = response.text().await
            .map_err(|e| anyhow!("Failed to get response text from Ollama API: {}", e))?;
        
        // Try to parse the response
        match serde_json::from_str::<serde_json::Value>(&response_text) {
            Ok(value) => {
                if let Some(version) = value.get("version").and_then(|v| v.as_str()) {
                    Ok(version.to_string())
                } else {
                    // Log the raw response for debugging
                    error!("Invalid Ollama API version format. Raw response: {}", response_text);
                    Err(anyhow!("Invalid Ollama API version format"))
                }
            },
            Err(e) => {
                // Log the raw response for debugging
                error!("Failed to parse Ollama API version response: {}. Raw response: {}", e, response_text);
                Err(anyhow!("Failed to parse Ollama API version response: {}", e))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    #[ignore]
    async fn test_ollama_generate() {
        // This test should only run if Ollama is available locally
        let client = Ollama::new("http://localhost", 11434);
        
        // Try to get the version, if it fails, skip the test
        if client.version().await.is_err() {
            debug!("Skipping test because Ollama is not available");
            return;
        }
        
        let request = GenerationRequest::new("llama2", "Hello, world!")
            .system("You are a helpful assistant.")
            .temperature(0.7);
        
        let response = client.generate(request).await;
        assert!(response.is_ok());
    }
    
    #[tokio::test]
    #[ignore]
    async fn test_ollama_chat() {
        // This test should only run if Ollama is available locally
        let client = Ollama::new("http://localhost", 11434);
        
        // Try to get the version, if it fails, skip the test
        if client.version().await.is_err() {
            debug!("Skipping test because Ollama is not available");
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
} 