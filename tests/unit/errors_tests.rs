/*!
 * Tests for error types and conversions
 */

use yastwai::errors::{ProviderError, TranslationError, AppError};

#[test]
fn test_providerError_requestFailed_shouldDisplayCorrectly() {
    let error = ProviderError::RequestFailed("Connection timeout".to_string());
    let display = format!("{}", error);
    assert!(display.contains("API request failed"));
    assert!(display.contains("Connection timeout"));
}

#[test]
fn test_providerError_parseError_shouldDisplayCorrectly() {
    let error = ProviderError::ParseError("Invalid JSON".to_string());
    let display = format!("{}", error);
    assert!(display.contains("Failed to parse API response"));
    assert!(display.contains("Invalid JSON"));
}

#[test]
fn test_providerError_apiError_shouldDisplayStatusAndMessage() {
    let error = ProviderError::ApiError {
        status_code: 429,
        message: "Too many requests".to_string(),
    };
    let display = format!("{}", error);
    assert!(display.contains("429"));
    assert!(display.contains("Too many requests"));
}

#[test]
fn test_providerError_connectionError_shouldDisplayCorrectly() {
    let error = ProviderError::ConnectionError("Host unreachable".to_string());
    let display = format!("{}", error);
    assert!(display.contains("Connection error"));
    assert!(display.contains("Host unreachable"));
}

#[test]
fn test_providerError_rateLimitExceeded_shouldDisplayCorrectly() {
    let error = ProviderError::RateLimitExceeded { message: "Retry after 60s".to_string(), retry_after_secs: None };
    let display = format!("{}", error);
    assert!(display.contains("Rate limit exceeded"));
    assert!(display.contains("Retry after 60s"));
}

#[test]
fn test_providerError_authenticationError_shouldDisplayCorrectly() {
    let error = ProviderError::AuthenticationError("Invalid API key".to_string());
    let display = format!("{}", error);
    assert!(display.contains("Authentication error"));
    assert!(display.contains("Invalid API key"));
}

#[test]
fn test_translationError_fromProviderError_shouldWrapCorrectly() {
    let provider_error = ProviderError::RequestFailed("Test error".to_string());
    let translation_error: TranslationError = provider_error.into();
    let display = format!("{}", translation_error);
    assert!(display.contains("Provider error"));
}

#[test]
fn test_appError_fromProviderError_shouldWrapCorrectly() {
    let provider_error = ProviderError::ConnectionError("Network down".to_string());
    let app_error: AppError = provider_error.into();
    let display = format!("{}", app_error);
    assert!(display.contains("Provider error"));
}

#[test]
fn test_appError_fromIoError_shouldWrapAsFileError() {
    let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
    let app_error: AppError = io_error.into();
    let display = format!("{}", app_error);
    assert!(display.contains("File error"));
    assert!(display.contains("File not found"));
}

#[test]
fn test_appError_fromAnyhowError_shouldWrapAsUnknown() {
    let anyhow_error = anyhow::anyhow!("Something went wrong");
    let app_error: AppError = anyhow_error.into();
    let display = format!("{}", app_error);
    assert!(display.contains("Unknown error"));
    assert!(display.contains("Something went wrong"));
}

#[test]
fn test_appError_file_shouldDisplayCorrectly() {
    let error = AppError::File("Permission denied".to_string());
    let display = format!("{}", error);
    assert!(display.contains("File error"));
    assert!(display.contains("Permission denied"));
}

#[test]
fn test_appError_unknown_shouldDisplayCorrectly() {
    let error = AppError::Unknown("Unexpected state".to_string());
    let display = format!("{}", error);
    assert!(display.contains("Unknown error"));
    assert!(display.contains("Unexpected state"));
}

#[test]
fn test_providerError_debug_shouldBeImplemented() {
    let error = ProviderError::RequestFailed("test".to_string());
    let debug = format!("{:?}", error);
    assert!(debug.contains("RequestFailed"));
}

#[test]
fn test_translationError_debug_shouldBeImplemented() {
    let provider_error = ProviderError::ParseError("test".to_string());
    let translation_error: TranslationError = provider_error.into();
    let debug = format!("{:?}", translation_error);
    assert!(debug.contains("Provider"));
}

#[test]
fn test_appError_debug_shouldBeImplemented() {
    let error = AppError::File("test".to_string());
    let debug = format!("{:?}", error);
    assert!(debug.contains("File"));
}

