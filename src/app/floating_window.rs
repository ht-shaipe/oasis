use gpui::{
    div, px, App, ClickEvent, Context, InteractiveElement as _, MouseButton, MouseDownEvent,
    MouseMoveEvent, ParentElement, Pixels, Point, Render,
    StatefulInteractiveElement as _, Styled as _, Window,
};
use gpui_component::ActiveTheme as _;

/// 可拖动 + 可调整大小 + 可最大化的浮动窗口
pub struct FloatingWindow {
    position: Point<Pixels>,
    size: Point<Pixels>,
    dragging: bool,
    drag_start: Point<Pixels>,
    drag_origin: Point<Pixels>,
    resizing: bool,
    resize_start: Point<Pixels>,
    resize_origin_size: Point<Pixels>,
    maximized: bool,
    restore_position: Point<Pixels>,
    restore_size: Point<Pixels>,
    visible: bool,
}

impl FloatingWindow {
    pub fn new(_window: &mut Window, _cx: &mut App) -> Self {
        let initial_size = Point::new(px(320.), px(240.));
        let initial_pos = Point::new(px(80.0), px(60.0));
        Self {
            position: initial_pos,
            size: initial_size,
            dragging: false,
            drag_start: Point::default(),
            drag_origin: Point::default(),
            resizing: false,
            resize_start: Point::default(),
            resize_origin_size: Point::default(),
            maximized: false,
            restore_position: initial_pos,
            restore_size: initial_size,
            visible: true,
        }
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    fn start_drag(&mut self, pos: Point<Pixels>) {
        self.dragging = true;
        self.drag_start = pos;
        self.drag_origin = self.position;
    }

    fn start_resize(&mut self, pos: Point<Pixels>) {
        self.resizing = true;
        self.resize_start = pos;
        self.resize_origin_size = self.size;
    }

    fn handle_move(&mut self, pos: Point<Pixels>) {
        if self.dragging {
            self.position = Point::new(
                self.drag_origin.x + (pos.x - self.drag_start.x),
                self.drag_origin.y + (pos.y - self.drag_start.y),
            );
        } else if self.resizing {
            let dx = pos.x - self.resize_start.x;
            let dy = pos.y - self.resize_start.y;
            self.size = Point::new(
                (self.resize_origin_size.x + dx).max(px(160.0)),
                (self.resize_origin_size.y + dy).max(px(100.0)),
            );
        }
    }

    fn end_interaction(&mut self) {
        self.dragging = false;
        self.resizing = false;
    }

    fn toggle_maximize(&mut self) {
        if self.maximized {
            self.position = self.restore_position;
            self.size = self.restore_size;
            self.maximized = false;
        } else {
            self.restore_position = self.position;
            self.restore_size = self.size;
            self.position = Point::new(px(0.0), px(0.0));
            self.size = Point::new(px(1200.0), px(800.0));
            self.maximized = true;
        }
    }

    fn minimize(&mut self) {
        self.visible = false;
    }

    fn close(&mut self) {
        self.visible = false;
    }
}

impl Render for FloatingWindow {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let theme = cx.theme();
        let is_dark = theme.mode.is_dark();

        let entity = cx.entity().downgrade();

        let bg_color = if is_dark {
            theme.colors.background.opacity(0.92)
        } else {
            theme.colors.background.opacity(0.97)
        };
        let title_bar_bg = if is_dark {
            theme.colors.muted.opacity(0.4)
        } else {
            theme.colors.muted.opacity(0.25)
        };
        let border_color = theme.colors.border.opacity(0.2);
        let content_bg = if is_dark {
            theme.colors.background.opacity(0.85)
        } else {
            theme.colors.background.opacity(0.95)
        };

        let w = self.size.x;
        let h = self.size.y;
        let x = self.position.x;
        let y = self.position.y;

        let e_drag = entity.clone();
        let e_move = entity.clone();
        let e_up = entity.clone();
        let e_resize = entity.clone();
        let e_max = entity.clone();
        let e_close = entity.clone();
        let e_restore = entity.clone();

        // Traffic-light colors (RGBA 8-char hex, no .opacity() on rgba)
        let close_bg = gpui::rgba(0xff5f57ff);
        let min_bg = gpui::rgba(0xffbd2eff);
        let max_bg = gpui::rgba(0x28c840ff);

        let hint_text = if self.maximized {
            "已最大化 · 点击绿色按钮还原"
        } else if !self.visible {
            "窗口已最小化"
        } else {
            "拖动标题栏移动 · 右下角拖动调整大小 · 红=关闭 黄=最小化 绿=最大化"
        };


        div()
            .id("floating-window")
            .absolute()
            .left(x)
            .top(y)
            .w(w)
            .h(h)
            .flex()
            .flex_col()
            .rounded_lg()
            .bg(bg_color)
            .shadow_lg()
            .overflow_hidden()
            // .cursor_pointer()
            // ── 全局鼠标移动/抬起 ────────────────────────────────────────
            .on_mouse_move(move |event: &MouseMoveEvent, _window, cx| {
                if let Some(e) = e_move.upgrade() {
                    e.update(cx, |this, _cx| {
                        this.handle_move(event.position);
                    });
                }
            })
            .on_mouse_up(MouseButton::Left, move |_, _window, cx| {
                if let Some(e) = e_up.upgrade() {
                    e.update(cx, |this, _cx| {
                        this.end_interaction();
                    });
                }
            })
            // ── 标题栏 ──────────────────────────────────────────────────
            .child(
                div()
                    .id("floating-title-bar")
                    .flex()
                    .flex_row()
                    .items_center()
                    .px(px(12.))
                    .py(px(8.))
                    .bg(title_bar_bg)
                    .cursor_grab()
                    .on_mouse_down(MouseButton::Left, move |event: &MouseDownEvent, _window, cx| {
                        if let Some(e) = e_drag.upgrade() {
                            e.update(cx, |this, _cx| {
                                this.start_drag(event.position);
                            });
                        }
                    })
            // 关闭按钮（独立点击区域）
                    .child(
                        div()
                            .id("floating-close-btn")
                            .size(px(12.))
                            .rounded_full()
                            .bg(close_bg)
                            // .cursor_pointer()
                            .on_click(move |_ev: &ClickEvent, _window, cx| {
                                if let Some(e) = e_close.upgrade() {
                                    e.update(cx, |this, _cx| {
                                        this.close();
                                    });
                                }
                            }),
                    )
                    // 最小化按钮（独立点击区域）
                    .child(
                        div()
                            .id("floating-min-btn")
                            .size(px(12.))
                            .rounded_full()
                            .mx(px(4.))
                            .bg(min_bg)
                            // .cursor_pointer()
                            .on_click(move |_ev: &ClickEvent, _window, cx| {
                                if let Some(e) = e_restore.upgrade() {
                                    e.update(cx, |this, _cx| {
                                        this.minimize();
                                    });
                                }
                            }),
                    )
                    // 最大化按钮（独立点击区域）
                    .child(
                        div()
                            .id("floating-max-btn")
                            .size(px(12.))
                            .rounded_full()
                            .bg(max_bg)
                            // .cursor_pointer()
                            .on_click(move |_ev: &ClickEvent, _window, cx| {
                                if let Some(e) = e_max.upgrade() {
                                    e.update(cx, |this, _cx| {
                                        this.toggle_maximize();
                                    });
                                }
                            })
                            .child(
                                div()
                                    .absolute()
                                    .right(px(0.))
                                    .top(px(0.))
                                    .w(px(5.))
                                    .h(px(5.))
                                    .border_1()
                                    .border_color(theme.colors.foreground.opacity(0.55))
                                    .bg(title_bar_bg)
                                    .opacity(if self.maximized { 1.0 } else { 0.0 }),
                            ),
                    )
                    // 标题文字
                    .child(
                        div()
                            .flex_1()
                            .px(px(6.))
                            .child(
                                div()
                                    .text_size(px(13.))
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(theme.colors.foreground.opacity(0.8))
                                    .child("浮动窗口"),
                            ),
                    ),
            )
            // ── 内容区 ─────────────────────────────────────────────────
            .child(
                div()
                    .id("floating-content")
                    .flex()
                    .flex_col()
                    .flex_1()
                    .px(px(14.))
                    .py(px(12.))
                    .gap(px(10.))
                    .bg(content_bg)
                    .children([
                        div()
                            .text_size(px(11.5))
                            .text_color(theme.colors.muted_foreground)
                            .child(hint_text),
                        // 功能卡片行
                        div()
                            .flex_1()
                            .flex()
                            .flex_row()
                            .gap(px(8.))
                            .child(
                                div()
                                    .flex_1()
                                    .rounded_md()
                                    .bg(theme.colors.muted.opacity(0.2))
                                    .border_1()
                                    .border_color(theme.colors.border.opacity(0.15))
                                    .p(px(10.))
                                    .flex()
                                    .flex_col()
                                    .items_center()
                                    .justify_center()
                                    .gap(px(4.))
                                    .child(
                                        div()
                                            .text_size(px(11.))
                                            .font_weight(gpui::FontWeight::MEDIUM)
                                            .text_color(theme.colors.foreground.opacity(0.7))
                                            .child("面板 A"),
                                    )
                                    .child(
                                        div()
                                            .text_size(px(10.))
                                            .text_color(theme.colors.muted_foreground)
                                            .child("可放入任意内容"),
                                    ),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .rounded_md()
                                    .bg(theme.colors.muted.opacity(0.2))
                                    .border_1()
                                    .border_color(theme.colors.border.opacity(0.15))
                                    .p(px(10.))
                                    .flex()
                                    .flex_col()
                                    .items_center()
                                    .justify_center()
                                    .gap(px(4.))
                                    .child(
                                        div()
                                            .text_size(px(11.))
                                            .font_weight(gpui::FontWeight::MEDIUM)
                                            .text_color(theme.colors.foreground.opacity(0.7))
                                            .child("面板 B"),
                                    )
                                    .child(
                                        div()
                                            .text_size(px(10.))
                                            .text_color(theme.colors.muted_foreground)
                                            .child("可放入任意内容"),
                                    ),
                            ),
                    ]),
            )
            // ── 右下角 resize handle ───────────────────────────────────
            .child(
                div()
                    .id("floating-resize-handle")
                    .absolute()
                    .right(px(0.))
                    .bottom(px(0.))
                    .w(px(22.))
                    .h(px(22.))
                    .cursor_nwse_resize()
                    .on_mouse_down(MouseButton::Left, move |event: &MouseDownEvent, _window, cx| {
                        if let Some(e) = e_resize.upgrade() {
                            e.update(cx, |this, _cx| {
                                this.start_resize(event.position);
                            });
                        }
                    })
                    // resize 图标：三条横线
                    .child(
                        div()
                            .absolute()
                            .right(px(4.))
                            .bottom(px(4.))
                            .flex()
                            .flex_col()
                            .gap(px(2.))
                            .child(
                                div()
                                    .h(px(1.5))
                                    .w(px(10.))
                                    .rounded_full()
                                    .bg(theme.colors.muted_foreground.opacity(0.35)),
                            )
                            .child(
                                div()
                                    .h(px(1.5))
                                    .w(px(7.))
                                    .rounded_full()
                                    .bg(theme.colors.muted_foreground.opacity(0.35)),
                            )
                            .child(
                                div()
                                    .h(px(1.5))
                                    .w(px(4.))
                                    .rounded_full()
                                    .bg(theme.colors.muted_foreground.opacity(0.35)),
                            ),
                    ),
            )
    }
}
