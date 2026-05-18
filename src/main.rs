#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use gpui::{div, App, AppContext as _, Context, IntoElement, Render, Styled as _, Window};

fn main() {
    #[cfg(target_os = "windows")]
    std::env::set_var("GPUI_DISABLE_DIRECT_COMPOSITION", "true");

    gpui::Application::new().run(move |cx: &mut App| {
        oasis::init(cx);

        // 初始化 WASM 插件管理器
        oasis::plugins::wasm_loader::init_wasm_manager(cx);

        cx.on_action(|_: &oasis::Quit, cx| {
            cx.quit();
        });

        oasis::open_new("oasis", |_window, cx| cx.new(|_| WasmPluginView), cx);
    });
}

struct WasmPluginView;

impl Render for WasmPluginView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {

        div()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .size_full()
    }
}
