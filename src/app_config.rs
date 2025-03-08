use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::default::Default;
use log::{info, warn};

/// Application configuration module
/// This module handles the application configuration including loading,
/// validating and saving configuration settings.
/// Represents the application configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    /// Source language code (e.g., "en", "fr")
    pub source_language: String,
    
    /// Target language code for translation
    pub target_language: String,
    
    /// Translation services configuration
    pub translation: TranslationConfig,
    
    /// Log verbosity level
    #[serde(default)]
    pub log_level: LogLevel,
}

/// Translation provider type
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum TranslationProvider {
    /// Ollama LLM translation service
    #[default]
    Ollama,
    /// OpenAI API translation service
    OpenAI,
    /// Anthropic API translation service
    Anthropic,
}

impl TranslationProvider {
    /// Returns the properly capitalized name of the provider
    pub fn display_name(&self) -> &str {
        match self {
            Self::Ollama => "Ollama",
            Self::OpenAI => "OpenAI",
            Self::Anthropic => "Anthropic",
        }
    }
}

/// Ollama service configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OllamaConfig {
    /// Model name (e.g., "llama2", "mistral")
    #[serde(default = "default_ollama_model")]
    pub model: String,
    
    /// Service endpoint URL
    #[serde(default = "default_ollama_endpoint")]
    pub endpoint: String,

    /// Maximum number of concurrent requests
    #[serde(default = "default_concurrent_requests")]
    pub concurrent_requests: usize,
    
    /// Maximum subtitle characters per request
    #[serde(default = "default_max_chars_per_request")]
    pub max_chars_per_request: usize,
    
    /// Request timeout in seconds
    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            model: default_ollama_model(),
            endpoint: default_ollama_endpoint(),
            concurrent_requests: default_concurrent_requests(),
            max_chars_per_request: default_max_chars_per_request(),
            timeout_secs: default_timeout_secs(),
        }
    }
}

/// OpenAI service configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OpenAIConfig {
    /// Model name (e.g., "gpt-4", "gpt-3.5-turbo")
    #[serde(default = "default_openai_model")]
    pub model: String,
    
    /// API key for the service
    #[serde(default = "String::new")]
    pub api_key: String,
    
    /// Service endpoint URL (optional, for Azure OpenAI or self-hosted)
    #[serde(default = "default_openai_endpoint")]
    pub endpoint: String,

    /// Maximum number of concurrent requests
    #[serde(default = "default_concurrent_requests")]
    pub concurrent_requests: usize,
    
    /// Maximum subtitle characters per request
    #[serde(default = "default_max_chars_per_request")]
    pub max_chars_per_request: usize,
    
    /// Request timeout in seconds
    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,
}

impl Default for OpenAIConfig {
    fn default() -> Self {
        Self {
            model: default_openai_model(),
            api_key: String::new(),
            endpoint: default_openai_endpoint(),
            concurrent_requests: default_concurrent_requests(),
            max_chars_per_request: default_max_chars_per_request(),
            timeout_secs: default_timeout_secs(),
        }
    }
}

/// Anthropic service configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AnthropicConfig {
    /// Model name (e.g., "claude-3-opus", "claude-3-sonnet")
    #[serde(default = "default_anthropic_model")]
    pub model: String,
    
    /// API key for the service
    #[serde(default = "String::new")]
    pub api_key: String,
    
    /// Service endpoint URL (optional, for self-hosted)
    #[serde(default = "default_anthropic_endpoint")]
    pub endpoint: String,

    /// Maximum number of concurrent requests
    #[serde(default = "default_concurrent_requests")]
    pub concurrent_requests: usize,
    
    /// Maximum subtitle characters per request
    #[serde(default = "default_max_chars_per_request")]
    pub max_chars_per_request: usize,
    
    /// Request timeout in seconds
    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,
}

impl Default for AnthropicConfig {
    fn default() -> Self {
        Self {
            model: default_anthropic_model(),
            api_key: String::new(),
            endpoint: default_anthropic_endpoint(),
            concurrent_requests: default_concurrent_requests(),
            max_chars_per_request: default_max_chars_per_request(),
            timeout_secs: default_timeout_secs(),
        }
    }
}

/// Translation service configuration
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct TranslationConfig {
    /// Translation provider to use
    #[serde(default)]
    pub provider: TranslationProvider,
    
    /// Ollama configuration
    #[serde(default)]
    pub ollama: OllamaConfig,
    
    /// OpenAI configuration
    #[serde(default)]
    pub openai: OpenAIConfig,
    
    /// Anthropic configuration
    #[serde(default)]
    pub anthropic: AnthropicConfig,
    
    /// Common translation settings
    #[serde(default)]
    pub common: TranslationCommonConfig,
}

/// Common translation settings applicable to all providers
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TranslationCommonConfig {
    /// System prompt template for translation
    /// Placeholders: {source_language}, {target_language}
    #[serde(default = "default_system_prompt")]
    pub system_prompt: String,
    
    /// Rate limit delay in milliseconds between consecutive requests
    #[serde(default = "default_rate_limit_delay_ms")]
    pub rate_limit_delay_ms: u64,
    
    /// Retry count for failed requests
    #[serde(default = "default_retry_count")]
    pub retry_count: u32,
    
    /// Backoff multiplier for retries (in milliseconds)
    #[serde(default = "default_retry_backoff_ms")]
    pub retry_backoff_ms: u64,
}

impl Default for TranslationCommonConfig {
    fn default() -> Self {
        Self {
            system_prompt: default_system_prompt(),
            rate_limit_delay_ms: default_rate_limit_delay_ms(),
            retry_count: default_retry_count(),
            retry_backoff_ms: default_retry_backoff_ms(),
        }
    }
}

/// Information about a subtitle track
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubtitleInfo {
    /// The index/id of the subtitle track
    pub index: usize,
    /// The codec name of the subtitle track
    pub codec_name: String,
    /// The language code (ISO 639-1 or ISO 639-2)
    pub language: Option<String>,
    /// The title of the subtitle track if available
    pub title: Option<String>,
}

/// Configuration for subtitle processing
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SubtitleConfig {
    /// Whether to preserve formatting tags
    #[serde(default = "default_true")]
    pub preserve_formatting: bool,
    
    /// Whether to adjust timing for better readability
    #[serde(default)]
    pub adjust_timing: bool,
}

impl Default for SubtitleConfig {
    fn default() -> Self {
        Self {
            preserve_formatting: true,
            adjust_timing: false,
        }
    }
}

/// Log verbosity level
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Error,
    Warn,
    #[default]
    Info,
    Debug,
    Trace,
}

fn default_concurrent_requests() -> usize {
    4
}

fn default_max_chars_per_request() -> usize {
    1000
}

fn default_timeout_secs() -> u64 {
    30
}

fn default_rate_limit_delay_ms() -> u64 {
    500 // 500ms default delay between requests
}

fn default_retry_count() -> u32 {
    3 // Default to 3 retries
}

fn default_retry_backoff_ms() -> u64 {
    1000 // 1 second base backoff time, doubled on each retry
}

fn default_true() -> bool {
    true
}

fn default_ollama_endpoint() -> String {
    "http://localhost:11434".to_string()
}

fn default_openai_endpoint() -> String {
    "https://api.openai.com/v1".to_string()
}

fn default_anthropic_endpoint() -> String {
    "https://api.anthropic.com".to_string()
}

fn default_ollama_model() -> String {
    "llama2".to_string()
}

fn default_openai_model() -> String {
    "gpt-3.5-turbo".to_string()
}

fn default_anthropic_model() -> String {
    "claude-3-haiku".to_string()
}

fn default_system_prompt() -> String {
    "You are a professional translator. Translate the following text from {source_language} to {target_language}. Preserve formatting and maintain the original meaning and tone.".to_string()
}

impl Config {
    /// Load configuration from a file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::open(path.as_ref())
            .with_context(|| format!("Failed to open config file: {:?}", path.as_ref()))?;
        
        let reader = BufReader::new(file);
        let config: Config = serde_json::from_reader(reader)?;
        
        // Don't log the entire config - it may contain API keys
        info!("Config loaded from {:?}", path.as_ref());
        
        Ok(config)
    }
    
    /// Validate the configuration for consistency and required values
    pub fn validate(&self) -> Result<()> {
        // Validate languages
        let source_name = crate::language_utils::get_language_name(&self.source_language)?;
        let target_name = crate::language_utils::get_language_name(&self.target_language)?;
        
        info!("Translation: {} ({}) → {} ({})", 
            self.source_language, source_name, 
            self.target_language, target_name);
        
        // Validate API key for all providers except Ollama
        match self.translation.provider {
            TranslationProvider::OpenAI if self.translation.openai.api_key.is_empty() => {
                return Err(anyhow::anyhow!("Translation API key is required for OpenAI provider"));
            },
            TranslationProvider::Anthropic if self.translation.anthropic.api_key.is_empty() => {
                return Err(anyhow::anyhow!("Translation API key is required for Anthropic provider"));
            },
            _ => {}
        }
        
        // No need to log successful validation
        
        Ok(())
    }
    
    /// Create a new configuration with default values
    pub fn default_config() -> Self {
        Config {
            source_language: "en".to_string(),
            target_language: "fr".to_string(),
            translation: TranslationConfig::default(),
            log_level: LogLevel::default(),
        }
    }
    
    /// Save the configuration to a file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, json)?;
        
        info!("Config saved");
        
        Ok(())
    }
}

pub fn create_default_config_file<P: AsRef<Path>>(path: P) -> Result<Config> {
    let config = Config::default_config();
    config.save_to_file(&path)?;
    Ok(config)
}

impl TranslationConfig {
    /// Get the provider-specific optimal concurrent requests
    pub fn optimal_concurrent_requests(&self) -> usize {
        // Get the provider-specific configured value
        let base_concurrent = match self.provider {
            TranslationProvider::OpenAI => self.openai.concurrent_requests,
            TranslationProvider::Anthropic => self.anthropic.concurrent_requests,
            TranslationProvider::Ollama => self.ollama.concurrent_requests,
        };
        
        // Adjust based on the provider
        match self.provider {
            TranslationProvider::OpenAI => {
                // OpenAI allows up to 500 RPM for newer models like GPT-4o
                // But we'll be conservative with 60 RPM (1 per second)
                base_concurrent.min(10)
            },
            TranslationProvider::Anthropic => {
                // Anthropic Claude has stricter rate limits (~100 RPM)
                // We'll be more conservative with ~30 RPM
                base_concurrent.min(5)
            },
            TranslationProvider::Ollama => {
                // For local Ollama, limit based on local hardware capabilities
                // This is more about preventing resource exhaustion than API limits
                base_concurrent.min(8)
            },
        }
    }
    
    /// Get the provider-specific rate limit delay in milliseconds
    pub fn rate_limit_delay_ms(&self) -> u64 {
        // Start with the configured delay
        let base_delay = self.common.rate_limit_delay_ms;
        
        // Adjust based on the provider
        match self.provider {
            TranslationProvider::OpenAI => {
                // For OpenAI, maintain a minimum of 100ms between requests
                base_delay.max(100)
            },
            TranslationProvider::Anthropic => {
                // For Anthropic, ensure at least 200ms between requests
                base_delay.max(200)
            },
            TranslationProvider::Ollama => {
                // For local Ollama, minimal delay needed
                base_delay.max(50)
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default_config();
        
        // Test default values
        assert_eq!(config.source_language, "en");
        assert_eq!(config.target_language, "fr");
        assert_eq!(config.translation.provider, TranslationProvider::Ollama);
        assert_eq!(config.translation.ollama.concurrent_requests, 4);
        assert_eq!(config.translation.ollama.max_chars_per_request, 1000);
        assert_eq!(config.translation.ollama.timeout_secs, 30);
        assert_eq!(config.translation.ollama.model, "llama2");
        assert_eq!(config.log_level, LogLevel::Info);
    }

    #[test]
    fn test_config_validation() {
        // Valid config
        let mut config = Config::default_config();
        assert!(config.validate().is_ok());
        
        // Invalid source language
        config.source_language = "".to_string();
        assert!(config.validate().is_err());
        config.source_language = "en".to_string();
        
        // Invalid target language
        config.target_language = "".to_string();
        assert!(config.validate().is_err());
        config.target_language = "fr".to_string();
        
        // Missing API key for providers that require it
        config.translation.provider = TranslationProvider::OpenAI;
        config.translation.openai.api_key = "".to_string();
        assert!(config.validate().is_err());
        
        // Valid with API key
        config.translation.openai.api_key = "sk-1234567890".to_string();
        assert!(config.validate().is_ok());
        
        // Ollama doesn't require API key
        config.translation.provider = TranslationProvider::Ollama;
        assert!(config.validate().is_ok());
    }
} 