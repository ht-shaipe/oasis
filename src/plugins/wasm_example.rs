//! WASM 插件完整示例
//!
//! 这个示例展示了 WASM 插件系统的概念和结构

use gpui::{div, px, App, Context, IntoElement, ParentElement, Render, Styled, Window};
use gpui_component::ActiveTheme as _;

/// WASM 插件运行时封装
pub struct WasmRuntime {
    /// 插件名称
    pub name: String,
    /// 当前值
    pub current_value: f64,
    /// 历史记录
    pub history: Vec<String>,
}

impl WasmRuntime {
    pub fn new(name: String) -> Self {
        Self {
            name,
            current_value: 0.0,
            history: Vec::new(),
        }
    }

    /// 执行计算（模拟 WASM 函数调用）
    pub fn execute(&mut self, operation: &str, value: f64) -> Result<String, String> {
        let result = match operation {
            "add" => {
                self.current_value += value;
                format!("{} + {} = {}", self.current_value - value, value, self.current_value)
            }
            "subtract" => {
                self.current_value -= value;
                format!("{} - {} = {}", self.current_value + value, value, self.current_value)
            }
            "multiply" => {
                let old = self.current_value;
                self.current_value *= value;
                format!("{} × {} = {}", old, value, self.current_value)
            }
            "divide" => {
                if value == 0.0 {
                    return Err("Cannot divide by zero".to_string());
                }
                let old = self.current_value;
                self.current_value /= value;
                format!("{} ÷ {} = {}", old, value, self.current_value)
            }
            "clear" => {
                self.current_value = 0.0;
                self.history.clear();
                return Ok("Cleared".to_string());
            }
            _ => return Err(format!("Unknown operation: {}", operation)),
        };

        self.history.push(result.clone());
        Ok(result)
    }

    pub fn get_display(&self) -> String {
        if self.current_value == 0.0 {
            "0".to_string()
        } else if self.current_value.fract() == 0.0 {
            format!("{}", self.current_value as i64)
        } else {
            format!("{:.2}", self.current_value)
        }
    }
}

/// WASM 插件示例 UI
pub struct WasmPluginExample {
    runtime: WasmRuntime,
}

impl WasmPluginExample {
    pub fn new() -> Self {
        Self {
            runtime: WasmRuntime::new("Calculator".to_string()),
        }
    }
}

impl Render for WasmPluginExample {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        // 执行一些示例计算
        let _ = self.runtime.execute("add", 10.0);
        let _ = self.runtime.execute("multiply", 5.0);
        let _ = self.runtime.execute("subtract", 3.0);

        div()
            .flex()
            .flex_col()
            .gap(px(12.))
            .p(px(16.))
            .rounded_lg()
            .bg(theme.colors.background)
            .border_1()
            .border_color(theme.colors.border)
            .shadow_lg()
            .child(
                div()
                    .text_size(px(14.))
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(theme.colors.foreground)
                    .child("🔌 WASM 插件系统示例"),
            )
            .child(
                div()
                    .text_size(px(12.))
                    .text_color(theme.colors.muted_foreground)
                    .child("这个示例展示了 WASM 插件系统的概念"),
            )
            .child(
                div()
                    .w(px(200.))
                    .h(px(50.))
                    .bg(gpui::rgb(0x1a1a1a))
                    .flex()
                    .items_center()
                    .justify_end()
                    .px(px(16.))
                    .rounded_lg()
                    .text_size(px(20.))
                    .text_color(gpui::rgb(0xffffff))
                    .child(self.runtime.get_display()),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(6.))
                    .child(
                        div()
                            .text_size(px(11.))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(theme.colors.muted_foreground)
                            .child("📝 计算历史:"),
                    )
                    .children(
                        self.runtime.history.iter().map(|entry| {
                            div()
                                .text_size(px(10.))
                                .text_color(theme.colors.muted_foreground)
                                .child(format!("  • {}", entry))
                        }),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(8.))
                    .child(
                        div()
                            .text_size(px(11.))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(theme.colors.muted_foreground)
                            .child("📁 文件结构:"),
                    )
                    .children(
                        [
                            "crates/wasm-plugin/",
                            "  ├── src/lib.rs         # WASM 插件实现",
                            "  ├── Cargo.toml         # 插件配置",
                            "  └── pkg/               # 构建输出",
                            "    ├── *.wasm           # 编译后的 WASM 文件",
                            "    └── *.js             # JavaScript 绑定",
                            "src/plugins/",
                            "  ├── wasm_example.rs    # 示例实现",
                            "  └── wasm_host.rs       # 主机框架",
                        ]
                        .iter()
                        .map(|line| {
                            div()
                                .text_size(px(10.))
                                .text_color(theme.colors.muted_foreground)
                                .child(format!("  {}", line))
                        }),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(6.))
                    .child(
                        div()
                            .text_size(px(11.))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(theme.colors.muted_foreground)
                            .child("🚀 使用步骤:"),
                    )
                    .children(
                        [
                            "1. 构建 WASM 插件:",
                            "   cd crates/wasm-plugin",
                            "   wasm-pack build --target web",
                            "",
                            "2. 插件结构:",
                            "   - PluginMetadata: 插件元数据",
                            "   - PluginState: 状态数据 (JSON)",
                            "   - CalculatorPlugin: 插件实现",
                            "",
                            "3. 主机集成:",
                            "   - 加载 WASM 文件",
                            "   - 调用导出函数",
                            "   - 处理返回值",
                        ]
                        .iter()
                        .map(|line| {
                            div()
                                .text_size(px(10.))
                                .text_color(theme.colors.muted_foreground)
                                .child(line.to_string())
                        }),
                    ),
            )
    }
}

/// 初始化 WASM 示例
pub fn init_example(_cx: &mut App) {
    tracing::info!("🔌 WASM 插件示例初始化");
    tracing::info!("📁 WASM 文件位置: crates/wasm-plugin/pkg/");
    tracing::info!("🚀 这是一个展示 WASM 插件概念的示例");
}
