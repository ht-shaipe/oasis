#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use gpui::{div, App, AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Window};
use gpui_component::ActiveTheme;

fn main() {
    #[cfg(target_os = "windows")]
    std::env::set_var("GPUI_DISABLE_DIRECT_COMPOSITION", "true");

    gpui::Application::new()
        .run(move |cx: &mut App| {
            oasis::init(cx);

            cx.on_action(|_: &oasis::Quit, cx| {
                cx.quit();
            });

            // 传入背景图片路径（本地绝对路径）；若不需要背景图则传 None
            // 示例：Some("/Users/yourname/Pictures/bg.jpg")
            oasis::open_new(
                "oasis",
                None, // 暂不设置背景图，如需启用请改为 Some("/absolute/path/to/bg.jpg")
                |_window, cx| cx.new(|_| PlaceholderView),
                cx,
            );
        });
}

struct PlaceholderView;

impl Render for PlaceholderView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        div()
            .flex()
            .items_center()
            .justify_center()
            .bg(theme.colors.background)
            .size_full()
            .child("Oasis")
    }
}
