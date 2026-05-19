//! 记事本挂件 — 独立 cdylib 插件
//!
//! 编译为 `libwidget_notepad.dylib`，宿主运行时动态加载。

use gpui::{
    div, px, AppContext as _, Context, IntoElement, ParentElement, Render,
    SharedString, Styled, Window,
};
use gpui_component::ActiveTheme as _;
use gpui_component::scroll::ScrollableElement as _;
use plugin_sdk::{Widget, WidgetManifest};

// ---------------------------------------------------------------------------
// NotepadWidget
// ---------------------------------------------------------------------------

pub struct NotepadWidget {
    content: SharedString,
}

impl Widget for NotepadWidget {
    fn widget_id() -> &'static str {
        "notepad"
    }

    fn manifest() -> WidgetManifest {
        WidgetManifest {
            id: "notepad".into(),
            display_name: "记事本".into(),
            description: "一个简易文本编辑器插件".into(),
            icon_emoji: "📝".into(),
            icon_svg: include_str!("../icon.svg").into(),
            window_width: 400.0,
            window_height: 350.0,
        }
    }

    fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            content: SharedString::from(
                "欢迎使用记事本！\n\n这是一个简易文本编辑器插件。\n你可以在未来的版本中编辑文本内容。",
            ),
        }
    }
}

impl Render for NotepadWidget {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let is_dark = theme.mode.is_dark();

        let char_count = self.content.chars().count();
        let line_count = self.content.lines().count();

        let text_area_bg = if is_dark {
            theme.colors.muted.opacity(0.2)
        } else {
            theme.colors.muted.opacity(0.1)
        };

        let status_bar_bg = if is_dark {
            theme.colors.muted.opacity(0.3)
        } else {
            theme.colors.muted.opacity(0.15)
        };

        div()
            .flex()
            .flex_col()
            .h_full()
            .child(
                div()
                    .flex_1()
                    .p(px(12.))
                    .overflow_y_scrollbar()
                    .bg(text_area_bg)
                    .child(
                        div()
                            .text_size(px(13.))
                            .text_color(theme.colors.foreground.opacity(0.85))
                            .line_height(gpui::relative(1.6))
                            .child(self.content.clone()),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .px(px(12.))
                    .py(px(4.))
                    .bg(status_bar_bg)
                    .border_t_1()
                    .border_color(theme.colors.border.opacity(0.1))
                    .child(
                        div()
                            .text_size(px(11.))
                            .text_color(theme.colors.muted_foreground)
                            .child(format!("字符数: {}", char_count)),
                    )
                    .child(
                        div()
                            .text_size(px(11.))
                            .text_color(theme.colors.muted_foreground)
                            .child(format!("行数: {}", line_count)),
                    ),
            )
    }
}

// ---------------------------------------------------------------------------
// FFI 导出
// ---------------------------------------------------------------------------

/// 导出清单 JSON — 宿主通过 libloading 读取此符号获取清单
#[unsafe(no_mangle)]
pub extern "C" fn widget_manifest_json() -> *const std::ffi::c_char {
    static MANIFEST_JSON: &str = r#"{"id":"notepad","display_name":"记事本","description":"一个简易文本编辑器插件","icon_emoji":"📝","icon_svg":"","window_width":400.0,"window_height":350.0}"#;
    MANIFEST_JSON.as_ptr() as *const std::ffi::c_char
}

/// 导出视图工厂函数
#[unsafe(no_mangle)]
pub extern "C" fn widget_factory(app: *mut gpui::App) -> *mut std::ffi::c_void {
    unsafe {
        let cx = &mut *app;
        // 直接创建实体并装箱为 AnyView，然后作为裸指针返回
        let view: gpui::AnyView = cx.new(|cx| NotepadWidget::new(cx)).into();
        // 将 AnyView 装箱并返回裸指针
        let boxed: Box<dyn std::any::Any> = Box::new(view);
        Box::into_raw(boxed) as *mut std::ffi::c_void
    }
}
