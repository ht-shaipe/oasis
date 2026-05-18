use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants as _};
use gpui_component::dock::{Panel, PanelEvent};
use gpui_component::h_flex;
use gpui_component::label::Label;
use gpui_component::setting::{
    NumberFieldOptions, SettingField, SettingGroup, SettingItem, SettingPage,
};
use gpui_component::v_flex;
use gpui_component::ActiveTheme;
use gpui_component::{Icon, IconName, Sizable, Theme, ThemeMode};
use rust_i18n::t;

use crate::app_menus;
use crate::app_state::{AppSettings, tool_id_from_tab};
use crate::core::updater::{UpdateCheckResult, UpdateManager, Version};
use crate::panels::toolbox_panel::ToolboxPanel;
use crate::panels::credential_manager::CredentialManagerPanel;
use crate::panels::code_editor::CodeEditorPanel;
use crate::panels::markdown_editor::MarkdownEditorPanel;

const TAB_WORKBENCH: usize = 0;
const TAB_CONFIG: usize = 1;
const TAB_LOG: usize = 2;
const TAB_MONITOR: usize = 3;
const TOOL_TAB_ID_BASE: usize = 1000;

/// 检查是否为工具标签
fn is_tool_tab(tab_id: usize) -> bool {
    tab_id >= TOOL_TAB_ID_BASE
}

/// Update check status
#[derive(Debug, Clone, Default)]
pub enum UpdateStatus {
    #[default]
    Idle,
    Checking,
    NoUpdate,
    Available {
        version: String,
        notes: String,
    },
    Error(String),
}

pub struct CenterPanel {
    focus_handle: FocusHandle,
    update_manager: UpdateManager,
    update_status: UpdateStatus,
    open_tabs: Vec<usize>,  // Track which tabs are open
    tool_content: Entity<ToolboxPanel>, // Tool content rendered in center
    credential_content: Entity<CredentialManagerPanel>, // Credential manager panel
    code_editor_content: Entity<CodeEditorPanel>, // Code editor panel
    markdown_editor_content: Entity<MarkdownEditorPanel>, // Markdown editor panel
}

impl CenterPanel {
    pub fn new(_window: &mut Window, cx: &mut App) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            update_manager: UpdateManager::default(),
            update_status: UpdateStatus::default(),
            open_tabs: vec![TAB_WORKBENCH, TAB_CONFIG, TAB_LOG, TAB_MONITOR],  // All tabs open by default
            tool_content: cx.new(|cx| ToolboxPanel::new(_window, cx)),
            credential_content: cx.new(|cx| CredentialManagerPanel::new(_window, cx)),
            code_editor_content: CodeEditorPanel::view(_window, cx),
            markdown_editor_content: MarkdownEditorPanel::view(_window, cx),
        }
    }

    /// Close a tab
    fn close_tab(&mut self, tab_idx: usize, cx: &mut Context<Self>) {
        // 如果是工具标签，直接关闭
        if is_tool_tab(tab_idx) {
            AppSettings::global_mut(cx).close_tool_tab(tab_idx);
            return;
        }

        // 静态标签：不要关闭最后一个标签
        if self.open_tabs.len() <= 1 {
            return;
        }
        // 移除标签
        self.open_tabs.retain(|&t| t != tab_idx);
        // 如果关闭的是当前选中的标签，切换到第一个可用标签
        let settings = AppSettings::global(cx);
        let is_selected = if tab_idx == TAB_CONFIG {
            settings.show_settings
        } else {
            false
        };
        // 检查当前标签是否仍然打开
        let current_tab_open = self.open_tabs.contains(&tab_idx);
        if is_selected || !current_tab_open {
            // 切换到工作台（始终可用）
            AppSettings::global_mut(cx).show_settings = false;
        }
    }

    fn get_selected_tab(&self, cx: &App) -> usize {
        let settings = AppSettings::global(cx);

        // 工具标签优先级最高
        if let Some(tool_tab_id) = settings.active_tool_tab_id {
            return tool_tab_id;
        }

        // 检查是否显示配置标签
        if settings.show_settings {
            TAB_CONFIG
        } else {
            TAB_WORKBENCH
        }
    }

    fn render_tab_bar(&self, selected_tab: usize, cx: &mut Context<Self>) -> impl IntoElement {
        let show_left = AppSettings::global(cx).show_left_panel;
        let show_right = AppSettings::global(cx).show_right_panel;
        let show_bottom = AppSettings::global(cx).show_bottom_panel;
        let theme = cx.theme();
        let open_tabs = &self.open_tabs;
        let entity = cx.entity().clone();
        let settings = AppSettings::global(cx);
        let tool_tabs = settings.get_all_tool_tabs();

        // 静态标签定义
        let static_tab_defs = [
            (TAB_WORKBENCH, t!("tab.workbench").to_string(), IconName::LayoutDashboard),
            (TAB_CONFIG, t!("tab.config").to_string(), IconName::Settings),
            (TAB_LOG, t!("tab.log").to_string(), IconName::SquareTerminal),
            (TAB_MONITOR, t!("tab.monitor").to_string(), IconName::ChartPie),
        ];

        // 构建混合标签列表
        let mut all_tabs: Vec<(usize, String, IconName)> = static_tab_defs
            .iter()
            .filter(|(id, _, _)| open_tabs.contains(id))
            .map(|(id, label, icon)| (*id, label.clone(), icon.clone()))
            .collect();

        // 找到工作台标签的插入位置
        let workbench_idx = all_tabs
            .iter()
            .position(|(id, _, _)| *id == TAB_WORKBENCH)
            .unwrap_or(0);

        // 插入工具标签到工作台之后
        for tool_tab in tool_tabs {
            let tab_entry = (tool_tab.id, tool_tab.title.to_string(), tool_tab.icon.clone());
            all_tabs.insert(workbench_idx + 1, tab_entry);
        }

        h_flex()
            .id("tab-bar")
            .h(px(40.))
            .w_full()
            .bg(theme.colors.tab_bar)
            .border_b(px(1.0))
            .border_color(theme.colors.border)
            .items_center()
            // 左侧面板切换按钮
            .child(
                Button::new("toggle-left")
                    .ghost()
                    .icon(if show_left {
                        IconName::PanelLeftClose
                    } else {
                        IconName::PanelLeftOpen
                    })
                    .on_click(|_ev, _window: &mut Window, cx: &mut App| {
                        AppSettings::global_mut(cx).show_left_panel =
                            !AppSettings::global(cx).show_left_panel;
                    }),
            )
            // 标签区域
            .children(all_tabs.into_iter().map(|(tab_idx, tab_label, tab_icon)| {
                let is_selected = selected_tab == tab_idx;
                let is_tool = is_tool_tab(tab_idx);
                let tab_bg = theme.colors.tab_active;
                let tab_bar_bg = theme.colors.tab_bar;
                let primary = theme.colors.primary;
                let foreground = theme.colors.foreground;
                let muted_fg = theme.colors.muted_foreground;
                let entity_clone = entity.clone();

                div()
                    .id(tab_idx)
                    .px(px(12.))
                    .h_full()
                    .flex()
                    .items_center()
                    .gap(px(6.))
                    .cursor_pointer()
                    .when(is_selected, |this| {
                        this.bg(tab_bg).border_b(px(2.0)).border_color(primary)
                    })
                    .when(!is_selected, |this| {
                        this.border_b(px(2.0)).border_color(tab_bar_bg)
                    })
                    .child(
                        Icon::new(tab_icon)
                            .text_size(px(14.))
                            .text_color(if is_selected { foreground } else { muted_fg }),
                    )
                    .child(
                        div()
                            .text_color(if is_selected { foreground } else { muted_fg })
                            .text_size(px(13.))
                            .font_weight(if is_selected {
                                FontWeight::BOLD
                            } else {
                                FontWeight::NORMAL
                            })
                            .child(tab_label),
                    )
                    // 关闭按钮：工具标签始终显示，静态标签仅在 hover 时显示
                    .when(is_tool || self.open_tabs.len() > 1, |this| {
                        let tab_to_close = tab_idx;
                        let entity = entity_clone.clone();
                        this.child(
                            div()
                                .when(!is_tool, |div| {
                                    div.opacity(0.0).hover(|style| style.opacity(1.0))
                                })
                                .child(
                                    Button::new(("close-tab", tab_idx))
                                        .ghost()
                                        .icon(IconName::Close)
                                        .on_click(move |_ev, window: &mut Window, cx: &mut App| {
                                            cx.stop_propagation();
                                            entity.update(cx, |this, cx| {
                                                this.close_tab(tab_to_close, cx);
                                            });
                                            window.refresh();
                                        })
                                )
                        )
                    })
                    .on_click(move |_ev, window: &mut Window, cx: &mut App| {
                        if is_tool_tab(tab_idx) {
                            // 工具标签：切换到该工具
                            AppSettings::global_mut(cx).set_active_tool_tab(tab_idx);
                        } else if tab_idx == TAB_CONFIG {
                            AppSettings::global_mut(cx).show_settings = true;
                        } else {
                            AppSettings::global_mut(cx).show_settings = false;
                        }
                        window.refresh();
                    })
            }))
            // 间隔
            .child(div().flex_1())
            // 右侧面板切换按钮
            .child(
                Button::new("toggle-right")
                    .ghost()
                    .icon(if show_right {
                        IconName::PanelRightClose
                    } else {
                        IconName::PanelRightOpen
                    })
                    .on_click(|_ev, _window: &mut Window, cx: &mut App| {
                        AppSettings::global_mut(cx).show_right_panel =
                            !AppSettings::global(cx).show_right_panel;
                    }),
            )
            // 底部面板切换按钮
            .child(
                Button::new("toggle-bottom")
                    .ghost()
                    .icon(if show_bottom {
                        IconName::PanelBottom
                    } else {
                        IconName::PanelBottomOpen
                    })
                    .on_click(|_ev, _window: &mut Window, cx: &mut App| {
                        AppSettings::global_mut(cx).show_bottom_panel =
                            !AppSettings::global(cx).show_bottom_panel;
                    }),
            )
    }

    fn render_tool_content(&self, tab_id: usize, cx: &mut Context<Self>) -> AnyElement {
        let tool_id = tool_id_from_tab(tab_id).or_else(|| {
            AppSettings::global(cx).tool_tabs.get(&tab_id).map(|t| t.tool_id.as_str())
        });

        if let Some(tool_id_str) = tool_id {
            // Handle credential_manager separately (it has its own panel)
            if tool_id_str == "credential_manager" {
                return self.credential_content.clone().into_any_element();
            }

            // Handle code_editor separately (it has its own panel)
            if tool_id_str == "code_editor" {
                return self.code_editor_content.clone().into_any_element();
            }

            // Handle markdown_editor separately (it has its own panel)
            if tool_id_str == "markdown_editor" {
                return self.markdown_editor_content.clone().into_any_element();
            }

            use crate::panels::toolbox_panel::ViewState;
            let view = match tool_id_str {
                "csv_stats" => ViewState::Tool(crate::panels::toolbox_panel::ToolId::CsvStats),
                "csv_split" => ViewState::Tool(crate::panels::toolbox_panel::ToolId::CsvSplit),
                "csv_convert" => ViewState::Tool(crate::panels::toolbox_panel::ToolId::CsvExcelConvert),
                "batch_rename" => ViewState::Tool(crate::panels::toolbox_panel::ToolId::BatchRename),
                "excel_move" => ViewState::Tool(crate::panels::toolbox_panel::ToolId::ExcelMoveFiles),
                "api_request" => ViewState::Tool(crate::panels::toolbox_panel::ToolId::ApiRequest),
                "api_batch_download" => ViewState::Tool(crate::panels::toolbox_panel::ToolId::ApiBatchDownload),
                "json_convert" => ViewState::Tool(crate::panels::toolbox_panel::ToolId::JsonToCsvExcel),
                "json_merge" => ViewState::Tool(crate::panels::toolbox_panel::ToolId::JsonMerge),
                "network_scan" => ViewState::Tool(crate::panels::toolbox_panel::ToolId::NetworkScan),
                _ => ViewState::Home,
            };

            self.tool_content.update(cx, |tp, cx| {
                tp.view = view;
                cx.notify();
            });

            self.tool_content.clone().into_any_element()
        } else {
            self.render_workbench_content(cx).into_any_element()
        }
    }

    fn render_workbench_content(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        // 只有在没有工具标签打开时显示欢迎界面
        let has_tool_tabs = !AppSettings::global(cx).tool_tabs.is_empty();

        div()
            .id("workbench-content")
            .flex()
            .flex_1()
            .items_center()
            .justify_center()
            .bg(theme.colors.background)
            .children(if !has_tool_tabs {
                vec![
                    div()
                        .text_color(theme.colors.muted_foreground)
                        .text_size(px(24.))
                        .child(t!("tab.workbench").to_string())
                        .into_any_element()
                ]
            } else {
                vec![]
            })
    }

    fn render_config_content(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let default_settings = AppSettings::default();

        let settings_page = SettingPage::new(t!("config.page.title").to_string())
            .resettable(true)
            .default_open(true)
            .groups(vec![
                SettingGroup::new()
                    .title(t!("config.group.appearance").to_string())
                    .items(vec![
                        SettingItem::new(
                            t!("config.appearance.dark_mode.label").to_string(),
                            SettingField::switch(
                                |cx: &App| cx.theme().mode.is_dark(),
                                |val: bool, cx: &mut App| {
                                    let mode = if val {
                                        ThemeMode::Dark
                                    } else {
                                        ThemeMode::Light
                                    };
                                    Theme::global_mut(cx).mode = mode;
                                    Theme::change(mode, None, cx);
                                    crate::app::themes::save_state(cx);
                                },
                            )
                            .default_value(false),
                        )
                        .description(t!("config.appearance.dark_mode.description").to_string()),
                        SettingItem::new(
                            t!("config.appearance.language.label").to_string(),
                            SettingField::dropdown(
                                vec![
                                    ("zh-CN".into(), t!("lang.zh_cn").into()),
                                    ("en".into(), t!("lang.en").into()),
                                ],
                                |cx: &App| AppSettings::global(cx).locale.clone(),
                                |val: SharedString, cx: &mut App| {
                                    AppSettings::global_mut(cx).locale = val.clone();
                                    rust_i18n::set_locale(val.as_ref());
                                    app_menus::refresh(cx);
                                    cx.refresh_windows();
                                    crate::app::themes::save_state(cx);
                                },
                            )
                            .default_value(default_settings.locale),
                        )
                        .description(t!("config.appearance.language.description").to_string()),
                    ]),
                SettingGroup::new()
                    .title(t!("config.group.font").to_string())
                    .items(vec![
                        SettingItem::new(
                            t!("config.font.label").to_string(),
                            SettingField::dropdown(
                                vec![
                                    ("Arial".into(), "Arial".into()),
                                    ("Helvetica".into(), "Helvetica".into()),
                                    ("Times New Roman".into(), "Times New Roman".into()),
                                    ("Courier New".into(), "Courier New".into()),
                                ],
                                |cx: &App| AppSettings::global(cx).font_family.clone(),
                                |val: SharedString, cx: &mut App| {
                                    AppSettings::global_mut(cx).font_family = val;
                                    crate::app::themes::save_state(cx);
                                },
                            )
                            .default_value(default_settings.font_family),
                        )
                        .description(t!("config.font.description").to_string()),
                        SettingItem::new(
                            t!("config.font.size.label").to_string(),
                            SettingField::number_input(
                                NumberFieldOptions {
                                    min: 10.0,
                                    max: 32.0,
                                    step: 1.0,
                                    ..Default::default()
                                },
                                |cx: &App| AppSettings::global(cx).font_size,
                                |val: f64, cx: &mut App| {
                                    AppSettings::global_mut(cx).font_size = val;
                                    crate::app::themes::save_state(cx);
                                },
                            )
                            .default_value(default_settings.font_size),
                        )
                        .description(t!("config.font.size.description").to_string()),
                        SettingItem::new(
                            t!("config.font.line_height.label").to_string(),
                            SettingField::number_input(
                                NumberFieldOptions {
                                    min: 8.0,
                                    max: 32.0,
                                    step: 1.0,
                                    ..Default::default()
                                },
                                |cx: &App| AppSettings::global(cx).line_height,
                                |val: f64, cx: &mut App| {
                                    AppSettings::global_mut(cx).line_height = val;
                                    crate::app::themes::save_state(cx);
                                },
                            )
                            .default_value(default_settings.line_height),
                        )
                        .description(t!("config.font.line_height.description").to_string()),
                    ]),
                SettingGroup::new()
                    .title(t!("config.group.other").to_string())
                    .items(vec![SettingItem::new(
                        t!("config.auto_switch_theme.label").to_string(),
                        SettingField::checkbox(
                            |cx: &App| AppSettings::global(cx).auto_switch_theme,
                            |val: bool, cx: &mut App| {
                                AppSettings::global_mut(cx).auto_switch_theme = val;
                                crate::app::themes::save_state(cx);
                            },
                        )
                        .default_value(default_settings.auto_switch_theme),
                    )
                    .description(
                        t!("config.auto_switch_theme.description").to_string(),
                    )]),
                // Software Update
                SettingGroup::new()
                    .title(t!("config.group.update").to_string())
                    .items(vec![
                        // Current version & update status
                        SettingItem::render({
                            let current_version = Version::current().to_string();
                            let update_status = self.update_status.clone();
                            move |_options, _window, cx| {
                                v_flex()
                                    .gap_2()
                                    .w_full()
                                    .child(
                                        h_flex()
                                            .gap_2()
                                            .items_center()
                                            .child(
                                                Label::new(
                                                    t!("config.update.current_version.label")
                                                        .to_string(),
                                                )
                                                .text_sm(),
                                            )
                                            .child(
                                                Label::new(&current_version)
                                                    .text_sm()
                                                    .text_color(cx.theme().colors.muted_foreground),
                                            ),
                                    )
                                    .child(match &update_status {
                                        UpdateStatus::Idle | UpdateStatus::NoUpdate => h_flex()
                                            .gap_2()
                                            .items_center()
                                            .child(Icon::new(IconName::Check))
                                            .child(
                                                Label::new(
                                                    t!("config.update.status.up_to_date")
                                                        .to_string(),
                                                )
                                                .text_xs()
                                                .text_color(cx.theme().colors.success_foreground),
                                            ),
                                        UpdateStatus::Checking => h_flex()
                                            .gap_2()
                                            .items_center()
                                            .child(Icon::new(IconName::LoaderCircle))
                                            .child(
                                                Label::new(
                                                    t!("config.update.status.checking")
                                                        .to_string(),
                                                )
                                                .text_xs()
                                                .text_color(cx.theme().colors.muted_foreground),
                                            ),
                                        UpdateStatus::Available { version, notes } => {
                                            let notes_elem = if notes.is_empty() {
                                                None
                                            } else {
                                                Some(
                                                    Label::new(notes)
                                                        .text_xs()
                                                        .text_color(cx.theme().colors.muted_foreground),
                                                )
                                            };
                                            v_flex()
                                                .gap_2()
                                                .w_full()
                                                .child(
                                                    h_flex()
                                                        .gap_2()
                                                        .items_center()
                                                        .child(Icon::new(IconName::ArrowDown))
                                                        .child(
                                                            Label::new(
                                                                t!(
                                                                    "config.update.status.available",
                                                                    version = version
                                                                )
                                                                .to_string(),
                                                            )
                                                            .text_xs()
                                                            .text_color(cx.theme().colors.accent_foreground),
                                                        ),
                                                )
                                                .children(notes_elem)
                                        }
                                        UpdateStatus::Error(err) => div()
                                            .mt_2()
                                            .p(px(8.))
                                            .rounded(px(6.))
                                            .bg(cx.theme().colors.danger.alpha(0.1))
                                            .border(px(1.))
                                            .border_color(cx.theme().colors.danger.alpha(0.3))
                                            .child(
                                                h_flex()
                                                    .gap_2()
                                                    .items_center()
                                                    .child(Icon::new(IconName::CircleX).text_color(cx.theme().colors.danger))
                                                    .child(
                                                        div()
                                                            .text_sm()
                                                            .text_color(cx.theme().colors.danger)
                                                            .child(err.clone())
                                                    )
                                            ),
                                    })
                            }
                        }),
                        // Check for updates button
                        SettingItem::new(
                            t!("config.update.check.label").to_string(),
                            SettingField::render({
                                let view = cx.entity().clone();
                                move |options, _window, _cx| {
                                    Button::new("check-updates")
                                        .icon(IconName::LoaderCircle)
                                        .label(t!("config.update.check.button").to_string())
                                        .outline()
                                        .with_size(options.size)
                                        .on_click({
                                            let view = view.clone();
                                            move |_, window, cx| {
                                                view.update(cx, |this, cx| {
                                                    this.check_for_updates(window, cx);
                                                });
                                            }
                                        })
                                }
                            }),
                        )
                        .description(t!("config.update.check.description").to_string()),
                        // Auto check on startup
                        SettingItem::new(
                            t!("config.update.auto_check.label").to_string(),
                            SettingField::switch(
                                |cx: &App| AppSettings::global(cx).auto_check_on_startup,
                                |val: bool, cx: &mut App| {
                                    AppSettings::global_mut(cx).auto_check_on_startup = val;
                                    crate::app::themes::save_state(cx);
                                },
                            )
                            .default_value(default_settings.auto_check_on_startup),
                        )
                        .description(t!("config.update.auto_check.description").to_string()),
                        // Notifications
                        SettingItem::new(
                            t!("config.update.notifications.label").to_string(),
                            SettingField::switch(
                                |cx: &App| AppSettings::global(cx).notifications_enabled,
                                |val: bool, cx: &mut App| {
                                    AppSettings::global_mut(cx).notifications_enabled = val;
                                    crate::app::themes::save_state(cx);
                                },
                            )
                            .default_value(default_settings.notifications_enabled),
                        )
                        .description(t!("config.update.notifications.description").to_string()),
                        // Auto update
                        SettingItem::new(
                            t!("config.update.auto_update.label").to_string(),
                            SettingField::switch(
                                |cx: &App| AppSettings::global(cx).auto_update,
                                |val: bool, cx: &mut App| {
                                    AppSettings::global_mut(cx).auto_update = val;
                                    crate::app::themes::save_state(cx);
                                },
                            )
                            .default_value(default_settings.auto_update),
                        )
                        .description(t!("config.update.auto_update.description").to_string()),
                        // Check frequency
                        SettingItem::new(
                            t!("config.update.frequency.label").to_string(),
                            SettingField::number_input(
                                NumberFieldOptions {
                                    min: 1.0,
                                    max: 30.0,
                                    step: 1.0,
                                    ..Default::default()
                                },
                                |cx: &App| AppSettings::global(cx).check_frequency_days,
                                |val: f64, cx: &mut App| {
                                    AppSettings::global_mut(cx).check_frequency_days = val;
                                    crate::app::themes::save_state(cx);
                                },
                            )
                            .default_value(default_settings.check_frequency_days),
                        )
                        .description(t!("config.update.frequency.description").to_string()),
                    ]),
            ]);

        let theme = cx.theme();
        div()
            .id("config-content")
            .flex()
            .flex_1()
            .overflow_scroll()
            // .p(px(16.))
            .bg(theme.colors.background)
            .child(gpui_component::setting::Settings::new("settings").page(settings_page))
    }

    /// Check for software updates
    pub fn check_for_updates(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.update_status = UpdateStatus::Checking;
        cx.notify();

        let update_manager = self.update_manager.clone();
        let entity = cx.entity().downgrade();

        cx.spawn(async move |_this, cx| {
            let result = update_manager.check_for_updates().await;

            let _ = cx.update(|cx| {
                let _ = entity.update(cx, |this, cx| {
                    this.update_status = match result {
                        UpdateCheckResult::NoUpdate => UpdateStatus::NoUpdate,
                        UpdateCheckResult::UpdateAvailable(info) => UpdateStatus::Available {
                            version: info.version,
                            notes: info.release_notes,
                        },
                        UpdateCheckResult::Error(err) => UpdateStatus::Error(err),
                    };cx.notify();
                });
            });
        })
        .detach();
    }

    fn render_log_content(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        div()
            .id("log-content")
            .flex()
            .flex_1()
            .items_center()
            .justify_center()
            .bg(theme.colors.background)
            .child(
                div()
                    .text_color(theme.colors.muted_foreground)
                    .text_size(px(24.))
                    .child(t!("tab.log").to_string()),
            )
    }

    fn render_monitor_content(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        div()
            .id("monitor-content")
            .flex()
            .flex_1()
            .items_center()
            .justify_center()
            .bg(theme.colors.background)
            .child(
                div()
                    .text_color(theme.colors.muted_foreground)
                    .text_size(px(24.))
                    .child(t!("tab.monitor").to_string()),
            )
    }
}

impl Panel for CenterPanel {
    fn panel_name(&self) -> &'static str {
        "CenterPanel"
    }

    fn title(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        t!("panel.workspace").to_string()
    }
}

impl EventEmitter<PanelEvent> for CenterPanel {}

impl Focusable for CenterPanel {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for CenterPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // If show_settings is true but config tab is closed, add it back
        if AppSettings::global(cx).show_settings && !self.open_tabs.contains(&TAB_CONFIG) {
            self.open_tabs.push(TAB_CONFIG);
        }

        let selected_tab = self.get_selected_tab(cx);

        div()
            .id("center-panel")
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .child(self.render_tab_bar(selected_tab, cx))
            .child(if is_tool_tab(selected_tab) {
                self.render_tool_content(selected_tab, cx)
            } else if selected_tab == TAB_CONFIG {
                self.render_config_content(cx).into_any_element()
            } else if selected_tab == TAB_LOG {
                self.render_log_content(cx).into_any_element()
            } else if selected_tab == TAB_MONITOR {
                self.render_monitor_content(cx).into_any_element()
            } else {
                self.render_workbench_content(cx).into_any_element()
            })
    }
}
