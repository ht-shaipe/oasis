use gpui::{
    div, px, rgb, AnyView, ClickEvent, Context, InteractiveElement as _, IntoElement,
    MouseButton, MouseDownEvent, ParentElement, Pixels, Point, ReadGlobal, Render,
    SharedString, StatefulInteractiveElement as _, Styled as _, Window,
};
use gpui_component::ActiveTheme as _;

use crate::app::drag_state::SharedGlobalDragState;

use super::PluginRegistry;

/// 调整大小方向
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResizeDirection {
    Top,
    Bottom,
    Left,
    Right,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

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
    /// 调整大小方向
    pub resize_direction: Option<ResizeDirection>,
    /// 调整大小开始时鼠标位置
    pub resize_start: Point<Pixels>,
    /// 调整大小开始时窗口位置和尺寸
    pub resize_origin: (Point<Pixels>, (Pixels, Pixels)),
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
            resize_direction: None,
            resize_start: Point::default(),
            resize_origin: (Point::default(), (px(0.), px(0.))),
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

    /// 从全局拖动状态更新位置或大小
    pub fn update_from_global_drag(&mut self, window_id: &str, mouse_pos: Point<Pixels>) {
        if window_id != format!("plugin-{}", self.plugin_id) {
            return;
        }

        if self.dragging {
            let dx = mouse_pos.x - self.drag_start.x;
            let dy = mouse_pos.y - self.drag_start.y;
            self.position = Point::new(self.drag_origin.x + dx, self.drag_origin.y + dy);
        } else if let Some(direction) = self.resize_direction {
            let dx = mouse_pos.x - self.resize_start.x;
            let dy = mouse_pos.y - self.resize_start.y;
            let (origin_pos, origin_size) = self.resize_origin;
            let min_w = px(200.0);
            let min_h = px(150.0);

            match direction {
                ResizeDirection::Right => {
                    self.size.0 = (origin_size.0 + dx).max(min_w);
                }
                ResizeDirection::Bottom => {
                    self.size.1 = (origin_size.1 + dy).max(min_h);
                }
                ResizeDirection::Left => {
                    let new_width = (origin_size.0 - dx).max(min_w);
                    self.size.0 = new_width;
                    self.position.x = origin_pos.x + (origin_size.0 - new_width);
                }
                ResizeDirection::Top => {
                    let new_height = (origin_size.1 - dy).max(min_h);
                    self.size.1 = new_height;
                    self.position.y = origin_pos.y + (origin_size.1 - new_height);
                }
                ResizeDirection::TopRight => {
                    let new_height = (origin_size.1 - dy).max(min_h);
                    self.size.0 = (origin_size.0 + dx).max(min_w);
                    self.size.1 = new_height;
                    self.position.y = origin_pos.y + (origin_size.1 - new_height);
                }
                ResizeDirection::BottomRight => {
                    self.size.0 = (origin_size.0 + dx).max(min_w);
                    self.size.1 = (origin_size.1 + dy).max(min_h);
                }
                ResizeDirection::BottomLeft => {
                    let new_width = (origin_size.0 - dx).max(min_w);
                    self.size.0 = new_width;
                    self.size.1 = (origin_size.1 + dy).max(min_h);
                    self.position.x = origin_pos.x + (origin_size.0 - new_width);
                }
                ResizeDirection::TopLeft => {
                    let new_width = (origin_size.0 - dx).max(min_w);
                    let new_height = (origin_size.1 - dy).max(min_h);
                    self.size.0 = new_width;
                    self.size.1 = new_height;
                    self.position.x = origin_pos.x + (origin_size.0 - new_width);
                    self.position.y = origin_pos.y + (origin_size.1 - new_height);
                }
            }
        }
    }

    /// 结束拖拽或调整大小
    pub fn end_interaction(&mut self) {
        self.dragging = false;
        self.resizing = false;
        self.resize_direction = None;
    }
}

impl Render for PluginWindow {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let is_dark = theme.mode.is_dark();

        let entity = cx.entity().downgrade();
        let entity_for_drag = entity.clone();
        let entity_for_resize = entity.clone();
        let entity_min = entity.clone();
        let entity_max = entity.clone();

        // 关闭按钮需要 plugin_id
        let plugin_id_for_close = self.plugin_id.clone();

        // 用于全局拖动状态的唯一 ID
        let window_id = format!("plugin-{}", self.plugin_id);
        let window_id_for_drag = window_id.clone();

        // 当前窗口位置和尺寸（用于拖动开始时记录）
        let current_pos = self.position;
        let current_size = self.size;

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

        // 调整手柄样式
        let handle_color = theme.colors.muted_foreground.opacity(0.3);

        div()
            .id(SharedString::from(format!("plugin-window-{}", self.plugin_id)))
            .absolute()
            .left(self.position.x)
            .top(self.position.y)
            .w(self.size.0)
            .h(self.size.1)
            .flex()
            .flex_col()
            .rounded_2xl()
            .bg(bg_color)
            .shadow_xl()
            .border_1()
            .border_color(theme.colors.border.opacity(0.2))
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
                    // 拖拽：鼠标按下 - 使用全局拖动状态
                    .on_mouse_down(
                        MouseButton::Left,
                        move |event: &MouseDownEvent, _window, cx| {
                            // 记录当前窗口状态到全局
                            let drag_state = SharedGlobalDragState::global(cx);
                            drag_state.start_drag(
                                window_id_for_drag.clone(),
                                event.position,
                                current_pos,
                            );

                            // 同时更新本地状态
                            if let Some(e) = entity_for_drag.upgrade() {
                                e.update(cx, |this, _cx| {
                                    this.dragging = true;
                                    this.drag_start = event.position;
                                    this.drag_origin = current_pos;
                                });
                            }
                        },
                    )
                    // macOS 风格窗口按钮
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
                                    .bg(rgb(0xFF5C60))
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
                                    .bg(rgb(0xFAC800))
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
                                    .bg(rgb(0x35C759))
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
            // === 8 个调整大小的手柄 ===
            // 上边手柄
            .child(
                div()
                    .absolute()
                    .top(px(0.))
                    .left(px(12.))
                    .right(px(12.))
                    .h(px(4.))
                    .cursor_n_resize()
                    .on_mouse_down(MouseButton::Left, {
                        let entity = entity_for_resize.clone();
                        let window_id = window_id.clone();
                        let current_pos = current_pos;
                        let current_size = current_size;
                        move |event: &MouseDownEvent, _window, cx| {
                            let drag_state = SharedGlobalDragState::global(cx);
                            drag_state.start_resize(window_id.clone(), event.position, current_size);
                            if let Some(e) = entity.upgrade() {
                                e.update(cx, |this, _cx| {
                                    this.resizing = true;
                                    this.resize_direction = Some(ResizeDirection::Top);
                                    this.resize_start = event.position;
                                    this.resize_origin = (current_pos, current_size);
                                });
                            }
                        }
                    }),
            )
            // 下边手柄
            .child(
                div()
                    .absolute()
                    .bottom(px(0.))
                    .left(px(12.))
                    .right(px(12.))
                    .h(px(4.))
                    .cursor_s_resize()
                    .on_mouse_down(MouseButton::Left, {
                        let entity = entity_for_resize.clone();
                        let window_id = window_id.clone();
                        let current_pos = current_pos;
                        let current_size = current_size;
                        move |event: &MouseDownEvent, _window, cx| {
                            let drag_state = SharedGlobalDragState::global(cx);
                            drag_state.start_resize(window_id.clone(), event.position, current_size);
                            if let Some(e) = entity.upgrade() {
                                e.update(cx, |this, _cx| {
                                    this.resizing = true;
                                    this.resize_direction = Some(ResizeDirection::Bottom);
                                    this.resize_start = event.position;
                                    this.resize_origin = (current_pos, current_size);
                                });
                            }
                        }
                    }),
            )
            // 左边手柄
            .child(
                div()
                    .absolute()
                    .left(px(0.))
                    .top(px(12.))
                    .bottom(px(12.))
                    .w(px(4.))
                    .cursor_w_resize()
                    .on_mouse_down(MouseButton::Left, {
                        let entity = entity_for_resize.clone();
                        let window_id = window_id.clone();
                        let current_pos = current_pos;
                        let current_size = current_size;
                        move |event: &MouseDownEvent, _window, cx| {
                            let drag_state = SharedGlobalDragState::global(cx);
                            drag_state.start_resize(window_id.clone(), event.position, current_size);
                            if let Some(e) = entity.upgrade() {
                                e.update(cx, |this, _cx| {
                                    this.resizing = true;
                                    this.resize_direction = Some(ResizeDirection::Left);
                                    this.resize_start = event.position;
                                    this.resize_origin = (current_pos, current_size);
                                });
                            }
                        }
                    }),
            )
            // 右边手柄
            .child(
                div()
                    .absolute()
                    .right(px(0.))
                    .top(px(12.))
                    .bottom(px(12.))
                    .w(px(4.))
                    .cursor_e_resize()
                    .on_mouse_down(MouseButton::Left, {
                        let entity = entity_for_resize.clone();
                        let window_id = window_id.clone();
                        let current_pos = current_pos;
                        let current_size = current_size;
                        move |event: &MouseDownEvent, _window, cx| {
                            let drag_state = SharedGlobalDragState::global(cx);
                            drag_state.start_resize(window_id.clone(), event.position, current_size);
                            if let Some(e) = entity.upgrade() {
                                e.update(cx, |this, _cx| {
                                    this.resizing = true;
                                    this.resize_direction = Some(ResizeDirection::Right);
                                    this.resize_start = event.position;
                                    this.resize_origin = (current_pos, current_size);
                                });
                            }
                        }
                    }),
            )
            // 左上角手柄
            .child(
                div()
                    .absolute()
                    .left(px(0.))
                    .top(px(0.))
                    .w(px(12.))
                    .h(px(12.))
                    .cursor_nwse_resize()
                    .on_mouse_down(MouseButton::Left, {
                        let entity = entity_for_resize.clone();
                        let window_id = window_id.clone();
                        let current_pos = current_pos;
                        let current_size = current_size;
                        move |event: &MouseDownEvent, _window, cx| {
                            let drag_state = SharedGlobalDragState::global(cx);
                            drag_state.start_resize(window_id.clone(), event.position, current_size);
                            if let Some(e) = entity.upgrade() {
                                e.update(cx, |this, _cx| {
                                    this.resizing = true;
                                    this.resize_direction = Some(ResizeDirection::TopLeft);
                                    this.resize_start = event.position;
                                    this.resize_origin = (current_pos, current_size);
                                });
                            }
                        }
                    }),
            )
            // 右上角手柄
            .child(
                div()
                    .absolute()
                    .right(px(0.))
                    .top(px(0.))
                    .w(px(12.))
                    .h(px(12.))
                    .cursor_n_resize()
                    .on_mouse_down(MouseButton::Left, {
                        let entity = entity_for_resize.clone();
                        let window_id = window_id.clone();
                        let current_pos = current_pos;
                        let current_size = current_size;
                        move |event: &MouseDownEvent, _window, cx| {
                            let drag_state = SharedGlobalDragState::global(cx);
                            drag_state.start_resize(window_id.clone(), event.position, current_size);
                            if let Some(e) = entity.upgrade() {
                                e.update(cx, |this, _cx| {
                                    this.resizing = true;
                                    this.resize_direction = Some(ResizeDirection::TopRight);
                                    this.resize_start = event.position;
                                    this.resize_origin = (current_pos, current_size);
                                });
                            }
                        }
                    }),
            )
            // 左下角手柄
            .child(
                div()
                    .absolute()
                    .left(px(0.))
                    .bottom(px(0.))
                    .w(px(12.))
                    .h(px(12.))
                    .cursor_s_resize()
                    .on_mouse_down(MouseButton::Left, {
                        let entity = entity_for_resize.clone();
                        let window_id = window_id.clone();
                        let current_pos = current_pos;
                        let current_size = current_size;
                        move |event: &MouseDownEvent, _window, cx| {
                            let drag_state = SharedGlobalDragState::global(cx);
                            drag_state.start_resize(window_id.clone(), event.position, current_size);
                            if let Some(e) = entity.upgrade() {
                                e.update(cx, |this, _cx| {
                                    this.resizing = true;
                                    this.resize_direction = Some(ResizeDirection::BottomLeft);
                                    this.resize_start = event.position;
                                    this.resize_origin = (current_pos, current_size);
                                });
                            }
                        }
                    })
                    // 左下角调整图标
                    .child(
                        div()
                            .absolute()
                            .left(px(3.))
                            .bottom(px(3.))
                            .flex()
                            .flex_col()
                            .gap(px(1.))
                            .child(div().h(px(1.)).w(px(6.)).rounded_full().bg(handle_color))
                            .child(div().h(px(1.)).w(px(4.)).rounded_full().bg(handle_color))
                            .child(div().h(px(1.)).w(px(2.)).rounded_full().bg(handle_color)),
                    ),
            )
            // 右下角手柄
            .child(
                div()
                    .id(SharedString::from(format!(
                        "plugin-resize-handle-{}",
                        self.plugin_id
                    )))
                    .absolute()
                    .right(px(0.))
                    .bottom(px(0.))
                    .w(px(12.))
                    .h(px(12.))
                    .cursor_nwse_resize()
                    .on_mouse_down(MouseButton::Left, {
                        let entity = entity_for_resize.clone();
                        let window_id = window_id.clone();
                        let current_pos = current_pos;
                        let current_size = current_size;
                        move |event: &MouseDownEvent, _window, cx| {
                            let drag_state = SharedGlobalDragState::global(cx);
                            drag_state.start_resize(window_id.clone(), event.position, current_size);
                            if let Some(e) = entity.upgrade() {
                                e.update(cx, |this, _cx| {
                                    this.resizing = true;
                                    this.resize_direction = Some(ResizeDirection::BottomRight);
                                    this.resize_start = event.position;
                                    this.resize_origin = (current_pos, current_size);
                                });
                            }
                        }
                    })
                    // 右下角调整图标
                    .child(
                        div()
                            .absolute()
                            .right(px(3.))
                            .bottom(px(3.))
                            .flex()
                            .flex_col()
                            .gap(px(1.))
                            .child(div().h(px(1.)).w(px(6.)).rounded_full().bg(handle_color))
                            .child(div().h(px(1.)).w(px(4.)).rounded_full().bg(handle_color))
                            .child(div().h(px(1.)).w(px(2.)).rounded_full().bg(handle_color)),
                    ),
            )
    }
}
