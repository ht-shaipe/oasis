# 🔌 WASM 插件示例 - 快速查看

## ✅ 已创建的文件

### WASM 插件（已构建）
```
crates/wasm-plugin/pkg/wasm_plugin_bg.wasm  (89.2KB)
```
这是编译后的 WebAssembly 插件文件！

### 源代码文件

1. **插件实现**: `crates/wasm-plugin/src/lib.rs`
   - `PluginMetadata` - 插件元数据
   - `CalculatorPlugin` - 计算器插件示例
   - 导出函数: `create_plugin()`, `execute()`, `on_button_click()`

2. **主机示例**: `src/plugins/wasm_example.rs`
   - `WasmRuntime` - WASM 运行时封装
   - `WasmPluginExample` - UI 示例组件
   - 展示了如何与 WASM 插件交互

## 🚀 使用示例

### 1. WASM 插件代码（Rust）

```rust
#[wasm_bindgen]
pub struct CalculatorPlugin {
    current_value: f64,
}

#[wasm_bindgen]
impl CalculatorPlugin {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self { current_value: 0.0 }
    }

    #[wasm_bindgen]
    pub fn execute(&mut self, operation: &str, value: f64) -> PluginState {
        match operation {
            "add" => self.current_value += value,
            "subtract" => self.current_value -= value,
            // ...
        }
        self.get_state()
    }
}
```

### 2. 主机代码（Rust）

```rust
// 模拟 WASM 调用
let mut runtime = WasmRuntime::new("Calculator".to_string());

// 执行计算
runtime.execute("add", 10.0)?;
runtime.execute("multiply", 5.0)?;

// 获取结果
println!("结果: {}", runtime.get_display());
```

## 📊 架构

```
用户 → UI → 主机框架 → WASM 运行时 → WASM 插件
       ↑                              ↓
       └──────── 更新状态 ←───────────┘
```

## 🎯 核心概念

1. **插件定义** - 使用 `#[wasm_bindgen]` 导出函数
2. **状态管理** - 通过 JSON 序列化传递数据
3. **主机框架** - 负责加载和调用 WASM
4. **UI 渲染** - 主应用负责，WASM 提供逻辑

## 🔍 下一步

1. **构建 WASM**:
   ```bash
   cd crates/wasm-plugin
   wasm-pack build --target web
   ```

2. **集成到主应用**:
   ```rust
   use crate::plugins::wasm_example;
   wasm_example::init_example(cx);
   ```

3. **扩展功能**:
   - 集成 wasmi 运行时
   - 实现热加载
   - 添加更多插件示例

查看完整文档: [EXAMPLE.md](./EXAMPLE.md)
