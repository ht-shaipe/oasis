//! WASM 插件内容视图
//!
//! 提供可以在 PluginRegistry 中使用的 WASM 插件视图

use gpui::{div, px, AnyView, App, AppContext as _, Context, InteractiveElement as _, IntoElement, ParentElement, Render, StatefulInteractiveElement as _, Styled as _, Window};
use gpui_component::ActiveTheme as _;

/// WASM 计数器插件视图
pub struct WasmCounterView {
    count: i32,
    max: i32,
}

impl WasmCounterView {
    pub fn new(_window: &mut Window, _cx: &mut Context<Self>) -> Self {
        Self { count: 0, max: 100 }
    }

    fn percentage(&self) -> i32 {
        if self.max == 0 {
            0
        } else {
            (self.count * 100 / self.max).max(0).min(100)
        }
    }

    fn increment(&mut self) {
        self.count = (self.count + 1).min(self.max);
    }

    fn decrement(&mut self) {
        self.count = (self.count - 1).max(0);
    }

    fn reset(&mut self) {
        self.count = 0;
    }
}

impl Render for WasmCounterView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let count = self.count;
        let max = self.max;
        let percentage = self.percentage();
        let entity = cx.entity().downgrade();
        let entity2 = entity.clone();
        let entity3 = entity.clone();

        div()
            .flex()
            .flex_col()
            .gap(px(16.))
            .p(px(24.))
            .size_full()
            .bg(theme.colors.background)
            .child(
                // 标题栏
                div()
                    .flex()
                    .items_center()
                    .gap(px(12.))
                    .child(
                        div()
                            .text_size(px(24.))
                            .child("🔢"),
                    )
                    .child(
                        div()
                            .text_size(px(18.))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(theme.colors.foreground)
                            .child("计数器"),
                    ),
            )
            .child(
                // 计数器显示
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(20.))
                    .p(px(32.))
                    .bg(theme.colors.muted.opacity(0.3))
                    .rounded_lg()
                    .child(
                        div()
                            .text_size(px(64.))
                            .font_weight(gpui::FontWeight::BOLD)
                            .text_color(theme.colors.foreground)
                            .child(format!("{}", count)),
                    )
                    .child(
                        div()
                            .text_size(px(14.))
                            .text_color(theme.colors.muted_foreground)
                            .child(format!("{} / {}", count, max)),
                    )
                    .child(
                        div()
                            .w(px(240.))
                            .h(px(12.))
                            .bg(theme.colors.muted)
                            .rounded_full()
                            .child(
                                div()
                                    .h(px(12.))
                                    .bg(theme.colors.primary)
                                    .rounded_full()
                                    .flex_shrink_0()
                                    .w(px((240.0 * percentage as f32 / 100.0) as _)),
                            ),
                    ),
            )
            .child(
                // 控制按钮
                div()
                    .flex()
                    .flex_row()
                    .gap(px(16.))
                    .child(
                        div()
                            .id("wasm-counter-decrement")
                            .flex()
                            .items_center()
                            .justify_center()
                            .size(px(64.))
                            .bg(theme.colors.muted)
                            .rounded_lg()
                            .cursor_pointer()
                            .text_size(px(28.))
                            .text_color(theme.colors.foreground)
                            .child("➖")
                            .on_click(move |_event, _window, cx| {
                                if let Some(e) = entity.upgrade() {
                                    e.update(cx, |view, cx| {
                                        view.decrement();
                                        cx.notify();
                                    });
                                }
                            }),
                    )
                    .child(
                        div()
                            .id("wasm-counter-reset")
                            .flex()
                            .items_center()
                            .justify_center()
                            .size(px(64.))
                            .bg(theme.colors.muted)
                            .rounded_lg()
                            .cursor_pointer()
                            .text_size(px(24.))
                            .text_color(theme.colors.foreground)
                            .child("🔄")
                            .on_click(move |_event, _window, cx| {
                                if let Some(e) = entity2.upgrade() {
                                    e.update(cx, |view, cx| {
                                        view.reset();
                                        cx.notify();
                                    });
                                }
                            }),
                    )
                    .child(
                        div()
                            .id("wasm-counter-increment")
                            .flex()
                            .items_center()
                            .justify_center()
                            .size(px(64.))
                            .bg(theme.colors.primary)
                            .rounded_lg()
                            .cursor_pointer()
                            .text_size(px(28.))
                            .text_color(gpui::rgb(0xffffff))
                            .child("➕")
                            .on_click(move |_event, _window, cx| {
                                if let Some(e) = entity3.upgrade() {
                                    e.update(cx, |view, cx| {
                                        view.increment();
                                        cx.notify();
                                    });
                                }
                            }),
                    ),
            )
            .child(
                // 插件信息
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
                            .child("ℹ️ 插件信息"),
                    )
                    .children(
                        [
                            format!("ID: {}", "counter"),
                            format!("类型: {}", "WASM"),
                            format!("描述: {}", "一个简单的计数器插件"),
                        ]
                        .iter()
                        .map(|text| {
                            div()
                                .text_size(px(11.))
                                .text_color(gpui::rgb(0x888888))
                                .child(text.clone())
                        }),
                    ),
            )
    }
}

/// 创建计数器视图（供 PluginRegistry 使用）
pub fn create_counter_view(window: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|cx| WasmCounterView::new(window, cx)).into()
}
