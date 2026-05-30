# 快速上手

本指南将帮助您在本地环境搭建、运行和构建 Oasis 项目。

## 🛠️ 前置要求

在开始之前，请确保您的开发环境已安装以下工具：

- **Rust**: [安装 Rust](https://www.rust-lang.org/tools/install) (建议使用 stable 版本)
- **Node.js**: [安装 Node.js](https://nodejs.org/) (建议使用 v18+)
- **Bun**: [安装 Bun](https://bun.sh/) (用于快速安装前端依赖)
- **Tauri CLI**: `cargo install tauri-cli`

## 📥 安装依赖

1. 克隆代码仓库：
   ```bash
   git clone <repository-url>
   cd oasis
   ```

2. 安装前端依赖：
   ```bash
   bun install
   ```

## 🚀 开发环境运行

您可以同时启动前端 Vite 开发服务器和 Tauri 后端：

```bash
bun run tauri dev
```

这会自动打开应用窗口。前端支持热重载（HMR），后端 Rust 代码修改后会自动重新编译。

## 🏗️ 生产版本构建

构建适用于您当前操作系统的生产版本：

```bash
bun run tauri build
```

构建产物位于 `src-tauri/target/release/bundle/` 目录下。

## 🌐 Web (WASM) 版本开发

如果您需要开发 WASM 插件或运行 Web 预览：

1. 添加 WASM 编译目标：
   ```bash
   rustup target add wasm32-unknown-unknown
   ```

2. 运行 Makefile 命令：
   ```bash
   make dev-web
   ```

访问 `http://localhost:3000` 即可预览。

## 📂 项目结构说明

- `src/`: Vue 3 前端源码。
- `src-tauri/`: Rust 后端源码。
- `crates/`: 内部 Rust crate，包含插件 SDK 和预置插件。
- `docs/`: 详细技术文档。
