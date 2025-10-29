# 🚀 Immaterium Terminal

A modern, AI-powered terminal emulator inspired by Warp, built with Rust and egui. Immaterium combines traditional shell functionality with intelligent automation, block-based workflows, and Model Context Protocol (MCP) server integration.

![Milestone](https://img.shields.io/badge/Milestone-1-blue)
![License](https://img.shields.io/badge/license-MIT-green)
![Rust](https://img.shields.io/badge/rust-1.70%2B-orange)

## ✨ Features

- **🧱 Block-based Workflow**: Each command and its output is a discrete, manageable block
- **🤖 AI Integration**: Support for multiple LLM providers (Ollama, Groq, OpenAI)
- **🔌 MCP Server Support**: Spawn and manage Model Context Protocol servers
- **🎨 Modern UI**: Built with egui for a responsive, native experience
- **💾 Session Persistence**: Save and restore terminal sessions
- **🎯 Command Completion**: Intelligent suggestions from multiple sources
- **🌈 Syntax Highlighting**: Beautiful code highlighting for commands and output
- **⚡ Multi-pane Support**: Split your terminal horizontally or vertically
- **⌨️ Customizable Keybindings**: Configure shortcuts to match your workflow
- **🎭 Themes**: Multiple built-in themes plus custom theme support

## 🏗️ Project Status

**Current Milestone**: 1 - Project Foundation & Basic GUI ✅

This project is under active development. See [MILESTONES.md](MILESTONES.md) for the complete roadmap and [DESIGN.md](DESIGN.md) for architectural details.

## 🚀 Quick Start

### Prerequisites

- Rust 1.70 or higher
- Linux (initial target platform)
- Optional: Ollama for local AI model support

### Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/immaterium.git
cd immaterium

# Build and run
cargo run --release
```

### Development Build

```bash
# Build in debug mode
cargo build

# Run with logging
RUST_LOG=immaterium=debug cargo run
```

## 📖 Configuration

Immaterium stores its configuration in `~/.config/immaterium/config.toml`. On first run, a default configuration will be created.

### Example Configuration

```toml
[general]
default_shell = "/bin/bash"
save_history = true
max_history_size = 10000

[appearance]
theme = "dark"
font_family = "monospace"
font_size = 14.0

[ai]
default_provider = "ollama"
enable_suggestions = true

[ai.providers.ollama]
base_url = "http://localhost:11434"
model = "codellama"
enabled = true

[keybindings]
new_block = "Ctrl+Enter"
ai_suggest = "Ctrl+Space"
search = "Ctrl+F"
```

See [config/default.toml](config/default.toml) for all available options.

## 🎯 Usage

### Basic Commands

- **Enter**: Execute command (creates a new block)
- **Ctrl+Space**: Trigger AI command suggestion
- **Ctrl+F**: Search across all blocks
- **Ctrl+R**: Open command history
- **Ctrl+Shift+H**: Split pane horizontally
- **Ctrl+Shift+V**: Split pane vertically

### AI Features (Coming Soon)

```bash
# Get AI suggestions for commands
Ctrl+Space

# Explain a command
Right-click on block → "Explain Command"

# Fix errors automatically
Error in output → AI suggests fixes
```

### MCP Servers (Coming Soon)

Configure MCP servers in your `config.toml`:

```toml
[[mcp.servers]]
name = "filesystem"
command = "mcp-server-filesystem"
args = ["/home/user"]
auto_start = true
```

## 🏛️ Architecture

```
immaterium/
├── src/
│   ├── main.rs           # Application entry point
│   ├── lib.rs            # Library root
│   ├── config/           # Configuration management
│   ├── ui/               # GUI layer (egui)
│   ├── core/             # Block and session management
│   ├── shell/            # Shell execution
│   ├── ai/               # AI engine and providers
│   ├── mcp/              # MCP server management
│   └── utils/            # Utilities
├── config/               # Default configurations
└── assets/               # Icons and resources
```

See [DESIGN.md](DESIGN.md) for detailed architecture documentation.

## 🛣️ Roadmap

### Completed Milestones

- ✅ **M1**: Project Foundation & Basic GUI

### In Progress

- 🚧 **M2**: Shell Executor & Basic Terminal

### Upcoming

- **M3**: Block System Implementation
- **M4**: Session Management & Persistence
- **M5**: Syntax Highlighting & Themes
- **M6**: AI Integration - Core Engine
- **M7**: AI Features - Commands & Suggestions
- **M8**: MCP Server Integration

See [MILESTONES.md](MILESTONES.md) for the complete development plan.

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run with logging
RUST_LOG=debug cargo test

# Run specific test
cargo test test_name
```

## 🤝 Contributing

Contributions are welcome! Please read our contributing guidelines (coming soon) before submitting PRs.

### Development Setup

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run tests (`cargo test`)
5. Run formatting (`cargo fmt`)
6. Run linting (`cargo clippy`)
7. Commit your changes (`git commit -m 'Add amazing feature'`)
8. Push to the branch (`git push origin feature/amazing-feature`)
9. Open a Pull Request

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Inspired by [Warp Terminal](https://www.warp.dev/)
- Built with [egui](https://github.com/emilk/egui) - an excellent immediate mode GUI library
- Uses [Model Context Protocol](https://modelcontextprotocol.io/) for extensibility

## 📬 Contact

- Issues: [GitHub Issues](https://github.com/yourusername/immaterium/issues)
- Discussions: [GitHub Discussions](https://github.com/yourusername/immaterium/discussions)

---

**Note**: This project is in early development. Features are being actively implemented according to the milestone plan.
