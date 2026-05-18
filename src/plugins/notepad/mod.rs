use gpui::{
    div, px, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render,
    SharedString, Styled as _, Window,
};
use gpui_component::ActiveTheme as _;
use gpui_component::scroll::ScrollableElement as _;

use crate::plugins::{Plugin, PluginEntry};

// ---------------------------------------------------------------------------
// NotepadView
// ---------------------------------------------------------------------------

/// 简易记事本视图
pub struct NotepadView {
    /// 文本内容
    content: SharedString,
}

impl Plugin for NotepadView {
    fn plugin_id() -> &'static str {
        "notepad"
    }

    fn new(_window: &mut Window, _cx: &mut Context<Self>) -> Self {
        Self {
            content: SharedString::from(
                "欢迎使用记事本！\n\n这是一个简易文本编辑器插件。\n你可以在未来的版本中编辑文本内容。",
            ),
        }
    }
}

impl Render for NotepadView {
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
            // 内容区域
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
            // 底部状态栏
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
// inventory 提交
// ---------------------------------------------------------------------------

/// 创建 NotepadView 并转为 AnyView
fn create_notepad_view(window: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|cx| NotepadView::new(window, cx)).into()
}

inventory::submit! {
    PluginEntry {
        id: "notepad",
        manifest_toml: include_str!("manifest.toml"),
        icon_svg: include_str!("icon.svg"),
        create_view: create_notepad_view,
    }
}
