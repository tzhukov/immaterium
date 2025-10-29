use super::Config;
use anyhow::{Context, Result};
use directories::ProjectDirs;
use std::fs;
use std::path::PathBuf;

impl Config {
    /// Load configuration from the default location or create a new one
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;
        
        if config_path.exists() {
            Self::load_from_file(&config_path)
        } else {
            tracing::info!("Config file not found, creating default configuration");
            let config = Config::default();
            config.save()?;
            Ok(config)
        }
    }

    /// Load configuration from a specific file
    pub fn load_from_file(path: &PathBuf) -> Result<Self> {
        let contents = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {:?}", path))?;
        
        let mut config: Config = toml::from_str(&contents)
            .with_context(|| "Failed to parse config file")?;
        
        // Expand environment variables
        config.expand_env_vars();
        
        tracing::debug!("Loaded configuration from {:?}", path);
        Ok(config)
    }

    /// Save configuration to the default location
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;
        
        // Create parent directories if they don't exist
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory: {:?}", parent))?;
        }
        
        let contents = toml::to_string_pretty(self)
            .with_context(|| "Failed to serialize config")?;
        
        fs::write(&config_path, contents)
            .with_context(|| format!("Failed to write config file: {:?}", config_path))?;
        
        tracing::info!("Saved configuration to {:?}", config_path);
        Ok(())
    }

    /// Get the default configuration file path
    pub fn config_path() -> Result<PathBuf> {
        let proj_dirs = ProjectDirs::from("com", "immaterium", "immaterium")
            .context("Failed to determine project directories")?;
        
        Ok(proj_dirs.config_dir().join("config.toml"))
    }

    /// Get the data directory path
    pub fn data_dir() -> Result<PathBuf> {
        let proj_dirs = ProjectDirs::from("com", "immaterium", "immaterium")
            .context("Failed to determine project directories")?;
        
        let data_dir = proj_dirs.data_dir();
        fs::create_dir_all(data_dir)
            .with_context(|| format!("Failed to create data directory: {:?}", data_dir))?;
        
        Ok(data_dir.to_path_buf())
    }

    /// Expand environment variables in configuration values
    fn expand_env_vars(&mut self) {
        // Expand AI provider API keys
        for provider in self.ai.providers.values_mut() {
            if let Some(api_key) = &provider.api_key {
                provider.api_key = Some(Self::expand_env_var(api_key));
            }
        }
        
        // Expand MCP server environment variables
        for server in &mut self.mcp.servers {
            for (_, value) in server.env.iter_mut() {
                *value = Self::expand_env_var(value);
            }
        }
    }

    /// Expand a single environment variable
    fn expand_env_var(value: &str) -> String {
        if value.starts_with("${") && value.ends_with('}') {
            let var_name = &value[2..value.len() - 1];
            std::env::var(var_name).unwrap_or_else(|_| {
                tracing::warn!("Environment variable not found: {}", var_name);
                value.to_string()
            })
        } else {
            value.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.general.default_shell, "/bin/bash");
        assert!(config.general.save_history);
        assert_eq!(config.appearance.theme, "dark");
    }

    #[test]
    fn test_env_var_expansion() {
        std::env::set_var("TEST_VAR", "test_value");
        let expanded = Config::expand_env_var("${TEST_VAR}");
        assert_eq!(expanded, "test_value");
        
        let not_expanded = Config::expand_env_var("normal_value");
        assert_eq!(not_expanded, "normal_value");
    }
}
