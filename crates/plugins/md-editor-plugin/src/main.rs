//! md-editor-plugin 独立进程入口
//!
//! 作为子进程运行，拥有独立的 gpui 实例和窗口。
//! 宿主通过 IPC 控制生命周期。

use gpui::{App, AppContext as _, Application, Bounds, SharedString, Size, WindowBounds, WindowOptions, px, size};
use gpui_component::TitleBar;
use md_editor_plugin::AsterView;

fn main() {
    // 读取命令行参数：宿主传入 IPC socket 路径
    let args: Vec<String> = std::env::args().collect();
    let _socket_path = args.get(1).cloned();

    let app = Application::new();

    app.run(move |cx| {
        // 初始化 gpui 组件
        gpui_component::init(cx);

        // 计算窗口尺寸
        let mut window_size = size(px(900.0), px(700.0));
        if let Some(display) = cx.primary_display() {
            let display_size = display.bounds().size;
            window_size.width = window_size.width.min(display_size.width * 0.85);
            window_size.height = window_size.height.min(display_size.height * 0.85);
        }
        let window_bounds = Bounds::centered(None, window_size, cx);

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(window_bounds)),
                titlebar: Some(TitleBar::title_bar_options()),
                window_min_size: Some(Size {
                    width: px(600.0),
                    height: px(400.0),
                }),
                ..Default::default()
            },
            |window, cx| {
                cx.new(|cx| AsterView::new(window, cx))
            },
        )
        .expect("failed to open md-editor-plugin window");

        eprintln!("✅ md-editor-plugin started as subprocess");

        // TODO: 连接 IPC socket，接收宿主命令
        // if let Some(path) = socket_path {
        //     // 异步连接 Unix socket
        // }
    });
}
