use super::Session;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedSession {
    pub session: Session,
}

impl ExportedSession {
    pub fn new(session: Session) -> Self {
        Self { session }
    }

    /// Export session to JSON format
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(&self)
            .context("Failed to serialize session to JSON")
    }

    /// Export session to a JSON file
    pub fn to_json_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let json = self.to_json()?;
        std::fs::write(path.as_ref(), json)
            .context("Failed to write JSON file")?;
        Ok(())
    }

    /// Import session from JSON string
    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json)
            .context("Failed to deserialize session from JSON")
    }

    /// Import session from a JSON file
    pub fn from_json_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let json = std::fs::read_to_string(path.as_ref())
            .context("Failed to read JSON file")?;
        Self::from_json(&json)
    }

    /// Export session to Markdown format
    pub fn to_markdown(&self) -> String {
        let mut md = String::new();
        
        md.push_str(&format!("# Session: {}\n\n", self.session.name));
        md.push_str(&format!("**Created:** {}\n\n", self.session.created_at.format("%Y-%m-%d %H:%M:%S")));
        md.push_str(&format!("**Working Directory:** `{}`\n\n", self.session.working_directory.display()));
        
        if !self.session.blocks.is_empty() {
            md.push_str("## Commands\n\n");
            
            for (i, block) in self.session.blocks.iter().enumerate() {
                md.push_str(&format!("### Block {} - {}\n\n", i + 1, block.timestamp.format("%H:%M:%S")));
                
                // Command
                md.push_str("**Command:**\n```bash\n");
                md.push_str(&block.command);
                md.push_str("\n```\n\n");
                
                // Output
                if !block.output.is_empty() {
                    md.push_str("**Output:**\n```\n");
                    md.push_str(&block.output);
                    md.push_str("\n```\n\n");
                }
                
                // Status
                md.push_str(&format!("**Status:** {:?}", block.state));
                if let Some(code) = block.exit_code {
                    md.push_str(&format!(" (exit code: {})", code));
                }
                md.push_str("\n\n");
                
                // Duration
                if let Some(duration) = block.metadata.duration {
                    md.push_str(&format!("**Duration:** {:.2}s\n\n", duration.as_secs_f64()));
                }
                
                md.push_str("---\n\n");
            }
        }
        
        md
    }

    /// Export session to a Markdown file
    pub fn to_markdown_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let markdown = self.to_markdown();
        std::fs::write(path.as_ref(), markdown)
            .context("Failed to write Markdown file")?;
        Ok(())
    }

    /// Export session to plain text format
    pub fn to_text(&self) -> String {
        let mut text = String::new();
        
        text.push_str(&format!("Session: {}\n", self.session.name));
        text.push_str(&format!("Created: {}\n", self.session.created_at.format("%Y-%m-%d %H:%M:%S")));
        text.push_str(&format!("Working Directory: {}\n\n", self.session.working_directory.display()));
        
        if !self.session.blocks.is_empty() {
            for (i, block) in self.session.blocks.iter().enumerate() {
                text.push_str(&format!("[Block {}] {}\n", i + 1, block.timestamp.format("%H:%M:%S")));
                text.push_str(&format!("$ {}\n", block.command));
                
                if !block.output.is_empty() {
                    text.push_str(&block.output);
                    if !block.output.ends_with('\n') {
                        text.push('\n');
                    }
                }
                
                text.push_str(&format!("Status: {:?}", block.state));
                if let Some(code) = block.exit_code {
                    text.push_str(&format!(" (exit code: {})", code));
                }
                text.push_str("\n\n");
            }
        }
        
        text
    }

    /// Export session to a plain text file
    pub fn to_text_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let text = self.to_text();
        std::fs::write(path.as_ref(), text)
            .context("Failed to write text file")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Block;
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[test]
    fn test_json_export_import() {
        let session = Session::new("test".to_string(), PathBuf::from("/tmp"));
        let exported = ExportedSession::new(session.clone());
        
        let json = exported.to_json().unwrap();
        let imported = ExportedSession::from_json(&json).unwrap();
        
        assert_eq!(imported.session.id, session.id);
        assert_eq!(imported.session.name, session.name);
    }

    #[test]
    fn test_json_file_export_import() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("session.json");
        
        let session = Session::new("test".to_string(), PathBuf::from("/tmp"));
        let exported = ExportedSession::new(session.clone());
        
        exported.to_json_file(&file_path).unwrap();
        assert!(file_path.exists());
        
        let imported = ExportedSession::from_json_file(&file_path).unwrap();
        assert_eq!(imported.session.id, session.id);
    }

    #[test]
    fn test_markdown_export() {
        let mut session = Session::new("test".to_string(), PathBuf::from("/tmp"));
        let mut block = Block::new("echo hello".to_string(), PathBuf::from("/tmp"));
        block.start_execution();
        block.append_output("hello\n".to_string());
        block.complete_execution(0);
        session.blocks.push(block);
        
        let exported = ExportedSession::new(session);
        let markdown = exported.to_markdown();
        
        assert!(markdown.contains("# Session: test"));
        assert!(markdown.contains("echo hello"));
        assert!(markdown.contains("hello"));
    }

    #[test]
    fn test_text_export() {
        let mut session = Session::new("test".to_string(), PathBuf::from("/tmp"));
        let mut block = Block::new("ls".to_string(), PathBuf::from("/tmp"));
        block.start_execution();
        block.append_output("file1\nfile2\n".to_string());
        block.complete_execution(0);
        session.blocks.push(block);
        
        let exported = ExportedSession::new(session);
        let text = exported.to_text();
        
        assert!(text.contains("Session: test"));
        assert!(text.contains("$ ls"));
        assert!(text.contains("file1"));
        assert!(text.contains("file2"));
    }
}
