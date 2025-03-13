/*! 
 * Provider implementations for different translation services.
 * 
 * This module contains client implementations for various LLM providers:
 * - Ollama: Local LLM server
 * - OpenAI: OpenAI API integration 
 * - Anthropic: Anthropic API integration
 */

use async_trait::async_trait;
use std::fmt::Debug;

use crate::errors::ProviderError;

/// Common trait for all LLM providers
/// 
/// This trait defines the interface that all provider implementations must follow,
/// allowing them to be used interchangeably in the translation service.
#[async_trait]
pub trait Provider: Send + Sync + Debug {
    /// The request type for this provider
    type Request: Send + Sync;
    
    /// The response type for this provider
    type Response: Send + Sync;
    
    /// Complete a request using this provider
    /// 
    /// # Arguments
    /// * `request` - The request to complete
    /// 
    /// # Returns
    /// * `Result<Self::Response, ProviderError>` - The response from the provider or an error
    async fn complete(&self, request: Self::Request) -> Result<Self::Response, ProviderError>;
    
    /// Test the connection to the provider
    /// 
    /// # Returns
    /// * `Result<(), ProviderError>` - Ok if the connection is successful, or an error
    async fn test_connection(&self) -> Result<(), ProviderError>;
    
    /// Extract text from the provider response
    /// 
    /// # Arguments
    /// * `response` - The response from the provider
    /// 
    /// # Returns
    /// * `String` - The extracted text
    fn extract_text(response: &Self::Response) -> String;
}

pub mod ollama;
pub mod openai;
pub mod anthropic; 