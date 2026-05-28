use gpui::{App, Global, SharedString};
use serde::{Deserialize, Serialize};

pub use crate::app::background::{BackgroundManager, BackgroundSettings};
pub use crate::app::dock::DockHoverState;
pub use crate::app::drag_state::SharedGlobalDragState;

/// Application settings persisted to state file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    #[serde(default = "default_locale")]
    pub locale: SharedString,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            locale: default_locale(),
        }
    }
}

impl Global for AppSettings {}

impl AppSettings {
    pub fn global(cx: &App) -> &Self {
        cx.global::<Self>()
    }

    pub fn global_mut(cx: &mut App) -> &mut Self {
        cx.global_mut::<Self>()
    }
}

fn default_locale() -> SharedString {
    detect_system_locale().unwrap_or_else(|| "en".into())
}

fn detect_system_locale() -> Option<SharedString> {
    let raw = sys_locale::get_locale().or_else(|| std::env::var("LANG").ok())?;
    normalize_locale(&raw).map(SharedString::from)
}

fn normalize_locale(locale: &str) -> Option<&'static str> {
    let lower = locale.to_lowercase();
    if lower.starts_with("zh") {
        return Some("zh_cn");
    }
    if lower.starts_with("en") {
        return Some("en");
    }
    None
}

/// Minimal app state
pub struct AppState {
    app_title: SharedString,
}

impl AppState {
    pub fn init(cx: &mut App) {
        tracing::info!("🚀 AppState::init() 开始初始化");
        cx.set_global::<AppState>(Self {
            app_title: SharedString::from(""),
        });

        // Initialize background settings
        tracing::info!("📋 设置 BackgroundSettings 全局状态");
        cx.set_global::<BackgroundSettings>(BackgroundSettings::default());

        // Initialize dock hover state
        tracing::info!("🖱️ 设置 DockHoverState 全局状态");
        cx.set_global::<DockHoverState>(DockHoverState::default());

        // Initialize global drag state
        tracing::info!("🖐️ 设置 SharedGlobalDragState 全局状态");
        cx.set_global::<SharedGlobalDragState>(SharedGlobalDragState::default());

        tracing::info!("🎨 初始化 BackgroundManager");
        crate::app::background::init(cx);
        tracing::info!("✅ AppState::init() 完成");
    }

    pub fn global(cx: &App) -> &Self {
        cx.global::<Self>()
    }

    pub fn global_mut(cx: &mut App) -> &mut Self {
        cx.global_mut::<Self>()
    }

    pub fn set_app_title(&mut self, title: SharedString) {
        self.app_title = title;
    }

    pub fn app_title(&self) -> &SharedString {
        &self.app_title
    }
}

impl Global for AppState {}