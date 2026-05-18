//! macOS-style Dock bar at bottom

use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::dock::{Panel, PanelControl, PanelEvent};
use gpui_component::{ActiveTheme as _, Icon, IconName};
use crate::app_state::AppSettings;

/// macOS-style Dock item
pub struct DockItem {
    pub id: &'static str,
    pub icon: IconName,
    pub label: String,
    pub is_active: bool,
}

/// macOS-style Dock bar
pub struct MacDockPanel {
    focus_handle: FocusHandle,
    hovered_item: Option<String>,
}

impl MacDockPanel {
    pub fn new(_window: &mut Window, cx: &mut App) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            hovered_item: None,
        }
    }

    fn get_dock_items(cx: &App) -> Vec<DockItem> {
        let settings = AppSettings::global(cx);
        let active_tool = settings.get_active_tool_id();

        vec![
            DockItem {
                id: "finder",
                icon: IconName::Folder,
                label: "文件".to_string(),
                is_active: false,
            },
            DockItem {
                id: "code_editor",
                icon: IconName::File,
                label: "代码".to_string(),
                is_active: active_tool == Some("code_editor"),
            },
            DockItem {
                id: "markdown_editor",
                icon: IconName::BookOpen,
                label: "笔记".to_string(),
                is_active: active_tool == Some("markdown_editor"),
            },
            DockItem {
                id: "credential_manager",
                icon: IconName::CircleUser,
                label: "钥匙".to_string(),
                is_active: active_tool == Some("credential_manager"),
            },
            DockItem {
                id: "toolbox",
                icon: IconName::Settings,
                label: "工具".to_string(),
                is_active: false,
            },
            DockItem {
                id: "settings",
                icon: IconName::Settings2,
                label: "设置".to_string(),
                is_active: settings.show_settings,
            },
        ]
    }

    fn open_dock_item(&mut self, item_id: &str, window: &mut Window, cx: &mut Context<Self>) {
        match item_id {
            "finder" => {
                // TODO: Open file browser
            }
            "code_editor" | "markdown_editor" | "credential_manager" => {
                AppSettings::global_mut(cx).open_tool_tab(item_id.to_string());
            }
            "toolbox" => {
                AppSettings::global_mut(cx).show_right_panel = true;
            }
            "settings" => {
                AppSettings::global_mut(cx).show_settings = true;
            }
            _ => {}
        }
        window.refresh();
    }
}

impl Panel for MacDockPanel {
    fn panel_name(&self) -> &'static str {
        "MacDockPanel"
    }

    fn title(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().h_px().into_any_element()
    }

    fn zoomable(&self, _cx: &App) -> Option<PanelControl> {
        None
    }
}

impl EventEmitter<PanelEvent> for MacDockPanel {}

impl Focusable for MacDockPanel {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for MacDockPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let items = Self::get_dock_items(cx);
        let entity = cx.entity().clone();

        // macOS Dock: centered icons on translucent bar
        div()
            .id("mac-dock")
            .flex()
            .flex_row()
            .w_full()
            .h_full()
            .items_center()
            .justify_center()
            .gap(px(4.))
            .bg(theme.colors.background)
            .border_t(px(1.))
            .border_color(theme.colors.border.alpha(0.3))
            .children(items.iter().map(|item| {
                let item_id = item.id;
                let is_active = item.is_active;
                let is_hovered = self.hovered_item.as_deref() == Some(item_id);
                let entity_clone = entity.clone();

                div()
                    .id(SharedString::from(item_id))
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .w(px(52.))
                    .h(px(52.))
                    .cursor_pointer()
                    .rounded(px(10.))
                    .when(is_hovered, |this| {
                        this.bg(theme.colors.accordion_hover)
                    })
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .items_center()
                            .gap(px(2.))
                            .child(
                                Icon::new(item.icon.clone())
                                    .size(px(28.))
                                    .text_color(if is_hovered || is_active {
                                        theme.colors.foreground
                                    } else {
                                        theme.colors.muted_foreground
                                    })
                            )
                            // Active indicator dot
                            .when(is_active, |this| {
                                this.child(
                                    div()
                                        .w(px(4.))
                                        .h(px(4.))
                                        .rounded(px(2.))
                                        .bg(theme.colors.primary)
                                )
                            })
                    )
                    .on_mouse_move({
                        let item_id = item_id.to_string();
                        let entity_clone = entity_clone.clone();
                        move |_ev, _window, cx| {
                            entity_clone.update(cx, |this, _cx| {
                                this.hovered_item = Some(item_id.clone());
                            });
                        }
                    })
                    .on_click({
                        let item_id = item_id.to_string();
                        move |_ev, window, cx| {
                            entity_clone.update(cx, |this, cx| {
                                this.open_dock_item(&item_id, window, cx);
                            });
                        }
                    })
            }))
    }
}