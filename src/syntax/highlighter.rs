use syntect::highlighting::{ThemeSet, HighlightIterator, Highlighter, Style};
use syntect::parsing::{SyntaxSet, SyntaxReference};
use syntect::easy::HighlightLines;
use lazy_static::lazy_static;

lazy_static! {
    /// Global syntax set with common languages
    pub static ref SYNTAX_SET: SyntaxSet = SyntaxSet::load_defaults_newlines();
    
    /// Default highlighting theme (we'll use our own theme colors)
    pub static ref THEME_SET: ThemeSet = ThemeSet::load_defaults();
}

pub struct SyntaxHighlighter {
    syntax_set: &'static SyntaxSet,
}

impl SyntaxHighlighter {
    pub fn new() -> Self {
        Self {
            syntax_set: &SYNTAX_SET,
        }
    }

    /// Detect language from code snippet or command
    pub fn detect_syntax(&self, text: &str) -> Option<&SyntaxReference> {
        // Try to detect from first line (shebang)
        if text.starts_with("#!") {
            if text.contains("bash") || text.contains("sh") {
                return self.syntax_set.find_syntax_by_name("Bourne Again Shell (bash)");
            } else if text.contains("python") {
                return self.syntax_set.find_syntax_by_name("Python");
            } else if text.contains("node") || text.contains("javascript") {
                return self.syntax_set.find_syntax_by_name("JavaScript");
            }
        }

        // Default to bash for shell commands
        self.syntax_set.find_syntax_by_name("Bourne Again Shell (bash)")
    }

    /// Highlight a shell command
    pub fn highlight_command(&self, command: &str) -> Vec<(Style, String)> {
        let syntax = self.detect_syntax(command)
            .or_else(|| self.syntax_set.find_syntax_by_name("Bourne Again Shell (bash)"))
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());

        let theme = &THEME_SET.themes["base16-ocean.dark"];
        let mut highlighter = HighlightLines::new(syntax, theme);
        
        let mut result = Vec::new();
        for line in command.lines() {
            let ranges = highlighter.highlight_line(line, &self.syntax_set).unwrap_or_default();
            for (style, text) in ranges {
                result.push((style, text.to_string()));
            }
            result.push((Style::default(), "\n".to_string()));
        }
        
        result
    }

    /// Detect if output contains code blocks (markdown-style)
    pub fn detect_code_blocks(&self, output: &str) -> Vec<CodeBlock> {
        let mut blocks = Vec::new();
        let mut in_block = false;
        let mut current_block = String::new();
        let mut current_lang = None;
        let mut start_line = 0;

        for (line_num, line) in output.lines().enumerate() {
            if line.starts_with("```") {
                if in_block {
                    // End of code block
                    blocks.push(CodeBlock {
                        content: current_block.clone(),
                        language: current_lang.clone(),
                        start_line,
                        end_line: line_num,
                    });
                    current_block.clear();
                    current_lang = None;
                    in_block = false;
                } else {
                    // Start of code block
                    let lang = line.trim_start_matches("```").trim();
                    current_lang = if lang.is_empty() {
                        None
                    } else {
                        Some(lang.to_string())
                    };
                    start_line = line_num + 1;
                    in_block = true;
                }
            } else if in_block {
                current_block.push_str(line);
                current_block.push('\n');
            }
        }

        blocks
    }
}

impl Default for SyntaxHighlighter {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct CodeBlock {
    pub content: String,
    pub language: Option<String>,
    pub start_line: usize,
    pub end_line: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_highlighter_creation() {
        let highlighter = SyntaxHighlighter::new();
        assert!(highlighter.syntax_set.syntaxes().len() > 0);
    }

    #[test]
    fn test_detect_bash_syntax() {
        let highlighter = SyntaxHighlighter::new();
        let syntax = highlighter.detect_syntax("echo hello");
        assert!(syntax.is_some());
        assert!(syntax.unwrap().name.contains("bash") || syntax.unwrap().name.contains("Shell"));
    }

    #[test]
    fn test_detect_python_shebang() {
        let highlighter = SyntaxHighlighter::new();
        let syntax = highlighter.detect_syntax("#!/usr/bin/env python\nprint('hello')");
        assert!(syntax.is_some());
        assert_eq!(syntax.unwrap().name, "Python");
    }

    #[test]
    fn test_highlight_simple_command() {
        let highlighter = SyntaxHighlighter::new();
        let result = highlighter.highlight_command("echo 'hello world'");
        assert!(!result.is_empty());
    }

    #[test]
    fn test_detect_code_blocks() {
        let highlighter = SyntaxHighlighter::new();
        let output = r#"
Some text before
```python
def hello():
    print("world")
```
Some text after
"#;
        let blocks = highlighter.detect_code_blocks(output);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].language, Some("python".to_string()));
        assert!(blocks[0].content.contains("def hello"));
    }

    #[test]
    fn test_multiple_code_blocks() {
        let highlighter = SyntaxHighlighter::new();
        let output = r#"
```bash
echo "first"
```
middle text
```python
print("second")
```
"#;
        let blocks = highlighter.detect_code_blocks(output);
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].language, Some("bash".to_string()));
        assert_eq!(blocks[1].language, Some("python".to_string()));
    }
}
