# Oasis

> 基于 GPUI 的插件化桌面效率工具箱，让开发者专注创造，而非琐碎

Oasis 是一个使用 Rust + GPUI 构建的原生桌面应用，采用插件化架构设计。支持三种插件模式：静态链接（rlib）、动态库（cdylib）、WebAssembly（WASM），让开发者可以轻松扩展功能。

---

## ✨ 核心特性

### 🖥️ 强大的宿主功能
- **浮动窗口管理** — 插件在主窗口内以浮动方式运行，支持拖拽、缩放、最小化、最大化
- **应用启动器** — 全屏网格启动器，一键打开插件
- **Dock 栏** — 底部浮动 Dock，悬停放大动画，快速访问所有插件
- **主题系统** — 深色/浅色切换，支持自定义主题热加载
- **国际化** — 内置中英文，可随时切换

### 🔌 丰富的插件生态
- **📝 Markdown 编辑器** — 基于 ropey 的高性能文本编辑器，支持语法高亮、文件浏览器、撤销/重做
- **🗒️ 记事本** — 轻量级文本编辑，实时字数/行数统计
- **🧰 工具箱** — CSV 统计/分割/转换、批量重命名、Excel 处理、API 请求、JSON 合并等多工具集合
- **🔐 凭据管理** — AES-GCM 加密存储（开发中）
- **🌐 WASM 插件** — 基于 wasmi 沙箱运行，安全隔离

---

## 🚀 快速开始

### 前置要求

- Rust stable toolchain（`cargo`）
- macOS / Linux（Windows 理论支持）

### 运行桌面应用

```bash
git clone <repository-url>
cd oasis

# 编译并运行
cargo run

# Release 构建
cargo build --release
```

### 运行 Web（WASM）版本

```bash
# 前置：Rust nightly + wasm32 target + wasm-bindgen-cli + Bun
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli

# 构建 WASM + 启动开发服务器
make dev-web

# 访问 http://localhost:3000
```

---

## 📁 项目结构

```
oasis/
├── src/                     # 宿主应用源码
│   ├── main.rs             # 桌面二进制入口
│   ├── lib.rs              # 库根
│   ├── app/                # 应用基础设施层
│   │   ├── dock.rs         # 底部浮动 Dock 栏
│   │   ├── app_launcher.rs # 全屏应用启动器
│   │   ├── themes.rs       # 主题系统
│   │   └── ...
│   └── plugins/            # 插件系统
│       ├── mod.rs          # PluginRegistry 注册中心
│       ├── plugin_window.rs # PluginWindow 浮动窗口
│       └── wasm_*.rs       # WASM 插件支持
├── crates/                 # 子 crate
│   ├── plugins/            # 插件相关
│   │   ├── plugin-sdk/     # 插件 Trait + 元数据
│   │   ├── notepad-plugin/ # 记事本插件
│   │   ├── toolbox-plugin/ # 工具箱插件
│   │   └── md-editor-plugin/ # Markdown 编辑器
│   └── widgets/            # WASM widgets
├── docs/                   # 📚 完整文档
│   ├── user/               # 用户指南
│   ├── development/        # 开发者指南
│   └── maintenance/        # 维护者指南
└── plugins/                # 运行时插件目录
```

---

## 📚 完整文档

### 👤 用户文档
- **[快速开始](docs/user/getting-started.md)** — 新手入门指南
- **[用户手册](docs/user/user-guide.md)** — 完整功能说明
- **[配置参考](docs/user/configuration.md)** — 高级配置选项
- **[故障排除](docs/user/troubleshooting.md)** — 常见问题解决

### 🔌 开发者文档
- **[插件开发总览](docs/development/plugin-guide.md)** — 插件开发指南
- **[WASM 插件](docs/development/wasm-plugins.md)** — WASM 插件详细教程
- **[API 参考](docs/development/api-reference.md)** — 完整 API 文档
- **[架构文档](docs/development/architecture.md)** — 技术架构

### 🛠️ 维护者文档
- **[贡献指南](CONTRIBUTING.md)** — 如何贡献代码
- **[发布流程](docs/maintenance/release-process.md)** — 版本发布规范
- **[变更日志](CHANGELOG.md)** — 版本历史记录

---

## 🎯 快速上手示例

### 创建你的第一个 WASM 插件

```rust
// src/lib.rs
use serde_json::{json, Value};

#[no_mangle]
pub extern "C" fn plugin_get_manifest() -> i32 {
    // 返回插件元数据和 UI 描述
}

#[no_mangle]
pub extern "C" fn plugin_get_state() -> i32 {
    // 返回插件状态
}

#[no_mangle]
pub extern "C" fn plugin_handle_action(ptr: i32, len: i32) -> i32 {
    // 处理用户交互
}
```

**快速链接**: [完整 WASM 插件教程](docs/development/wasm-plugins.md)

---

## 🗺️ 发展路线图

### 已完成 ✅
- [x] 插件化架构（rlib / cdylib / WASM 三模式）
- [x] 浮动插件窗口（拖拽/缩放/最大化）
- [x] Dock 栏 + 应用启动器
- [x] 主题系统（深色/浅色/自定义）
- [x] 国际化（中/英文）
- [x] Markdown 编辑器插件
- [x] 工具箱插件（CSV/Excel/API 等）
- [x] WASM 插件沙箱（wasmi）

### 进行中 🚧
- [ ] 插件间通信（plugin-ipc 协议）
- [ ] 凭据管理插件（AES-GCM 加密）
- [ ] 网络工具插件

### 计划中 📋
- [ ] 插件热重载
- [ ] LSP 集成（代码补全/诊断）
- [ ] 插件市场

---

## 🤝 贡献

欢迎贡献！请查看：

- **[贡献指南](CONTRIBUTING.md)** — 贡献流程和代码规范
- **[问题追踪](../../issues)** — 报告问题或建议功能
- **[讨论区](../../discussions)** — 一般性讨论

### 快速贡献流程

1. Fork 仓库
2. 创建功能分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'feat: add AmazingFeature'`)
4. 推送分支 (`git push origin feature/AmazingFeature`)
5. 开启 Pull Request

---

## 📄 许可证

MIT License — 详见 [LICENSE](LICENSE) 文件

---

## 🙏 致谢

- [GPUI](https://github.com/zed-industries/zed) — Zed Industries 出品的 GPU 加速 UI 框架
- [gpui-component](https://github.com/ht-shaipe/gpui-component) — UI 组件库
- [wasmi](https://github.com/wasmi-labs/wasmi) — 轻量级 WASM 运行时

---

<div align="center">

**⭐ 如果觉得有用，请给一个 Star！**

**🔖 关注项目获取最新更新**

*Made with ❤️ by [shaipe](https://github.com/shaipe)*

</div>