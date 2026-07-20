# Oasis

> 基于 Tauri v2 + Vue 3 构建的 macOS 风格桌面效率平台。

Oasis 是一个模拟 macOS 桌面环境的桌面应用，集成 Finder、代码编辑器、AI 对话、工具箱、凭据管理器等生产力工具，结合 Rust 高性能后端与 Vue 3 响应式前端。

---

## 核心特性

| 模块                | 说明                                                                                                     |
| ------------------- | -------------------------------------------------------------------------------------------------------- |
| 仿 macOS 桌面 Shell | MenuBar 顶部菜单栏、Dock 底部程序坞、DesktopIcons 桌面图标、ContextMenu 右键菜单，三种视图模式与两种排序 |
| 多窗口管理          | 14 个内置应用支持独立窗口，可最小化/还原，通过 Teleport 渲染到 body                                      |
| 系统托盘            | 窗口显隐控制                                                                                             |
| 凭据管理器          | SQLite 持久化 + Ring AES-GCM 加密，主密钥 PBKDF2 派生，支持分类管理、网站账号管理、浏览器数据导入         |
| 工具箱              | CSV 统计/拆分/转换、Excel 行列移动、JSON 转换/合并、网络端口扫描                                         |
| 浏览器控制          | Chrome CDP 路径探测与启动                                                                                |
| 浏览器数据提取      | 支持 Chrome/Edge/Firefox/Safari 等浏览器的密码、Cookie、书签、历史记录提取                               |
| 代码编辑器          | Monaco Editor 集成，支持代码生成联动                                                                     |
| AI 对话             | 多模型 AI Chat + Agent 模式                                                                              |
| 知识库              | RAG 知识检索与嵌入                                                                                       |

---

## 技术栈

| 层         | 技术                                       |
| ---------- | ------------------------------------------ |
| 前端框架   | Vue 3.5 (Composition API) + TypeScript 6.0 |
| 构建工具   | Vite 8 + Bun                               |
| UI 库      | Element Plus 2.14 + UnoCSS 66.7            |
| 状态管理   | Pinia 3.0                                  |
| 国际化     | vue-i18n 11.4                              |
| 路由       | vue-router 5.0                             |
| 代码编辑器 | Monaco Editor (`@monaco-editor/loader`)    |
| 桌面框架   | Tauri 2                                    |
| 后端语言   | Rust (edition 2024)                        |
| 数据库     | SQLite (rusqlite, bundled)                 |
| 加密       | Ring 0.17 (AES-GCM)                        |
| 包管理     | Bun                                        |

---

## 项目结构

```text
oasis/
├── src/                              # 前端源码
│   ├── main.ts                       # 入口：挂载 Pinia / Router / i18n / Element Plus
│   ├── App.vue                       # 根组件：el-config-provider + RouterView
│   ├── views/
│   │   └── HomeView.vue              # 桌面 Shell 主视图（窗口管理/事件路由）
│   ├── apps/                         # 内置应用组件 (14 个)
│   │   ├── Finder.vue                # 文件资源管理（旧版）
│   │   ├── Finder/                   # 文件资源管理（新版目录）
│   │   ├── Generator.vue             # 代码生成器
│   │   ├── CodeEditor.vue            # Monaco 代码编辑器
│   │   ├── Safari.vue                # 内置网页预览
│   │   ├── Browser.vue               # 浏览器 CDP 控制器
│   │   ├── Notes.vue                 # 备忘录
│   │   ├── Profile.vue               # 用户资料
│   │   ├── ContinueDialog.vue        # AI 续写对话框
│   │   ├── Settings/                 # 设置面板
│   │   │   ├── index.vue
│   │   │   └── panels/              # AboutPanel / GeneralPanel / AppearancePanel /
│   │   │                             # AgentConfigPanel / LlmPanel / EmbeddingModelPanel
│   │   ├── Credential/               # 凭据管理器 (12 文件)
│   │   │   ├── Index.vue / Sidebar.vue / Toolbar.vue
│   │   │   ├── AuthCard.vue / SiteAccountManager.vue / SiteList.vue
│   │   │   ├── CredentialFormDialog.vue / CredentialTable.vue
│   │   │   ├── BrowserImportDialog.vue / MergePreviewDialog.vue
│   │   │   ├── TemplateManager.vue / credentialForm.ts
│   │   ├── Toolbox/                  # 工具箱
│   │   │   ├── Index.vue / Sidebar.vue
│   │   │   ├── tools/               # CsvTool / ExcelMove / JsonConvert / JsonMerge / NetworkScan
│   │   │   ├── composables/ / constants.ts / types.ts
│   │   ├── Chat/                     # AI 对话
│   │   │   ├── Index.vue / ChatView.vue / components/
│   │   ├── AgentChat/               # AI Agent
│   │   │   ├── Index.vue / AgentChatView.vue / components/
│   │   │   └── Knowledge/           # 知识库
│   │       └── Index.vue
│   ├── components/                   # 系统组件
│   │   ├── system/                   # 桌面 Shell 组件 (9 个)
│   │   │   ├── MenuBar.vue / Dock.vue / DesktopIcons.vue / ContextMenu.vue
│   │   │   ├── LoginScreen.vue / NotificationCenter.vue / Calendar.vue
│   │   │   ├── SignInModal.vue / UpdateDialog.vue
│   │   ├── common/                   # 通用组件 (2 个)
│   │   │   ├── MacWindow.vue / AppDialog.vue
│   ├── config/
│   │   ├── apps.ts                   # 应用注册表 (id/name/icon/component/dock/desktop)
│   │   └── menuBar.ts                # 菜单栏组件显隐配置
│   ├── store/
│   │   ├── locale.ts                 # 语言 Pinia store
│   │   ├── theme.ts                  # 主题 Pinia store
│   │   ├── fontSize.ts              # 字号 Pinia store
│   │   ├── agent.ts                 # Agent Pinia store
│   │   └── chat.ts                  # Chat Pinia store
│   ├── locales/
│   │   ├── zh-CN.json               # 简体中文
│   │   ├── en.json                   # English
│   │   └── index.ts                  # i18n 初始化
│   ├── composables/
│   │   ├── useCredential.ts          # 凭据管理 composable
│   │   ├── useAppUpdate.ts           # 应用更新 composable
│   │   └── useFileDialog.ts          # 文件对话框 composable
│   ├── utils/
│   │   ├── apiService.ts             # API 服务封装
│   │   ├── request.ts                # HTTP 请求工具
│   │   ├── mockData.ts               # Mock 数据
│   │   └── sseChat.ts                # SSE 聊天工具
│   ├── styles/
│   │   └── theme.css                 # 主题样式变量
│   └── router/
│       └── index.ts                  # 路由 (仅 / → HomeView)
├── public/assets/
│   ├── icons/                        # SVG 图标
│   ├── logo.png
│   ├── profile.jpg
│   └── wallpaper/
├── src-tauri/                        # Rust 后端
│   ├── src/
│   │   ├── main.rs                   # 程序入口
│   │   ├── lib.rs                    # 核心库：setup() / setup_tray() / 命令注册（code-generated）
│   │   ├── commands.rs               # 通用命令 (greet / update_tray_locale / check_update)
│   │   ├── tary_icon.rs              # 托盘图标
│   │   └── net/                      # 网络代理模块
│   │       ├── client.rs             # HTTP 客户端
│   │       ├── config.rs             # 代理配置 (proxy.toml)
│   │       ├── proxy.rs              # 代理管理（路径路由）
│   │       └── response.rs           # 统一响应体
│   ├── crates/                       # 子 crate (13 个)
│   │   ├── oasis-credential/         # 凭据管理 (加密存储)
│   │   ├── oasis-toolbox/            # 工具箱 (CSV/Excel/JSON/网络扫描)
│   │   ├── oasis-browser/            # 浏览器 CDP 控制
│   │   ├── oasis-browser-data-extract/ # 浏览器数据提取
│   │   ├── oasis-ai/                 # AI 功能
│   │   ├── oasis-chat/               # 聊天功能
│   │   ├── oasis-embed/              # 嵌入模型
│   │   ├── oasis-knowledge/          # 知识库
│   │   ├── oasis-agent/              # Agent 运行时
│   │   ├── oasis-agent-config/       # Agent 配置
│   │   ├── oasis-project/            # 项目管理
│   │   ├── oasis-hub/                # Hub 服务
│   │   └── oasis-local-llm/          # 本地 LLM
│   ├── build.rs                      # 构建脚本：扫描 #[tauri::command] 生成命令注册
│   ├── tauri.conf.json               # Tauri 配置 (窗口/安全/打包)
│   ├── capabilities/default.json     # 权限声明
│   └── Cargo.toml                    # Rust workspace 配置
├── docs/                             # 详细技术文档
│   ├── getting-started.md
│   ├── architecture.md
│   ├── credential-storage.md
│   ├── credential-backend-spec.md
│   ├── credential-frontend-spec.md
│   └── UPDATE_RELEASE_GUIDE.md
├── scripts/                          # 构建脚本
├── vite.config.ts
├── tsconfig.json
├── uno.config.ts
├── Makefile
└── package.json
```

---

## 内置应用

| 应用           | ID                   | Dock  | 桌面  | 说明                                   |
| -------------- | -------------------- | :---: | :---: | -------------------------------------- |
| Finder         | `Finder`             |   ✅   |       | 文件资源管理与版本控制                 |
| Generator      | `generator`          |   ✅   |       | 代码片段生成，联动 CodeEditor + Safari |
| CodeEditor     | `editor`             |   ✅   |       | Monaco Editor，支持多语言编辑          |
| Safari         | `safari`             |   ✅   |       | 内置网页预览                           |
| Browser        | `browser`            |   ✅   |       | Chrome CDP 启动与控制                  |
| Toolbox        | `toolbox`            |   ✅   |   ✅   | 多合 1 工具箱                          |
| Credential     | `credential-manager` |   ✅   |   ✅   | 加密凭据管理                           |
| Settings       | `settings`           |   ✅   |   ✅   | 系统设置                               |
| Knowledge      | `knowledge`          |   ✅   |   ✅   | RAG 知识库                             |
| Chat           | `chat`               |   ✅   |   ✅   | AI 多模型对话                          |
| Agent Chat     | `agent-chat`         |   ✅   |   ✅   | AI Agent 模式                          |
| Notes          | `notes`              |       |   ✅   | 备忘录                                 |
| Profile        | `profile`            |       |       | 用户资料                               |
| ContinueDialog | `continue-dialog`    |       |       | AI 续写                                |

---

## Rust 后端

共 **13 个**子 crate，命令注册由 `build.rs` 自动扫描 `#[tauri::command]` 注解生成 `generated_invoke_handler.rs`，`lib.rs` 通过 `include!` 引入——**不要手动注册命令**。

| Crate                         | 功能                                                   |
| ----------------------------- | ------------------------------------------------------ |
| `oasis-credential`            | 凭据加密存储 (SQLite + Ring AES-GCM，主密钥 PBKDF2)    |
| `oasis-toolbox`               | CSV 统计/拆分/转换、Excel 移动预览/应用、JSON 转换/合并、网络扫描 |
| `oasis-browser`               | Chrome CDP 路径探测与启动                              |
| `oasis-browser-data-extract`  | 浏览器密码/Cookie/书签/历史记录/下载记录提取           |
| `oasis-ai`                    | AI 功能                                                |
| `oasis-chat`                  | 聊天功能                                               |
| `oasis-embed`                 | 嵌入模型                                               |
| `oasis-knowledge`             | RAG 知识库                                             |
| `oasis-agent`                 | Agent 运行时与插件系统                                 |
| `oasis-agent-config`          | Agent 配置管理                                         |
| `oasis-project`               | 项目管理                                               |
| `oasis-hub`                   | Hub 服务                                               |
| `oasis-local-llm`             | 本地 LLM                                               |

---

## 快速开始

### 前置要求

- [Rust](https://www.rust-lang.org/tools/install) stable
- [Bun](https://bun.sh/)
- macOS: Xcode Command Line Tools

### 开发

```bash
bun install
bun run tauri
# 或
make dev
```

Vite Dev Server 端口 1488。Rust 代码修改后自动重新编译，前端支持热重载。

### 构建

```bash
bun run tauri:build
# 或
make bundle
```

产物位于 `src-tauri/target/release/bundle/`（macOS: `.app` + `.dmg`）。

---

## 架构要点

```
Vue 3 Frontend  ──invoke()──▶  Tauri Commands  ──▶  Rust Crates
       ▲                            │
       │                            ▼
       ◀── listen() ────  Events (tray-action, etc.)
```

- **前端**：Vue 3 Composition API + Element Plus + Pinia，通过 `@tauri-apps/api/core` 的 `invoke` 调用后端命令。
- **后端**：Rust workspace，13 个功能 crate，命令注册由 `build.rs` 自动生成。
- **窗口管理**：`HomeView.vue` 通过 `windowStates` reactive 对象 + `<Teleport to="body">` 管理应用窗口的显隐与最小化状态。
- **数据流**：Generator 生成代码 → emit → HomeView 打开 CodeEditor + Safari；Finder 加载版本 → emit → 联动编辑器与预览。
- **安全**：凭据模块使用 Ring AES-256-GCM 加密，主密钥 PBKDF2 派生，存储在 SQLite (bundled)。

---

## 构建配置

| 配置       | 值                               |
| ---------- | -------------------------------- |
| Tauri 窗口 | 1400×1000，居中，Overlay 标题栏  |
| Dev 端口   | 1488                             |
| TypeScript | strict 全开，target ES2021       |
| 路径别名   | `@` → `./src`                    |
| Rust 版本  | edition 2024                     |
| 打包格式   | macOS: app + dmg                 |

---

## 本地依赖

`tube` crate 引用路径为 `../../../../rust/kit/tube`（项目外相对路径）。若 `cargo build` 报 `tube` 相关错误，请确认该路径存在。

---

## 文档

- [快速上手](docs/getting-started.md)
- [架构设计](docs/architecture.md)
- [凭据存储方案](docs/credential-storage.md)
- [凭据管理后端规范](docs/credential-backend-spec.md)
- [凭据管理前端规范](docs/credential-frontend-spec.md)
- [更新发布指南](docs/UPDATE_RELEASE_GUIDE.md)
- [浏览器数据提取](src-tauri/crates/browser-data-extract/README.md)

---

## 许可证

[MIT License](LICENSE)
