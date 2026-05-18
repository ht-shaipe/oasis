#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use gpui::{div, App, AppContext, Context, IntoElement, ParentElement, Render, Styled, Window};
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
            oasis::open_new(
                "oasis",
                None,
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
