use crate::ai::provider::{AiError, ChatRequest, ChatResponse, LlmProvider, StreamResponse, Usage};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use futures::StreamExt;

pub struct GroqProvider {
    client: Client,
    api_key: String,
    default_model: String,
}

impl GroqProvider {
    pub fn new(api_key: String, default_model: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            default_model,
        }
    }

    fn chat_url(&self) -> &str {
        "https://api.groq.com/openai/v1/chat/completions"
    }

    fn models_url(&self) -> &str {
        "https://api.groq.com/openai/v1/models"
    }
}

#[async_trait]
impl LlmProvider for GroqProvider {
    fn name(&self) -> &str {
        "groq"
    }

    async fn is_available(&self) -> bool {
        // Try to list models to check if API key is valid
        self.client
            .get(self.models_url())
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }

    async fn chat_completion(&self, request: ChatRequest) -> Result<ChatResponse, AiError> {
        let groq_request = GroqChatRequest {
            model: request.model,
            messages: request
                .messages
                .iter()
                .map(|m| GroqMessage {
                    role: format!("{:?}", m.role).to_lowercase(),
                    content: m.content.clone(),
                })
                .collect(),
            temperature: request.temperature,
            max_tokens: request.max_tokens,
            stream: false,
        };

        let response = self
            .client
            .post(self.chat_url())
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&groq_request)
            .send()
            .await
            .map_err(|e| AiError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            
            return Err(if status.as_u16() == 429 {
                AiError::RateLimitExceeded
            } else {
                AiError::ApiError(format!("Groq API error {}: {}", status, error_text))
            });
        }

        let groq_response: GroqChatResponse = response
            .json()
            .await
            .map_err(|e| AiError::ApiError(e.to_string()))?;

        let choice = groq_response
            .choices
            .first()
            .ok_or_else(|| AiError::ApiError("No choices in response".to_string()))?;

        Ok(ChatResponse {
            content: choice.message.content.clone(),
            model: groq_response.model,
            finish_reason: Some(choice.finish_reason.clone()),
            usage: groq_response.usage.map(|u| Usage {
                prompt_tokens: u.prompt_tokens,
                completion_tokens: u.completion_tokens,
                total_tokens: u.total_tokens,
            }),
        })
    }

    async fn chat_completion_stream(&self, request: ChatRequest) -> Result<StreamResponse, AiError> {
        let groq_request = GroqChatRequest {
            model: request.model,
            messages: request
                .messages
                .iter()
                .map(|m| GroqMessage {
                    role: format!("{:?}", m.role).to_lowercase(),
                    content: m.content.clone(),
                })
                .collect(),
            temperature: request.temperature,
            max_tokens: request.max_tokens,
            stream: true,
        };

        let response = self
            .client
            .post(self.chat_url())
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&groq_request)
            .send()
            .await
            .map_err(|e| AiError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(AiError::ApiError(format!(
                "Groq API error {}: {}",
                status, error_text
            )));
        }

        let stream = response.bytes_stream();
        let mapped_stream = stream.filter_map(|result| async move {
            match result {
                Ok(bytes) => {
                    let text = String::from_utf8_lossy(&bytes).to_string();
                    // Parse SSE format: "data: {...}\n\n"
                    for line in text.lines() {
                        if let Some(json_str) = line.strip_prefix("data: ") {
                            if json_str == "[DONE]" {
                                return None;
                            }
                            if let Ok(chunk) = serde_json::from_str::<GroqStreamChunk>(json_str) {
                                if let Some(choice) = chunk.choices.first() {
                                    if let Some(content) = &choice.delta.content {
                                        return Some(Ok(content.clone()));
                                    }
                                }
                            }
                        }
                    }
                    None
                }
                Err(e) => Some(Err(AiError::StreamError(e.to_string()))),
            }
        });

        Ok(Box::pin(mapped_stream))
    }

    async fn list_models(&self) -> Result<Vec<String>, AiError> {
        let response = self
            .client
            .get(self.models_url())
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .map_err(|e| AiError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(AiError::ApiError(format!(
                "Failed to list models: {}",
                response.status()
            )));
        }

        let models_response: GroqModelsResponse = response
            .json()
            .await
            .map_err(|e| AiError::ApiError(e.to_string()))?;

        Ok(models_response.data.into_iter().map(|m| m.id).collect())
    }
}

#[derive(Debug, Serialize)]
struct GroqChatRequest {
    model: String,
    messages: Vec<GroqMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    stream: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct GroqMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct GroqChatResponse {
    id: String,
    model: String,
    choices: Vec<GroqChoice>,
    usage: Option<GroqUsage>,
}

#[derive(Debug, Deserialize)]
struct GroqChoice {
    message: GroqMessage,
    finish_reason: String,
}

#[derive(Debug, Deserialize)]
struct GroqUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct GroqStreamChunk {
    choices: Vec<GroqStreamChoice>,
}

#[derive(Debug, Deserialize)]
struct GroqStreamChoice {
    delta: GroqDelta,
}

#[derive(Debug, Deserialize)]
struct GroqDelta {
    content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GroqModelsResponse {
    data: Vec<GroqModel>,
}

#[derive(Debug, Deserialize)]
struct GroqModel {
    id: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_groq_provider_creation() {
        let provider = GroqProvider::new(
            "gsk_test_key".to_string(),
            "llama3-70b-8192".to_string(),
        );
        assert_eq!(provider.name(), "groq");
        assert_eq!(provider.default_model, "llama3-70b-8192");
    }

    #[test]
    fn test_groq_urls() {
        let provider = GroqProvider::new(
            "gsk_test_key".to_string(),
            "llama3-70b-8192".to_string(),
        );
        assert_eq!(
            provider.chat_url(),
            "https://api.groq.com/openai/v1/chat/completions"
        );
        assert_eq!(
            provider.models_url(),
            "https://api.groq.com/openai/v1/models"
        );
    }
}
