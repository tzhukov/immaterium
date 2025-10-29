// Placeholder for LLM provider trait and implementations
// Will be implemented in Milestone 6

use anyhow::Result;

#[async_trait::async_trait]
pub trait LlmProvider: Send + Sync {
    async fn complete(&self, prompt: &str) -> Result<String>;
    fn name(&self) -> &str;
}
