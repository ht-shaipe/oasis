#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use gpui::{App, AppContext as _};

fn main() {
    #[cfg(target_os = "windows")]
    std::env::set_var("GPUI_DISABLE_DIRECT_COMPOSITION", "true");

    let app = gpui::Application::new();

    // macOS: 点击 Dock 图标重新打开窗口
    app.on_reopen(|cx| {
        if cx.windows().is_empty() {
            oasis::open_new("oasis", |window, cx| {
                cx.new(|cx| oasis::SamplePanel::new(window, cx))
            }, cx);
        } else {
            cx.activate(true);
        }
    });

    app.run(move |cx: &mut App| {
        oasis::init(cx);

        // oasis::plugins::wasm_loader::init_wasm_manager(cx); // 暂时注释排查 panic

        cx.on_action(|_: &oasis::Quit, cx| {
            cx.quit();
        });

        oasis::open_new("oasis", |window, cx| {
            cx.new(|cx| oasis::SamplePanel::new(window, cx))
        }, cx);
    });
}
