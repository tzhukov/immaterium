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
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BlockState {
    Editing,
    Running,
    Completed,
    Failed,
    Collapsed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockMetadata {
    pub duration: Option<Duration>,
    pub working_directory: PathBuf,
    pub environment: HashMap<String, String>,
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
            },
        }
    }
}
