//! WASM / cdylib 插件通用视图
//!
//! 通用 schema 驱动渲染：UiNode.component 字段指定组件类型，
//! 宿主渲染器按字符串分发到对应 gpui-component / 手写渲染函数。
//! 未识别的组件优雅降级为占位符。

use gpui::{
    div, px, AnyView, App, AppContext as _, Context, InteractiveElement as _, IntoElement,
    ParentElement, Render, Styled as _, Window,
};
use gpui_component::ActiveTheme as _;

use super::plugin_render::{render_header, ActionHandler, RenderContext};
use super::wasm_runtime::WasmLoadedPlugin;
use ui_schema::WasmManifest;

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

struct WasmActionHandler {
    entity: gpui::WeakEntity<WasmPluginView>,
}

impl ActionHandler for WasmActionHandler {
    fn handle(&self, action: String, cx: &mut gpui::App) {
        if let Some(e) = self.entity.upgrade() {
            let action = action.clone();
            e.update(cx, |view, cx| {
                view.handle_action(action, cx);
            });
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

        let ctx = RenderContext::from_theme(&theme);

        div()
            .flex()
            .flex_col()
            .gap(px(16.))
            .p(px(24.))
            .size_full()
            .bg(theme.colors.background)
            // 标题栏
            .child(render_header(&manifest.title, &manifest.icon, "WASM", &ctx))
            .child(
                div()
                    .p(px(16.))
                    .rounded_lg()
                    .bg(ctx.muted.opacity(0.15))
                    .text_color(ctx.muted_fg)
                    .child("WASM schema rendering is temporarily disabled during startup"),
            )
    }
}

// ---------------------------------------------------------------------------

pub(crate) struct WasmPluginErrorView {
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
