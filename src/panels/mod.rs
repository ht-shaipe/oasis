use gpui::*;
use gpui_component::dock::{Panel, PanelControl, PanelEvent};
use gpui_component::scroll::ScrollableElement as _;
use gpui_component::ActiveTheme as _;
use rust_i18n::t;

/// 主面板 — 展示工具箱入口
pub struct SamplePanel {
    focus_handle: FocusHandle,
}

impl SamplePanel {
    pub fn new(_window: &mut Window, cx: &mut App) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
        }
    }
}

impl Panel for SamplePanel {
    fn panel_name(&self) -> &'static str {
        "SamplePanel"
    }

    fn title(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().h(px(0.)).into_any_element()
    }

    fn zoomable(&self, _cx: &App) -> Option<PanelControl> {
        None
    }
}

impl EventEmitter<PanelEvent> for SamplePanel {}

impl Focusable for SamplePanel {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for SamplePanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let is_dark = theme.mode.is_dark();

        // 收集所有已注册插件
        let registry = cx.global::<crate::plugins::PluginRegistry>();
        let plugins: Vec<_> = registry
            .plugins
            .iter()
            .map(|p| {
                (
                    p.manifest.id.clone(),
                    p.manifest.display_name.clone(),
                    p.manifest.description.clone(),
                    p.icon_emoji.clone().unwrap_or_default(),
                    p.is_wasm,
                )
            })
            .collect();

        let card_bg = if is_dark {
            theme.colors.secondary.opacity(0.8)
        } else {
            theme.colors.secondary.opacity(0.9)
        };

        let card_hover_bg = if is_dark {
            theme.colors.muted.opacity(0.3)
        } else {
            theme.colors.muted.opacity(0.15)
        };

        let title_color = theme.colors.foreground;
        let desc_color = theme.colors.muted_foreground;
        let badge_bg = if is_dark {
            gpui::hsla(220.0 / 360.0, 0.6, 0.55, 0.3)
        } else {
            gpui::hsla(220.0 / 360.0, 0.6, 0.55, 0.15)
        };

        div()
            .id("sample-panel")
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .overflow_y_scrollbar()
            .p(px(24.))
            // 标题区
            .child(
                div()
                    .flex()
                    .flex_col()
                    .mb(px(24.))
                    .child(
                        div()
                            .text_color(title_color)
                            .text_size(px(22.))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .child(t!("app.title").to_string()),
                    )
                    .child(
                        div()
                            .mt(px(6.))
                            .text_color(desc_color)
                            .text_size(px(13.))
                            .child(t!("welcome.message").to_string()),
                    ),
            )
            // 插件网格
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .gap(px(12.))
                    .children(plugins.into_iter().map(|(id, name, desc, emoji, is_wasm)| {
                        let card_bg = card_bg;
                        let card_hover_bg = card_hover_bg;
                        let desc_color = desc_color;
                        let title_color = title_color;
                        let badge_bg = badge_bg;

                        let badge_text = if is_wasm { "WASM" } else { "Native" };

                        div()
                            .id(SharedString::from(format!("plugin-card-{}", id)))
                            .flex()
                            .flex_col()
                            .w(px(180.))
                            .p(px(14.))
                            .rounded_md()
                            .bg(card_bg)
                            .border_1()
                            .border_color(theme.colors.border.opacity(0.08))
                            .cursor_pointer()
                            .hover(|s| s.bg(card_hover_bg))
                            .on_click(move |_ev, _window, cx| {
                                crate::plugins::PluginRegistry::open_plugin(&id, _window, cx);
                            })
                            // 图标 + 名称行
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap(px(8.))
                                    .child(
                                        div()
                                            .text_size(px(20.))
                                            .child(emoji.clone()),
                                    )
                                    .child(
                                        div()
                                            .text_size(px(13.))
                                            .font_weight(gpui::FontWeight::MEDIUM)
                                            .text_color(title_color)
                                            .child(name.clone()),
                                    ),
                            )
                            // 描述
                            .child(
                                div()
                                    .mt(px(6.))
                                    .text_size(px(11.))
                                    .text_color(desc_color)
                                    .child(desc.clone()),
                            )
                            // 类型标签
                            .child(
                                div()
                                    .mt(px(8.))
                                    .flex()
                                    .flex_row()
                                    .child(
                                        div()
                                            .px(px(6.))
                                            .py(px(2.))
                                            .rounded(px(3.))
                                            .bg(badge_bg)
                                            .text_size(px(9.))
                                            .text_color(theme.colors.foreground.opacity(0.7))
                                            .child(badge_text.to_string()),
                                    ),
                            )
                    })),
            )
    }
}
