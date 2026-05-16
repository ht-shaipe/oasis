//! 批量下载小工具：
//! - 配置下载目录
//! - 配置 URL 模板（支持 `{0}` 占位）
//! - 输入一批待替换路径（每行一个）
//! - 按并发方式下载 PDF

use std::{
    io::Read,
    path::Path,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    time::Duration,
};

use crate::panels::browser_panel::{
    WebViewLogMessage, create_webview_entity, register_webview_download,
};
use crate::panels::toolbox_panel::ToolboxPanel;
use crate::utils;
use futures::StreamExt as _;
use gpui::{
    AppContext as _, Context, Entity, IntoElement, ParentElement as _, SharedString, Styled,
    Subscription, Window, prelude::FluentBuilder as _, px,
};
use gpui_component::{
    ActiveTheme, Disableable, Icon, IconName,
    button::{Button, ButtonVariants as _},
    h_flex,
    input::{Input, NumberInput, NumberInputEvent, StepAction},
    label::Label,
    scroll::ScrollableElement,
    v_flex,
};
use gpui_wry::WebView;
use reqwest::header::{
    ACCEPT, ACCEPT_LANGUAGE, CACHE_CONTROL, HeaderMap, HeaderValue, PRAGMA, REFERER,
};

const WEBVIEW_HOME_URL: &str = "about:blank";
const WILEY_HOSTS: [&str; 2] = [
    "onlinelibrary.wiley.com",
    "advanced.onlinelibrary.wiley.com",
];

#[derive(Clone, Debug)]
pub struct BatchDownloadItem {
    pub source_path: String,
    pub file_name: String,
    pub url: String,
    pub save_path: String,
    pub status: String,
    pub downloaded_bytes: u64,
    pub total_bytes: Option<u64>,
    pub progress_percent: f32,
}

#[derive(Debug)]
enum DownloadProgressEvent {
    Started {
        index: usize,
        total_bytes: Option<u64>,
    },
    Progress {
        index: usize,
        downloaded_bytes: u64,
        total_bytes: Option<u64>,
    },
    Finished {
        index: usize,
    },
    Failed {
        index: usize,
        error: String,
    },
}

struct BatchDownloadWebViewSlot {
    id: usize,
    entity: Option<Entity<WebView>>,
    status: String,
    target_url: String,
    current_url: String,
    current_index: Option<usize>,
    waiting_for_user: bool,
    manual_paused: bool,
    request_in_flight: bool,
}

/// 批量下载工具的状态
pub struct BatchDownloadState {
    /// URL 模板，如 https://advanced.onlinelibrary.wiley.com/doi/pdfdirect/{0}
    pub template_input: Entity<gpui_component::input::InputState>,
    /// 多行输入：每行一个待替换路径，如 DOI
    pub paths_input: Entity<gpui_component::input::InputState>,
    /// 并发数输入
    pub concurrency_input: Entity<gpui_component::input::InputState>,
    /// 下载目录
    pub output_dir: Option<PathBuf>,
    /// 是否正在下载
    pub running: bool,
    /// 状态文本
    pub status: String,
    /// WebView 状态文本
    pub webview_status: String,
    /// 下载项列表
    pub items: Vec<BatchDownloadItem>,
    /// 成功数量
    pub success_count: usize,
    /// 失败数量
    pub failed_count: usize,
    /// 是否使用 WebView 会话下载
    pub use_webview_mode: bool,
    /// 是否显示专门的 WebView 执行界面
    pub show_webview_dashboard: bool,
    /// 内嵌 WebView 池
    webview_slots: Vec<BatchDownloadWebViewSlot>,
    /// 停止标记
    pub stop_flag: Arc<AtomicBool>,
    pub _subscriptions: Vec<Subscription>,
}

impl BatchDownloadState {
    pub fn new(window: &mut Window, cx: &mut Context<ToolboxPanel>) -> Self {
        let template_input = cx.new(|cx| {
            let mut s = gpui_component::input::InputState::new(window, cx);
            s.set_placeholder(
                "https://onlinelibrary.wiley.com/doi/pdfdirect/{0}".to_string(),
                window,
                cx,
            );
            s.set_value(
                gpui::SharedString::from("https://onlinelibrary.wiley.com/doi/pdfdirect/{0}"),
                window,
                cx,
            );
            s
        });

        let paths_input = cx.new(|cx| {
            gpui_component::input::InputState::new(window, cx)
                .multi_line(true)
                .rows(16)
        });

        let concurrency_input = cx.new(|cx| {
            let mut s = gpui_component::input::InputState::new(window, cx);
            s.set_placeholder("3".to_string(), window, cx);
            s.set_value(gpui::SharedString::from("3"), window, cx);
            s
        });

        let subscriptions = vec![cx.subscribe_in(&concurrency_input, window, {
            move |_, input, event: &NumberInputEvent, window, cx| match event {
                NumberInputEvent::Step(action) => input.update(cx, |input, cx| {
                    let current = input.value().to_string().trim().parse::<i64>().unwrap_or(3);
                    let next = match action {
                        StepAction::Increment => current + 1,
                        StepAction::Decrement => current - 1,
                    }
                    .clamp(1, 32);
                    input.set_value(SharedString::from(next.to_string()), window, cx);
                }),
            }
        })];
        let mut state = Self {
            template_input,
            paths_input,
            concurrency_input,
            output_dir: None,
            running: false,
            status: String::new(),
            webview_status: "WebView 待命中".to_string(),
            items: Vec::new(),
            success_count: 0,
            failed_count: 0,
            use_webview_mode: false,
            show_webview_dashboard: false,
            webview_slots: Vec::new(),
            stop_flag: Arc::new(AtomicBool::new(false)),
            _subscriptions: subscriptions,
        };
        state.sync_webview_pool(window, cx, 3);
        state
    }

    pub fn render(
        state: &mut BatchDownloadState,
        entity: Entity<ToolboxPanel>,
        window: &mut Window,
        cx: &mut Context<ToolboxPanel>,
    ) -> impl IntoElement {
        let desired_webviews = state.current_concurrency(cx);
        state.sync_webview_pool(window, cx, desired_webviews);
        let theme = cx.theme();

        let template_input = state.template_input.clone();
        let paths_input = state.paths_input.clone();
        let concurrency_input = state.concurrency_input.clone();

        let run_entity = entity.clone();
        let stop_entity = entity.clone();
        let stop_dashboard_entity = entity.clone();
        let pick_dir_entity = entity.clone();
        let open_webview_entity = entity.clone();
        let continue_webview_entity = entity.clone();
        let reload_webview_entity = entity.clone();

        let top_bar = h_flex()
            .gap_3()
            .items_center()
            .w_full()
            .child(
                Button::new("batch-download-run")
                    .label("开始批量下载".to_string())
                    .icon(Icon::new(IconName::ArrowRight).text_color(theme.primary))
                    .primary()
                    .disabled(state.running)
                    .on_click(move |_, _, cx| {
                        run_entity.update(cx, |this, cx| {
                            this.api_batch_download.start_batch_download(cx);
                        });
                    }),
            )
            .child(
                Button::new("batch-download-stop")
                    .label("停止".to_string())
                    .icon(Icon::new(IconName::CircleX).text_color(theme.red))
                    .outline()
                    .disabled(!state.running)
                    .on_click(move |_, _, cx| {
                        stop_entity.update(cx, |this, cx| {
                            this.api_batch_download.stop_batch_download(cx);
                        });
                    }),
            );

        let execution_top_bar = h_flex()
            .gap_3()
            .items_center()
            .w_full()
            .child(
                Button::new("batch-download-stop-dashboard")
                    .label("停止".to_string())
                    .icon(Icon::new(IconName::CircleX).text_color(theme.red))
                    .outline()
                    .disabled(!state.running)
                    .on_click(move |_, _, cx| {
                        stop_dashboard_entity.update(cx, |this, cx| {
                            this.api_batch_download.stop_batch_download(cx);
                        });
                    }),
            )
            .child(
                Button::new("batch-download-open-webview-dashboard")
                    .label("打开当前下载地址".to_string())
                    .outline()
                    .on_click(move |_, _, cx| {
                        open_webview_entity.update(cx, |this, cx| {
                            this.api_batch_download.open_current_webview_target(cx);
                        });
                    }),
            )
            .child(
                Button::new("batch-download-continue-dashboard")
                    .label("继续 WebView 下载".to_string())
                    .primary()
                    .disabled(
                        !state.use_webview_mode
                            || !state.running
                            || !state
                                .webview_slots
                                .iter()
                                .any(|slot| slot.current_index.is_some()),
                    )
                    .on_click(move |_, _, cx| {
                        continue_webview_entity.update(cx, |this, cx| {
                            this.api_batch_download.continue_webview_download(cx);
                        });
                    }),
            )
            .child(
                Button::new("batch-download-reload-dashboard")
                    .label("刷新 WebView".to_string())
                    .outline()
                    .on_click(move |_, _, cx| {
                        reload_webview_entity.update(cx, |this, cx| {
                            this.api_batch_download.reload_webview(cx);
                        });
                    }),
            );

        let template_row = v_flex()
            .gap_2()
            .child(
                Label::new("下载地址模板（使用 {0} 作为占位）".to_string())
                    .text_sm()
                    .text_color(theme.muted_foreground),
            )
            .child(
                gpui::div()
                    .w_full()
                    .child(Input::new(&template_input).w_full()),
            )
            .child(
                Label::new("示例: https://onlinelibrary.wiley.com/doi/pdfdirect/{0}".to_string())
                    .text_xs()
                    .text_color(theme.muted_foreground),
            );

        let output_dir_label = state
            .output_dir
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "未选择下载目录".to_string());

        let output_dir_row = h_flex().gap_3().items_center().w_full().child(
            Button::new("batch-download-pick-output-dir")
                .label("选择下载目录".to_string())
                .icon(Icon::new(IconName::Folder).text_color(theme.blue))
                .outline()
                .disabled(state.running)
                .on_click(move |_, _, cx| {
                    let entity = pick_dir_entity.downgrade();
                    cx.spawn(async move |cx| {
                        let title = "选择批量下载目录".to_string();
                        let path = utils::pick_folder(&title).await;
                        let _ = cx.update(|cx| {
                            if let Some(ent) = entity.upgrade() {
                                ent.update(cx, |this, cx| {
                                    this.api_batch_download.output_dir = path;
                                    cx.notify();
                                });
                            }
                        });
                    })
                    .detach();
                }),
        );

        let paths_area = v_flex()
            .gap_2()
            .child(
                Label::new("待替换路径列表（每行一个）".to_string())
                    .text_sm()
                    .text_color(theme.muted_foreground),
            )
            .child(
                Label::new("示例: 10.1002/smsc.202000067".to_string())
                    .text_xs()
                    .text_color(theme.muted_foreground),
            )
            .child(
                gpui::div()
                    .w_full()
                    .min_h(px(280.))
                    .border_1()
                    .border_color(theme.border)
                    .rounded_md()
                    .child(
                        v_flex()
                            .px_2()
                            .py_1()
                            .child(Input::new(&paths_input).w_full().h(px(260.))),
                    ),
            );

        let concurrency_row = h_flex()
            .gap_2()
            .items_center()
            .child(
                Label::new("最大并发数".to_string())
                    .text_sm()
                    .text_color(theme.muted_foreground),
            )
            .child(
                gpui::div()
                    .w(px(160.))
                    .child(NumberInput::new(&concurrency_input).disabled(state.running)),
            )
            .child(
                Label::new("（建议 1–10，默认 3）".to_string())
                    .text_xs()
                    .text_color(theme.muted_foreground),
            );

        let status = state.status.clone();
        let status_row = if !status.is_empty() {
            h_flex()
                .gap_2()
                .items_center()
                .pt_2()
                .child(
                    Icon::new(IconName::Info)
                        .text_color(theme.muted_foreground)
                        .size_12(),
                )
                .child(
                    Label::new(status)
                        .text_sm()
                        .text_color(theme.muted_foreground),
                )
        } else {
            h_flex().gap_0()
        };

        let settings_left_column = v_flex()
            .gap_3()
            .w_full()
            .rounded_lg()
            .border_1()
            .border_color(theme.border)
            .bg(theme.background)
            .p_4()
            .child(top_bar)
            .child(template_row)
            .child(
                output_dir_row.child(
                    Label::new(output_dir_label)
                        .text_sm()
                        .text_color(theme.muted_foreground)
                        .overflow_hidden()
                        .whitespace_nowrap()
                        .truncate()
                        .flex_1(),
                ),
            )
            .child(concurrency_row)
            .child(status_row);

        let settings_right_column = v_flex()
            .gap_3()
            .w_full()
            .rounded_lg()
            .border_1()
            .border_color(theme.border)
            .bg(theme.background)
            .p_4()
            .child(paths_area)
            .child(
                Label::new(format!(
                    "配置完成后，开始下载会按当前线程数创建对应数量的 WebView 执行窗口。当前将创建 {} 个。",
                    state.webview_slots.len()
                ))
                .text_xs()
                .text_color(theme.muted_foreground),
            );

        let webview_columns = if state.webview_slots.len() > 4 {
            3
        } else if state.webview_slots.len() > 1 {
            2
        } else {
            1
        };

        let webview_grid =
            v_flex()
                .gap_3()
                .children(state.webview_slots.chunks(webview_columns).map(|row| {
                    h_flex()
                        .w_full()
                        .gap_3()
                        .items_start()
                        .children(row.iter().map(|slot| {
                            let slot_id = slot.id;
                            let slot_entity_start = entity.clone();
                            let slot_entity_stop = entity.clone();
                            let slot_is_active =
                                slot.current_index.is_some() || slot.waiting_for_user;
                            let slot_is_paused = slot.manual_paused;
                            let webview_area = gpui::div()
                                .w_full()
                                .min_h(px(220.))
                                .max_h(px(260.))
                                .overflow_hidden()
                                .rounded_md()
                                .border_1()
                                .border_color(theme.border);

                            let slot_header = v_flex()
                                .gap_2()
                                .child(
                                    h_flex()
                                        .justify_between()
                                        .items_center()
                                        .gap_2()
                                        .child(
                                            Label::new(format!("WebView #{}", slot.id + 1))
                                                .text_sm()
                                                .text_color(theme.foreground),
                                        )
                                        .child(
                                            h_flex()
                                                .gap_2()
                                                .items_center()
                                                .child({
                                                    let slot_start_id = SharedString::from(format!("batch-download-slot-start-{}", slot_id));
                                                    Button::new(slot_start_id)
                                                    .label("开始".to_string())
                                                    .primary()
                                                    .disabled(
                                                        !state.running
                                                            || !state.use_webview_mode
                                                            || (!slot_is_paused && slot_is_active),
                                                    )
                                                    .on_click(move |_, _, cx| {
                                                        slot_entity_start.update(cx, |this, cx| {
                                                            this.api_batch_download
                                                                .resume_webview_slot(slot_id, cx);
                                                        });
                                                    })
                                                })
                                                .child({
                                                    let slot_stop_id = SharedString::from(format!("batch-download-slot-stop-{}", slot_id));
                                                    Button::new(slot_stop_id)
                                                    .label("停止".to_string())
                                                    .outline()
                                                    .disabled(
                                                        !state.running
                                                            || !state.use_webview_mode
                                                            || slot_is_paused
                                                            || !slot_is_active,
                                                    )
                                                    .on_click(move |_, _, cx| {
                                                        slot_entity_stop.update(cx, |this, cx| {
                                                            this.api_batch_download
                                                                .pause_webview_slot(slot_id, cx);
                                                        });
                                                    })
                                                })
                                        ),
                                )
                                .child(
                                    Label::new(slot.status.clone())
                                        .text_xs()
                                        .text_color(theme.muted_foreground),
                                )
                                .child(
                                    Label::new(format!("任务地址: {}", slot.target_url))
                                        .text_xs()
                                        .text_color(theme.muted_foreground)
                                        .overflow_hidden()
                                        .whitespace_nowrap()
                                        .truncate(),
                                )
                                .child(
                                    Label::new(format!("当前页面: {}", slot.current_url))
                                        .text_xs()
                                        .text_color(theme.muted_foreground)
                                        .overflow_hidden()
                                        .whitespace_nowrap()
                                        .truncate(),
                                );

                            let slot_body = if let Some(ref webview_entity) = slot.entity {
                                webview_area
                                    .child(webview_entity.clone())
                                    .into_any_element()
                            } else {
                                webview_area
                                    .child(
                                        v_flex().size_full().items_center().justify_center().child(
                                            Label::new("当前平台未启用 WebView".to_string())
                                                .text_sm()
                                                .text_color(theme.muted_foreground),
                                        ),
                                    )
                                    .into_any_element()
                            };

                            let card_width = if webview_columns == 3 {
                                px(320.)
                            } else if webview_columns == 2 {
                                px(500.)
                            } else {
                                px(9999.)
                            };

                            gpui::div()
                                .w(card_width)
                                .min_w(px(280.))
                                .flex_1()
                                .child(v_flex().gap_2().child(slot_header).child(slot_body))
                        }))
                }));

        let webview_section = v_flex()
            .gap_3()
            .w_full()
            .rounded_lg()
            .border_1()
            .border_color(theme.border)
            .p_4()
            .child(execution_top_bar)
            .child(
                Label::new("Wiley 会话下载窗口".to_string())
                    .text_sm()
                    .text_color(theme.muted_foreground),
            )
            .child(
                Label::new(state.webview_status.clone())
                    .text_xs()
                    .text_color(theme.muted_foreground),
            )
            .child(
                Label::new(format!(
                    "当前 WebView 数量: {}，当前布局: 一行 {} 个",
                    state.webview_slots.len(),
                    webview_columns
                ))
                .text_xs()
                .text_color(theme.muted_foreground),
            )
            .child(webview_grid);

        let summary_row = if !state.items.is_empty() {
            h_flex()
                .gap_4()
                .items_center()
                .child(
                    Label::new(format!("总数: {}", state.items.len()))
                        .text_sm()
                        .text_color(theme.muted_foreground),
                )
                .child(
                    Label::new(format!("成功: {}", state.success_count))
                        .text_sm()
                        .text_color(theme.green),
                )
                .child(
                    Label::new(format!("失败: {}", state.failed_count))
                        .text_sm()
                        .text_color(theme.danger),
                )
        } else {
            h_flex().gap_0()
        };

        let list = if !state.items.is_empty() {
            gpui::div()
                .w_full()
                .min_h(px(180.))
                .rounded_lg()
                .border_1()
                .border_color(theme.border)
                .p_3()
                .child(v_flex().gap_2().children(state.items.iter().map(|item| {
                    let percent = format!("{:.0}%", item.progress_percent);
                    let bytes_label = match item.total_bytes {
                        Some(total) => format!("{}/{} bytes", item.downloaded_bytes, total),
                        None => format!("{} bytes", item.downloaded_bytes),
                    };

                    v_flex()
                        .gap_1()
                        .w_full()
                        .rounded_md()
                        .border_1()
                        .border_color(theme.border)
                        .p_2()
                        .child(
                            h_flex()
                                .justify_between()
                                .items_start()
                                .gap_2()
                                .child(
                                    v_flex()
                                        .gap_1()
                                        .flex_1()
                                        .child(
                                            Label::new(item.file_name.clone())
                                                .text_sm()
                                                .font_weight(gpui::FontWeight::MEDIUM)
                                                .overflow_hidden()
                                                .whitespace_nowrap()
                                                .truncate(),
                                        )
                                        .child(
                                            Label::new(format!("保存文件名: {}", item.file_name))
                                                .text_xs()
                                                .text_color(theme.muted_foreground)
                                                .overflow_hidden()
                                                .whitespace_nowrap()
                                                .truncate(),
                                        ),
                                )
                                .child(
                                    Label::new(item.status.clone())
                                        .text_xs()
                                        .text_color(theme.muted_foreground),
                                ),
                        )
                        .child(
                            gpui::div()
                                .w_full()
                                .h(px(8.))
                                .rounded_md()
                                .bg(theme.muted)
                                .child(
                                    gpui::div().h_full().rounded_md().bg(theme.primary).w(px(
                                        (item.progress_percent.clamp(0.0, 100.0) * 2.6) as f32,
                                    )),
                                ),
                        )
                        .child(
                            v_flex()
                                .gap_1()
                                .child(
                                    Label::new(format!("源路径: {}", item.source_path))
                                        .text_xs()
                                        .text_color(theme.muted_foreground)
                                        .overflow_hidden()
                                        .whitespace_nowrap()
                                        .truncate()
                                        .w_full(),
                                )
                                .child(
                                    Label::new(format!("下载地址: {}", item.url))
                                        .text_xs()
                                        .text_color(theme.muted_foreground)
                                        .overflow_hidden()
                                        .whitespace_nowrap()
                                        .truncate()
                                        .w_full(),
                                )
                                .child(
                                    Label::new(format!("保存路径: {}", item.save_path))
                                        .text_xs()
                                        .text_color(theme.muted_foreground)
                                        .overflow_hidden()
                                        .whitespace_nowrap()
                                        .truncate()
                                        .w_full(),
                                )
                                .child(
                                    h_flex()
                                        .justify_between()
                                        .items_center()
                                        .gap_2()
                                        .child(
                                            Label::new(bytes_label)
                                                .text_xs()
                                                .text_color(theme.muted_foreground),
                                        )
                                        .child(
                                            Label::new(percent)
                                                .text_xs()
                                                .text_color(theme.muted_foreground),
                                        ),
                                ),
                        )
                })))
                .into_any_element()
        } else {
            gpui::div().w_full().into_any_element()
        };

        gpui::div()
            .size_full()
            .overflow_y_scrollbar()
            .child(
                v_flex()
                    .w_full()
                    .gap_4()
                    .p_4()
                    .child(
                        v_flex()
                            .gap_1()
                            .child(
                                Label::new("批量下载".to_string())
                                    .text_lg()
                                    .font_weight(gpui::FontWeight::SEMIBOLD),
                            )
                            .child(
                                Label::new(
                                    "上方为下载设置区域，下方为可视化 WebView 下载区域。设置区固定两列，下载区按数量自动分列。"
                                        .to_string(),
                                )
                                .text_sm()
                                .text_color(theme.muted_foreground),
                            ),
                    )
                    .child(
                        h_flex()
                            .w_full()
                            .gap_4()
                            .items_start()
                            .child(gpui::div().flex_1().min_w(px(0.)).child(settings_left_column))
                            .child(gpui::div().flex_1().min_w(px(0.)).child(settings_right_column)),
                    )
                    .child(summary_row)
                    .child(webview_section)
                    .child(list),
            )
    }

    fn make_output_filename(path: &str) -> String {
        let last = path.rsplit('/').next().unwrap_or(path).trim();
        let file_stem = if last.to_ascii_lowercase().ends_with(".pdf") {
            last[..last.len().saturating_sub(4)].to_string()
        } else {
            last.to_string()
        };
        format!("{file_stem}.pdf")
    }

    fn build_items(
        template: &str,
        output_dir: &PathBuf,
        paths: &[String],
    ) -> Vec<BatchDownloadItem> {
        paths
            .iter()
            .map(|path| {
                let source_path = path.trim().to_string();
                let file_name = Self::make_output_filename(&source_path);
                let url = template.replace("{0}", &source_path);
                let save_path = output_dir.join(&file_name).display().to_string();
                BatchDownloadItem {
                    source_path,
                    file_name,
                    url,
                    save_path,
                    status: "等待中".to_string(),
                    downloaded_bytes: 0,
                    total_bytes: None,
                    progress_percent: 0.0,
                }
            })
            .collect()
    }

    fn build_request_headers(item: &BatchDownloadItem) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            ACCEPT,
            HeaderValue::from_static(
                "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8",
            ),
        );
        headers.insert(
            ACCEPT_LANGUAGE,
            HeaderValue::from_static("zh-CN,zh;q=0.9,en;q=0.8"),
        );
        headers.insert(CACHE_CONTROL, HeaderValue::from_static("no-cache"));
        headers.insert(PRAGMA, HeaderValue::from_static("no-cache"));

        if item.url.contains("/doi/pdfdirect/") {
            let referer = item.url.replace("/doi/pdfdirect/", "/doi/");
            if let Ok(value) = HeaderValue::from_str(&referer) {
                headers.insert(REFERER, value);
            }
        }

        headers
    }

    fn url_host(url: &str) -> Option<String> {
        reqwest::Url::parse(url)
            .ok()
            .and_then(|url| url.host_str().map(|host| host.to_string()))
    }

    fn is_wiley_host(host: &str) -> bool {
        WILEY_HOSTS
            .iter()
            .any(|candidate| host == *candidate || host.ends_with(&format!(".{candidate}")))
    }

    fn should_use_webview(items: &[BatchDownloadItem]) -> bool {
        items.iter().any(|item| {
            Self::url_host(&item.url)
                .map(|host| Self::is_wiley_host(&host))
                .unwrap_or(false)
        })
    }

    fn same_host(left: &str, right: &str) -> bool {
        matches!(
            (Self::url_host(left), Self::url_host(right)),
            (Some(left), Some(right))
                if left == right || (Self::is_wiley_host(&left) && Self::is_wiley_host(&right))
        )
    }

    fn ensure_valid_pdf_file(path: &Path) -> Result<(), String> {
        let mut file = std::fs::File::open(path)
            .map_err(|err| format!("打开下载文件失败 {}: {}", path.display(), err))?;
        let mut header = [0_u8; 5];
        let read = file
            .read(&mut header)
            .map_err(|err| format!("读取下载文件失败 {}: {}", path.display(), err))?;

        if read < header.len() || header != *b"%PDF-" {
            let _ = std::fs::remove_file(path);
            return Err(format!("下载结果不是有效 PDF，已删除: {}", path.display()));
        }

        Ok(())
    }

    fn current_concurrency(&self, cx: &mut Context<ToolboxPanel>) -> usize {
        let mut concurrency = 3usize;
        self.concurrency_input.update(cx, |s, _| {
            let text = s.text().to_string();
            if let Ok(v) = text.trim().parse::<usize>() {
                if v > 0 {
                    concurrency = v.min(32);
                }
            }
        });
        concurrency
    }

    fn sync_webview_pool(
        &mut self,
        window: &mut Window,
        cx: &mut Context<ToolboxPanel>,
        desired_count: usize,
    ) {
        let target = desired_count.max(1);
        while self.webview_slots.len() < target {
            let slot_id = self.webview_slots.len();
            self.webview_slots
                .push(Self::create_webview_slot(slot_id, window, cx));
        }
        while self.webview_slots.len() > target {
            self.webview_slots.pop();
        }
    }

    fn create_webview_slot(
        slot_id: usize,
        window: &mut Window,
        cx: &mut Context<ToolboxPanel>,
    ) -> BatchDownloadWebViewSlot {
        let (webview_log_tx, webview_log_rx) = mpsc::channel::<WebViewLogMessage>();
        let entity = create_webview_entity(window, cx, WEBVIEW_HOME_URL, webview_log_tx);
        let weak_entity = cx.entity().downgrade();

        cx.spawn(async move |_entity, cx| {
            loop {
                let mut disconnected = false;
                loop {
                    match webview_log_rx.try_recv() {
                        Ok(msg) => {
                            let Some(raw) = msg.raw else {
                                continue;
                            };
                            let Some(ent) = weak_entity.upgrade() else {
                                disconnected = true;
                                break;
                            };

                            cx.update(|cx| {
                                ent.update(cx, |this, cx| {
                                    let json_value: serde_json::Value = raw.parse().unwrap_or(serde_json::Value::Null);
                                    this.api_batch_download.handle_webview_message(
                                        slot_id,
                                        json_value,
                                        cx,
                                    );
                                });
                            });
                        }
                        Err(mpsc::TryRecvError::Empty) => break,
                        Err(mpsc::TryRecvError::Disconnected) => {
                            disconnected = true;
                            break;
                        }
                    }
                }

                if disconnected {
                    break;
                }

                smol::Timer::after(Duration::from_millis(80)).await;
            }
        })
        .detach();

        BatchDownloadWebViewSlot {
            id: slot_id,
            entity,
            status: "待命".to_string(),
            target_url: WEBVIEW_HOME_URL.to_string(),
            current_url: WEBVIEW_HOME_URL.to_string(),
            current_index: None,
            waiting_for_user: false,
            manual_paused: false,
            request_in_flight: false,
        }
    }

    fn handle_webview_message(
        &mut self,
        slot_id: usize,
        raw: serde_json::Value,
        cx: &mut Context<ToolboxPanel>,
    ) {
        if raw.get("type").and_then(|v| v.as_str()) == Some("nav") {
            let event = raw
                .get("event")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let url = raw
                .get("url")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();

            let should_auto_continue =
                event == "load" && self.should_auto_continue_webview(slot_id, &url);
            if let Some(slot) = self.webview_slots.get_mut(slot_id) {
                slot.current_url = url.clone();
                if event == "load" && self.use_webview_mode {
                    if should_auto_continue {
                        slot.status =
                            format!("槽位 {} 已加载目标页面，正在检查页面有效性", slot_id + 1);
                    } else {
                        slot.status = format!("槽位 {} 已加载: {}", slot_id + 1, url);
                    }
                }
            }
            if should_auto_continue {
                self.webview_status = format!(
                    "WebView #{} 已加载目标页面，准备检查是否为有效 PDF",
                    slot_id + 1
                );
                self.inspect_current_webview_page_for_slot(slot_id, cx);
            }
            return;
        }

        match raw.get("type").and_then(|v| v.as_str()).unwrap_or_default() {
            "download" => {
                let event = raw
                    .get("event")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string();
                let index = raw
                    .get("index")
                    .and_then(|v| v.as_u64())
                    .map(|v| v as usize);
                let save_path = raw
                    .get("savePath")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string();
                let success = raw
                    .get("success")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                match event.as_str() {
                    "started" => {
                        let Some(index) = index else {
                            return;
                        };
                        if let Some(item) = self.items.get_mut(index) {
                            item.status = format!("WebView #{} 原生下载中", slot_id + 1);
                            item.progress_percent = 0.0;
                        }
                        if let Some(slot) = self.webview_slots.get_mut(slot_id) {
                            slot.waiting_for_user = false;
                            slot.current_index = Some(index);
                            slot.request_in_flight = true;
                            slot.status = format!(
                                "槽位 {} 正在下载 {}",
                                slot_id + 1,
                                self.items[index].file_name
                            );
                        }
                        self.webview_status = format!("WebView #{} 已开始原生下载", slot_id + 1);
                    }
                    "completed" => {
                        let Some(index) = index else {
                            return;
                        };
                        if success {
                            let validated_path = if !save_path.is_empty() {
                                save_path.clone()
                            } else {
                                self.items
                                    .get(index)
                                    .map(|item| item.save_path.clone())
                                    .unwrap_or_default()
                            };

                            match Self::ensure_valid_pdf_file(Path::new(&validated_path)) {
                                Ok(()) => {
                                    if let Some(item) = self.items.get_mut(index) {
                                        item.status = "已完成".to_string();
                                        item.progress_percent = 100.0;
                                        if !validated_path.is_empty() {
                                            item.save_path = validated_path.clone();
                                        }
                                    }
                                    self.success_count += 1;
                                    if let Some(slot) = self.webview_slots.get_mut(slot_id) {
                                        slot.status = format!("槽位 {} 下载完成", slot_id + 1);
                                        slot.target_url = WEBVIEW_HOME_URL.to_string();
                                        slot.current_index = None;
                                        slot.waiting_for_user = false;
                                        slot.request_in_flight = false;
                                    }
                                    self.webview_status =
                                        format!("WebView #{} 下载完成", slot_id + 1);
                                    self.try_start_next_webview_item_for_slot(slot_id, cx);
                                }
                                Err(err) => {
                                    self.handle_webview_failure(slot_id, index, err, None, cx);
                                }
                            }
                        } else {
                            self.handle_webview_failure(
                                slot_id,
                                index,
                                "WebView 原生下载未成功完成".to_string(),
                                None,
                                cx,
                            );
                        }
                    }
                    _ => {}
                }
            }
            "batch_download" => {
                let event = raw
                    .get("event")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string();
                let index = raw
                    .get("index")
                    .and_then(|v| v.as_u64())
                    .map(|v| v as usize);
                if event == "requested" {
                    let Some(index) = index else {
                        return;
                    };
                    if let Some(item) = self.items.get_mut(index) {
                        item.status = format!("已触发 WebView #{} 下载", slot_id + 1);
                    }
                    if let Some(slot) = self.webview_slots.get_mut(slot_id) {
                        slot.current_index = Some(index);
                        slot.request_in_flight = true;
                        slot.status = format!("槽位 {} 已触发浏览器原生下载", slot_id + 1);
                    }
                    self.webview_status =
                        format!("WebView #{} 已触发原生下载，等待保存完成", slot_id + 1);
                } else if event == "failed" {
                    let Some(index) = index else {
                        return;
                    };
                    let error = raw
                        .get("error")
                        .and_then(|v| v.as_str())
                        .unwrap_or("WebView 下载失败")
                        .to_string();
                    let status_code = raw.get("status").and_then(|v| v.as_u64()).map(|v| v as u16);
                    self.handle_webview_failure(slot_id, index, error, status_code, cx);
                } else if event == "page_state" {
                    let Some(index) = index else {
                        return;
                    };
                    let is_pdf = raw.get("isPdf").and_then(|v| v.as_bool()).unwrap_or(false);
                    let looks_404 = raw
                        .get("looks404")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);
                    let needs_verification = raw
                        .get("needsVerification")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);
                    let url = raw
                        .get("url")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default()
                        .to_string();
                    let content_type = raw
                        .get("contentType")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default()
                        .to_string();
                    let title = raw
                        .get("title")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default()
                        .to_string();

                    if let Some(slot) = self.webview_slots.get_mut(slot_id) {
                        slot.request_in_flight = false;
                    }

                    if looks_404 {
                        self.handle_webview_failure(
                            slot_id,
                            index,
                            format!("页面返回 404/Not Found: {}", url),
                            None,
                            cx,
                        );
                        return;
                    }

                    if needs_verification {
                        self.pause_webview_current_item(
                            slot_id,
                            index,
                            format!("检测到人机验证页面，请完成验证后点击开始继续: {}", url),
                            true,
                            cx,
                        );
                        return;
                    }

                    if !is_pdf {
                        let detail = if !content_type.is_empty() {
                            format!("content-type={}", content_type)
                        } else if !title.is_empty() {
                            format!("title={}", title)
                        } else {
                            format!("url={}", url)
                        };
                        self.handle_webview_failure(
                            slot_id,
                            index,
                            format!("当前页面不是 PDF，已跳过: {}", detail),
                            None,
                            cx,
                        );
                        return;
                    }

                    if let Some(slot) = self.webview_slots.get_mut(slot_id) {
                        slot.status = format!("槽位 {} 页面检查通过，准备开始下载", slot_id + 1);
                    }
                    self.webview_status =
                        format!("WebView #{} 页面检查通过，开始下载", slot_id + 1);
                    self.continue_webview_download_for_slot(slot_id, cx);
                }
            }
            _ => {}
        }
    }

    fn should_auto_continue_webview(&self, slot_id: usize, loaded_url: &str) -> bool {
        if !self.running || !self.use_webview_mode {
            return false;
        }

        let Some(slot) = self.webview_slots.get(slot_id) else {
            return false;
        };

        if !slot.waiting_for_user || slot.request_in_flight {
            return false;
        }

        if slot.manual_paused {
            return false;
        }

        !loaded_url.is_empty() && loaded_url != WEBVIEW_HOME_URL
    }

    fn resolve_webview_download_url(&self, slot_id: usize, item: &BatchDownloadItem) -> String {
        let current_url = self
            .webview_slots
            .get(slot_id)
            .map(|slot| slot.current_url.trim().to_string())
            .unwrap_or_default();
        let lower = current_url.to_ascii_lowercase();

        if current_url.starts_with("blob:") || lower.ends_with(".pdf") || lower.contains(".pdf?") {
            return current_url.to_string();
        }

        item.url.clone()
    }

    fn load_webview_url_for_slot(
        &mut self,
        slot_id: usize,
        url: &str,
        cx: &mut Context<ToolboxPanel>,
    ) -> Result<(), String> {
        let Some(slot) = self.webview_slots.get_mut(slot_id) else {
            return Err(format!("不存在的 WebView 槽位: {}", slot_id + 1));
        };
        let Some(ref webview_entity) = slot.entity else {
            return Err("当前平台未启用 WebView".to_string());
        };

        let target = url.to_string();
        webview_entity.update(cx, |webview, _| {
            webview.show();
            webview.load_url(&target);
        });
        slot.target_url = target.clone();
        slot.current_url = target.clone();
        slot.status = format!("槽位 {} 已打开: {}", slot_id + 1, target);
        Ok(())
    }

    fn build_webview_download_script(
        item: &BatchDownloadItem,
        index: usize,
        download_url: &str,
    ) -> Result<String, String> {
        let payload = serde_json::json!({
            "index": index,
            "url": download_url,
            "savePath": item.save_path,
            "fileName": item.file_name,
        });
        let payload_json = serde_json::to_string(&payload)
            .map_err(|err| format!("序列化 WebView 下载参数失败: {}", err))?;

        Ok(format!(
            r#"(function() {{
  const payload = {payload_json};
  const endpoint = 'browserLog://log';
  const send = (message) => {{
    const body = JSON.stringify(Object.assign({{ type: 'batch_download' }}, message));
    const transport = window._browserLog_fetch || window.fetch;
    return transport.call(window, endpoint, {{ method: 'POST', body }}).catch(() => {{}});
  }};

  if (window.__batkBatchAbortController) {{
    try {{ window.__batkBatchAbortController.abort(); }} catch (_) {{}}
  }}
  window.__batkBatchAbortController = new AbortController();

  (async () => {{
    try {{
      const link = document.createElement('a');
      link.href = payload.url;
      link.download = payload.fileName || '';
      link.rel = 'noopener';
      link.style.display = 'none';
      document.body.appendChild(link);
      link.click();
      link.remove();

      await send({{
        event: 'requested',
        index: payload.index,
        savePath: payload.savePath,
        fileName: payload.fileName,
        url: payload.url,
      }});
    }} catch (error) {{
      await send({{
        event: 'failed',
        index: payload.index,
        error: String(error && error.message ? error.message : error),
        savePath: payload.savePath,
        fileName: payload.fileName,
      }});
    }}
  }})();
}})();"#,
        ))
    }

    fn build_webview_page_state_script(index: usize) -> Result<String, String> {
        let payload_json = serde_json::to_string(&serde_json::json!({ "index": index }))
            .map_err(|err| format!("序列化页面检查参数失败: {}", err))?;

        Ok(format!(
            r#"(function() {{
  const payload = {payload_json};
  const endpoint = 'browserLog://log';
  const send = (message) => {{
    const body = JSON.stringify(Object.assign({{ type: 'batch_download' }}, message));
    const transport = window._browserLog_fetch || window.fetch;
    return transport.call(window, endpoint, {{ method: 'POST', body }}).catch(() => {{}});
  }};

  (async () => {{
    try {{
      const href = String(window.location.href || '');
      const contentType = String(document.contentType || '').toLowerCase();
      const title = String(document.title || '');
      const bodyText = String(document.body?.innerText || '').slice(0, 2000).toLowerCase();
      const isVisible = (el) => {{
        if (!el) return false;
        const style = window.getComputedStyle(el);
        if (!style) return false;
        if (style.display === 'none' || style.visibility === 'hidden') return false;
        const rect = el.getBoundingClientRect();
        return rect.width > 0 && rect.height > 0;
      }};
      const hasPdfEmbed = !!document.querySelector(
        'embed[type*="pdf"], object[type*="pdf"], iframe[src^="blob:"], iframe[src*=".pdf"], embed[src^="blob:"], embed[src*=".pdf"], object[data^="blob:"], object[data*=".pdf"]'
      );
      const verificationSelectors = [
        '.cf-turnstile',
        '.h-captcha',
        '.g-recaptcha',
        'iframe[src*="recaptcha"]',
        'iframe[src*="turnstile"]',
        '[name="cf-turnstile-response"]',
        '[name="g-recaptcha-response"]',
        '#challenge-form',
        'form[action*="/challenge"]',
        'form[action*="/cdn-cgi/challenge-platform"]',
        '[data-sitekey][class*="captcha"]',
        '[data-sitekey][class*="turnstile"]'
      ];
      const hasVerificationWidget = verificationSelectors.some((selector) =>
        Array.from(document.querySelectorAll(selector)).some((el) => isVisible(el))
      );
      const isPdfUrl = href.startsWith('blob:') || /\.pdf(?:$|\?)/i.test(href);
      const isPdf = isPdfUrl || contentType.includes('pdf') || hasPdfEmbed;
      const looks404 =
        /\b404\b/.test(title) ||
        /\b404\b/.test(bodyText) ||
        /not found/i.test(title) ||
        /not found/i.test(bodyText) ||
        /page not found/i.test(title) ||
        /page not found/i.test(bodyText) ||
        /不存在/.test(title) ||
        /不存在/.test(bodyText);
      const isChallengeUrl =
        href.includes('/challenge') ||
        href.includes('/cdn-cgi/challenge-platform') ||
        href.includes('/captcha');
      const needsVerification =
        hasVerificationWidget || isChallengeUrl;

      await send({{
        event: 'page_state',
        index: payload.index,
        url: href,
        contentType,
        title,
        isPdf,
        looks404,
        needsVerification,
      }});
    }} catch (error) {{
      await send({{
        event: 'failed',
        index: payload.index,
        error: '页面检查失败: ' + String(error && error.message ? error.message : error),
      }});
    }}
  }})();
}})();"#,
        ))
    }

    fn inspect_current_webview_page_for_slot(
        &mut self,
        slot_id: usize,
        cx: &mut Context<ToolboxPanel>,
    ) {
        if !self.running || !self.use_webview_mode {
            return;
        }

        let Some(index) = self
            .webview_slots
            .get(slot_id)
            .and_then(|slot| slot.current_index)
        else {
            return;
        };

        let Some(slot) = self.webview_slots.get(slot_id) else {
            return;
        };
        let Some(ref webview_entity) = slot.entity else {
            self.webview_status = "当前平台未启用 WebView".to_string();
            return;
        };

        let script = match Self::build_webview_page_state_script(index) {
            Ok(script) => script,
            Err(err) => {
                self.webview_status = err;
                return;
            }
        };

        let eval_result = webview_entity.update(cx, |webview, _| webview.evaluate_script(&script));
        match eval_result {
            Ok(()) => {
                if let Some(slot) = self.webview_slots.get_mut(slot_id) {
                    slot.request_in_flight = true;
                    slot.status = format!("槽位 {} 正在检查页面状态", slot_id + 1);
                }
            }
            Err(err) => {
                self.handle_webview_failure(
                    slot_id,
                    index,
                    format!("执行页面检查失败: {}", err),
                    None,
                    cx,
                );
            }
        }
    }

    fn start_webview_batch_download(
        &mut self,
        items: Vec<BatchDownloadItem>,
        cx: &mut Context<ToolboxPanel>,
    ) {
        self.stop_flag.store(false, Ordering::Relaxed);
        self.running = true;
        self.use_webview_mode = true;
        self.show_webview_dashboard = true;
        self.success_count = 0;
        self.failed_count = 0;
        self.items = items;

        for slot in &mut self.webview_slots {
            slot.status = format!("槽位 {} 待命", slot.id + 1);
            slot.target_url = WEBVIEW_HOME_URL.to_string();
            slot.current_url = WEBVIEW_HOME_URL.to_string();
            slot.current_index = None;
            slot.waiting_for_user = false;
            slot.manual_paused = false;
            slot.request_in_flight = false;
        }

        self.webview_status = format!(
            "检测到 Wiley 地址，已切换为 WebView 下载模式。根据并发配置已创建 {} 个 WebView。",
            self.webview_slots.len()
        );
        self.status = format!("已准备 {} 个 WebView 下载任务", self.items.len());

        if self.webview_slots.is_empty() {
            self.status = "当前平台未启用 WebView，无法使用会话下载".to_string();
            self.running = false;
            self.use_webview_mode = false;
            cx.notify();
            return;
        }

        for slot_id in 0..self.webview_slots.len() {
            self.try_start_next_webview_item_for_slot(slot_id, cx);
        }

        cx.notify();
    }

    fn open_current_webview_target(&mut self, cx: &mut Context<ToolboxPanel>) {
        let mut opened = 0usize;
        for slot_id in 0..self.webview_slots.len() {
            let target = self
                .webview_slots
                .get(slot_id)
                .and_then(|slot| slot.current_index)
                .and_then(|index| self.items.get(index))
                .map(|item| item.url.clone())
                .unwrap_or_else(|| {
                    self.webview_slots
                        .get(slot_id)
                        .map(|slot| slot.current_url.clone())
                        .unwrap_or_else(|| WEBVIEW_HOME_URL.to_string())
                });

            if target.is_empty() {
                continue;
            }

            if self.load_webview_url_for_slot(slot_id, &target, cx).is_ok() {
                opened += 1;
            }
        }

        self.webview_status = if opened > 0 {
            format!("已刷新 {} 个 WebView 的当前下载地址", opened)
        } else {
            "当前没有可打开的下载地址".to_string()
        };
        cx.notify();
    }

    fn reload_webview(&mut self, cx: &mut Context<ToolboxPanel>) {
        let mut count = 0usize;
        for slot in &mut self.webview_slots {
            if let Some(ref webview_entity) = slot.entity {
                if webview_entity
                    .update(cx, |webview, _| webview.reload())
                    .is_ok()
                {
                    count += 1;
                }
            }
        }

        self.webview_status = if count > 0 {
            format!("已刷新 {} 个 WebView", count)
        } else {
            "当前平台未启用 WebView".to_string()
        };
        cx.notify();
    }

    fn continue_webview_download(&mut self, cx: &mut Context<ToolboxPanel>) {
        if !self.running || !self.use_webview_mode {
            self.webview_status = "当前批次未处于 WebView 下载模式".to_string();
            cx.notify();
            return;
        }

        let mut started = 0usize;
        for slot_id in 0..self.webview_slots.len() {
            if self
                .webview_slots
                .get(slot_id)
                .and_then(|slot| slot.current_index)
                .is_some()
            {
                self.resume_webview_slot(slot_id, cx);
                started += 1;
            }
        }

        if started == 0 {
            self.webview_status = "当前没有待处理的 WebView 下载任务".to_string();
        }
        cx.notify();
    }

    fn pause_webview_slot(&mut self, slot_id: usize, cx: &mut Context<ToolboxPanel>) {
        let Some(slot) = self.webview_slots.get_mut(slot_id) else {
            return;
        };

        slot.manual_paused = true;
        slot.waiting_for_user = true;
        slot.request_in_flight = false;

        if let Some(index) = slot.current_index {
            if let Some(item) = self.items.get_mut(index) {
                item.status = format!("已暂停，等待 WebView #{} 手动开始", slot_id + 1);
                item.downloaded_bytes = 0;
                item.progress_percent = 0.0;
            }
            slot.status = format!("槽位 {} 已手动暂停", slot_id + 1);
        } else {
            slot.status = format!("槽位 {} 已暂停，等待开始", slot_id + 1);
        }

        if let Some(ref webview_entity) = slot.entity {
            let _ = webview_entity.update(cx, |webview, _| {
                webview.evaluate_script(
                    "try { window.__batkBatchAbortController && window.__batkBatchAbortController.abort(); } catch (_) {}",
                )
            });
        }

        self.webview_status = format!("WebView #{} 已暂停，等待手动开始", slot_id + 1);
        cx.notify();
    }

    fn resume_webview_slot(&mut self, slot_id: usize, cx: &mut Context<ToolboxPanel>) {
        let Some(slot) = self.webview_slots.get_mut(slot_id) else {
            return;
        };

        slot.manual_paused = false;

        if slot.current_index.is_some() {
            slot.waiting_for_user = true;
            slot.request_in_flight = false;
            slot.status = format!("槽位 {} 已开始，正在检查当前页面", slot_id + 1);
            self.webview_status = format!("WebView #{} 已开始，正在检查当前页面状态", slot_id + 1);
            self.inspect_current_webview_page_for_slot(slot_id, cx);
        } else {
            slot.status = format!("槽位 {} 已开始，正在领取下一个任务", slot_id + 1);
            self.webview_status = format!("WebView #{} 已开始，准备执行后续任务", slot_id + 1);
            self.try_start_next_webview_item_for_slot(slot_id, cx);
            cx.notify();
        }
    }

    fn continue_webview_download_for_slot(
        &mut self,
        slot_id: usize,
        cx: &mut Context<ToolboxPanel>,
    ) {
        if !self.running || !self.use_webview_mode {
            return;
        }

        if self
            .webview_slots
            .get(slot_id)
            .map(|slot| slot.manual_paused)
            .unwrap_or(false)
        {
            return;
        }

        let Some(slot_current_index) = self
            .webview_slots
            .get(slot_id)
            .and_then(|slot| slot.current_index)
        else {
            return;
        };

        let Some(item) = self.items.get(slot_current_index).cloned() else {
            return;
        };

        let current_url = self
            .webview_slots
            .get(slot_id)
            .map(|slot| slot.current_url.clone())
            .unwrap_or_default();

        if !current_url.is_empty()
            && current_url != WEBVIEW_HOME_URL
            && !Self::same_host(&current_url, &item.url)
        {
            if self
                .load_webview_url_for_slot(slot_id, &item.url, cx)
                .is_ok()
            {
                if let Some(slot) = self.webview_slots.get_mut(slot_id) {
                    slot.waiting_for_user = true;
                    slot.request_in_flight = false;
                    slot.status = format!("槽位 {} 已切换下载域名，等待页面加载完成", slot_id + 1);
                }
                self.webview_status = format!(
                    "WebView #{} 已切换下载域名，页面加载完成后会自动继续",
                    slot_id + 1
                );
            }
            return;
        }

        let Some(slot) = self.webview_slots.get(slot_id) else {
            return;
        };
        let Some(ref webview_entity) = slot.entity else {
            self.webview_status = "当前平台未启用 WebView".to_string();
            return;
        };

        let download_url = self.resolve_webview_download_url(slot_id, &item);
        log::info!(
            "webview slot download starting: slot={}, source_url={}, current_url={}, file_name={}, save_path={}",
            slot_id,
            download_url,
            current_url,
            item.file_name,
            item.save_path
        );
        register_webview_download(
            &download_url,
            slot_current_index,
            PathBuf::from(&item.save_path),
        );

        let script =
            match Self::build_webview_download_script(&item, slot_current_index, &download_url) {
                Ok(script) => script,
                Err(err) => {
                    self.webview_status = err;
                    return;
                }
            };

        let eval_result = webview_entity.update(cx, |webview, _| webview.evaluate_script(&script));
        self.webview_status = match eval_result {
            Ok(()) => {
                if let Some(item) = self.items.get_mut(slot_current_index) {
                    item.status = format!("等待 WebView #{} 返回数据", slot_id + 1);
                }
                if let Some(slot) = self.webview_slots.get_mut(slot_id) {
                    slot.waiting_for_user = false;
                    slot.request_in_flight = true;
                    slot.status = format!(
                        "槽位 {} 已发起下载: {} <- {}",
                        slot_id + 1,
                        item.file_name,
                        download_url
                    );
                }
                format!(
                    "已通过 WebView #{} 会话发起下载: {}",
                    slot_id + 1,
                    item.file_name
                )
            }
            Err(err) => format!("更新 WebView #{} 失败: {}", slot_id + 1, err),
        };
    }

    fn pause_webview_current_item(
        &mut self,
        slot_id: usize,
        index: usize,
        error: String,
        needs_verification: bool,
        cx: &mut Context<ToolboxPanel>,
    ) {
        if let Some(item) = self.items.get_mut(index) {
            item.status = if needs_verification {
                format!("等待 WebView #{} 完成验证", slot_id + 1)
            } else {
                format!("失败: {}", error)
            };
            item.downloaded_bytes = 0;
            item.progress_percent = 0.0;
        }
        if let Some(slot) = self.webview_slots.get_mut(slot_id) {
            slot.current_index = Some(index);
            slot.waiting_for_user = needs_verification;
            slot.manual_paused = needs_verification;
            slot.request_in_flight = false;
            if let Some(item) = self.items.get(index) {
                slot.target_url = item.url.clone();
            }
            slot.status = if needs_verification {
                format!("槽位 {} 等待用户验证", slot_id + 1)
            } else {
                format!("槽位 {} 失败: {}", slot_id + 1, error)
            };
        }
        self.webview_status = if needs_verification {
            format!(
                "WebView #{} 当前请求被拦截：{}。请在该窗口完成验证后继续。",
                slot_id + 1,
                error
            )
        } else {
            error
        };
        cx.notify();
    }

    fn handle_webview_failure(
        &mut self,
        slot_id: usize,
        index: usize,
        error: String,
        status_code: Option<u16>,
        cx: &mut Context<ToolboxPanel>,
    ) {
        let needs_verification = status_code == Some(403);
        if needs_verification {
            self.pause_webview_current_item(slot_id, index, error, true, cx);
            return;
        }

        if let Some(item) = self.items.get_mut(index) {
            item.status = format!("失败: {}", error);
            item.downloaded_bytes = 0;
            item.progress_percent = 0.0;
        }
        if let Some(slot) = self.webview_slots.get_mut(slot_id) {
            slot.status = format!("槽位 {} 下载失败", slot_id + 1);
            slot.target_url = WEBVIEW_HOME_URL.to_string();
            slot.current_index = None;
            slot.waiting_for_user = false;
            slot.manual_paused = false;
            slot.request_in_flight = false;
        }
        self.failed_count += 1;
        self.webview_status = format!("WebView #{} 下载失败: {}", slot_id + 1, error);
        self.try_start_next_webview_item_for_slot(slot_id, cx);
        cx.notify();
    }

    fn try_start_next_webview_item_for_slot(
        &mut self,
        slot_id: usize,
        cx: &mut Context<ToolboxPanel>,
    ) {
        if !self.running || !self.use_webview_mode {
            return;
        }

        if self
            .webview_slots
            .get(slot_id)
            .map(|slot| slot.manual_paused)
            .unwrap_or(false)
        {
            return;
        }

        let next_index = self.items.iter().position(|item| item.status == "等待中");
        if let Some(index) = next_index {
            let next_url = self.items[index].url.clone();

            if let Some(item) = self.items.get_mut(index) {
                item.status = format!("等待 WebView #{} 加载任务", slot_id + 1);
            }
            if let Some(slot) = self.webview_slots.get_mut(slot_id) {
                slot.current_index = Some(index);
                slot.waiting_for_user = true;
                slot.target_url = next_url.clone();
                slot.manual_paused = false;
                slot.request_in_flight = false;
                slot.status = format!("槽位 {} 正在加载任务 {}", slot_id + 1, index + 1);
            }

            self.webview_status = format!(
                "WebView #{} 正在加载下一个任务地址: {}",
                slot_id + 1,
                next_url
            );
            let _ = self.load_webview_url_for_slot(slot_id, &next_url, cx);
            return;
        }

        if let Some(slot) = self.webview_slots.get_mut(slot_id) {
            slot.current_index = None;
            slot.waiting_for_user = false;
            slot.target_url = WEBVIEW_HOME_URL.to_string();
            slot.manual_paused = false;
            slot.request_in_flight = false;
            slot.status = format!("槽位 {} 待命", slot_id + 1);
        }
        self.finish_webview_batch_if_done();
    }

    fn finish_webview_batch_if_done(&mut self) {
        let has_pending = self.items.iter().any(|item| {
            item.status == "等待中"
                || item.status.contains("等待 WebView")
                || item.status == "已触发 WebView 下载"
                || item.status.contains("原生下载中")
                || item.status.contains("等待 WebView #")
        });
        let has_active_slot = self.webview_slots.iter().any(|slot| {
            slot.current_index.is_some() || slot.waiting_for_user || slot.manual_paused
        });

        if has_pending || has_active_slot {
            return;
        }

        self.running = false;
        self.use_webview_mode = false;
        self.status = format!(
            "WebView 下载完成，共 {} 个，成功 {}，失败 {}",
            self.items.len(),
            self.success_count,
            self.failed_count
        );
        self.webview_status = "当前批次的 WebView 下载已结束".to_string();
    }

    fn close_webview_dashboard(&mut self, cx: &mut Context<ToolboxPanel>) {
        if self.running {
            self.webview_status = "当前仍在执行下载，停止后才能返回下载设置页。".to_string();
            cx.notify();
            return;
        }

        self.show_webview_dashboard = false;
        self.webview_status = "已返回下载设置页".to_string();
        cx.notify();
    }

    async fn download_all(
        items: Vec<BatchDownloadItem>,
        output_dir: PathBuf,
        concurrency: usize,
        stop_flag: Arc<AtomicBool>,
        tx: mpsc::Sender<DownloadProgressEvent>,
    ) -> Result<(usize, usize), String> {
        std::fs::create_dir_all(&output_dir).map_err(|e| e.to_string())?;

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(120))
            .connect_timeout(Duration::from_secs(20))
            .user_agent("Mozilla/5.0 BatK Batch Downloader")
            .build()
            .map_err(|e| e.to_string())?;

        let success_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let failed_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));

        futures::stream::iter(items.into_iter().enumerate())
            .for_each_concurrent(concurrency, |(index, item)| {
                let client = client.clone();
                let output_dir = output_dir.clone();
                let stop_flag = stop_flag.clone();
                let tx = tx.clone();
                let success_count = success_count.clone();
                let failed_count = failed_count.clone();
                async move {
                    if stop_flag.load(Ordering::Relaxed) {
                        let _ = tx.send(DownloadProgressEvent::Failed {
                            index,
                            error: "已停止".to_string(),
                        });
                        failed_count.fetch_add(1, Ordering::Relaxed);
                        return;
                    }

                    let target = output_dir.join(&item.file_name);
                    log::info!(
                        "batch download starting: url={}, save_path={}, file_name={}",
                        item.url,
                        target.display(),
                        item.file_name
                    );
                    let _ = tx.send(DownloadProgressEvent::Started {
                        index,
                        total_bytes: None,
                    });

                    let result: Result<(), String> = async {
                        let headers = Self::build_request_headers(&item);
                        let resp = client
                            .get(&item.url)
                            .headers(headers)
                            .send()
                            .await
                            .map_err(|e| e.to_string())?;
                        if !resp.status().is_success() {
                            let status = resp.status();
                            let cf_mitigated = resp
                                .headers()
                                .get("cf-mitigated")
                                .and_then(|v| v.to_str().ok())
                                .unwrap_or_default()
                                .to_string();

                            if status == reqwest::StatusCode::FORBIDDEN && cf_mitigated == "challenge" {
                                return Err(
                                    "HTTP 403（Cloudflare challenge，浏览器里能打开但脚本下载被拦截）"
                                        .to_string(),
                                );
                            }

                            return Err(format!("HTTP {}", status));
                        }
                        let total_bytes = resp.content_length();
                        let _ = tx.send(DownloadProgressEvent::Started { index, total_bytes });

                        let mut file = tokio::fs::File::create(&target)
                            .await
                            .map_err(|e| e.to_string())?;
                        let mut downloaded_bytes = 0u64;
                        let mut stream = resp.bytes_stream();

                        while let Some(chunk) = stream.next().await {
                            if stop_flag.load(Ordering::Relaxed) {
                                return Err("已停止".to_string());
                            }
                            let chunk = chunk.map_err(|e| e.to_string())?;
                            tokio::io::AsyncWriteExt::write_all(&mut file, &chunk)
                                .await
                                .map_err(|e| e.to_string())?;
                            downloaded_bytes += chunk.len() as u64;
                            let _ = tx.send(DownloadProgressEvent::Progress {
                                index,
                                downloaded_bytes,
                                total_bytes,
                            });
                        }
                        tokio::io::AsyncWriteExt::flush(&mut file)
                            .await
                            .map_err(|e| e.to_string())?;
                        Self::ensure_valid_pdf_file(&target)?;
                        Ok(())
                    }
                    .await;

                    match result {
                        Ok(()) => {
                            log::info!(
                                "batch download finished: url={}, save_path={}, file_name={}",
                                item.url,
                                target.display(),
                                item.file_name
                            );
                            let _ = tx.send(DownloadProgressEvent::Finished { index });
                            success_count.fetch_add(1, Ordering::Relaxed);
                        }
                        Err(err) => {
                            let _ = tokio::fs::remove_file(&target).await;
                            let _ = tx.send(DownloadProgressEvent::Failed {
                                index,
                                error: err.clone(),
                            });
                            failed_count.fetch_add(1, Ordering::Relaxed);
                            log::warn!(
                                "batch download failed: url={}, save_path={}, file_name={}, error={}",
                                item.url,
                                target.display(),
                                item.file_name,
                                err
                            );
                        }
                    }
                }
            })
            .await;

        Ok((
            success_count.load(Ordering::Relaxed),
            failed_count.load(Ordering::Relaxed),
        ))
    }

    pub fn start_batch_download(&mut self, cx: &mut Context<ToolboxPanel>) {
        if self.running {
            return;
        }

        let mut template = String::new();
        self.template_input.update(cx, |s, _| {
            template = s.text().to_string();
        });

        if !template.contains("{0}") {
            self.status = "URL 模板必须包含 {0} 占位符".to_string();
            cx.notify();
            return;
        }

        let Some(output_dir) = self.output_dir.clone() else {
            self.status = "请先选择下载目录".to_string();
            cx.notify();
            return;
        };

        let mut paths_text = String::new();
        self.paths_input.update(cx, |s, _| {
            paths_text = s.text().to_string();
        });

        let paths: Vec<String> = paths_text
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .map(|s| s.to_string())
            .collect();

        if paths.is_empty() {
            self.status = "请先输入至少一个待替换路径".to_string();
            cx.notify();
            return;
        }

        let mut concurrency = 3usize;
        self.concurrency_input.update(cx, |s, _| {
            let text = s.text().to_string();
            if let Ok(v) = text.trim().parse::<usize>() {
                if v > 0 {
                    concurrency = v.min(32);
                }
            }
        });

        let items = Self::build_items(&template, &output_dir, &paths);
        if Self::should_use_webview(&items) {
            self.start_webview_batch_download(items, cx);
            return;
        }

        self.stop_flag.store(false, Ordering::Relaxed);
        self.running = true;
        self.use_webview_mode = false;
        self.success_count = 0;
        self.failed_count = 0;
        for slot in &mut self.webview_slots {
            slot.target_url = WEBVIEW_HOME_URL.to_string();
            slot.current_index = None;
            slot.waiting_for_user = false;
            slot.request_in_flight = false;
            slot.status = format!("槽位 {} 待命", slot.id + 1);
        }
        self.items = items;
        self.status = format!(
            "准备下载 {} 个文件，最大并发 {}，目录: {}",
            paths.len(),
            concurrency,
            output_dir.display()
        );
        cx.notify();

        let entity = cx.entity().downgrade();
        let stop_flag = self.stop_flag.clone();
        let (tx, rx) = mpsc::channel::<DownloadProgressEvent>();

        // UI 线程中轮询下载进度并更新列表
        let poll_entity = entity.clone();
        cx.spawn(async move |entity, cx| {
            loop {
                let mut disconnected = false;
                loop {
                    match rx.try_recv() {
                        Ok(event) => {
                            if let Some(ent) = poll_entity.upgrade() {
                                cx.update(|cx| {
                                    ent.update(cx, |this, cx| {
                                        let state = &mut this.api_batch_download;
                                        match event {
                                            DownloadProgressEvent::Started {
                                                index,
                                                total_bytes,
                                            } => {
                                                if let Some(item) = state.items.get_mut(index) {
                                                    item.status = "下载中".to_string();
                                                    item.total_bytes = total_bytes;
                                                }
                                            }
                                            DownloadProgressEvent::Progress {
                                                index,
                                                downloaded_bytes,
                                                total_bytes,
                                            } => {
                                                if let Some(item) = state.items.get_mut(index) {
                                                    item.downloaded_bytes = downloaded_bytes;
                                                    item.total_bytes = total_bytes;
                                                    item.progress_percent = total_bytes
                                                        .filter(|v| *v > 0)
                                                        .map(|total| {
                                                            ((downloaded_bytes as f64
                                                                / total as f64)
                                                                * 100.0)
                                                                as f32
                                                        })
                                                        .unwrap_or(0.0);
                                                }
                                            }
                                            DownloadProgressEvent::Finished { index } => {
                                                if let Some(item) = state.items.get_mut(index) {
                                                    item.status = "已完成".to_string();
                                                    item.progress_percent = 100.0;
                                                }
                                                state.success_count += 1;
                                            }
                                            DownloadProgressEvent::Failed { index, error } => {
                                                if let Some(item) = state.items.get_mut(index) {
                                                    item.status = format!("失败: {}", error);
                                                }
                                                state.failed_count += 1;
                                            }
                                        }
                                        cx.notify();
                                    });
                                });
                            }
                        }
                        Err(mpsc::TryRecvError::Empty) => break,
                        Err(mpsc::TryRecvError::Disconnected) => {
                            disconnected = true;
                            break;
                        }
                    }
                }

                if disconnected {
                    break;
                }
                smol::Timer::after(Duration::from_millis(80)).await;
            }
        })
        .detach();

        cx.spawn(async move |entity, cx| {
            let total = paths.len();
            let result = std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
                rt.block_on(Self::download_all(
                    Self::build_items(&template, &output_dir, &paths),
                    output_dir,
                    concurrency,
                    stop_flag,
                    tx,
                ))
            })
            .join()
            .map_err(|e| format!("Thread panic: {:?}", e))
            .and_then(|r| r);

            if let Some(ent) = entity.upgrade() {
                cx.update(|cx| {
                    ent.update(cx, |this, cx| {
                        let stats = &mut this.api_batch_download;
                        stats.running = false;
                        stats.status = match result {
                            Ok((success, failed)) => {
                                format!(
                                    "下载完成，共 {} 个，成功 {}，失败 {}",
                                    total, success, failed
                                )
                            }
                            Err(err) => format!("下载失败: {}", err),
                        };
                        cx.notify();
                    });
                });
            }
        })
        .detach();
    }

    pub fn stop_batch_download(&mut self, cx: &mut Context<ToolboxPanel>) {
        if !self.running {
            return;
        }
        self.stop_flag.store(true, Ordering::Relaxed);
        if self.use_webview_mode {
            for slot in &self.webview_slots {
                if let Some(ref webview_entity) = slot.entity {
                    let _ = webview_entity.update(cx, |webview, _| {
                        webview.evaluate_script(
                            "try { window.__batkBatchAbortController && window.__batkBatchAbortController.abort(); } catch (_) {}",
                        )
                    });
                }
            }
        }
        self.running = false;
        self.use_webview_mode = false;
        for slot in &mut self.webview_slots {
            slot.target_url = WEBVIEW_HOME_URL.to_string();
            slot.current_index = None;
            slot.waiting_for_user = false;
            slot.manual_paused = false;
            slot.request_in_flight = false;
            slot.status = format!("槽位 {} 已停止", slot.id + 1);
        }
        self.status = "已请求停止，正在结束当前批次…".to_string();
        self.webview_status = "已请求停止当前 WebView/网络下载批次".to_string();
        cx.notify();
    }

    pub fn destroy_webview_pool(&mut self, cx: &mut Context<ToolboxPanel>) {
        self.stop_flag.store(true, Ordering::Relaxed);

        for slot in &self.webview_slots {
            if let Some(ref webview_entity) = slot.entity {
                let _ = webview_entity.update(cx, |webview, _| {
                    webview.evaluate_script(
                        "try { window.__batkBatchAbortController && window.__batkBatchAbortController.abort(); } catch (_) {}",
                    )
                });
            }
        }

        self.running = false;
        self.use_webview_mode = false;
        self.show_webview_dashboard = false;
        self.webview_slots.clear();
        self.webview_status = "已销毁批量下载页面创建的 WebView".to_string();
        self.status = "已关闭批量下载页面并释放 WebView 资源".to_string();
        cx.notify();
    }
}
