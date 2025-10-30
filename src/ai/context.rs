use crate::core::block::{Block, BlockState};
use crate::core::session::Session;
use serde::{Deserialize, Serialize};

/// Configuration for context building
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextConfig {
    /// Maximum tokens to use for context
    pub max_tokens: usize,
    /// Approximate tokens per character (rough estimate)
    pub tokens_per_char: f32,
    /// Whether to include system information
    pub include_system_info: bool,
    /// Whether to include full output or truncate
    pub truncate_output: bool,
    /// Max characters per output if truncating
    pub max_output_chars: usize,
    /// How many recent blocks to prioritize
    pub recent_blocks_count: usize,
}

impl Default for ContextConfig {
    fn default() -> Self {
        Self {
            max_tokens: 4000,           // Conservative default
            tokens_per_char: 0.25,      // Rough estimate: 4 chars per token
            include_system_info: true,
            truncate_output: true,
            max_output_chars: 500,
            recent_blocks_count: 10,
        }
    }
}

impl ContextConfig {
    pub fn new(max_tokens: usize) -> Self {
        Self {
            max_tokens,
            ..Default::default()
        }
    }

    /// Estimate token count from text
    pub fn estimate_tokens(&self, text: &str) -> usize {
        (text.len() as f32 * self.tokens_per_char).ceil() as usize
    }

    /// Check if text fits within token budget
    pub fn fits_in_budget(&self, text: &str, used_tokens: usize) -> bool {
        let estimated = self.estimate_tokens(text);
        used_tokens + estimated <= self.max_tokens
    }
}

/// Builder for constructing LLM context from session data
pub struct ContextBuilder {
    config: ContextConfig,
    parts: Vec<String>,
    token_count: usize,
}

impl ContextBuilder {
    pub fn new(config: ContextConfig) -> Self {
        Self {
            config,
            parts: Vec::new(),
            token_count: 0,
        }
    }

    /// Add a section to the context if it fits
    fn try_add_section(&mut self, section: String) -> bool {
        let tokens = self.config.estimate_tokens(&section);
        if self.token_count + tokens <= self.config.max_tokens {
            self.token_count += tokens;
            self.parts.push(section);
            true
        } else {
            false
        }
    }

    /// Add system information
    pub fn add_system_info(&mut self) -> &mut Self {
        if !self.config.include_system_info {
            return self;
        }

        let info = format!(
            "=== System Information ===\nOS: {}\nArchitecture: {}\nShell: bash\n",
            std::env::consts::OS,
            std::env::consts::ARCH
        );

        self.try_add_section(info);
        self
    }

    /// Add session information
    pub fn add_session_info(&mut self, session: &Session) -> &mut Self {
        let info = format!(
            "=== Session ===\nName: {}\nWorking Directory: {}\n",
            session.name,
            session.working_directory.display()
        );

        self.try_add_section(info);
        self
    }

    /// Add a single block to context
    pub fn add_block(&mut self, block: &Block) -> bool {
        let mut block_text = String::new();

        // Add command
        block_text.push_str(&format!("$ {}\n", block.command));

        // Add output
        if !block.output.is_empty() {
            let output = if self.config.truncate_output
                && block.output.len() > self.config.max_output_chars
            {
                let truncated = &block.output[..self.config.max_output_chars];
                format!("{}...\n[Output truncated]\n", truncated)
            } else {
                format!("{}\n", block.output)
            };
            block_text.push_str(&output);
        }

        // Add status if not running
        match block.state {
            BlockState::PendingApproval => block_text.push_str("[Pending Approval]\n"),
            BlockState::Completed => {
                if let Some(code) = block.exit_code {
                    block_text.push_str(&format!("[Exit: {}]\n", code));
                }
            }
            BlockState::Failed => block_text.push_str("[Failed]\n"),
            BlockState::Running => block_text.push_str("[Running...]\n"),
            BlockState::Cancelled => block_text.push_str("[Cancelled]\n"),
            BlockState::Editing => block_text.push_str("[Editing]\n"),
        }

        self.try_add_section(block_text)
    }

    /// Add blocks with smart selection
    pub fn add_blocks(&mut self, blocks: &[Block]) -> &mut Self {
        if blocks.is_empty() {
            return self;
        }

        // Add header
        self.try_add_section("=== Command History ===\n".to_string());

        // Prioritize recent blocks
        let start_idx = if blocks.len() > self.config.recent_blocks_count {
            blocks.len() - self.config.recent_blocks_count
        } else {
            0
        };

        // Try to add blocks from most recent backwards
        for block in blocks[start_idx..].iter().rev() {
            if !self.add_block(block) {
                // If we can't fit more blocks, stop
                break;
            }
        }

        // Reverse the parts to get chronological order
        let history_start = self.parts.iter().position(|p| p.contains("Command History"));
        if let Some(start) = history_start {
            let history_parts = self.parts.drain(start + 1..).collect::<Vec<_>>();
            self.parts.extend(history_parts.into_iter().rev());
        }

        self
    }

    /// Add a custom section
    pub fn add_custom(&mut self, content: String) -> &mut Self {
        self.try_add_section(content);
        self
    }

    /// Add a prompt/question
    pub fn add_prompt(&mut self, prompt: &str) -> &mut Self {
        let section = format!("=== Question ===\n{}\n", prompt);
        self.try_add_section(section);
        self
    }

    /// Build the final context string
    pub fn build(self) -> String {
        self.parts.join("\n")
    }

    /// Get current token count
    pub fn token_count(&self) -> usize {
        self.token_count
    }

    /// Get remaining token budget
    pub fn remaining_tokens(&self) -> usize {
        self.config.max_tokens.saturating_sub(self.token_count)
    }
}

/// Helper to build context from session
pub fn build_session_context(
    session: &Session,
    blocks: &[Block],
    prompt: &str,
    config: ContextConfig,
) -> String {
    let mut builder = ContextBuilder::new(config);

    builder
        .add_system_info()
        .add_session_info(session)
        .add_blocks(blocks)
        .add_prompt(prompt);

    builder.build()
}

/// Helper to build minimal context (just recent commands)
pub fn build_minimal_context(blocks: &[Block], prompt: &str, max_blocks: usize) -> String {
    let config = ContextConfig {
        max_tokens: 2000,
        truncate_output: true,
        max_output_chars: 200,
        recent_blocks_count: max_blocks,
        include_system_info: false,
        ..Default::default()
    };

    let mut builder = ContextBuilder::new(config);
    builder.add_blocks(blocks).add_prompt(prompt);
    builder.build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::path::PathBuf;
    use uuid::Uuid;

    fn create_test_block(command: &str, output: &str, state: BlockState, exit_code: Option<i32>) -> Block {
        Block {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            command: command.to_string(),
            output: output.to_string(),
            exit_code,
            state,
            metadata: crate::core::block::BlockMetadata {
                duration: None,
                working_directory: PathBuf::from("/home/user"),
                environment: std::collections::HashMap::new(),
                started_at: Some(Utc::now()),
                completed_at: Some(Utc::now()),
            },
            is_collapsed: false,
            is_selected: false,
            original_input: None,
        }
    }

    fn create_test_session() -> Session {
        Session {
            id: Uuid::new_v4(),
            name: "test-session".to_string(),
            working_directory: PathBuf::from("/home/user"),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            blocks: Vec::new(),
            environment: std::collections::HashMap::new(),
        }
    }

    #[test]
    fn test_context_config_default() {
        let config = ContextConfig::default();
        assert_eq!(config.max_tokens, 4000);
        assert!(config.include_system_info);
        assert!(config.truncate_output);
    }

    #[test]
    fn test_token_estimation() {
        let config = ContextConfig::default();
        let text = "Hello, world!"; // 13 chars
        let tokens = config.estimate_tokens(text);
        // 13 * 0.25 = 3.25, rounds up to 4
        assert_eq!(tokens, 4);
    }

    #[test]
    fn test_context_builder_basic() {
        let config = ContextConfig::new(1000);
        let mut builder = ContextBuilder::new(config);

        let session = create_test_session();
        builder.add_session_info(&session);

        let context = builder.build();
        assert!(context.contains("test-session"));
        assert!(context.contains("/home/user"));
    }

    #[test]
    fn test_add_single_block() {
        let config = ContextConfig::new(1000);
        let mut builder = ContextBuilder::new(config);

        let block = create_test_block("ls -la", "total 8\ndrwxr-xr-x", BlockState::Completed, Some(0));
        assert!(builder.add_block(&block));

        let context = builder.build();
        assert!(context.contains("$ ls -la"));
        assert!(context.contains("total 8"));
        assert!(context.contains("[Exit: 0]"));
    }

    #[test]
    fn test_add_multiple_blocks() {
        let config = ContextConfig::new(2000);
        let mut builder = ContextBuilder::new(config);

        let blocks = vec![
            create_test_block("echo 'first'", "first", BlockState::Completed, Some(0)),
            create_test_block("echo 'second'", "second", BlockState::Completed, Some(0)),
            create_test_block("echo 'third'", "third", BlockState::Completed, Some(0)),
        ];

        builder.add_blocks(&blocks);
        let context = builder.build();

        assert!(context.contains("$ echo 'first'"));
        assert!(context.contains("$ echo 'second'"));
        assert!(context.contains("$ echo 'third'"));
    }

    #[test]
    fn test_output_truncation() {
        let config = ContextConfig {
            truncate_output: true,
            max_output_chars: 10,
            ..Default::default()
        };
        let mut builder = ContextBuilder::new(config);

        let long_output = "a".repeat(100);
        let block = create_test_block("cmd", &long_output, BlockState::Completed, Some(0));

        builder.add_block(&block);
        let context = builder.build();

        assert!(context.contains("aaaaaaaaaa..."));
        assert!(context.contains("[Output truncated]"));
    }

    #[test]
    fn test_token_budget() {
        let config = ContextConfig::new(100); // Very small budget
        let mut builder = ContextBuilder::new(config);

        let large_text = "x".repeat(1000); // 250 tokens
        let result = builder.try_add_section(large_text);

        assert!(!result); // Should fail to add
        assert_eq!(builder.token_count(), 0); // Should not have added anything
    }

    #[test]
    fn test_build_session_context() {
        let session = create_test_session();
        let blocks = vec![
            create_test_block("pwd", "/home/user", BlockState::Completed, Some(0)),
            create_test_block("ls", "file1.txt\nfile2.txt", BlockState::Completed, Some(0)),
        ];

        let context = build_session_context(
            &session,
            &blocks,
            "What files are in the current directory?",
            ContextConfig::default(),
        );

        assert!(context.contains("test-session"));
        assert!(context.contains("$ pwd"));
        assert!(context.contains("$ ls"));
        assert!(context.contains("What files are in the current directory?"));
    }

    #[test]
    fn test_build_minimal_context() {
        let blocks = vec![
            create_test_block("cmd1", "output1", BlockState::Completed, Some(0)),
            create_test_block("cmd2", "output2", BlockState::Completed, Some(0)),
        ];

        let context = build_minimal_context(&blocks, "Explain cmd2", 2);

        assert!(context.contains("$ cmd1"));
        assert!(context.contains("$ cmd2"));
        assert!(context.contains("Explain cmd2"));
        assert!(!context.contains("System Information")); // Minimal mode
    }

    #[test]
    fn test_recent_blocks_prioritization() {
        let config = ContextConfig {
            recent_blocks_count: 2,
            max_tokens: 5000,
            ..Default::default()
        };
        let mut builder = ContextBuilder::new(config);

        let blocks = vec![
            create_test_block("old1", "out1", BlockState::Completed, Some(0)),
            create_test_block("old2", "out2", BlockState::Completed, Some(0)),
            create_test_block("recent1", "out3", BlockState::Completed, Some(0)),
            create_test_block("recent2", "out4", BlockState::Completed, Some(0)),
        ];

        builder.add_blocks(&blocks);
        let context = builder.build();

        // Should only include the 2 most recent
        assert!(context.contains("$ recent1"));
        assert!(context.contains("$ recent2"));
    }

    #[test]
    fn test_remaining_tokens() {
        let config = ContextConfig::new(1000);
        let mut builder = ContextBuilder::new(config);

        assert_eq!(builder.remaining_tokens(), 1000);

        builder.add_custom("x".repeat(100)); // ~25 tokens
        assert!(builder.remaining_tokens() < 1000);
        assert!(builder.remaining_tokens() > 900);
    }
}
