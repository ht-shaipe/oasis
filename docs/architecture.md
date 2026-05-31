---
AIGC:
    Label: "1"
    ContentProducer: 001191440300708461136T1XGW3
    ProduceID: c8bc0470d7bdfb9241ace971be1a0b2c_d5b1fc9d5c9011f1abc85254006c9bbf
    ReservedCode1: g2P0CcvO69MHGFHjdnSvcq7/f0mlMFOFjDvC35XdjMKoV8KiQrntFF/9fIkh0tHAsePs+bH4Uihr2dSwq4hJvLEfgdg76VDE8kUaLX8fx6nKCxoyhj+S41pOHHOkCeZqnz+8IFwjJ3NUSPVl31EK9RmjhBsq+CTZzblSZH9OhPrTSdgdgzjyqpjAoGI=
    ContentPropagator: 001191440300708461136T1XGW3
    PropagateID: c8bc0470d7bdfb9241ace971be1a0b2c_d5b1fc9d5c9011f1abc85254006c9bbf
    ReservedCode2: g2P0CcvO69MHGFHjdnSvcq7/f0mlMFOFjDvC35XdjMKoV8KiQrntFF/9fIkh0tHAsePs+bH4Uihr2dSwq4hJvLEfgdg76VDE8kUaLX8fx6nKCxoyhj+S41pOHHOkCeZqnz+8IFwjJ3NUSPVl31EK9RmjhBsq+CTZzblSZH9OhPrTSdgdgzjyqpjAoGI=
---

# 架构设计

Oasis 采用 **Tauri 2 + Vue 3** 的混合架构，Rust workspace 按功能拆分为 3 个独立 crate。

## 整体架构

```
┌────────────────────────────────────────────────┐
│                  Vue 3 Frontend                 │
│  ┌─────────┐ ┌──────────┐ ┌──────────────────┐ │
│  │  Apps/  │ │Components│ │ Stores / Composables│ │
│  │ 11 apps │ │ 9 system │ │  Pinia / i18n     │ │
│  └────┬────┘ └──────────┘ └──────────────────┘ │
│       │  invoke() / listen()                    │
├───────┼────────────────────────────────────────┤
│       │          Tauri 2 IPC Bridge             │
├───────┼────────────────────────────────────────┤
│       ▼           Rust Backend                  │
│  ┌───────────────────────────────────────────┐  │
│  │              lib.rs (setup / tray / cmd)  │  │
│  │  ┌──────────────┐ ┌────────────────────┐  │  │
│  │  │  commands.rs │ │      net/           │  │  │
│  │  │ greet / tray │ │ client / proxy /    │  │  │
│  │  │   locale     │ │ config / response   │  │  │
│  │  └──────────────┘ └────────────────────┘  │  │
│  │                                            │  │
│  │  crates/                                   │  │
│  │  ┌────────────────┐ ┌───────────────────┐  │  │
│  │  │ oasis-credential│ │  oasis-toolbox    │  │  │
│  │  │ 12 commands    │ │  9 commands        │  │  │
│  │  │ Rusqlite + Ring│ │  CSV/Excel/JSON/   │  │  │
│  │  │ AES-GCM 加密   │ │  Network Scan      │  │  │
│  │  └────────────────┘ └───────────────────┘  │  │
│  │  ┌────────────────┐                         │  │
│  │  │  oasis-browser │                         │  │
│  │  │  2 commands    │                         │  │
│  │  │  Chrome CDP    │                         │  │
│  │  └────────────────┘                         │  │
│  └───────────────────────────────────────────┘  │
└────────────────────────────────────────────────┘
```

## 分层说明

### 1. 前端层 (Vue 3 + TypeScript)

| 层级 | 目录 | 说明 |
|------|------|------|
| 应用组件 | `src/apps/` | 11 个内置应用，通过动态 `<component :is>` 渲染到窗口 |
| 系统组件 | `src/components/` | 桌面 Shell 组件（MenuBar / Dock / DesktopIcons / ContextMenu / MacWindow） |
| 桌面编排 | `src/views/HomeView.vue` | 核心调度：窗口状态管理、应用间事件路由、桌面右键菜单处理 |
| 状态管理 | `src/store/` | Pinia store：locale（语言）/ theme（主题） |
| 国际化 | `src/locales/` | zh-CN.json / en.json |
| 应用注册 | `src/config/apps.ts` | 定义每个应用的 id / name / icon / component / showInDock / showOnDesktop |

**窗口管理机制**：

- `HomeView.vue` 持有 `windowStates` reactive 对象，为每个应用维护 `{ show, isMinimized }`
- 应用窗口通过 `<Teleport to="body">` 渲染到 body 下，脱离组件层级
- 窗口组件使用 `MacWindow` 包装，提供拖拽标题栏和最小化交互

**应用间通信**：

- Generator 生成代码 → emit `updateGeneratedCode` → HomeView 打开 CodeEditor + Safari
- Finder 加载版本 → emit `handleLoadVersion` → HomeView 打开 Editor + Safari

### 2. 后端层 (Tauri 2 + Rust)

| 模块 | 路径 | 说明 |
|------|------|------|
| 核心入口 | `src-tauri/src/main.rs` | 程序入口，调用 `oasis_lib::run()` |
| 核心库 | `src-tauri/src/lib.rs` | `setup()` 初始化 + `setup_tray()` 系统托盘 + 所有 crate 命令注册 |
| 通用命令 | `src-tauri/src/commands.rs` | `greet` / `update_tray_locale` |
| 网络代理 | `src-tauri/src/net/` | API 代理（按服务名路由）+ 统一响应体 |

**命令注册宏**：

每个功能 crate 通过 `_handlers!()` 宏批量声明 Tauri 命令，在 `lib.rs` 中统一注册：

```rust
.invoke_handler(tauri::generate_handler![
    commands::greet,
    commands::update_tray_locale,
    oasis_credential::credential_handlers!(),   // 12 命令
    oasis_toolbox::toolbox_handlers!(),         // 9 命令
    oasis_browser::browser_handlers!(),         // 2 命令
])
```

### 3. 数据持久化

| 数据类型 | 存储方式 | 说明 |
|----------|----------|------|
| 应用偏好 | localStorage | 侧边栏宽度、视图模式、排序方式等 |
| 用户配置 | 本地 JSON | 代理配置 `proxy.toml` |
| 凭据数据 | SQLite (bundled) | Ring AES-256-GCM 加密，主密钥 PBKDF2 派生 |

## Tauri 配置要点

| 项目 | 配置值 |
|------|--------|
| 窗口 | 1200×800，居中，Overlay 标题栏 (hiddenTitle) |
| 安全 | `macOSPrivateApi: true`，CSP: null |
| 权限 | core:default / opener:default / event / window / start-dragging / show / hide / close |
| 打包 | app + dmg |
*（内容由AI生成，仅供参考）*
