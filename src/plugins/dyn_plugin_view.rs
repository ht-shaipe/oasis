//! cdylib 插件通用视图
//!
//! 封装 Arc<dyn Plugin>，实现 Render 接口，内部通过 UiSchema 渲染。
//! 与 WasmPluginView 共用 render_node + ActionHandler，渲染逻辑完全一致。

use std::cell::RefCell;
use std::sync::Arc;

use gpui::{
    div, px, AppContext as _, Context, IntoElement, ParentElement, Render, Styled as _, Window,
};
use gpui_component::ActiveTheme as _;

use super::wasm_plugin_view::{render_header, render_node, ActionHandler, RenderContext};
use plugin_sdk::Plugin;

// ---------------------------------------------------------------------------
// TestPluginView — 测试视图（跳过 cdylib Plugin）
// ---------------------------------------------------------------------------

/// 测试视图：直接用静态 UiSchema 渲染，不调用 cdylib Plugin trait
pub struct TestPluginView {
    pub schema: ui_schema::UiSchema,
    pub state: serde_json::Value,
}

impl Render for TestPluginView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let ctx = RenderContext::from_theme(&theme);
        div()
            .flex()
            .flex_col()
            .gap(px(16.))
            .p(px(24.))
            .size_full()
            .bg(theme.colors.background)
            .child(render_header("Test Plugin", "🔧", "test", &ctx))
            .children(
                self.schema.children.iter().enumerate().map(|(idx, node)| {
                    render_node(node, &self.state, &ctx, Arc::new(NoopHandler), idx)
                }),
            )
    }
}

struct NoopHandler;
impl ActionHandler for NoopHandler {
    fn handle(&self, _action: String, _cx: &mut gpui::App) {}
}

// ---------------------------------------------------------------------------
// DynPluginView — cdylib 插件视图
// ---------------------------------------------------------------------------

pub struct DynPluginView {
    /// 插件实例（Arc 共享，handle_action 用 &self 不需要可变）
    plugin: Arc<dyn Plugin>,
    /// 状态快照（RefCell 内部可变，供 handle_action 后刷新）
    state: RefCell<serde_json::Value>,
    /// UI schema（声明式布局）
    ui_schema: ui_schema::UiSchema,
    /// 插件元数据（仅用于标题栏渲染）
    meta: plugin_sdk::PluginMeta,
}

impl DynPluginView {
    /// 从 Arc<dyn Plugin> 创建视图（由 PluginRegistry 调用）
    pub fn create_from_plugin(plugin: Arc<dyn Plugin>) -> Self {
        tracing::info!("🧪 DynPluginView::create_from_plugin 开始");
        let meta = plugin.meta();
        tracing::info!("🧪 meta: {} ({})", meta.name, meta.id);
        let state = plugin.state();
        tracing::info!("🧪 state acquired");
        let ui_schema = plugin.ui_schema();
        tracing::info!("🧪 ui_schema acquired, children: {}", ui_schema.children.len());
        tracing::info!("✅ cdylib 插件视图创建: {} ({})", meta.name, meta.id);
        Self {
            plugin,
            state: RefCell::new(state),
            ui_schema,
            meta,
        }
    }

    /// 执行动作并刷新状态（由 on_click 通过 Entity::update 调用）
    fn handle_action(&mut self, action: String, cx: &mut Context<Self>) {
        let new_state = self.plugin.handle_action(&action, serde_json::Value::Null);
        *self.state.borrow_mut() = new_state;
        cx.notify();
    }
}

// ---------------------------------------------------------------------------
// ActionHandler — 供 render_node 调用，更新 DynPluginView 状态
// ---------------------------------------------------------------------------

pub(crate) struct DynActionHandler {
    entity: gpui::WeakEntity<DynPluginView>,
}

impl ActionHandler for DynActionHandler {
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
// Render
// ---------------------------------------------------------------------------

impl Render for DynPluginView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        tracing::info!("🧪 DynPluginView::render START for '{}'", self.meta.name);
        let theme = cx.theme();
        tracing::info!("🧪 DynPluginView::render got theme");
        let ctx = RenderContext::from_theme(&theme);
        tracing::info!("🧪 DynPluginView::render got ctx");
        let meta = &self.meta;
        tracing::info!("🧪 DynPluginView::render meta ok");
        let state = self.state.borrow();
        tracing::info!("🧪 DynPluginView::render state borrowed, children={}", self.ui_schema.children.len());

        let action_handler: Arc<dyn ActionHandler> =
            Arc::new(DynActionHandler { entity: cx.entity().downgrade() });

        tracing::info!("🧪 DynPluginView::render building div");
        let element = div()
            .flex()
            .flex_col()
            .gap(px(16.))
            .p(px(24.))
            .size_full()
            .bg(theme.colors.background)
            // 标题栏
            .child(render_header(&meta.name, &meta.icon, "cdylib", &ctx))
            // 逐个渲染 schema children
            .children(
                self.ui_schema.children.iter().enumerate().map(|(idx, node)| {
                    render_node(node, &*state, &ctx, action_handler.clone(), idx)
                }),
            );
        tracing::info!("🧪 DynPluginView::render DONE");
        element
    }
}
