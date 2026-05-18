//! DSL 计数器插件示例
//!
//! 展示如何使用声明式 UI DSL 创建插件

use gpui::{
    div, px, AnyView, App, AppContext as _, CursorStyle,
    IntoElement, ParentElement, Render, Styled as _, Window,
};
use gpui_component::ActiveTheme as _;

use crate::plugins::ui_dsl::{Action, Align, ButtonStyle, PluginState, TextStyle, Widget, Widget::*};

/// DSL 计数器插件
pub struct DslCounterPlugin {
    count: i32,
    max: i32,
}

impl DslCounterPlugin {
    pub fn new(_window: &mut Window, _cx: &mut gpui::Context<Self>) -> Self {
        Self { count: 0, max: 100 }
    }

    /// 执行动作
    pub fn execute_action(&mut self, action: &Action) {
        match action {
            Action::Increment => self.increment(),
            Action::Decrement => self.decrement(),
            Action::Reset => self.reset(),
            Action::Custom(name) => {
                tracing::info!("执行自定义动作: {}", name);
            }
            _ => {}
        }
    }

    fn increment(&mut self) {
        self.count = (self.count + 1).min(self.max);
        tracing::info!("计数 +1 = {}", self.count);
    }

    fn decrement(&mut self) {
        self.count = (self.count - 1).max(0);
        tracing::info!("计数 -1 = {}", self.count);
    }

    fn reset(&mut self) {
        self.count = 0;
        tracing::info!("重置计数 = 0");
    }

    /// 获取当前状态
    fn get_state(&self) -> PluginState {
        PluginState {
            data: serde_json::json!({
                "count": self.count,
                "max": self.max,
                "percentage": if self.max > 0 {
                    (self.count * 100 / self.max).max(0).min(100)
                } else {
                    0
                }
            }),
            actions: vec!["increment".to_string(), "decrement".to_string(), "reset".to_string()],
        }
    }
}

impl Render for DslCounterPlugin {
    fn render(&mut self, _window: &mut Window, cx: &mut gpui::Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let state = self.get_state();

        // 渲染主容器
        div()
            .flex()
            .flex_col()
            .size_full()
            .p(px(24.))
            .gap(px(16.))
            .bg(theme.colors.background)
            // 插件标题
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(12.))
                    .child(div().text_size(px(24.)).child("🎨"))
                    .child(
                        div()
                            .text_size(px(18.))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(theme.colors.foreground)
                            .child("DSL 计数器（声明式 UI）"),
                    ),
            )
            // 计数显示
            .child(
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
                            .child(format!("{}", self.count)),
                    )
                    .child(
                        div()
                            .text_size(px(14.))
                            .text_color(theme.colors.muted_foreground)
                            .child(format!("{}/{}", self.count, self.max)),
                    )
            )
            // 按钮行（暂时没有点击事件）
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap(px(12.))
                    .items_center()
                    .child(
                        div()
                            .w(px(56.0))
                            .h(px(56.0))
                            .bg(gpui::hsla(0.0, 0.7, 0.56, 1.0)) // #f44336
                            .text_size(px(28.))
                            .text_color(gpui::white())
                            .rounded_lg()
                            .flex()
                            .items_center()
                            .justify_center()
                            .cursor(CursorStyle::PointingHand)
                            .child("-")
                    )
                    .child(
                        div()
                            .w(px(56.0))
                            .h(px(56.0))
                            .bg(gpui::hsla(0.08, 0.9, 0.5, 1.0)) // #ff9800
                            .text_size(px(28.))
                            .text_color(gpui::white())
                            .rounded_lg()
                            .flex()
                            .items_center()
                            .justify_center()
                            .cursor(CursorStyle::PointingHand)
                            .child("⟲")
                    )
                    .child(
                        div()
                            .w(px(56.0))
                            .h(px(56.0))
                            .bg(gpui::hsla(0.28, 0.7, 0.45, 1.0)) // #4CAF50
                            .text_size(px(28.))
                            .text_color(gpui::white())
                            .rounded_lg()
                            .flex()
                            .items_center()
                            .justify_center()
                            .cursor(CursorStyle::PointingHand)
                            .child("+")
                    ),
            )
            // 状态信息
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(8.))
                    .p(px(16.))
                    .bg(theme.colors.muted.opacity(0.3))
                    .rounded_lg()
                    .child(
                        div()
                            .text_size(px(12.))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(theme.colors.foreground)
                            .child("🔧 可用动作（宿主可调用）"),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap(px(8.))
                            .children(state.actions.iter().map(|action| {
                                div()
                                    .px(px(8.))
                                    .py(px(4.))
                                    .bg(theme.colors.primary.opacity(0.2))
                                    .rounded_md()
                                    .text_size(px(10.))
                                    .text_color(theme.colors.primary)
                                    .child(action.clone())
                            })),
                    ),
            )
    }
}

/// 创建 DSL 计数器视图（供 PluginRegistry 使用）
pub fn create_dsl_counter_view(window: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|cx| DslCounterPlugin::new(window, cx)).into()
}
