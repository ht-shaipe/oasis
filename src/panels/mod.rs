use gpui::*;
use gpui_component::dock::{Panel, PanelControl, PanelEvent};
use gpui_component::ActiveTheme as _;
use rust_i18n::t;

/// Simple sample panel
pub struct SamplePanel {
    focus_handle: FocusHandle,
}

impl SamplePanel {
    pub fn new(_window: &mut Window, cx: &mut App) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
        }
    }
}

impl Panel for SamplePanel {
    fn panel_name(&self) -> &'static str {
        "SamplePanel"
    }

    fn title(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().h(px(0.)).into_any_element()
    }

    fn zoomable(&self, _cx: &App) -> Option<PanelControl> {
        None
    }
}

impl EventEmitter<PanelEvent> for SamplePanel {}

impl Focusable for SamplePanel {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for SamplePanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        div()
            .id("sample-panel")
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .items_center()
            .justify_center()
            // 不设置背景色，保持透明，使窗体背景图透过
            .child(
                div()
                    .text_color(theme.colors.foreground)
                    .text_size(px(18.))
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .child(t!("app.title").to_string())
            )
            .child(
                div()
                    .mt(px(8.))
                    .text_color(theme.colors.muted_foreground)
                    .text_size(px(13.))
                    .child(t!("welcome.message").to_string())
            )
    }
}