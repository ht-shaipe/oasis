# Oasis

> 基于 GPUI 的插件化桌面效率工具箱，让开发者专注创造，而非琐碎

Oasis 是一个使用 Rust + GPUI 构建的原生桌面应用，采用插件化架构设计。宿主提供窗口管理、Dock 栏、主题系统等基础设施，功能由独立插件提供，支持三种插件模式：静态链接（rlib）、动态库（cdylib）、WebAssembly（WASM）。

---

## ✨ 特性

### 宿主功能
- **🖥️ 浮动窗口管理** — 插件在主窗口内以浮动方式运行，支持拖拽、缩放、最小化、最大化
- **🚀 应用启动器** — 全屏网格启动器（类 macOS Launchpad），一键打开插件
- **🎣 Dock 栏** — 底部浮动 Dock，悬停放大动画，快速访问所有插件
- **🎨 主题系统** — 深色/浅色切换，支持自定义主题热加载（`./themes` 目录）
- **🌍 国际化** — 内置中英文（`locales/`），可随时切换
- **🖼️ 桌面背景** — 支持自定义背景图片（右键菜单设置/清除）
- **📦 系统托盘** — 跨平台系统托盘图标（桌面模式）

### 插件功能
- **📝 Markdown 编辑器** — 基于 ropey Rope 的高性能文本编辑器，支持语法高亮、文件浏览器、撤销/重做
- **🗒️ 记事本** — 轻量级文本编辑，实时字数/行数统计
- **🧰 工具箱** — CSV 统计/分割/转换、批量重命名、Excel 处理、API 请求、JSON 合并等多工具集合
- **🔐 凭据管理** — AES-GCM 加密存储（开发中）
- **🔌 WASM 插件** — 基于 wasmi 沙箱运行，安全隔离

---

## 🚀 快速开始

### 前置要求

- Rust stable toolchain（`cargo` 即可）
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
# 或者手动执行:
# ./scripts/build-wasm.sh && cd www && bun install && bun run dev
```

访问 `http://localhost:3000`

---

## 📁 项目结构

```
oasis/
├── Cargo.toml                  # 主包配置（workspace 根）
├── src/                        # 宿主应用源码
│   ├── main.rs                 # 桌面二进制入口
│   ├── lib.rs                  # 库根（DockRoot 主视图 + WASM 入口）
│   ├── assets.rs               # 资源重导出
│   ├── i18n.rs                 # 国际化初始化
│   ├── app/                    # 应用基础设施层
│   │   ├── actions.rs          # GPUI Action 定义
│   │   ├── app_launcher.rs     # 全屏应用启动器
│   │   ├── app_menus.rs        # 菜单栏（主题/语言切换）
│   │   ├── app_state.rs        # 全局应用状态
│   │   ├── background.rs       # 桌面背景图片管理
│   │   ├── dock.rs             # 底部浮动 Dock 栏
│   │   ├── drag_state.rs       # 插件窗口全局拖动/缩放状态
│   │   ├── embedded_themes.rs  # WASM 内嵌主题
│   │   ├── key_binding.rs      # 键盘快捷键注册
│   │   ├── system_tray.rs      # 系统托盘
│   │   ├── themes.rs           # 主题加载与持久化
│   │   └── title_bar.rs        # 应用标题栏
│   ├── panels/
│   │   └── mod.rs              # SamplePanel（主内容面板）
│   └── plugins/                # 插件系统
│       ├── mod.rs              # PluginRegistry 注册中心
│       ├── plugin_window.rs    # PluginWindow 浮动窗口组件
│       ├── dyn_plugin_view.rs  # cdylib 插件渲染包装
│       ├── wasm_plugin_view.rs # WASM 插件渲染 + 通用渲染器
│       ├── wasm_runtime.rs     # wasmi WASM 运行时封装
│       ├── wasm_loader.rs      # WASM 插件扫描/加载 UI
│       ├── wasm_plugin_system.rs # WASM 插件注册
│       └── wasm_example.rs     # WASM 运行时示例
├── crates/                     # 子 crate（插件 + 共享类型）
│   ├── ui-schema/              # UI 描述契约（无 gpui 依赖）
│   ├── plugins/
│   │   ├── plugin-sdk/         # 插件 Trait + 元数据定义
│   │   ├── plugin-ipc/         # 子进程插件通信协议
│   │   ├── notepad-plugin/     # 记事本插件（dylib）
│   │   ├── toolbox-plugin/     # 工具箱插件（cdylib）
│   │   ├── network-plugin/     # 网络工具（占位）
│   │   ├── credential-plugin/  # 凭据管理（占位）
│   │   └── md-editor-plugin/   # Markdown 编辑器（rlib + 独立 bin）
│   └── widgets/
│       └── wasm-widget/        # WASM 插件示例（wasm-bindgen）
├── plugins/                    # 运行时插件部署目录
│   ├── notepad/
│   │   └── libnotepad.dylib
│   ├── toolbox-plugin/
│   │   ├── manifest.toml
│   │   └── libtoolbox-plugin.dylib
│   └── wasm/
│       └── dsl_counter.wasm
├── locales/                    # 国际化翻译文件
│   ├── en.yml
│   └── zh-CN.yml
├── assets/                     # 图标与静态资源
├── themes/                     # 自定义主题目录（JSON 格式，热加载）
└── www/                        # WASM Web 前端（Vite）
```

---

## 🔌 插件系统

Oasis 支持三种插件开发模式：

### 1. 静态链接插件（rlib）

适合需要完整 GPUI 能力的复杂插件（如 Markdown 编辑器）：

```rust
// 在 src/plugins/mod.rs 中注册
use crate::plugins::{PluginEntry, PluginManifest};
inventory::submit!(PluginEntry {
    id: "my-plugin",
    manifest_toml: include_str!("manifest.toml"),
    icon_svg: include_str!("icon.svg"),
    create_view: my_plugin::create_view,
});
```

或者作为独立子进程运行（md-editor-plugin 双模式）。

### 2. 动态库插件（cdylib）

将插件编译为 `.dylib`，放入 `plugins/<plugin-id>/` 目录，宿主自动扫描加载：

```rust
// 插件必须导出此函数
#[no_mangle]
pub fn plugin_entry() -> Arc<dyn Plugin> {
    Arc::new(MyPlugin::new())
}
```

插件目录结构：
```
plugins/
└── my-plugin/
    ├── manifest.toml       # 可选，定义展示名称/窗口尺寸
    ├── icon.svg            # 可选，Dock 图标
    └── libmy-plugin.dylib  # 编译产物
```

### 3. WASM 插件

将插件编译为 `.wasm`，放入 `plugins/wasm/` 目录，运行在 wasmi 安全沙箱中：

```rust
// WASM 插件必须导出的函数
#[no_mangle]
pub extern "C" fn plugin_get_manifest() -> i32 { /* 返回 WasmManifest JSON */ }
#[no_mangle]
pub extern "C" fn plugin_get_state() -> i32 { /* 返回当前状态 JSON */ }
#[no_mangle]
pub extern "C" fn plugin_handle_action(ptr: i32, len: i32) -> i32 { /* 处理动作 */ }
```

宿主提供的注入函数：
- `env::host_log(ptr, len)` — 日志输出
- `env::host_get_context(ptr, len)` — 获取宿主上下文（当前文件、选区等）
- `env::host_read_file(ptr, len)` — 读取文件
- `env::host_write_file(...)` — 写入文件
- `env::host_show_notification(ptr, len)` — 显示通知

参考实现：`crates/widgets/wasm-widget/`

### manifest.toml 格式

```toml
id = "my-plugin"
display_name = "我的插件"
description = "插件描述"
icon = "🔧"
window_width = 600.0
window_height = 400.0
```

### UI Schema 渲染器

插件通过 `UiSchema` 描述界面，宿主渲染器（`wasm_plugin_view.rs`）负责渲染，无需插件依赖 GPUI：

```rust
// crates/ui-schema 定义的类型
pub struct UiSchema {
    pub layout: String,       // "flex-col" | "flex-row" | "grid"
    pub children: Vec<UiNode>,
}

pub struct UiNode {
    pub component: String,    // "label" | "button" | "input" | "table" | "progress" | ...
    pub props: serde_json::Value,
    pub bind: Option<String>, // 绑定状态字段
    pub on_action: Option<String>,
    pub children: Vec<UiNode>,
}
```

支持的组件：`display`、`label`、`button`、`button_row`、`input`、`select`、`table`、`progress`、`switch`、`info`、`divider`、`card`

---

## 🛠️ 开发

### 编译全部

```bash
# 编译主应用（含所有内置插件）
cargo build

# Release 编译
cargo build --release
```

### 编译单个插件

```bash
# 编译记事本插件
cargo build -p notepad-plugin

# 编译工具箱插件
cargo build -p toolbox-plugin

# 将产物复制到插件目录
cp target/debug/libnotepad.dylib plugins/notepad/
cp target/debug/libtoolbox_plugin.dylib plugins/toolbox-plugin/
```

### 编译 WASM 插件

```bash
cd crates/widgets/wasm-widget
./build.sh
# 产物输出到 plugins/wasm/dsl_counter.wasm
```

### 自定义主题

在 `themes/` 目录下放置 JSON 主题文件，运行时热加载（无需重启），格式参考 [gpui-component 主题规范](https://github.com/ht-shaipe/gpui-component)。

### 添加翻译

编辑 `locales/en.yml` 和 `locales/zh-CN.yml`，格式：

```yaml
app_title: "Oasis"
plugin_open: "打开插件"
```

在代码中使用：`t!("app_title")`

---

## 🗺️ Roadmap

- [x] 插件化架构（rlib / cdylib / WASM 三模式）
- [x] 浮动插件窗口（拖拽/缩放/最大化）
- [x] Dock 栏 + 应用启动器
- [x] 主题系统（深色/浅色/自定义）
- [x] 国际化（中/英文）
- [x] Markdown 编辑器插件
- [x] 工具箱插件（CSV/Excel/API 等）
- [x] WASM 插件沙箱（wasmi）
- [x] 系统托盘
- [ ] 插件间通信（plugin-ipc 协议）
- [ ] 凭据管理插件（AES-GCM 加密）
- [ ] 网络工具插件
- [ ] 插件热重载
- [ ] LSP 集成（代码补全/诊断）
- [ ] 插件市场

---

## 🤝 贡献

欢迎提交 Pull Request 或 Issue！

1. Fork 仓库
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'feat: add AmazingFeature'`)
4. 推送分支 (`git push origin feature/AmazingFeature`)
5. 开启 Pull Request

---

## 📄 许可证

MIT License — 详见 LICENSE 文件

---

## 🙏 致谢

- [GPUI](https://github.com/zed-industries/zed) — Zed Industries 出品的 GPU 加速 UI 框架
- [gpui-component](https://github.com/ht-shaipe/gpui-component) — UI 组件库
- [wasmi](https://github.com/wasmi-labs/wasmi) — 轻量级 WASM 运行时

---

*Made with ❤️ by shaipe*
