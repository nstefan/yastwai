/*! 
 * Error types for the yastwai application.
 * 
 * This module contains custom error types for different parts of the application,
 * using the thiserror crate for ergonomic error definitions.
 */

use thiserror::Error;

/// Errors that can occur when working with provider APIs
#[derive(Error, Debug)]
pub enum ProviderError {
    /// Error when making an API request fails
    #[error("API request failed: {0}")]
    RequestFailed(String),
    
    /// Error when parsing an API response fails
    #[error("Failed to parse API response: {0}")]
    ParseError(String),
    
    /// Error returned by the API itself
    #[error("API responded with error: {status_code} - {message}")]
    ApiError { 
        /// HTTP status code 
        status_code: u16, 
        /// Error message from the API
        message: String 
    },
    
    /// Error establishing or maintaining a connection
    #[error("Connection error: {0}")]
    ConnectionError(String),
    
    /// Error related to rate limiting
    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),
    
    /// Error with authentication
    #[error("Authentication error: {0}")]
    AuthenticationError(String),
}

/// Errors that can occur during subtitle processing
#[derive(Error, Debug)]
pub enum SubtitleError {
    /// Error when parsing a subtitle file
    #[error("Failed to parse subtitle file: {0}")]
    ParseError(String),
    
    /// Error when writing a subtitle file
    #[error("Failed to write subtitle file: {0}")]
    WriteError(String),
    
    /// Error when converting subtitle formats
    #[error("Failed to convert subtitle format: {0}")]
    ConversionError(String),
}

/// Errors that can occur during translation
#[derive(Error, Debug)]
pub enum TranslationError {
    /// Error from the provider API
    #[error("Provider error: {0}")]
    ProviderError(#[from] ProviderError),
    
    /// Error with subtitle processing
    #[error("Subtitle error: {0}")]
    SubtitleError(#[from] SubtitleError),
    
    /// Error with configuration
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    /// Error with unsupported language
    #[error("Unsupported language: {0}")]
    UnsupportedLanguage(String),
}

/// Main application error type that wraps all other errors
#[derive(Error, Debug)]
pub enum AppError {
    /// Error from a file operation
    #[error("File error: {0}")]
    FileError(String),
    
    /// Error from a provider
    #[error("Provider error: {0}")]
    ProviderError(#[from] ProviderError),
    
    /// Error from subtitle processing
    #[error("Subtitle error: {0}")]
    SubtitleError(#[from] SubtitleError),
    
    /// Error from translation
    #[error("Translation error: {0}")]
    TranslationError(#[from] TranslationError),
    
    /// Error from configuration
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Any other error
    #[error("Unknown error: {0}")]
    Unknown(String),
}

// Utility functions for error conversion
impl From<anyhow::Error> for AppError {
    fn from(error: anyhow::Error) -> Self {
        Self::Unknown(error.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> Self {
        Self::FileError(error.to_string())
    }
} 