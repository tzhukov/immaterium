use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub general: GeneralConfig,
    pub appearance: AppearanceConfig,
    pub ai: AiConfig,
    pub mcp: McpConfig,
    pub keybindings: KeybindingsConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            appearance: AppearanceConfig::default(),
            ai: AiConfig::default(),
            mcp: McpConfig::default(),
            keybindings: KeybindingsConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub default_shell: String,
    pub save_history: bool,
    pub max_history_size: usize,
    pub auto_save_interval: u64, // seconds
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            default_shell: "/bin/bash".to_string(),
            save_history: true,
            max_history_size: 10000,
            auto_save_interval: 30,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppearanceConfig {
    pub theme: String,
    pub font_family: String,
    pub font_size: f32,
    pub show_line_numbers: bool,
    pub block_spacing: f32,
}

impl Default for AppearanceConfig {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            font_family: "monospace".to_string(),
            font_size: 14.0,
            show_line_numbers: true,
            block_spacing: 8.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    pub default_provider: String,
    pub enable_suggestions: bool,
    pub operation_mode: OperationMode,
    pub providers: HashMap<String, AiProviderConfig>,
    #[serde(default)]
    pub selected_model: Option<String>, // Last selected model
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OperationMode {
    #[serde(rename = "terminal_only")]
    TerminalOnly,      // Mode 1: Only execute as shell commands
    #[serde(rename = "ai_prompt_only")]
    AiPromptOnly,      // Mode 2: Always treat as AI prompts
    #[serde(rename = "hybrid")]
    Hybrid,            // Mode 3: Auto-detect (default)
}

impl Default for AiConfig {
    fn default() -> Self {
        let mut providers = HashMap::new();
        
        providers.insert(
            "ollama".to_string(),
            AiProviderConfig {
                base_url: Some("http://localhost:11434".to_string()),
                api_key: None,
                model: "codellama".to_string(),
                enabled: true,
            },
        );
        
        providers.insert(
            "openai".to_string(),
            AiProviderConfig {
                base_url: None,
                api_key: Some("${OPENAI_API_KEY}".to_string()),
                model: "gpt-4".to_string(),
                enabled: false,
            },
        );
        
        providers.insert(
            "groq".to_string(),
            AiProviderConfig {
                base_url: Some("https://api.groq.com/openai/v1".to_string()),
                api_key: Some("${GROQ_API_KEY}".to_string()),
                model: "mixtral-8x7b-32768".to_string(),
                enabled: false,
            },
        );

        Self {
            default_provider: "ollama".to_string(),
            enable_suggestions: true,
            operation_mode: OperationMode::Hybrid,
            providers,
            selected_model: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiProviderConfig {
    pub base_url: Option<String>,
    pub api_key: Option<String>,
    pub model: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    pub servers: Vec<McpServerConfig>,
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            servers: vec![
                // Example MCP server configuration
                // McpServerConfig {
                //     name: "filesystem".to_string(),
                //     command: "mcp-server-filesystem".to_string(),
                //     args: vec!["/home/user".to_string()],
                //     env: HashMap::new(),
                //     auto_start: true,
                // },
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub auto_start: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeybindingsConfig {
    pub new_block: String,
    pub ai_suggest: String,
    pub search: String,
    pub history: String,
    pub split_horizontal: String,
    pub split_vertical: String,
    pub close_pane: String,
    pub settings: String,
}

impl Default for KeybindingsConfig {
    fn default() -> Self {
        Self {
            new_block: "Ctrl+Enter".to_string(),
            ai_suggest: "Ctrl+Space".to_string(),
            search: "Ctrl+F".to_string(),
            history: "Ctrl+R".to_string(),
            split_horizontal: "Ctrl+Shift+H".to_string(),
            split_vertical: "Ctrl+Shift+V".to_string(),
            close_pane: "Ctrl+Shift+W".to_string(),
            settings: "Ctrl+,".to_string(),
        }
    }
}
