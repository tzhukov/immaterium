use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use tokio_stream::Stream;

pub type StreamResponse = Pin<Box<dyn Stream<Item = Result<String, AiError>> + Send>>;

/// Trait for LLM providers
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Get the provider name
    fn name(&self) -> &str;

    /// Check if the provider is available/configured
    async fn is_available(&self) -> bool;

    /// Send a chat completion request
    async fn chat_completion(&self, request: ChatRequest) -> Result<ChatResponse, AiError>;

    /// Send a streaming chat completion request
    async fn chat_completion_stream(&self, request: ChatRequest) -> Result<StreamResponse, AiError>;

    /// List available models
    async fn list_models(&self) -> Result<Vec<String>, AiError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub messages: Vec<Message>,
    pub model: String,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub stream: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    System,
    User,
    Assistant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub content: String,
    pub model: String,
    pub finish_reason: Option<String>,
    pub usage: Option<Usage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, thiserror::Error)]
pub enum AiError {
    #[error("Provider not configured: {0}")]
    NotConfigured(String),
    
    #[error("API error: {0}")]
    ApiError(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    
    #[error("Model not found: {0}")]
    ModelNotFound(String),
    
    #[error("Streaming error: {0}")]
    StreamError(String),
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl ChatRequest {
    pub fn new(model: String) -> Self {
        Self {
            messages: Vec::new(),
            model,
            temperature: Some(0.7),
            max_tokens: Some(2048),
            stream: false,
        }
    }

    pub fn with_system_message(mut self, content: String) -> Self {
        self.messages.push(Message {
            role: MessageRole::System,
            content,
        });
        self
    }

    pub fn with_user_message(mut self, content: String) -> Self {
        self.messages.push(Message {
            role: MessageRole::User,
            content,
        });
        self
    }

    pub fn with_assistant_message(mut self, content: String) -> Self {
        self.messages.push(Message {
            role: MessageRole::Assistant,
            content,
        });
        self
    }

    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    pub fn streaming(mut self, stream: bool) -> Self {
        self.stream = stream;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_request_builder() {
        let request = ChatRequest::new("gpt-4".to_string())
            .with_system_message("You are a helpful assistant".to_string())
            .with_user_message("Hello".to_string())
            .with_temperature(0.8)
            .with_max_tokens(1000);

        assert_eq!(request.model, "gpt-4");
        assert_eq!(request.messages.len(), 2);
        assert_eq!(request.messages[0].role, MessageRole::System);
        assert_eq!(request.messages[1].role, MessageRole::User);
        assert_eq!(request.temperature, Some(0.8));
        assert_eq!(request.max_tokens, Some(1000));
    }

    #[test]
    fn test_message_roles() {
        let system = Message {
            role: MessageRole::System,
            content: "test".to_string(),
        };
        let user = Message {
            role: MessageRole::User,
            content: "test".to_string(),
        };
        let assistant = Message {
            role: MessageRole::Assistant,
            content: "test".to_string(),
        };

        assert_eq!(system.role, MessageRole::System);
        assert_eq!(user.role, MessageRole::User);
        assert_eq!(assistant.role, MessageRole::Assistant);
    }
}

