use super::schema::Theme;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::Path;

pub struct ThemeLoader {
    themes: HashMap<String, Theme>,
    current_theme: String,
}

impl ThemeLoader {
    pub fn new() -> Self {
        let mut themes = HashMap::new();
        
        // Load built-in themes
        let dark = Theme::dark();
        let light = Theme::light();
        let high_contrast = Theme::high_contrast();
        let warp = Theme::warp();
        
        themes.insert(dark.name.clone(), dark);
        themes.insert(light.name.clone(), light);
        themes.insert(high_contrast.name.clone(), high_contrast);
        themes.insert(warp.name.clone(), warp);
        
        Self {
            themes,
            current_theme: "Dark".to_string(),
        }
    }

    /// Get the current theme
    pub fn current(&self) -> &Theme {
        self.themes.get(&self.current_theme)
            .unwrap_or_else(|| self.themes.get("Dark").unwrap())
    }

    /// Switch to a different theme
    pub fn set_theme(&mut self, name: &str) -> Result<()> {
        if self.themes.contains_key(name) {
            self.current_theme = name.to_string();
            Ok(())
        } else {
            anyhow::bail!("Theme '{}' not found", name)
        }
    }

    /// Get all available theme names
    pub fn available_themes(&self) -> Vec<String> {
        self.themes.keys().cloned().collect()
    }

    /// Load a custom theme from TOML file
    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let content = std::fs::read_to_string(path.as_ref())
            .context("Failed to read theme file")?;
        
        let theme: Theme = toml::from_str(&content)
            .context("Failed to parse theme TOML")?;
        
        let name = theme.name.clone();
        self.themes.insert(name.clone(), theme);
        
        tracing::info!("Loaded custom theme: {}", name);
        Ok(())
    }

    /// Load all themes from a directory
    pub fn load_from_directory<P: AsRef<Path>>(&mut self, dir: P) -> Result<()> {
        let dir_path = dir.as_ref();
        
        if !dir_path.exists() {
            return Ok(());
        }

        for entry in std::fs::read_dir(dir_path)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                if let Err(e) = self.load_from_file(&path) {
                    tracing::warn!("Failed to load theme from {:?}: {}", path, e);
                }
            }
        }

        Ok(())
    }

    /// Export a theme to a TOML file
    pub fn export_theme<P: AsRef<Path>>(&self, theme_name: &str, path: P) -> Result<()> {
        let theme = self.themes.get(theme_name)
            .ok_or_else(|| anyhow::anyhow!("Theme '{}' not found", theme_name))?;
        
        let toml = toml::to_string_pretty(theme)
            .context("Failed to serialize theme to TOML")?;
        
        std::fs::write(path.as_ref(), toml)
            .context("Failed to write theme file")?;
        
        Ok(())
    }

    /// Apply theme to egui context
    pub fn apply_to_egui(&self, ctx: &egui::Context) {
        let theme = self.current();
        let mut visuals = egui::Visuals::dark();
        
        // Background colors
        visuals.panel_fill = theme.colors.background.to_egui();
        visuals.window_fill = theme.colors.background_secondary.to_egui();
        visuals.extreme_bg_color = theme.colors.background_tertiary.to_egui();
        
        // Text colors
        visuals.override_text_color = Some(theme.colors.text_primary.to_egui());
        visuals.warn_fg_color = theme.colors.block_error.to_egui();
        
        // Widgets
        visuals.widgets.noninteractive.bg_fill = theme.colors.background_secondary.to_egui();
        visuals.widgets.noninteractive.fg_stroke.color = theme.colors.text_primary.to_egui();
        visuals.widgets.inactive.bg_fill = theme.colors.background_tertiary.to_egui();
        visuals.widgets.hovered.bg_fill = theme.colors.highlight.to_egui();
        visuals.widgets.active.bg_fill = theme.colors.selection.to_egui();
        
        // Selection
        visuals.selection.bg_fill = theme.colors.selection.to_egui();
        visuals.selection.stroke.color = theme.colors.text_primary.to_egui();
        
        // Hyperlinks
        visuals.hyperlink_color = theme.colors.block_running.to_egui();
        
        ctx.set_visuals(visuals);
        
        // Update text styles
        let mut style = (*ctx.style()).clone();
        style.text_styles.insert(
            egui::TextStyle::Monospace,
            egui::FontId::new(theme.fonts.size, egui::FontFamily::Monospace),
        );
        style.text_styles.insert(
            egui::TextStyle::Body,
            egui::FontId::new(theme.fonts.size, egui::FontFamily::Proportional),
        );
        style.spacing.item_spacing = egui::vec2(theme.spacing.padding, theme.spacing.padding);
        
        ctx.set_style(style);
    }
}

impl Default for ThemeLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_theme_loader_creation() {
        let loader = ThemeLoader::new();
        let themes = loader.available_themes();
        assert!(themes.contains(&"Dark".to_string()));
        assert!(themes.contains(&"Light".to_string()));
        assert!(themes.contains(&"High Contrast".to_string()));
        assert!(themes.contains(&"Warp".to_string()));
        assert_eq!(themes.len(), 4);
    }

    #[test]
    fn test_theme_switching() {
        let mut loader = ThemeLoader::new();
        assert_eq!(loader.current().name, "Dark");
        
        loader.set_theme("Light").unwrap();
        assert_eq!(loader.current().name, "Light");
    }

    #[test]
    fn test_export_import_theme() {
        let temp_dir = tempdir().unwrap();
        let theme_file = temp_dir.path().join("test_theme.toml");
        
        let loader = ThemeLoader::new();
        loader.export_theme("Dark", &theme_file).unwrap();
        
        assert!(theme_file.exists());
        
        let mut loader2 = ThemeLoader::new();
        loader2.load_from_file(&theme_file).unwrap();
        
        assert!(loader2.available_themes().len() >= 4);
    }

    #[test]
    #[ignore] // Run with: cargo test -- --ignored
    fn export_default_themes() {
        let loader = ThemeLoader::new();
        std::fs::create_dir_all("themes").unwrap();
        
        loader.export_theme("Dark", "themes/dark.toml").unwrap();
        loader.export_theme("Light", "themes/light.toml").unwrap();
        loader.export_theme("High Contrast", "themes/high_contrast.toml").unwrap();
        loader.export_theme("Warp", "themes/warp.toml").unwrap();
        
        println!("✅ Exported all themes to themes/ directory");
    }
}
