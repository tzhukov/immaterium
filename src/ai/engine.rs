use super::provider::{AiError, ChatRequest, ChatResponse, LlmProvider, StreamResponse};
use std::collections::HashMap;
use std::sync::Arc;

pub struct AiEngine {
    providers: HashMap<String, Arc<dyn LlmProvider>>,
    default_provider: Option<String>,
}

impl AiEngine {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
            default_provider: None,
        }
    }

    /// Register a provider
    pub fn register_provider(&mut self, provider: Arc<dyn LlmProvider>) {
        let name = provider.name().to_string();
        tracing::info!("Registering AI provider: {}", name);
        
        // Set as default if it's the first provider
        if self.default_provider.is_none() {
            self.default_provider = Some(name.clone());
        }
        
        self.providers.insert(name, provider);
    }

    /// Set the default provider
    pub fn set_default_provider(&mut self, name: &str) -> Result<(), AiError> {
        if !self.providers.contains_key(name) {
            return Err(AiError::NotConfigured(format!(
                "Provider '{}' not registered",
                name
            )));
        }
        self.default_provider = Some(name.to_string());
        Ok(())
    }

    /// Get the default provider
    pub fn get_default_provider(&self) -> Option<&Arc<dyn LlmProvider>> {
        self.default_provider
            .as_ref()
            .and_then(|name| self.providers.get(name))
    }

    /// Get a specific provider by name
    pub fn get_provider(&self, name: &str) -> Option<&Arc<dyn LlmProvider>> {
        self.providers.get(name)
    }

    /// List all registered provider names
    pub fn list_providers(&self) -> Vec<String> {
        self.providers.keys().cloned().collect()
    }

    /// Check if any provider is available
    pub async fn has_available_provider(&self) -> bool {
        for provider in self.providers.values() {
            if provider.is_available().await {
                return true;
            }
        }
        false
    }

    /// Send a chat completion request using the default provider
    pub async fn chat_completion(&self, request: ChatRequest) -> Result<ChatResponse, AiError> {
        let provider = self.get_default_provider().ok_or_else(|| {
            AiError::NotConfigured("No default provider set".to_string())
        })?;

        if !provider.is_available().await {
            return Err(AiError::NotConfigured(format!(
                "Provider '{}' is not available",
                provider.name()
            )));
        }

        provider.chat_completion(request).await
    }

    /// Send a streaming chat completion request using the default provider
    pub async fn chat_completion_stream(
        &self,
        request: ChatRequest,
    ) -> Result<StreamResponse, AiError> {
        let provider = self.get_default_provider().ok_or_else(|| {
            AiError::NotConfigured("No default provider set".to_string())
        })?;

        if !provider.is_available().await {
            return Err(AiError::NotConfigured(format!(
                "Provider '{}' is not available",
                provider.name()
            )));
        }

        provider.chat_completion_stream(request).await
    }

    /// Send a chat completion request using a specific provider
    pub async fn chat_completion_with_provider(
        &self,
        provider_name: &str,
        request: ChatRequest,
    ) -> Result<ChatResponse, AiError> {
        let provider = self.get_provider(provider_name).ok_or_else(|| {
            AiError::NotConfigured(format!("Provider '{}' not found", provider_name))
        })?;

        if !provider.is_available().await {
            return Err(AiError::NotConfigured(format!(
                "Provider '{}' is not available",
                provider_name
            )));
        }

        provider.chat_completion(request).await
    }
}

impl Default for AiEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;

    struct MockProvider {
        name: String,
        available: bool,
    }

    #[async_trait]
    impl LlmProvider for MockProvider {
        fn name(&self) -> &str {
            &self.name
        }

        async fn is_available(&self) -> bool {
            self.available
        }

        async fn chat_completion(&self, _request: ChatRequest) -> Result<ChatResponse, AiError> {
            Ok(ChatResponse {
                content: "Mock response".to_string(),
                model: "mock-model".to_string(),
                finish_reason: Some("stop".to_string()),
                usage: None,
            })
        }

        async fn chat_completion_stream(
            &self,
            _request: ChatRequest,
        ) -> Result<StreamResponse, AiError> {
            Err(AiError::Unknown("Not implemented".to_string()))
        }

        async fn list_models(&self) -> Result<Vec<String>, AiError> {
            Ok(vec!["mock-model".to_string()])
        }
    }

    #[test]
    fn test_engine_creation() {
        let engine = AiEngine::new();
        assert_eq!(engine.list_providers().len(), 0);
        assert!(engine.default_provider.is_none());
    }

    #[test]
    fn test_register_provider() {
        let mut engine = AiEngine::new();
        let provider = Arc::new(MockProvider {
            name: "test".to_string(),
            available: true,
        });

        engine.register_provider(provider);
        assert_eq!(engine.list_providers().len(), 1);
        assert_eq!(engine.default_provider, Some("test".to_string()));
    }

    #[tokio::test]
    async fn test_chat_completion() {
        let mut engine = AiEngine::new();
        let provider = Arc::new(MockProvider {
            name: "test".to_string(),
            available: true,
        });

        engine.register_provider(provider);

        let request = ChatRequest::new("mock-model".to_string())
            .with_user_message("Hello".to_string());

        let response = engine.chat_completion(request).await.unwrap();
        assert_eq!(response.content, "Mock response");
    }

    #[tokio::test]
    async fn test_unavailable_provider() {
        let mut engine = AiEngine::new();
        let provider = Arc::new(MockProvider {
            name: "test".to_string(),
            available: false,
        });

        engine.register_provider(provider);

        let request = ChatRequest::new("mock-model".to_string())
            .with_user_message("Hello".to_string());

        let result = engine.chat_completion(request).await;
        assert!(result.is_err());
    }
}

