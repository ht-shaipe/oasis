# Oasis

macOS-style desktop efficiency platform built with Tauri v2 + Vue 3 + Rust.

## Commands

```bash
bun install          # install JS deps (uses Bun, not npm/yarn)
bun run dev          # Vite dev server only (port 1420, HMR port 1421)
bun run build        # vue-tsc --noEmit && vite build (frontend only)
bun run tauri        # tauri dev (starts Vite + Rust backend)
bun run tauri:build  # tauri build (frontend build + Rust compile + bundle)
```

`make` shortcuts: `make dev`, `make build`, `make bundle`, `make install`

Build script: `scripts/build.sh [web|tauri|all]`

## Architecture

**Frontend** (`src/`): Vue 3 Composition API + Element Plus + Pinia + vue-i18n + vue-router. Entry: `src/main.ts`. Path alias `@` → `src/`.

**Backend** (`src-tauri/`): Rust workspace with 5 sub-crates:
- `crates/credential` — encrypted credential storage (SQLite + Ring AES-GCM)
- `crates/toolbox` — CSV/Excel/JSON tools + network scanner
- `crates/browser` — Chrome CDP launch & control
- `crates/ai` — AI features
- `crates/browser-data-extract` — browser data extraction

**Tauri command registration is code-generated**: `build.rs` scans `#[tauri::command]` annotations in each crate's `commands.rs` and writes `generated_invoke_handler.rs` into `OUT_DIR`. `lib.rs` includes this file — do NOT manually register handlers.

**Local dependency**: `tube` crate is referenced at `../../../../rust/kit/tube` (relative path outside this repo). If `cargo build` fails on `tube`, check that path exists.

## Conventions

- **UnoCSS attribute mode**: write utility classes as HTML attributes, not class strings (via `presetAttributify` in `uno.config.ts`)
- **Minimum font size**: 13px — do not use smaller text
- **App registration**: all built-in apps are declared in `src/config/apps.ts` with id/name/icon/component/dock/desktop flags
- **Window management**: `HomeView.vue` manages app windows via `windowStates` reactive object + `<Teleport to="body">`
- **i18n**: use `nameKey` from app config with vue-i18n; locale files in `src/locales/`

## Key paths

- Tauri config: `src-tauri/tauri.conf.json` (window 1400x1000, Overlay titleBar, macOS private API enabled)
- TypeScript: strict mode, ES2021 target, `noUnusedLocals`/`noUnusedParameters` on
- Rust edition: 2024
- Build output: `src-tauri/target/release/bundle/` (macOS: .app + .dmg)
