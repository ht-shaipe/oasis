// DockPanel trait - provides a standard interface for dockable panels.
// Implemented by ToolboxPanel, CodeEditorPanel, MarkdownEditorPanel, etc.

use gpui::{App, Entity, IntoElement, Pixels, Render, Window};

pub trait DockPanel: Render + Sized {
    fn title() -> &'static str;
    fn title_key() -> Option<&'static str> {
        None
    }
    fn description() -> &'static str {
        ""
    }
    fn new_view(window: &mut Window, cx: &mut App) -> Entity<Self>
    where
        Self: Sized;
    fn paddings() -> Pixels {
        gpui::px(0.)
    }
    fn tab_icon() -> Option<gpui_component::IconName> {
        None
    }
    fn on_active(&mut self, _active: bool, _window: &mut Window, _cx: &mut App) {}
}