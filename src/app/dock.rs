use gpui::{
    div, px, App, Context, IntoElement, ParentElement, Render, SharedString, Styled as _, Window,
};
use gpui_component::ActiveTheme as _;
use rust_i18n::t;

/// 底部浮动 Dock
pub struct FloatingDock {
    label: SharedString,
}

impl FloatingDock {
    pub fn new(_window: &mut Window, _cx: &mut App) -> Self {
        Self {
            label: t!("app.title").into(),
        }
    }
}

impl Render for FloatingDock {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let is_dark = theme.mode.is_dark();

        // Dock 背景色：半透明，深色模式下更暗，浅色模式下偏灰
        let bg_color = if is_dark {
            theme.colors.background.opacity(0.35)
        } else {
            theme.colors.background.opacity(0.45)
        };

        // 图标占位颜色
        let icon_bg = theme.colors.muted.opacity(0.5);
        let icon_fg = theme.colors.foreground;

        // 右上角标签背景
        let label_bg = if is_dark {
            theme.colors.background.opacity(0.4)
        } else {
            theme.colors.background.opacity(0.5)
        };

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
                // 右上角标签
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .mb(px(6.))
                    .px(px(12.))
                    .py(px(4.))
                    .rounded_2xl()
                    .bg(label_bg)
                    .shadow_sm()
                    .child(
                        div()
                            .text_size(px(12.))
                            .text_color(theme.colors.foreground.opacity(0.8))
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .child(self.label.to_string()),
                    ),
            )
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
                    // 图标列表
                    .children(dock_icons(&icon_bg, &icon_fg)),
            )
    }
}

/// 生成 Dock 图标占位符
fn dock_icons(icon_bg: &gpui::Hsla, icon_fg: &gpui::Hsla) -> Vec<impl IntoElement> {
    let icon_count = 8;
    let mut icons = Vec::with_capacity(icon_count);

    for i in 0..icon_count {
        let icon_label = match i {
            0 => "A",
            1 => "B",
            2 => "C",
            3 => "D",
            4 => "E",
            5 => "F",
            6 => "G",
            7 => "H",
            _ => "?",
        };

        icons.push(
            div()
                .flex()
                .flex_col()
                .items_center()
                .gap(px(4.))
                .child(
                    // 图标圆形背景
                    div()
                        .flex()
                        .items_center()
                        .justify_center()
                        .size(px(44.))
                        .rounded_lg()
                        .bg(*icon_bg)
                        .child(
                            div()
                                .text_size(px(16.))
                                .text_color(*icon_fg)
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .child(icon_label),
                        ),
                )
                .child(
                    // 底部小圆点指示器
                    div()
                        .size(px(4.))
                        .rounded_full()
                        .bg(icon_fg.opacity(0.5)),
                ),
        );
    }

    icons
}
