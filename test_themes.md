# Theme Testing Checklist

## Steps to test themes:

1. **Launch Application** ✓
   - App should start with Dark theme by default
   - Check that background is dark (Catppuccin Mocha colors)

2. **Open Theme Selector**
   - Click "View" menu → "🎨 Change Theme..."
   - Dialog should appear with 4 themes listed

3. **Test Each Theme**
   
   ### Dark Theme (default)
   - Background: Dark blue-gray (#1e1e2e)
   - Text: Light (#cdd6f4)
   - Should be easy on the eyes
   
   ### Light Theme
   - Background: Light gray (#eff1f5)
   - Text: Dark (#4c4f69)
   - Should be bright and clean
   
   ### High Contrast Theme
   - Background: Pure black (#000000)
   - Text: Pure white (#ffffff)
   - Borders should be more visible
   - Better for accessibility
   
   ### Warp Theme
   - Background: Blue-gray (#232832)
   - Text: Light (#dce1eb)
   - Similar to Warp terminal aesthetic

4. **Test Real-time Switching**
   - Switch between themes
   - UI should update immediately without restart
   - All UI elements should respect theme colors

5. **Test Session Functionality with Themes**
   - Create a new session
   - Run a command: `echo "Testing themes"`
   - Check that block colors match theme
   - Success block should use theme's success color

6. **Test Custom Theme Loading**
   - Check that custom themes directory is created
   - Verify themes load from ~/.config/immaterium/themes/

## Expected Behavior:
- Theme switching is instant
- All colors update consistently
- Text remains readable in all themes
- No visual glitches during switching
