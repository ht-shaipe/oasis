# WASM 插件系统示例

这是一个完整的 WASM 插件系统示例，展示了如何在 Rust 应用中使用 WebAssembly 作为插件加载方式。

## 📁 文件结构

```
oasis/
├── crates/
│   └── wasm-plugin/           # WASM 插件实现
│       ├── src/
│       │   └── lib.rs         # 插件源码
│       ├── Cargo.toml         # 插件配置
│       └── pkg/               # 编译输出（运行 wasm-pack build 后生成）
│           ├── *.wasm         # WebAssembly 文件
│           ├── *.js           # JavaScript 绑定
│           └── *.d.ts         # TypeScript 定义
├── src/
│   └── plugins/
│       ├── wasm_example.rs    # 主机示例实现
│       └── wasm_host.rs       # WASM 主机框架
```

## 🚀 快速开始

### 1. 构建 WASM 插件

```bash
cd crates/wasm-plugin
wasm-pack build --target web --out-dir pkg
```

构建成功后会在 `pkg/` 目录下生成以下文件：
- `wasm_plugin_bg.wasm` - 编译后的 WASM 文件
- `wasm_plugin.js` - JavaScript 绑定
- `wasm_plugin.d.ts` - TypeScript 类型定义

### 2. 插件结构

WASM 插件包含以下主要部分：

#### PluginMetadata - 插件元数据
```rust
#[wasm_bindgen]
pub struct PluginMetadata {
    name: String,
    version: String,
    description: String,
}
```

#### PluginState - 插件状态
```rust
#[wasm_bindgen]
pub struct PluginState {
    data: String,  // JSON 序列化的状态数据
}
```

#### CalculatorPlugin - 插件实现
```rust
#[wasm_bindgen]
pub struct CalculatorPlugin {
    current_value: f64,
}

#[wasm_bindgen]
impl CalculatorPlugin {
    pub fn new() -> Self;
    pub fn metadata(&self) -> PluginMetadata;
    pub fn get_state(&self) -> PluginState;
    pub fn execute(&mut self, operation: &str, value: f64) -> PluginState;
    pub fn on_button_click(&mut self, button: &str) -> PluginState;
}
```

### 3. 主机集成

在主应用中，可以通过以下方式集成 WASM 插件：

```rust
// 初始化 WASM 插件系统
plugins::wasm_example::init_example(cx);

// 创建 WASM 插件视图
let wasm_view = plugins::wasm_example::WasmPluginExample::new();
```

## 📊 工作原理

### 架构图

```
┌─────────────────────────────────────────────────────────┐
│                    主应用 (Oasis)                        │
│  ┌─────────────────────────────────────────────────┐   │
│  │              WASM 插件主机                        │   │
│  │  • 加载 WASM 文件                                │   │
│  │  • 调用 WASM 函数                                │   │
│  │  • 序列化/反序列化数据                           │   │
│  └─────────────────────────────────────────────────┘   │
│                         ↕                               │
│  ┌─────────────────────────────────────────────────┐   │
│  │         WASM 运行时 (wasmi/wasmtime)             │   │
│  │  • 沙箱执行环境                                   │   │
│  │  • 内存管理                                       │   │
│  │  • 函数调用                                       │   │
│  └─────────────────────────────────────────────────┘   │
│                         ↕                               │
│  ┌─────────────────────────────────────────────────┐   │
│  │            WASM 插件                            │   │
│  │  • 业务逻辑                                       │   │
│  │  • 状态管理                                       │   │
│  │  • 导出函数                                       │   │
│  └─────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
```

### 数据流

```
1. 用户操作 → 主应用 UI
2. 主应用 → 调用 WASM 插件函数
3. WASM 运行时 → 执行插件代码
4. 插件 → 返回结果（JSON）
5. 主应用 → 反序列化并更新 UI
```

## ✨ 优势

| 优势 | 说明 |
|------|------|
| **动态加载** | 运行时加载插件，无需重新编译主应用 |
| **沙箱安全** | WASM 运行在隔离环境中，提供安全边界 |
| **跨平台** | WASM 模块可以在不同平台上运行 |
| **热更新** | 可以动态替换插件而无需重启应用 |
| **多语言** | 可以使用多种语言编写插件（Rust, C++, AssemblyScript 等） |

## ⚠️ 限制

| 限制 | 说明 |
|------|------|
| **UI 渲染** | WASM 不能直接渲染 UI，需要通过主应用 |
| **性能开销** | 主机与 WASM 的通信有序列化开销 |
| **调试难度** | WASM 插件的调试比原生代码复杂 |
| **内存限制** | WASM 有独立的内存空间，数据共享需要复制 |

## 📝 当前实现状态

- ✅ **WASM 插件定义** - 完整的插件接口和示例
- ✅ **构建系统** - wasm-pack 配置和自动化构建
- ✅ **主机框架** - Rust 端的插件加载和管理框架
- ✅ **示例实现** - 可运行的计算器示例
- ⏳ **完整运行时** - 需要集成 wasmi 或 wasmtime
- ⏳ **热加载** - 动态加载和卸载插件
- ⏳ **插件市场** - 插件发现和分发机制

## 🔧 进一步开发

### 集成完整 WASM 运行时

要实现完整的 WASM 支持，需要：

1. **添加依赖**：
   ```toml
   [dependencies]
   wasmi = "0.34"
   ```

2. **实现 WASM 加载器**：
   ```rust
   use wasmi::{Engine, Module, Store};

   pub fn load_wasm_plugin(wasm_bytes: &[u8]) -> Result<Plugin, Error> {
       let engine = Engine::default();
       let module = Module::new(&engine, wasm_bytes)?;
       // ... 实例化和链接
   }
   ```

3. **实现函数调用**：
   ```rust
   pub fn call_plugin_function(
       &mut self,
       func_name: &str,
       args: &[Value]
   ) -> Result<Value, Error> {
       // 调用 WASM 函数并处理返回值
   }
   ```

### 参考资源

- [wasm-bindgen](https://rustwasm.github.io/wasm-bindgen/) - Rust 和 WebAssembly 之间的绑定
- [wasmi](https://github.com/paritytech/wasmi) - 纯 Rust 实现的 WASM 解释器
- [wasmtime](https://wasmtime.dev/) - 高性能 WASM 运行时
- [wasm-pack](https://rustwasm.github.io/wasm-pack/) - Rust WASM 工作链

## 📄 许可证

MIT OR Apache-2.0
