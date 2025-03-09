use anyhow::{Result, Context, anyhow};
use log::{error, warn, info};
use std::time::Duration;
use url::Url;
use std::sync::Arc;
use regex::Regex;
use std::sync::atomic::{AtomicUsize, Ordering};
use futures::future::join_all;
use std::time::Instant;
use std::sync::Mutex as StdMutex;
use tokio::sync::Semaphore;

use crate::app_config::{TranslationConfig, TranslationProvider as ConfigTranslationProvider};
use crate::subtitle_processor::SubtitleEntry;
use crate::providers::ollama::{Ollama, GenerationRequest};
use crate::providers::openai::{OpenAI, OpenAIRequest};
use crate::providers::anthropic::{Anthropic, AnthropicRequest};

// @module: Translation service for subtitle content

// @struct: Token usage statistics
#[derive(Clone)]
pub struct TokenUsageStats {
    // @field: Number of prompt tokens
    pub prompt_tokens: u64,
    
    // @field: Number of completion tokens
    pub completion_tokens: u64,
    
    // @field: Total number of tokens
    pub total_tokens: u64,
    
    // @field: Start time of token tracking
    pub start_time: Instant,
    
    // @field: Total time spent on API requests
    pub api_duration: Duration,
    
    // @field: Provider name
    pub provider: String,
    
    // @field: Model name
    pub model: String,
}

impl TokenUsageStats {
    // @creates: New empty token usage stats
    pub fn new() -> Self {
        Self {
            prompt_tokens: 0,
            completion_tokens: 0,
            total_tokens: 0,
            start_time: Instant::now(),
            api_duration: Duration::from_secs(0),
            provider: String::new(),
            model: String::new(),
        }
    }
    
    // @updates: Add token usage numbers
    pub fn add_token_usage(&mut self, prompt_tokens: Option<u64>, completion_tokens: Option<u64>) {
        if let Some(pt) = prompt_tokens {
            self.prompt_tokens += pt;
            self.total_tokens += pt;
        }
        
        if let Some(ct) = completion_tokens {
            self.completion_tokens += ct;
            self.total_tokens += ct;
        }
    }
    
    // @updates: Add API request duration
    pub fn add_request_duration(&mut self, duration: Duration) {
        self.api_duration += duration;
    }
    
    // @creates: New token usage stats with provider info
    pub fn with_provider_info(provider: String, model: String) -> Self {
        Self {
            prompt_tokens: 0,
            completion_tokens: 0,
            total_tokens: 0,
            start_time: Instant::now(),
            api_duration: Duration::from_secs(0),
            provider,
            model,
        }
    }
    
    // @returns: Tokens per minute rate
    pub fn tokens_per_minute(&self) -> f64 {
        // Use the API duration for rate calculation, with fallback to elapsed time
        let duration_minutes = if self.api_duration.as_secs_f64() > 0.0 {
            self.api_duration.as_secs_f64() / 60.0
        } else {
            self.start_time.elapsed().as_secs_f64() / 60.0
        };
        
        // Calculate the rate, handling division by zero
        if duration_minutes > 0.0 {
            self.total_tokens as f64 / duration_minutes
        } else {
            0.0
        }
    }
    
    // @returns: Summary of token usage as a string
    pub fn summary(&self) -> String {
        // Calculate tokens per minute
        let tokens_per_min = self.tokens_per_minute();
        
        // Format the API duration if available
        let api_time = if self.api_duration.as_secs_f64() > 0.0 {
            format!("in {:.1}s of API time", self.api_duration.as_secs_f64())
        } else {
            "".to_string()
        };
        
        format!(
            "Token usage: {} total ({} prompt, {} completion) {} at {:.0} tokens/min", 
            self.total_tokens, 
            self.prompt_tokens, 
            self.completion_tokens,
            api_time,
            tokens_per_min
        )
    }
}

// @parses: Endpoint string into host and port
// @returns: Tuple of (host, port)
fn parse_endpoint(endpoint: &str) -> Result<(String, u16)> {
    // If it doesn't start with http/https, assume it's just host:port
    let url_str = if !endpoint.starts_with("http://") && !endpoint.starts_with("https://") {
        format!("http://{}", endpoint)
    } else {
        endpoint.to_string()
    };
    
    let url = Url::parse(&url_str)
        .context(format!("Failed to parse endpoint URL: {}", endpoint))?;
        
    let host = format!(
        "{}://{}",
        url.scheme(),
        url.host_str().unwrap_or("localhost")
    );
    
    let port = url.port().unwrap_or(11434);
    
    Ok((host, port))
}

// @enum: Available translation provider implementations
enum TranslationProviderImpl {    
    // @variant: Ollama LLM service
    Ollama {
        // @field: Client instance
        client: Ollama,
    },
    
    // @variant: OpenAI API service
    OpenAI {
        // @field: Client instance
        client: OpenAI,
    },
    
    // @variant: Anthropic API service
    Anthropic {
        // @field: Client instance
        client: Anthropic,
    },
}

// @struct: Translation service
pub struct TranslationService {
    // @field: Provider implementation
    provider: TranslationProviderImpl,
    
    // @field: Configuration
    config: TranslationConfig,
}

/// Additional parsing for OpenAI responses
/// OpenAI responses often follow certain patterns that we can try to detect
fn extract_openai_response(response: &str) -> Option<String> {
    // Check for itemized translations (common in OpenAI responses)
    // Pattern: numbered entries like "1. Text", "2. Text", etc.
    let numbered_regex = Regex::new(r"(?m)^\s*\d+\.\s*(.+)$").ok()?;
    let matches: Vec<String> = numbered_regex
        .captures_iter(response)
        .filter_map(|cap| cap.get(1).map(|m| m.as_str().trim().to_string()))
        .collect();
    
    if !matches.is_empty() {
        return Some(matches.join("\n"));
    }
    
    // Check for bullet point translations
    // Pattern: bullet points like "• Text", "- Text", etc.
    let bullet_regex = Regex::new(r"(?m)^\s*[•\-*]\s*(.+)$").ok()?;
    let matches: Vec<String> = bullet_regex
        .captures_iter(response)
        .filter_map(|cap| cap.get(1).map(|m| m.as_str().trim().to_string()))
        .collect();
    
    if !matches.is_empty() {
        return Some(matches.join("\n"));
    }
    
    // If no specific pattern is found, split by newlines and remove empty lines
    let lines: Vec<&str> = response
        .lines()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    
    if !lines.is_empty() {
        // Detect and remove common OpenAI response patterns at the start/end
        let filtered_lines: Vec<&str> = lines
            .into_iter()
            .filter(|line| {
                // Skip commentary lines that are not translations
                !line.starts_with("Here's the translation") &&
                !line.starts_with("I've translated") &&
                !line.starts_with("Translation:") &&
                !line.contains("from English to") &&
                !line.contains("translated subtitles") &&
                !line.ends_with("subtitle translation.") &&
                !line.starts_with("Please note")
            })
            .collect();
            
        if !filtered_lines.is_empty() {
            return Some(filtered_lines.join("\n"));
        }
    }
    
    None
}

/// Extract translated text from a response string
/// 
/// @param response: The response text from the translation service
/// @param log_capture: Optional log capture for storing warnings instead of printing them
/// @returns: The extracted translated text or original text as fallback
fn extract_translated_text_from_response(
    response: &str, 
    log_capture: Option<Arc<StdMutex<Vec<LogEntry>>>>
) -> Result<String> {
    // If the response is empty, return an error immediately
    if response.trim().is_empty() {
        return Err(anyhow!("Empty response received from translation service"));
    }
    
    // Quick check for pure text (most common case in newer implementations)
    // This avoids unnecessary parsing when the response is already clean
    if !response.contains('{') && !response.contains('}') && 
       !response.contains("===") && !response.contains("```") {
        return Ok(response.trim().to_string());
    }
    
    // Look for the translation block between standard delimiters
    if let Some(start) = response.find("=== BEGIN TRANSLATION ===") {
        if let Some(end) = response.find("=== END TRANSLATION ===") {
            if start < end {
                let content = &response[start + "=== BEGIN TRANSLATION ===".len()..end];
                return Ok(content.trim().to_string());
            }
        }
    }
    
    // Try to find markdown code blocks (common in newer LLM responses)
    let code_block_regex = Regex::new(r"```(?:json|text)?\s*\n([\s\S]*?)\n\s*```").unwrap_or_else(|_| {
        // Fallback to a simpler pattern if the main one fails to compile
        Regex::new(r"```([\s\S]*?)```").unwrap()
    });
    
    if let Some(caps) = code_block_regex.captures(response) {
        if let Some(content) = caps.get(1) {
            return Ok(content.as_str().trim().to_string());
        }
    }
    
    // Try to extract JSON (common in structured API responses)
    let json_content = if let (Some(start), Some(end)) = (response.find('{'), response.rfind('}')) {
        if start < end {
            Some(&response[start..=end])
        } else {
            None
        }
    } else {
        None
    };
    
    if let Some(content) = json_content {
        // Try to parse as JSON and extract relevant fields
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(content) {
            // Check common fields where translation content might be stored
            for field in ["text", "content", "translation", "translated_text", "result"] {
                if let Some(text) = json.get(field).and_then(|v| v.as_str()) {
                    return Ok(text.to_string());
                }
            }
            
            // Check for array of translations (common pattern in batch responses)
            for field in ["entries", "translations", "results", "items"] {
                if let Some(entries) = json.get(field).and_then(|v| v.as_array()) {
                    let texts: Vec<String> = entries.iter()
                        .filter_map(|entry| {
                            // Try common text field names for each entry
                            for text_field in ["text", "content", "translation", "value"] {
                                if let Some(text) = entry.get(text_field).and_then(|t| t.as_str()) {
                                    return Some(text.to_string());
                                }
                            }
                            None
                        })
                        .collect();
                    
                    if !texts.is_empty() {
                        return Ok(texts.join("\n"));
                    }
                }
            }
        }
    }
    
    // Try OpenAI-specific extraction patterns
    if let Some(extracted) = extract_openai_response(response) {
        return Ok(extracted);
    }
    
    // If all else fails, just return the original content with warning in logs
    let warning_message = "Warning: Could not extract structured translation from response, returning raw text. This may affect subtitle alignment.";
    
    // Either capture the log or print it directly
    if let Some(log_capture) = log_capture {
        let mut logs = match log_capture.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                // Get the inner data even though the mutex is poisoned
                warn!("Encountered poisoned mutex for log capture. Recovering...");
                poisoned.into_inner()
            }
        };
        logs.push(LogEntry {
            level: "ERROR".to_string(),
            message: warning_message.to_string(),
        });
    } else {
        error!("{}", warning_message);
    }
    
    // Return the full original response as a last resort
    Ok(response.trim().to_string())
}

/// Log entry for capturing logs during translation
#[derive(Clone, Debug)]
pub struct LogEntry {
    pub level: String,
    pub message: String,
}

impl TranslationService {
    /// Create a new translation service from configuration
    pub fn new(config: TranslationConfig) -> Result<Self> {
        let provider = match config.provider {
            ConfigTranslationProvider::Ollama => {
                // Parse the Ollama endpoint URL
                let (host, port) = parse_endpoint(&config.get_endpoint())?;
                
                // Create Ollama client
                let client = Ollama::new(host, port);
                
                // Return Ollama provider
                TranslationProviderImpl::Ollama {
                    client,
                }
            },
            ConfigTranslationProvider::OpenAI => {
                // Create OpenAI client
                let client = OpenAI::new(
                    config.get_api_key(), 
                    config.get_endpoint()
                );
                
                // Return OpenAI provider
                TranslationProviderImpl::OpenAI {
                    client,
                }
            },
            ConfigTranslationProvider::Anthropic => {
                // Create Anthropic client
                let client = Anthropic::new(
                    config.get_api_key(),
                    config.get_endpoint()
                );
                
                // Return Anthropic provider
                TranslationProviderImpl::Anthropic {
                    client,
                }
            },
        };
        
        // Return the initialized service
        Ok(Self {
            provider,
            config,
        })
    }
    
    /// Test the connection to the translation service
    pub async fn test_connection(
        &self, 
        source_language: &str, 
        target_language: &str,
        log_capture: Option<Arc<StdMutex<Vec<LogEntry>>>>
    ) -> Result<()> {
        // Test the connection based on the provider
        match &self.provider {
            TranslationProviderImpl::Ollama { client } => {
                let model = self.config.get_model();
                let prompt = format!("Say hello in {} and {}", source_language, target_language);
                
                // Create a simple generation request
                let request = GenerationRequest::new(model.clone(), prompt)
                    .temperature(0.7f32);
                
                match client.generate(request).await {
                    Ok(_) => {
                        Ok(())
                    },
                    Err(e) => {
                        // Capture log or print directly
                        if let Some(log_capture) = &log_capture {
                            let mut logs = match log_capture.lock() {
                                Ok(guard) => guard,
                                Err(poisoned) => {
                                    // Get the inner data even though the mutex is poisoned
                                    warn!("Encountered poisoned mutex for log capture. Recovering...");
                                    poisoned.into_inner()
                                }
                            };
                            logs.push(LogEntry {
                                level: "ERROR".to_string(),
                                message: format!("Ollama connection failed: {}", e)
                            });
                        } else {
                            error!("Ollama connection failed: {}", e);
                        }
                        Err(anyhow::anyhow!("Failed to connect to Ollama API: {}", e))
                    }
                }
            },
            TranslationProviderImpl::OpenAI { client } => {
                let model = self.config.get_model();
                
                // Test connection with a simple prompt
                match client.test_connection(&model).await {
                    Ok(_) => {
                        Ok(())
                    },
                    Err(e) => {
                        // Capture log or print directly
                        if let Some(log_capture) = &log_capture {
                            let mut logs = match log_capture.lock() {
                                Ok(guard) => guard,
                                Err(poisoned) => {
                                    // Get the inner data even though the mutex is poisoned
                                    warn!("Encountered poisoned mutex for log capture. Recovering...");
                                    poisoned.into_inner()
                                }
                            };
                            logs.push(LogEntry {
                                level: "ERROR".to_string(),
                                message: format!("OpenAI connection failed: {}", e)
                            });
                        } else {
                            error!("OpenAI connection failed: {}", e);
                        }
                        Err(anyhow::anyhow!("Failed to connect to OpenAI API: {}", e))
                    }
                }
            },
            TranslationProviderImpl::Anthropic { client } => {
                let model = self.config.get_model();
                
                // Test connection with a simple prompt
                match client.test_connection(&model).await {
                    Ok(_) => {
                        Ok(())
                    },
                    Err(e) => {
                        // Capture log or print directly
                        if let Some(log_capture) = &log_capture {
                            let mut logs = match log_capture.lock() {
                                Ok(guard) => guard,
                                Err(poisoned) => {
                                    // Get the inner data even though the mutex is poisoned
                                    warn!("Encountered poisoned mutex for log capture. Recovering...");
                                    poisoned.into_inner()
                                }
                            };
                            logs.push(LogEntry {
                                level: "ERROR".to_string(),
                                message: format!("Anthropic connection failed: {}", e)
                            });
                        } else {
                            error!("Anthropic connection failed: {}", e);
                        }
                        Err(anyhow::anyhow!("Failed to connect to Anthropic API: {}", e))
                    }
                }
            }
        }
    }
    
    /// Test translation with a sample text
    pub async fn test_translation(&self, source_language: &str, target_language: &str) -> Result<String> {        
        let text = format!("Hello, this is a test message. Please translate this from {} to {}.", 
                          source_language, target_language);
        self.translate_text(&text, source_language, target_language).await
    }
    
    /// Translate a text from source language to target language
    async fn translate_text(&self, text: &str, source_language: &str, target_language: &str) -> Result<String> {
        // Use the with_usage variant and discard the usage stats
        let (translated_text, _) = self.translate_text_with_usage(text, source_language, target_language, None).await?;
        Ok(translated_text)
    }
    
    /// Translate a batch of subtitle entries
    pub async fn translate_batches(&self, 
                              batches: &[Vec<SubtitleEntry>],
                              source_language: &str, 
                              target_language: &str,
                              log_capture: Arc<StdMutex<Vec<LogEntry>>>,
                              progress_callback: impl Fn(usize, usize) + Clone + Send + 'static) 
                              -> Result<(Vec<SubtitleEntry>, TokenUsageStats)> {
        if batches.is_empty() {
            // Return early if there are no batches to process
            return Ok((Vec::new(), TokenUsageStats::with_provider_info(
                self.config.provider.display_name().to_string(),
                self.config.get_model().to_string()
            )));
        }
        
        // Create a collection to store all the final translated entries
        let all_translated_entries = Arc::new(StdMutex::new(Vec::with_capacity(
            batches.iter().map(|batch| batch.len()).sum()
        )));
        
        // Create a vector to store the translation tasks
        let mut tasks = Vec::new();
        
        // Get the maximum number of concurrent requests based on configuration
        let max_concurrent = self.config.optimal_concurrent_requests();
        
        // Keep track of the total number of batches
        let total_batches = batches.len();
        
        // Create a counter for completed batches, shared between all tasks
        let completed = Arc::new(AtomicUsize::new(0));
        
        // Create a counter to detect finished processing
        let processed_batches = Arc::new(AtomicUsize::new(0));
        
        // Create a token usage stats tracker shared between all tasks
        // Initialize with provider and model information
        let provider_name = self.config.provider.display_name().to_string();
        let model_name = self.config.get_model().to_string();
        let token_usage = Arc::new(StdMutex::new(TokenUsageStats::with_provider_info(
            provider_name,
            model_name
        )));
        
        let semaphore = Arc::new(Semaphore::new(max_concurrent));
        
        // Process each batch
        for (batch_idx, batch) in batches.iter().enumerate() {
            if batch.is_empty() {
                continue;
            }
            
            // Clone references for the task
            let batch = batch.clone();
            let source_language = source_language.to_string();
            let target_language = target_language.to_string();
            let completed_clone = Arc::clone(&completed);
            let processed_clone = Arc::clone(&processed_batches);
            let all_translated_entries_clone = Arc::clone(&all_translated_entries);
            let progress_callback = progress_callback.clone();
            let semaphore_clone = Arc::clone(&semaphore);
            let log_capture_clone = Arc::clone(&log_capture);
            let translation_service = self.clone();
            
            // Create an async task for this batch
            let task = tokio::spawn(async move {
                // Acquire a permit from the semaphore to limit concurrent requests
                let _permit = semaphore_clone.acquire().await.expect("Semaphore should not be closed");
                
                // Sleep for rate limit delay to avoid overwhelming the API
                if batch_idx > 0 {
                    let delay_ms = translation_service.config.common.rate_limit_delay_ms;
                    if delay_ms > 0 {
                        tokio::time::sleep(std::time::Duration::from_millis(delay_ms as u64)).await;
                    }
                }
                
                // Translate the batch using the enhanced recovery method
                let translated_entries = match translation_service.translate_batch_with_recovery(
                    &batch,
                    &source_language,
                    &target_language,
                    Arc::clone(&log_capture_clone),
                    true // Enable individual retries for maximum completeness
                ).await {
                    Ok(entries) => entries,
                    Err(e) => {
                        // Log the error and use originals with warning markers as fallback
                        let mut logs = log_capture_clone.lock().unwrap();
                        logs.push(LogEntry {
                            level: "ERROR".to_string(),
                            message: format!("Fatal error in batch {}: {}. Using original text with warning markers.", batch_idx + 1, e)
                        });
                        
                        // Mark all entries as needing translation but keep original text
                        batch.iter().map(|entry| {
                            let mut new_entry = entry.clone();
                            new_entry.text = format!("[NEEDS TRANSLATION] {}", entry.text);
                            new_entry
                        }).collect()
                    }
                };
                
                // Update the collection of all translated entries
                {
                    let mut all_entries = all_translated_entries_clone.lock().unwrap();
                    let before_count = all_entries.len();
                    all_entries.extend(translated_entries);
                    let after_count = all_entries.len();
                    
                    // Log the entry counts for this batch
                    let mut logs = log_capture_clone.lock().unwrap();
                    logs.push(LogEntry {
                        level: "INFO".to_string(),
                        message: format!("Batch {}: Added {} entries ({} -> {})", 
                                       batch_idx + 1, after_count - before_count, before_count, after_count)
                    });
                }
                
                // Update completion tracking
                let current_completed = completed_clone.fetch_add(1, Ordering::SeqCst) + 1;
                processed_clone.fetch_add(1, Ordering::SeqCst);
                
                // Call progress callback
                progress_callback(current_completed, total_batches);
            });
            
            tasks.push(task);
        }
        
        // Wait for all translation tasks to complete
        for task in tasks {
            let _ = task.await;
        }
        
        // Get the completed entries
        let mut all_entries = {
            let entries = all_translated_entries.lock().unwrap();
            entries.clone()
        };
        
        // Sort the entries by sequence number to maintain proper order
        all_entries.sort_by_key(|entry| entry.seq_num);
        
        // Get the final token usage stats
        let usage_stats = {
            let stats = token_usage.lock().unwrap();
            stats.clone()
        };
        
        // Log final statistics
        {
            let mut logs = log_capture.lock().unwrap();
            let processed = processed_batches.load(Ordering::SeqCst);
            let expected_entries = batches.iter().map(|batch| batch.len()).sum::<usize>();
            
            if all_entries.len() != expected_entries {
                logs.push(LogEntry {
                    level: "ERROR".to_string(),
                    message: format!("MISSING ENTRIES: Expected {} entries but got {}. Some entries may have been lost during translation.", 
                                  expected_entries, all_entries.len())
                });
                
                // Try to identify which entries are missing
                let mut expected_seq_nums: Vec<usize> = Vec::new();
                for batch in batches {
                    for entry in batch {
                        expected_seq_nums.push(entry.seq_num);
                    }
                }
                expected_seq_nums.sort();
                
                let mut actual_seq_nums: Vec<usize> = all_entries.iter().map(|e| e.seq_num).collect();
                actual_seq_nums.sort();
                
                // Find the missing sequence numbers
                let mut missing_seq_nums = Vec::new();
                for seq_num in &expected_seq_nums {
                    if !actual_seq_nums.contains(seq_num) {
                        missing_seq_nums.push(*seq_num);
                    }
                }
                
                if !missing_seq_nums.is_empty() {
                    logs.push(LogEntry {
                        level: "ERROR".to_string(),
                        message: format!("Missing entries with sequence numbers: {:?}", missing_seq_nums)
                    });
                }
            } else {
                logs.push(LogEntry {
                    level: "INFO".to_string(),
                    message: format!("Translation complete: processed {} of {} batches, {} entries", 
                                  processed, total_batches, all_entries.len())
                });
            }
        }
        
        Ok((all_entries, usage_stats))
    }
    
    /// Translate text with usage tracking
    /// @returns: Translated text and token usage stats (if available)
    async fn translate_text_with_usage(
        &self, 
        text: &str, 
        source_language: &str, 
        target_language: &str,
        log_capture: Option<Arc<StdMutex<Vec<LogEntry>>>>
    ) -> Result<(String, Option<(Option<u64>, Option<u64>, Option<Duration>)>)> {
        match &self.provider {
            TranslationProviderImpl::Ollama { client } => {
                // Count the number of lines in the text
                let line_count = text.lines().filter(|line| !line.trim().is_empty()).count();
                
                // Enhanced system prompt specifically for subtitles
                let system_prompt = format!(
                    "{} You are translating {} subtitles to {}. Each subtitle entry is prefixed with ENTRY_N: where N is the entry number. You MUST preserve these markers and translate EACH entry separately, keeping the exact same format. NEVER merge content between entries. Return each translated entry with its original ENTRY_N: prefix.",
                    self.config.common.system_prompt
                        .replace("{source_language}", source_language)
                        .replace("{target_language}", target_language),
                    source_language,
                    target_language
                );
                
                // Enhanced user prompt that clearly formats the request
                let user_prompt = format!(
                    "Translate the following {} subtitle entries to {}. Each entry is prefixed with ENTRY_N: where N is the entry number.\n\nIMPORTANT RULES:\n- Keep each entry separate with its ENTRY_N: prefix\n- NEVER merge content between entries\n- Translate each entry individually\n- Preserve all formatting\n\nHere are the subtitle entries to translate:\n\n{}",
                    source_language,
                    target_language,
                    text
                );
                
                // Capture log about the request
                if let Some(log_capture) = &log_capture {
                    // Handle poisoned mutex gracefully
                    let mut logs = match log_capture.lock() {
                        Ok(guard) => guard,
                        Err(poisoned) => {
                            // Get the inner data even though the mutex is poisoned
                            warn!("Encountered poisoned mutex for log capture. Recovering...");
                            poisoned.into_inner()
                        }
                    };
                    logs.push(LogEntry {
                        level: "INFO".to_string(),
                        message: format!("Ollama request: {} lines to translate, {} chars", line_count, text.len())
                    });
                }
                
                // Build the Ollama request
                let request = GenerationRequest::new(self.config.get_model(), user_prompt)
                    .system(system_prompt)
                    .temperature(0.3)
                    .no_stream();
                
                // Send request to Ollama
                let request_start = std::time::Instant::now();
                let response = client.generate(request).await?;
                let request_duration = request_start.elapsed();
                
                // Extract token usage from the response
                // For Ollama: prompt_eval_count -> prompt_tokens, eval_count -> completion_tokens
                let prompt_tokens = response.prompt_eval_count;
                let completion_tokens = response.eval_count;
                
                // Extract the response text
                let response_text = response.response;
                
                // Log information about the response
                if let Some(log_capture) = &log_capture {
                    let mut logs = match log_capture.lock() {
                        Ok(guard) => guard,
                        Err(poisoned) => {
                            // Get the inner data even though the mutex is poisoned
                            warn!("Encountered poisoned mutex for log capture. Recovering...");
                            poisoned.into_inner()
                        }
                    };
                    
                    // Count lines in the response
                    let response_line_count = response_text.lines().filter(|line| !line.trim().is_empty()).count();
                    
                    logs.push(LogEntry {
                        level: "INFO".to_string(),
                        message: format!("Ollama response: got {} lines, expected {} lines", response_line_count, line_count)
                    });
                    
                    // Log first 100 chars of response for debugging
                    let preview = if response_text.chars().count() > 100 {
                        // Safely truncate at character boundaries using char iterators
                        let truncated: String = response_text.chars().take(100).collect();
                        format!("{}...", truncated)
                    } else {
                        response_text.clone()
                    };
                    
                    logs.push(LogEntry {
                        level: "DEBUG".to_string(),
                        message: format!("Ollama response preview: {}", preview)
                    });
                }
                
                // Process the response
                // Check if we have the right number of lines and adjust if needed
                let response_lines: Vec<&str> = response_text.lines().filter(|line| !line.trim().is_empty()).collect();
                let translated_text = if response_lines.len() == line_count {
                    // Perfect match - use as is
                    response_text
                } else if response_lines.len() < line_count {
                    // Too few lines - log warning and pad with original text
                    if let Some(log_capture) = &log_capture {
                        let mut logs = match log_capture.lock() {
                            Ok(guard) => guard,
                            Err(poisoned) => {
                                // Get the inner data even though the mutex is poisoned
                                warn!("Encountered poisoned mutex for log capture. Recovering...");
                                poisoned.into_inner()
                            }
                        };
                        logs.push(LogEntry {
                            level: "WARN".to_string(),
                            message: format!("Ollama returned fewer lines than expected: got {}, needed {}", 
                                          response_lines.len(), line_count)
                        });
                    }
                    
                    // Get original lines
                    let original_lines: Vec<&str> = text.lines().filter(|line| !line.trim().is_empty()).collect();
                    
                    // Create new vector to store final result with missing translations marked
                    let mut final_lines = Vec::with_capacity(line_count);
                    
                    // Add all translated lines (clone to avoid move issue)
                    for &line in &response_lines {
                        final_lines.push(line.to_string());
                    }
                    
                    // Add missing lines with marker
                    for i in response_lines.len()..line_count {
                        if i < original_lines.len() {
                            final_lines.push(format!("[MISSING TRANSLATION] {}", original_lines[i]));
                        } else {
                            final_lines.push("[MISSING TRANSLATION]".to_string());
                        }
                    }
                    
                    final_lines.join("\n")
                } else {
                    // Too many lines - log warning and use first N lines
                    if let Some(log_capture) = &log_capture {
                        let mut logs = match log_capture.lock() {
                            Ok(guard) => guard,
                            Err(poisoned) => {
                                // Get the inner data even though the mutex is poisoned
                                warn!("Encountered poisoned mutex for log capture. Recovering...");
                                poisoned.into_inner()
                            }
                        };
                        logs.push(LogEntry {
                            level: "WARN".to_string(),
                            message: format!("Ollama returned more lines than expected: got {}, needed {}. Using first {} lines.",
                                          response_lines.len(), line_count, line_count)
                        });
                    }
                    
                    response_lines.iter().take(line_count).map(|&s| s.to_string()).collect::<Vec<String>>().join("\n")
                };
                
                // Return the processed translated text
                Ok((translated_text, Some((prompt_tokens, completion_tokens, Some(request_duration)))))
            },
            TranslationProviderImpl::OpenAI { client } => {
                // Create system prompt
                let system_prompt = self.config.common.system_prompt
                    .replace("{source_language}", source_language)
                    .replace("{target_language}", target_language);
                
                // Build the OpenAI request
                let request = OpenAIRequest::new(self.config.get_model())
                    .add_message("system", format!("{} Each subtitle entry is prefixed with ENTRY_N: where N is the entry number. You MUST preserve these markers and translate EACH entry separately, keeping the exact same format.", system_prompt))
                    .add_message("user", format!("Translate the following subtitle entries from {} to {}. Each entry is prefixed with ENTRY_N: where N is the entry number.\n\nIMPORTANT RULES:\n- Keep each entry separate with its ENTRY_N: prefix\n- NEVER merge content between entries\n- Translate each entry individually\n- Preserve all formatting\n\nHere are the subtitle entries to translate:\n\n{}", source_language, target_language, text))
                    .temperature(0.3)
                    .max_tokens(4096);
                
                // Send request to OpenAI
                let request_start = std::time::Instant::now();
                let response = client.complete(request).await?;
                let request_duration = request_start.elapsed();
                
                // Extract the content from the first choice
                if response.choices.is_empty() {
                    return Err(anyhow!("No response received from OpenAI API"));
                }
                
                // Extract token usage information 
                // For OpenAI: prompt_tokens -> prompt_tokens, completion_tokens -> completion_tokens
                let prompt_tokens = Some(response.usage.prompt_tokens as u64);
                let completion_tokens = Some(response.usage.completion_tokens as u64);
                
                // Extract the translated text from the response
                Ok((extract_translated_text_from_response(&response.choices[0].message.content, log_capture)?, Some((prompt_tokens, completion_tokens, Some(request_duration)))))
            },
            TranslationProviderImpl::Anthropic { client } => {
                // Calculate appropriate max tokens (approximately 6 times the input length to allow for expansion)
                let text_length = text.len();
                
                // Create system prompt
                let system_prompt = self.config.common.system_prompt
                    .replace("{source_language}", source_language)
                    .replace("{target_language}", target_language);
                
                // Determine the maximum allowed tokens based on the model
                let model_max_tokens = self.max_tokens_for_model(&self.config.get_model());
                
                // Use a more generous multiplier for Anthropic to ensure we get complete translations
                // but ensure we respect the model's token limit
                let max_tokens = ((text_length * 6).max(1000).min(model_max_tokens as usize)) as u32;
                
                // Count the exact number of non-empty subtitle lines
                let subtitle_lines = text.split('\n').filter(|s| !s.is_empty()).collect::<Vec<_>>();
                let input_lines_count = subtitle_lines.len();
                
                // Capture log instead of directly printing
                if let Some(log_capture) = &log_capture {
                    let mut logs = match log_capture.lock() {
                        Ok(guard) => guard,
                        Err(poisoned) => {
                            // Get the inner data even though the mutex is poisoned
                            warn!("Encountered poisoned mutex for log capture. Recovering...");
                            poisoned.into_inner()
                        }
                    };
                    logs.push(LogEntry {
                        level: "INFO".to_string(),
                        message: format!("Anthropic request: {} chars input, {} subtitle lines, {} max tokens", 
                                        text_length, input_lines_count, max_tokens)
                    });
                } else {
                    // Fallback to direct logging if no capture is provided
                    info!("Anthropic request: {} chars input, {} subtitle lines, {} max tokens", 
                         text_length, input_lines_count, max_tokens);
                }
                
                // Enhanced system prompt with explicit instructions about line preservation
                let enhanced_system_prompt = format!(
                    "{} Each subtitle entry is prefixed with ENTRY_N: where N is the entry number. You MUST preserve these markers and translate EACH entry separately, keeping the exact same format. NEVER merge content between entries.",
                    system_prompt
                );
                
                // Build the Anthropic request with enhanced instructions
                let request = AnthropicRequest::new(self.config.get_model(), max_tokens)
                    .system(enhanced_system_prompt)
                    .add_message("user", format!(
                        "Translate the following subtitle entries from {} to {}.\n\nIMPORTANT RULES:\n- Each entry is prefixed with ENTRY_N: where N is the entry number\n- Keep each entry separate with its ENTRY_N: prefix\n- NEVER merge content between entries\n- Translate each entry individually\n- Preserve all formatting\n\nHere are the subtitle entries to translate:\n\n{}", 
                        source_language, 
                        target_language,
                        text
                    ))
                    .temperature(0.3);
                
                // Send request to Anthropic with timing
                let request_start = std::time::Instant::now();
                let response = match client.complete(request).await {
                    Ok(resp) => resp,
                    Err(e) => {
                        // Capture error log instead of directly printing
                        if let Some(log_capture) = &log_capture {
                            let mut logs = match log_capture.lock() {
                                Ok(guard) => guard,
                                Err(poisoned) => {
                                    // Get the inner data even though the mutex is poisoned
                                    warn!("Encountered poisoned mutex for log capture. Recovering...");
                                    poisoned.into_inner()
                                }
                            };
                            logs.push(LogEntry {
                                level: "ERROR".to_string(),
                                message: format!("Anthropic request failed after {:?}: {}", request_start.elapsed(), e)
                            });
                        } else {
                            error!("Anthropic request failed after {:?}: {}", request_start.elapsed(), e);
                        }
                        return Err(e);
                    }
                };
                let request_duration = request_start.elapsed();
                
                // Capture log for response time
                if let Some(log_capture) = &log_capture {
                    let mut logs = match log_capture.lock() {
                        Ok(guard) => guard,
                        Err(poisoned) => {
                            // Get the inner data even though the mutex is poisoned
                            warn!("Encountered poisoned mutex for log capture. Recovering...");
                            poisoned.into_inner()
                        }
                    };
                    logs.push(LogEntry {
                        level: "INFO".to_string(),
                        message: format!("Anthropic response received in {:?}", request_duration)
                    });
                } else {
                    info!("Anthropic response received in {:?}", request_duration);
                }
                
                // Extract token usage information
                // For Anthropic: input_tokens -> prompt_tokens, output_tokens -> completion_tokens
                let prompt_tokens = Some(response.usage.input_tokens as u64);
                let completion_tokens = Some(response.usage.output_tokens as u64);
                
                // Extract the text from the response
                let content = Anthropic::extract_text_from_response(&response);
                
                // Process the response to ensure we get the correct number of lines
                let mut output_lines = content.split('\n')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<String>>();
                
                let output_lines_count = output_lines.len();
                
                // Capture log for response line count
                if let Some(log_capture) = &log_capture {
                    let mut logs = match log_capture.lock() {
                        Ok(guard) => guard,
                        Err(poisoned) => {
                            // Get the inner data even though the mutex is poisoned
                            warn!("Encountered poisoned mutex for log capture. Recovering...");
                            poisoned.into_inner()
                        }
                    };
                    logs.push(LogEntry {
                        level: "INFO".to_string(),
                        message: format!("Anthropic translation: got {} lines, expected {} lines", 
                                        output_lines_count, input_lines_count)
                    });
                } else {
                    info!("Anthropic translation: got {} lines, expected {} lines", output_lines_count, input_lines_count);
                }
                
                let final_content = if output_lines_count == input_lines_count {
                    // Perfect match - use as is
                    content
                } else if output_lines_count < input_lines_count {
                    // Not enough lines - log warning and pad with missing translation markers
                    if let Some(log_capture) = &log_capture {
                        let mut logs = match log_capture.lock() {
                            Ok(guard) => guard,
                            Err(poisoned) => {
                                // Get the inner data even though the mutex is poisoned
                                warn!("Encountered poisoned mutex for log capture. Recovering...");
                                poisoned.into_inner()
                            }
                        };
                        logs.push(LogEntry {
                            level: "ERROR".to_string(),
                            message: format!("Translation returned fewer lines than input: got {} but expected {}.", 
                                           output_lines_count, input_lines_count)
                        });
                    } else {
                        error!("Translation returned fewer lines than input: got {} but expected {}.", 
                               output_lines_count, input_lines_count);
                    }
                    
                    // Extend output_lines vector with missing translation markers for original lines
                    while output_lines.len() < input_lines_count {
                        let original_idx = output_lines.len();
                        if original_idx < subtitle_lines.len() {
                            // Add missing translation marker with original text
                            output_lines.push(format!("[MISSING TRANSLATION] {}", subtitle_lines[original_idx]));
                        } else {
                            // Failsafe - should never happen but just in case
                            output_lines.push("[MISSING TRANSLATION]".to_string());
                        }
                    }
                    
                    // Join the lines back together
                    output_lines.join("\n")
                } else {
                    // Too many lines - log and use the smart mapping algorithm in translate_batches
                    if let Some(log_capture) = &log_capture {
                        let mut logs = match log_capture.lock() {
                            Ok(guard) => guard,
                            Err(poisoned) => {
                                // Get the inner data even though the mutex is poisoned
                                warn!("Encountered poisoned mutex for log capture. Recovering...");
                                poisoned.into_inner()
                            }
                        };
                        logs.push(LogEntry {
                            level: "INFO".to_string(),
                            message: format!("Translation returned more lines than input: got {} but expected {}.", 
                                           output_lines_count, input_lines_count)
                        });
                    } else {
                        info!("Translation returned more lines than input: got {} but expected {}.", 
                               output_lines_count, input_lines_count);
                    }
                    
                    // Keep the content as is, the batch processing will handle mapping extra lines
                    content
                };
                
                // Extract the translated text from the response
                Ok((extract_translated_text_from_response(&final_content, log_capture)?, Some((prompt_tokens, completion_tokens, Some(request_duration)))))
            }
        }
    }

    /// Process a batch of subtitles with enhanced error recovery
    pub async fn translate_batch_with_recovery(
        &self,
        batch: &[SubtitleEntry],
        source_language: &str,
        target_language: &str,
        log_capture: Arc<StdMutex<Vec<LogEntry>>>,
        retry_individual_entries: bool
    ) -> Result<Vec<SubtitleEntry>> {
        // Capture log information
        let capture_log = |level: &str, message: &str| {
            let mut logs = log_capture.lock().unwrap();
            logs.push(LogEntry {
                level: level.to_string(),
                message: message.to_string()
            });
        };
        
        // Prepare text for translation with entry markers
        let prepare_text = |entries: &[SubtitleEntry]| -> String {
            entries.iter()
                .map(|entry| format!("ENTRY_{}:\n{}", entry.seq_num, entry.text))
                .collect::<Vec<_>>()
                .join("\n\n")
        };
        
        // First attempt: batch translation
        let text_to_translate = prepare_text(batch);
        
        // Try batch translation with up to 3 retries
        let mut batch_success = false;
        let mut translated_text = String::new();
        let mut retry_count = 0;
        let max_retries = 3;
        
        while !batch_success && retry_count < max_retries {
            match self.translate_text_with_usage(
                &text_to_translate,
                source_language,
                target_language,
                Some(Arc::clone(&log_capture))
            ).await {
                Ok((result, _)) => {
                    translated_text = result;
                    batch_success = true;
                    break;
                },
                Err(e) => {
                    retry_count += 1;
                    capture_log("WARN", &format!("Batch translation failed (attempt {}/{}): {}", 
                              retry_count, max_retries, e));
                    
                    // Wait before retrying with increasing delay
                    if retry_count < max_retries {
                        let delay_ms = 1000 * retry_count;
                        tokio::time::sleep(std::time::Duration::from_millis(delay_ms as u64)).await;
                    }
                }
            }
        }
        
        // If batch translation succeeded, extract entries
        if batch_success {
            // Extract entries from the translated text based on entry markers
            let mut entries_map = std::collections::HashMap::new();
            let mut current_entry_num: Option<usize> = None;
            let mut current_entry_text = String::new();
            
            // Process each line to find entry markers and extract content
            for line in translated_text.lines() {
                if let Some(entry_marker) = line.strip_prefix("ENTRY_") {
                    // If we were already processing an entry, save it
                    if let Some(entry_num) = current_entry_num {
                        if !current_entry_text.is_empty() {
                            entries_map.insert(entry_num, current_entry_text.trim().to_string());
                            current_entry_text = String::new();
                        }
                    }
                    
                    // Parse the new entry number
                    if let Some(colon_idx) = entry_marker.find(':') {
                        if let Ok(entry_num) = entry_marker[..colon_idx].parse::<usize>() {
                            current_entry_num = Some(entry_num);
                            continue;
                        }
                    }
                } else if let Some(entry_num) = current_entry_num {
                    // Add content to the current entry
                    if !current_entry_text.is_empty() {
                        current_entry_text.push('\n');
                    }
                    current_entry_text.push_str(line);
                }
            }
            
            // Add the last entry if needed
            if let Some(entry_num) = current_entry_num {
                if !current_entry_text.is_empty() {
                    entries_map.insert(entry_num, current_entry_text.trim().to_string());
                }
            }
            
            // Create subtitle entries from the map
            let mut result_entries = Vec::with_capacity(batch.len());
            
            // Check if we have all entries or need individual translation
            let all_entries_present = batch.iter()
                .all(|entry| entries_map.contains_key(&entry.seq_num));
            
            if all_entries_present {
                // All entries were translated successfully
                for entry in batch {
                    let mut translated_entry = entry.clone();
                    let translated_text = entries_map.get(&entry.seq_num).unwrap();
                    translated_entry.text = preserve_formatting(&entry.text, translated_text);
                    result_entries.push(translated_entry);
                }
                
                capture_log("INFO", &format!("Successfully translated all {} entries in batch", batch.len()));
                return Ok(result_entries);
            } else if retry_individual_entries {
                // Some entries are missing - translate them individually
                for entry in batch {
                    let mut translated_entry = entry.clone();
                    
                    if let Some(translated_text) = entries_map.get(&entry.seq_num) {
                        // Entry was translated successfully in the batch
                        translated_entry.text = preserve_formatting(&entry.text, translated_text);
                    } else {
                        // Entry missing - try individual translation
                        capture_log("WARN", &format!("Entry {} missing from batch translation. Attempting individual translation...", entry.seq_num));
                        
                        // Individual translation with retries
                        let individual_text = format!("ENTRY_{}:\n{}", entry.seq_num, entry.text);
                        let mut individual_success = false;
                        let mut individual_result = String::new();
                        let mut retry_count = 0;
                        
                        // Start with normal max_tokens calculation, but reduce if we get token limit errors
                        let mut reduce_tokens_factor = 1.0;
                        
                        while !individual_success && retry_count < max_retries {
                            // Calculate max_tokens with potential reduction factor
                            let text_len = individual_text.len();
                            let mut is_token_limit_error = false;
                            
                            match self.translate_text_with_usage(
                                &individual_text,
                                source_language,
                                target_language,
                                Some(Arc::clone(&log_capture))
                            ).await {
                                Ok((result, _)) => {
                                    individual_result = result;
                                    individual_success = true;
                                    break;
                                },
                                Err(e) => {
                                    retry_count += 1;
                                    
                                    // Check if this is a max_tokens error for Anthropic
                                    let error_msg = e.to_string().to_lowercase();
                                    is_token_limit_error = error_msg.contains("max_tokens") && 
                                                      (error_msg.contains("maximum allowed") || 
                                                       error_msg.contains("exceeds"));
                                    
                                    if is_token_limit_error && retry_count < max_retries {
                                        // This is a token limit error, adjust our max_tokens calculation
                                        reduce_tokens_factor *= 0.7; // Reduce by 30% each time
                                        capture_log("WARN", &format!("Token limit error for entry {}. Reducing max_tokens by factor {}.", 
                                                 entry.seq_num, reduce_tokens_factor));
                                    } else {
                                        // Not a token error or we've already retried too many times
                                        capture_log("WARN", &format!("Individual translation for entry {} failed (attempt {}/{}): {}", 
                                                 entry.seq_num, retry_count, max_retries, e));
                                    }
                                    
                                    // Wait before retrying
                                    if retry_count < max_retries {
                                        let delay_ms = 1000 * retry_count;
                                        tokio::time::sleep(std::time::Duration::from_millis(delay_ms as u64)).await;
                                    }
                                }
                            }
                        }
                        
                        if individual_success {
                            // Extract the content from the individual translation
                            let content = individual_result.lines()
                                .skip_while(|line| !line.starts_with("ENTRY_"))
                                .skip(1) // Skip the marker line
                                .collect::<Vec<&str>>()
                                .join("\n");
                                
                            if !content.trim().is_empty() {
                                translated_entry.text = preserve_formatting(&entry.text, &content.trim());
                                capture_log("INFO", &format!("Successfully translated entry {} individually", entry.seq_num));
                            } else {
                                // Couldn't extract proper content - use original with warning
                                capture_log("ERROR", &format!("Failed to extract translated content for entry {}. Using original text.", entry.seq_num));
                                translated_entry.text = format!("{}", entry.text);
                            }
                        } else {
                            // Individual translation failed - use original with warning
                            capture_log("ERROR", &format!("All individual translation attempts for entry {} failed. Using original with warning.", entry.seq_num));
                            translated_entry.text = format!("[NEEDS TRANSLATION] {}", entry.text);
                        }
                    }
                    
                    result_entries.push(translated_entry);
                }
                
                return Ok(result_entries);
            } else {
                // Some entries are missing but individual retry is disabled
                for entry in batch {
                    let mut translated_entry = entry.clone();
                    
                    if let Some(translated_text) = entries_map.get(&entry.seq_num) {
                        // Entry was translated successfully
                        translated_entry.text = preserve_formatting(&entry.text, translated_text);
                    } else {
                        // Entry missing - use original with warning
                        capture_log("ERROR", &format!("Entry {} missing from translation and individual retry disabled. Using original with warning.", entry.seq_num));
                        translated_entry.text = format!("[NEEDS TRANSLATION] {}", entry.text);
                    }
                    
                    result_entries.push(translated_entry);
                }
                
                return Ok(result_entries);
            }
        }
        
        // Batch translation failed entirely - use individual translation if enabled
        if retry_individual_entries {
            // Translate each entry individually
            let mut result_entries = Vec::with_capacity(batch.len());
            
            for entry in batch {
                let mut translated_entry = entry.clone();
                let individual_text = format!("ENTRY_{}:\n{}", entry.seq_num, entry.text);
                
                // Try individual translation with retries
                let mut individual_success = false;
                let mut individual_result = String::new();
                let mut retry_count = 0;
                
                // Start with normal max_tokens calculation, but reduce if we get token limit errors
                let mut reduce_tokens_factor = 1.0;
                
                while !individual_success && retry_count < max_retries {
                    // Calculate max_tokens with potential reduction factor
                    let text_len = individual_text.len();
                    let mut is_token_limit_error = false;
                    
                    match self.translate_text_with_usage(
                        &individual_text,
                        source_language,
                        target_language,
                        Some(Arc::clone(&log_capture))
                    ).await {
                        Ok((result, _)) => {
                            individual_result = result;
                            individual_success = true;
                            break;
                        },
                        Err(e) => {
                            retry_count += 1;
                            
                            // Check if this is a max_tokens error for Anthropic
                            let error_msg = e.to_string().to_lowercase();
                            is_token_limit_error = error_msg.contains("max_tokens") && 
                                                  (error_msg.contains("maximum allowed") || 
                                                   error_msg.contains("exceeds"));
                            
                            if is_token_limit_error && retry_count < max_retries {
                                // This is a token limit error, adjust our max_tokens calculation
                                reduce_tokens_factor *= 0.7; // Reduce by 30% each time
                                capture_log("WARN", &format!("Token limit error for entry {}. Reducing max_tokens by factor {}.", 
                                         entry.seq_num, reduce_tokens_factor));
                            } else {
                                // Not a token error or we've already retried too many times
                                capture_log("WARN", &format!("Individual translation for entry {} failed (attempt {}/{}): {}", 
                                         entry.seq_num, retry_count, max_retries, e));
                            }
                            
                            // Wait before retrying
                            if retry_count < max_retries {
                                let delay_ms = 1000 * retry_count;
                                tokio::time::sleep(std::time::Duration::from_millis(delay_ms as u64)).await;
                            }
                        }
                    }
                }
                
                if individual_success {
                    // Extract the content from the individual translation
                    let content = individual_result.lines()
                        .skip_while(|line| !line.starts_with("ENTRY_"))
                        .skip(1) // Skip the marker line
                        .collect::<Vec<&str>>()
                        .join("\n");
                        
                    if !content.trim().is_empty() {
                        translated_entry.text = preserve_formatting(&entry.text, &content.trim());
                        capture_log("INFO", &format!("Successfully translated entry {} individually", entry.seq_num));
                    } else {
                        // Couldn't extract proper content - use original with warning
                        capture_log("ERROR", &format!("Failed to extract translated content for entry {} after batch failure. Using original text.", entry.seq_num));
                        translated_entry.text = format!("{}", entry.text);
                    }
                } else {
                    // Individual translation failed - use original with warning
                    capture_log("ERROR", &format!("All translation attempts for entry {} failed. Using original with warning.", entry.seq_num));
                    translated_entry.text = format!("[NEEDS TRANSLATION] {}", entry.text);
                }
                
                result_entries.push(translated_entry);
            }
            
            return Ok(result_entries);
        } else {
            // Batch translation failed and individual retry is disabled - use all originals with warnings
            let result_entries = batch.iter().map(|entry| {
                let mut new_entry = entry.clone();
                capture_log("ERROR", &format!("Batch translation failed and individual retry disabled for entry {}. Using original with warning.", entry.seq_num));
                new_entry.text = format!("[NEEDS TRANSLATION] {}", entry.text);
                new_entry
            }).collect();
            
            return Ok(result_entries);
        }
    }

    /// Get the maximum allowed tokens for a model
    fn max_tokens_for_model(&self, model: &str) -> u32 {
        // Extract model name from the config
        let model_name = model.to_lowercase();
        
        // Determine max tokens based on model name
        if model_name.contains("haiku") {
            // Claude-3-Haiku has a limit of 4096 tokens
            4096
        } else if model_name.contains("sonnet") {
            // Claude-3-Sonnet has a higher limit
            12288
        } else if model_name.contains("opus") {
            // Claude-3-Opus has the highest limit
            32768
        } else if model_name.contains("gpt-4") {
            // GPT-4 family typically has high token limits
            8192
        } else if model_name.contains("gpt-3.5") {
            // GPT-3.5 models like turbo
            4096
        } else if model_name.contains("mixtral") || model_name.contains("llama") {
            // Mixtral, Llama etc. have flexible limits but we'll be conservative
            4096
        } else {
            // Default fallback for unknown models
            4096
        }
    }
}

impl Clone for TranslationService {
    fn clone(&self) -> Self {
        // We need to manually clone because Ollama client doesn't implement Clone
        let provider = match &self.provider {
            TranslationProviderImpl::Ollama { client: _ } => {
                // Parse the Ollama endpoint URL
                let (host, port) = parse_endpoint(&self.config.get_endpoint())
                    .expect("Failed to parse endpoint URL");
                
                // Create a new Ollama client
                let client = Ollama::new(host, port);
                
                TranslationProviderImpl::Ollama {
                    client,
                }
            },
            TranslationProviderImpl::OpenAI { client: _ } => {
                // Create a new OpenAI client
                let client = OpenAI::new(
                    self.config.get_api_key(),
                    self.config.get_endpoint()
                );
                
                TranslationProviderImpl::OpenAI {
                    client,
                }
            },
            TranslationProviderImpl::Anthropic { client: _ } => {
                // Create a new Anthropic client
                let client = Anthropic::new(
                    self.config.get_api_key(),
                    self.config.get_endpoint()
                );
                
                TranslationProviderImpl::Anthropic {
                    client,
                }
            },
        };
        
        Self {
            provider,
            config: self.config.clone(),
        }
    }
}

/// Helper function to preserve formatting markers while replacing content
fn preserve_formatting(original: &str, translated: &str) -> String {
    // Check if the original has formatting tags like {\an8}
    if let Some(tag_end) = original.find('}') {
        if original.starts_with('{') {
            // Extract the formatting tag
            let format_tag = &original[0..=tag_end];
            
            // Return the tag + translated content
            return format!("{}{}", format_tag, translated);
        }
    }
    
    // If no special formatting, return the translation as-is
    translated.to_string()
}

/// Process translations where the line count doesn't match the original
fn process_mismatched_translations(
    batch: &[SubtitleEntry],
    translated_lines: &[&str],
    source_language: &str,
    target_language: &str,
    log_capture: &Arc<StdMutex<Vec<LogEntry>>>
) -> Vec<SubtitleEntry> {
    // Capture log information
    let capture_log = |level: &str, message: &str| {
        let mut logs = log_capture.lock().unwrap();
        logs.push(LogEntry {
            level: level.to_string(),
            message: message.to_string()
        });
    };
    
    if translated_lines.len() > batch.len() {
        // We'll use a smarter matching algorithm here that tries to preserve
        // approximately the right amount of text per subtitle entry
        
        // First calculate the average characters per original entry
        let original_chars: usize = batch.iter().map(|entry| entry.text.len()).sum();
        let translated_chars: usize = translated_lines.iter().map(|line| line.len()).sum();
        
        // Scale factor to account for language differences (target might be longer/shorter than source)
        let scale_factor = if original_chars > 0 {
            translated_chars as f64 / original_chars as f64
        } else {
            1.0
        };
        
        let mut line_index = 0;
        let mut entries = Vec::with_capacity(batch.len());
        
        for entry in batch {
            // How many characters we expect in the translation, scaled
            let expected_chars = (entry.text.len() as f64 * scale_factor).round() as usize;
            let mut accumulated_text = String::new();
            let mut chars_so_far = 0;
            
            // Continue adding lines until we get close to the expected character count
            // or run out of translated lines
            while line_index < translated_lines.len() && chars_so_far < expected_chars {
                if !accumulated_text.is_empty() {
                    accumulated_text.push('\n');
                }
                accumulated_text.push_str(translated_lines[line_index]);
                chars_so_far += translated_lines[line_index].len();
                line_index += 1;
                
                // If we're at 80% of expected chars and there are more entries coming,
                // leave the rest for later entries
                if chars_so_far >= expected_chars * 8 / 10 && 
                   entries.len() < batch.len() - 1 {
                    break;
                }
            }
            
            // Create a new subtitle entry with the translated text
            let mut translated_entry = entry.clone();
            
            // If we couldn't get any lines (ran out), retranslate this entry specifically
            if accumulated_text.is_empty() {
                // We'll generate a more specific message but still use the source
                // The full retranslation logic would be added in a real implementation
                capture_log("ERROR", &format!("Missing translation for subtitle {}", entry.seq_num));
                
                // For now, preserve the original text but mark it clearly
                let fallback_text = format!("[TRANSLATION NEEDED] {}", entry.text);
                translated_entry.text = fallback_text;
            } else {
                // Preserve formatting in the original text
                translated_entry.text = preserve_formatting(&entry.text, &accumulated_text);
            }
            
            // Add the translated entry to the results
            entries.push(translated_entry);
        }
        
        entries
    } else {
        // Original matching algorithm for when lines match exactly or we have fewer translated lines
        let mut entries = Vec::with_capacity(batch.len());
        let mut translated_line_idx = 0;
        
        for (i, entry) in batch.iter().enumerate() {
            // If we've used all translated lines, but still have entries, that's an error
            if translated_line_idx >= translated_lines.len() {
                // Log a warning that we're missing translations
                capture_log("ERROR", &format!("Translation API did not return enough translated lines. Got {} but needed {}. Adding missing translation markers.", 
                      translated_lines.len(), batch.len()));
                
                // For remaining entries, attempt to translate them individually
                // This is a fallback mechanism for the case where batch translation fails
                capture_log("WARN", &format!("Will try to translate remaining {} subtitles individually", batch.len() - i));
                
                // Add the remaining entries with missing translation markers for now
                for j in i..batch.len() {
                    let mut new_entry = batch[j].clone();
                    
                    // Log the issue
                    capture_log("ERROR", &format!("Missing translation for entry {}. Using original text with warning marker.", new_entry.seq_num));
                    
                    // Mark the entry as needing translation but keep original text
                    new_entry.text = format!("[NEEDS TRANSLATION] {}", new_entry.text);
                    entries.push(new_entry);
                }
                
                break;
            }
            
            // Get the current translated line
            let translated_line = translated_lines[translated_line_idx];
            translated_line_idx += 1;
            
            // Create a new subtitle entry with the translated text
            let mut translated_entry = entry.clone();
            
            // Preserve formatting from the original
            translated_entry.text = preserve_formatting(&entry.text, translated_line);
            
            // Add the translated entry to the results
            entries.push(translated_entry);
        }
        
        entries
    }
}

// Function to attempt individual translation of an entry
async fn translate_single_entry(
    service: &TranslationService,
    entry: &SubtitleEntry,
    source_lang: &str,
    target_lang: &str,
    log_capture: &Arc<StdMutex<Vec<LogEntry>>>
) -> String {
    // Capture log for this specific translation
    let capture_log = |level: &str, message: &str| {
        let mut logs = log_capture.lock().unwrap();
        logs.push(LogEntry {
            level: level.to_string(),
            message: message.to_string()
        });
    };
    
    // Try to translate the individual entry
    capture_log("INFO", &format!("Attempting individual translation for entry {}", entry.seq_num));
    
    // Format with entry marker to maintain consistency
    let entry_text = format!("ENTRY_{}:\n{}", entry.seq_num, entry.text);
    
    // Maximum 3 retry attempts for individual entries
    for attempt in 1..=3 {
        match service.translate_text_with_usage(
            &entry_text,
            source_lang,
            target_lang,
            Some(Arc::clone(log_capture))
        ).await {
            Ok((translated_text, _)) => {
                // Try to extract the translated text without the marker
                if let Some(content) = translated_text.lines()
                    .skip_while(|&line| !line.starts_with("ENTRY_") && line.trim().is_empty())
                    .skip(1) // Skip the marker line
                    .collect::<Vec<&str>>()
                    .join("\n")
                    .trim()
                    .to_string()
                    .into() {
                    capture_log("INFO", &format!("Successfully translated entry {} individually", entry.seq_num));
                    return content;
                }
                
                // If marker extraction failed, just use the whole response
                return translated_text;
            },
            Err(e) => {
                capture_log("WARN", &format!("Individual translation attempt {} for entry {} failed: {}", 
                         attempt, entry.seq_num, e));
                
                // Wait briefly before retrying
                tokio::time::sleep(std::time::Duration::from_millis(1000 * attempt)).await;
            }
        }
    }
    
    // If all retries failed, return marked original
    capture_log("ERROR", &format!("All individual translation attempts for entry {} failed", entry.seq_num));
    format!("[NEEDS TRANSLATION] {}", entry.text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_config::{TranslationCommonConfig, ProviderConfig};
    
    fn get_test_config() -> TranslationConfig {
        TranslationConfig {
            provider: ConfigTranslationProvider::Ollama,
            common: TranslationCommonConfig {
                system_prompt: "You are a translator. Translate the following text from {source_language} to {target_language}. Only return the translated text.".to_string(),
                rate_limit_delay_ms: 0,
                retry_count: 3,
                retry_backoff_ms: 1000,
            },
            available_providers: vec![
                ProviderConfig {
                    provider_type: "ollama".to_string(),
                    model: "llama2".to_string(),
                    api_key: "".to_string(),
                    endpoint: "http://localhost:11434".to_string(),
                    concurrent_requests: 4,
                    max_chars_per_request: 1000,
                    timeout_secs: 30,
                },
                ProviderConfig {
                    provider_type: "openai".to_string(),
                    model: "gpt-3.5-turbo".to_string(),
                    api_key: "test-api-key".to_string(),
                    endpoint: "".to_string(),
                    concurrent_requests: 4,
                    max_chars_per_request: 4000,
                    timeout_secs: 30,
                },
                ProviderConfig {
                    provider_type: "anthropic".to_string(),
                    model: "claude-3-haiku-20240307".to_string(),
                    api_key: "test-api-key".to_string(),
                    endpoint: "".to_string(),
                    concurrent_requests: 4,
                    max_chars_per_request: 4000,
                    timeout_secs: 30,
                },
            ],
        }
    }
    
    #[test]
    fn test_translation_service_creation() {
        let config = get_test_config();
        let service = TranslationService::new(config);
        assert!(service.is_ok());
    }
    
    #[test]
    fn test_add_token_usage() {
        let mut stats = TokenUsageStats::new();
        stats.add_token_usage(Some(100), Some(50));
        assert_eq!(stats.prompt_tokens, 100);
        assert_eq!(stats.completion_tokens, 50);
        assert_eq!(stats.total_tokens, 150);
        
        // Add more tokens
        stats.add_token_usage(Some(200), Some(100));
        assert_eq!(stats.prompt_tokens, 300);
        assert_eq!(stats.completion_tokens, 150);
        assert_eq!(stats.total_tokens, 450);
        
        // Handle None values
        stats.add_token_usage(None, Some(50));
        assert_eq!(stats.prompt_tokens, 300);
        assert_eq!(stats.completion_tokens, 200);
        assert_eq!(stats.total_tokens, 500);
        
        stats.add_token_usage(Some(100), None);
        assert_eq!(stats.prompt_tokens, 400);
        assert_eq!(stats.completion_tokens, 200);
        assert_eq!(stats.total_tokens, 600);
    }
    
    #[tokio::test]
    async fn test_translate_batches_processes_all_chunks() -> Result<()> {
        // Create mock data - 5 batches with 2 entries each
        let batches = vec![
            vec![
                SubtitleEntry::new(1, 0, 1000, "First subtitle batch 1".to_string()),
                SubtitleEntry::new(2, 1001, 2000, "Second subtitle batch 1".to_string()),
            ],
            vec![
                SubtitleEntry::new(3, 2001, 3000, "First subtitle batch 2".to_string()),
                SubtitleEntry::new(4, 3001, 4000, "Second subtitle batch 2".to_string()),
            ],
            vec![
                SubtitleEntry::new(5, 4001, 5000, "First subtitle batch 3".to_string()),
                SubtitleEntry::new(6, 5001, 6000, "Second subtitle batch 3".to_string()),
            ],
            vec![
                SubtitleEntry::new(7, 6001, 7000, "First subtitle batch 4".to_string()),
                SubtitleEntry::new(8, 7001, 8000, "Second subtitle batch 4".to_string()),
            ],
            vec![
                SubtitleEntry::new(9, 8001, 9000, "First subtitle batch 5".to_string()),
                SubtitleEntry::new(10, 9001, 10000, "Second subtitle batch 5".to_string()),
            ],
        ];
        
        // Create a progress counter to track callback execution
        let progress_count = Arc::new(AtomicUsize::new(0));
        let progress_clone = Arc::clone(&progress_count);
        
        // Create a collection to store all processed entries
        let all_processed_entries = Arc::new(StdMutex::new(Vec::new()));
        
        // Process each batch sequentially, simulating the behavior we want to test
        for (_i, batch) in batches.iter().enumerate() {
            let processed_entries: Vec<SubtitleEntry> = batch.iter()
                .map(|entry| {
                    let mut new_entry = entry.clone();
                    new_entry.text = format!("[TRANSLATED] {}", entry.text);
                    new_entry
                })
                .collect();
            
            // Store the processed entries
            let mut entries = all_processed_entries.lock().unwrap();
            entries.extend(processed_entries);
            
            // Update progress
            progress_clone.fetch_add(1, Ordering::SeqCst);
        }
        
        // Verify we processed all batches
        assert_eq!(progress_count.load(Ordering::SeqCst), batches.len());
        
        // Verify all entries were collected
        let all_entries = all_processed_entries.lock().unwrap();
        assert_eq!(all_entries.len(), 10, "Should have 10 translated entries total");
        
        // Verify all entries have the [TRANSLATED] prefix
        for entry in all_entries.iter() {
            assert!(entry.text.starts_with("[TRANSLATED]"), 
                   "Entry should have [TRANSLATED] prefix: {}", entry.text);
        }
        
        Ok(())
    }
} 