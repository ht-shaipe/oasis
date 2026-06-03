---
AIGC:
    Label: "1"
    ContentProducer: 001191440300708461136T1XGW3
    ProduceID: c8bc0470d7bdfb9241ace971be1a0b2c_d4daaeb85c9011f1abc85254006c9bbf
    ReservedCode1: gxdfFPtU2kiT00pH98+bGauIw/dRpR4DPV6y+/ArKUYqQlJ2VjujRvdaZpCNshu/QW23VUUeWHPE0NTXd9CRXx0200KCymnPEdbOfCo87Vqd5LGMMABdSV6RjGPOl1AWDfwtvm4jONF2yJ1wTF+m3OCkt0ZT+ewzkdeFKfgVo4a/4VI1d1f7Zoq1LTY=
    ContentPropagator: 001191440300708461136T1XGW3
    PropagateID: c8bc0470d7bdfb9241ace971be1a0b2c_d4daaeb85c9011f1abc85254006c9bbf
    ReservedCode2: gxdfFPtU2kiT00pH98+bGauIw/dRpR4DPV6y+/ArKUYqQlJ2VjujRvdaZpCNshu/QW23VUUeWHPE0NTXd9CRXx0200KCymnPEdbOfCo87Vqd5LGMMABdSV6RjGPOl1AWDfwtvm4jONF2yJ1wTF+m3OCkt0ZT+ewzkdeFKfgVo4a/4VI1d1f7Zoq1LTY=
---

# Oasis

> 基于 Tauri v2 + Vue 3 构建的 macOS 风格桌面效率平台。

Oasis 是一个模拟 macOS 桌面环境的桌面应用，集成 Finder、代码编辑器、工具箱、凭据管理器等生产力工具，结合 Rust 高性能后端与 Vue 3 响应式前端。

---

## 核心特性

| 模块                | 说明                                                                                                     |
| ------------------- | -------------------------------------------------------------------------------------------------------- |
| 仿 macOS 桌面 Shell | MenuBar 顶部菜单栏、Dock 底部程序坞、DesktopIcons 桌面图标、ContextMenu 右键菜单，三种视图模式与两种排序 |
| 多窗口管理          | 11 个内置应用支持独立窗口，可最小化/还原，通过 Teleport 渲染到 body                                      |
| 系统托盘            | 中/英双语切换，窗口显隐控制                                                                              |
| 凭据管理器          | SQLite 持久化 + Ring AES-GCM 加密，主密钥 PBKDF2 派生，支持分类管理                                      |
| 工具箱              | CSV 统计/拆分/转换、Excel 行列移动、JSON 转换/合并、网络端口扫描                                         |
| 浏览器控制          | Chrome CDP 路径探测与启动                                                                                |
| 代码编辑器          | Monaco Editor 集成，支持代码生成联动                                                                     |

---

## 技术栈

| 层         | 技术                                       |
| ---------- | ------------------------------------------ |
| 前端框架   | Vue 3.5 (Composition API) + TypeScript 6.0 |
| 构建工具   | Vite 8 + Bun                               |
| UI 库      | Element Plus 2.14 + UnoCSS 66.7            |
| 状态管理   | Pinia 3.0 + Vuex 4.1                       |
| 国际化     | vue-i18n 11.4                              |
| 路由       | vue-router 5.0                             |
| 代码编辑器 | Monaco Editor (`@monaco-editor/loader`)    |
| 桌面框架   | Tauri 2                                    |
| 后端语言   | Rust (stable)                              |
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
│   ├── apps/                         # 内置应用组件 (11 个)
│   │   ├── Finder.vue                # 文件资源管理 / 版本管理
│   │   ├── Generator.vue             # 代码生成器
│   │   ├── CodeEditor.vue            # Monaco 代码编辑器
│   │   ├── Safari.vue                # 内置网页预览
│   │   ├── Browser.vue               # 浏览器 CDP 控制器
│   │   ├── Notes.vue                 # 备忘录
│   │   ├── About.vue                 # 关于 / 设置面板
│   │   ├── Profile.vue               # 用户资料
│   │   ├── ContinueDialog.vue        # AI 续写对话框
│   │   ├── Toolbox/                  # 工具箱 (含 7 个子工具面板)
│   │   │   └── Index.vue
│   │   └── Credential/        # 凭据管理器 (5 文件)
│   │       ├── index.vue / Sidebar.vue / Toolbar.vue
│   │       ├── AuthCard.vue / credentialForm.ts
│   ├── components/                   # 系统组件 (9 个)
│   │   ├── MenuBar.vue               # 顶部菜单栏
│   │   ├── Dock.vue                  # 底部程序坞
│   │   ├── DesktopIcons.vue          # 桌面图标（三视图模式）
│   │   ├── ContextMenu.vue           # 桌面右键菜单
│   │   ├── MacWindow.vue             # macOS 风格应用窗口
│   │   ├── LoginScreen.vue           # 登录界面
│   │   ├── LoginForm.vue             # 登录表单
│   │   ├── ProfileMenu.vue           # 用户菜单
│   │   └── Settings.vue              # 设置面板
│   ├── config/
│   │   ├── apps.ts                   # 应用注册表 (id/name/icon/component/Dock/Desktop)
│   │   └── menuBar.ts                # 菜单栏组件显隐配置
│   ├── store/
│   │   ├── locale.ts                 # 语言 Pinia store
│   │   └── theme.ts                  # 主题 Pinia store
│   ├── locales/
│   │   ├── zh-CN.json                # 简体中文
│   │   └── en.json                   # English
│   ├── composables/
│   │   └── useCredential.ts          # 凭据管理 composable
│   ├── utils/
│   │   ├── apiService.ts             # API 服务封装
│   │   ├── request.ts                # HTTP 请求工具
│   │   └── mockData.ts               # Mock 数据
│   ├── styles/
│   │   └── theme.css                 # 主题样式变量
│   └── router/
│       └── index.ts                  # 路由 (仅 / → HomeView)
├── public/assets/
│   └── icons/                        # SVG 图标 (47 个)
│       ├── Toolbox.svg / Browser.svg / Finder.svg / ...
│       └── CsvStats.svg / CsvSplit.svg / ...          # 工具箱子工具图标
├── src-tauri/                        # Rust 后端
│   ├── src/
│   │   ├── main.rs                   # 程序入口
│   │   ├── lib.rs                    # 核心库：setup() / setup_tray() / 命令注册
│   │   ├── commands.rs               # 通用命令 (greet / update_tray_locale)
│   │   └── net/                      # 网络代理模块
│   │       ├── client.rs             # HTTP 客户端
│   │       ├── config.rs             # 代理配置 (proxy.toml)
│   │       ├── proxy.rs              # 代理管理（路径路由）
│   │       └── response.rs           # 统一响应体
│   ├── crates/                       # 子 crate (3 个)
│   │   ├── oasis-credential/         # 凭据管理 (12 个 Tauri 命令)
│   │   ├── oasis-toolbox/            # 工具箱 (9 个 Tauri 命令)
│   │   └── oasis-browser/            # 浏览器控制 (2 个 Tauri 命令)
│   ├── tauri.conf.json               # Tauri 配置 (窗口/安全/打包)
│   ├── capabilities/default.json     # 权限声明
│   └── Cargo.toml                    # Rust workspace 配置
├── docs/                             # 详细技术文档
│   ├── getting-started.md
│   ├── architecture.md
│   ├── credential-storage.md
│   ├── credential-backend-spec.md
│   └── credential-frontend-spec.md
├── vite.config.ts
├── tsconfig.json
├── uno.config.ts
└── package.json
```

---

## 内置应用

| 应用           | ID                   | Dock  | 桌面  | 说明                                   |
| -------------- | -------------------- | :---: | :---: | -------------------------------------- |
| Finder         | `finder`             |   ✅   |       | 文件资源管理与版本控制                 |
| Generator      | `generator`          |   ✅   |       | 代码片段生成，联动 CodeEditor + Safari |
| CodeEditor     | `code-editor`        |   ✅   |       | Monaco Editor，支持多语言编辑          |
| Safari         | `safari`             |   ✅   |       | 内置网页预览                           |
| Browser        | `browser`            |   ✅   |       | Chrome CDP 启动与控制                  |
| Toolbox        | `toolbox`            |   ✅   |       | 7 合 1 工具箱                          |
| Credential     | `credential-manager` |   ✅   |   ✅   | 加密凭据管理                           |
| About          | `about`              |   ✅   |   ✅   | 关于与系统设置                         |
| Notes          | `notes`              |       |   ✅   | 备忘录                                 |
| Profile        | `profile`            |       |       | 用户资料                               |
| ContinueDialog | `continue-dialog`    |       |       | AI 续写                                |

---

## Rust 后端 — Tauri 命令

共注册 **25 个** Tauri 命令，分布在 1 个核心模块 + 3 个 crate 中：

| 模块                 | 命令数 | 功能                                                                   |
| -------------------- | :----: | ---------------------------------------------------------------------- |
| 核心 (`commands.rs`) |   2    | greet / update_tray_locale                                             |
| `oasis-credential`   |   12   | 主密钥管理 (4) + 分类管理 (3) + 凭据 CRUD (5)                          |
| `oasis-toolbox`      |   9    | CSV 统计/拆分/转换、Excel 移动预览/应用、JSON 转换/批量/合并、网络扫描 |
| `oasis-browser`      |   2    | find_chrome_path / launch_chrome_cdp                                   |

### `oasis-credential` 命令清单

```
is_master_key_set  setup_master_key  verify_master_key  change_master_key
list_categories    create_category   delete_category
list_credentials   get_credential    create_credential   update_credential  delete_credential
```

### `oasis-toolbox` 命令清单

```
csv_scan_dir       csv_split_file    csv_convert_file
excel_move_preview excel_move_apply
json_convert_file  json_convert_batch  json_merge_files
network_scan_ports
```

### `oasis-browser` 命令清单

```
find_chrome_path  launch_chrome_cdp
```

---

## 快速开始

### 前置要求

- [Rust](https://www.rust-lang.org/tools/install) stable
- [Bun](https://bun.sh/)
- macOS: Xcode Command Line Tools
- Windows: WebView2 Runtime

### 开发

```bash
bun install
bun tauri dev
```

前端 Vite Dev Server 端口 1420，HMR 端口 1421。Rust 代码修改后自动重新编译。

### 构建

```bash
bun tauri build
```

产物位于 `src-tauri/target/release/bundle/`。

---

## 架构要点

```
Vue 3 Frontend  ──invoke()──▶  Tauri Commands  ──▶  Rust Crates
       ▲                            │
       │                            ▼
       ◀── listen() ────  Events (tray-action, etc.)
```

- **前端**：Vue 3 Composition API + Element Plus + Pinia，通过 `@tauri-apps/api/core` 的 `invoke` 调用后端命令。
- **后端**：Rust workspace，3 个功能 crate 通过 `_handlers!()` 宏批量注册 Tauri 命令到 `lib.rs`。
- **窗口管理**：`HomeView.vue` 通过 `windowStates` reactive 对象 + `<Teleport to="body">` 管理 11 个应用窗口的显隐与最小化状态。
- **数据流**：Generator 生成代码 → emit → HomeView 打开 CodeEditor + Safari；Finder 加载版本 → emit → 联动编辑器与预览。
- **安全**：凭据模块使用 Ring AES-256-GCM 加密，主密钥 PBKDF2 派生，存储在 SQLite (bundled)。

---

## 构建配置

| 配置       | 值                             |
| ---------- | ------------------------------ |
| Tauri 窗口 | 1200×800，居中，Overlay 标题栏 |
| TypeScript | strict 全开，target ES2021     |
| 路径别名   | `@` → `./src`                  |
| 打包格式   | macOS: app + dmg               |

---

## 文档

- [快速上手](docs/getting-started.md)
- [架构设计](docs/architecture.md)
- [凭据存储方案](docs/credential-storage.md)
- [凭据管理后端规范](docs/credential-backend-spec.md)
- [凭据管理前端规范](docs/credential-frontend-spec.md)

---

## 许可证

[MIT License](LICENSE)
*（内容由AI生成，仅供参考）*
