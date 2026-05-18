//! WASM 插件通用视图
//!
//! 从 WASM 插件读取 UI schema，通用渲染。
//! 宿主不包含任何插件特定代码，完全由 schema 驱动。

use gpui::{
    div, px, AnyView, App, AppContext as _, Context, Hsla, InteractiveElement as _, IntoElement,
    ParentElement, Render, SharedString, StatefulInteractiveElement as _, Styled as _, Window,
};
use gpui_component::ActiveTheme as _;

use super::wasm_runtime::WasmLoadedPlugin;
use wasm_plugin_types::{UiNode, WasmManifest};

// ---------------------------------------------------------------------------
// 状态读取辅助（独立函数，不 impl 外部类型）
// ---------------------------------------------------------------------------

fn state_get_i32(state: &serde_json::Value, field: &str) -> i32 {
    state.get(field).and_then(|v| v.as_i64()).unwrap_or(0) as i32
}

fn state_get_str(state: &serde_json::Value, field: &str) -> String {
    state.get(field).and_then(|v| v.as_str()).unwrap_or("").to_string()
}

/// 简单插值：将 `{field}` 替换为状态中的值
fn state_interpolate(state: &serde_json::Value, template: &str) -> String {
    let mut result = template.to_string();
    loop {
        let start = result.find('{');
        let end = result.find('}');
        match (start, end) {
            (Some(s), Some(e)) if s < e => {
                let key = &result[s + 1..e];
                let value = state.get(key)
                    .map(|v| match v {
                        serde_json::Value::String(s) => s.clone(),
                        serde_json::Value::Number(n) => n.to_string(),
                        _ => v.to_string(),
                    })
                    .unwrap_or_default();
                result = result.replacen(&format!("{{{}}}", key), &value, 1);
            }
            _ => break,
        }
    }
    result
}

// ---------------------------------------------------------------------------
// WasmPluginView — 通用渲染器
// ---------------------------------------------------------------------------

pub struct WasmPluginView {
    /// WASM 运行时插件实例
    plugin: WasmLoadedPlugin,
    /// 当前状态快照
    state: serde_json::Value,
    /// 清单快照
    manifest: WasmManifest,
}

impl WasmPluginView {
    /// 从 .wasm 文件创建视图（通用工厂）
    pub fn create_from_wasm(wasm_path: &std::path::Path, cx: &mut App) -> AnyView {
        match WasmLoadedPlugin::load(wasm_path) {
            Ok(mut plugin) => {
                let manifest = plugin.manifest.clone();
                let state = plugin.get_state().unwrap_or(serde_json::Value::Null);
                tracing::info!("✅ WASM 插件视图创建: {} ({})", manifest.title, manifest.id);
                cx.new(|_cx| Self { plugin, state, manifest }).into()
            }
            Err(e) => {
                tracing::error!("❌ 加载 WASM 插件失败: {}", e);
                let message = format!("{}", e);
                cx.new(|_cx| WasmPluginErrorView { message }).into()
            }
        }
    }

    /// 通用工厂函数（供 PluginRegistry 调用，签名匹配 create_view）
    pub fn create_view(_window: &mut Window, cx: &mut App) -> AnyView {
        let base_dir = std::env::current_dir().unwrap_or_default();
        let wasm_path = base_dir.join("plugins").join("wasm").join("dsl_counter.wasm");
        Self::create_from_wasm(&wasm_path, cx)
    }

    /// 执行动作并刷新状态
    fn handle_action(&mut self, action: String, cx: &mut Context<Self>) {
        if let Ok(new_state) = self.plugin.handle_action(&action) {
            self.state = new_state;
            cx.notify();
        } else {
            tracing::warn!("WASM 插件动作执行失败: {}", action);
        }
    }
}

// ---------------------------------------------------------------------------
// Render — 通用 schema 驱动渲染
// ---------------------------------------------------------------------------

impl Render for WasmPluginView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let manifest = &self.manifest;
        let state = &self.state;

        // 预提取颜色值（避免闭包中借 theme）
        let fg = theme.colors.foreground;
        let muted_fg = theme.colors.muted_foreground;
        let muted = theme.colors.muted;
        let primary = theme.colors.primary;
        let white = gpui::rgb(0xffffff).into(); // Rgba → Hsla

        let entity = cx.entity().downgrade();

        div()
            .flex()
            .flex_col()
            .gap(px(16.))
            .p(px(24.))
            .size_full()
            .bg(theme.colors.background)
            // 标题栏
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(12.))
                    .child(div().text_size(px(24.)).child(manifest.icon.clone()))
                    .child(
                        div()
                            .text_size(px(18.))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(fg)
                            .child(manifest.title.clone()),
                    )
                    .child(
                        div()
                            .px(px(6.))
                            .py(px(2.))
                            .rounded(px(4.))
                            .bg(primary.opacity(0.15))
                            .text_size(px(10.))
                            .text_color(primary)
                            .child("WASM"),
                    ),
            )
            // 逐个渲染 schema children
            .children(
                manifest.ui.children.iter().enumerate().map(|(idx, node)| {
                    render_node(node, state, fg, muted_fg, muted, primary, white, &entity, idx)
                })
            )
    }
}

// ---------------------------------------------------------------------------
// 节点渲染函数
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
fn render_node(
    node: &UiNode,
    state: &serde_json::Value,
    fg: Hsla,
    muted_fg: Hsla,
    muted: Hsla,
    primary: Hsla,
    white: Hsla,
    entity: &gpui::WeakEntity<WasmPluginView>,
    node_idx: usize,
) -> gpui::Div {
    match node {
        UiNode::Display { field, style } => {
            let value = state_get_i32(state, field);
            let text_size = match style.as_str() {
                "large_number" => px(64.),
                _ => px(32.),
            };
            div()
                .flex()
                .flex_col()
                .items_center()
                .gap(px(20.))
                .p(px(32.))
                .bg(muted.opacity(0.3))
                .rounded_lg()
                .child(
                    div()
                        .text_size(text_size)
                        .font_weight(gpui::FontWeight::BOLD)
                        .text_color(fg)
                        .child(format!("{}", value)),
                )
        }

        UiNode::Label { text } => {
            let interpolated = state_interpolate(state, text);
            div()
                .flex()
                .justify_center()
                .child(
                    div()
                        .text_size(px(14.))
                        .text_color(muted_fg)
                        .child(interpolated),
                )
        }

        UiNode::Progress { field } => {
            let pct = state_get_i32(state, field);
            let bar_width = 240.0_f32;
            let fill_width = bar_width * pct as f32 / 100.0;
            div()
                .flex()
                .justify_center()
                .child(
                    div()
                        .w(px(bar_width))
                        .h(px(12.))
                        .bg(muted)
                        .rounded_full()
                        .child(
                            div()
                                .h(px(12.))
                                .bg(primary)
                                .rounded_full()
                                .flex_shrink_0()
                                .w(px(fill_width)),
                        ),
                )
        }

        UiNode::ButtonRow { buttons } => {
            let mut row = div()
                .flex()
                .flex_row()
                .gap(px(16.))
                .justify_center();

            for (i, btn) in buttons.iter().enumerate() {
                let is_primary = btn.variant == "primary";
                let bg = if is_primary { primary } else { muted };
                let text_color = if is_primary { white } else { fg };
                let action = btn.action.clone();
                let weak = entity.clone();
                let btn_id = SharedString::from(format!("wasm-btn-{}-{}", node_idx, i));

                row = row.child(
                    div()
                        .id(btn_id)
                        .flex()
                        .items_center()
                        .justify_center()
                        .size(px(64.))
                        .bg(bg)
                        .rounded_lg()
                        .cursor_pointer()
                        .text_size(px(28.))
                        .text_color(text_color)
                        .child(btn.label.clone())
                        .on_click(move |_ev, _window, cx| {
                            if let Some(e) = weak.upgrade() {
                                let action = action.clone();
                                e.update(cx, |view, cx| {
                                    view.handle_action(action, cx);
                                });
                            }
                        })
                );
            }

            row
        }

        UiNode::Info { fields } => {
            div()
                .flex()
                .flex_col()
                .gap(px(8.))
                .p(px(16.))
                .bg(muted.opacity(0.2))
                .rounded_lg()
                .child(
                    div()
                        .text_size(px(12.))
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_color(muted_fg)
                        .child("ℹ️ 插件信息"),
                )
                .children(
                    fields.iter().map(|f| {
                        let value = state_get_str(state, &f.field);
                        div()
                            .text_size(px(11.))
                            .text_color(muted_fg)
                            .child(format!("{}: {}", f.label, value))
                    }),
                )
        }
    }
}

// ---------------------------------------------------------------------------
// 错误占位视图
// ---------------------------------------------------------------------------

struct WasmPluginErrorView {
    message: String,
}

impl Render for WasmPluginErrorView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        div()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .size_full()
            .gap(px(16.))
            .bg(theme.colors.background)
            .child(div().text_size(px(48.)).child("⚠️"))
            .child(
                div()
                    .text_size(px(16.))
                    .text_color(theme.colors.red)
                    .child("WASM 插件加载失败"),
            )
            .child(
                div()
                    .text_size(px(12.))
                    .text_color(theme.colors.muted_foreground)
                    .child(self.message.clone()),
            )
    }
}
