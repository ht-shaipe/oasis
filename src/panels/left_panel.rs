//! Left panel - Sidebar with project features

use gpui::prelude::*;
use gpui::InteractiveElement;
use gpui::*;
use gpui_component::dock::{Panel, PanelControl};
use gpui_component::{ActiveTheme as _, Icon, IconName};
use rust_i18n::t;
use crate::components::sidebar::{Sidebar, SidebarMenuItem};
use crate::app_state::AppSettings;

/// 左侧面板 - 功能导航
pub struct LeftPanel {
    focus_handle: FocusHandle,
    collapsed: bool,
}

impl LeftPanel {
    pub fn new(_window: &mut Window, cx: &mut App) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            collapsed: false,
        }
    }

    /// 打开工具标签
    fn open_tool(&self, tool_id: &str, window: &mut Window, cx: &mut Context<Self>) {
        AppSettings::global_mut(cx).open_tool_tab(tool_id.to_string());
        window.refresh();
    }
}

impl Panel for LeftPanel {
    fn panel_name(&self) -> &'static str {
        "功能导航"
    }

    fn title(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().h_px().into_any_element()
    }

    fn zoomable(&self, _cx: &App) -> Option<PanelControl> {
        None
    }
}

impl EventEmitter<gpui_component::dock::PanelEvent> for LeftPanel {}

impl Focusable for LeftPanel {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for LeftPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let collapsed = self.collapsed;
        let this_panel = cx.entity().clone();

        // 创建工具箱子菜单（带子菜单）
        let toolbox_menu = SidebarMenuItem::new("工具箱")
            .icon(Icon::new(IconName::Settings).size_4())
            .default_open(true)
            .click_to_open(true)
            .children(vec![
                // CSV 工具子菜单
                SidebarMenuItem::new("CSV 工具")
                    .icon(Icon::new(IconName::File).size_3())
                    .default_open(true)
                    .click_to_open(true)
                    .children(vec![
                        SidebarMenuItem::new("CSV 统计")
                            .icon(Icon::new(IconName::File).size_3())
                            .on_click({
                                let this_panel = this_panel.clone();
                                move |_ev, window, cx| {
                                    this_panel.update(cx, |this, cx| {
                                        this.open_tool("csv_stats", window, cx);
                                    });
                                }
                            }),
                        SidebarMenuItem::new("CSV 拆分")
                            .icon(Icon::new(IconName::File).size_3())
                            .on_click({
                                let this_panel = this_panel.clone();
                                move |_ev, window, cx| {
                                    this_panel.update(cx, |this, cx| {
                                        this.open_tool("csv_split", window, cx);
                                    });
                                }
                            }),
                        SidebarMenuItem::new("CSV 转换")
                            .icon(Icon::new(IconName::File).size_3())
                            .on_click({
                                let this_panel = this_panel.clone();
                                move |_ev, window, cx| {
                                    this_panel.update(cx, |this, cx| {
                                        this.open_tool("csv_convert", window, cx);
                                    });
                                }
                            }),
                    ]),
                // JSON 工具子菜单
                SidebarMenuItem::new("JSON 工具")
                    .icon(Icon::new(IconName::File).size_3())
                    .default_open(true)
                    .click_to_open(true)
                    .children(vec![
                        SidebarMenuItem::new("JSON 转换")
                            .icon(Icon::new(IconName::File).size_3())
                            .on_click({
                                let this_panel = this_panel.clone();
                                move |_ev, window, cx| {
                                    this_panel.update(cx, |this, cx| {
                                        this.open_tool("json_convert", window, cx);
                                    });
                                }
                            }),
                        SidebarMenuItem::new("JSON 合并")
                            .icon(Icon::new(IconName::File).size_3())
                            .on_click({
                                let this_panel = this_panel.clone();
                                move |_ev, window, cx| {
                                    this_panel.update(cx, |this, cx| {
                                        this.open_tool("json_merge", window, cx);
                                    });
                                }
                            }),
                    ]),
                // API 工具子菜单
                SidebarMenuItem::new("API 工具")
                    .icon(Icon::new(IconName::Globe).size_3())
                    .default_open(true)
                    .click_to_open(true)
                    .children(vec![
                        SidebarMenuItem::new("API 请求")
                            .icon(Icon::new(IconName::Globe).size_3())
                            .on_click({
                                let this_panel = this_panel.clone();
                                move |_ev, window, cx| {
                                    this_panel.update(cx, |this, cx| {
                                        this.open_tool("api_request", window, cx);
                                    });
                                }
                            }),
                        SidebarMenuItem::new("API 批量下载")
                            .icon(Icon::new(IconName::ArrowRight).size_3())
                            .on_click({
                                let this_panel = this_panel.clone();
                                move |_ev, window, cx| {
                                    this_panel.update(cx, |this, cx| {
                                        this.open_tool("api_batch_download", window, cx);
                                    });
                                }
                            }),
                    ]),
                // 文件工具子菜单
                SidebarMenuItem::new("文件工具")
                    .icon(Icon::new(IconName::Folder).size_3())
                    .default_open(true)
                    .click_to_open(true)
                    .children(vec![
                        SidebarMenuItem::new("批量重命名")
                            .icon(Icon::new(IconName::File).size_3())
                            .on_click({
                                let this_panel = this_panel.clone();
                                move |_ev, window, cx| {
                                    this_panel.update(cx, |this, cx| {
                                        this.open_tool("batch_rename", window, cx);
                                    });
                                }
                            }),
                        SidebarMenuItem::new("Excel 移动文件")
                            .icon(Icon::new(IconName::Folder).size_3())
                            .on_click({
                                let this_panel = this_panel.clone();
                                move |_ev, window, cx| {
                                    this_panel.update(cx, |this, cx| {
                                        this.open_tool("excel_move", window, cx);
                                    });
                                }
                            }),
                    ]),
                // 网络工具
                SidebarMenuItem::new("网络扫描")
                    .icon(Icon::new(IconName::Globe).size_3())
                    .on_click({
                        let this_panel = this_panel.clone();
                        move |_ev, window, cx| {
                            this_panel.update(cx, |this, cx| {
                                this.open_tool("network_scan", window, cx);
                            });
                        }
                    }),
            ]);

        // 创建主菜单
        let main_menu_items = vec![
            // 代码编辑
            SidebarMenuItem::new(t!("code_editor.title"))
                .icon(Icon::new(IconName::File).size_4())
                .on_click({
                    let this_panel = this_panel.clone();
                    move |_ev, window, cx| {
                        this_panel.update(cx, |this, cx| {
                            this.open_tool("code_editor", window, cx);
                        });
                    }
                }),
            // Markdown 编辑
            SidebarMenuItem::new(t!("markdown_editor.title"))
                .icon(Icon::new(IconName::File).size_4())
                .on_click({
                    let this_panel = this_panel.clone();
                    move |_ev, window, cx| {
                        this_panel.update(cx, |this, cx| {
                            this.open_tool("markdown_editor", window, cx);
                        });
                    }
                }),
            // 凭据管理
            SidebarMenuItem::new("凭据管理")
                .icon(Icon::new(IconName::Settings).size_4())
                .on_click({
                    let this_panel = this_panel.clone();
                    move |_ev, window, cx| {
                        this_panel.update(cx, |this, cx| {
                            this.open_tool("credential_manager", window, cx);
                        });
                    }
                }),
            // 工具箱（带子菜单）
            toolbox_menu,
        ];

        div()
            .id("left-panel")
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .bg(cx.theme().colors.background)
            .child(
                // 使用 Sidebar 组件
                Sidebar::new("main-sidebar")
                    .collapsed(collapsed)
                    .children(main_menu_items)
            )
    }
}
