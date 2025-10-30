use crate::ai::provider::{AiError, ChatRequest, ChatResponse, LlmProvider, MessageRole, StreamResponse, Usage};
use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestUserMessageArgs, ChatCompletionRequestAssistantMessageArgs,
        CreateChatCompletionRequestArgs, CreateChatCompletionResponse,
    },
    Client,
};
use async_trait::async_trait;
use futures::StreamExt;

pub struct OpenAiProvider {
    client: Client<OpenAIConfig>,
    default_model: String,
}

impl OpenAiProvider {
    pub fn new(api_key: String, default_model: String) -> Self {
        let config = OpenAIConfig::new().with_api_key(api_key);
        let client = Client::with_config(config);
        
        Self {
            client,
            default_model,
        }
    }

    fn convert_messages(
        &self,
        messages: &[crate::ai::provider::Message],
    ) -> Vec<ChatCompletionRequestMessage> {
        messages
            .iter()
            .map(|m| match m.role {
                crate::ai::provider::MessageRole::System => {
                    ChatCompletionRequestMessage::System(
                        ChatCompletionRequestSystemMessageArgs::default()
                            .content(m.content.clone())
                            .build()
                            .unwrap(),
                    )
                }
                crate::ai::provider::MessageRole::User => {
                    ChatCompletionRequestMessage::User(
                        ChatCompletionRequestUserMessageArgs::default()
                            .content(m.content.clone())
                            .build()
                            .unwrap(),
                    )
                }
                crate::ai::provider::MessageRole::Assistant => {
                    ChatCompletionRequestMessage::Assistant(
                        ChatCompletionRequestAssistantMessageArgs::default()
                            .content(m.content.clone())
                            .build()
                            .unwrap(),
                    )
                }
            })
            .collect()
    }
}

#[async_trait]
impl LlmProvider for OpenAiProvider {
    fn name(&self) -> &str {
        "openai"
    }

    async fn is_available(&self) -> bool {
        // Try to list models to check if API key is valid
        self.client.models().list().await.is_ok()
    }

    async fn chat_completion(&self, request: ChatRequest) -> Result<ChatResponse, AiError> {
        let messages = self.convert_messages(&request.messages);

        let mut req = CreateChatCompletionRequestArgs::default();
        req.model(&request.model).messages(messages);
        
        if let Some(temp) = request.temperature {
            req.temperature(temp);
        }

        if let Some(max_tokens) = request.max_tokens {
            req.max_tokens(max_tokens as u16);
        }

        let chat_request = req
            .build()
            .map_err(|e| AiError::InvalidRequest(e.to_string()))?;

        let response = self
            .client
            .chat()
            .create(chat_request)
            .await
            .map_err(|e| {
                let err_str = e.to_string();
                if err_str.contains("rate") || err_str.contains("429") {
                    AiError::RateLimitExceeded
                } else if err_str.contains("model") {
                    AiError::ModelNotFound(err_str)
                } else {
                    AiError::ApiError(err_str)
                }
            })?;

        self.parse_response(response)
    }

    async fn chat_completion_stream(&self, request: ChatRequest) -> Result<StreamResponse, AiError> {
        let messages = self.convert_messages(&request.messages);

        let mut req = CreateChatCompletionRequestArgs::default();
        req.model(&request.model).messages(messages);
        
        if let Some(temp) = request.temperature {
            req.temperature(temp);
        }

        if let Some(max_tokens) = request.max_tokens {
            req.max_tokens(max_tokens as u16);
        }

        let chat_request = req
            .build()
            .map_err(|e| AiError::InvalidRequest(e.to_string()))?;

        let mut stream = self
            .client
            .chat()
            .create_stream(chat_request)
            .await
            .map_err(|e| AiError::ApiError(e.to_string()))?;

        let mapped_stream = async_stream::stream! {
            while let Some(result) = stream.next().await {
                match result {
                    Ok(response) => {
                        if let Some(choice) = response.choices.first() {
                            if let Some(content) = &choice.delta.content {
                                yield Ok(content.clone());
                            }
                        }
                    }
                    Err(e) => {
                        yield Err(AiError::StreamError(e.to_string()));
                        break;
                    }
                }
            }
        };

        Ok(Box::pin(mapped_stream))
    }

    async fn list_models(&self) -> Result<Vec<String>, AiError> {
        let models = self
            .client
            .models()
            .list()
            .await
            .map_err(|e| AiError::ApiError(e.to_string()))?;

        Ok(models
            .data
            .into_iter()
            .map(|m| m.id)
            .filter(|id| id.starts_with("gpt"))
            .collect())
    }
}

impl OpenAiProvider {
    fn parse_response(&self, response: CreateChatCompletionResponse) -> Result<ChatResponse, AiError> {
        let choice = response
            .choices
            .first()
            .ok_or_else(|| AiError::ApiError("No choices in response".to_string()))?;

        let content = choice
            .message
            .content
            .clone()
            .ok_or_else(|| AiError::ApiError("No content in message".to_string()))?;

        Ok(ChatResponse {
            content,
            model: response.model,
            finish_reason: choice.finish_reason.as_ref().map(|r| format!("{:?}", r)),
            usage: response.usage.map(|u| Usage {
                prompt_tokens: u.prompt_tokens,
                completion_tokens: u.completion_tokens,
                total_tokens: u.total_tokens,
            }),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openai_provider_creation() {
        let provider = OpenAiProvider::new(
            "sk-test-key".to_string(),
            "gpt-4".to_string(),
        );
        assert_eq!(provider.name(), "openai");
        assert_eq!(provider.default_model, "gpt-4");
    }
}
