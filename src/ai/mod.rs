// AI engine module
// Handles LLM provider integration

pub mod context;
pub mod engine;
pub mod provider;
pub mod providers;

pub use context::{build_minimal_context, build_session_context, ContextBuilder, ContextConfig};
pub use engine::AiEngine;
pub use provider::{AiError, ChatRequest, ChatResponse, LlmProvider, Message, MessageRole, StreamResponse, Usage};
pub use providers::OllamaProvider;
