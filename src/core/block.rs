use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub command: String,
    pub output: String,
    pub exit_code: Option<i32>,
    pub state: BlockState,
    pub metadata: BlockMetadata,
    pub is_collapsed: bool,
    pub is_selected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BlockState {
    Editing,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockMetadata {
    pub duration: Option<Duration>,
    pub working_directory: PathBuf,
    pub environment: HashMap<String, String>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl Block {
    pub fn new(command: String, working_directory: PathBuf) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            command,
            output: String::new(),
            exit_code: None,
            state: BlockState::Editing,
            metadata: BlockMetadata {
                duration: None,
                working_directory,
                environment: HashMap::new(),
                started_at: None,
                completed_at: None,
            },
            is_collapsed: false,
            is_selected: false,
        }
    }

    pub fn start_execution(&mut self) {
        self.state = BlockState::Running;
        self.metadata.started_at = Some(Utc::now());
    }

    pub fn complete_execution(&mut self, exit_code: i32) {
        self.exit_code = Some(exit_code);
        self.metadata.completed_at = Some(Utc::now());
        
        if exit_code == 0 {
            self.state = BlockState::Completed;
        } else {
            self.state = BlockState::Failed;
        }

        // Calculate duration
        if let (Some(start), Some(end)) = (self.metadata.started_at, self.metadata.completed_at) {
            if let Ok(duration) = (end - start).to_std() {
                self.metadata.duration = Some(duration);
            }
        }
    }

    pub fn append_output(&mut self, text: String) {
        self.output.push_str(&text);
    }

    pub fn toggle_collapsed(&mut self) {
        self.is_collapsed = !self.is_collapsed;
    }

    pub fn set_selected(&mut self, selected: bool) {
        self.is_selected = selected;
    }

    pub fn is_running(&self) -> bool {
        matches!(self.state, BlockState::Running)
    }

    pub fn is_completed(&self) -> bool {
        matches!(self.state, BlockState::Completed | BlockState::Failed | BlockState::Cancelled)
    }

    pub fn get_display_command(&self) -> String {
        if self.command.len() > 100 {
            format!("{}...", &self.command[..97])
        } else {
            self.command.clone()
        }
    }

    pub fn get_display_output(&self) -> String {
        if self.is_collapsed {
            String::new()
        } else {
            self.output.clone()
        }
    }

    pub fn format_duration(&self) -> String {
        match self.metadata.duration {
            Some(d) if d.as_secs() > 0 => format!("{}s", d.as_secs()),
            Some(d) => format!("{}ms", d.as_millis()),
            None => "".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_creation() {
        let block = Block::new("echo test".to_string(), PathBuf::from("/tmp"));
        assert_eq!(block.command, "echo test");
        assert_eq!(block.state, BlockState::Editing);
        assert!(!block.is_collapsed);
        assert!(!block.is_selected);
    }

    #[test]
    fn test_block_execution_lifecycle() {
        let mut block = Block::new("echo test".to_string(), PathBuf::from("/tmp"));
        
        block.start_execution();
        assert_eq!(block.state, BlockState::Running);
        assert!(block.metadata.started_at.is_some());
        
        std::thread::sleep(std::time::Duration::from_millis(10));
        
        block.complete_execution(0);
        assert_eq!(block.state, BlockState::Completed);
        assert_eq!(block.exit_code, Some(0));
        assert!(block.metadata.completed_at.is_some());
        assert!(block.metadata.duration.is_some());
    }

    #[test]
    fn test_block_failed_execution() {
        let mut block = Block::new("false".to_string(), PathBuf::from("/tmp"));
        block.start_execution();
        block.complete_execution(1);
        
        assert_eq!(block.state, BlockState::Failed);
        assert_eq!(block.exit_code, Some(1));
    }

    #[test]
    fn test_block_collapse() {
        let mut block = Block::new("echo test".to_string(), PathBuf::from("/tmp"));
        block.output = "test output".to_string();
        
        assert!(!block.is_collapsed);
        assert_eq!(block.get_display_output(), "test output");
        
        block.toggle_collapsed();
        assert!(block.is_collapsed);
        assert_eq!(block.get_display_output(), "");
        
        block.toggle_collapsed();
        assert!(!block.is_collapsed);
        assert_eq!(block.get_display_output(), "test output");
    }
}
