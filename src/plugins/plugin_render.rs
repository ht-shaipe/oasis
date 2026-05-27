//! 插件通用组件渲染器
//!
//! 为 WASM 和 cdylib 插件提供统一的 schema 驱动 UI 渲染。
//! 从 wasm_plugin_view.rs 提取，dyn_plugin_view 和 wasm_plugin_view 共享。

use std::collections::HashMap;
use std::sync::Arc;

use gpui::{
    div, px, Entity, Hsla, InteractiveElement as _, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement as _, Styled as _,
};
use gpui_component::input::{Input, InputState};
use plugin_sdk::UiNode;
use ui_schema::{state_get, state_get_i64, state_get_str, state_interpolate};

use gpui_component::accordion::Accordion;
use gpui_component::alert::{Alert, AlertVariant};
use gpui_component::badge::Badge;
use gpui_component::button::{Button, ButtonGroup, ButtonVariants as _};
use gpui_component::checkbox::Checkbox;
use gpui_component::collapsible::Collapsible;
use gpui_component::group_box::GroupBox;
use gpui_component::kbd::Kbd;
use gpui_component::link::Link;
use gpui_component::radio::Radio;
use gpui_component::skeleton::Skeleton;
use gpui_component::spinner::Spinner;
use gpui_component::switch::Switch;
use gpui_component::tag::Tag;

pub(crate) trait ActionHandler: 'static + Send + Sync {
    fn handle(&self, action: String, cx: &mut gpui::App);
}

// ---------------------------------------------------------------------------
// 渲染上下文（预提取颜色，避免闭包借 theme）
// ---------------------------------------------------------------------------

pub(crate) struct RenderContext {
    pub fg: Hsla,
    pub muted_fg: Hsla,
    pub muted: Hsla,
    pub primary: Hsla,
    pub primary_hover: Hsla,
    pub primary_foreground: Hsla,
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
            primary_hover: theme.colors.primary_hover,
            primary_foreground: theme.colors.primary_foreground,
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

pub(crate) fn render_header(
    title: &str,
    icon: &str,
    badge: &str,
    ctx: &RenderContext,
) -> gpui::AnyElement {
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
    input_states: Option<&HashMap<String, Entity<InputState>>>,
) -> gpui::AnyElement {
    let component = node.component.clone();
    let component = component.as_str();

    if component == "display" {
        render_display(node, state, ctx)
    } else if component == "label" {
        render_label(node, state, ctx)
    } else if component == "progress" {
        render_progress(node, state, ctx)
    } else if component == "button" {
        render_button(node, ctx, handler, node_idx)
    } else if component == "nav-item" {
        render_nav_item(node, ctx, handler, node_idx)
    } else if component == "button_row" {
        render_button_row(node, ctx, handler, node_idx)
    } else if component == "info" {
        render_info(node, state, ctx)
    } else if component == "divider" {
        render_divider(ctx)
    } else if component == "input" {
        render_input(node, state, ctx, handler, node_idx, input_states)
    } else if component == "table" {
        render_table(node, state, ctx, handler, node_idx)
    } else if component == "card" {
        render_card(node, state, ctx, handler, node_idx, input_states)
    } else if component == "split" {
        render_split(node, state, ctx, handler, node_idx, input_states)
    } else if component == "tree" {
        render_tree(node, state, ctx, handler, node_idx, input_states)
    } else if component == "select" {
        render_select(node, state, ctx, handler, node_idx)
    } else if component == "switch" {
        render_switch(node, state, ctx, handler, node_idx)
    } else if component == "button_group" || component == "button-group" {
        render_button_group(node, state, ctx, handler, node_idx)
    } else if component == "form"
        || component == "container"
        || component == "flex"
        || component == "flex-col"
        || component == "flex_row"
        || component == "flex-row"
        || component == "tab"
    {
        render_container(node, state, ctx, handler, node_idx, input_states)
    } else if component == "checkbox" {
        render_checkbox(node, state, ctx, handler, node_idx)
    } else if component == "radio" {
        render_radio(node, state, ctx, handler, node_idx)
    } else if component == "tag" {
        render_tag(node, state, ctx)
    } else if component == "badge" {
        render_badge(node, state, ctx)
    } else if component == "spinner" {
        render_spinner(node, ctx)
    } else if component == "skeleton" {
        render_skeleton(node, ctx)
    } else if component == "alert" {
        render_alert(node, state, ctx, node_idx)
    } else if component == "link" {
        render_link(node, state, ctx, handler, node_idx)
    } else if component == "kbd" {
        render_kbd(node, state, ctx)
    } else if component == "accordion" {
        render_accordion(node, state, ctx, handler, node_idx, input_states)
    } else if component == "collapsible" {
        render_collapsible(node, state, ctx, handler, node_idx, input_states)
    } else if component == "group_box" || component == "group-box" {
        render_group_box(node, state, ctx, handler, node_idx, input_states)
    } else {
        render_unsupported(component, ctx)
    }
}

// ---------------------------------------------------------------------------
// 具体组件渲染函数
// ---------------------------------------------------------------------------

/// Checkbox 复选框组件 — 基于 gpui_component::checkbox::Checkbox
fn render_checkbox(
    node: &UiNode,
    state: &serde_json::Value,
    ctx: &RenderContext,
    handler: Arc<dyn ActionHandler>,
    node_idx: usize,
) -> gpui::AnyElement {
    let bind = node.bind.as_deref().unwrap_or("");
    let checked = if !bind.is_empty() {
        state_get(state, bind)
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    } else {
        false
    };
    let label = ui_schema::prop_str_or(&node.props, "label", "");
    let action = node.on_action.clone().unwrap_or_default();
    let checkbox_id = SharedString::from(format!("checkbox-{}", node_idx));

    let cb = Checkbox::new(checkbox_id.clone()).checked(checked);

    let cb = if !label.is_empty() {
        cb.label(label.to_string())
    } else {
        cb
    };

    if !action.is_empty() {
        let handler = handler.clone();
        cb.on_click(move |_checked, _window, cx| {
            handler.handle(action.clone(), cx);
        })
        .into_any_element()
    } else {
        cb.into_any_element()
    }
}

/// Radio 单选框组件 — 基于 gpui_component::radio::Radio
fn render_radio(
    node: &UiNode,
    state: &serde_json::Value,
    ctx: &RenderContext,
    handler: Arc<dyn ActionHandler>,
    node_idx: usize,
) -> gpui::AnyElement {
    let bind = node.bind.as_deref().unwrap_or("");
    let checked = if !bind.is_empty() {
        state_get(state, bind)
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    } else {
        false
    };
    let label = ui_schema::prop_str_or(&node.props, "label", "");
    let action = node.on_action.clone().unwrap_or_default();
    let radio_id = SharedString::from(format!("radio-{}", node_idx));

    let r = Radio::new(radio_id).checked(checked);

    let r = if !label.is_empty() {
        r.label(label.to_string())
    } else {
        r
    };

    if !action.is_empty() {
        let handler = handler.clone();
        r.on_click(move |_checked, _window, cx| {
            handler.handle(action.clone(), cx);
        })
        .into_any_element()
    } else {
        r.into_any_element()
    }
}

/// Tag 标签组件 — 基于 gpui_component::tag::Tag
fn render_tag(node: &UiNode, state: &serde_json::Value, ctx: &RenderContext) -> gpui::AnyElement {
    let text = ui_schema::prop_str_or(&node.props, "text", "");
    let resolved = if !text.is_empty() {
        text.to_string()
    } else {
        node.bind
            .as_deref()
            .and_then(|b| state_get(state, b))
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_default()
    };
    Tag::new().child(resolved).into_any_element()
}

/// Badge 徽标组件
fn render_badge(node: &UiNode, state: &serde_json::Value, ctx: &RenderContext) -> gpui::AnyElement {
    let text = ui_schema::prop_str_or(&node.props, "text", "");
    let resolved = if !text.is_empty() {
        text.to_string()
    } else {
        node.bind
            .as_deref()
            .and_then(|b| state_get(state, b))
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_default()
    };
    Badge::new().child(resolved).into_any_element()
}

/// Spinner 加载指示器 — 基于 gpui_component::spinner::Spinner
fn render_spinner(node: &UiNode, ctx: &RenderContext) -> gpui::AnyElement {
    Spinner::new().into_any_element()
}

/// Skeleton 骨架屏 — 基于 gpui_component::skeleton::Skeleton
fn render_skeleton(node: &UiNode, ctx: &RenderContext) -> gpui::AnyElement {
    let secondary = ui_schema::prop_bool(&node.props, "secondary").unwrap_or(false);
    let mut sk = Skeleton::new();
    if secondary {
        sk = sk.secondary();
    }
    sk.into_any_element()
}

/// Alert 提示组件 — 基于 gpui_component::alert::Alert
fn render_alert(
    node: &UiNode,
    state: &serde_json::Value,
    ctx: &RenderContext,
    node_idx: usize,
) -> gpui::AnyElement {
    let variant = ui_schema::prop_str_or(&node.props, "variant", "info");
    let v = match variant {
        "success" => AlertVariant::Success,
        "warning" => AlertVariant::Warning,
        "error" => AlertVariant::Error,
        "secondary" => AlertVariant::Secondary,
        _ => AlertVariant::Info,
    };
    let title = ui_schema::prop_str_or(&node.props, "title", "");
    let message = ui_schema::prop_str_or(&node.props, "message", "");

    let alert_id = SharedString::from(format!("alert-{}", node_idx));
    let alert = Alert::new(alert_id, message.to_string()).with_variant(v);
    let alert = if !title.is_empty() {
        alert.title(title.to_string())
    } else {
        alert
    };
    alert.into_any_element()
}

/// Link 链接组件 — 基于 gpui_component::link::Link
fn render_link(
    node: &UiNode,
    state: &serde_json::Value,
    ctx: &RenderContext,
    handler: Arc<dyn ActionHandler>,
    node_idx: usize,
) -> gpui::AnyElement {
    let text = ui_schema::prop_str_or(&node.props, "text", "");
    let resolved = if !text.is_empty() {
        text.to_string()
    } else {
        node.bind
            .as_deref()
            .and_then(|b| state_get(state, b))
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_default()
    };
    let action = node.on_action.clone().unwrap_or_default();
    let link_id = SharedString::from(format!("link-{}", node_idx));
    let href = ui_schema::prop_str_or(&node.props, "href", "");

    let mut link = Link::new(link_id).child(resolved);
    if !href.is_empty() {
        link = link.href(href.to_string());
    }
    if !action.is_empty() {
        let handler = handler.clone();
        link = link.on_click(move |_ev, _window, cx| {
            handler.handle(action.clone(), cx);
        });
    }
    link.into_any_element()
}

/// Kbd 键盘快捷键组件 — 基于 gpui_component::kbd::Kbd
fn render_kbd(node: &UiNode, state: &serde_json::Value, ctx: &RenderContext) -> gpui::AnyElement {
    let key = ui_schema::prop_str_or(&node.props, "key", "");
    let resolved = if !key.is_empty() {
        key.to_string()
    } else {
        node.bind
            .as_deref()
            .and_then(|b| state_get(state, b))
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_default()
    };
    let stroke = gpui::Keystroke::parse(&resolved).unwrap_or(gpui::Keystroke::parse(" ").unwrap());
    Kbd::new(stroke).into_any_element()
}

/// Switch 开关组件 — 使用 gpui_component::switch::Switch
fn render_switch(
    node: &UiNode,
    state: &serde_json::Value,
    _ctx: &RenderContext,
    handler: Arc<dyn ActionHandler>,
    node_idx: usize,
) -> gpui::AnyElement {
    let bind = node.bind.as_deref().unwrap_or("");
    let is_checked = if !bind.is_empty() {
        state_get(state, bind)
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    } else {
        false
    };

    let action = node.on_action.clone().unwrap_or_default();
    let switch_id = SharedString::from(format!("switch-{}", node_idx));

    let mut switch = Switch::new(switch_id).checked(is_checked);

    // 只在有 action 时添加点击处理
    if !action.is_empty() {
        let handler = handler.clone();
        switch = switch.on_click(move |_checked, _window, cx| {
            handler.handle(action.clone(), cx);
        });
    }

    switch.into_any_element()
}

/// 从flex-row容器渲染ButtonGroup - 保持按钮点击功能
fn render_button_group_from_flex_row(
    node: &UiNode,
    _ctx: &RenderContext,
    handler: Arc<dyn ActionHandler>,
    node_idx: usize,
) -> gpui::AnyElement {
    let mut button_group = ButtonGroup::new(SharedString::from(format!("btn-group-{}", node_idx)));

    for (i, child) in node.children.iter().enumerate() {
        if child.component == "button" {
            let label = ui_schema::prop_str_or(&child.props, "label", "Button").to_string();
            let action = child.on_action.clone().unwrap_or_default();
            let variant = ui_schema::prop_str_or(&child.props, "variant", "");
            let btn_id = SharedString::from(format!("btn-{}-{}", node_idx, i));

            let mut btn = Button::new(btn_id).label(label);

            // 设置variant样式
            if variant == "primary" {
                btn = btn.primary();
            } else if variant == "danger" {
                btn = btn.danger();
            } else if variant == "outline" {
                btn = btn.outline();
            } else {
                btn = btn.ghost();
            }

            // 添加点击处理
            if !action.is_empty() {
                let handler = handler.clone();
                let action_clone = action.clone();
                btn = btn.on_click(move |_event, _window, cx| {
                    handler.handle(action_clone.clone(), cx);
                });
            }

            button_group = button_group.child(btn);
        }
    }

    button_group.into_any_element()
}

/// Button Group 按钮组组件 - 处理flex-row中的按钮容器
fn render_button_group(
    node: &UiNode,
    _state: &serde_json::Value,
    _ctx: &RenderContext,
    handler: Arc<dyn ActionHandler>,
    node_idx: usize,
) -> gpui::AnyElement {
    // 处理flex-row中的按钮 - 这是credential_list使用的结构
    if node.component == "flex-row" {
        let mut button_group = ButtonGroup::new(SharedString::from(format!("btn-group-{}", node_idx)));

        for (i, child) in node.children.iter().enumerate() {
            if child.component == "button" {
                let label = ui_schema::prop_str_or(&child.props, "label", "Button").to_string();
                let action = child.on_action.clone().unwrap_or_default();
                let variant = ui_schema::prop_str_or(&child.props, "variant", "");
                let btn_id = SharedString::from(format!("btn-{}-{}", node_idx, i));

                let mut btn = Button::new(btn_id).label(label);

                // 设置variant样式
                if variant == "primary" {
                    btn = btn.primary();
                } else if variant == "danger" {
                    btn = btn.danger();
                } else if variant == "outline" {
                    btn = btn.outline();
                } else {
                    btn = btn.ghost();
                }

                // 添加点击处理
                if !action.is_empty() {
                    let handler = handler.clone();
                    let action_clone = action.clone();
                    btn = btn.on_click(move |_selected_indices, _window, cx| {
                        handler.handle(action_clone.clone(), cx);
                    });
                }

                button_group = button_group.child(btn);
            }
        }

        return button_group.into_any_element();
    }

    // 原有的button_group组件处理逻辑
    let group_id = SharedString::from(format!("btn-group-{}", node_idx));
    let mut button_group = ButtonGroup::new(group_id);

    for (i, child) in node.children.iter().enumerate() {
        if child.component == "button" {
            let label = ui_schema::prop_str_or(&child.props, "label", "Button").to_string();
            let action = child.on_action.clone().unwrap_or_default();
            let variant = ui_schema::prop_str_or(&child.props, "variant", "");
            let btn_id = SharedString::from(format!("btn-{}-{}", node_idx, i));

            let mut btn = Button::new(btn_id).label(label);

            // 设置variant样式
            if variant == "primary" {
                btn = btn.primary();
            } else if variant == "danger" {
                btn = btn.danger();
            } else if variant == "outline" {
                btn = btn.outline();
            } else {
                btn = btn.ghost();
            }

            // 添加点击处理
            if !action.is_empty() {
                let handler = handler.clone();
                let action_clone = action.clone();
                btn = btn.on_click(move |_selected_indices, _window, cx| {
                    handler.handle(action_clone.clone(), cx);
                });
            }

            button_group = button_group.child(btn);
        }
    }

    button_group.into_any_element()
}

fn render_display(
    node: &UiNode,
    state: &serde_json::Value,
    ctx: &RenderContext,
) -> gpui::AnyElement {
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
    let text_size = ui_schema::prop_i64(&node.props, "size")
        .map(|s| px(s as f32))
        .unwrap_or(px(14.));

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

fn render_progress(
    node: &UiNode,
    state: &serde_json::Value,
    ctx: &RenderContext,
) -> gpui::AnyElement {
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
    _ctx: &RenderContext,
    handler: Arc<dyn ActionHandler>,
    node_idx: usize,
) -> gpui::AnyElement {
    let label = ui_schema::prop_str_or(&node.props, "label", "Button").to_string();
    let action = node.on_action.clone().unwrap_or_default();
    let variant = ui_schema::prop_str_or(&node.props, "variant", "");

    let btn_id = SharedString::from(format!(
        "btn-{}-{}",
        node.id.as_deref().unwrap_or(&node_idx.to_string()),
        node_idx
    ));

    let mut btn = Button::new(btn_id).label(label);

    // 直接按照 API 使用，不做任何"修复"
    if variant == "primary" {
        btn = btn.primary();
    } else if variant == "danger" {
        btn = btn.danger();
    } else if variant == "outline" {
        btn = btn.outline();
    } else {
        btn = btn.ghost();
    }

    // 添加点击处理
    if !action.is_empty() {
        let handler = handler.clone();
        btn = btn.on_click(move |_event, _window, cx| {
            handler.handle(action.clone(), cx);
        });
    }

    btn.into_any_element()
}

/// 导航菜单项：无背景、简洁文字、支持 active 高亮
fn render_nav_item(
    node: &UiNode,
    ctx: &RenderContext,
    handler: Arc<dyn ActionHandler>,
    node_idx: usize,
) -> gpui::AnyElement {
    let label = ui_schema::prop_str_or(&node.props, "label", "").to_string();
    let active = ui_schema::prop_bool(&node.props, "active").unwrap_or(false);
    let action = node.on_action.clone().unwrap_or_default();

    let bg = if active {
        ctx.primary.opacity(0.12)
    } else {
        gpui::transparent_black()
    };
    let text_color = if active { ctx.primary } else { ctx.muted_fg };

    let btn_id = SharedString::from(format!(
        "nav-{}-{}",
        node.id.as_deref().unwrap_or(&node_idx.to_string()),
        node_idx
    ));

    let item = div()
        .id(btn_id.clone())
        .flex()
        .w_full()
        .px(px(12.))
        .py(px(8.))
        .rounded(px(6.))
        .bg(bg)
        .text_size(px(13.))
        .cursor_pointer()
        .hover(|style| {
            if !active {
                style.bg(ctx.muted.opacity(0.2))
            } else {
                style
            }
        })
        .child(
            div()
                .text_color(text_color)
                .font_weight(if active {
                    gpui::FontWeight::MEDIUM
                } else {
                    gpui::FontWeight::NORMAL
                })
                .child(label),
        );

    if action.is_empty() {
        return item.into_any_element();
    }

    item.on_click(move |_ev, _window, cx| {
        let action = action.clone();
        if !action.is_empty() {
            handler.handle(action, cx);
        }
    })
    .into_any_element()
}

fn render_button_row(
    node: &UiNode,
    _ctx: &RenderContext,
    handler: Arc<dyn ActionHandler>,
    node_idx: usize,
) -> gpui::AnyElement {
    // 兼容旧格式：props.buttons 数组
    let buttons = ui_schema::prop_array(&node.props, "buttons");

    let mut row = div()
        .flex()
        .flex_row()
        .gap(px(8.))
        .items_center()
        .flex_wrap();

    if let Some(btns) = buttons {
        for (i, btn) in btns.iter().enumerate() {
            let label = btn
                .get("label")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let action = btn
                .get("action")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let variant = btn.get("variant").and_then(|v| v.as_str()).unwrap_or("");

            let btn_id = SharedString::from(format!("btn-row-{}-{}", node_idx, i));

            let mut btn = Button::new(btn_id).label(label);

            // 直接按照 API 使用，不做任何"修复"
            if variant == "primary" || !action.is_empty() {
                btn = btn.primary();
            } else {
                btn = btn.ghost();
            }

            // 添加点击处理
            if !action.is_empty() {
                let handler = handler.clone();
                btn = btn.on_click(move |_event, _window, cx| {
                    handler.handle(action.clone(), cx);
                });
            }

            row = row.child(btn);
        }
    }

    // 同时渲染 children 中的 button 节点
    for (i, child) in node.children.iter().enumerate() {
        row = row.child(render_button(
            child,
            _ctx,
            handler.clone(),
            node_idx * 100 + i,
        ));
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
    div().w_full().h(px(1.)).bg(ctx.border).into_any_element()
}

fn render_input(
    node: &UiNode,
    state: &serde_json::Value,
    ctx: &RenderContext,
    handler: Arc<dyn ActionHandler>,
    node_idx: usize,
    input_states: Option<&HashMap<String, Entity<InputState>>>,
) -> gpui::AnyElement {
    let placeholder = ui_schema::prop_str_or(&node.props, "placeholder", "").to_string();
    let bind = node.bind.clone().unwrap_or_default();
    let value = if !bind.is_empty() {
        state_get_str(state, &bind)
    } else {
        String::new()
    };

    let input_id = SharedString::from(format!(
        "input-{}-{}",
        node.id.as_deref().unwrap_or(&node_idx.to_string()),
        node_idx
    ));

    // 处理 width 属性
    let width_str = ui_schema::prop_str_or(&node.props, "width", "100%");
    let width = if width_str.ends_with('%') {
        let pct = width_str
            .trim_end_matches('%')
            .parse::<f32>()
            .unwrap_or(100.0);
        pct / 100.0
    } else {
        1.0
    };

    // 优先使用 InputState（可编辑输入框）
    if let Some(states) = input_states {
        if let Some(input_state) = states.get(&bind) {
            let mut input_div = div()
                .id(input_id.clone())
                .flex()
                .items_center()
                .px(px(12.))
                .py(px(8.))
                .bg(ctx.muted.opacity(0.3))
                .rounded_lg()
                .border_1()
                .border_color(ctx.border);

            // 设置宽度
            if width < 1.0 {
                input_div = input_div.w(px(300.0_f32 * width));
            } else {
                input_div = input_div.flex_1();
            }

            return input_div
                .child(Input::new(input_state).appearance(false))
                .into_any_element();
        }
    }

    // Fallback: 只读显示
    let mut input_div = div()
        .id(input_id)
        .flex()
        .items_center()
        .px(px(12.))
        .py(px(8.))
        .bg(ctx.muted.opacity(0.3))
        .rounded_lg()
        .border_1()
        .border_color(ctx.border)
        .cursor_text();

    // 设置宽度
    if width < 1.0 {
        input_div = input_div.w(px(300.0_f32 * width));
    } else {
        input_div = input_div.flex_1();
    }

    input_div
        .child(if value.is_empty() {
            div()
                .text_size(px(14.))
                .text_color(ctx.muted_fg)
                .child(placeholder)
        } else {
            div().text_size(px(14.)).text_color(ctx.fg).child(value)
        })
        .into_any_element()
}

fn render_table(
    node: &UiNode,
    state: &serde_json::Value,
    ctx: &RenderContext,
    handler: Arc<dyn ActionHandler>,
    node_idx: usize,
) -> gpui::AnyElement {
    let bind = node.bind.clone().unwrap_or_default();
    let columns = ui_schema::prop_array(&node.props, "columns");
    let on_row_click = node.on_action.clone();

    let mut table = div()
        .flex()
        .flex_col()
        .border_1()
        .border_color(ctx.border)
        .rounded_lg()
        .overflow_hidden();

    // 解析列定义：支持 string（label==field）和 object {label, field}
    let col_specs: Vec<(String, String)> = columns
        .iter()
        .flat_map(|v| v.iter())
        .map(|col| {
            if let Some(s) = col.as_str() {
                (s.to_string(), s.to_string())
            } else {
                let label = col
                    .get("label")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let field = col
                    .get("field")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                (label, field)
            }
        })
        .collect();

    // 表头
    let mut header = div()
        .flex()
        .flex_row()
        .bg(ctx.muted.opacity(0.3))
        .border_b_1()
        .border_color(ctx.border);

    for (col_label, _col_field) in &col_specs {
        header = header.child(
            div()
                .flex_1()
                .px(px(12.))
                .py(px(8.))
                .text_size(px(12.))
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .text_color(ctx.muted_fg)
                .child(col_label.clone()),
        );
    }
    table = table.child(header);

    // 数据行
    if !bind.is_empty() {
        if let Some(rows) = state_get(state, &bind).and_then(|v| v.as_array()) {
            for (row_idx, row) in rows.iter().enumerate() {
                let mut row_div = div()
                    .flex()
                    .flex_row()
                    .border_b_1()
                    .border_color(ctx.border.opacity(0.5))
                    .cursor_pointer()
                    .hover(|style| style.bg(ctx.muted.opacity(0.1)));

                // 检查是否有 selected 字段
                let is_selected = row
                    .get("selected")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                for (col_label, col_field) in &col_specs {
                    // 空 field 表示操作列，跳过数据渲染
                    let cell_val = if col_field.is_empty() {
                        String::new()
                    } else if col_field == "selected" {
                        if is_selected {
                            "✓".to_string()
                        } else {
                            "✗".to_string()
                        }
                    } else {
                        row.get(col_field.as_str())
                            .map(|v| match v {
                                serde_json::Value::String(s) => s.clone(),
                                serde_json::Value::Number(n) => n.to_string(),
                                serde_json::Value::Bool(b) => b.to_string(),
                                _ => v.to_string(),
                            })
                            .unwrap_or_default()
                    };

                    // 状态列颜色高亮
                    let text_color = if col_field == "is_active" {
                        if cell_val == "true" {
                            gpui::rgb(0x22C55E).into() // green-500
                        } else {
                            gpui::rgb(0xEF4444).into() // red-500
                        }
                    } else {
                        ctx.fg
                    };

                    row_div = row_div.child(
                        div()
                            .flex_1()
                            .px(px(12.))
                            .py(px(6.))
                            .text_size(px(12.))
                            .text_color(text_color)
                            .child(cell_val),
                    );
                }

                // 选中行背景高亮
                if is_selected {
                    row_div = row_div.bg(ctx.primary.opacity(0.08));
                }

                // 行点击：如果有 on_action，发送 toggle_select
                let row_id = SharedString::from(format!("table-row-{}-{}", node_idx, row_idx));

                if let Some(ref action_prefix) = on_row_click {
                    let action_prefix = action_prefix.clone();
                    let click_handler = handler.clone();
                    table = table.child(row_div.id(row_id).on_click(move |_ev, _window, cx| {
                        let action = format!("{}:{}", action_prefix, row_idx);
                        click_handler.handle(action, cx);
                    }));
                } else {
                    table = table.child(row_div);
                }
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
    input_states: Option<&HashMap<String, Entity<InputState>>>,
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
        card = card.child(render_node(
            child,
            state,
            ctx,
            handler.clone(),
            node_idx * 100 + i,
            input_states,
        ));
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
    input_states: Option<&HashMap<String, Entity<InputState>>>,
) -> gpui::AnyElement {
    let direction = match node.component.as_str() {
        "flex_row" | "flex-row" => "row",
        _ => "col",
    };

    // 检测flex-row中的纯按钮容器，渲染为ButtonGroup以保持点击功能
    if direction == "row" && !node.children.is_empty() {
        let all_buttons = node.children.iter().all(|child| child.component == "button");
        if all_buttons {
            return render_button_group_from_flex_row(node, ctx, handler, node_idx);
        }
    }

    let gap = ui_schema::prop_i64(&node.props, "gap")
        .map(|g| px(g as f32))
        .unwrap_or(px(8.));
    let padding = ui_schema::prop_i64(&node.props, "padding").map(|p| px(p as f32));
    let margin_left = ui_schema::prop_str_or(&node.props, "margin_left", "");

    let mut container = div().flex().gap(gap);
    if direction == "col" {
        container = container.flex_col().flex_1();
    }

    // 添加内边距
    if let Some(padding) = padding {
        container = container.p(padding);
    }

    // 添加左边距（用于"auto"实现push效果）
    if margin_left == "auto" {
        container = container.flex_1();
    }

    for (i, child) in node.children.iter().enumerate() {
        container = container.child(render_node(
            child,
            state,
            ctx,
            handler.clone(),
            node_idx * 100 + i,
            input_states,
        ));
    }

    container.into_any_element()
}

/// 未识别组件 → 占位符
fn render_select(
    node: &UiNode,
    state: &serde_json::Value,
    ctx: &RenderContext,
    handler: Arc<dyn ActionHandler>,
    node_idx: usize,
) -> gpui::AnyElement {
    let bind = node.bind.as_deref().unwrap_or("");
    let current_value = ui_schema::state_get_str(state, bind);
    let placeholder = ui_schema::prop_str_or(&node.props, "placeholder", "请选择");
    let on_action = node
        .on_action
        .clone()
        .unwrap_or_else(|| format!("select_{}", bind.replace('.', "_")));

    let options: Vec<(String, String)> = ui_schema::prop_array(&node.props, "options")
        .map(|arr| {
            arr.iter()
                .filter_map(|opt| {
                    let label = opt
                        .get("label")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let value = opt
                        .get("value")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    if label.is_empty() && value.is_empty() {
                        None
                    } else {
                        Some((label, value))
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    // 找到当前选中项的 label
    let selected_label = options
        .iter()
        .find(|(_, v)| v == &current_value)
        .map(|(l, _)| l.clone())
        .unwrap_or_else(|| placeholder.to_string());

    // 选项按钮组（始终显示，类似 radio 按钮组）
    let options_container = div()
        .flex()
        .flex_row()
        .flex_wrap()
        .gap(px(4.))
        .mt(px(4.))
        .children(options.iter().enumerate().map(|(i, (label, value))| {
            let is_selected = value == &current_value;
            let btn_handler = handler.clone();
            let action_str = format!("{}:{}", on_action, value);
            let btn_id = SharedString::from(format!("select-opt-{}-{}", node_idx, i));
            let fg = ctx.fg;
            let primary = ctx.primary;
            let border = ctx.border;
            let muted = ctx.muted;
            let action_str = action_str.clone();
            div()
                .id(btn_id)
                .px(px(10.))
                .py(px(4.))
                .rounded(px(4.))
                .cursor_pointer()
                .text_sm()
                .bg(if is_selected {
                    primary.opacity(0.15)
                } else {
                    muted
                })
                .border_1()
                .border_color(if is_selected { primary } else { border })
                .text_color(if is_selected { primary } else { fg })
                .on_click(move |_ev, _window, cx| {
                    btn_handler.handle(action_str.clone(), cx);
                })
                .child(label.clone())
                .into_any_element()
        }));

    // 标题 + 当前选中值 + 选项列表
    let trigger_id = SharedString::from(format!("select-trigger-{}", node_idx));
    div()
        .flex()
        .flex_col()
        .w_full()
        .child(
            div()
                .id(trigger_id)
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .w_full()
                .h(px(28.))
                .px(px(8.))
                .rounded(px(4.))
                .border_1()
                .border_color(ctx.border)
                .bg(ctx.card)
                .cursor_pointer()
                .hover(|s| s.border_color(ctx.primary))
                .child(
                    div()
                        .text_sm()
                        .text_color(if current_value.is_empty() {
                            ctx.muted_fg
                        } else {
                            ctx.fg
                        })
                        .child(selected_label),
                )
                .child(div().text_xs().text_color(ctx.muted_fg).child("▾")),
        )
        .child(options_container)
        .into_any_element()
}

fn render_unsupported(component: &str, ctx: &RenderContext) -> gpui::AnyElement {
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
                .child(format!("Unsupported component: {}", component)),
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
    input_states: Option<&HashMap<String, Entity<InputState>>>,
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

    let mut container = div().flex().flex_1().gap(px(gap));
    if !is_row {
        container = container.flex_col();
    }

    if is_row {
        // 左右分栏：左侧固定宽度，右侧 flex-1
        container = container.child(
            div()
                .w(px(left_width))
                .min_w(px(150.))
                .h_full()
                .flex()
                .flex_col()
                .overflow_hidden()
                .child(render_node(
                    children[0],
                    state,
                    ctx,
                    handler.clone(),
                    node_idx * 100 + 0,
                    input_states,
                )),
        );
        container = container.child(
            div()
                .id("plugin-split-right")
                .flex_1()
                .h_full()
                .flex()
                .flex_col()
                .overflow_y_scroll()
                .child(render_node(
                    children[1],
                    state,
                    ctx,
                    handler.clone(),
                    node_idx * 100 + 1,
                    input_states,
                )),
        );
    } else {
        // 上下分栏：上方固定高度，下方 flex-1
        for (i, child) in children.iter().enumerate() {
            let is_last = i == children.len() - 1;
            if is_last {
                container =
                    container.child(div().flex_1().w_full().overflow_hidden().child(render_node(
                        child,
                        state,
                        ctx,
                        handler.clone(),
                        node_idx * 100 + i,
                        input_states,
                    )));
            } else {
                let h = if i == 0 { px(left_width) } else { px(200.) };
                container =
                    container.child(div().h(h).w_full().overflow_hidden().child(render_node(
                        child,
                        state,
                        ctx,
                        handler.clone(),
                        node_idx * 100 + i,
                        input_states,
                    )));
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
    _input_states: Option<&HashMap<String, Entity<InputState>>>,
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
    let is_dir = entry
        .get("is_dir")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let path = entry.get("path").and_then(|v| v.as_str()).unwrap_or("");
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
            div().w(indent).h(px(1.)), // indent spacer
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
// Accordion 折叠面板组件 — 基于 gpui_component::accordion::Accordion
// ---------------------------------------------------------------------------

fn render_accordion(
    node: &UiNode,
    state: &serde_json::Value,
    ctx: &RenderContext,
    handler: Arc<dyn ActionHandler>,
    node_idx: usize,
    input_states: Option<&HashMap<String, Entity<InputState>>>,
) -> gpui::AnyElement {
    let accordion_id = SharedString::from(format!("accordion-{}", node_idx));
    let multiple = ui_schema::prop_bool(&node.props, "multiple").unwrap_or(false);
    let bordered = ui_schema::prop_bool(&node.props, "bordered").unwrap_or(true);

    let mut acc = Accordion::new(accordion_id).bordered(bordered);
    if multiple {
        acc = acc.multiple(true);
    }

    // 将子节点渲染为 AccordionItem：每个子节点的 label prop 作为 trigger
    for (i, child) in node.children.iter().enumerate() {
        let title = ui_schema::prop_str_or(&child.props, "title", "");
        let trigger = if title.is_empty() {
            div().child(format!("Section {}", i + 1)).into_any_element()
        } else {
            div().child(title.to_string()).into_any_element()
        };

        let content = render_node(
            child,
            state,
            ctx,
            handler.clone(),
            node_idx * 100 + i,
            input_states,
        );

        acc = acc.item(|item| item.title(trigger).child(content));
    }

    acc.into_any_element()
}

// ---------------------------------------------------------------------------
// Collapsible 折叠组件 — 基于 gpui_component::collapsible::Collapsible
// ---------------------------------------------------------------------------

fn render_collapsible(
    node: &UiNode,
    state: &serde_json::Value,
    ctx: &RenderContext,
    handler: Arc<dyn ActionHandler>,
    node_idx: usize,
    input_states: Option<&HashMap<String, Entity<InputState>>>,
) -> gpui::AnyElement {
    let open = ui_schema::prop_bool(&node.props, "open").unwrap_or(false);

    let mut col = Collapsible::new().open(open);

    for (i, child) in node.children.iter().enumerate() {
        col = col.child(render_node(
            child,
            state,
            ctx,
            handler.clone(),
            node_idx * 100 + i,
            input_states,
        ));
    }

    col.into_any_element()
}

// ---------------------------------------------------------------------------
// GroupBox 分组容器 — 基于 gpui_component::group_box::GroupBox
// ---------------------------------------------------------------------------

fn render_group_box(
    node: &UiNode,
    state: &serde_json::Value,
    ctx: &RenderContext,
    handler: Arc<dyn ActionHandler>,
    node_idx: usize,
    input_states: Option<&HashMap<String, Entity<InputState>>>,
) -> gpui::AnyElement {
    let label = ui_schema::prop_str_or(&node.props, "label", "");

    let mut gb = GroupBox::new();
    if !label.is_empty() {
        gb = gb.title(SharedString::from(label.to_string()));
    }

    for (i, child) in node.children.iter().enumerate() {
        gb = gb.child(render_node(
            child,
            state,
            ctx,
            handler.clone(),
            node_idx * 100 + i,
            input_states,
        ));
    }

    gb.into_any_element()
}
