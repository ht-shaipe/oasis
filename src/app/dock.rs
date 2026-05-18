use gpui::{
    div, px, App, ClickEvent, Context, InteractiveElement as _, IntoElement, ParentElement, Render,
    SharedString, StatefulInteractiveElement as _, Styled as _, Window,
};
use gpui_component::ActiveTheme as _;

use crate::plugins::PluginRegistry;

/// 底部浮动 Dock — 从 PluginRegistry 动态读取插件列表
pub struct FloatingDock {}

impl FloatingDock {
    pub fn new(_window: &mut Window, _cx: &mut App) -> Self {
        Self {}
    }
}

impl Render for FloatingDock {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let is_dark = theme.mode.is_dark();

        // Dock 背景色：半透明 (70% 不透明度，30% 透明度)
        let bg_color = if is_dark {
            theme.colors.background.opacity(0.5)
        } else {
            theme.colors.background.opacity(0.5)
        };

        // 图标占位颜色
        let icon_bg = theme.colors.muted.opacity(0.5);
        let icon_fg = theme.colors.foreground;

        // 从 PluginRegistry 读取插件列表
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
                    .border_color(theme.colors.border.opacity(0.15))
                    .children(
                        registry.plugins.iter().map(|plugin| {
                            let plugin_id = plugin.manifest.id.clone();
                            let display_name = plugin.manifest.display_name.clone();

                            // 使用 emoji 如果可用，否则使用首字母
                            let display_icon = if let Some(emoji) = plugin.icon_emoji.as_ref() {
                                emoji.clone()
                            } else {
                                display_name.chars().next().unwrap_or('?').to_string()
                            };

                            let is_open = registry.open_windows.contains_key(&plugin_id);
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
                                .gap(px(4.))
                                .cursor_pointer()
                                .on_click(move |_ev: &ClickEvent, window, cx| {
                                    PluginRegistry::open_plugin(&plugin_id, window, cx);
                                })
                                .child(
                                    // 图标圆形背景
                                    div()
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .size(px(44.))
                                        .rounded_lg()
                                        .bg(icon_bg_copy)
                                        .child(
                                            div()
                                                .text_size(px(16.))
                                                .text_color(icon_fg_copy)
                                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                                .child(display_icon),
                                        ),
                                )
                                .child(
                                    // 底部小圆点指示器
                                    div().size(px(4.)).rounded_full().bg(dot_color),
                                )
                        }),
                    ),
            )
    }
}
