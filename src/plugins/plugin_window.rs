use gpui::{
    div, px, AnyView, ClickEvent, Context, InteractiveElement as _, IntoElement, MouseButton,
    MouseDownEvent, MouseMoveEvent, MouseUpEvent, ParentElement, Pixels, Point, Render,
    SharedString, StatefulInteractiveElement as _, Styled as _, Window,
};
use gpui_component::ActiveTheme as _;

use super::PluginRegistry;

/// 插件浮动窗口
pub struct PluginWindow {
    /// 所属插件 ID
    pub plugin_id: String,
    /// 窗口标题
    pub title: String,
    /// 窗口位置
    pub position: Point<Pixels>,
    /// 窗口尺寸 (width, height)
    pub size: (Pixels, Pixels),
    /// 是否正在拖拽
    pub dragging: bool,
    /// 拖拽开始时鼠标位置
    pub drag_start: Point<Pixels>,
    /// 拖拽开始时窗口位置
    pub drag_origin: Point<Pixels>,
    /// 是否正在调整大小
    pub resizing: bool,
    /// 调整大小开始时鼠标位置
    pub resize_start: Point<Pixels>,
    /// 调整大小开始时窗口尺寸
    pub resize_origin_size: (Pixels, Pixels),
    /// 是否最小化（隐藏）
    pub minimized: bool,
    /// 是否最大化
    pub maximized: bool,
    /// 最大化前的位置（用于还原）
    pub restore_position: Point<Pixels>,
    /// 最大化前的尺寸（用于还原）
    pub restore_size: (Pixels, Pixels),
    /// 插件内容视图
    pub content: AnyView,
}

impl PluginWindow {
    /// 构造 PluginWindow（不依赖 Window/cx，由外层 `cx.new(|_| ...)` 包装为 Entity）
    pub fn new(plugin_id: &str, title: String, window_size: (f32, f32), content: AnyView) -> Self {
        Self {
            plugin_id: plugin_id.to_string(),
            title,
            position: Point::new(px(120.0), px(80.0)),
            size: (px(window_size.0), px(window_size.1)),
            dragging: false,
            drag_start: Point::default(),
            drag_origin: Point::default(),
            resizing: false,
            resize_start: Point::default(),
            resize_origin_size: (px(window_size.0), px(window_size.1)),
            minimized: false,
            maximized: false,
            restore_position: Point::new(px(120.0), px(80.0)),
            restore_size: (px(window_size.0), px(window_size.1)),
            content,
        }
    }

    /// 最小化窗口（隐藏）
    pub fn minimize(&mut self) {
        self.minimized = true;
    }

    /// 还原窗口（从最小化）
    pub fn restore(&mut self) {
        self.minimized = false;
    }

    /// 切换最大化状态
    pub fn toggle_maximize(&mut self) {
        if self.maximized {
            // 还原
            self.position = self.restore_position;
            self.size = self.restore_size;
            self.maximized = false;
        } else {
            // 最大化
            self.restore_position = self.position;
            self.restore_size = self.size;
            // TODO: 实际窗口尺寸，暂时硬编码
            self.position = Point::new(px(0.0), px(0.0));
            self.size = (px(1200.0), px(800.0));
            self.maximized = true;
        }
    }

    /// 开始调整大小
    pub fn start_resize(&mut self, pos: Point<Pixels>) {
        self.resizing = true;
        self.resize_start = pos;
        self.resize_origin_size = self.size;
    }

    /// 处理拖拽或调整大小
    pub fn handle_interaction(&mut self, pos: Point<Pixels>) {
        if self.dragging {
            let dx = pos.x - self.drag_start.x;
            let dy = pos.y - self.drag_start.y;
            self.position = Point::new(self.drag_origin.x + dx, self.drag_origin.y + dy);
        } else if self.resizing {
            let dx = pos.x - self.resize_start.x;
            let dy = pos.y - self.resize_start.y;
            let min_w = px(200.0);
            let min_h = px(150.0);
            self.size = (
                (self.resize_origin_size.0 + dx).max(min_w),
                (self.resize_origin_size.1 + dy).max(min_h),
            );
        }
    }

    /// 结束拖拽或调整大小
    pub fn end_interaction(&mut self) {
        self.dragging = false;
        self.resizing = false;
    }
}

impl Render for PluginWindow {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let is_dark = theme.mode.is_dark();

        let entity = cx.entity().downgrade();
        let entity2 = entity.clone();
        let entity3 = entity.clone();
        let entity_resize = entity.clone();
        let entity_min = entity.clone();
        let entity_max = entity.clone();

        // 关闭按钮需要 plugin_id
        let plugin_id_for_close = self.plugin_id.clone();

        // 窗口背景
        let bg_color = if is_dark {
            theme.colors.background.opacity(0.85)
        } else {
            theme.colors.background.opacity(0.92)
        };

        // 标题栏背景
        let title_bar_bg = if is_dark {
            theme.colors.muted.opacity(0.3)
        } else {
            theme.colors.muted.opacity(0.15)
        };

        div()
            .absolute()
            .left(self.position.x)
            .top(self.position.y)
            .w(self.size.0)
            .h(self.size.1)
            .flex()
            .flex_col()
            .rounded_lg()
            .bg(bg_color)
            .shadow_lg()
            .overflow_hidden()
			.cursor_default()
            // 标题栏 —— 可拖拽区域
            .child(
                div()
                    .id(SharedString::from(format!(
                        "plugin-window-title_bar-{}",
                        self.plugin_id
                    )))
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .px(px(12.))
                    .py(px(8.))
                    .bg(title_bar_bg)
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
                    // 拖拽/调整大小：鼠标移动
                    .on_mouse_move(move |event: &MouseMoveEvent, _window, cx| {
                        if let Some(e) = entity2.upgrade() {
                            e.update(cx, |this, _cx| {
                                this.handle_interaction(event.position);
                            });
                        }
                    })
                    // 拖拽/调整大小：鼠标抬起
                    .on_mouse_up(
                        MouseButton::Left,
                        move |_event: &MouseUpEvent, _window, cx| {
                            if let Some(e) = entity3.upgrade() {
                                e.update(cx, |this, _cx| {
                                    this.end_interaction();
                                });
                            }
                        },
                    )
                    // macOS 风格窗口按钮（红色 = 关闭）
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(6.))
                            // 红色关闭按钮
                            .child(
                                div()
                                    .id(SharedString::from(format!(
                                        "plugin-close-btn-{}",
                                        self.plugin_id
                                    )))
                                    .size(px(12.))
                                    .rounded_full()
                                    .bg(gpui::red().opacity(0.8))
                                    .cursor_pointer()
                                    .on_click(move |_ev: &ClickEvent, _window, cx| {
                                        PluginRegistry::close_plugin(&plugin_id_for_close, cx);
                                    }),
                            )
                            // 黄色最小化按钮
                            .child(
                                div()
                                    .id(SharedString::from(format!(
                                        "plugin-min-btn-{}",
                                        self.plugin_id
                                    )))
                                    .size(px(12.))
                                    .rounded_full()
                                    .bg(gpui::yellow().opacity(0.8))
                                    .cursor_pointer()
                                    .on_click(move |_ev: &ClickEvent, _window, cx| {
                                        if let Some(e) = entity_min.upgrade() {
                                            e.update(cx, |this, _cx| {
                                                this.minimize();
                                            });
                                        }
                                    }),
                            )
                            // 绿色最大化按钮
                            .child(
                                div()
                                    .id(SharedString::from(format!(
                                        "plugin-max-btn-{}",
                                        self.plugin_id
                                    )))
                                    .size(px(12.))
                                    .rounded_full()
                                    .bg(gpui::green().opacity(0.8))
                                    .cursor_pointer()
                                    .on_click(move |_ev: &ClickEvent, _window, cx| {
                                        if let Some(e) = entity_max.upgrade() {
                                            e.update(cx, |this, _cx| {
                                                this.toggle_maximize();
                                            });
                                        }
                                    }),
                            ),
                    )
                    // 标题文字
                    .child(
                        div()
                            .text_size(px(13.))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(theme.colors.foreground.opacity(0.9))
                            .child(self.title.clone()),
                    ),
            )
            // 内容区
            .child(div().flex_1().overflow_hidden().child(self.content.clone()))
            // 右下角 resize handle
            .child(
                div()
                    .id(SharedString::from(format!(
                        "plugin-resize-handle-{}",
                        self.plugin_id
                    )))
                    .absolute()
                    .right(px(0.))
                    .bottom(px(0.))
                    .w(px(18.))
                    .h(px(18.))
                    .cursor_nwse_resize()
                    .on_mouse_down(
                        MouseButton::Left,
                        move |event: &MouseDownEvent, _window, cx| {
                            if let Some(e) = entity_resize.upgrade() {
                                e.update(cx, |this, _cx| {
                                    this.start_resize(event.position);
                                });
                            }
                        },
                    )
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
