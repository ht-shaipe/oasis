use gpui::{
    div, img, px, App, ClickEvent, InteractiveElement as _, IntoElement, KeyDownEvent,
    ObjectFit, ParentElement, SharedString, StatefulInteractiveElement as _, Styled, StyledImage,
};
use gpui_component::ActiveTheme as _;
use rust_i18n::t;

use crate::plugins::PluginRegistry;

/// 全局状态 — 控制 AppLauncher 可见性
#[derive(Debug, Clone, Default)]
pub struct AppLauncherState {
    pub visible: bool,
}

impl gpui::Global for AppLauncherState {}

/// 渲染应用启动器覆盖层（纯函数，在 DockRoot::render 中调用）
pub fn render_launcher(cx: &mut App) -> impl IntoElement {
    let (overlay_bg, panel_bg, panel_border, text_fg, text_muted, icon_bg) = {
        let theme = cx.theme();
        (
            theme.colors.background.opacity(0.75),
            theme.colors.secondary,
            theme.colors.border.opacity(0.1),
            theme.colors.foreground,
            theme.colors.muted_foreground,
            theme.colors.muted.opacity(0.4),
        )
    };

    let registry = cx.global::<PluginRegistry>();
    let plugins: Vec<_> = registry.plugins.iter().collect();

    // 每行 5 个图标，每个图标区域宽度 = (680 - 48 padding) / 5 ≈ 126
    let item_width = px(120.);

    div()
        .id("app-launcher-overlay")
        .absolute()
        .inset_0()
        .flex()
        .items_center()
        .justify_center()
        // 点击遮罩空白区域关闭
        .on_click(move |_ev: &ClickEvent, _window, cx| {
            cx.global_mut::<AppLauncherState>().visible = false;
            cx.refresh_windows();
        })
        // ESC 关闭
        .on_key_down(move |ev: &KeyDownEvent, _window, cx| {
            if &ev.keystroke.key == "escape" {
                cx.global_mut::<AppLauncherState>().visible = false;
                cx.refresh_windows();
            }
        })
        .child(
            // 半透明遮罩
            div()
                .absolute()
                .inset_0()
                .bg(overlay_bg),
        )
        .child(
            // 内容区域（阻止点击冒泡到遮罩）
            div()
                .id("app-launcher-content")
                .relative()
                .w(px(680.))
                .max_h(px(520.))
                .flex()
                .flex_col()
                .rounded_2xl()
                .bg(panel_bg)
                .border_1()
                .border_color(panel_border)
                .shadow_xl()
                .overflow_hidden()
                // 阻止内容区域点击关闭
                .on_click(|_ev: &ClickEvent, _window, _cx| {
                    // 拦截冒泡
                })
                .child(
                    // 标题栏
                    div()
                        .flex()
                        .items_center()
                        .justify_center()
                        .px(px(24.))
                        .py(px(16.))
                        .border_b_1()
                        .border_color(panel_border)
                        .child(
                            div()
                                .text_size(px(18.))
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(text_fg)
                                .child(t!("launcher.all_apps").to_string()),
                        ),
                )
                .child(
                    // 应用网格（flex wrap 布局）
                    div()
                        .id("app-launcher-grid")
                        .flex_1()
                        .overflow_y_scroll()
                        .p(px(24.))
                        .flex()
                        .flex_wrap()
                        .gap(px(20.))
                        .children(
                            plugins.iter().map(|plugin| {
                                let plugin_id = plugin.manifest.id.clone();
                                let display_name = plugin.manifest.display_name.clone();
                                let icon_svg_path = plugin.icon_svg_path.clone();
                                let display_icon = if let Some(emoji) = plugin.icon_emoji.as_ref() {
                                    emoji.clone()
                                } else {
                                    display_name.chars().next().unwrap_or('?').to_string()
                                };

                                let icon_bg_c = icon_bg;
                                let text_muted_c = text_muted;

                                div()
                                    .id(SharedString::from(format!("launcher-{}", plugin_id)))
                                    .w(item_width)
                                    .flex()
                                    .flex_col()
                                    .items_center()
                                    .gap(px(8.))
                                    .py(px(8.))
                                    .cursor_pointer()
                                    .on_click({
                                        let plugin_id = plugin_id.clone();
                                        move |_ev: &ClickEvent, window, cx| {
                                            // 关闭启动器
                                            cx.global_mut::<AppLauncherState>().visible = false;
                                            cx.refresh_windows();
                                            // 打开插件
                                            PluginRegistry::open_plugin(&plugin_id, window, cx);
                                        }
                                    })
                                    .child(
                                        // 图标
                                        div()
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .size(px(56.))
                                            .rounded_xl()
                                            .bg(icon_bg_c)
                                            .child(
                                            if let Some(ref svg_path) = icon_svg_path {
                                                img(std::path::PathBuf::from(svg_path))
                                                    .object_fit(ObjectFit::Contain)
                                                    .size(px(48.))
                                                    .into_any_element()
                                            } else {
                                                div()
                                                    .text_size(px(32.))
                                                    .child(display_icon)
                                                    .into_any_element()
                                            },
                                        ),
                                    )
                                    .child(
                                        // 名称
                                        div()
                                            .text_size(px(12.))
                                            .text_color(text_muted_c)
                                            .text_center()
                                            .max_w(px(100.))
                                            .overflow_hidden()
                                            .child(display_name),
                                    )
                            }),
                        ),
                ),
        )
}
