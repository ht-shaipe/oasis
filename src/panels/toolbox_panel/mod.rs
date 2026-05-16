//! 工具箱面板：工具入口首页与各子工具（CSV 统计、CSV 分割等）。

use std::path::PathBuf;

use gpui::{
    App, AppContext as _, Context, Entity, EventEmitter, FocusHandle, Focusable,
    InteractiveElement as _, IntoElement, ParentElement as _, Render, Styled, Window,
    prelude::FluentBuilder as _, px,
};
use gpui_component::{
    dock::{Panel, PanelControl, PanelEvent},
    ActiveTheme, Icon, IconName,
    button::{Button, ButtonVariants as _},
    h_flex,
    label::Label,
    v_flex,
};
use rust_i18n::t;

use crate::panels::dock_panel::DockPanel;
use crate::utils;

mod api;
mod excel;
mod excel_move;
mod home;
mod json_convert;
mod json_merge;
mod network_scan;
mod rename;

pub use excel::{
    ConvertFormat, CsvConvertState, CsvEntry, CsvSplitState, CsvStatsState, CsvTableDelegate,
    do_convert, do_split,
};
pub use network_scan::{NetworkScanState, ScanResult};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ToolId {
    CsvStats,
    CsvSplit,
    CsvExcelConvert,
    BatchRename,
    ExcelMoveFiles,
    ApiRequest,
    ApiBatchDownload,
    JsonToCsvExcel,
    JsonMerge,
    NetworkScan,
}

#[derive(Clone)]
pub enum ViewState {
    Home,
    Tool(ToolId),
}

/// 工具箱面板
pub struct ToolboxPanel {
    pub focus_handle: FocusHandle,
    pub view: ViewState,
    pub csv_stats: excel::CsvStatsState,
    pub csv_split: excel::CsvSplitState,
    pub csv_convert: excel::CsvConvertState,
    pub batch_rename: rename::BatchRenameState,
    pub excel_move: excel_move::ExcelMoveState,
    pub api_request: api::ApiRequestState,
    pub api_batch_download: api::BatchDownloadState,
    pub json_convert: json_convert::JsonConvertState,
    pub json_merge: json_merge::JsonMergeState,
    pub network_scan: network_scan::NetworkScanState,
}

impl DockPanel for ToolboxPanel {
    fn title() -> &'static str {
        "Toolbox"
    }

    fn title_key() -> Option<&'static str> {
        Some("toolbox.title")
    }

    fn description() -> &'static str {
        "Utility tools: CSV stats, CSV split, etc."
    }

    fn new_view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }

    fn paddings() -> gpui::Pixels {
        px(16.)
    }

    fn tab_icon() -> Option<gpui_component::IconName> {
        Some(gpui_component::IconName::Folder)
    }
}

impl ToolboxPanel {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            view: ViewState::Home,
            csv_stats: excel::CsvStatsState::new(window, cx),
            csv_split: excel::CsvSplitState::new(window, cx),
            csv_convert: excel::CsvConvertState::new(),
            batch_rename: rename::BatchRenameState::new(window, cx),
            excel_move: excel_move::ExcelMoveState::new(window, cx),
            api_request: api::ApiRequestState::new(window, cx),
            api_batch_download: api::BatchDownloadState::new(window, cx),
            json_convert: json_convert::JsonConvertState::new(window, cx),
            json_merge: json_merge::JsonMergeState::new(window, cx),
            network_scan: network_scan::NetworkScanState::new(window, cx),
        }
    }

    pub fn pick_excel_move_file(&mut self, cx: &mut Context<Self>) {
        if self.excel_move.loading {
            return;
        }
        let entity = cx.entity().downgrade();
        cx.spawn(async move |entity, cx| {
            let title = t!("toolbox.excel_move.pick_excel_title").to_string();
            let path = utils::pick_file(
                &title,
                Some("Excel/CSV"),
                Some(&["xlsx", "xls", "xlsm", "csv"]),
            )
            .await;
            let _ = cx.update(|cx| {
                if let Some(ent) = entity.upgrade() {
                    ent.update(cx, |this, cx| {
                        this.excel_move.excel_path = path.clone();
                        this.excel_move.preview = None;
                        this.excel_move.ready_count = 0;
                        this.excel_move.message = None;
                        this.excel_move.headers.clear();
                        if let Some(p) = path {
                            match excel_move::read_headers(&p) {
                                Ok(headers) => this.excel_move.headers = headers,
                                Err(e) => {
                                    this.excel_move.message = Some(e);
                                    this.excel_move.message_ok = false;
                                }
                            }
                        }
                        cx.notify();
                    });
                }
            });
        })
        .detach();
    }

    pub fn pick_excel_move_input_dir(&mut self, cx: &mut Context<Self>) {
        if self.excel_move.loading {
            return;
        }
        let entity = cx.entity().downgrade();
        cx.spawn(async move |entity, cx| {
            let title = t!("toolbox.excel_move.pick_input_title").to_string();
            let dir = utils::pick_folder(&title).await;
            let _ = cx.update(|cx| {
                if let Some(ent) = entity.upgrade() {
                    ent.update(cx, |this, cx| {
                        this.excel_move.input_dir = dir;
                        this.excel_move.preview = None;
                        this.excel_move.ready_count = 0;
                        this.excel_move.message = None;
                        cx.notify();
                    });
                }
            });
        })
        .detach();
    }

    pub fn pick_excel_move_output_dir(&mut self, cx: &mut Context<Self>) {
        if self.excel_move.loading {
            return;
        }
        let entity = cx.entity().downgrade();
        cx.spawn(async move |entity, cx| {
            let title = t!("toolbox.excel_move.pick_output_title").to_string();
            let dir = utils::pick_folder(&title).await;
            let _ = cx.update(|cx| {
                if let Some(ent) = entity.upgrade() {
                    ent.update(cx, |this, cx| {
                        this.excel_move.output_dir = dir;
                        this.excel_move.preview = None;
                        this.excel_move.ready_count = 0;
                        this.excel_move.message = None;
                        cx.notify();
                    });
                }
            });
        })
        .detach();
    }

    pub fn excel_move_preview(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        let excel = match &self.excel_move.excel_path {
            Some(p) => p.clone(),
            None => {
                self.excel_move.message = Some(t!("toolbox.excel_move.no_excel").to_string());
                self.excel_move.message_ok = false;
                cx.notify();
                return;
            }
        };
        let input_dir = match &self.excel_move.input_dir {
            Some(p) => p.clone(),
            None => {
                self.excel_move.message = Some(t!("toolbox.excel_move.no_input_dir").to_string());
                self.excel_move.message_ok = false;
                cx.notify();
                return;
            }
        };
        let output_dir = match &self.excel_move.output_dir {
            Some(p) => p.clone(),
            None => {
                self.excel_move.message = Some(t!("toolbox.excel_move.no_output_dir").to_string());
                self.excel_move.message_ok = false;
                cx.notify();
                return;
            }
        };

        let suffixes = self.excel_move.parse_suffixes(cx);
        if suffixes.is_empty() {
            self.excel_move.message = Some(t!("toolbox.excel_move.no_suffixes").to_string());
            self.excel_move.message_ok = false;
            cx.notify();
            return;
        }

        let header = self.excel_move.col_header(cx);
        let idx = self.excel_move.col_index_1based(cx);

        match excel_move::build_match_plan(&excel, &header, idx, &input_dir, &suffixes) {
            Ok(statuses) => {
                let (ready, text) = excel_move::preview_text(&statuses, &output_dir);
                self.excel_move.preview = Some(text);
                self.excel_move.ready_count = ready;
                self.excel_move.message = Some(t!("toolbox.excel_move.preview_ok").to_string());
                self.excel_move.message_ok = true;
                cx.notify();
            }
            Err(e) => {
                self.excel_move.preview = None;
                self.excel_move.ready_count = 0;
                self.excel_move.message = Some(e);
                self.excel_move.message_ok = false;
                cx.notify();
            }
        }
    }

    pub fn excel_move_execute(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        let excel = match &self.excel_move.excel_path {
            Some(p) => p.clone(),
            None => {
                self.excel_move.message = Some(t!("toolbox.excel_move.no_excel").to_string());
                self.excel_move.message_ok = false;
                cx.notify();
                return;
            }
        };
        let input_dir = match &self.excel_move.input_dir {
            Some(p) => p.clone(),
            None => {
                self.excel_move.message = Some(t!("toolbox.excel_move.no_input_dir").to_string());
                self.excel_move.message_ok = false;
                cx.notify();
                return;
            }
        };
        let output_dir = match &self.excel_move.output_dir {
            Some(p) => p.clone(),
            None => {
                self.excel_move.message = Some(t!("toolbox.excel_move.no_output_dir").to_string());
                self.excel_move.message_ok = false;
                cx.notify();
                return;
            }
        };

        let suffixes = self.excel_move.parse_suffixes(cx);
        if suffixes.is_empty() {
            self.excel_move.message = Some(t!("toolbox.excel_move.no_suffixes").to_string());
            self.excel_move.message_ok = false;
            cx.notify();
            return;
        }

        let header = self.excel_move.col_header(cx);
        let idx = self.excel_move.col_index_1based(cx);

        let statuses =
            match excel_move::build_match_plan(&excel, &header, idx, &input_dir, &suffixes) {
                Ok(s) => s,
                Err(e) => {
                    self.excel_move.message = Some(e);
                    self.excel_move.message_ok = false;
                    cx.notify();
                    return;
                }
            };

        self.excel_move.loading = true;
        self.excel_move.message = None;
        cx.notify();

        let entity = cx.entity().downgrade();
        cx.spawn(async move |entity, cx| {
            let (ok, errs) = excel_move::apply_move(&statuses, &output_dir, &suffixes);
            let _ = cx.update(|cx| {
                if let Some(ent) = entity.upgrade() {
                    ent.update(cx, |this, cx| {
                        this.excel_move.loading = false;
                        this.excel_move.preview = None;
                        this.excel_move.ready_count = 0;
                        let mut msg = format!(
                            "{} {} {}",
                            t!("toolbox.excel_move.done_moved"),
                            ok,
                            t!("toolbox.excel_move.done_files_suffix")
                        );
                        if !errs.is_empty() {
                            msg.push_str(&format!(
                                "\n{}:\n{}",
                                t!("toolbox.excel_move.done_partial_errors"),
                                errs.join("\n")
                            ));
                            this.excel_move.message_ok = false;
                        } else {
                            this.excel_move.message_ok = true;
                        }
                        this.excel_move.message = Some(msg);
                        cx.notify();
                    });
                }
            });
        })
        .detach();
    }

    /// 选择目录并扫描 CSV、统计行数（CSV 统计工具）
    pub fn pick_and_scan_csv_stats(&mut self, cx: &mut Context<Self>) {
        if self.csv_stats.loading {
            return;
        }
        self.csv_stats.loading = true;
        cx.notify();

        let entity = cx.entity().downgrade();
        cx.spawn(async move |entity, cx| {
            let title = t!("toolbox.csv.pick_folder").to_string();
            let dir = utils::pick_folder(&title).await;
            let _ = cx.update(|cx| {
                if let Some(ent) = entity.upgrade() {
                    ent.update(cx, |this, cx| {
                        this.csv_stats.loading = false;
                        if let Some(path) = dir {
                            this.csv_stats.selected_dir = Some(path.clone());
                            let (entries, total) = excel::scan_csv_in_dir(&path);
                            this.csv_stats.csv_entries = entries.clone();
                            this.csv_stats.total_lines = total;
                            this.csv_stats.table_state.update(cx, |state, cx| {
                                state.delegate_mut().set_entries(entries);
                                state.refresh(cx);
                            });
                        }
                        cx.notify();
                    });
                }
            });
        })
        .detach();
    }

    /// 选择要分割的 CSV 文件，并统计总行数
    pub fn pick_csv_split_file(&mut self, cx: &mut Context<Self>) {
        let entity = cx.entity().downgrade();
        cx.spawn(async move |entity, cx| {
            let title = t!("toolbox.csv_split.pick_file_title").to_string();
            let path = utils::pick_file(&title, Some("CSV"), Some(&["csv"])).await;
            let total_lines = path.as_ref().map(|p| excel::count_lines(p)).unwrap_or(0);
            let _ = cx.update(|cx| {
                if let Some(ent) = entity.upgrade() {
                    ent.update(cx, |this, cx| {
                        this.csv_split.selected_file = path;
                        this.csv_split.total_lines =
                            this.csv_split.selected_file.as_ref().map(|_| total_lines);
                        this.csv_split.message = None;
                        cx.notify();
                    });
                }
            });
        })
        .detach();
    }

    /// 选择输出目录
    pub fn pick_csv_split_output_dir(&mut self, cx: &mut Context<Self>) {
        let entity = cx.entity().downgrade();
        cx.spawn(async move |entity, cx| {
            let title = t!("toolbox.csv_split.pick_output_title").to_string();
            let path = utils::pick_folder(&title).await;
            let _ = cx.update(|cx| {
                if let Some(ent) = entity.upgrade() {
                    ent.update(cx, |this, cx| {
                        this.csv_split.output_dir = path;
                        this.csv_split.message = None;
                        cx.notify();
                    });
                }
            });
        })
        .detach();
    }

    /// 执行 CSV 分割
    pub fn execute_csv_split(&mut self, cx: &mut Context<Self>) {
        let file = match &self.csv_split.selected_file {
            Some(p) => p.clone(),
            None => {
                self.csv_split.message = Some(t!("toolbox.csv_split.no_file").to_string());
                self.csv_split.message_ok = false;
                cx.notify();
                return;
            }
        };
        let out_dir = match &self.csv_split.output_dir {
            Some(p) => p.clone(),
            None => {
                self.csv_split.message = Some(t!("toolbox.csv_split.no_output_dir").to_string());
                self.csv_split.message_ok = false;
                cx.notify();
                return;
            }
        };
        let n: u32 = self
            .csv_split
            .num_parts_input
            .read(cx)
            .value()
            .to_string()
            .trim()
            .parse()
            .unwrap_or(2);
        if n < 1 || n > 1000 {
            self.csv_split.message = Some("分割数量请设为 1–1000".to_string());
            self.csv_split.message_ok = false;
            cx.notify();
            return;
        }

        self.csv_split.loading = true;
        self.csv_split.message = None;
        cx.notify();

        let entity = cx.entity().downgrade();
        cx.spawn(async move |entity, cx| {
            let res = excel::do_split(&file, &out_dir, n);
            let _ = cx.update(|cx| {
                if let Some(ent) = entity.upgrade() {
                    ent.update(cx, |this, cx| {
                        this.csv_split.loading = false;
                        match res {
                            Ok(()) => {
                                this.csv_split.message =
                                    Some(t!("toolbox.csv_split.success").to_string());
                                this.csv_split.message_ok = true;
                            }
                            Err(e) => {
                                this.csv_split.message =
                                    Some(format!("{}: {}", t!("toolbox.csv_split.error"), e));
                                this.csv_split.message_ok = false;
                            }
                        }
                        cx.notify();
                    });
                }
            });
        })
        .detach();
    }

    /// 选择要转换的 CSV/Excel 文件
    pub fn pick_convert_file(&mut self, cx: &mut Context<Self>) {
        let entity = cx.entity().downgrade();
        cx.spawn(async move |entity, cx| {
            let title = t!("toolbox.csv_convert.pick_file_title").to_string();
            let path =
                utils::pick_file(&title, Some("CSV/Excel"), Some(&["csv", "xlsx", "xls"])).await;
            let _ = cx.update(|cx| {
                if let Some(ent) = entity.upgrade() {
                    ent.update(cx, |this, cx| {
                        this.csv_convert.selected_file = path;
                        this.csv_convert.message = None;
                        cx.notify();
                    });
                }
            });
        })
        .detach();
    }

    /// 选择转换后的输出文件路径（保存为 .json 或 .sql）
    pub fn pick_convert_output(&mut self, cx: &mut Context<Self>) {
        let format = self.csv_convert.output_format;
        let default_name = self
            .csv_convert
            .selected_file
            .as_ref()
            .and_then(|p| p.file_stem())
            .and_then(|s| s.to_str())
            .map(|s| {
                let ext = if format == excel::ConvertFormat::Json {
                    "json"
                } else {
                    "sql"
                };
                format!("{}.{}", s, ext)
            })
            .unwrap_or_else(|| "output.json".to_string());
        let entity = cx.entity().downgrade();
        cx.spawn(async move |entity, cx| {
            let title = t!("toolbox.csv_convert.pick_output_title").to_string();
            let (filter_name, exts): (&str, &[&str]) = if format == excel::ConvertFormat::Json {
                ("JSON", &["json"])
            } else {
                ("SQL", &["sql"])
            };
            let path =
                utils::pick_save_file(&title, Some(&default_name), Some(filter_name), Some(exts))
                    .await;
            let _ = cx.update(|cx| {
                if let Some(ent) = entity.upgrade() {
                    ent.update(cx, |this, cx| {
                        this.csv_convert.output_path = path;
                        this.csv_convert.message = None;
                        cx.notify();
                    });
                }
            });
        })
        .detach();
    }

    /// 执行 CSV/Excel 转 JSON 或 SQL
    pub fn execute_convert(&mut self, cx: &mut Context<Self>) {
        let input = match &self.csv_convert.selected_file {
            Some(p) => p.clone(),
            None => {
                self.csv_convert.message = Some(t!("toolbox.csv_convert.no_file").to_string());
                self.csv_convert.message_ok = false;
                cx.notify();
                return;
            }
        };
        let output = match &self.csv_convert.output_path {
            Some(p) => p.clone(),
            None => {
                self.csv_convert.message = Some(t!("toolbox.csv_convert.no_output").to_string());
                self.csv_convert.message_ok = false;
                cx.notify();
                return;
            }
        };
        let format = self.csv_convert.output_format;

        self.csv_convert.loading = true;
        self.csv_convert.message = None;
        cx.notify();

        let entity = cx.entity().downgrade();
        cx.spawn(async move |entity, cx| {
            let res = excel::do_convert(&input, &output, format);
            let _ = cx.update(|cx| {
                if let Some(ent) = entity.upgrade() {
                    ent.update(cx, |this, cx| {
                        this.csv_convert.loading = false;
                        match res {
                            Ok(()) => {
                                this.csv_convert.message =
                                    Some(t!("toolbox.csv_convert.success").to_string());
                                this.csv_convert.message_ok = true;
                            }
                            Err(e) => {
                                this.csv_convert.message =
                                    Some(format!("{}: {}", t!("toolbox.csv_convert.error"), e));
                                this.csv_convert.message_ok = false;
                            }
                        }
                        cx.notify();
                    });
                }
            });
        })
        .detach();
    }

    /// 选择 JSON 文件（JSON 转 CSV/Excel 工具）
    pub fn pick_json_convert_file(&mut self, cx: &mut Context<Self>) {
        let entity = cx.entity().downgrade();
        cx.spawn(async move |entity, cx| {
            let title = t!("toolbox.json_convert.pick_file_title").to_string();
            let path = utils::pick_file(&title, Some("JSON"), Some(&["json"])).await;
            let _ = cx.update(|cx| {
                if let Some(ent) = entity.upgrade() {
                    ent.update(cx, |this, cx| {
                        this.json_convert.selected_file = path;
                        this.json_convert.message = None;
                        cx.notify();
                    });
                }
            });
        })
        .detach();
    }

    /// 选择 JSON 转换的输出文件路径
    pub fn pick_json_convert_output(&mut self, cx: &mut Context<Self>) {
        let format = self.json_convert.output_format;
        let default_name = self
            .json_convert
            .selected_file
            .as_ref()
            .and_then(|p| p.file_stem())
            .and_then(|s| s.to_str())
            .map(|s| {
                let ext = if format == json_convert::JsonOutputFormat::Csv {
                    "csv"
                } else {
                    "xlsx"
                };
                format!("{}.{}", s, ext)
            })
            .unwrap_or_else(|| "output.csv".to_string());
        let entity = cx.entity().downgrade();
        cx.spawn(async move |entity, cx| {
            let title = t!("toolbox.json_convert.pick_output_title").to_string();
            let (filter_name, exts): (&str, &[&str]) =
                if format == json_convert::JsonOutputFormat::Csv {
                    ("CSV", &["csv"])
                } else {
                    ("Excel", &["xlsx"])
                };
            let path =
                utils::pick_save_file(&title, Some(&default_name), Some(filter_name), Some(exts))
                    .await;
            let _ = cx.update(|cx| {
                if let Some(ent) = entity.upgrade() {
                    ent.update(cx, |this, cx| {
                        this.json_convert.output_path = path;
                        this.json_convert.message = None;
                        cx.notify();
                    });
                }
            });
        })
        .detach();
    }

    /// 执行 JSON 转 CSV/Excel
    pub fn execute_json_convert(&mut self, cx: &mut Context<Self>) {
        let input = match &self.json_convert.selected_file {
            Some(p) => p.clone(),
            None => {
                self.json_convert.message =
                    Some(t!("toolbox.json_convert.no_file").to_string());
                self.json_convert.message_ok = false;
                cx.notify();
                return;
            }
        };
        let output = match &self.json_convert.output_path {
            Some(p) => p.clone(),
            None => {
                self.json_convert.message =
                    Some(t!("toolbox.json_convert.no_output").to_string());
                self.json_convert.message_ok = false;
                cx.notify();
                return;
            }
        };
        let json_path = self
            .json_convert
            .json_path
            .read(cx)
            .value()
            .to_string()
            .trim()
            .to_string();
        let fields_str = self
            .json_convert
            .fields_input
            .read(cx)
            .value()
            .to_string();
        let fields: Vec<String> = fields_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        let format = self.json_convert.output_format;

        self.json_convert.loading = true;
        self.json_convert.message = None;
        cx.notify();

        let entity = cx.entity().downgrade();
        cx.spawn(async move |entity, cx| {
            let res = json_convert::do_json_convert(&input, &output, &json_path, &fields, format);
            let _ = cx.update(|cx| {
                if let Some(ent) = entity.upgrade() {
                    ent.update(cx, |this, cx| {
                        this.json_convert.loading = false;
                        match res {
                            Ok(()) => {
                                this.json_convert.message =
                                    Some(t!("toolbox.json_convert.success").to_string());
                                this.json_convert.message_ok = true;
                            }
                            Err(e) => {
                                this.json_convert.message = Some(format!(
                                    "{}: {}",
                                    t!("toolbox.json_convert.error"),
                                    e
                                ));
                                this.json_convert.message_ok = false;
                            }
                        }
                        cx.notify();
                    });
                }
            });
        })
        .detach();
    }

    /// 选择批量转换的输入目录
    pub fn pick_json_batch_input_dir(&mut self, cx: &mut Context<Self>) {
        let entity = cx.entity().downgrade();
        cx.spawn(async move |entity, cx| {
            let title = t!("toolbox.json_convert.pick_input_dir_title").to_string();
            let dir = utils::pick_folder(&title).await;
            let _ = cx.update(|cx| {
                if let Some(ent) = entity.upgrade() {
                    ent.update(cx, |this, cx| {
                        this.json_convert.input_dir = dir;
                        this.json_convert.message = None;
                        cx.notify();
                    });
                }
            });
        })
        .detach();
    }

    /// 选择批量转换的输出目录
    pub fn pick_json_batch_output_dir(&mut self, cx: &mut Context<Self>) {
        let entity = cx.entity().downgrade();
        cx.spawn(async move |entity, cx| {
            let title = t!("toolbox.json_convert.pick_output_dir_title").to_string();
            let dir = utils::pick_folder(&title).await;
            let _ = cx.update(|cx| {
                if let Some(ent) = entity.upgrade() {
                    ent.update(cx, |this, cx| {
                        this.json_convert.output_dir = dir;
                        this.json_convert.message = None;
                        cx.notify();
                    });
                }
            });
        })
        .detach();
    }

    /// 执行批量 JSON → CSV/Excel 转换
    pub fn execute_json_batch_convert(&mut self, cx: &mut Context<Self>) {
        let input_dir = match &self.json_convert.input_dir {
            Some(p) => p.clone(),
            None => {
                self.json_convert.message =
                    Some(t!("toolbox.json_convert.no_input_dir").to_string());
                self.json_convert.message_ok = false;
                cx.notify();
                return;
            }
        };
        let output_dir = match &self.json_convert.output_dir {
            Some(p) => p.clone(),
            None => {
                self.json_convert.message =
                    Some(t!("toolbox.json_convert.no_output_dir").to_string());
                self.json_convert.message_ok = false;
                cx.notify();
                return;
            }
        };
        let json_path = self
            .json_convert
            .json_path
            .read(cx)
            .value()
            .to_string()
            .trim()
            .to_string();
        let fields_str = self
            .json_convert
            .fields_input
            .read(cx)
            .value()
            .to_string();
        let fields: Vec<String> = fields_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        let format = self.json_convert.output_format;

        self.json_convert.loading = true;
        self.json_convert.message = None;
        cx.notify();

        let entity = cx.entity().downgrade();
        cx.spawn(async move |entity, cx| {
            let res = json_convert::do_batch_json_convert(
                &input_dir, &output_dir, &json_path, &fields, format,
            );
            let _ = cx.update(|cx| {
                if let Some(ent) = entity.upgrade() {
                    ent.update(cx, |this, cx| {
                        this.json_convert.loading = false;
                        match res {
                            Ok((ok, errs)) => {
                                let mut msg = format!(
                                    "{} {} {}",
                                    t!("toolbox.json_convert.batch_done"),
                                    ok,
                                    t!("toolbox.json_convert.batch_files_suffix")
                                );
                                if !errs.is_empty() {
                                    msg.push_str(&format!(
                                        "\n{}:\n{}",
                                        t!("toolbox.json_convert.batch_partial_errors"),
                                        errs.join("\n")
                                    ));
                                    this.json_convert.message_ok = false;
                                } else {
                                    this.json_convert.message_ok = true;
                                }
                                this.json_convert.message = Some(msg);
                            }
                            Err(e) => {
                                this.json_convert.message = Some(format!(
                                    "{}: {}",
                                    t!("toolbox.json_convert.error"),
                                    e
                                ));
                                this.json_convert.message_ok = false;
                            }
                        }
                        cx.notify();
                    });
                }
            });
        })
        .detach();
    }

    /// 选择 JSON 合并的输入目录
    pub fn pick_json_merge_input_dir(&mut self, cx: &mut Context<Self>) {
        let entity = cx.entity().downgrade();
        cx.spawn(async move |entity, cx| {
            let title = t!("toolbox.json_merge.pick_input_dir_title").to_string();
            let dir = utils::pick_folder(&title).await;
            let _ = cx.update(|cx| {
                if let Some(ent) = entity.upgrade() {
                    ent.update(cx, |this, cx| {
                        this.json_merge.input_dir = dir;
                        this.json_merge.message = None;
                        cx.notify();
                    });
                }
            });
        })
        .detach();
    }

    /// 选择 JSON 合并的输出文件
    pub fn pick_json_merge_output(&mut self, cx: &mut Context<Self>) {
        let default_name = "merged.json".to_string();
        let entity = cx.entity().downgrade();
        cx.spawn(async move |entity, cx| {
            let title = t!("toolbox.json_merge.pick_output_title").to_string();
            let path = utils::pick_save_file(&title, Some(&default_name), Some("JSON"), Some(&["json"]))
                .await;
            let _ = cx.update(|cx| {
                if let Some(ent) = entity.upgrade() {
                    ent.update(cx, |this, cx| {
                        this.json_merge.output_path = path;
                        this.json_merge.message = None;
                        cx.notify();
                    });
                }
            });
        })
        .detach();
    }

    /// 执行 JSON 合并
    pub fn execute_json_merge(&mut self, cx: &mut Context<Self>) {
        let input_dir = match &self.json_merge.input_dir {
            Some(p) => p.clone(),
            None => {
                self.json_merge.message =
                    Some(t!("toolbox.json_merge.no_input_dir").to_string());
                self.json_merge.message_ok = false;
                cx.notify();
                return;
            }
        };
        let output = match &self.json_merge.output_path {
            Some(p) => p.clone(),
            None => {
                self.json_merge.message =
                    Some(t!("toolbox.json_merge.no_output").to_string());
                self.json_merge.message_ok = false;
                cx.notify();
                return;
            }
        };
        let json_path = self
            .json_merge
            .json_path
            .read(cx)
            .value()
            .to_string()
            .trim()
            .to_string();

        self.json_merge.loading = true;
        self.json_merge.message = None;
        cx.notify();

        let entity = cx.entity().downgrade();
        cx.spawn(async move |entity, cx| {
            let res = json_merge::do_json_merge(&input_dir, &output, &json_path);
            let _ = cx.update(|cx| {
                if let Some(ent) = entity.upgrade() {
                    ent.update(cx, |this, cx| {
                        this.json_merge.loading = false;
                        match res {
                            Ok(count) => {
                                this.json_merge.message = Some(format!(
                                    "{} {} {}",
                                    t!("toolbox.json_merge.success"),
                                    count,
                                    t!("toolbox.json_merge.items_suffix")
                                ));
                                this.json_merge.message_ok = true;
                            }
                            Err(e) => {
                                this.json_merge.message = Some(format!(
                                    "{}: {}",
                                    t!("toolbox.json_merge.error"),
                                    e
                                ));
                                this.json_merge.message_ok = false;
                            }
                        }
                        cx.notify();
                    });
                }
            });
        })
        .detach();
    }

    /// 选择批量重命名目标目录
    pub fn pick_batch_rename_dir(&mut self, cx: &mut Context<Self>) {
        let entity = cx.entity().downgrade();
        cx.spawn(async move |entity, cx| {
            let title = t!("toolbox.rename.pick_dir_title").to_string();
            let path = utils::pick_folder(&title).await;
            let _ = cx.update(|cx| {
                if let Some(ent) = entity.upgrade() {
                    ent.update(cx, |this, cx| {
                        this.batch_rename.dir = path;
                        this.batch_rename.plan.clear();
                        this.batch_rename.preview_summary = None;
                        this.batch_rename.message = None;
                        cx.notify();
                    });
                }
            });
        })
        .detach();
    }

    /// 扫描并预览将重命名的文件
    pub fn batch_rename_preview(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        let dir = match &self.batch_rename.dir {
            Some(d) => d.clone(),
            None => {
                self.batch_rename.message = Some(t!("toolbox.rename.no_dir").to_string());
                self.batch_rename.message_ok = false;
                cx.notify();
                return;
            }
        };
        let needle = self.batch_rename.needle_input.read(cx).value().to_string();
        let replacement = self.batch_rename.replace_input.read(cx).value().to_string();
        let recursive = self.batch_rename.recursive;

        let files = match rename::list_files(&dir, recursive) {
            Ok(f) => f,
            Err(e) => {
                self.batch_rename.message =
                    Some(format!("{}: {}", t!("toolbox.rename.scan_error"), e));
                self.batch_rename.message_ok = false;
                cx.notify();
                return;
            }
        };

        match rename::build_rename_plan(&files, &needle, &replacement) {
            Ok(plan) => {
                if plan.is_empty() {
                    self.batch_rename.plan = vec![];
                    self.batch_rename.preview_summary =
                        Some(t!("toolbox.rename.no_matches").to_string());
                    self.batch_rename.message = Some(t!("toolbox.rename.no_matches").to_string());
                    self.batch_rename.message_ok = true;
                } else {
                    self.batch_rename.plan = plan.clone();
                    let lines: Vec<String> = plan
                        .iter()
                        .take(200)
                        .filter_map(|(o, n)| {
                            let on = o.file_name()?.to_string_lossy().into_owned();
                            let nn = n.file_name()?.to_string_lossy().into_owned();
                            Some(format!("{on} → {nn}"))
                        })
                        .collect();
                    let more = if plan.len() > 200 {
                        format!(
                            "\n… {} {}",
                            plan.len() - 200,
                            t!("toolbox.rename.more_omitted")
                        )
                    } else {
                        String::new()
                    };
                    let head = format!(
                        "{} {}\n",
                        t!("toolbox.rename.preview_count_prefix"),
                        plan.len()
                    );
                    let summary = format!("{head}{}{more}", lines.join("\n"));
                    self.batch_rename.preview_summary = Some(summary);
                    self.batch_rename.message = Some(t!("toolbox.rename.preview_ok").to_string());
                    self.batch_rename.message_ok = true;
                }
                cx.notify();
            }
            Err(e) => {
                self.batch_rename.plan = vec![];
                self.batch_rename.preview_summary = None;
                self.batch_rename.message = Some(e);
                self.batch_rename.message_ok = false;
                cx.notify();
            }
        }
    }

    /// 按当前规则执行重命名
    pub fn batch_rename_execute(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        let dir = match &self.batch_rename.dir {
            Some(d) => d.clone(),
            None => {
                self.batch_rename.message = Some(t!("toolbox.rename.no_dir").to_string());
                self.batch_rename.message_ok = false;
                cx.notify();
                return;
            }
        };
        let needle = self.batch_rename.needle_input.read(cx).value().to_string();
        let replacement = self.batch_rename.replace_input.read(cx).value().to_string();
        let recursive = self.batch_rename.recursive;

        let files = match rename::list_files(&dir, recursive) {
            Ok(f) => f,
            Err(e) => {
                self.batch_rename.message =
                    Some(format!("{}: {}", t!("toolbox.rename.scan_error"), e));
                self.batch_rename.message_ok = false;
                cx.notify();
                return;
            }
        };

        let plan = match rename::build_rename_plan(&files, &needle, &replacement) {
            Ok(p) => p,
            Err(e) => {
                self.batch_rename.message = Some(e);
                self.batch_rename.message_ok = false;
                cx.notify();
                return;
            }
        };

        if plan.is_empty() {
            self.batch_rename.message = Some(t!("toolbox.rename.no_matches").to_string());
            self.batch_rename.message_ok = false;
            cx.notify();
            return;
        }

        self.batch_rename.loading = true;
        self.batch_rename.message = None;
        cx.notify();

        let entity = cx.entity().downgrade();
        cx.spawn(async move |entity, cx| {
            let (ok, errs) = rename::apply_rename_plan(&plan);
            let _ = cx.update(|cx| {
                if let Some(ent) = entity.upgrade() {
                    ent.update(cx, |this, cx| {
                        this.batch_rename.loading = false;
                        this.batch_rename.plan.clear();
                        this.batch_rename.preview_summary = None;
                        let mut msg = format!(
                            "{} {} {}",
                            t!("toolbox.rename.done_renamed"),
                            ok,
                            t!("toolbox.rename.done_files_suffix")
                        );
                        if !errs.is_empty() {
                            msg.push_str(&format!(
                                "\n{}:\n{}",
                                t!("toolbox.rename.done_partial_errors"),
                                errs.join("\n")
                            ));
                            this.batch_rename.message_ok = false;
                        } else {
                            this.batch_rename.message_ok = true;
                        }
                        this.batch_rename.message = Some(msg);
                        cx.notify();
                    });
                }
            });
        })
        .detach();
    }

    /// 执行接口请求（API 请求工具）
    pub fn execute_api_request(&mut self, cx: &mut Context<Self>) {
        let url = self
            .api_request
            .url_input
            .read(cx)
            .value()
            .to_string()
            .trim()
            .to_string();
        if url.is_empty() {
            self.api_request.response_error = Some(t!("toolbox.api.url_required").to_string());
            self.api_request.response_status = None;
            self.api_request.response_body.clear();
            cx.notify();
            return;
        }
        let method = self.api_request.method;
        let params_str = self.api_request.params_input.read(cx).value().to_string();
        let headers_str = self.api_request.headers_input.read(cx).value().to_string();
        let body_str = self
            .api_request
            .body_input
            .read(cx)
            .value()
            .to_string()
            .trim()
            .to_string();

        let url_with_query = build_url_with_params(&url, &params_str);

        self.api_request.loading = true;
        self.api_request.response_status = None;
        self.api_request.response_body.clear();
        self.api_request.response_error = None;
        cx.notify();

        let entity = cx.entity().downgrade();
        cx.spawn(async move |entity, cx| {
            let result = std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
                rt.block_on(run_request(
                    &url_with_query,
                    method,
                    &headers_str,
                    &body_str,
                ))
            })
            .join()
            .unwrap_or_else(|_| Err("request thread panicked".to_string()));

            let _ = cx.update(|cx| {
                if let Some(ent) = entity.upgrade() {
                    ent.update(cx, |this, cx| {
                        this.api_request.loading = false;
                        match result {
                            Ok((status, body)) => {
                                this.api_request.response_status = Some(status);
                                this.api_request.response_body = body;
                                this.api_request.response_error = None;
                            }
                            Err(e) => {
                                this.api_request.response_status = None;
                                this.api_request.response_body.clear();
                                this.api_request.response_error = Some(e);
                            }
                        }
                        cx.notify();
                    });
                }
            });
        })
        .detach();
    }
}

/// 将 Params 区内容（每行 key=value）拼到 URL 的 query 部分（简单拼接，特殊字符需用户自行编码）
fn build_url_with_params(base: &str, params_str: &str) -> String {
    let pairs: Vec<String> = params_str
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() {
                return None;
            }
            let (k, v) = line.split_once('=').unwrap_or((line, ""));
            let k = k.trim();
            if k.is_empty() {
                return None;
            }
            Some(format!("{}={}", k, v.trim()))
        })
        .collect();
    if pairs.is_empty() {
        return base.to_string();
    }
    let query = pairs.join("&");
    let sep = if base.contains('?') { "&" } else { "?" };
    format!("{}{}{}", base, sep, query)
}

/// 在后台执行 HTTP 请求，返回 (status, body) 或错误信息
async fn run_request(
    url: &str,
    method: api::HttpMethod,
    headers_str: &str,
    body_str: &str,
) -> Result<(u16, String), String> {
    let client = reqwest::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| e.to_string())?;

    let mut req = match method {
        api::HttpMethod::Get => client.get(url),
        api::HttpMethod::Post => client.post(url),
        api::HttpMethod::Put => client.put(url),
        api::HttpMethod::Delete => client.delete(url),
        api::HttpMethod::Patch => client.patch(url),
    };

    for line in headers_str.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some((k, v)) = line.split_once(':') {
            let k = k.trim();
            let v = v.trim().trim_matches(' ');
            if !k.is_empty() {
                req = req.header(k, v);
            }
        }
    }

    if !body_str.is_empty()
        && matches!(
            method,
            api::HttpMethod::Post | api::HttpMethod::Put | api::HttpMethod::Patch
        )
    {
        req = req.body(body_str.to_string());
    }

    let resp = req.send().await.map_err(|e| e.to_string())?;
    let status = resp.status().as_u16();
    let body = resp.text().await.unwrap_or_else(|e| e.to_string());
    Ok((status, body))
}

impl Render for ToolboxPanel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let is_tool = matches!(&self.view, ViewState::Tool(_));

        let body = match &self.view {
            ViewState::Home => {
                let e = entity.clone();
                home::render_home(e, window, cx).into_any_element()
            }
            ViewState::Tool(ToolId::CsvStats) => {
                excel::CsvStatsState::render(&mut self.csv_stats, entity.clone(), window, cx)
                    .into_any_element()
            }
            ViewState::Tool(ToolId::CsvSplit) => {
                excel::CsvSplitState::render(&mut self.csv_split, entity.clone(), window, cx)
                    .into_any_element()
            }
            ViewState::Tool(ToolId::CsvExcelConvert) => {
                excel::CsvConvertState::render(&mut self.csv_convert, entity.clone(), window, cx)
                    .into_any_element()
            }
            ViewState::Tool(ToolId::BatchRename) => {
                rename::BatchRenameState::render(&mut self.batch_rename, entity.clone(), window, cx)
                    .into_any_element()
            }
            ViewState::Tool(ToolId::ExcelMoveFiles) => {
                excel_move::ExcelMoveState::render(&mut self.excel_move, entity.clone(), window, cx)
                    .into_any_element()
            }
            ViewState::Tool(ToolId::ApiRequest) => {
                api::ApiRequestState::render(&mut self.api_request, entity.clone(), window, cx)
                    .into_any_element()
            }
            ViewState::Tool(ToolId::ApiBatchDownload) => api::BatchDownloadState::render(
                &mut self.api_batch_download,
                entity.clone(),
                window,
                cx,
            )
            .into_any_element(),
            ViewState::Tool(ToolId::JsonToCsvExcel) => {
                json_convert::JsonConvertState::render(
                    &mut self.json_convert,
                    entity.clone(),
                    window,
                    cx,
                )
                .into_any_element()
            }
            ViewState::Tool(ToolId::JsonMerge) => {
                json_merge::JsonMergeState::render(
                    &mut self.json_merge,
                    entity.clone(),
                    window,
                    cx,
                )
                .into_any_element()
            }
            ViewState::Tool(ToolId::NetworkScan) => network_scan::NetworkScanState::render(
                &mut self.network_scan,
                entity.clone(),
                window,
                cx,
            )
            .into_any_element(),
        };

        let back_entity = entity.clone();
        let theme = cx.theme();
        let back_btn = Button::new("toolbox-back")
            .label(t!("toolbox.back").to_string())
            .icon(Icon::new(IconName::ChevronLeft).text_color(theme.muted_foreground))
            .outline()
            .on_click(move |_, _, cx| {
                back_entity.update(cx, |this, cx| {
                    if matches!(this.view, ViewState::Tool(ToolId::ApiBatchDownload)) {
                        this.api_batch_download.destroy_webview_pool(cx);
                    }
                    this.view = ViewState::Home;
                    cx.notify();
                });
            });

        let content = if is_tool {
            v_flex()
                .gap_3()
                .size_full()
                .overflow_hidden()
                .child(
                    h_flex().gap_2().items_center().child(back_btn).child(
                        Label::new(t!("toolbox.title").to_string())
                            .text_sm()
                            .text_color(theme.muted_foreground),
                    ),
                )
                .child(
                    gpui::div()
                        .flex_1()
                        .min_h(px(0.))
                        .overflow_hidden()
                        .child(body),
                )
                .into_any_element()
        } else {
            body
        };

        v_flex()
            .size_full()
            .overflow_hidden()
            .p_2()
            .track_focus(&self.focus_handle)
            .child(
                gpui::div()
                    .flex_1()
                    .min_h(px(0.))
                    .overflow_hidden()
                    .child(content),
            )
    }
}

impl EventEmitter<PanelEvent> for ToolboxPanel {}

impl Panel for ToolboxPanel {
    fn panel_name(&self) -> &'static str {
        "ToolboxPanel"
    }

    fn title(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        t!("toolbox.title").to_string()
    }

    fn zoomable(&self, _cx: &App) -> Option<PanelControl> {
        None
    }
}

impl Focusable for ToolboxPanel {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
