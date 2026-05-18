mod app;
mod i18n;
mod panels;
pub mod plugins;

rust_i18n::i18n!("locales", fallback = "en");

pub use app::{
    actions::{Quit, SelectLocale, SwitchTheme, SwitchThemeMode},
    app_menus, app_state, background, dock, floating_window, key_binding, system_tray, themes,
    title_bar,
};
pub use panels::SamplePanel;

use gpui::{
    div, img, AnyView, App, AppContext as _, BorrowAppContext, Context, Entity, IntoElement,
    ObjectFit, ParentElement, Render, SharedString, Styled, StyledImage, Window, WindowOptions,
};
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
    dock: Entity<app::dock::FloatingDock>,
    view: AnyView,
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
        let dock = cx.new(|cx| app::dock::FloatingDock::new(window, cx));
        Self {
            title_bar,
            dock,
            view: view.into(),
        }
    }

    /// 弹出文件选择器，选取后更新背景图
    fn pick_background_image(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let entity = cx.entity().downgrade();
        window
            .spawn(cx, async move |cx| {
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
                    tracing::info!("📸 用户选择了背景图片: {:?}", path);
                    let path_str = path.to_str().map(|s| s.to_string());
                    cx.update(|_, cx| {
                        cx.update_default_global::<app::background::BackgroundSettings, _>(|settings, _cx| {
                            settings.set_background_image(path_str);
                        });
                    })
                    .ok();

                    if let Some(e) = entity.upgrade() {
                        e.update(cx, |_this, cx| {
                            cx.notify();
                        })
                        .ok();
                    }
                } else {
                    tracing::info!("❌ 用户取消了背景图片选择");
                }
            })
            .detach();
    }

    /// 清除背景图片
    fn clear_background_image(&mut self, cx: &mut Context<Self>) {
        tracing::info!("🗑️ 清除背景图片");
        cx.update_default_global::<app::background::BackgroundSettings, _>(|settings, _cx| {
            settings.set_background_image(None);
        });
        cx.notify();
    }
}

#[cfg(not(target_family = "wasm"))]
impl Render for DockRoot {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        // 外层容器
        let mut root = div()
            .w_full()
            .h_full()
            .flex()
            .flex_col()
            .relative();

        // 背景图片层（从全局 BackgroundSettings 读取）
        let bg_settings = app::background::BackgroundSettings::global(cx);
        let bg_path = bg_settings.get_path_buf();
        if let Some(bg_path) = bg_path {
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

        // 内容区 + 右键菜单（透明背景，让背景图片显示）
        let entity = cx.entity().downgrade();
        let entity2 = cx.entity().downgrade();
        let content = div()
            .flex_1()
            .w_full()
            .overflow_hidden()
            .bg(gpui::rgba(0x00000000)) // 透明背景
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

        // 添加标题栏（必须在 content 之前，这样才会显示在顶部）
        root = root.child(self.title_bar.clone());

        // 添加内容区
        root = root.child(content);

        // 添加浮动 Dock
        root = root.child(self.dock.clone());

        // 渲染所有打开的插件窗口
        let open_windows = cx
            .global::<plugins::PluginRegistry>()
            .open_windows
            .values()
            .cloned()
            .collect::<Vec<_>>();

        for win in open_windows {
            root = root.child(win);
        }

        // 添加全局层
        let sheet_layer = Root::render_sheet_layer(_window, cx);
        let dialog_layer = Root::render_dialog_layer(_window, cx);
        let notification_layer = Root::render_notification_layer(_window, cx);

        root = root.children(sheet_layer);
        root = root.children(dialog_layer);
        root = root.children(notification_layer);

        root
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
pub fn open_new<F, E>(title: &str, crate_view_fn: F, cx: &mut App)
where
    E: Into<AnyView>,
    F: FnOnce(&mut Window, &mut App) -> E + 'static,
{
    let title = SharedString::from(title.to_string());

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
            // 直接使用 DockRoot，不包装在 Root 中
            cx.new(|cx| DockRoot::new(title.clone(), view, window, cx))
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
    background::init(cx);
    themes::init(cx);
    plugins::PluginRegistry::init(cx);
    plugins::register_builtin_wasm_plugins(cx);
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
