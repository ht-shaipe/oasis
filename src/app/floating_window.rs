use gpui::{
    div, px, App, Context, IntoElement, InteractiveElement as _, MouseButton, MouseDownEvent,
    MouseMoveEvent, MouseUpEvent, ParentElement, Pixels, Point, Render, Styled as _, Window,
};
use gpui_component::ActiveTheme as _;

/// 可拖动的小窗口
pub struct FloatingWindow {
    /// 窗口左上角位置（相对于父容器）
    position: Point<Pixels>,
    /// 是否正在拖拽
    dragging: bool,
    /// 拖拽开始时鼠标位置
    drag_start: Point<Pixels>,
    /// 拖拽开始时窗口位置
    drag_origin: Point<Pixels>,
}

impl FloatingWindow {
    pub fn new(_window: &mut Window, _cx: &mut App) -> Self {
        Self {
            position: Point::new(px(80.0), px(60.0)),
            dragging: false,
            drag_start: Point::default(),
            drag_origin: Point::default(),
        }
    }
}

impl Render for FloatingWindow {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let is_dark = theme.mode.is_dark();

        let entity = cx.entity().downgrade();
        let entity2 = entity.clone();
        let entity3 = entity.clone();

        // 窗口背景
        let bg_color = if is_dark {
            theme.colors.background.opacity(0.85)
        } else {
            theme.colors.background.opacity(0.92)
        };

        // 标题栏背景
        let titlebar_bg = if is_dark {
            theme.colors.muted.opacity(0.3)
        } else {
            theme.colors.muted.opacity(0.15)
        };

        div()
            .absolute()
            .left(self.position.x)
            .top(self.position.y)
            .w(px(280.))
            .flex()
            .flex_col()
            .rounded_lg()
            .bg(bg_color)
            .shadow_lg()
            .border_1()
            .border_color(theme.colors.border.opacity(0.2))
            .overflow_hidden()
            // 标题栏 —— 可拖拽区域
            .child(
                div()
                    .id("floating-window-titlebar")
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .px(px(12.))
                    .py(px(8.))
                    .bg(titlebar_bg)
                    .cursor_grab()
                    // 拖拽：鼠标按下
                    .on_mouse_down(
                        MouseButton::Left,
                        move |event: &MouseDownEvent, _window, cx| {
                            if let Some(e) = entity.upgrade() {
                                e.update(cx, |this, _cx| {
                                    this.dragging = true;
                                    this.drag_start = event.position;
                                    this.drag_origin = this.position;
                                });
                            }
                        },
                    )
                    // 拖拽：鼠标移动
                    .on_mouse_move(move |event: &MouseMoveEvent, _window, cx| {
                        if let Some(e) = entity2.upgrade() {
                            e.update(cx, |this, _cx| {
                                if this.dragging {
                                    let dx = event.position.x - this.drag_start.x;
                                    let dy = event.position.y - this.drag_start.y;
                                    this.position = Point::new(
                                        this.drag_origin.x + dx,
                                        this.drag_origin.y + dy,
                                    );
                                }
                            });
                        }
                    })
                    // 拖拽：鼠标抬起
                    .on_mouse_up(
                        MouseButton::Left,
                        move |_event: &MouseUpEvent, _window, cx| {
                            if let Some(e) = entity3.upgrade() {
                                e.update(cx, |this, _cx| {
                                    this.dragging = false;
                                });
                            }
                        },
                    )
                    // 标题文字
                    .child(
                        div()
                            .text_size(px(13.))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(theme.colors.foreground.opacity(0.9))
                            .child("小窗口"),
                    )
                    // macOS 风格窗口按钮
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(6.))
                            .child(
                                div()
                                    .size(px(12.))
                                    .rounded_full()
                                    .bg(gpui::red().opacity(0.8)),
                            )
                            .child(
                                div()
                                    .size(px(12.))
                                    .rounded_full()
                                    .bg(gpui::yellow().opacity(0.8)),
                            )
                            .child(
                                div()
                                    .size(px(12.))
                                    .rounded_full()
                                    .bg(gpui::green().opacity(0.8)),
                            ),
                    ),
            )
            // 窗口内容区
            .child(
                div()
                    .px(px(12.))
                    .py(px(12.))
                    .flex()
                    .flex_col()
                    .gap(px(8.))
                    .child(
                        div()
                            .text_size(px(12.))
                            .text_color(theme.colors.muted_foreground)
                            .child("这是一个可拖动的小窗口，拖动标题栏即可移动位置。"),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap(px(8.))
                            .child(
                                div()
                                    .flex_1()
                                    .rounded_md()
                                    .bg(theme.colors.muted.opacity(0.3))
                                    .p(px(8.))
                                    .child(
                                        div()
                                            .text_size(px(11.))
                                            .text_color(theme.colors.foreground.opacity(0.7))
                                            .child("左"),
                                    ),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .rounded_md()
                                    .bg(theme.colors.muted.opacity(0.3))
                                    .p(px(8.))
                                    .child(
                                        div()
                                            .text_size(px(11.))
                                            .text_color(theme.colors.foreground.opacity(0.7))
                                            .child("右"),
                                    ),
                            ),
                    ),
            )
    }
}
