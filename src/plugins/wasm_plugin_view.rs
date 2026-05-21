//! WASM / cdylib 插件通用视图
//!
//! 通用 schema 驱动渲染：UiNode.component 字段指定组件类型，
//! 宿主渲染器按字符串分发到对应 gpui-component / 手写渲染函数。
//! 未识别的组件优雅降级为占位符。

use std::sync::Arc;

use gpui::{
    div, px, AnyView, App, AppContext as _, Context, Hsla, InteractiveElement as _, IntoElement,
    ParentElement, Render, SharedString, StatefulInteractiveElement as _, Styled as _, Window,
};
use gpui_component::ActiveTheme as _;

use super::wasm_runtime::WasmLoadedPlugin;
use plugin_sdk::UiNode;
use ui_schema::{state_get, state_get_i64, state_get_str, state_interpolate, WasmManifest};

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
// ActionHandler — render_node 的泛型抽象，让所有插件视图共用同一套渲染
// ---------------------------------------------------------------------------

pub(crate) trait ActionHandler: 'static + Send + Sync {
    fn handle(&self, action: String, cx: &mut gpui::App);
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
        let state = &self.state;

        let ctx = RenderContext::from_theme(&theme);
        let entity = cx.entity().downgrade();
        let action_handler: Arc<dyn ActionHandler> = Arc::new(WasmActionHandler { entity });

        // 将 manifest.ui (serde_json::Value) 反序列化为 UiSchema
        let ui_schema: ui_schema::UiSchema = match serde_json::from_value(manifest.ui.clone()) {
            Ok(v) => v,
            Err(e) => {
                tracing::error!("Failed to parse UI schema: {e}");
                return div().child("UI schema parse error");
            }
        };

        div()
            .flex()
            .flex_col()
            .gap(px(16.))
            .p(px(24.))
            .size_full()
            .bg(theme.colors.background)
            // 标题栏
            .child(render_header(&manifest.title, &manifest.icon, "WASM", &ctx))
            // 逐个渲染 schema children
            .children(
                ui_schema.children.iter().enumerate().map(|(idx, node)| {
                    render_node(node, state, &ctx, action_handler.clone(), idx)
                })
            )
    }
}

// ---------------------------------------------------------------------------
// 渲染上下文（预提取颜色，避免闭包借 theme）
// ---------------------------------------------------------------------------

pub(crate) struct RenderContext {
    pub fg: Hsla,
    pub muted_fg: Hsla,
    pub muted: Hsla,
    pub primary: Hsla,
    pub white: Hsla,
    pub background: Hsla,
    pub border: Hsla,
    pub card: Hsla,
}

impl RenderContext {
    pub fn from_theme(theme: &gpui_component::theme::Theme) -> Self {
        Self {
            fg: theme.colors.foreground,
            muted_fg: theme.colors.muted_foreground,
            muted: theme.colors.muted,
            primary: theme.colors.primary,
            white: gpui::rgb(0xffffff).into(),
            background: theme.colors.background,
            border: theme.colors.border,
            card: theme.colors.group_box,
        }
    }
}

// ---------------------------------------------------------------------------
// 标题栏渲染
// ---------------------------------------------------------------------------

pub(crate) fn render_header(title: &str, icon: &str, badge: &str, ctx: &RenderContext) -> gpui::AnyElement {
    div()
        .flex()
        .items_center()
        .gap(px(12.))
        .child(div().text_size(px(24.)).child(icon.to_string()))
        .child(
            div()
                .text_size(px(18.))
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .text_color(ctx.fg)
                .child(title.to_string()),
        )
        .child(
            div()
                .px(px(6.))
                .py(px(2.))
                .rounded(px(4.))
                .bg(ctx.primary.opacity(0.15))
                .text_size(px(10.))
                .text_color(ctx.primary)
                .child(badge.to_string()),
        )
        .into_any_element()
}

// ---------------------------------------------------------------------------
// 节点渲染 — 按 component 字段分发
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
pub(crate) fn render_node(
    node: &UiNode,
    state: &serde_json::Value,
    ctx: &RenderContext,
    handler: Arc<dyn ActionHandler>,
    node_idx: usize,
) -> gpui::AnyElement {
    match node.component.as_str() {
        // 已有组件
        "display" => render_display(node, state, ctx),
        "label" => render_label(node, state, ctx),
        "progress" => render_progress(node, state, ctx),
        "button" => render_button(node, ctx, handler, node_idx),
        "button_row" => render_button_row(node, ctx, handler, node_idx),
        "info" => render_info(node, state, ctx),
        "divider" => render_divider(ctx),
        "input" => render_input(node, state, ctx, handler, node_idx),
        "table" => render_table(node, state, ctx),
        "card" => render_card(node, state, ctx, handler, node_idx),

        // 分栏 & 树
        "split" => render_split(node, state, ctx, handler, node_idx),
        "tree" => render_tree(node, state, ctx, handler, node_idx),

        // 容器组件
        "form" => render_container(node, state, ctx, handler, node_idx),
        "flex" | "flex-col" | "flex_row" | "flex-row" => render_container(node, state, ctx, handler, node_idx),
        "tab" => render_container(node, state, ctx, handler, node_idx),

        // 未识别 → 占位符
        _ => render_unsupported(node, ctx),
    }
}

// ---------------------------------------------------------------------------
// 具体组件渲染函数
// ---------------------------------------------------------------------------

fn render_display(node: &UiNode, state: &serde_json::Value, ctx: &RenderContext) -> gpui::AnyElement {
    let field = node.bind.as_deref().unwrap_or("");
    let style = ui_schema::prop_str_or(&node.props, "style", "");

    let text_size = if style == "large_number" {
        px(64.)
    } else if style == "large_text" {
        px(18.)
    } else {
        px(32.)
    };

    let value = if !field.is_empty() {
        state_get_str(state, field)
    } else {
        // 尝试从 props.text 取
        ui_schema::prop_str_or(&node.props, "text", "").to_string()
    };

    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(20.))
        .p(px(32.))
        .bg(ctx.muted.opacity(0.3))
        .rounded_lg()
        .child(
            div()
                .text_size(text_size)
                .font_weight(gpui::FontWeight::BOLD)
                .text_color(ctx.fg)
                .child(value),
        )
        .into_any_element()
}

fn render_label(node: &UiNode, state: &serde_json::Value, ctx: &RenderContext) -> gpui::AnyElement {
    let text = ui_schema::prop_str_or(&node.props, "text", "");
    let interpolated = state_interpolate(state, text);
    let text_size = ui_schema::prop_i64(&node.props, "size").map(|s| px(s as f32)).unwrap_or(px(14.));

    div()
        .flex()
        .justify_center()
        .child(
            div()
                .text_size(text_size)
                .text_color(ctx.muted_fg)
                .child(interpolated),
        )
        .into_any_element()
}

fn render_progress(node: &UiNode, state: &serde_json::Value, ctx: &RenderContext) -> gpui::AnyElement {
    let field = node.bind.as_deref().unwrap_or("progress");
    let pct = state_get_i64(state, field) as f32;
    let bar_width = 240.0_f32;
    let fill_width = bar_width * pct / 100.0;

    div()
        .flex()
        .justify_center()
        .child(
            div()
                .w(px(bar_width))
                .h(px(12.))
                .bg(ctx.muted)
                .rounded_full()
                .child(
                    div()
                        .h(px(12.))
                        .bg(ctx.primary)
                        .rounded_full()
                        .flex_shrink_0()
                        .w(px(fill_width)),
                ),
        )
        .into_any_element()
}

fn render_button(
    node: &UiNode,
    ctx: &RenderContext,
    handler: Arc<dyn ActionHandler>,
    node_idx: usize,
) -> gpui::AnyElement {
    let label = ui_schema::prop_str_or(&node.props, "label", "Button").to_string();
    let variant = ui_schema::prop_str_or(&node.props, "variant", "").to_string();
    let action = node.on_action.clone().unwrap_or_default();

    let is_primary = variant == "primary" || !action.is_empty();
    let bg = if is_primary { ctx.primary } else { ctx.muted };
    let text_color = if is_primary { ctx.white } else { ctx.fg };

    let btn_id = SharedString::from(format!(
        "btn-{}-{}",
        node.id.as_deref().unwrap_or(&node_idx.to_string()),
        node_idx
    ));

    div()
        .id(btn_id)
        .flex()
        .items_center()
        .justify_center()
        .px(px(16.))
        .py(px(8.))
        .bg(bg)
        .rounded_lg()
        .cursor_pointer()
        .text_size(px(14.))
        .text_color(text_color)
        .child(label)
        .on_click(move |_ev, _window, cx| {
            let action = action.clone();
            if !action.is_empty() {
                handler.handle(action, cx);
            }
        })
        .into_any_element()
}

fn render_button_row(
    node: &UiNode,
    ctx: &RenderContext,
    handler: Arc<dyn ActionHandler>,
    node_idx: usize,
) -> gpui::AnyElement {
    // 兼容旧格式：props.buttons 数组
    let buttons = ui_schema::prop_array(&node.props, "buttons");

    let mut row = div()
        .flex()
        .flex_row()
        .gap(px(16.))
        .justify_center();

    if let Some(btns) = buttons {
        for (i, btn) in btns.iter().enumerate() {
            let label = btn.get("label").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let action = btn.get("action").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let variant = btn.get("variant").and_then(|v| v.as_str()).unwrap_or("").to_string();

            let is_primary = variant == "primary";
            let bg = if is_primary { ctx.primary } else { ctx.muted };
            let text_color = if is_primary { ctx.white } else { ctx.fg };
            let handler = handler.clone();
            let btn_id = SharedString::from(format!("btn-row-{}-{}", node_idx, i));

            row = row.child(
                div()
                    .id(btn_id)
                    .flex()
                    .items_center()
                    .justify_center()
                    .px(px(16.))
                    .py(px(8.))
                    .bg(bg)
                    .rounded_lg()
                    .cursor_pointer()
                    .text_size(px(14.))
                    .text_color(text_color)
                    .child(label)
                    .on_click(move |_ev, _window, cx| {
                        let action = action.clone();
                        if !action.is_empty() {
                            handler.handle(action, cx);
                        }
                    }),
            );
        }
    }

    // 同时渲染 children 中的 button 节点
    for (i, child) in node.children.iter().enumerate() {
        if child.component == "button" {
            row = row.child(render_button(child, ctx, handler.clone(), node_idx * 100 + i));
        }
    }

    row.into_any_element()
}

fn render_info(node: &UiNode, state: &serde_json::Value, ctx: &RenderContext) -> gpui::AnyElement {
    let fields = ui_schema::prop_array(&node.props, "fields");

    let mut container = div()
        .flex()
        .flex_col()
        .gap(px(8.))
        .p(px(16.))
        .bg(ctx.muted.opacity(0.2))
        .rounded_lg()
        .child(
            div()
                .text_size(px(12.))
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .text_color(ctx.muted_fg)
                .child("ℹ️ 插件信息"),
        );

    if let Some(fields) = fields {
        for f in fields {
            let label = f.get("label").and_then(|v| v.as_str()).unwrap_or("");
            let field = f.get("field").and_then(|v| v.as_str()).unwrap_or("");
            let value = state_get_str(state, field);
            container = container.child(
                div()
                    .text_size(px(11.))
                    .text_color(ctx.muted_fg)
                    .child(format!("{}: {}", label, value)),
            );
        }
    }

    container.into_any_element()
}

fn render_divider(ctx: &RenderContext) -> gpui::AnyElement {
    div()
        .w_full()
        .h(px(1.))
        .bg(ctx.border)
        .into_any_element()
}

fn render_input(
    node: &UiNode,
    state: &serde_json::Value,
    ctx: &RenderContext,
    handler: Arc<dyn ActionHandler>,
    node_idx: usize,
) -> gpui::AnyElement {
    let placeholder = ui_schema::prop_str_or(&node.props, "placeholder", "").to_string();
    let bind = node.bind.clone().unwrap_or_default();
    let value = if !bind.is_empty() {
        state_get_str(state, &bind)
    } else {
        String::new()
    };
    let _action = node.on_action.clone();

    let input_id = SharedString::from(format!(
        "input-{}-{}",
        node.id.as_deref().unwrap_or(&node_idx.to_string()),
        node_idx
    ));

    div()
        .id(input_id)
        .flex()
        .items_center()
        .px(px(12.))
        .py(px(8.))
        .bg(ctx.muted.opacity(0.3))
        .rounded_lg()
        .border_1()
        .border_color(ctx.border)
        .cursor_text()
        .child(
            if value.is_empty() {
                div()
                    .text_size(px(14.))
                    .text_color(ctx.muted_fg)
                    .child(placeholder)
            } else {
                div()
                    .text_size(px(14.))
                    .text_color(ctx.fg)
                    .child(value)
            },
        )
        .into_any_element()
    // TODO: 接入 gpui-component Input 组件后替换此简易占位
}

fn render_table(node: &UiNode, state: &serde_json::Value, ctx: &RenderContext) -> gpui::AnyElement {
    let bind = node.bind.clone().unwrap_or_default();
    let columns = ui_schema::prop_array(&node.props, "columns");

    let mut table = div()
        .flex()
        .flex_col()
        .border_1()
        .border_color(ctx.border)
        .rounded_lg()
        .overflow_hidden();

    // 表头
    if let Some(cols) = columns {
        let mut header = div()
            .flex()
            .flex_row()
            .bg(ctx.muted.opacity(0.3))
            .border_b_1()
            .border_color(ctx.border);

        for col in cols {
            let col_name = col.as_str().unwrap_or("");
            header = header.child(
                div()
                    .flex_1()
                    .px(px(12.))
                    .py(px(8.))
                    .text_size(px(12.))
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(ctx.muted_fg)
                    .child(col_name.to_string()),
            );
        }
        table = table.child(header);
    }

    // 数据行
    if !bind.is_empty() {
        if let Some(rows) = state_get(state, &bind).and_then(|v| v.as_array()) {
            for row in rows {
                let mut row_div = div()
                    .flex()
                    .flex_row()
                    .border_b_1()
                    .border_color(ctx.border.opacity(0.5));

                if let Some(cols) = columns {
                    for col in cols {
                        let col_key = col.as_str().unwrap_or("");
                        let cell_val = row.get(col_key)
                            .map(|v| match v {
                                serde_json::Value::String(s) => s.clone(),
                                serde_json::Value::Number(n) => n.to_string(),
                                serde_json::Value::Bool(b) => b.to_string(),
                                _ => v.to_string(),
                            })
                            .unwrap_or_default();
                        row_div = row_div.child(
                            div()
                                .flex_1()
                                .px(px(12.))
                                .py(px(6.))
                                .text_size(px(12.))
                                .text_color(ctx.fg)
                                .child(cell_val),
                        );
                    }
                } else {
                    // 无 columns 定义 → 显示 JSON
                    row_div = row_div.child(
                        div()
                            .flex_1()
                            .px(px(12.))
                            .py(px(6.))
                            .text_size(px(12.))
                            .text_color(ctx.fg)
                            .child(row.to_string()),
                    );
                }

                table = table.child(row_div);
            }
        }
    }

    table.into_any_element()
}

fn render_card(
    node: &UiNode,
    state: &serde_json::Value,
    ctx: &RenderContext,
    handler: Arc<dyn ActionHandler>,
    node_idx: usize,
) -> gpui::AnyElement {
    let title = ui_schema::prop_str_or(&node.props, "title", "").to_string();

    let mut card = div()
        .flex()
        .flex_col()
        .gap(px(12.))
        .p(px(16.))
        .bg(ctx.card)
        .rounded_lg()
        .border_1()
        .border_color(ctx.border);

    if !title.is_empty() {
        card = card.child(
            div()
                .text_size(px(14.))
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .text_color(ctx.fg)
                .child(title),
        );
    }

    for (i, child) in node.children.iter().enumerate() {
        card = card.child(render_node(child, state, ctx, handler.clone(), node_idx * 100 + i));
    }

    card.into_any_element()
}

/// 通用容器渲染（form / flex / tab 等容器组件）
fn render_container(
    node: &UiNode,
    state: &serde_json::Value,
    ctx: &RenderContext,
    handler: Arc<dyn ActionHandler>,
    node_idx: usize,
) -> gpui::AnyElement {
    let direction = match node.component.as_str() {
        "flex_row" | "flex-row" => "row",
        _ => "col",
    };

    let gap = ui_schema::prop_i64(&node.props, "gap").map(|g| px(g as f32)).unwrap_or(px(8.));

    let mut container = div().flex().gap(gap);
    if direction == "col" {
        container = container.flex_col();
    }

    for (i, child) in node.children.iter().enumerate() {
        container = container.child(render_node(child, state, ctx, handler.clone(), node_idx * 100 + i));
    }

    container.into_any_element()
}

/// 未识别组件 → 占位符
fn render_unsupported(node: &UiNode, ctx: &RenderContext) -> gpui::AnyElement {
    tracing::warn!("Unsupported component: {}", node.component);
    div()
        .flex()
        .items_center()
        .justify_center()
        .p(px(12.))
        .bg(ctx.muted.opacity(0.1))
        .rounded_lg()
        .border_1()
        .border_dashed()
        .border_color(ctx.border)
        .child(
            div()
                .text_size(px(12.))
                .text_color(ctx.muted_fg)
                .child(format!("⚠ Unsupported: {}", node.component)),
        )
        .into_any_element()
}

// ---------------------------------------------------------------------------
// 分栏容器（左右/上下分割）
// ---------------------------------------------------------------------------

fn render_split(
    node: &UiNode,
    state: &serde_json::Value,
    ctx: &RenderContext,
    handler: Arc<dyn ActionHandler>,
    node_idx: usize,
) -> gpui::AnyElement {
    let direction = ui_schema::prop_str_or(&node.props, "direction", "row");
    let left_width = ui_schema::prop_i64(&node.props, "left_width").unwrap_or(300) as f32;
    let gap = ui_schema::prop_i64(&node.props, "gap").unwrap_or(1) as f32;

    // 必须有至少 2 个子节点
    let children: Vec<_> = node.children.iter().collect();
    if children.len() < 2 {
        return div()
            .child("⚠ split: need at least 2 children")
            .into_any_element();
    }

    let is_row = direction == "row";

    let mut container = div().flex().gap(px(gap));
    if !is_row {
        container = container.flex_col();
    }

    if is_row {
        // 左右分栏：左侧固定宽度，右侧 flex-1
        container = container.child(
            div()
                .w(px(left_width))
                .min_w(px(200.))
                .max_w(px(600.))
                .h_full()
                .overflow_hidden()
                .child(render_node(
                    children[0],
                    state,
                    ctx,
                    handler.clone(),
                    node_idx * 100 + 0,
                )),
        );
        container = container.child(
            div()
                .flex_1()
                .h_full()
                .overflow_hidden()
                .child(render_node(
                    children[1],
                    state,
                    ctx,
                    handler.clone(),
                    node_idx * 100 + 1,
                )),
        );
    } else {
        // 上下分栏：上方固定高度，下方 flex-1
        for (i, child) in children.iter().enumerate() {
            let is_last = i == children.len() - 1;
            if is_last {
                container = container.child(
                    div()
                        .flex_1()
                        .w_full()
                        .overflow_hidden()
                        .child(render_node(
                            child,
                            state,
                            ctx,
                            handler.clone(),
                            node_idx * 100 + i,
                        )),
                );
            } else {
                let h = if i == 0 {
                    px(left_width)
                } else {
                    px(200.)
                };
                container = container.child(
                    div()
                        .h(h)
                        .w_full()
                        .overflow_hidden()
                        .child(render_node(
                            child,
                            state,
                            ctx,
                            handler.clone(),
                            node_idx * 100 + i,
                        )),
                );
            }
        }
    }

    container.into_any_element()
}

// ---------------------------------------------------------------------------
// 文件树组件
// ---------------------------------------------------------------------------

fn render_tree(
    node: &UiNode,
    state: &serde_json::Value,
    ctx: &RenderContext,
    handler: Arc<dyn ActionHandler>,
    node_idx: usize,
) -> gpui::AnyElement {
    let bind = node.bind.clone().unwrap_or_default();
    let tree_data: Vec<&serde_json::Value> = if !bind.is_empty() {
        state_get(state, &bind)
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().collect())
            .unwrap_or_default()
    } else {
        vec![]
    };

    let on_action = node.on_action.clone().unwrap_or_default();
    let handler = handler.clone();

    let mut tree_view = div()
        .flex()
        .flex_col()
        .w_full()
        .h_full()
        .overflow_hidden()
        .gap(px(2.))
        .p(px(8.));

    // 渲染树：递归展开顶级条目
    for (i, entry) in tree_data.iter().enumerate() {
        tree_view = tree_view.child(render_tree_entry(
            entry,
            state,
            ctx,
            handler.clone(),
            &on_action,
            node_idx * 100 + i,
            0,
        ));
    }

    tree_view.into_any_element()
}

/// 渲染单个树节点（递归）
fn render_tree_entry(
    entry: &serde_json::Value,
    state: &serde_json::Value,
    ctx: &RenderContext,
    handler: Arc<dyn ActionHandler>,
    on_action: &str,
    entry_idx: usize,
    depth: usize,
) -> gpui::AnyElement {
    let name = entry.get("name").and_then(|v| v.as_str()).unwrap_or("");
    let is_dir = entry.get("is_dir").and_then(|v| v.as_bool()).unwrap_or(false);
    let path = entry
        .get("path")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let children = entry.get("children").and_then(|v| v.as_array());

    let indent = px(16.) * depth as f32;
    let icon = if is_dir { "📁" } else { "📄" };

    let entry_id = SharedString::from(format!("tree-entry-{}-{}", entry_idx, depth));

    let mut row = div()
        .id(entry_id)
        .flex()
        .items_center()
        .gap(px(6.))
        .px(px(6.))
        .py(px(3.))
        .rounded_md()
        .cursor_pointer()
        .text_size(px(13.))
        .text_color(ctx.fg)
        .child(
            div()
                .w(indent)
                .h(px(1.)), // indent spacer
        )
        .child(div().text_size(px(14.)).child(icon))
        .child(div().child(name.to_string()));

    if !on_action.is_empty() {
        let on_action = on_action.to_string();
        let path = path.to_string();
        let click_handler = handler.clone();
        row = row.on_click(move |_ev, _window, cx| {
            let action = format!("{}", on_action);
            // 将选中路径注入 action 参数
            // 约定：action 可含 {path} 占位符
            let action = action.replace("{path}", &path);
            click_handler.handle(action, cx);
        });
    }

    let mut container = div().flex().flex_col().w_full();
    container = container.child(row);

    // 递归渲染子节点
    if let Some(children) = children {
        for (i, child) in children.iter().enumerate() {
            container = container.child(render_tree_entry(
                child,
                state,
                ctx,
                handler.clone(),
                on_action,
                entry_idx * 100 + i,
                depth + 1,
            ));
        }
    }

    container.into_any_element()
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
