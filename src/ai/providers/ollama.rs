use crate::ai::provider::{AiError, ChatRequest, ChatResponse, LlmProvider, Message, StreamResponse, Usage};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio_stream::StreamExt;

pub struct OllamaProvider {
    client: Client,
    base_url: String,
    default_model: String,
}

impl OllamaProvider {
    pub fn new(base_url: String, default_model: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
            default_model,
        }
    }

    fn chat_url(&self) -> String {
        format!("{}/api/chat", self.base_url)
    }

    fn models_url(&self) -> String {
        format!("{}/api/tags", self.base_url)
    }
}

#[async_trait]
impl LlmProvider for OllamaProvider {
    fn name(&self) -> &str {
        "ollama"
    }

    async fn is_available(&self) -> bool {
        // Try to connect to Ollama
        self.client
            .get(&self.models_url())
            .send()
            .await
            .is_ok()
    }

    async fn chat_completion(&self, request: ChatRequest) -> Result<ChatResponse, AiError> {
        let ollama_request = OllamaChatRequest {
            model: request.model,
            messages: request
                .messages
                .iter()
                .map(|m| OllamaMessage {
                    role: format!("{:?}", m.role).to_lowercase(),
                    content: m.content.clone(),
                })
                .collect(),
            stream: false,
            options: Some(OllamaOptions {
                temperature: request.temperature,
                num_predict: request.max_tokens.map(|t| t as i32),
            }),
        };

        let response = self
            .client
            .post(&self.chat_url())
            .json(&ollama_request)
            .send()
            .await
            .map_err(|e| AiError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(AiError::ApiError(format!(
                "Ollama API error {}: {}",
                status, error_text
            )));
        }

        let ollama_response: OllamaChatResponse = response
            .json()
            .await
            .map_err(|e| AiError::ApiError(e.to_string()))?;

        Ok(ChatResponse {
            content: ollama_response.message.content,
            model: ollama_response.model,
            finish_reason: Some(ollama_response.done_reason.unwrap_or_else(|| "stop".to_string())),
            usage: ollama_response.prompt_eval_count.map(|prompt_tokens| Usage {
                prompt_tokens: prompt_tokens as u32,
                completion_tokens: ollama_response.eval_count.unwrap_or(0) as u32,
                total_tokens: (prompt_tokens + ollama_response.eval_count.unwrap_or(0)) as u32,
            }),
        })
    }

    async fn chat_completion_stream(&self, request: ChatRequest) -> Result<StreamResponse, AiError> {
        let ollama_request = OllamaChatRequest {
            model: request.model,
            messages: request
                .messages
                .iter()
                .map(|m| OllamaMessage {
                    role: format!("{:?}", m.role).to_lowercase(),
                    content: m.content.clone(),
                })
                .collect(),
            stream: true,
            options: Some(OllamaOptions {
                temperature: request.temperature,
                num_predict: request.max_tokens.map(|t| t as i32),
            }),
        };

        let response = self
            .client
            .post(&self.chat_url())
            .json(&ollama_request)
            .send()
            .await
            .map_err(|e| AiError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(AiError::ApiError(format!(
                "Ollama API error {}: {}",
                status, error_text
            )));
        }

        let stream = response.bytes_stream();
        let mapped_stream = stream.map(|result| {
            result
                .map_err(|e| AiError::StreamError(e.to_string()))
                .and_then(|bytes| {
                    let text = String::from_utf8_lossy(&bytes).to_string();
                    serde_json::from_str::<OllamaChatResponse>(&text)
                        .map(|resp| resp.message.content)
                        .map_err(|e| AiError::StreamError(e.to_string()))
                })
        });

        Ok(Box::pin(mapped_stream))
    }

    async fn list_models(&self) -> Result<Vec<String>, AiError> {
        let response = self
            .client
            .get(&self.models_url())
            .send()
            .await
            .map_err(|e| AiError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(AiError::ApiError(format!(
                "Failed to list models: {}",
                response.status()
            )));
        }

        let models_response: OllamaModelsResponse = response
            .json()
            .await
            .map_err(|e| AiError::ApiError(e.to_string()))?;

        Ok(models_response.models.into_iter().map(|m| m.name).collect())
    }
}

#[derive(Debug, Serialize)]
struct OllamaChatRequest {
    model: String,
    messages: Vec<OllamaMessage>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<OllamaOptions>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OllamaMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct OllamaOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    num_predict: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct OllamaChatResponse {
    model: String,
    message: OllamaMessage,
    #[serde(default)]
    done_reason: Option<String>,
    #[serde(default)]
    prompt_eval_count: Option<i64>,
    #[serde(default)]
    eval_count: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct OllamaModelsResponse {
    models: Vec<OllamaModel>,
}

#[derive(Debug, Deserialize)]
struct OllamaModel {
    name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ollama_provider_creation() {
        let provider = OllamaProvider::new(
            "http://localhost:11434".to_string(),
            "llama2".to_string(),
        );
        assert_eq!(provider.name(), "ollama");
        assert_eq!(provider.base_url, "http://localhost:11434");
    }

    #[test]
    fn test_ollama_urls() {
        let provider = OllamaProvider::new(
            "http://localhost:11434".to_string(),
            "llama2".to_string(),
        );
        assert_eq!(provider.chat_url(), "http://localhost:11434/api/chat");
        assert_eq!(provider.models_url(), "http://localhost:11434/api/tags");
    }
}
