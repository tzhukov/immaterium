use super::Block;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub name: String,
    pub blocks: Vec<Block>,
    pub environment: HashMap<String, String>,
    pub working_directory: PathBuf,
}

impl Session {
    pub fn new(name: String, working_directory: PathBuf) -> Self {
        Self {
            id: Uuid::new_v4(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            name,
            blocks: Vec::new(),
            environment: HashMap::new(),
            working_directory,
        }
    }

    pub fn add_block(&mut self, block: Block) {
        self.blocks.push(block);
        self.updated_at = Utc::now();
    }
}
