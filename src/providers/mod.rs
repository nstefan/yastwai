/*! 
 * Provider implementations for different translation services.
 * 
 * This module contains client implementations for various LLM providers:
 * - Ollama: Local LLM server
 * - OpenAI: OpenAI API integration 
 * - Anthropic: Anthropic API integration
 */

pub mod ollama;
pub mod openai;
pub mod anthropic; 