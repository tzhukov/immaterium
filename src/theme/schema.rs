use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub colors: ColorScheme,
    pub fonts: FontConfig,
    pub spacing: SpacingConfig,
    pub syntax: SyntaxColors,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorScheme {
    // Background colors
    pub background: Color,
    pub background_secondary: Color,
    pub background_tertiary: Color,
    
    // Text colors
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_disabled: Color,
    
    // UI elements
    pub border: Color,
    pub selection: Color,
    pub cursor: Color,
    pub highlight: Color,
    
    // Block states
    pub block_running: Color,
    pub block_success: Color,
    pub block_error: Color,
    pub block_editing: Color,
    
    // Terminal ANSI colors
    pub ansi_black: Color,
    pub ansi_red: Color,
    pub ansi_green: Color,
    pub ansi_yellow: Color,
    pub ansi_blue: Color,
    pub ansi_magenta: Color,
    pub ansi_cyan: Color,
    pub ansi_white: Color,
    pub ansi_bright_black: Color,
    pub ansi_bright_red: Color,
    pub ansi_bright_green: Color,
    pub ansi_bright_yellow: Color,
    pub ansi_bright_blue: Color,
    pub ansi_bright_magenta: Color,
    pub ansi_bright_cyan: Color,
    pub ansi_bright_white: Color,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    #[serde(default = "default_alpha")]
    pub a: u8,
}

fn default_alpha() -> u8 {
    255
}

impl Color {
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn to_egui(&self) -> egui::Color32 {
        egui::Color32::from_rgba_premultiplied(self.r, self.g, self.b, self.a)
    }

    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.trim_start_matches('#');
        
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        let a = if hex.len() == 8 {
            u8::from_str_radix(&hex[6..8], 16).ok()?
        } else {
            255
        };

        Some(Self { r, g, b, a })
    }

    pub fn to_hex(&self) -> String {
        if self.a == 255 {
            format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
        } else {
            format!("#{:02x}{:02x}{:02x}{:02x}", self.r, self.g, self.b, self.a)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontConfig {
    #[serde(default = "default_font_family")]
    pub family: String,
    #[serde(default = "default_font_size")]
    pub size: f32,
    #[serde(default = "default_line_height")]
    pub line_height: f32,
}

fn default_font_family() -> String {
    "monospace".to_string()
}

fn default_font_size() -> f32 {
    14.0
}

fn default_line_height() -> f32 {
    1.5
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpacingConfig {
    #[serde(default = "default_block_spacing")]
    pub block_spacing: f32,
    #[serde(default = "default_padding")]
    pub padding: f32,
    #[serde(default = "default_border_width")]
    pub border_width: f32,
    #[serde(default = "default_border_radius")]
    pub border_radius: f32,
}

fn default_block_spacing() -> f32 {
    10.0
}

fn default_padding() -> f32 {
    8.0
}

fn default_border_width() -> f32 {
    1.0
}

fn default_border_radius() -> f32 {
    4.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntaxColors {
    pub keyword: Color,
    pub string: Color,
    pub comment: Color,
    pub function: Color,
    pub variable: Color,
    pub number: Color,
    pub operator: Color,
    pub type_name: Color,
}

impl Theme {
    /// Create a default dark theme
    pub fn dark() -> Self {
        Self {
            name: "Dark".to_string(),
            colors: ColorScheme {
                background: Color::rgb(30, 30, 46),
                background_secondary: Color::rgb(36, 36, 59),
                background_tertiary: Color::rgb(49, 50, 68),
                text_primary: Color::rgb(205, 214, 244),
                text_secondary: Color::rgb(166, 173, 200),
                text_disabled: Color::rgb(108, 112, 134),
                border: Color::rgb(69, 71, 90),
                selection: Color::rgba(137, 180, 250, 50),
                cursor: Color::rgb(245, 194, 231),
                highlight: Color::rgba(250, 179, 135, 30),
                block_running: Color::rgb(137, 180, 250),
                block_success: Color::rgb(166, 227, 161),
                block_error: Color::rgb(243, 139, 168),
                block_editing: Color::rgb(249, 226, 175),
                // Catppuccin Mocha ANSI colors
                ansi_black: Color::rgb(69, 71, 90),
                ansi_red: Color::rgb(243, 139, 168),
                ansi_green: Color::rgb(166, 227, 161),
                ansi_yellow: Color::rgb(249, 226, 175),
                ansi_blue: Color::rgb(137, 180, 250),
                ansi_magenta: Color::rgb(245, 194, 231),
                ansi_cyan: Color::rgb(148, 226, 213),
                ansi_white: Color::rgb(205, 214, 244),
                ansi_bright_black: Color::rgb(88, 91, 112),
                ansi_bright_red: Color::rgb(243, 139, 168),
                ansi_bright_green: Color::rgb(166, 227, 161),
                ansi_bright_yellow: Color::rgb(249, 226, 175),
                ansi_bright_blue: Color::rgb(137, 180, 250),
                ansi_bright_magenta: Color::rgb(245, 194, 231),
                ansi_bright_cyan: Color::rgb(148, 226, 213),
                ansi_bright_white: Color::rgb(166, 173, 200),
            },
            fonts: FontConfig {
                family: "monospace".to_string(),
                size: 14.0,
                line_height: 1.5,
            },
            spacing: SpacingConfig {
                block_spacing: 10.0,
                padding: 8.0,
                border_width: 1.0,
                border_radius: 4.0,
            },
            syntax: SyntaxColors {
                keyword: Color::rgb(203, 166, 247),
                string: Color::rgb(166, 227, 161),
                comment: Color::rgb(108, 112, 134),
                function: Color::rgb(137, 180, 250),
                variable: Color::rgb(205, 214, 244),
                number: Color::rgb(250, 179, 135),
                operator: Color::rgb(148, 226, 213),
                type_name: Color::rgb(249, 226, 175),
            },
        }
    }

    /// Create a default light theme
    pub fn light() -> Self {
        Self {
            name: "Light".to_string(),
            colors: ColorScheme {
                background: Color::rgb(239, 241, 245),
                background_secondary: Color::rgb(230, 233, 239),
                background_tertiary: Color::rgb(220, 224, 232),
                text_primary: Color::rgb(76, 79, 105),
                text_secondary: Color::rgb(92, 95, 119),
                text_disabled: Color::rgb(156, 160, 176),
                border: Color::rgb(188, 192, 204),
                selection: Color::rgba(30, 102, 245, 50),
                cursor: Color::rgb(234, 118, 203),
                highlight: Color::rgba(254, 100, 11, 30),
                block_running: Color::rgb(30, 102, 245),
                block_success: Color::rgb(64, 160, 43),
                block_error: Color::rgb(210, 15, 57),
                block_editing: Color::rgb(223, 142, 29),
                // Catppuccin Latte ANSI colors
                ansi_black: Color::rgb(76, 79, 105),
                ansi_red: Color::rgb(210, 15, 57),
                ansi_green: Color::rgb(64, 160, 43),
                ansi_yellow: Color::rgb(223, 142, 29),
                ansi_blue: Color::rgb(30, 102, 245),
                ansi_magenta: Color::rgb(234, 118, 203),
                ansi_cyan: Color::rgb(23, 146, 153),
                ansi_white: Color::rgb(76, 79, 105),
                ansi_bright_black: Color::rgb(92, 95, 119),
                ansi_bright_red: Color::rgb(210, 15, 57),
                ansi_bright_green: Color::rgb(64, 160, 43),
                ansi_bright_yellow: Color::rgb(223, 142, 29),
                ansi_bright_blue: Color::rgb(30, 102, 245),
                ansi_bright_magenta: Color::rgb(234, 118, 203),
                ansi_bright_cyan: Color::rgb(23, 146, 153),
                ansi_bright_white: Color::rgb(108, 111, 133),
            },
            fonts: FontConfig {
                family: "monospace".to_string(),
                size: 14.0,
                line_height: 1.5,
            },
            spacing: SpacingConfig {
                block_spacing: 10.0,
                padding: 8.0,
                border_width: 1.0,
                border_radius: 4.0,
            },
            syntax: SyntaxColors {
                keyword: Color::rgb(136, 57, 239),
                string: Color::rgb(64, 160, 43),
                comment: Color::rgb(156, 160, 176),
                function: Color::rgb(30, 102, 245),
                variable: Color::rgb(76, 79, 105),
                number: Color::rgb(254, 100, 11),
                operator: Color::rgb(23, 146, 153),
                type_name: Color::rgb(223, 142, 29),
            },
        }
    }

    /// Create a high contrast theme for accessibility
    pub fn high_contrast() -> Self {
        Self {
            name: "High Contrast".to_string(),
            colors: ColorScheme {
                background: Color::rgb(0, 0, 0),
                background_secondary: Color::rgb(20, 20, 20),
                background_tertiary: Color::rgb(40, 40, 40),
                text_primary: Color::rgb(255, 255, 255),
                text_secondary: Color::rgb(220, 220, 220),
                text_disabled: Color::rgb(150, 150, 150),
                border: Color::rgb(255, 255, 255),
                selection: Color::rgba(0, 120, 215, 100),
                cursor: Color::rgb(255, 255, 0),
                highlight: Color::rgba(255, 255, 0, 50),
                block_running: Color::rgb(0, 150, 255),
                block_success: Color::rgb(0, 255, 0),
                block_error: Color::rgb(255, 0, 0),
                block_editing: Color::rgb(255, 255, 0),
                // High contrast ANSI colors
                ansi_black: Color::rgb(0, 0, 0),
                ansi_red: Color::rgb(255, 0, 0),
                ansi_green: Color::rgb(0, 255, 0),
                ansi_yellow: Color::rgb(255, 255, 0),
                ansi_blue: Color::rgb(0, 150, 255),
                ansi_magenta: Color::rgb(255, 0, 255),
                ansi_cyan: Color::rgb(0, 255, 255),
                ansi_white: Color::rgb(255, 255, 255),
                ansi_bright_black: Color::rgb(100, 100, 100),
                ansi_bright_red: Color::rgb(255, 100, 100),
                ansi_bright_green: Color::rgb(100, 255, 100),
                ansi_bright_yellow: Color::rgb(255, 255, 100),
                ansi_bright_blue: Color::rgb(100, 200, 255),
                ansi_bright_magenta: Color::rgb(255, 100, 255),
                ansi_bright_cyan: Color::rgb(100, 255, 255),
                ansi_bright_white: Color::rgb(255, 255, 255),
            },
            fonts: FontConfig {
                family: "monospace".to_string(),
                size: 14.0,
                line_height: 1.5,
            },
            spacing: SpacingConfig {
                block_spacing: 12.0,
                padding: 10.0,
                border_width: 2.0,
                border_radius: 2.0,
            },
            syntax: SyntaxColors {
                keyword: Color::rgb(255, 100, 255),
                string: Color::rgb(0, 255, 100),
                comment: Color::rgb(150, 150, 150),
                function: Color::rgb(100, 200, 255),
                variable: Color::rgb(255, 255, 255),
                number: Color::rgb(255, 200, 100),
                operator: Color::rgb(0, 255, 255),
                type_name: Color::rgb(255, 255, 0),
            },
        }
    }

    /// Create a Warp-inspired theme
    pub fn warp() -> Self {
        Self {
            name: "Warp".to_string(),
            colors: ColorScheme {
                background: Color::rgb(35, 40, 50),
                background_secondary: Color::rgb(42, 47, 57),
                background_tertiary: Color::rgb(50, 55, 65),
                text_primary: Color::rgb(220, 225, 235),
                text_secondary: Color::rgb(180, 185, 195),
                text_disabled: Color::rgb(120, 125, 135),
                border: Color::rgb(70, 75, 85),
                selection: Color::rgba(100, 150, 255, 60),
                cursor: Color::rgb(255, 200, 100),
                highlight: Color::rgba(100, 150, 255, 30),
                block_running: Color::rgb(100, 150, 255),
                block_success: Color::rgb(100, 200, 150),
                block_error: Color::rgb(255, 100, 100),
                block_editing: Color::rgb(255, 200, 100),
                // Warp ANSI colors
                ansi_black: Color::rgb(50, 55, 65),
                ansi_red: Color::rgb(255, 100, 100),
                ansi_green: Color::rgb(100, 200, 150),
                ansi_yellow: Color::rgb(255, 200, 100),
                ansi_blue: Color::rgb(100, 150, 255),
                ansi_magenta: Color::rgb(200, 150, 255),
                ansi_cyan: Color::rgb(100, 200, 200),
                ansi_white: Color::rgb(220, 225, 235),
                ansi_bright_black: Color::rgb(100, 105, 115),
                ansi_bright_red: Color::rgb(255, 130, 130),
                ansi_bright_green: Color::rgb(130, 220, 170),
                ansi_bright_yellow: Color::rgb(255, 220, 130),
                ansi_bright_blue: Color::rgb(130, 170, 255),
                ansi_bright_magenta: Color::rgb(220, 170, 255),
                ansi_bright_cyan: Color::rgb(130, 220, 220),
                ansi_bright_white: Color::rgb(240, 245, 255),
            },
            fonts: FontConfig {
                family: "monospace".to_string(),
                size: 14.0,
                line_height: 1.6,
            },
            spacing: SpacingConfig {
                block_spacing: 12.0,
                padding: 10.0,
                border_width: 1.0,
                border_radius: 6.0,
            },
            syntax: SyntaxColors {
                keyword: Color::rgb(200, 150, 255),
                string: Color::rgb(100, 200, 150),
                comment: Color::rgb(120, 125, 135),
                function: Color::rgb(100, 150, 255),
                variable: Color::rgb(220, 225, 235),
                number: Color::rgb(255, 200, 100),
                operator: Color::rgb(100, 200, 200),
                type_name: Color::rgb(255, 220, 130),
            },
        }
    }
}
