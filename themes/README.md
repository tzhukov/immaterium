# Immaterium Themes

This directory contains example theme files for the Immaterium terminal.

## Built-in Themes

### Dark (default)
Catppuccin Mocha-inspired dark theme with soothing purple and blue tones.
- File: `dark.toml`
- Best for: Low-light environments, extended coding sessions

### Light
Catppuccin Latte-inspired light theme with warm, professional colors.
- File: `light.toml`
- Best for: Bright environments, presentations

### High Contrast
Accessibility-focused theme with maximum contrast between text and background.
- File: `high_contrast.toml`
- Best for: Visual accessibility, outdoor use, bright screens

### Warp
Inspired by the Warp terminal, featuring modern blue-gray tones.
- File: `warp.toml`
- Best for: Users familiar with Warp terminal

## Creating Custom Themes

### 1. Copy an existing theme
```bash
cp dark.toml my_custom_theme.toml
```

### 2. Edit the theme file

The theme file is in TOML format with the following structure:

```toml
name = "My Custom Theme"

[colors]
# Main background colors
background = { r = 30, g = 30, b = 46, a = 255 }
background_secondary = { r = 36, g = 36, b = 59, a = 255 }
background_tertiary = { r = 49, g = 50, b = 68, a = 255 }

# Text colors
text_primary = { r = 205, g = 214, b = 244, a = 255 }
text_secondary = { r = 166, g = 173, b = 200, a = 255 }
text_disabled = { r = 108, g = 112, b = 134, a = 255 }

# UI element colors
border = { r = 69, g = 71, b = 90, a = 255 }
selection = { r = 137, g = 180, b = 250, a = 50 }
cursor = { r = 245, g = 194, b = 231, a = 255 }
highlight = { r = 250, g = 179, b = 135, a = 30 }

# Block state colors
block_running = { r = 137, g = 180, b = 250, a = 255 }
block_success = { r = 166, g = 227, b = 161, a = 255 }
block_error = { r = 243, g = 139, b = 168, a = 255 }
block_editing = { r = 249, g = 226, b = 175, a = 255 }

# ANSI terminal colors (16 colors)
ansi_black = { r = 69, g = 71, b = 90, a = 255 }
# ... and so on for all 16 ANSI colors

[fonts]
family = "monospace"  # Or specific font like "JetBrains Mono"
size = 14.0
line_height = 1.5

[spacing]
block_spacing = 10.0
padding = 8.0
border_width = 1.0
border_radius = 4.0

[syntax]
# Syntax highlighting colors
keyword = { r = 203, g = 166, b = 247, a = 255 }
string = { r = 166, g = 227, b = 161, a = 255 }
comment = { r = 108, g = 112, b = 134, a = 255 }
function = { r = 137, g = 180, b = 250, a = 255 }
variable = { r = 205, g = 214, b = 244, a = 255 }
number = { r = 250, g = 179, b = 135, a = 255 }
operator = { r = 148, g = 226, b = 213, a = 255 }
type_name = { r = 249, g = 226, b = 175, a = 255 }
```

### 3. Install your custom theme

Place your custom theme file in:
```
~/.config/immaterium/themes/my_custom_theme.toml
```

On Linux: `~/.config/immaterium/themes/`
On macOS: `~/Library/Application Support/immaterium/themes/`
On Windows: `%APPDATA%\immaterium\themes\`

### 4. Use your theme

1. Launch Immaterium
2. Go to View → Change Theme
3. Select your custom theme from the list

## Color Guidelines

### Contrast
- Ensure good contrast between text and background (WCAG AA: 4.5:1 minimum)
- Use higher contrast for important UI elements

### ANSI Colors
The 16 ANSI colors are used for terminal output:
- 0-7: Normal colors (black, red, green, yellow, blue, magenta, cyan, white)
- 8-15: Bright variants

### Block States
- `block_running`: Shown while command is executing (typically blue)
- `block_success`: Shown when exit code is 0 (typically green)
- `block_error`: Shown when exit code is non-zero (typically red)
- `block_editing`: Shown when editing a command (typically yellow/orange)

## Tips

1. **Start from an existing theme** that's close to what you want
2. **Test in different lighting** conditions
3. **Check readability** of all text colors
4. **Use alpha channel** for subtle overlays (selection, highlight)
5. **Keep syntax colors distinct** but harmonious

## Sharing Themes

To share your theme with others:

```bash
# Export from the app (coming soon)
# Or simply share your .toml file
```

## Resources

- [Catppuccin](https://github.com/catppuccin/catppuccin) - Color palette inspiration
- [Nord](https://www.nordtheme.com/) - Another great palette
- [Dracula](https://draculatheme.com/) - Popular dark theme
- [Tokyo Night](https://github.com/tokyo-night/tokyo-night-vscode-theme) - Modern dark theme

## Troubleshooting

**Theme not loading?**
- Check TOML syntax with `cat ~/.config/immaterium/themes/your_theme.toml`
- Look for logs in the terminal output
- Ensure all required fields are present

**Colors look wrong?**
- RGB values must be 0-255
- Alpha (a) values must be 0-255 (0 = transparent, 255 = opaque)
- Use alpha < 255 only for overlays like selection/highlight
