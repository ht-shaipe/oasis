mod app;
mod i18n;
mod panels;

rust_i18n::i18n!("locales", fallback = "en");

pub use app::{
    actions::{Quit, SelectLocale, SwitchTheme, SwitchThemeMode},
    app_menus, app_state, key_binding, system_tray, themes, title_bar,
};
pub use panels::SamplePanel;

use gpui::{
    div, AnyView, App, AppContext as _, Context, Entity, IntoElement, ObjectFit, ParentElement,
    Render, SharedString, Styled, Window, WindowOptions, img,
};
#[cfg(not(target_family = "wasm"))]
use gpui::{px, size, Bounds, WindowBounds, WindowKind};
#[cfg(not(target_family = "wasm"))]
use gpui_component::{ActiveTheme, TitleBar};
use gpui_component::{dock::register_panel, Root};

#[cfg(target_family = "wasm")]
use wasm_bindgen::prelude::*;

const PANEL_NAME: &str = "SamplePanel";

#[cfg(target_family = "wasm")]
const GPUI_COMPONENT_ASSETS_BASE: &str = "/gpui-component/gallery/";

// ---- WASM entry ----

#[cfg(target_family = "wasm")]
#[wasm_bindgen]
pub fn init_web() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Info).expect("Failed to initialize logger");
    tracing_wasm::set_as_global_default();

    let app = gpui_platform::single_threaded_web();

    app.with_assets(gpui_component_assets::Assets::new(SharedString::from(
        GPUI_COMPONENT_ASSETS_BASE,
    )))
    .run(move |cx: &mut App| {
        let http_client = unsafe {
            gpui_web::FetchHttpClient::with_user_agent("oasis/0.1.0")
                .expect("failed to create FetchHttpClient")
        };
        cx.set_http_client(std::sync::Arc::new(http_client));

        gpui_component::init(cx);
        app_state::AppState::init(cx);
        themes::init(cx);
        i18n::init(cx);
        key_binding::init(cx);

        register_panel(cx, PANEL_NAME, |_dock_area, _state, _info, window, cx| {
            let panel: gpui::Entity<SamplePanel> = cx.new(|cx| SamplePanel::new(window, cx));
            Box::new(panel) as Box<dyn gpui_component::dock::PanelView>
        });

        cx.on_action(|_: &Quit, cx| {
            cx.quit();
        });

        cx.open_window(WindowOptions::default(), |window, cx| {
            let workspace = Workspace::new(window, cx);
            cx.new(|cx| Root::new(workspace, window, cx))
        })
        .expect("failed to open window");

        cx.activate(true);
    });

    Ok(())
}

// ---- Desktop-only components ----

#[cfg(not(target_family = "wasm"))]
struct DockRoot {
    title_bar: Entity<title_bar::AppTitleBar>,
    view: AnyView,
    /// 背景图片路径，可以是本地绝对路径或嵌入资源路径。
    /// 示例：`Some("/path/to/bg.jpg".into())` 或 `None`（不显示背景图）
    background_image: Option<SharedString>,
}

#[cfg(not(target_family = "wasm"))]
impl DockRoot {
    pub fn new(
        title: impl Into<SharedString>,
        view: impl Into<AnyView>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let title_bar = cx.new(|cx| title_bar::AppTitleBar::new(title, window, cx));
        Self {
            title_bar,
            view: view.into(),
            background_image: None,
        }
    }

    /// 设置背景图片路径（本地文件绝对路径 或 http/https URL）。
    pub fn with_background_image(mut self, path: impl Into<SharedString>) -> Self {
        self.background_image = Some(path.into());
        self
    }
}

#[cfg(not(target_family = "wasm"))]
impl Render for DockRoot {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let sheet_layer = Root::render_sheet_layer(window, cx);
        let dialog_layer = Root::render_dialog_layer(window, cx);
        let notification_layer = Root::render_notification_layer(window, cx);
        let theme = cx.theme();

        // 外层容器：相对定位，用于背景图片绝对定位参照
        let mut root = div()
            .w_full()
            .h_full()
            .flex()
            .flex_col()
            .relative();

        // 背景图片层：绝对定位，铺满整个窗体，位于所有内容之下
        if let Some(bg_path) = self.background_image.clone() {
            root = root.child(
                img(bg_path)
                    .absolute()
                    .inset_0()
                    .w_full()
                    .h_full()
                    .object_fit(ObjectFit::Cover),
            );
        } else {
            // 无背景图时沿用主题纯色背景
            root = root.bg(theme.colors.background);
        }

        root.child(self.title_bar.clone())
            .child(
                div()
                    .flex_1()
                    .w_full()
                    .overflow_hidden()
                    .child(self.view.clone()),
            )
            .children(sheet_layer)
            .children(dialog_layer)
            .children(notification_layer)
    }
}

/// Minimal workspace
struct Workspace {
    content: Entity<SamplePanel>,
}

impl Workspace {
    pub fn new(window: &mut Window, cx: &mut App) -> Entity<Self> {
        let content = cx.new(|cx| SamplePanel::new(window, cx));
        cx.new(|_| Self { content })
    }
}

impl Render for Workspace {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        self.content.clone()
    }
}

#[cfg(not(target_family = "wasm"))]
pub fn open_new<F, E>(title: &str, background_image: Option<&str>, crate_view_fn: F, cx: &mut App)
where
    E: Into<AnyView>,
    F: FnOnce(&mut Window, &mut App) -> E + 'static,
{
    let title = SharedString::from(title.to_string());
    let bg_image: Option<SharedString> = background_image.map(|s| SharedString::from(s.to_string()));

    let mut window_size = size(px(800.0), px(600.0));
    if let Some(display) = cx.primary_display() {
        let display_size = display.bounds().size;
        window_size.width = window_size.width.min(display_size.width * 0.85);
        window_size.height = window_size.height.min(display_size.height * 0.85);
    }
    let window_bounds = Bounds::centered(None, window_size, cx);

    cx.open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(window_bounds)),
            titlebar: Some(TitleBar::title_bar_options()),
            window_min_size: Some(gpui::Size {
                width: px(480.),
                height: px(320.),
            }),
            kind: WindowKind::Normal,
            ..Default::default()
        },
        |window, cx| {
            let view = crate_view_fn(window, cx);
            let root = cx.new(|cx| {
                let mut dock = DockRoot::new(title.clone(), view, window, cx);
                if let Some(bg) = bg_image.clone() {
                    dock = dock.with_background_image(bg);
                }
                dock
            });
            cx.new(|cx| Root::new(root, window, cx))
        },
    )
    .expect("failed to open window")
    .update(cx, |_, window, _| {
        window.activate_window();
        window.set_window_title(&title);
    })
    .expect("failed to update window");
}

#[cfg(not(target_family = "wasm"))]
pub fn init(cx: &mut App) {
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    let _ = tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("gpui_component=trace".parse().unwrap()),
        )
        .try_init();

    gpui_component::init(cx);
    app_state::AppState::init(cx);
    themes::init(cx);
    i18n::init(cx);
    app_menus::init("oasis", cx);
    key_binding::init(cx);

    register_panel(cx, PANEL_NAME, |_dock_area, _state, _info, window, cx| {
        let panel: gpui::Entity<SamplePanel> = cx.new(|cx| SamplePanel::new(window, cx));
        Box::new(panel) as Box<dyn gpui_component::dock::PanelView>
    });

    system_tray::init(cx);
    cx.activate(true);
}