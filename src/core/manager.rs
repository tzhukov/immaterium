use super::Block;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct BlockManager {
    blocks: Vec<Block>,
    selected_block: Option<Uuid>,
}

impl BlockManager {
    pub fn new() -> Self {
        Self {
            blocks: Vec::new(),
            selected_block: None,
        }
    }

    pub fn add_block(&mut self, block: Block) -> Uuid {
        let id = block.id;
        self.blocks.push(block);
        id
    }

    pub fn get_block(&self, id: &Uuid) -> Option<&Block> {
        self.blocks.iter().find(|b| &b.id == id)
    }

    pub fn get_block_mut(&mut self, id: &Uuid) -> Option<&mut Block> {
        self.blocks.iter_mut().find(|b| &b.id == id)
    }

    pub fn get_blocks(&self) -> &[Block] {
        &self.blocks
    }

    pub fn get_blocks_mut(&mut self) -> &mut Vec<Block> {
        &mut self.blocks
    }

    pub fn remove_block(&mut self, id: &Uuid) -> Option<Block> {
        if let Some(pos) = self.blocks.iter().position(|b| &b.id == id) {
            Some(self.blocks.remove(pos))
        } else {
            None
        }
    }

    pub fn clear_all(&mut self) {
        self.blocks.clear();
        self.selected_block = None;
    }

    pub fn count(&self) -> usize {
        self.blocks.len()
    }

    pub fn select_block(&mut self, id: Uuid) {
        // Deselect all blocks
        for block in &mut self.blocks {
            block.set_selected(false);
        }

        // Select the specified block
        if let Some(block) = self.get_block_mut(&id) {
            block.set_selected(true);
            self.selected_block = Some(id);
        }
    }

    pub fn deselect_all(&mut self) {
        for block in &mut self.blocks {
            block.set_selected(false);
        }
        self.selected_block = None;
    }

    pub fn get_selected_block(&self) -> Option<&Block> {
        self.selected_block
            .as_ref()
            .and_then(|id| self.get_block(id))
    }

    pub fn get_selected_block_mut(&mut self) -> Option<&mut Block> {
        if let Some(id) = self.selected_block {
            self.get_block_mut(&id)
        } else {
            None
        }
    }

    pub fn toggle_block_collapsed(&mut self, id: &Uuid) {
        if let Some(block) = self.get_block_mut(id) {
            block.toggle_collapsed();
        }
    }

    pub fn get_running_blocks(&self) -> Vec<&Block> {
        self.blocks.iter().filter(|b| b.is_running()).collect()
    }

    pub fn get_last_block(&self) -> Option<&Block> {
        self.blocks.last()
    }

    pub fn get_last_block_mut(&mut self) -> Option<&mut Block> {
        self.blocks.last_mut()
    }

    pub fn copy_block_command(&self, id: &Uuid) -> Option<String> {
        self.get_block(id).map(|b| b.command.clone())
    }

    pub fn copy_block_output(&self, id: &Uuid) -> Option<String> {
        self.get_block(id).map(|b| b.output.clone())
    }

    pub fn copy_block_full(&self, id: &Uuid) -> Option<String> {
        self.get_block(id).map(|b| {
            format!(
                "$ {}\n{}\n[Exit code: {}]",
                b.command,
                b.output,
                b.exit_code.unwrap_or(-1)
            )
        })
    }

    /// Create a new block from editing an existing one
    pub fn duplicate_block_for_edit(&mut self, id: &Uuid) -> Option<Uuid> {
        if let Some(original) = self.get_block(id) {
            let new_block = Block::new(
                original.command.clone(),
                original.metadata.working_directory.clone(),
            );
            let new_id = new_block.id;
            self.blocks.push(new_block);
            Some(new_id)
        } else {
            None
        }
    }
}

impl Default for BlockManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_block_manager_creation() {
        let manager = BlockManager::new();
        assert_eq!(manager.count(), 0);
    }

    #[test]
    fn test_add_and_get_block() {
        let mut manager = BlockManager::new();
        let block = Block::new("echo test".to_string(), PathBuf::from("/tmp"));
        let id = block.id;
        
        manager.add_block(block);
        assert_eq!(manager.count(), 1);
        
        let retrieved = manager.get_block(&id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().command, "echo test");
    }

    #[test]
    fn test_remove_block() {
        let mut manager = BlockManager::new();
        let block = Block::new("echo test".to_string(), PathBuf::from("/tmp"));
        let id = block.id;
        
        manager.add_block(block);
        assert_eq!(manager.count(), 1);
        
        let removed = manager.remove_block(&id);
        assert!(removed.is_some());
        assert_eq!(manager.count(), 0);
    }

    #[test]
    fn test_block_selection() {
        let mut manager = BlockManager::new();
        let block1 = Block::new("echo 1".to_string(), PathBuf::from("/tmp"));
        let block2 = Block::new("echo 2".to_string(), PathBuf::from("/tmp"));
        let id1 = block1.id;
        let id2 = block2.id;
        
        manager.add_block(block1);
        manager.add_block(block2);
        
        manager.select_block(id1);
        assert!(manager.get_selected_block().is_some());
        assert_eq!(manager.get_selected_block().unwrap().id, id1);
        
        manager.select_block(id2);
        assert_eq!(manager.get_selected_block().unwrap().id, id2);
        
        manager.deselect_all();
        assert!(manager.get_selected_block().is_none());
    }

    #[test]
    fn test_copy_operations() {
        let mut manager = BlockManager::new();
        let mut block = Block::new("echo test".to_string(), PathBuf::from("/tmp"));
        block.output = "test output".to_string();
        block.exit_code = Some(0);
        let id = block.id;
        
        manager.add_block(block);
        
        assert_eq!(manager.copy_block_command(&id), Some("echo test".to_string()));
        assert_eq!(manager.copy_block_output(&id), Some("test output".to_string()));
        
        let full = manager.copy_block_full(&id).unwrap();
        assert!(full.contains("echo test"));
        assert!(full.contains("test output"));
        assert!(full.contains("[Exit code: 0]"));
    }
}
