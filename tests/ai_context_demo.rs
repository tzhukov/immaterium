// Integration test demonstrating context building with AI
use immaterium::ai::{
    build_minimal_context, build_session_context, ContextBuilder, ContextConfig,
};
use immaterium::ai::{providers::OllamaProvider, ChatRequest, LlmProvider};
use immaterium::core::block::{Block, BlockMetadata, BlockState};
use immaterium::core::session::Session;
use chrono::Utc;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use uuid::Uuid;

fn create_sample_blocks() -> Vec<Block> {
    vec![
        Block {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            command: "ls -la".to_string(),
            output: "total 24\ndrwxr-xr-x  3 user user 4096 Oct 29 10:30 .\ndrwxr-xr-x 10 user user 4096 Oct 29 09:15 ..\n-rw-r--r--  1 user user  220 Oct 29 09:15 .bashrc\n-rw-r--r--  1 user user  807 Oct 29 09:15 .profile".to_string(),
            exit_code: Some(0),
            state: BlockState::Completed,
            metadata: BlockMetadata {
                duration: None,
                working_directory: PathBuf::from("/home/user"),
                environment: HashMap::new(),
                started_at: Some(Utc::now()),
                completed_at: Some(Utc::now()),
            },
            is_collapsed: false,
            is_selected: false,
        },
        Block {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            command: "cat .bashrc".to_string(),
            output: "# ~/.bashrc: executed by bash(1) for non-login shells.\n\n# If not running interactively, don't do anything\ncase $- in\n    *i*) ;;\n      *) return;;\nesac\n\n# enable color support\nif [ -x /usr/bin/dircolors ]; then\n    test -r ~/.dircolors && eval \"$(dircolors -b ~/.dircolors)\" || eval \"$(dircolors -b)\"\n    alias ls='ls --color=auto'\nfi".to_string(),
            exit_code: Some(0),
            state: BlockState::Completed,
            metadata: BlockMetadata {
                duration: None,
                working_directory: PathBuf::from("/home/user"),
                environment: HashMap::new(),
                started_at: Some(Utc::now()),
                completed_at: Some(Utc::now()),
            },
            is_collapsed: false,
            is_selected: false,
        },
        Block {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            command: "grep -n 'alias' .bashrc".to_string(),
            output: "11:    alias ls='ls --color=auto'\n".to_string(),
            exit_code: Some(0),
            state: BlockState::Completed,
            metadata: BlockMetadata {
                duration: None,
                working_directory: PathBuf::from("/home/user"),
                environment: HashMap::new(),
                started_at: Some(Utc::now()),
                completed_at: Some(Utc::now()),
            },
            is_collapsed: false,
            is_selected: false,
        },
    ]
}

#[test]
fn test_context_builder_demo() {
    let blocks = create_sample_blocks();

    // Create context with different configs
    let config = ContextConfig {
        max_tokens: 2000,
        truncate_output: true,
        max_output_chars: 300,
        recent_blocks_count: 5,
        include_system_info: true,
        ..Default::default()
    };

    let mut builder = ContextBuilder::new(config);
    builder
        .add_system_info()
        .add_blocks(&blocks)
        .add_prompt("What alias is configured in the bashrc file?");

    let token_count = builder.token_count();
    let context = builder.build();

    println!("=== Context Generated ===");
    println!("{}", context);
    println!("\n=== Stats ===");
    println!("Estimated tokens: {}", token_count);
    println!("Blocks included: {}", blocks.len());

    assert!(context.contains("System Information"));
    assert!(context.contains("$ ls -la"));
    assert!(context.contains("$ grep -n 'alias'"));
    assert!(context.contains("What alias is configured"));
}

#[test]
fn test_minimal_context_demo() {
    let blocks = create_sample_blocks();

    let context = build_minimal_context(&blocks, "Explain the last command", 3);

    println!("=== Minimal Context ===");
    println!("{}", context);

    assert!(context.contains("$ grep"));
    assert!(!context.contains("System Information"));
}

#[test]
fn test_session_context_demo() {
    let session = Session::new(
        "debugging-session".to_string(),
        PathBuf::from("/home/user"),
    );
    let blocks = create_sample_blocks();

    let context = build_session_context(
        &session,
        &blocks,
        "Summarize what was done in this session",
        ContextConfig::default(),
    );

    println!("=== Session Context ===");
    println!("{}", context);

    assert!(context.contains("debugging-session"));
    assert!(context.contains("Session"));
}

#[tokio::test]
#[ignore] // Run with: cargo test --test ai_context_demo -- --ignored --nocapture
async fn test_ai_with_context() {
    let provider = Arc::new(OllamaProvider::new(
        "http://localhost:11434".to_string(),
        "qwen2.5-coder:3b".to_string(),
    ));

    if !provider.is_available().await {
        println!("Skipping: Ollama not available");
        return;
    }

    let blocks = create_sample_blocks();

    // Build context
    let context = build_minimal_context(&blocks, "What alias is set in the bashrc file?", 3);

    println!("=== Sending Context to AI ===");
    println!("{}\n", context);

    // Create request with context as system message
    let request = ChatRequest::new("qwen2.5-coder:3b".to_string())
        .with_system_message("You are a helpful terminal assistant. Based on the command history provided, answer the user's question concisely.".to_string())
        .with_user_message(context)
        .with_temperature(0.1);

    match provider.chat_completion(request).await {
        Ok(response) => {
            println!("=== AI Response ===");
            println!("{}", response.content);
            println!("\n✓ AI successfully analyzed context");

            // The response should mention the ls alias
            let response_lower = response.content.to_lowercase();
            assert!(
                response_lower.contains("ls") || response_lower.contains("alias"),
                "Response should mention the ls alias"
            );
        }
        Err(e) => {
            panic!("Failed to get AI response: {:?}", e);
        }
    }
}

#[tokio::test]
#[ignore]
async fn test_token_budget_management() {
    let provider = Arc::new(OllamaProvider::new(
        "http://localhost:11434".to_string(),
        "qwen2.5-coder:3b".to_string(),
    ));

    if !provider.is_available().await {
        println!("Skipping: Ollama not available");
        return;
    }

    // Create many blocks
    let mut blocks = Vec::new();
    for i in 0..20 {
        blocks.push(Block {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            command: format!("echo 'Command number {}'", i),
            output: format!("Command number {}\n", i),
            exit_code: Some(0),
            state: BlockState::Completed,
            metadata: BlockMetadata {
                duration: None,
                working_directory: PathBuf::from("/home/user"),
                environment: HashMap::new(),
                started_at: Some(Utc::now()),
                completed_at: Some(Utc::now()),
            },
            is_collapsed: false,
            is_selected: false,
        });
    }

    // Build with limited token budget
    let config = ContextConfig {
        max_tokens: 500, // Very limited
        recent_blocks_count: 5,
        truncate_output: true,
        max_output_chars: 50,
        include_system_info: false,
        ..Default::default()
    };

    let mut builder = ContextBuilder::new(config);
    builder
        .add_blocks(&blocks)
        .add_prompt("How many commands were run?");

    let token_count = builder.token_count();
    let context = builder.build();

    println!("=== Token Budget Test ===");
    println!("Total blocks: {}", blocks.len());
    println!("Estimated tokens: {}", token_count);
    println!("Max tokens: 500");
    println!("\nContext:\n{}", context);

    assert!(token_count <= 500, "Should respect token budget");

    // Send to AI
    let request = ChatRequest::new("qwen2.5-coder:3b".to_string())
        .with_system_message(
            "Answer based only on the command history provided.".to_string(),
        )
        .with_user_message(context)
        .with_temperature(0.1);

    match provider.chat_completion(request).await {
        Ok(response) => {
            println!("\n=== AI Response ===");
            println!("{}", response.content);
            println!("\n✓ AI handled limited context successfully");
        }
        Err(e) => {
            panic!("Failed: {:?}", e);
        }
    }
}
