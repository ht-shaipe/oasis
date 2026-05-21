mod commands;
mod model;
mod services;
mod text_utils;
mod theme;
mod editor_view;
mod file_explorer;
mod aster_view;

pub use aster_view::AsterView;

use gpui::{AnyView, App, AppContext as _, Window};

/// 插件视图工厂 — 供宿主手动注册调用
pub fn create_aster_view(window: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|cx| AsterView::new(window, cx)).into()
}