# Immaterium Terminal - Design Document

**Version:** 1.0  
**Date:** October 29, 2025  
**Target Platform:** Linux (initial), expandable to macOS/Windows  
**Language:** Rust

---

## 1. Executive Summary

Immaterium is a modern terminal emulator inspired by Warp, featuring AI-powered assistance, block-based command execution, and Model Context Protocol (MCP) server integration. Built in Rust with egui for the GUI, it combines traditional shell functionality with intelligent automation and enhanced user experience.

### Key Features
- **Block-based workflow**: Each command/output pair is a discrete, manageable block
- **AI Integration**: Support for multiple LLM providers (Ollama, Groq, OpenAI)
- **MCP Server Management**: Spawn, configure, and manage MCP servers
- **Modern UX**: Command completion, syntax highlighting, themes, keyboard shortcuts
- **Session Persistence**: Save and restore terminal sessions
- **Advanced Features**: Multi-pane views, searchable history, collapsible blocks

---

## 2. Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                        Immaterium                            │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌──────────────────────────────────────────────────────┐  │
│  │              GUI Layer (egui)                         │  │
│  │  - Block Renderer                                     │  │
│  │  - Input Handler                                      │  │
│  │  - Theme Manager                                      │  │
│  │  - Layout Manager (Panes/Splits)                      │  │
│  └──────────────────────────────────────────────────────┘  │
│                           │                                  │
│  ┌──────────────────────────────────────────────────────┐  │
│  │           Application Core                            │  │
│  │  - Block Manager                                      │  │
│  │  - Session Manager                                    │  │
│  │  - Command History                                    │  │
│  │  - Configuration Manager                              │  │
│  └──────────────────────────────────────────────────────┘  │
│           │              │              │                    │
│  ┌────────┴────┐  ┌─────┴─────┐  ┌────┴──────┐            │
│  │   Shell     │  │    AI     │  │    MCP    │            │
│  │  Executor   │  │  Engine   │  │  Manager  │            │
│  └─────────────┘  └───────────┘  └───────────┘            │
│                                                              │
└─────────────────────────────────────────────────────────────┘
         │                  │                  │
    ┌────┴────┐      ┌─────┴─────┐     ┌─────┴──────┐
    │  Bash   │      │   LLM     │     │   MCP      │
    │  Shell  │      │ Providers │     │  Servers   │
    └─────────┘      └───────────┘     └────────────┘
```

---

## 3. Core Components

### 3.1 GUI Layer (egui)

**Responsibilities:**
- Render terminal blocks with syntax highlighting
- Handle user input (keyboard, mouse)
- Manage themes and visual styling
- Support multi-pane layouts
- Provide interactive UI elements (buttons, menus, dialogs)

**Key Modules:**
- `ui/app.rs` - Main application window
- `ui/block_renderer.rs` - Block visualization
- `ui/input_handler.rs` - Keyboard/mouse input processing
- `ui/theme.rs` - Theme definitions and application
- `ui/layout.rs` - Pane management and splitting
- `ui/widgets/` - Custom widgets (command input, output viewer, etc.)

**Technologies:**
- `egui` - Immediate mode GUI framework
- `eframe` - Framework wrapper for egui
- `syntect` - Syntax highlighting
- `egui_extras` - Additional widgets

---

### 3.2 Block Manager

**Responsibilities:**
- Create, update, and delete command blocks
- Manage block state (collapsed/expanded, selected)
- Handle block operations (copy, paste, edit)
- Persist blocks to session files

**Block Structure:**
```rust
struct Block {
    id: Uuid,
    timestamp: DateTime<Utc>,
    command: String,
    output: String,
    exit_code: Option<i32>,
    state: BlockState,
    metadata: BlockMetadata,
}

enum BlockState {
    Editing,
    Running,
    Completed,
    Failed,
    Collapsed,
}

struct BlockMetadata {
    duration: Option<Duration>,
    working_directory: PathBuf,
    environment: HashMap<String, String>,
    ai_suggestions: Vec<AiSuggestion>,
}
```

---

### 3.3 Shell Executor

**Responsibilities:**
- Execute commands through bash
- Capture stdout, stderr, and exit codes
- Handle environment variables
- Support pipes, redirects, and standard shell features
- Manage long-running processes
- Track background jobs

**Key Features:**
- Async command execution (tokio)
- Streaming output capture
- Process tree management
- Signal handling (Ctrl+C, Ctrl+Z)
- Working directory tracking

**Technologies:**
- `tokio::process` - Async process execution
- `nix` - Unix system calls
- `pty-process` or `portable-pty` - PTY handling

---

### 3.4 AI Engine

**Responsibilities:**
- Interface with multiple LLM providers
- Stream AI responses
- Generate command suggestions
- Explain commands and errors
- Auto-complete based on context

**Provider Support:**
```rust
trait LlmProvider {
    async fn complete(&self, prompt: &str) -> Result<String>;
    async fn stream_complete(&self, prompt: &str) -> impl Stream<Item = String>;
    fn name(&self) -> &str;
}

// Implementations:
- OllamaProvider
- GroqProvider  
- OpenAiProvider
```

**Technologies:**
- `reqwest` - HTTP client for API calls
- `tokio-stream` - Streaming responses
- `serde_json` - JSON serialization
- Provider-specific SDKs or REST APIs

---

### 3.5 MCP Manager

**Responsibilities:**
- Spawn and manage MCP server processes
- Configure server connections (stdio/HTTP)
- Handle server lifecycle (start, stop, restart)
- Route requests to appropriate servers
- Load configuration from config files
- Support runtime server addition/removal

**MCP Server Configuration:**
```rust
struct McpServerConfig {
    name: String,
    command: String,
    args: Vec<String>,
    env: HashMap<String, String>,
    protocol: McpProtocol,
    auto_start: bool,
}

enum McpProtocol {
    Stdio,
    Http { url: String },
}
```

**Technologies:**
- MCP protocol implementation
- `tokio::process` - Process management
- JSON-RPC for communication

---

### 3.6 Session Manager

**Responsibilities:**
- Save terminal sessions to disk
- Load previous sessions
- Export sessions (JSON, Markdown, plain text)
- Manage session metadata

**Session Format:**
```rust
struct Session {
    id: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    name: String,
    blocks: Vec<Block>,
    environment: HashMap<String, String>,
    working_directory: PathBuf,
}
```

**Storage:**
- SQLite database for session metadata and blocks
- Optional JSON export for portability

---

### 3.7 Configuration Manager

**Responsibilities:**
- Load user configuration from TOML/YAML
- Manage themes, keybindings, AI settings
- MCP server configurations
- User preferences (font, layout, behavior)

**Config Structure:**
```toml
[general]
default_shell = "/bin/bash"
save_history = true
max_history_size = 10000

[appearance]
theme = "dark"
font_family = "JetBrains Mono"
font_size = 14
show_line_numbers = true

[ai]
default_provider = "ollama"
enable_suggestions = true

[ai.providers.ollama]
base_url = "http://localhost:11434"
model = "codellama"

[ai.providers.openai]
api_key = "${OPENAI_API_KEY}"
model = "gpt-4"

[ai.providers.groq]
api_key = "${GROQ_API_KEY}"
model = "mixtral-8x7b"

[[mcp.servers]]
name = "filesystem"
command = "mcp-server-filesystem"
args = ["/home/user"]
auto_start = true

[keybindings]
new_block = "Ctrl+Enter"
ai_suggest = "Ctrl+Space"
search = "Ctrl+F"
```

---

## 4. Key Features Implementation

### 4.1 Block Workflow

**User Flow:**
1. User types command in input area
2. Press Enter → creates new block, executes command
3. Output streams into block in real-time
4. Block shows completion status, exit code, duration
5. User can collapse, copy, edit, or re-run blocks

**Block Operations:**
- **Copy**: Copy command only, output only, or both
- **Edit**: Modify historical command and re-run
- **Collapse**: Hide output to save screen space
- **Delete**: Remove block from session
- **AI Assist**: Get explanation or suggestions for command

---

### 4.2 Command Completion & Suggestions

**Sources:**
- Shell builtin completion (bash completion)
- Command history
- AI-powered suggestions based on context
- MCP server suggestions

**Implementation:**
- Parse partial input
- Query multiple sources in parallel
- Rank and deduplicate suggestions
- Display in popup with preview

---

### 4.3 Syntax Highlighting

**Highlight:**
- Shell commands (keywords, builtins, paths)
- Command output (errors in red, warnings in yellow)
- Code blocks in output (auto-detect language)

**Technologies:**
- `syntect` with TextMate grammar bundles
- Custom shell syntax highlighting
- ANSI color code support

---

### 4.4 Search Functionality

**Search Capabilities:**
- Search across all blocks in session
- Filter by command or output
- Regex support
- Case-sensitive/insensitive options
- Navigate results with keyboard

---

### 4.5 Multi-pane Support

**Layout Options:**
- Horizontal split
- Vertical split
- Grid layouts
- Draggable pane separators
- Each pane has independent session

---

### 4.6 Themes

**Built-in Themes:**
- Dark (default)
- Light
- High contrast
- Custom user themes

**Themeable Elements:**
- Background, foreground colors
- Syntax highlighting colors
- UI element colors
- Block borders and states

---

## 5. Data Flow Examples

### 5.1 Command Execution Flow

```
User Input → Input Handler → Block Manager (create block)
                                   ↓
                            Shell Executor (execute)
                                   ↓
                            Stream Output → Block Manager (update)
                                   ↓
                            GUI Renderer (display)
```

### 5.2 AI Suggestion Flow

```
User Request (Ctrl+Space) → AI Engine → Select Provider
                                           ↓
                                    Build Context (current block, history)
                                           ↓
                                    Call LLM API (streaming)
                                           ↓
                                    Stream Response → GUI (display)
```

### 5.3 MCP Server Interaction

```
Application Start → MCP Manager → Load Config
                                     ↓
                              Start Auto-start Servers
                                     ↓
                              Establish Connections
                                     
User Request → AI Engine → Query MCP → MCP Manager → Route to Server
                                                          ↓
                                                    Get Response
                                                          ↓
                                                    Return to AI
```

---

## 6. File Structure

```
immaterium/
├── Cargo.toml
├── README.md
├── DESIGN.md
├── MILESTONES.md
├── LICENSE
├── .gitignore
├── config/
│   ├── default.toml
│   └── themes/
│       ├── dark.toml
│       └── light.toml
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── app.rs              # Main application state
│   ├── config/
│   │   ├── mod.rs
│   │   ├── loader.rs
│   │   └── schema.rs
│   ├── ui/
│   │   ├── mod.rs
│   │   ├── app.rs
│   │   ├── block_renderer.rs
│   │   ├── input_handler.rs
│   │   ├── theme.rs
│   │   ├── layout.rs
│   │   └── widgets/
│   │       ├── mod.rs
│   │       ├── command_input.rs
│   │       ├── output_viewer.rs
│   │       └── block.rs
│   ├── core/
│   │   ├── mod.rs
│   │   ├── block.rs
│   │   ├── session.rs
│   │   └── history.rs
│   ├── shell/
│   │   ├── mod.rs
│   │   ├── executor.rs
│   │   ├── parser.rs
│   │   └── process.rs
│   ├── ai/
│   │   ├── mod.rs
│   │   ├── engine.rs
│   │   ├── provider.rs
│   │   ├── providers/
│   │   │   ├── mod.rs
│   │   │   ├── ollama.rs
│   │   │   ├── openai.rs
│   │   │   └── groq.rs
│   │   └── context.rs
│   ├── mcp/
│   │   ├── mod.rs
│   │   ├── manager.rs
│   │   ├── server.rs
│   │   ├── protocol.rs
│   │   └── client.rs
│   └── utils/
│       ├── mod.rs
│       ├── syntax.rs
│       └── keybindings.rs
├── tests/
│   ├── integration/
│   └── unit/
└── assets/
    ├── fonts/
    └── icons/
```

---

## 7. Technology Stack

### Core Dependencies
- **egui** (0.28+) - GUI framework
- **eframe** (0.28+) - Application framework
- **tokio** (1.0+) - Async runtime
- **serde** (1.0+) - Serialization
- **toml** (0.8+) - Configuration parsing
- **sqlx** (0.7+) - Database (SQLite)
- **uuid** (1.0+) - Unique identifiers
- **chrono** (0.4+) - Date/time handling

### Shell Execution
- **portable-pty** (0.8+) - PTY handling
- **nix** (0.27+) - Unix system calls

### AI Integration
- **reqwest** (0.11+) - HTTP client
- **tokio-stream** (0.1+) - Streaming
- **async-openai** (0.20+) - OpenAI client
- Custom clients for Ollama/Groq

### Syntax & Parsing
- **syntect** (5.0+) - Syntax highlighting
- **tree-sitter** (optional) - Code parsing

### Additional
- **anyhow** (1.0+) - Error handling
- **tracing** (0.1+) - Logging
- **directories** (5.0+) - Platform paths

---

## 8. Security Considerations

1. **Command Execution**: Validate and sanitize commands to prevent injection
2. **API Keys**: Store securely, support environment variables
3. **MCP Servers**: Validate server configurations, sandbox execution
4. **Session Files**: Encrypt sensitive data in saved sessions
5. **Network**: Validate TLS certificates for API calls

---

## 9. Performance Considerations

1. **Async Execution**: All I/O operations are async (tokio)
2. **Streaming**: Stream command output and AI responses
3. **Virtual Scrolling**: Render only visible blocks
4. **Lazy Loading**: Load historical sessions on demand
5. **Syntax Highlighting**: Cache highlighted content
6. **Resource Limits**: Limit output size per block, max blocks per session

---

## 10. Future Enhancements

### Phase 2 Features
- Plugin system for extensions
- Remote session support (SSH)
- Collaborative sessions
- Cloud sync for configurations
- Mobile companion app
- More LLM providers (Claude, Gemini, local models)

### Phase 3 Features
- Built-in script editor
- Command workflow automation
- Visual programming interface
- Integrated documentation browser
- Performance profiling tools

---

## 11. Testing Strategy

### Unit Tests
- Block management logic
- Command parsing
- Configuration loading
- AI context building

### Integration Tests
- Shell command execution
- AI provider communication
- MCP server lifecycle
- Session persistence

### End-to-End Tests
- Complete user workflows
- Multi-pane operations
- Theme switching
- Export/import sessions

### Performance Tests
- Large output handling
- Many blocks in session
- Concurrent command execution
- Memory usage profiling

---

## 12. Accessibility

- Keyboard navigation for all features
- Screen reader support (where applicable in egui)
- Configurable font sizes
- High contrast themes
- Customizable keybindings

---

## 13. Documentation Requirements

1. **User Documentation**
   - Installation guide
   - Quick start tutorial
   - Feature documentation
   - Configuration reference
   - Keybinding reference

2. **Developer Documentation**
   - Architecture overview
   - API documentation (rustdoc)
   - Contributing guidelines
   - Plugin development guide (future)

3. **Examples**
   - Sample configurations
   - AI prompt templates
   - MCP server setups
   - Custom themes

---

## 14. Release Strategy

- **Alpha**: Core terminal + basic blocks
- **Beta**: AI integration + MCP support
- **RC**: All features, bug fixes
- **v1.0**: Stable release with documentation
- **Post-1.0**: Regular updates, community features

---

*End of Design Document*
