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
    /// 插件内容视图
    pub content: AnyView,
}

impl PluginWindow {
    /// 构造 PluginWindow（不依赖 Window/cx，由外层 `cx.new(|_| ...)` 包装为 Entity）
    pub fn new(
        plugin_id: &str,
        title: String,
        window_size: (f32, f32),
        content: AnyView,
    ) -> Self {
        Self {
            plugin_id: plugin_id.to_string(),
            title,
            position: Point::new(px(120.0), px(80.0)),
            size: (px(window_size.0), px(window_size.1)),
            dragging: false,
            drag_start: Point::default(),
            drag_origin: Point::default(),
            content,
        }
    }
}

impl Render for PluginWindow {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let is_dark = theme.mode.is_dark();

        let entity = cx.entity().downgrade();
        let entity2 = entity.clone();
        let entity3 = entity.clone();

        // 关闭按钮需要 plugin_id
        let plugin_id_for_close = self.plugin_id.clone();

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
            .w(self.size.0)
            .h(self.size.1)
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
                    .id(SharedString::from(format!(
                        "plugin-window-titlebar-{}",
                        self.plugin_id
                    )))
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
                            .child(self.title.clone()),
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
                            // 黄色最小化按钮（MVP 无操作）
                            .child(
                                div().size(px(12.)).rounded_full().bg(gpui::yellow().opacity(0.8)),
                            )
                            // 绿色最大化按钮（MVP 无操作）
                            .child(
                                div().size(px(12.)).rounded_full().bg(gpui::green().opacity(0.8)),
                            ),
                    ),
            )
            // 内容区
            .child(
                div()
                    .flex_1()
                    .overflow_hidden()
                    .child(self.content.clone()),
            )
    }
}
