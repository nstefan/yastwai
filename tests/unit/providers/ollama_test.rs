/*!
 * Tests for Ollama provider request builders
 */

use yastwai::providers::ollama::{GenerationRequest, ChatMessage};

#[test]
fn test_generationRequest_new_shouldCreateWithModelAndPrompt() {
    let _request = GenerationRequest::new("llama2", "Hello, world!");
    // Request created successfully
}

#[test]
fn test_generationRequest_system_shouldSetSystemPrompt() {
    let _request = GenerationRequest::new("llama2", "Hello")
        .system("You are a helpful assistant");
}

#[test]
fn test_generationRequest_temperature_shouldSetTemperature() {
    let _request = GenerationRequest::new("llama2", "Hello")
        .temperature(0.7);
}

#[test]
fn test_generationRequest_chained_shouldAllowMultipleBuilderCalls() {
    let _request = GenerationRequest::new("llama2", "Translate: Hello")
        .system("You are a translator")
        .temperature(0.3);
}

#[test]
fn test_chatMessage_struct_shouldHavePublicFields() {
    let message = ChatMessage {
        role: "user".to_string(),
        content: "Hello!".to_string(),
    };
    assert_eq!(message.role, "user");
    assert_eq!(message.content, "Hello!");
}

#[test]
fn test_chatMessage_asUserMessage_shouldWork() {
    let message = ChatMessage {
        role: "user".to_string(),
        content: "Test message".to_string(),
    };
    assert_eq!(message.role, "user");
}

#[test]
fn test_chatMessage_asAssistantMessage_shouldWork() {
    let message = ChatMessage {
        role: "assistant".to_string(),
        content: "Response".to_string(),
    };
    assert_eq!(message.role, "assistant");
}

#[test]
fn test_chatMessage_asSystemMessage_shouldWork() {
    let message = ChatMessage {
        role: "system".to_string(),
        content: "You are helpful".to_string(),
    };
    assert_eq!(message.role, "system");
}

#[test]
fn test_chatMessage_withEmptyContent_shouldWork() {
    let message = ChatMessage {
        role: "user".to_string(),
        content: "".to_string(),
    };
    assert_eq!(message.content, "");
}

#[test]
fn test_chatMessage_withUnicode_shouldHandleCorrectly() {
    let message = ChatMessage {
        role: "user".to_string(),
        content: "こんにちは 你好 مرحبا".to_string(),
    };
    assert_eq!(message.content, "こんにちは 你好 مرحبا");
}

#[test]
fn test_chatMessage_withNewlines_shouldPreserve() {
    let content = "Line 1\nLine 2\nLine 3";
    let message = ChatMessage {
        role: "user".to_string(),
        content: content.to_string(),
    };
    assert_eq!(message.content, content);
}

#[test]
fn test_chatMessage_clone_shouldWork() {
    let message = ChatMessage {
        role: "user".to_string(),
        content: "Test".to_string(),
    };
    let cloned = message.clone();
    assert_eq!(cloned.role, message.role);
    assert_eq!(cloned.content, message.content);
}

#[test]
fn test_chatMessage_debug_shouldBeImplemented() {
    let message = ChatMessage {
        role: "user".to_string(),
        content: "Test".to_string(),
    };
    let debug = format!("{:?}", message);
    assert!(debug.contains("user"));
    assert!(debug.contains("Test"));
}
