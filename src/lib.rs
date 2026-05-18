mod app;
mod i18n;
mod panels;

rust_i18n::i18n!("locales", fallback = "en");

pub use app::{
    actions::{Quit, SelectLocale, SetBackground, SwitchTheme, SwitchThemeMode},
    app_menus, app_state, key_binding, system_tray, themes, title_bar,
};
pub use panels::SamplePanel;

use gpui::{
    div, img, AnyView, App, AppContext as _, Context, Entity, IntoElement, ObjectFit,
    ParentElement, Render, SharedString, StyledImage, Styled, Window, WindowOptions,
};
#[cfg(not(target_family = "wasm"))]
use std::path::PathBuf;
#[cfg(not(target_family = "wasm"))]
use gpui::{px, size, Bounds, WindowBounds, WindowKind};
#[cfg(not(target_family = "wasm"))]
use gpui_component::{
    menu::{ContextMenuExt, PopupMenuItem},
    ActiveTheme, TitleBar,
};
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
    /// 背景图片路径（本地文件系统路径），None 表示无背景图
    /// 必须使用 PathBuf 而非 SharedString，否则 img() 会走 AssetSource 而非 fs::read
    background_image: Option<PathBuf>,
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

    /// 设置初始背景图片路径
    pub fn with_background_image(mut self, path: impl Into<PathBuf>) -> Self {
        self.background_image = Some(path.into());
        self
    }

    /// 弹出文件选择器，选取后更新背景图
    fn pick_background_image(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let entity = cx.entity().downgrade();
        window
            .spawn(cx, async move |cx| {
                // rfd::FileDialog 在 macOS/Windows 上需在主线程运行；
                // smol::unblock 会把任务提交到线程池，rfd 内部自动回调主线程。
                let path = smol::unblock(|| {
                    rfd::FileDialog::new()
                        .set_title("选择背景图片")
                        .add_filter(
                            "图片",
                            &["png", "jpg", "jpeg", "webp", "gif", "bmp", "tiff"],
                        )
                        .pick_file()
                })
                .await;

                if let Some(path) = path {
                    cx.update(|_, cx| {
                        if let Some(e) = entity.upgrade() {
                            e.update(cx, |this, cx| {
                                this.background_image = Some(path);
                                cx.notify();
                            });
                        }
                    })
                    .ok();
                }
            })
            .detach();
    }

    /// 清除背景图片
    fn clear_background_image(&mut self, cx: &mut Context<Self>) {
        self.background_image = None;
        cx.notify();
    }
}

#[cfg(not(target_family = "wasm"))]
impl Render for DockRoot {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let sheet_layer = Root::render_sheet_layer(window, cx);
        let dialog_layer = Root::render_dialog_layer(window, cx);
        let notification_layer = Root::render_notification_layer(window, cx);
        let theme = cx.theme();

        // 外层容器
        let mut root = div()
            .w_full()
            .h_full()
            .flex()
            .flex_col()
            .relative();

        // 背景图片层（绝对定位，铺满，cover 模式）
        // 必须传 PathBuf 给 img()，传字符串会走 AssetSource 导致加载失败
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
            root = root.bg(theme.colors.background);
        }

        // 内容区 + 右键菜单
        let entity = cx.entity().downgrade();
        let entity2 = cx.entity().downgrade();
        let content = div()
            .flex_1()
            .w_full()
            .overflow_hidden()
            .child(self.view.clone())
            .context_menu(move |menu, _window, _cx| {
                menu.item(
                    PopupMenuItem::new("设置背景图片").on_click({
                        let entity = entity.clone();
                        move |_ev, window, cx| {
                            if let Some(e) = entity.upgrade() {
                                e.update(cx, |this, cx| {
                                    this.pick_background_image(window, cx);
                                });
                            }
                        }
                    }),
                )
                .item(
                    PopupMenuItem::new("清除背景图片").on_click({
                        let entity = entity2.clone();
                        move |_ev, _window, cx| {
                            if let Some(e) = entity.upgrade() {
                                e.update(cx, |this, cx| {
                                    this.clear_background_image(cx);
                                });
                            }
                        }
                    }),
                )
            });

        root.child(self.title_bar.clone())
            .child(content)
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
pub fn open_new<F, E>(title: &str, background_image: Option<&PathBuf>, crate_view_fn: F, cx: &mut App)
where
    E: Into<AnyView>,
    F: FnOnce(&mut Window, &mut App) -> E + 'static,
{
    let title = SharedString::from(title.to_string());
    let bg_image: Option<PathBuf> = background_image.cloned();

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
        window.toggle_fullscreen();
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