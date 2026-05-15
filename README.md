# Oasis

> 一个专为开发者打造的一站式效率工具箱，让你专注创造，而非琐碎

Oasis is a developer productivity tool built with GPUI, providing a comprehensive workspace for code editing, markdown writing, and workflow management.

## ✨ Features

### Core Functionality
- **🖥️ Multi-Panel Workspace** - Flexible dock-based layout with left, center, right, and bottom panels
- **📝 Code Editor** - Feature-rich code editing with syntax highlighting
- **📄 Markdown Editor** - Real-time markdown preview and editing
- **🧰 Toolbox Panel** - Quick access to developer utilities and tools
- **🔍 Search & Navigation** - Fast file search and workspace navigation

### User Experience
- **🎨 Theme System** - Beautiful light and dark themes with smooth transitions
- **🌍 Internationalization** - Full i18n support (English, Simplified Chinese)
- **⚙️ Settings Panel** - Comprehensive settings UI for themes, fonts, and languages
- **📦 System Tray** - Cross-platform system tray integration (desktop)
- **🔄 Auto-Updates** - Built-in update checker for seamless upgrades

### Technical Features
- **🚀 High Performance** - Built on GPUI for native-speed responsiveness
- **🌐 Web Support** - Optional WASM build for browser-based usage
- **🔧 Modular Architecture** - Clean separation of concerns with extensible panel system

## 🚀 Getting Started

### Desktop Application

**Prerequisites:** Rust stable toolchain

```bash
# Clone the repository
git clone <repository-url>
cd oasis

# Run the application
cargo run
```

### Web (WASM + Vite)

**Prerequisites:** Rust nightly, `wasm32-unknown-unknown` target, `wasm-bindgen-cli` 0.2.121, [Bun](https://bun.sh/)

```bash
# Install web dependencies
make install-web

# Start development server
make dev-web
# Or manually: ./scripts/build-wasm.sh && cd www && bun install && bun run dev
```

The application will be available at `http://localhost:3000`

## 📖 Usage

### Workspace Navigation

- **Toggle Panels**: Use menu items or keyboard shortcuts to show/hide left, right, and bottom panels
- **Panel Layout**: Drag and drop to rearrange panels according to your workflow
- **Settings**: Access settings via the menu bar or system tray to customize themes, fonts, and languages

### Code Editor

- **File Opening**: Use `Cmd/Ctrl+O` to open files
- **Syntax Highlighting**: Automatic detection based on file extension
- **Multi-file Editing**: Open multiple files in tabs

### Markdown Editor

- **Live Preview**: See formatted markdown in real-time
- **Rich Text Support**: Tables, code blocks, links, and more
- **Export Options**: Export to HTML or PDF (planned)

## 🛠️ Development

### Project Structure

```
oasis/
├── Cargo.toml              # Package configuration
├── src/
│   ├── lib.rs             # Library entry + WASM init
│   ├── main.rs            # Desktop binary entry
│   ├── app/               # Application-level modules
│   │   ├── actions.rs     # Action definitions
│   │   ├── app_menus.rs   # Menu bar configuration
│   │   ├── app_state.rs   # Global application state
│   │   ├── themes.rs      # Theme management
│   │   ├── title_bar.rs   # Custom title bar
│   │   └── system_tray.rs # System tray integration
│   ├── panels/            # UI panels
│   │   ├── center_panel.rs    # Main workspace panel
│   │   ├── left_panel.rs      # File explorer
│   │   ├── right_panel.rs     # Properties/details
│   │   ├── bottom_panel.rs    # Terminal/output
│   │   ├── code_editor/       # Code editing functionality
│   │   ├── markdown_editor/   # Markdown editing
│   │   └── toolbox_panel/     # Developer utilities
│   ├── core/              # Core utilities
│   │   └── updater/       # Update checking system
│   └── workspace.rs      # Main workspace container
├── locales/               # i18n translations
│   ├── en.yml
│   └── zh-CN.yml
└── assets/               # Icons and static assets
```

### Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# WASM build
./scripts/build-wasm.sh --release
```

### Configuration

Update the GitHub repository URL in `src/core/updater/checker.rs` for automatic updates:

```rust
check_url: "https://api.github.com/repos/your-username/oasis/releases/latest"
```

## 🎯 Roadmap

- [ ] Enhanced code editor with LSP integration
- [ ] Git integration and version control
- [ ] Extension system for plugins
- [ ] Cloud synchronization
- [ ] Collaboration features
- [ ] Performance optimizations
- [ ] Additional language support

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🙏 Acknowledgments

- Built with [GPUI](https://github.com/zed-industries/zed) - Framework by Zed Industries
- UI components from [gpui-component](https://github.com/ht-shaipe/gpui-component)
- Template based on [gpui-template](https://github.com/ht-shaipe/gpui-template)

## 📮 Contact

- Author: shaipe <shaipe@sina.com>
- Issues: [GitHub Issues](<repository-issues-url>)

---

Made with ❤️ by developers, for developers
