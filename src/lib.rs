mod app;
mod i18n;
mod panels;
pub mod plugins;

// 插件现在通过 libloading 动态加载（cdylib），无需静态链接

rust_i18n::i18n!("locales", fallback = "en");

pub use app::{
    actions::{About, Quit, SelectLocale, SwitchTheme, SwitchThemeMode},
    app_launcher, app_menus, app_state, background, dock, key_binding, system_tray, themes,
    title_bar,
};
pub use panels::SamplePanel;

use gpui::{
    div, img, AnyView, App, AppContext as _, BorrowAppContext, Context, Entity,
    InteractiveElement as _, IntoElement, ObjectFit, ParentElement, ReadGlobal, Render,
    SharedString, Styled, StyledImage, Window, WindowOptions,
};
#[cfg(not(target_family = "wasm"))]
use gpui::{px, size, Bounds, WindowBounds, WindowKind};
#[cfg(not(target_family = "wasm"))]
use gpui_component::{
    menu::{ContextMenuExt, PopupMenuItem},
    ActiveTheme, TitleBar,
};
use gpui_component::{dock::register_panel, Root};
use toolbox_plugin as _; // 强制链接 rlib 插件 crate

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

        cx.on_action(|_: &About, _cx| {
            // Show about info in console for now
            tracing::info!("oasis - 开发者绿洲 v1.0.0");
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

        // 克隆实体用于事件处理（预留）
        let _root_entity = cx.entity().downgrade();

        // 外层容器
        let mut root = div()
            .w_full()
            .h_full()
            .flex()
            .flex_col()
            .relative()
            // 全局鼠标移动 - 处理窗口拖动、调整大小和 Dock 悬停
            .on_mouse_move(move |event, _window, cx| {
                use crate::app::drag_state::SharedGlobalDragState;
                use crate::app::dock::DockHoverState;

                // 处理 Dock 悬停状态 - 鼠标移出 Dock 区域时清除
                {
                    let dock_state = DockHoverState::global(cx);
                    if dock_state.hovered_plugin_id.is_some() {
                        let mouse_y_px: f32 = event.position.y.into();
                        let window_height: f32 = _window.viewport_size().height.into();
                        // Dock 在底部，距离底部 20px，dock 自身高度约 100px
                        // 如果鼠标 y 坐标小于窗口底部往上 140px 的区域，认为移出了 Dock
                        let dock_top_threshold = window_height - 140.0;
                        let is_outside_dock = mouse_y_px < dock_top_threshold;
                        if is_outside_dock {
                            cx.global_mut::<DockHoverState>().hovered_plugin_id = None;
                        }
                    }
                }

                // 处理窗口拖动和调整大小
                let drag_state = SharedGlobalDragState::global(cx);
                if !drag_state.is_active() {
                    return;
                }

                // 获取拖动状态（克隆数据以避免长时间借用）
                let (dragging_window, _, _) = drag_state.get_drag_state();
                let (resizing_window, _, _) = drag_state.get_resize_state();
                let window_id = dragging_window.or(resizing_window);

                // 获取需要更新的窗口实体（克隆以避免借用）
                let window_entity_to_update = if let Some(wid) = &window_id {
                    let plugin_id = wid.strip_prefix("plugin-").unwrap_or("");
                    cx.global::<crate::plugins::PluginRegistry>()
                        .open_windows
                        .get(plugin_id)
                        .cloned()
                } else {
                    None
                };

                // 更新窗口位置/大小
                if let (Some(wid), Some(window_entity)) = (window_id, window_entity_to_update) {
                    let _ = window_entity.update(cx, |win, _cx| {
                        win.update_from_global_drag(&wid, event.position);
                    });
                }
            })
            // 全局鼠标抬起 - 结束拖动/调整大小
            .on_mouse_up(gpui::MouseButton::Left, move |_event, _window, cx| {
                use crate::app::drag_state::SharedGlobalDragState;

                let drag_state = SharedGlobalDragState::global(cx);
                if drag_state.is_active() {
                    drag_state.end();

                    // 收集需要更新的窗口实体
                    let window_entities: Vec<_> = cx.global::<crate::plugins::PluginRegistry>()
                        .open_windows
                        .values()
                        .cloned()
                        .collect();

                    // 更新所有窗口的本地状态
                    for window_entity in window_entities {
                        let _ = window_entity.update(cx, |win, _cx| {
                            win.end_interaction();
                        });
                    }
                }
            });

        // 背景图片层（从全局 BackgroundSettings 读取）
        let bg_settings = app::background::BackgroundSettings::global(cx);
        let bg_path = bg_settings.get_path_buf();
        if let Some(bg_path) = bg_path.as_ref().filter(|path| path.exists()) {
            root = root.child(
                img(bg_path.clone())
                    .absolute()
                    .inset_0()
                    .w_full()
                    .h_full()
                    .object_fit(ObjectFit::Cover),
            );
        } else {
            if bg_path.is_some() {
                tracing::warn!("background image missing, falling back to solid background");
            }
            root = root.bg(theme.colors.background);
        }

        // 内容区 + 右键菜单（透明背景，让背景图片显示）
        let entity = cx.entity().downgrade();
        let entity2 = cx.entity().downgrade();
        let content = div()
            .flex_1()
            .w_full()
            .overflow_hidden()
            // .bg(gpui::rgba(0x00000000)) // 透明背景
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

        // 应用启动器（根据全局状态条件渲染）
        let launcher_visible = cx.global::<app::app_launcher::AppLauncherState>().visible;
        if launcher_visible {
            root = root.child(app::app_launcher::render_launcher(cx));
        }

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

    let mut window_size = size(px(1200.0), px(800.0));
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
                width: px(1200.),
                height: px(800.),
            }),
            kind: WindowKind::Normal,
            ..Default::default()
        },
        |window, cx| {
            let view = crate_view_fn(window, cx);
            let dock_root = cx.new(|cx| DockRoot::new(title.clone(), view, window, cx));
            cx.new(|cx| Root::new(dock_root, window, cx))
        },
    )
    .expect("failed to open window")
    .update(cx, |_, window, _| {
        window.activate_window();
        window.set_window_title(&title);
        // toggle_fullscreen 在窗口未完成初始化时会 panic，暂不调用
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

        tracing::info!("init: gpui_component::init");
        gpui_component::init(cx);
        tracing::info!("init: app_state::AppState::init");
        app_state::AppState::init(cx);
    // 应用启动器全局状态
    cx.set_global(app::app_launcher::AppLauncherState::default());
    tracing::info!("init: background::init");
    background::init(cx);
    tracing::info!("init: themes::init");
    themes::init(cx);
    tracing::info!("init: plugins::PluginRegistry::init");
    plugins::PluginRegistry::init(cx);
    if std::env::var("OASIS_DISABLE_BUILTIN_WASM_PLUGIN").ok().as_deref() == Some("1") {
        tracing::warn!("init: skip plugins::register_builtin_wasm_plugins (OASIS_DISABLE_BUILTIN_WASM_PLUGIN=1)");
    } else {
        tracing::info!("init: plugins::register_builtin_wasm_plugins");
        plugins::register_builtin_wasm_plugins(cx);
    }
    tracing::info!("init: i18n::init");
    i18n::init(cx);
    tracing::info!("init: app_menus::init");
    app_menus::init("oasis", cx);
    tracing::info!("init: key_binding::init");
    key_binding::init(cx);

	// 注册面板
    register_panel(cx, PANEL_NAME, |_dock_area, _state, _info, window, cx| {
        let panel: gpui::Entity<SamplePanel> = cx.new(|cx| SamplePanel::new(window, cx));
        Box::new(panel) as Box<dyn gpui_component::dock::PanelView>
    });

    if std::env::var("OASIS_DISABLE_TRAY").ok().as_deref() == Some("1") {
        tracing::warn!("init: skip system_tray::init (OASIS_DISABLE_TRAY=1)");
    } else {
        tracing::info!("init: system_tray::init");
        system_tray::init(cx);
    }
    tracing::info!("init: activate");
    cx.activate(true);
}
