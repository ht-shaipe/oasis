# 架构设计

Oasis 采用 **Tauri 2 + Vue 3** 的混合架构，Rust workspace 按功能拆分为 13 个独立 crate。

## 整体架构

```
┌──────────────────────────────────────────────────────┐
│                    Vue 3 Frontend                     │
│  ┌──────────┐ ┌───────────┐ ┌────────────────────┐   │
│  │  Apps/   │ │Components │ │Stores / Composables│   │
│  │ 14 apps  │ │ system +  │ │  Pinia / i18n      │   │
│  │          │ │ common    │ │                    │   │
│  └────┬─────┘ └───────────┘ └────────────────────┘   │
│       │  invoke() / listen()                          │
├───────┼──────────────────────────────────────────────┤
│       │          Tauri 2 IPC Bridge                   │
├───────┼──────────────────────────────────────────────┤
│       ▼           Rust Backend                        │
│  ┌────────────────────────────────────────────────┐   │
│  │       lib.rs (setup / tray / code-gen cmd)     │   │
│  │  ┌──────────────┐ ┌────────────────────┐       │   │
│  │  │  commands.rs │ │      net/           │       │   │
│  │  │ greet / tray │ │ client / proxy /    │       │   │
│  │  │  / update    │ │ config / response   │       │   │
│  │  └──────────────┘ └────────────────────┘       │   │
│  │                                                 │   │
│  │  crates/ (13 个功能 crate)                      │   │
│  │  ┌──────────────┐ ┌───────────────┐            │   │
│  │  │ credential   │ │  toolbox      │            │   │
│  │  │ Rusqlite+Ring│ │  CSV/Excel/   │            │   │
│  │  │ AES-GCM 加密 │ │  JSON/Network │            │   │
│  │  └──────────────┘ └───────────────┘            │   │
│  │  ┌──────────────┐ ┌───────────────┐            │   │
│  │  │ browser      │ │browser-data-  │            │   │
│  │  │ Chrome CDP   │ │extract        │            │   │
│  │  └──────────────┘ └───────────────┘            │   │
│  │  ┌──────┐┌──────┐┌──────┐┌────────┐┌────────┐  │   │
│  │  │  ai  ││ chat ││embed ││knowledge││ agent  │  │   │
│  │  └──────┘└──────┘└──────┘└────────┘└────────┘  │   │
│  │  ┌────────────┐┌────────┐┌─────┐┌──────────┐   │   │
│  │  │agent-config││project ││ hub ││local-llm │   │   │
│  │  └────────────┘└────────┘└─────┘└──────────┘   │   │
│  └────────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────┘
```

## 分层说明

### 1. 前端层 (Vue 3 + TypeScript)

| 层级 | 目录 | 说明 |
|------|------|------|
| 应用组件 | `src/apps/` | 14 个内置应用，通过动态 `<component :is>` 渲染到窗口 |
| 系统组件 | `src/components/system/` | 桌面 Shell 组件（MenuBar / Dock / DesktopIcons / ContextMenu / LoginScreen / NotificationCenter / Calendar / SignInModal / UpdateDialog） |
| 通用组件 | `src/components/common/` | MacWindow / AppDialog |
| 桌面编排 | `src/views/HomeView.vue` | 核心调度：窗口状态管理、应用间事件路由、桌面右键菜单处理 |
| 状态管理 | `src/store/` | Pinia store：locale / theme / fontSize / agent / chat |
| 国际化 | `src/locales/` | zh-CN.json / en.json / index.ts |
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
| 核心库 | `src-tauri/src/lib.rs` | `setup()` 初始化 + `setup_tray()` 系统托盘 + 命令注册（code-generated） |
| 通用命令 | `src-tauri/src/commands.rs` | `greet` / `update_tray_locale` / `check_update` |
| 网络代理 | `src-tauri/src/net/` | API 代理（按服务名路由）+ 统一响应体 |

**命令注册（自动生成）**：

`build.rs` 扫描所有 crate 中 `#[tauri::command]` 注解，生成 `generated_invoke_handler.rs`，`lib.rs` 通过 `include!()` 引入：

```rust
.invoke_handler(include!(concat!(env!("OUT_DIR"), "/generated_invoke_handler.rs")))
```

不要手动在 `lib.rs` 中注册命令，只需在 crate 的 `commands.rs` 中添加 `#[tauri::command]` 函数即可。

### 3. 功能 Crate

| Crate | 说明 |
|-------|------|
| `oasis-credential` | 凭据加密存储：SQLite + Ring AES-256-GCM，主密钥 PBKDF2 派生 |
| `oasis-toolbox` | CSV 统计/拆分/转换、Excel 移动预览/应用、JSON 转换/合并、网络扫描 |
| `oasis-browser` | Chrome CDP 路径探测与启动 |
| `oasis-browser-data-extract` | 浏览器数据提取（密码/Cookie/书签/历史/下载/信用卡/扩展） |
| `oasis-ai` | AI 功能 |
| `oasis-chat` | 聊天功能 |
| `oasis-embed` | 嵌入模型 |
| `oasis-knowledge` | RAG 知识库 |
| `oasis-agent` | Agent 运行时与插件系统 |
| `oasis-agent-config` | Agent 配置管理 |
| `oasis-project` | 项目管理 |
| `oasis-hub` | Hub 服务 |
| `oasis-local-llm` | 本地 LLM 集成 |

### 4. 数据持久化

| 数据类型 | 存储方式 | 说明 |
|----------|----------|------|
| 应用偏好 | localStorage | 侧边栏宽度、视图模式、排序方式等 |
| 用户配置 | 本地 JSON | 代理配置 `proxy.toml` |
| 凭据数据 | SQLite (bundled) | Ring AES-256-GCM 加密，主密钥 PBKDF2 派生 |
| Agent 运行时 | Tauri managed state | `Mutex<AgentRegistry>` + `Mutex<HashMap<String, AgentProcess>>` |

## Tauri 配置要点

| 项目 | 配置值 |
|------|--------|
| 窗口 | 1400×1000，居中，Overlay 标题栏 (hiddenTitle) |
| 安全 | `macOSPrivateApi: true`，CSP: null |
| 权限 | core:default / opener:default / dialog / event / window / start-dragging / show / hide / close |
| 打包 | app + dmg |
| Dev 端口 | 1488 |
