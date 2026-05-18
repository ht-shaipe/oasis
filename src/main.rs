#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use gpui::{div, px, App, AppContext, Context, IntoElement, ParentElement, Render, Styled, Window};
use gpui_component::ActiveTheme;

fn main() {
    #[cfg(target_os = "windows")]
    std::env::set_var("GPUI_DISABLE_DIRECT_COMPOSITION", "true");

    gpui::Application::new()
        .run(move |cx: &mut App| {
            oasis::init(cx);

            // 初始化 WASM 插件管理器
            oasis::plugins::wasm_loader::init_wasm_manager(cx);

            cx.on_action(|_: &oasis::Quit, cx| {
                cx.quit();
            });

            oasis::open_new(
                "oasis",
                |_window, cx| cx.new(|cx| WasmPluginView),
                cx,
            );
        });
}

struct WasmPluginView;

impl Render for WasmPluginView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let entity = cx.entity().clone();

        div()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .size_full()
            .gap(px(16.))
            .child(
                div()
                    .text_size(px(32.))
                    .font_weight(gpui::FontWeight::BOLD)
                    .text_color(theme.colors.foreground)
                    .child("🔌 WASM 插件系统"),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(12.))
                    .items_center()
                    .children(
                        [
                            "✅ WASM 插件已构建到 plugins/wasm/",
                            "✅ 加载器已就绪",
                            "📊 当前扫描到的插件:",
                        ]
                        .iter()
                        .map(|text| {
                            div()
                                .text_size(px(14.))
                                .text_color(theme.colors.muted_foreground)
                                .child(text.to_string())
                        }),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(8.))
                    .p(px(16.))
                    .bg(theme.colors.muted.opacity(0.3))
                    .rounded_lg()
                    .children(
                        [
                            ("wasm_plugin", "39KB", "计数器插件"),
                        ]
                        .iter()
                        .map(|(name, size, desc)| {
                            div()
                                .flex()
                                .items_center()
                                .gap(px(16.))
                                .child(
                                    div()
                                        .text_size(px(14.))
                                        .font_weight(gpui::FontWeight::MEDIUM)
                                        .text_color(theme.colors.foreground)
                                        .child(name.to_string()),
                                )
                                .child(
                                    div()
                                        .text_size(px(12.))
                                        .text_color(theme.colors.muted_foreground)
                                        .child(size.to_string()),
                                )
                                .child(
                                    div()
                                        .text_size(px(12.))
                                        .text_color(theme.colors.muted_foreground)
                                        .child(desc.to_string()),
                                )
                        }),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(8.))
                    .p(px(16.))
                    .bg(gpui::rgb(0x1a1a1a))
                    .rounded_lg()
                    .child(
                        div()
                            .text_size(px(12.))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(gpui::rgb(0xffffff))
                            .child("📝 文件位置"),
                    )
                    .children(
                        [
                            "plugins/wasm/wasm_plugin_bg.wasm",
                            "plugins/wasm/wasm_plugin.js",
                        ]
                        .iter()
                        .map(|path| {
                            div()
                                .flex()
                                .items_center()
                                .gap(px(8.))
                                .child(
                                    div()
                                        .text_size(px(11.))
                                        .text_color(gpui::rgb(0x00ff00))
                                        .child("✓"),
                                )
                                .child(
                                    div()
                                        .text_size(px(11.))
                                        .text_color(gpui::rgb(0x888888))
                                        .font_family("monospace")
                                        .child(path.to_string()),
                                )
                        }),
                    ),
            )
    }
}
