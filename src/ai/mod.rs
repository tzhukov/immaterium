// AI engine module
// Handles LLM provider integration

pub mod engine;
pub mod provider;
pub mod providers;

pub use engine::AiEngine;
pub use provider::{AiError, ChatRequest, ChatResponse, LlmProvider, Message, MessageRole};
pub use providers::OllamaProvider;
