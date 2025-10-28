use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::default::Default;

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
    // @provider: LM Studio (OpenAI-compatible local server)
    LMStudio,
}

impl TranslationProvider {
    // @returns: Capitalized provider name
    pub fn display_name(&self) -> &str {
        match self {
            Self::Ollama => "Ollama",
            Self::OpenAI => "OpenAI",
            Self::Anthropic => "Anthropic",
            Self::LMStudio => "LM Studio",
        }
    }
    
    // @returns: Lowercase provider identifier
    pub fn to_lowercase_string(&self) -> String {
        match self {
            Self::Ollama => "ollama".to_string(),
            Self::OpenAI => "openai".to_string(),
            Self::Anthropic => "anthropic".to_string(),
            Self::LMStudio => "lmstudio".to_string(),
        }
    }
}

// Implement Display trait for TranslationProvider
impl std::fmt::Display for TranslationProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_lowercase_string())
    }
}

// Implement FromStr trait for TranslationProvider
impl std::str::FromStr for TranslationProvider {
    type Err = anyhow::Error;
    
    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "ollama" => Ok(Self::Ollama),
            "openai" => Ok(Self::OpenAI),
            "anthropic" => Ok(Self::Anthropic),
            "lmstudio" => Ok(Self::LMStudio),
            _ => Err(anyhow!("Invalid provider type: {}", s)),
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
    
    // @field: Rate limit (requests per minute)
    #[serde(default)]
    pub rate_limit: Option<u32>,
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
                rate_limit: default_ollama_rate_limit(),
            },
            TranslationProvider::OpenAI => Self {
                provider_type: "openai".to_string(),
                model: default_openai_model(),
                api_key: String::new(),
                endpoint: default_openai_endpoint(),
                concurrent_requests: default_concurrent_requests(),
                max_chars_per_request: default_max_chars_per_request(),
                timeout_secs: default_timeout_secs(),
                rate_limit: default_openai_rate_limit(),
            },
            TranslationProvider::Anthropic => Self {
                provider_type: "anthropic".to_string(),
                model: default_anthropic_model(),
                api_key: String::new(),
                endpoint: default_anthropic_endpoint(),
                concurrent_requests: default_concurrent_requests(),
                max_chars_per_request: default_anthropic_max_chars_per_request(),
                timeout_secs: default_anthropic_timeout_secs(),
                rate_limit: default_anthropic_rate_limit(),
            },
            TranslationProvider::LMStudio => Self {
                provider_type: "lmstudio".to_string(),
                model: default_lmstudio_model(),
                api_key: String::new(),
                endpoint: default_lmstudio_endpoint(),
                concurrent_requests: default_concurrent_requests(),
                max_chars_per_request: default_max_chars_per_request(),
                timeout_secs: default_timeout_secs(),
                rate_limit: default_lmstudio_rate_limit(),
            },
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
    #[serde(default = "default_anthropic_max_chars_per_request")]
    pub max_chars_per_request: usize,
    
    /// Request timeout in seconds
    #[serde(default = "default_anthropic_timeout_secs")]
    pub timeout_secs: u64,
    
    /// Rate limit in requests per minute (optional)
    /// 
    /// This controls the maximum number of requests that can be sent to the Anthropic API
    /// per minute. The default is 45 requests per minute (slightly below the Anthropic API
    /// limit of 50 requests per minute) to provide some safety margin.
    /// 
    /// Setting a value:
    /// - Higher than 50: Risk hitting API rate limits, causing failed requests
    /// - Between 1-45: More conservative rate limiting to avoid API rate limit errors
    /// - None or 0: Disables client-side rate limiting (not recommended)
    #[serde(default = "default_anthropic_rate_limit")]
    pub rate_limit: Option<u32>,
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
            rate_limit: default_anthropic_rate_limit(),
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
    
    /// Temperature parameter for text generation (0.0 to 1.0)
    /// Lower values make output more deterministic, higher values more creative
    #[serde(default = "default_temperature")]
    pub temperature: f32,
}

impl Default for TranslationCommonConfig {
    fn default() -> Self {
        Self {
            system_prompt: default_system_prompt(),
            rate_limit_delay_ms: default_rate_limit_delay_ms(),
            retry_count: default_retry_count(),
            retry_backoff_ms: default_retry_backoff_ms(),
            temperature: default_temperature(),
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

fn default_temperature() -> f32 {
    0.3
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

fn default_lmstudio_endpoint() -> String {
    // LM Studio default server (OpenAI compatible) runs on port 1234 under /v1
    "http://localhost:1234/v1".to_string()
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

fn default_lmstudio_model() -> String {
    // Placeholder; users should set to the loaded model name in LM Studio
    "local-model".to_string()
}

fn default_system_prompt() -> String {
    "You are a professional translator. Translate the following text from {source_language} to {target_language}. Preserve formatting and maintain the original meaning and tone.".to_string()
}

fn default_anthropic_rate_limit() -> Option<u32> {
    // Default to 45 requests per minute (slightly below the 50 limit for safety)
    // Anthropic's standard rate limit is 50 requests per minute
    // We use a slightly lower limit to prevent edge cases where our
    // timer might not be perfectly synced with Anthropic's
    Some(45)
}

// Default rate limits for providers
fn default_ollama_rate_limit() -> Option<u32> {
    None // No rate limit by default for local provider
}

fn default_openai_rate_limit() -> Option<u32> {
    Some(60) // 60 requests per minute by default
}

// LM Studio is local; do not enforce rate limiting by default
fn default_lmstudio_rate_limit() -> Option<u32> {
    None
}

impl Config {
    
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
                    return Err(anyhow!("Translation API key is required for OpenAI provider"));
                }
            },
            TranslationProvider::Anthropic => {
                let api_key = self.translation.get_api_key();
                if api_key.is_empty() {
                    return Err(anyhow!("Translation API key is required for Anthropic provider"));
                }
            },
            _ => {}
        }
        
        Ok(())
    }
    
    
}

/// Default implementation for Config
impl Default for Config {
    fn default() -> Self {
        Config {
            source_language: "en".to_string(),
            target_language: "fr".to_string(),
            translation: TranslationConfig::default(),
            log_level: LogLevel::default(),
        }
    }
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
        let provider_str = self.provider.to_lowercase_string();
        self.available_providers.iter()
            .find(|p| p.provider_type == provider_str)
    }
    
    /// Get a specific provider configuration by type for testing
    pub fn get_provider_config(&self, provider_type: &TranslationProvider) -> Option<&ProviderConfig> {
        let provider_str = provider_type.to_lowercase_string();
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
            TranslationProvider::LMStudio => default_lmstudio_model(),
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
            TranslationProvider::LMStudio => default_lmstudio_endpoint(),
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
    
    
    
    
    /// Get the rate limit for the active provider
    pub fn get_rate_limit(&self) -> Option<u32> {
        if let Some(provider_config) = self.get_active_provider_config() {
            return provider_config.rate_limit;
        }
        
        // Default fallback based on provider type
        match self.provider {
            TranslationProvider::Ollama => default_ollama_rate_limit(),
            TranslationProvider::OpenAI => default_openai_rate_limit(),
            TranslationProvider::Anthropic => default_anthropic_rate_limit(),
            TranslationProvider::LMStudio => default_lmstudio_rate_limit(),
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
        config.available_providers.push(ProviderConfig::new(TranslationProvider::LMStudio));
        
        config
    }
} 