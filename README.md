# Oasis

> 基于 Tauri v2 + Vue 3 构建的轻量级 Mac 风格桌面效率平台。

Oasis 是一个模拟 macOS 桌面体验的桌面应用，旨在为开发者和效率追求者提供一个集成化的工具环境。它结合了 Rust 的高性能后端与 Vue 3 的灵活前端，通过多窗口管理系统集成了多种实用工具。

---

## ✨ 核心特性

### 🖥️ 沉浸式桌面体验
- **Mac 风格界面** — 包含顶部菜单栏 (MenuBar)、底部 Dock 栏和桌面图标 (Desktop Icons)，提供原生的操作感受。
- **多窗口管理** — 支持应用窗口的拖拽、层级管理，模拟真实的操作系统多任务环境。
- **系统托盘** — 支持多语言快速切换（中/英），后台常驻运行。

### 🔌 内置应用集
- **Generator** — 快速生成代码或数据片段。
- **CodeEditor** — 轻量级代码编辑器。
- **Safari Preview** — 内置网页预览工具，支持快速查看网页内容。
- **Finder** — 文件资源管理。
- **Notes** — 随时随地记录灵感与笔记。
- **CredentialManager** — 安全的凭据管理器，保护您的敏感信息。

### ⚙️ 后端技术支撑
- **凭据安全** — 基于 SQLite 的持久化存储，并采用 AES-GCM (Ring) 工业级加密算法。
- **高性能** — 利用 Tauri v2 的优势，提供极小的内存占用与极快的响应速度。
- **跨平台** — 继承 Tauri 的跨平台特性，支持 macOS、Windows 等主流系统。

---

## 🛠️ 技术栈

- **前端 (Frontend)**: [Vue 3](https://vuejs.org/) (Composition API), [TypeScript](https://www.typescriptlang.org/), [Vite](https://vitejs.dev/)
- **UI 框架**: [Element Plus](https://element-plus.org/)
- **状态管理**: [Pinia](https://pinia.vuejs.org/)
- **后端 (Backend)**: [Tauri v2](https://v2.tauri.app/), [Rust](https://www.rust-lang.org/)
- **数据库**: [SQLite](https://sqlite.org/) ([Rusqlite](https://github.com/rusqlite/rusqlite))
- **加密安全**: [Ring](https://github.com/briansmith/ring) (AES-GCM Encryption)
- **包管理**: [Bun](https://bun.sh/)

---

## 📁 项目结构

```text
oasis/
├── src/                        # 前端源码 (Vue 3 + TS)
│   ├── components/
│   │   ├── apps/               # 窗口应用 (Generator, CodeEditor, Safari, etc.)
│   │   ├── system/             # 系统组件 (MenuBar, Dock, DesktopIcons)
│   │   └── common/             # 通用基础组件
│   ├── store/                  # Pinia 状态管理
│   ├── locales/                # 国际化多语言文件
│   ├── views/                  # 视图页面
│   └── App.vue                 # 应用根组件
├── src-tauri/                  # 后端源码 (Rust)
│   ├── src/
│   │   ├── credential/         # 凭据管理模块 (数据库操作、加密逻辑、指令集)
│   │   ├── main.rs             # Tauri 程序入口
│   │   └── lib.rs              # 核心库逻辑
│   ├── tauri.conf.json         # Tauri 配置文件
│   └── Cargo.toml              # Rust 依赖配置
├── public/                     # 静态资源
└── package.json                # 前端项目依赖配置
```

---

## 🚀 快速开始

### 前置要求

- [Rust](https://www.rust-lang.org/tools/install) 环境 (stable)
- [Bun](https://bun.sh/) 运行时
- 相应的系统开发工具包 (如 macOS 的 Xcode, Windows 的 WebView2 等)

### 开发环境启动

```bash
# 安装依赖
bun install

# 启动开发服务器 (Tauri dev)
bun tauri dev
```

### 构建打包

```bash
# 构建发布版本
bun tauri build
```

---

## 📚 文档指南

- **[快速上手](docs/getting-started.md)** — 环境搭建与运行指南。
- **[架构设计](docs/architecture.md)** — Tauri + Vue 前后端协作架构简述。
- **[凭据存储方案](docs/credential-storage.md)** — 凭据加密与 SQLite 存储逻辑。

---

## 📄 许可证

本项目采用 [MIT License](LICENSE) 许可。

---

<div align="center">
  <p>Made with ❤️ by Oasis Team</p>
</div>
