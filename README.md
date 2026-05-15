# GPUI Template

A cargo-generate template for creating GPUI desktop applications with optional WASM support.

## Features

- 🖥️ **Desktop App** - GPUI framework with native window management
- 🌐 **Web (WASM)** - Optional WASM build with Vite
- 🎨 **Theme System** - Built-in light/dark theme support
- 🌍 **i18n** - Internationalization with rust-i18n (en, zh-CN included)
- 📦 **System Tray** - Cross-platform system tray support
- ⚙️ **Settings Panel** - Pre-built settings UI with theme/font/language options
- 🔧 **Dock Panels** - Left/Center/Right/Bottom panel layout

## Usage

### Install cargo-generate

```bash
cargo install cargo-generate
```

### Create a new project

```bash
cargo generate --git https://github.com/ht-shaipe/gpui-template --name my-app
```

Or from a local path:

```bash
cargo generate --path /path/to/gpui-template --name my-app
```

### Configure GitHub Update URL

After generating, update the GitHub URL in `src/core/updater/checker.rs`:

```rust
check_url: "https://api.github.com/repos/th-shaipe/YOUR_REPO/releases/latest"
```

## Desktop

```bash
cd my-app
cargo run
```

Requires default Cargo features (`native-app`) for the desktop binary (system tray, etc.).

## Web (WASM + Vite)

**Prerequisites:** Rust **nightly**, `wasm32-unknown-unknown` target, `wasm-bindgen-cli` **0.2.121**, [Bun](https://bun.sh/).

```bash
make install-web   # nightly target + wasm-bindgen-cli + www deps
make dev-web       # debug WASM + Vite on http://localhost:3000
```

Or manually:

```bash
./scripts/build-wasm.sh          # or ./scripts/build-wasm.sh --release
cd www && bun install && bun run dev
```

WASM build uses `--no-default-features --lib` so the desktop binary is not linked for `wasm32`.

## Template Placeholders

This template uses the following placeholders:

| Placeholder               | Description               | Example                      |
| ------------------------- | ------------------------- | ---------------------------- |
| `oasis`        | Project name (user input) | `my-app`                     |
| `oasis`          | Crate name (snake_case)   | `my_app`                     |
| `shaipe`         | Author name               | `Your Name`                  |
| `shaipe@sina.com`        | Author email              | `you@example.com`            |
| `一个专为开发者打造的一站式效率工具箱，让你专注创造，而非琐碎` | Project description       | `A GPUI desktop application` |

## Project Structure

```
my-app/
├── Cargo.toml           # Package configuration
├── cargo-generate.toml  # Template configuration
├── src/
│   ├── lib.rs          # Library entry + WASM init
│   ├── main.rs         # Desktop binary entry
│   ├── app/            # App-level modules
│   │   ├── actions.rs  # Action definitions
│   │   ├── app_menus.rs # Menu bar
│   │   ├── app_state.rs # Global state
│   │   ├── themes.rs   # Theme management
│   │   ├── title_bar.rs # Custom title bar
│   │   └── system_tray.rs # System tray
│   ├── panels/         # UI panels
│   │   ├── center_panel.rs
│   │   ├── left_panel.rs
│   │   ├── right_panel.rs
│   │   └── bottom_panel.rs
│   ├── core/           # Core utilities
│   │   └── updater/    # Update checker
│   └── workspace.rs    # Main workspace
├── locales/            # i18n translations
│   ├── en.yml
│   └── zh-CN.yml
└── www/                # WASM web assets
```

## License

MIT
