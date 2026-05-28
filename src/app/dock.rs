use gpui::{
    div, img, px, App, ClickEvent, Context, InteractiveElement as _, IntoElement, ObjectFit,
    ParentElement, Render, SharedString, StatefulInteractiveElement as _, Styled, StyledImage,
    Window,
};
use gpui::prelude::FluentBuilder;
use gpui_component::ActiveTheme as _;

use crate::plugins::PluginRegistry;
use crate::app::app_launcher::AppLauncherState;
use rust_i18n::t;

/// 底部浮动 Dock — 从 PluginRegistry 动态读取插件列表
pub struct FloatingDock {}

impl FloatingDock {
    pub fn new(_window: &mut Window, _cx: &mut App) -> Self {
        Self {}
    }
}

impl Render for FloatingDock {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // 先获取所有需要的颜色值，避免借用冲突
        let (bg_color, icon_bg, icon_fg, border_color) = {
            let theme = cx.theme();
            let bg = theme.colors.background.opacity(0.5);
            let icon_bg = theme.colors.muted.opacity(0.5);
            let icon_fg = theme.colors.foreground;
            let border_color = theme.colors.border.opacity(0.15);
            (bg, icon_bg, icon_fg, border_color)
        };

        // 先获取悬停状态（需要可变借用）
        let hover_plugin_id = cx.global_mut::<DockHoverState>().hovered_plugin_id.clone();
        // 然后获取插件列表（不可变借用）
        let registry = cx.global::<PluginRegistry>();

        div()
            .absolute()
            .bottom(px(20.))
            .left_0()
            .right_0()
            .flex()
            .flex_col()
            .items_center()
            .justify_end()
            .child(
                // Dock 主体
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_center()
                    .gap(px(12.))
                    .px(px(16.))
                    .py(px(10.))
                    .rounded_2xl()
                    .bg(bg_color)
                    .shadow_lg()
                    .border_1()
                    .border_color(border_color)
                    .on_mouse_move(move |_event, _window, _cx| {
                        // 鼠标在 Dock 区域移动，悬停状态由各个图标处理
                    })
                    .children(
                        // "所有应用" 入口图标（固定在第一位）
                        {
                            let (icon_bg_all, icon_fg_all) = (icon_bg, icon_fg);
                            let is_hovered_all = hover_plugin_id.as_ref() == Some(&"__all_apps__".to_string());

                            Some(
                                div()
                                    .id(SharedString::from("dock-icon-all-apps"))
                                    .flex()
                                    .flex_col()
                                    .items_center()
                                    .cursor_pointer()
                                    .on_click(move |_ev: &ClickEvent, _window, cx| {
                                        cx.global_mut::<AppLauncherState>().visible = !cx.global::<AppLauncherState>().visible;
                                        cx.refresh_windows();
                                    })
                                    .on_mouse_move(move |_event, _window, cx| {
                                        cx.global_mut::<DockHoverState>().hovered_plugin_id = Some("__all_apps__".to_string());
                                    })
                                    .child(
                                        // 图标 slot：固定尺寸，悬停时内容溢出但不撑大 dock
                                        div()
                                            .relative()
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .w(px(44.))
                                            .h(px(48.))
                                            .child(
                                                // 放大图标用 absolute 定位向上偏移，不影响 dock 布局
                                                div()
                                                    .when(is_hovered_all, |el| el.absolute().bottom(px(8.)))
                                                    .flex()
                                                    .items_center()
                                                    .justify_center()
                                                    .size(if is_hovered_all { px(56.) } else { px(44.) })
                                                    .rounded_lg()
                                                    .bg(icon_bg_all)
                                                    .when(is_hovered_all, |el| el.shadow_xl())
                                                    .child(
                                                        div()
                                                            .text_size(if is_hovered_all { px(52.) } else { px(40.) })
                                                            .text_color(icon_fg_all)
                                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                                            .child(t!("launcher.icon").to_string()),
                                                    ),
                                            ),
                                    ),
                            )
                        },
                    )
                    .children(
                        registry.plugins.iter().map(|plugin| {
                            let plugin_id = plugin.manifest.id.clone();
                            let display_name = plugin.manifest.display_name.clone();

                            // SVG 图标路径（如有则优先渲染 SVG），否则回退 emoji/首字母
                            let icon_svg_path = plugin.icon_svg_path.clone();
                            // 使用 emoji 如果可用，否则使用首字母
                            let display_icon = if let Some(emoji) = plugin.icon_emoji.as_ref() {
                                emoji.clone()
                            } else {
                                display_name.chars().next().unwrap_or('?').to_string()
                            };

                            let is_open = registry.open_windows.contains_key(&plugin_id);
                            let is_hovered = hover_plugin_id.as_ref() == Some(&plugin_id);
                            let dot_color = if is_open {
                                icon_fg
                            } else {
                                icon_fg.opacity(0.3)
                            };

                            let icon_bg_copy = icon_bg;
                            let icon_fg_copy = icon_fg;

                            div()
                                .id(SharedString::from(format!("dock-icon-{}", plugin_id)))
                                .flex()
                                .flex_col()
                                .items_center()
                                .cursor_pointer()
                                .on_click({
                                    let plugin_id = plugin_id.clone();
                                    move |_ev: &ClickEvent, window, cx| {
                                        PluginRegistry::open_plugin(&plugin_id, window, cx);
                                    }
                                })
                                .on_mouse_move({
                                    let plugin_id = plugin_id.clone();
                                    move |_event, _window, cx| {
                                        cx.global_mut::<DockHoverState>().hovered_plugin_id = Some(plugin_id.clone());
                                    }
                                })
                                .child(
                                    // 图标 slot：固定尺寸，悬停时内容溢出但不撑大 dock
                                    div()
                                        .relative()
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .w(px(44.))
                                        .h(px(48.))
                                        .child(
                                            // 放大图标用 absolute 定位向上偏移，不影响 dock 布局
                                            div()
                                                .when(is_hovered, |el| el.absolute().bottom(px(8.)))
                                                .flex()
                                                .items_center()
                                                .justify_center()
                                                .size(if is_hovered { px(56.) } else { px(44.) })
                                                .rounded_lg()
                                                .bg(icon_bg_copy)
                                                .when(is_hovered, |el| el.shadow_xl())
                                                .child(
                                                    if let Some(ref svg_path) = icon_svg_path {
                                                        img(std::path::PathBuf::from(svg_path))
                                                            .object_fit(ObjectFit::Contain)
                                                            .size(if is_hovered { px(52.) } else { px(40.) })
                                                            .into_any_element()
                                                    } else {
                                                        div()
                                                            .text_size(if is_hovered { px(52.) } else { px(40.) })
                                                            .text_color(icon_fg_copy)
                                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                                            .child(display_icon)
                                                            .into_any_element()
                                                    },
                                                ),
                                        ),
                                )
                                .when(is_open, |dot| {
                                    // 底部小圆点指示器 - 只在窗口打开时显示
                                    dot.child(div().size(px(4.)).rounded_full().bg(dot_color))
                                })
                        }),
                    ),
            )
    }
}

/// Dock 悬停状态
#[derive(Debug, Clone, Default)]
pub struct DockHoverState {
    pub hovered_plugin_id: Option<String>,
}

impl gpui::Global for DockHoverState {}
