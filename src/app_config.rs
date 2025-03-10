use anyhow::{Result, anyhow, Context};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::default::Default;
use std::fmt;
use std::str::FromStr;

/// Application configuration module
/// This module handles the application configuration including loading,
/// validating and saving configuration settings.
/// Represents the application configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    /// Source language code (ISO)
    pub source_language: String,
    
    /// Target language code (ISO)
    pub target_language: String,
    
    /// Translation config
    pub translation: TranslationConfig,
    
    /// Log level
    #[serde(default)]
    pub log_level: LogLevel,
}

/// Translation provider type
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum TranslationProvider {
    // @provider: Ollama
    #[default]
    Ollama,
    // @provider: OpenAI
    OpenAI,
    // @provider: Anthropic
    Anthropic,
}

impl TranslationProvider {
    // @returns: Capitalized provider name
    pub fn display_name(&self) -> &str {
        match self {
            Self::Ollama => "Ollama",
            Self::OpenAI => "OpenAI",
            Self::Anthropic => "Anthropic",
        }
    }
    
    // @returns: Lowercase provider identifier
    pub fn to_string(&self) -> String {
        match self {
            Self::Ollama => "ollama".to_string(),
            Self::OpenAI => "openai".to_string(),
            Self::Anthropic => "anthropic".to_string(),
        }
    }
    
    // @param s: Provider identifier string
    // @returns: Provider enum or error
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "ollama" => Ok(Self::Ollama),
            "openai" => Ok(Self::OpenAI),
            "anthropic" => Ok(Self::Anthropic),
            _ => Err(anyhow::anyhow!("Invalid provider type: {}", s)),
        }
    }
}

/// Provider configuration wrapper
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProviderConfig {
    // @field: Provider type identifier
    #[serde(rename = "type")]
    pub provider_type: String,
    
    // @field: Model name
    #[serde(default = "String::new")]
    pub model: String,
    
    // @field: API key
    #[serde(default = "String::new")]
    pub api_key: String,
    
    // @field: Service URL
    #[serde(default = "String::new")]
    pub endpoint: String,
    
    // @field: Max concurrent requests
    #[serde(default = "default_concurrent_requests")]
    pub concurrent_requests: usize,
    
    // @field: Max chars per request
    #[serde(default = "default_max_chars_per_request")]
    pub max_chars_per_request: usize,
    
    // @field: Timeout seconds
    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,
}

impl ProviderConfig {
    // @param provider_type: Provider enum
    // @returns: Provider config with defaults
    pub fn new(provider_type: TranslationProvider) -> Self {
        match provider_type {
            TranslationProvider::Ollama => Self {
                provider_type: "ollama".to_string(),
                model: default_ollama_model(),
                api_key: String::new(),
                endpoint: default_ollama_endpoint(),
                concurrent_requests: default_concurrent_requests(),
                max_chars_per_request: default_max_chars_per_request(),
                timeout_secs: default_timeout_secs(),
            },
            TranslationProvider::OpenAI => Self {
                provider_type: "openai".to_string(),
                model: default_openai_model(),
                api_key: String::new(),
                endpoint: default_openai_endpoint(),
                concurrent_requests: default_concurrent_requests(),
                max_chars_per_request: default_max_chars_per_request(),
                timeout_secs: default_timeout_secs(),
            },
            TranslationProvider::Anthropic => Self {
                provider_type: "anthropic".to_string(),
                model: default_anthropic_model(),
                api_key: String::new(),
                endpoint: default_anthropic_endpoint(),
                concurrent_requests: default_concurrent_requests(),
                max_chars_per_request: default_anthropic_max_chars_per_request(),
                timeout_secs: default_anthropic_timeout_secs(),
            },
        }
    }
    
    // @returns: Provider enum from string field
    pub fn get_provider_type(&self) -> Result<TranslationProvider> {
        TranslationProvider::from_str(&self.provider_type)
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
    #[serde(default = "default_anthropic_max_chars_per_request")]
    pub max_chars_per_request: usize,
    
    /// Request timeout in seconds
    #[serde(default = "default_anthropic_timeout_secs")]
    pub timeout_secs: u64,
}

impl Default for AnthropicConfig {
    fn default() -> Self {
        Self {
            model: default_anthropic_model(),
            api_key: String::new(),
            endpoint: default_anthropic_endpoint(),
            concurrent_requests: default_concurrent_requests(),
            max_chars_per_request: default_anthropic_max_chars_per_request(),
            timeout_secs: default_anthropic_timeout_secs(),
        }
    }
}

/// Translation service configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TranslationConfig {
    /// Translation provider to use
    #[serde(default)]
    pub provider: TranslationProvider,
    
    /// Available translation providers
    #[serde(default)]
    pub available_providers: Vec<ProviderConfig>,
    
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

fn default_anthropic_max_chars_per_request() -> usize {
    8000
}

fn default_timeout_secs() -> u64 {
    30
}

fn default_anthropic_timeout_secs() -> u64 {
    60
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
        
        Ok(config)
    }
    
    /// Validate the configuration for consistency and required values
    pub fn validate(&self) -> Result<()> {
        // Validate languages
        let _source_name = crate::language_utils::get_language_name(&self.source_language)?;
        let _target_name = crate::language_utils::get_language_name(&self.target_language)?;
        
        // Validate API key for all providers except Ollama
        match self.translation.provider {
            TranslationProvider::OpenAI => {
                let api_key = self.translation.get_api_key();
                if api_key.is_empty() {
                    return Err(anyhow::anyhow!("Translation API key is required for OpenAI provider"));
                }
            },
            TranslationProvider::Anthropic => {
                let api_key = self.translation.get_api_key();
                if api_key.is_empty() {
                    return Err(anyhow::anyhow!("Translation API key is required for Anthropic provider"));
                }
            },
            _ => {}
        }
        
        Ok(())
    }
    
    /// Create a new configuration with default values
    pub fn default_config() -> Self {
        // Create default configuration
        let config = Config {
            source_language: "en".to_string(),
            target_language: "fr".to_string(),
            translation: TranslationConfig::default(),
            log_level: LogLevel::default(),
        };
        
        config
    }
    
    /// Save the configuration to a file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, json)?;
        
        Ok(())
    }
}

pub fn create_default_config_file<P: AsRef<Path>>(path: P) -> Result<Config> {
    let config = Config::default_config();
    config.save_to_file(&path)?;
    Ok(config)
}

impl TranslationConfig {
    pub fn optimal_concurrent_requests(&self) -> usize {
        // Check if the provider exists in the available_providers
        if let Some(provider_config) = self.get_active_provider_config() {
            return provider_config.concurrent_requests;
        }
        
        // Default fallback
        default_concurrent_requests()
    }
    
    /// Get the active provider configuration from the available_providers array
    pub fn get_active_provider_config(&self) -> Option<&ProviderConfig> {
        let provider_str = self.provider.to_string();
        self.available_providers.iter()
            .find(|p| p.provider_type == provider_str)
    }
    
    /// Get a specific provider configuration by type
    pub fn get_provider_config(&self, provider_type: &TranslationProvider) -> Option<&ProviderConfig> {
        let provider_str = provider_type.to_string();
        self.available_providers.iter()
            .find(|p| p.provider_type == provider_str)
    }
    
    /// Get the model for the active provider
    pub fn get_model(&self) -> String {
        if let Some(provider_config) = self.get_active_provider_config() {
            if !provider_config.model.is_empty() {
                return provider_config.model.clone();
            }
        }
        
        // Default fallback based on provider type
        match self.provider {
            TranslationProvider::Ollama => default_ollama_model(),
            TranslationProvider::OpenAI => default_openai_model(),
            TranslationProvider::Anthropic => default_anthropic_model(),
        }
    }
    
    /// Get the API key for the active provider
    pub fn get_api_key(&self) -> String {
        if let Some(provider_config) = self.get_active_provider_config() {
            if !provider_config.api_key.is_empty() {
                return provider_config.api_key.clone();
            }
        }
        
        // Default fallback - Ollama doesn't use API keys
        String::new()
    }
    
    /// Get the endpoint for the active provider
    pub fn get_endpoint(&self) -> String {
        if let Some(provider_config) = self.get_active_provider_config() {
            if !provider_config.endpoint.is_empty() {
                return provider_config.endpoint.clone();
            }
        }
        
        // Default fallback based on provider type
        match self.provider {
            TranslationProvider::Ollama => default_ollama_endpoint(),
            TranslationProvider::OpenAI => default_openai_endpoint(),
            TranslationProvider::Anthropic => default_anthropic_endpoint(),
        }
    }
    
    /// Get the max chars per request for the active provider
    pub fn get_max_chars_per_request(&self) -> usize {
        if let Some(provider_config) = self.get_active_provider_config() {
            if provider_config.max_chars_per_request > 0 {
                return provider_config.max_chars_per_request;
            }
        }
        
        // Default fallback
        default_max_chars_per_request()
    }
    
    /// Get the timeout in seconds for the active provider
    pub fn get_timeout_secs(&self) -> u64 {
        if let Some(provider_config) = self.get_active_provider_config() {
            if provider_config.timeout_secs > 0 {
                return provider_config.timeout_secs;
            }
        }
        
        // Default fallback
        default_timeout_secs()
    }
    
    /// Get the rate limit delay in milliseconds
    pub fn rate_limit_delay_ms(&self) -> u64 {
        if self.common.rate_limit_delay_ms > 0 {
            self.common.rate_limit_delay_ms
        } else {
            default_rate_limit_delay_ms()
        }
    }
}

impl Default for TranslationConfig {
    fn default() -> Self {
        let mut config = Self {
            provider: TranslationProvider::default(),
            available_providers: Vec::new(),
            common: TranslationCommonConfig::default(),
        };
        
        // Add default providers
        config.available_providers.push(ProviderConfig::new(TranslationProvider::Ollama));
        config.available_providers.push(ProviderConfig::new(TranslationProvider::OpenAI));
        config.available_providers.push(ProviderConfig::new(TranslationProvider::Anthropic));
        
        config
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
        
        // Test provider config values
        let ollama_config = config.translation.get_provider_config(&TranslationProvider::Ollama)
            .expect("Ollama provider config should exist");
        assert_eq!(ollama_config.concurrent_requests, default_concurrent_requests());
        assert_eq!(ollama_config.max_chars_per_request, default_max_chars_per_request());
        assert_eq!(ollama_config.timeout_secs, default_timeout_secs());
        assert_eq!(ollama_config.model, default_ollama_model());
        
        assert_eq!(config.log_level, LogLevel::Info);
    }

    #[test]
    fn test_config_validation() {
        // Start with a valid config
        let mut config = Config::default_config();
        assert!(config.validate().is_ok());
        
        // Invalid source language
        config.source_language = "xyz".to_string();
        assert!(config.validate().is_err());
        config.source_language = "en".to_string();
        
        // Invalid target language
        config.target_language = "".to_string();
        assert!(config.validate().is_err());
        config.target_language = "fr".to_string();
        
        // For OpenAI provider that requires an API key
        config.translation.provider = TranslationProvider::OpenAI;
        
        // Make sure available_providers has entries
        if config.translation.available_providers.is_empty() {
            // Initialize default providers if empty
            config.translation.available_providers.push(ProviderConfig::new(TranslationProvider::Ollama));
            config.translation.available_providers.push(ProviderConfig::new(TranslationProvider::OpenAI));
            config.translation.available_providers.push(ProviderConfig::new(TranslationProvider::Anthropic));
        }
        
        // First update the API key in available_providers 
        if let Some(provider) = config.translation
            .available_providers
            .iter_mut()
            .find(|p| p.provider_type == "openai") {
            provider.api_key = "".to_string();
        }
        
        // OpenAI with empty API key should fail validation
        assert!(config.validate().is_err());
        
        // Set a valid API key in available_providers
        if let Some(provider) = config.translation
            .available_providers
            .iter_mut()
            .find(|p| p.provider_type == "openai") {
            provider.api_key = "sk-1234567890".to_string();
        }
        
        // Valid with API key
        assert!(config.validate().is_ok());
        
        // Ollama doesn't require API key
        config.translation.provider = TranslationProvider::Ollama;
        assert!(config.validate().is_ok());
    }
} 