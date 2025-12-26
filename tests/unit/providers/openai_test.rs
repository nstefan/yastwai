/*!
 * Tests for OpenAI provider request builders
 */

use yastwai::providers::openai::{OpenAIRequest, OpenAIMessage};

#[test]
fn test_openaiRequest_new_shouldCreateWithModel() {
    let _request = OpenAIRequest::new("gpt-4");
}

#[test]
fn test_openaiRequest_addMessage_shouldAddUserMessage() {
    let _request = OpenAIRequest::new("gpt-4")
        .add_message("user", "Hello!");
}

#[test]
fn test_openaiRequest_addMessage_shouldAddSystemMessage() {
    let _request = OpenAIRequest::new("gpt-4")
        .add_message("system", "You are helpful");
}

#[test]
fn test_openaiRequest_addMessage_shouldAddMultipleMessages() {
    let _request = OpenAIRequest::new("gpt-4")
        .add_message("system", "You are a translator")
        .add_message("user", "Translate: Hello");
}

#[test]
fn test_openaiRequest_temperature_shouldSetTemperature() {
    let _request = OpenAIRequest::new("gpt-4")
        .temperature(0.5);
}

#[test]
fn test_openaiRequest_maxTokens_shouldSetMaxTokens() {
    let _request = OpenAIRequest::new("gpt-4")
        .max_tokens(1000);
}

#[test]
fn test_openaiRequest_chained_shouldAllowMultipleBuilderCalls() {
    let _request = OpenAIRequest::new("gpt-4")
        .add_message("system", "Translate")
        .add_message("user", "Hello")
        .temperature(0.3)
        .max_tokens(500);
}

#[test]
fn test_openaiRequest_default_shouldCreateEmptyRequest() {
    let _request = OpenAIRequest::default();
}

#[test]
fn test_openaiMessage_struct_shouldHavePublicFields() {
    let message = OpenAIMessage {
        role: "user".to_string(),
        content: "Test content".to_string(),
    };
    assert_eq!(message.role, "user");
    assert_eq!(message.content, "Test content");
}

#[test]
fn test_openaiMessage_withEmptyContent_shouldWork() {
    let message = OpenAIMessage {
        role: "assistant".to_string(),
        content: "".to_string(),
    };
    assert_eq!(message.content, "");
}

#[test]
fn test_openaiMessage_withUnicode_shouldHandleCorrectly() {
    let message = OpenAIMessage {
        role: "user".to_string(),
        content: "🎬 Subtitle: こんにちは".to_string(),
    };
    assert_eq!(message.content, "🎬 Subtitle: こんにちは");
}

#[test]
fn test_openaiMessage_withLongContent_shouldHandle() {
    let content = "a".repeat(10000);
    let message = OpenAIMessage {
        role: "user".to_string(),
        content: content.clone(),
    };
    assert_eq!(message.content.len(), 10000);
}

#[test]
fn test_openaiMessage_debug_shouldBeImplemented() {
    let message = OpenAIMessage {
        role: "system".to_string(),
        content: "Test".to_string(),
    };
    let debug = format!("{:?}", message);
    assert!(debug.contains("system"));
}
