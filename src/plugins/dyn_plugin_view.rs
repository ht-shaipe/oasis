//! cdylib 插件通用视图
//!
//! 封装 Arc<dyn Plugin>，实现 Render 接口，内部通过 UiSchema 渲染。
//! 与 WasmPluginView 共用 render_node + ActionHandler，渲染逻辑完全一致。

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use gpui::{
    div, px, AppContext as _, Context, Entity, IntoElement, ParentElement, Render, SharedString,
    Styled as _, Subscription, Window,
};
use gpui_component::ActiveTheme as _;
use gpui_component::input::{Input, InputEvent, InputState};
use plugin_sdk::Plugin;

use super::wasm_plugin_view::{render_header, render_node, ActionHandler, RenderContext};

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
                    render_node(node, &self.state, &ctx, Arc::new(NoopHandler), idx, None)
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
    /// 输入框状态（按 bind 路径索引）
    input_states: RefCell<HashMap<String, Entity<InputState>>>,
    /// InputState 变更订阅（保持存活）
    _input_subscriptions: RefCell<Vec<Subscription>>,
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
            input_states: RefCell::new(HashMap::new()),
            _input_subscriptions: RefCell::new(Vec::new()),
        }
    }

    /// 确保 ui_schema 中所有 input 节点都有对应的 InputState
    fn ensure_input_states(&self, window: &mut Window, cx: &mut Context<Self>) {
        let state = self.state.borrow();
        let mut new_binds: Vec<(String, String, String)> = Vec::new();
        collect_input_binds(&self.ui_schema.children, &state, &mut new_binds);
        drop(state);

        let mut input_states = self.input_states.borrow_mut();

        // 移除不存在的 bind
        let bind_set: std::collections::HashSet<String> =
            new_binds.iter().map(|(b, _, _)| b.clone()).collect();
        input_states.retain(|bind, _| bind_set.contains(bind));

        // 为新 bind 创建 InputState
        for (bind, placeholder, initial_value) in new_binds {
            if input_states.contains_key(&bind) {
                continue;
            }

            let input_state = cx.new(|cx| {
                let mut s = InputState::new(window, cx);
                if !placeholder.is_empty() {
                    s.set_placeholder(SharedString::from(placeholder), window, cx);
                }
                if !initial_value.is_empty() {
                    s.set_value(&initial_value, window, cx);
                }
                s
            });

            // 订阅文本变更 → 更新插件状态
            let plugin = Arc::clone(&self.plugin);
            let bind_clone = bind.clone();
            let weak_self = cx.entity().downgrade();
            let sub = cx.subscribe(&input_state, move |_view, source, event, cx| {
                if matches!(event, InputEvent::Change) {
                    let text = source.read(cx).text().to_string();
                    let action = format!("set_bind:{}", bind_clone);
                    let params = serde_json::json!({"bind": &bind_clone, "value": text});
                    let new_state = plugin.handle_action(&action, params);
                    *_view.state.borrow_mut() = new_state;
                    // 不重建 ui_schema，避免打断输入
                }
            });

            self._input_subscriptions.borrow_mut().push(sub);
            input_states.insert(bind, input_state);
        }
    }

    /// 执行动作并刷新状态（由 on_click 通过 Entity::update 调用）
    fn handle_action(&mut self, action: String, cx: &mut Context<Self>) {
        // 检测文件/目录选择动作，打开系统对话框
        let is_pick_file = action.contains(":pick_file");
        let is_pick_dir = action.contains(":pick_dir");

        if is_pick_file || is_pick_dir {
            let plugin = Arc::clone(&self.plugin);
            let entity = cx.entity().downgrade();

            gpui::App::spawn(cx, async move |async_cx| {
                let title = if is_pick_dir { "选择目录" } else { "选择文件" };
                let selected = smol::unblock(move || {
                    if is_pick_dir {
                        rfd::FileDialog::new().set_title(title).pick_folder()
                    } else {
                        rfd::FileDialog::new().set_title(title).pick_file()
                    }
                })
                .await;

                if let Some(path) = selected {
                    let path_str = path.to_string_lossy().to_string();
                    let params = serde_json::json!({"path": path_str});
                    let new_state = plugin.handle_action(&action, params);
                    if let Some(e) = entity.upgrade() {
                        e.update(async_cx, |view, cx| {
                            *view.state.borrow_mut() = new_state;
                            view.ui_schema = view.plugin.ui_schema();
                            cx.notify();
                        })
                        .ok();
                    }
                }
            })
            .detach();
            return;
        }

        // 普通动作：同步处理
        let new_state = self.plugin.handle_action(&action, serde_json::Value::Null);
        *self.state.borrow_mut() = new_state;

        // 重新获取 ui_schema 以反映状态变化（如 selected_tool 改变）
        self.ui_schema = self.plugin.ui_schema();

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
// 递归收集 input 节点
// ---------------------------------------------------------------------------

fn collect_input_binds(
    children: &[ui_schema::UiNode],
    state: &serde_json::Value,
    result: &mut Vec<(String, String, String)>,
) {
    for node in children {
        if node.component == "input" {
            if let Some(bind) = &node.bind {
                let placeholder =
                    ui_schema::prop_str_or(&node.props, "placeholder", "").to_string();
                let initial_value = ui_schema::state_get_str(state, bind);
                result.push((bind.clone(), placeholder, initial_value));
            }
        }
        collect_input_binds(&node.children, state, result);
    }
}

// ---------------------------------------------------------------------------
// Render
// ---------------------------------------------------------------------------

impl Render for DynPluginView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // 确保所有 input 节点都有 InputState
        self.ensure_input_states(_window, cx);

        let theme = cx.theme();
        let ctx = RenderContext::from_theme(&theme);
        let state = self.state.borrow();

        let action_handler: Arc<dyn ActionHandler> =
            Arc::new(DynActionHandler { entity: cx.entity().downgrade() });

        let input_states = self.input_states.borrow();

        // 根据 ui_schema 的 layout 属性创建容器
        let is_row = self.ui_schema.layout == "flex-row";
        let gap = px(self.ui_schema.gap as f32);

        let mut container = div()
            .flex()
            .gap(gap)
            .size_full()
            .overflow_hidden()
            .bg(theme.colors.background);

        // 设置布局方向
        if is_row {
            container = container.flex_row();
        } else {
            container = container.flex_col();
        }

        // 设置对齐方式
        if let Some(align) = &self.ui_schema.align_items {
            match align.as_str() {
                "center" => { container = container.items_center(); }
                "start" => { container = container.items_start(); }
                "end" => { container = container.items_end(); }
                "stretch" => { /* stretch is default in flex */ }
                _ => {}
            }
        }

        // 设置主轴对齐
        if let Some(justify) = &self.ui_schema.justify_content {
            match justify.as_str() {
                "center" => { container = container.justify_center(); }
                "start" => { container = container.justify_start(); }
                "end" => { container = container.justify_end(); }
                "space-between" => { container = container.justify_between(); }
                "space-around" => { container = container.justify_around(); }
                _ => {}
            }
        }

        let element = container
            .children(
                self.ui_schema.children.iter().enumerate().map(|(idx, node)| {
                    render_node(node, &*state, &ctx, action_handler.clone(), idx, Some(&*input_states))
                }),
            );
        element
    }
}
