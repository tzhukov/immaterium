// Integration tests for AI providers
// These tests require:
// 1. Ollama running on localhost:11434
// 2. Valid OpenAI API key (set OPENAI_API_KEY env var)
// 3. Valid Groq API key (set GROQ_API_KEY env var)

use immaterium::ai::{
    engine::AiEngine,
    provider::{ChatRequest, LlmProvider},
    providers::{GroqProvider, OllamaProvider, OpenAiProvider},
};
use std::sync::Arc;

#[tokio::test]
#[ignore] // Run with: cargo test --test ai_providers_integration -- --ignored
async fn test_ollama_provider_integration() {
    let provider = Arc::new(OllamaProvider::new(
        "http://localhost:11434".to_string(),
        "qwen2.5-coder:3b".to_string(), // Using available model
    ));

    println!("Testing Ollama provider...");

    // Test availability
    let available = provider.is_available().await;
    println!("Ollama available: {}", available);
    assert!(available, "Ollama should be running on localhost:11434");

    // Test model listing
    match provider.list_models().await {
        Ok(models) => {
            println!("Available Ollama models: {:?}", models);
            assert!(!models.is_empty(), "Should have at least one model");
        }
        Err(e) => {
            panic!("Failed to list Ollama models: {:?}", e);
        }
    }

    // Test chat completion
    let request = ChatRequest::new("qwen2.5-coder:3b".to_string())
        .with_user_message("Say 'Hello from Ollama!' and nothing else.".to_string())
        .with_temperature(0.1);

    match provider.chat_completion(request).await {
        Ok(response) => {
            println!("Ollama response: {}", response.content);
            assert!(!response.content.is_empty());
        }
        Err(e) => {
            panic!("Failed to get chat completion: {:?}", e);
        }
    }

    println!("✓ Ollama provider tests passed!");
}

#[tokio::test]
#[ignore]
async fn test_ollama_streaming() {
    let provider = Arc::new(OllamaProvider::new(
        "http://localhost:11434".to_string(),
        "qwen2.5-coder:3b".to_string(),
    ));

    println!("Testing Ollama streaming...");

    let request = ChatRequest::new("qwen2.5-coder:3b".to_string())
        .with_user_message("Count from 1 to 5.".to_string())
        .with_temperature(0.1);

    match provider.chat_completion_stream(request).await {
        Ok(mut stream) => {
            use futures::StreamExt;
            let mut chunks = Vec::new();
            while let Some(result) = stream.next().await {
                match result {
                    Ok(chunk) => {
                        print!("{}", chunk);
                        chunks.push(chunk);
                    }
                    Err(e) => {
                        panic!("Stream error: {:?}", e);
                    }
                }
            }
            println!();
            assert!(!chunks.is_empty(), "Should have received at least one chunk");
            println!("✓ Received {} chunks", chunks.len());
        }
        Err(e) => {
            panic!("Failed to start stream: {:?}", e);
        }
    }

    println!("✓ Ollama streaming tests passed!");
}

#[tokio::test]
#[ignore]
async fn test_openai_provider_integration() {
    let api_key = std::env::var("OPENAI_API_KEY")
        .expect("OPENAI_API_KEY environment variable must be set");

    let provider = Arc::new(OpenAiProvider::new(api_key, "gpt-3.5-turbo".to_string()));

    println!("Testing OpenAI provider...");

    // Test availability
    let available = provider.is_available().await;
    println!("OpenAI available: {}", available);
    assert!(available, "OpenAI should be available with valid API key");

    // Test model listing
    match provider.list_models().await {
        Ok(models) => {
            println!("Available OpenAI models: {:?}", models);
            assert!(!models.is_empty(), "Should have at least one GPT model");
        }
        Err(e) => {
            panic!("Failed to list OpenAI models: {:?}", e);
        }
    }

    // Test chat completion
    let request = ChatRequest::new("gpt-3.5-turbo".to_string())
        .with_user_message("Say 'Hello from OpenAI!' and nothing else.".to_string())
        .with_temperature(0.1);

    match provider.chat_completion(request).await {
        Ok(response) => {
            println!("OpenAI response: {}", response.content);
            assert!(!response.content.is_empty());
            if let Some(usage) = response.usage {
                println!(
                    "Tokens used: prompt={}, completion={}, total={}",
                    usage.prompt_tokens, usage.completion_tokens, usage.total_tokens
                );
            }
        }
        Err(e) => {
            panic!("Failed to get chat completion: {:?}", e);
        }
    }

    println!("✓ OpenAI provider tests passed!");
}

#[tokio::test]
#[ignore]
async fn test_groq_provider_integration() {
    let api_key =
        std::env::var("GROQ_API_KEY").expect("GROQ_API_KEY environment variable must be set");

    let provider = Arc::new(GroqProvider::new(
        api_key,
        "llama3-70b-8192".to_string(),
    ));

    println!("Testing Groq provider...");

    // Test availability
    let available = provider.is_available().await;
    println!("Groq available: {}", available);
    assert!(available, "Groq should be available with valid API key");

    // Test model listing
    match provider.list_models().await {
        Ok(models) => {
            println!("Available Groq models: {:?}", models);
            assert!(!models.is_empty(), "Should have at least one model");
        }
        Err(e) => {
            panic!("Failed to list Groq models: {:?}", e);
        }
    }

    // Test chat completion
    let request = ChatRequest::new("llama3-70b-8192".to_string())
        .with_user_message("Say 'Hello from Groq!' and nothing else.".to_string())
        .with_temperature(0.1);

    match provider.chat_completion(request).await {
        Ok(response) => {
            println!("Groq response: {}", response.content);
            assert!(!response.content.is_empty());
        }
        Err(e) => {
            panic!("Failed to get chat completion: {:?}", e);
        }
    }

    println!("✓ Groq provider tests passed!");
}

#[tokio::test]
#[ignore]
async fn test_multi_provider_engine() {
    println!("Testing multi-provider engine...");

    let mut engine = AiEngine::new();

    // Add Ollama provider
    let ollama = Arc::new(OllamaProvider::new(
        "http://localhost:11434".to_string(),
        "qwen2.5-coder:3b".to_string(),
    ));
    engine.register_provider(ollama.clone());
    engine.set_default_provider("ollama");

    // Test with Ollama
    let request = ChatRequest::new("qwen2.5-coder:3b".to_string())
        .with_user_message("What is 2+2? Answer with just the number.".to_string())
        .with_temperature(0.1);

    match engine.chat_completion(request).await {
        Ok(response) => {
            println!("Engine (Ollama) response: {}", response.content);
            assert!(!response.content.is_empty());
        }
        Err(e) => {
            panic!("Failed to get completion from engine: {:?}", e);
        }
    }

    // If OpenAI key is available, test provider switching
    if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
        let openai = Arc::new(OpenAiProvider::new(api_key, "gpt-3.5-turbo".to_string()));
        engine.register_provider(openai);

        let request = ChatRequest::new("gpt-3.5-turbo".to_string())
            .with_user_message("What is 3+3? Answer with just the number.".to_string())
            .with_temperature(0.1);

        match engine.chat_completion_with_provider("openai", request).await {
            Ok(response) => {
                println!("Engine (OpenAI) response: {}", response.content);
                assert!(!response.content.is_empty());
            }
            Err(e) => {
                panic!("Failed to get completion from OpenAI: {:?}", e);
            }
        }
    }

    println!("✓ Multi-provider engine tests passed!");
}

#[tokio::test]
#[ignore]
async fn test_conversation_with_context() {
    println!("Testing conversation with context...");

    let provider = Arc::new(OllamaProvider::new(
        "http://localhost:11434".to_string(),
        "qwen2.5-coder:3b".to_string(),
    ));

    // First message
    let request1 = ChatRequest::new("qwen2.5-coder:3b".to_string())
        .with_system_message("You are a helpful assistant. Be concise.".to_string())
        .with_user_message("My name is Alice.".to_string())
        .with_temperature(0.1);

    let response1 = provider
        .chat_completion(request1)
        .await
        .expect("First message should succeed");
    println!("Response 1: {}", response1.content);

    // Second message with context
    let request2 = ChatRequest::new("qwen2.5-coder:3b".to_string())
        .with_system_message("You are a helpful assistant. Be concise.".to_string())
        .with_user_message("My name is Alice.".to_string())
        .with_assistant_message(response1.content)
        .with_user_message("What is my name?".to_string())
        .with_temperature(0.1);

    let response2 = provider
        .chat_completion(request2)
        .await
        .expect("Second message should succeed");
    println!("Response 2: {}", response2.content);

    // The model should remember the name
    let response_lower = response2.content.to_lowercase();
    assert!(
        response_lower.contains("alice"),
        "Model should remember the name Alice"
    );

    println!("✓ Conversation context tests passed!");
}
