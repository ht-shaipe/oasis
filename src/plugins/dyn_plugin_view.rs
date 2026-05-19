//! cdylib 插件通用视图
//!
//! 封装 Arc<dyn Plugin>，实现 Render 接口，内部通过 UiSchema 渲染。
//! 与 WasmPluginView 共用 render_node + ActionHandler，渲染逻辑完全一致。

use std::cell::RefCell;
use std::sync::Arc;

use gpui::{
    div, px, App, AppContext as _, Context, Hsla, InteractiveElement as _,
    IntoElement, ParentElement, Render, StatefulInteractiveElement as _, Styled as _, Window,
};
use gpui_component::ActiveTheme as _;

use super::wasm_plugin_view::{render_node, ActionHandler};
use plugin_sdk::{Plugin, UiSchema};

// ---------------------------------------------------------------------------
// DynPluginView — cdylib 插件视图
// ---------------------------------------------------------------------------

pub struct DynPluginView {
    /// 插件实例（Arc 共享，handle_action 用 &self 不需要可变）
    plugin: Arc<dyn Plugin>,
    /// 状态快照（RefCell 内部可变，供 handle_action 后刷新）
    state: RefCell<serde_json::Value>,
    /// UI schema（声明式布局）
    ui_schema: UiSchema,
    /// 插件元数据（仅用于标题栏渲染）
    meta: plugin_sdk::PluginMeta,
}

impl DynPluginView {
    /// 从 Arc<dyn Plugin> 创建视图（由 PluginRegistry 调用）
    pub fn create_from_plugin(plugin: Arc<dyn Plugin>) -> Self {
        let meta = plugin.meta();
        let state = plugin.state();
        let ui_schema = plugin.ui_schema();
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
        let theme = cx.theme();
        let meta = &self.meta;
        let state = self.state.borrow();

        // 预提取颜色值（避免闭包中借 theme）
        let fg = theme.colors.foreground;
        let muted_fg = theme.colors.muted_foreground;
        let muted = theme.colors.muted;
        let primary = theme.colors.primary;
        let white = gpui::rgb(0xffffff).into();

        let action_handler: Arc<dyn ActionHandler> =
            Arc::new(DynActionHandler { entity: cx.entity().downgrade() });

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
                    .child(div().text_size(px(24.)).child(meta.icon.clone()))
                    .child(
                        div()
                            .text_size(px(18.))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(fg)
                            .child(meta.name.clone()),
                    )
                    .child(
                        div()
                            .px(px(6.))
                            .py(px(2.))
                            .rounded(px(4.))
                            .bg(primary.opacity(0.15))
                            .text_size(px(10.))
                            .text_color(primary)
                            .child("cdylib"),
                    ),
            )
            // 逐个渲染 schema children
            .children(
                self.ui_schema.children.iter().enumerate().map(|(idx, node)| {
                    render_node(
                        node,
                        &*state,
                        fg,
                        muted_fg,
                        muted,
                        primary,
                        white,
                        action_handler.clone(),
                        idx,
                    )
                }),
            )
    }
}