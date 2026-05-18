#![allow(dead_code)]
//! 从 Excel 指定列读取文件名，按多个后缀在目录中匹配，匹配到则移动到输出目录。

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use gpui::{
    AppContext as _, Context, Entity, IntoElement, ParentElement as _,
    SharedString, Styled, Window, prelude::FluentBuilder as _, px,
};
use gpui_component::{
    ActiveTheme, Disableable, Icon, IconName, Sizable,
    button::{Button, ButtonVariants as _},
    group_box::{GroupBox, GroupBoxVariant, GroupBoxVariants as _},
    h_flex,
    input::Input,
    label::Label,
    scroll::ScrollableElement as _,
    v_flex,
};
use rust_i18n::t;

use super::ToolboxPanel;

fn file_ext_lower(path: &Path) -> String {
    path.extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase()
}

fn read_csv_table(path: &Path) -> Result<(Vec<String>, Vec<Vec<String>>), String> {
    let mut rdr = csv::Reader::from_path(path).map_err(|e| e.to_string())?;
    let headers: Vec<String> = rdr
        .headers()
        .map_err(|e| e.to_string())?
        .iter()
        .map(|s| s.to_string())
        .collect();
    let rows: Vec<Vec<String>> = rdr
        .records()
        .map(|r| {
            r.map_err(|e| e.to_string())
                .map(|rec| rec.iter().map(|s| s.to_string()).collect())
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok((headers, rows))
}

fn read_excel_first_sheet(path: &Path) -> Result<(Vec<String>, Vec<Vec<String>>), String> {
    use calamine::{Reader as _, open_workbook_auto};
    let mut workbook = open_workbook_auto(path).map_err(|e| e.to_string())?;
    let sheet_names = workbook.sheet_names().to_vec();
    let name = sheet_names.first().ok_or("工作簿无工作表")?;
    let range = workbook
        .worksheet_range(name)
        .map_err(|e| format!("{:?}", e))?;

    let rows: Vec<Vec<String>> = range
        .rows()
        .map(|row| row.iter().map(cell_to_string).collect::<Vec<_>>())
        .collect();

    if rows.is_empty() {
        return Ok((Vec::new(), Vec::new()));
    }
    let headers = rows[0].clone();
    let data = rows[1..].to_vec();
    Ok((headers, data))
}

fn read_table(path: &Path) -> Result<(Vec<String>, Vec<Vec<String>>), String> {
    let ext = file_ext_lower(path);
    if ext == "csv" {
        return read_csv_table(path);
    }
    if ext == "xlsx" || ext == "xls" || ext == "xlsm" {
        return read_excel_first_sheet(path);
    }
    Err(format!(
        "{}: {}",
        t!("toolbox.excel_move.unsupported_format"),
        ext
    ))
}

pub fn read_headers(path: &Path) -> Result<Vec<String>, String> {
    let (headers, _rows) = read_table(path)?;
    Ok(headers)
}

fn cell_to_string(cell: &calamine::Data) -> String {
    use calamine::Data;
    match cell {
        Data::Empty => String::new(),
        Data::String(s) => s.clone(),
        Data::Float(f) => format!("{}", f),
        Data::Int(i) => format!("{}", i),
        Data::Bool(b) => format!("{}", b),
        Data::DateTime(d) => format!("{}", d),
        Data::DateTimeIso(s) => s.clone(),
        Data::DurationIso(s) => s.clone(),
        Data::Error(_) => String::new(),
    }
}

fn normalize_suffix(s: &str) -> Option<String> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    if s.starts_with('.') {
        Some(s.to_string())
    } else {
        Some(format!(".{s}"))
    }
}

fn normalize_base_name(raw: &str) -> String {
    let mut s = raw.trim();
    if s.is_empty() {
        return String::new();
    }

    // 去掉 URL query/fragment
    if let Some((left, _)) = s.split_once('?') {
        s = left;
    }
    if let Some((left, _)) = s.split_once('#') {
        s = left;
    }

    // DOI/URL 常见格式：取最后一个 path segment，例如 10.1002/adfm.202008487 -> adfm.202008487
    let s = s.trim_end_matches('/');
    if let Some(seg) = s.rsplit('/').find(|p| !p.trim().is_empty()) {
        return seg.trim().to_string();
    }

    s.to_string()
}

fn move_file_fallback(src: &Path, dst: &Path) -> Result<(), String> {
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    match fs::rename(src, dst) {
        Ok(()) => Ok(()),
        Err(e) => {
            // 常见：跨盘/跨设备 rename 失败，退化为 copy + remove
            fs::copy(src, dst).map_err(|e2| format!("{e}; copy failed: {e2}"))?;
            fs::remove_file(src).map_err(|e2| format!("{e}; remove failed: {e2}"))?;
            Ok(())
        }
    }
}

#[derive(Clone)]
pub enum MatchStatus {
    Found {
        source: PathBuf,
        file_name: String,
    },
    Missing {
        base: String,
    },
    Duplicate {
        base: String,
        candidates: Vec<String>,
    },
}

pub fn build_match_plan(
    excel_path: &Path,
    col_header: &str,
    col_index_1based: u32,
    input_dir: &Path,
    suffixes: &[String],
) -> Result<Vec<MatchStatus>, String> {
    let (headers, rows) = read_table(excel_path)?;
    if headers.is_empty() {
        return Err(t!("toolbox.excel_move.no_headers").to_string());
    }

    let idx = if !col_header.trim().is_empty() {
        headers
            .iter()
            .position(|h| h.trim() == col_header.trim())
            .ok_or_else(|| {
                format!(
                    "{}: {}",
                    t!("toolbox.excel_move.header_not_found"),
                    col_header.trim()
                )
            })?
    } else {
        let i = col_index_1based.saturating_sub(1) as usize;
        if i >= headers.len() {
            return Err(format!(
                "{}: {}",
                t!("toolbox.excel_move.index_out_of_range"),
                col_index_1based
            ));
        }
        i
    };

    let mut statuses = Vec::new();
    for row in rows {
        let raw = row.get(idx).map(|s| s.as_str()).unwrap_or("");
        let base = normalize_base_name(raw);
        if base.is_empty() {
            continue;
        }

        let mut hits = Vec::new();
        for suf in suffixes {
            let candidate = format!("{}{suf}", base);
            let p = input_dir.join(&candidate);
            if p.is_file() {
                hits.push(candidate);
            }
        }

        match hits.len() {
            0 => statuses.push(MatchStatus::Missing { base }),
            1 => {
                let file_name = hits[0].clone();
                statuses.push(MatchStatus::Found {
                    source: input_dir.join(&file_name),
                    file_name,
                })
            }
            _ => statuses.push(MatchStatus::Duplicate {
                base,
                candidates: hits,
            }),
        }
    }

    Ok(statuses)
}

pub struct ExcelMoveState {
    pub excel_path: Option<PathBuf>,
    pub headers: Vec<String>,
    pub input_dir: Option<PathBuf>,
    pub output_dir: Option<PathBuf>,
    pub col_header_input: Entity<gpui_component::input::InputState>,
    pub col_index_input: Entity<gpui_component::input::InputState>,
    pub suffixes_input: Entity<gpui_component::input::InputState>,
    pub preview: Option<String>,
    pub ready_count: usize,
    pub loading: bool,
    pub message: Option<String>,
    pub message_ok: bool,
}

impl ExcelMoveState {
    pub fn new(window: &mut Window, cx: &mut Context<ToolboxPanel>) -> Self {
        let col_header_input = cx.new(|cx| {
            let mut s = gpui_component::input::InputState::new(window, cx);
            s.set_placeholder(
                SharedString::from(t!("toolbox.excel_move.placeholder_header")),
                window,
                cx,
            );
            s
        });
        let col_index_input = cx.new(|cx| {
            let mut s = gpui_component::input::InputState::new(window, cx);
            s.set_placeholder(
                SharedString::from(t!("toolbox.excel_move.placeholder_index")),
                window,
                cx,
            );
            s.set_value(SharedString::from("1"), window, cx);
            s
        });
        let suffixes_input = cx.new(|cx| {
            let mut s = gpui_component::input::InputState::new(window, cx);
            s.set_placeholder(
                SharedString::from(t!("toolbox.excel_move.placeholder_suffixes")),
                window,
                cx,
            );
            s
        });

        Self {
            excel_path: None,
            headers: Vec::new(),
            input_dir: None,
            output_dir: None,
            col_header_input,
            col_index_input,
            suffixes_input,
            preview: None,
            ready_count: 0,
            loading: false,
            message: None,
            message_ok: true,
        }
    }

    pub fn render(
        state: &mut ExcelMoveState,
        entity: Entity<ToolboxPanel>,
        _window: &mut Window,
        cx: &mut Context<ToolboxPanel>,
    ) -> impl IntoElement {
        let theme = cx.theme();
        let mono = theme.mono_font_family.clone();

        let excel_label = state
            .excel_path
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        let excel_empty = excel_label.is_empty();

        let input_label = state
            .input_dir
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| t!("toolbox.excel_move.no_input_dir").to_string());
        let output_label = state
            .output_dir
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| t!("toolbox.excel_move.no_output_dir").to_string());

        let pick_excel_entity = entity.clone();
        let row_excel = h_flex()
            .gap_3()
            .items_center()
            .w_full()
            .child(
                Button::new("excel-move-pick-excel")
                    .label(t!("toolbox.excel_move.select_excel").to_string())
                    .icon(Icon::new(IconName::File).text_color(theme.blue))
                    .outline()
                    .disabled(state.loading)
                    .on_click(move |_, _, cx| {
                        pick_excel_entity.update(cx, |this, cx| this.pick_excel_move_file(cx));
                    }),
            )
            .child(
                Label::new(if excel_empty {
                    t!("toolbox.excel_move.no_excel").to_string()
                } else {
                    excel_label
                })
                .text_sm()
                .text_color(if excel_empty {
                    theme.muted_foreground
                } else {
                    theme.foreground
                })
                .overflow_hidden()
                .whitespace_nowrap()
                .truncate()
                .flex_1(),
            );

        let selected_header = state.col_header(cx).trim().to_string();
        let selected_index = state.col_index_1based(cx);
        let headers_block = if !state.headers.is_empty() {
            let mut header_chips = h_flex().gap_2().flex_wrap().items_center().w_full();
            for (idx0, h) in state.headers.iter().enumerate() {
                let idx1 = (idx0 + 1) as u32;
                let label = if h.trim().is_empty() {
                    format!("{idx1}. (空表头)")
                } else {
                    format!("{idx1}. {h}")
                };
                let is_selected = (!selected_header.is_empty() && h.trim() == selected_header)
                    || (selected_header.is_empty() && idx1 == selected_index);

                let h_clone = h.clone();
                let entity_click = entity.clone();
                header_chips = header_chips.child(
                    Button::new(("excel-move-header", idx0))
                        .label(label)
                        .small()
                        .when(is_selected, |b| b.primary())
                        .when(!is_selected, |b| b.outline())
                        .on_click(move |_, window, cx| {
                            entity_click.update(cx, |this, cx| {
                                this.excel_move.col_header_input.update(cx, |s, cx| {
                                    s.set_value(SharedString::from(h_clone.clone()), window, cx);
                                });
                                this.excel_move.col_index_input.update(cx, |s, cx| {
                                    s.set_value(SharedString::from(idx1.to_string()), window, cx);
                                });
                                this.excel_move.preview = None;
                                this.excel_move.ready_count = 0;
                                this.excel_move.message = None;
                                cx.notify();
                            });
                        }),
                );
            }
            gpui::div()
                .w_full()
                .max_h(px(120.))
                .border_1()
                .border_color(theme.border)
                .rounded_md()
                .p_2()
                .overflow_y_scrollbar()
                .child(
                    v_flex()
                        .gap_2()
                        .w_full()
                        .child(
                            Label::new(t!("toolbox.excel_move.headers_click_hint").to_string())
                                .text_xs()
                                .text_color(theme.muted_foreground),
                        )
                        .child(header_chips),
                )
                .into_any_element()
        } else {
            v_flex().into_any_element()
        };

        let suffixes_input = state.suffixes_input.clone();
        let selected_col_info = if !selected_header.is_empty() {
            format!(
                "{}: {} ({})",
                t!("toolbox.excel_move.col_header"),
                selected_header,
                selected_index
            )
        } else {
            format!("{}: {}", t!("toolbox.excel_move.col_index"), selected_index)
        };
        let selected_col_row = h_flex().w_full().child(
            Label::new(selected_col_info)
                .text_xs()
                .text_color(theme.muted_foreground),
        );

        let pick_in_entity = entity.clone();
        let row_input_dir = h_flex()
            .gap_3()
            .items_center()
            .w_full()
            .child(
                Button::new("excel-move-pick-input")
                    .label(t!("toolbox.excel_move.select_input_dir").to_string())
                    .icon(Icon::new(IconName::Folder).text_color(theme.blue))
                    .outline()
                    .disabled(state.loading)
                    .on_click(move |_, _, cx| {
                        pick_in_entity.update(cx, |this, cx| this.pick_excel_move_input_dir(cx));
                    }),
            )
            .child(
                Label::new(input_label)
                    .text_sm()
                    .text_color(theme.muted_foreground)
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .truncate()
                    .flex_1(),
            );

        let pick_out_entity = entity.clone();
        let row_output_dir = h_flex()
            .gap_3()
            .items_center()
            .w_full()
            .child(
                Button::new("excel-move-pick-output")
                    .label(t!("toolbox.excel_move.select_output_dir").to_string())
                    .icon(Icon::new(IconName::Folder).text_color(theme.blue))
                    .outline()
                    .disabled(state.loading)
                    .on_click(move |_, _, cx| {
                        pick_out_entity.update(cx, |this, cx| this.pick_excel_move_output_dir(cx));
                    }),
            )
            .child(
                Label::new(output_label)
                    .text_sm()
                    .text_color(theme.muted_foreground)
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .truncate()
                    .flex_1(),
            );

        let row_suffixes = v_flex()
            .gap_2()
            .w_full()
            .child(
                Label::new(t!("toolbox.excel_move.suffixes").to_string())
                    .text_sm()
                    .text_color(theme.muted_foreground),
            )
            .child(
                gpui::div()
                    .w_full()
                    .min_h(px(120.))
                    .border_1()
                    .border_color(theme.border)
                    .rounded_md()
                    .child(
                        v_flex()
                            .px_2()
                            .py_1()
                            .child(Input::new(&suffixes_input).w_full()),
                    ),
            );

        let entity_preview = entity.clone();
        let preview_btn = Button::new("excel-move-preview")
            .label(t!("toolbox.excel_move.preview").to_string())
            .outline()
            .disabled(state.loading)
            .on_click(move |_, window, cx| {
                entity_preview.update(cx, |this, cx| this.excel_move_preview(window, cx));
            });

        let entity_exec = entity.clone();
        let can_exec = state.ready_count > 0 && !state.loading;
        let exec_btn = Button::new("excel-move-exec")
            .label(t!("toolbox.excel_move.execute").to_string())
            .primary()
            .disabled(!can_exec)
            .on_click(move |_, window, cx| {
                entity_exec.update(cx, |this, cx| this.excel_move_execute(window, cx));
            });

        let preview_block = if let Some(ref s) = state.preview {
            let mut preview_lines = v_flex().gap_1().w_full();
            for line in s.lines() {
                preview_lines = preview_lines.child(
                    Label::new(line.to_string())
                        .text_xs()
                        .text_color(theme.muted_foreground)
                        .font_family(mono.clone()),
                );
            }
            v_flex()
                .gap_2()
                .w_full()
                .child(
                    Label::new(t!("toolbox.excel_move.preview_title").to_string())
                        .text_sm()
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_color(theme.foreground),
                )
                .child(
                    gpui::div()
                        .w_full()
                        .max_h(px(260.))
                        .border_1()
                        .border_color(theme.border)
                        .rounded_md()
                        .p_2()
                        .overflow_y_scrollbar()
                        .child(preview_lines),
                )
                .into_any_element()
        } else {
            v_flex().into_any_element()
        };

        let msg = state.message.clone();
        let msg_ok = state.message_ok;
        let msg_row = msg.map(|m| {
            let mut msg_lines = v_flex().gap_1();
            for line in m.lines() {
                msg_lines = msg_lines.child(Label::new(line.to_string()).text_sm().text_color(
                    if msg_ok {
                        theme.foreground
                    } else {
                        theme.danger
                    },
                ));
            }
            h_flex()
                .gap_2()
                .items_start()
                .pt_2()
                .child(
                    Icon::new(IconName::Info)
                        .text_color(if msg_ok { theme.green } else { theme.danger })
                        .size_12(),
                )
                .child(msg_lines)
                .into_any_element()
        });

        let content = GroupBox::new()
            .with_variant(GroupBoxVariant::Outline)
            .title(t!("toolbox.tools.excel_move_files").to_string())
            .gap_4()
            .child(row_excel)
            .child(headers_block)
            .child(selected_col_row)
            .child(row_input_dir)
            .child(row_output_dir)
            .child(row_suffixes)
            .child(
                h_flex()
                    .gap_3()
                    .items_center()
                    .child(preview_btn)
                    .child(exec_btn),
            )
            .child(preview_block);

        let mut root = v_flex()
            .size_full()
            .overflow_hidden()
            .gap_2()
            .child(content);
        if let Some(row) = msg_row {
            root = root.child(row);
        }
        root
    }

    pub fn parse_suffixes(&self, cx: &Context<ToolboxPanel>) -> Vec<String> {
        self.suffixes_input
            .read(cx)
            .value()
            .to_string()
            // 支持逗号/分号（中英文）分隔；也兼容空格与换行
            .split(|c: char| matches!(c, ',' | '，' | ';' | '；' | '\n' | '\r' | '\t' | ' '))
            .filter_map(normalize_suffix)
            .collect()
    }

    pub fn col_index_1based(&self, cx: &Context<ToolboxPanel>) -> u32 {
        self.col_index_input
            .read(cx)
            .value()
            .to_string()
            .trim()
            .parse::<u32>()
            .unwrap_or(1)
            .max(1)
    }

    pub fn col_header(&self, cx: &Context<ToolboxPanel>) -> String {
        self.col_header_input.read(cx).value().to_string()
    }
}

pub fn preview_text(statuses: &[MatchStatus], output_dir: &Path) -> (usize, String) {
    let mut found = 0usize;
    let mut missing = 0usize;
    let mut dup = 0usize;
    let mut lines = Vec::new();

    for s in statuses {
        match s {
            MatchStatus::Found { source, file_name } => {
                found += 1;
                let dst = output_dir.join(file_name);
                lines.push(format!(
                    "[OK] {}  →  {}",
                    source.file_name().and_then(|n| n.to_str()).unwrap_or(""),
                    dst.file_name().and_then(|n| n.to_str()).unwrap_or("")
                ));
            }
            MatchStatus::Missing { .. } => missing += 1,
            MatchStatus::Duplicate { .. } => dup += 1,
        }
    }

    let head = format!(
        "{} {} | {} {} | {} {} | {} {}",
        t!("toolbox.excel_move.preview_found_prefix"),
        found,
        t!("toolbox.excel_move.preview_total_prefix"),
        statuses.len(),
        t!("toolbox.excel_move.preview_missing_prefix"),
        missing,
        t!("toolbox.excel_move.preview_dup_prefix"),
        dup
    );
    (found, format!("{head}\n{}", lines.join("\n")))
}

pub fn apply_move(
    statuses: &[MatchStatus],
    output_dir: &Path,
    suffixes: &[String],
) -> (usize, Vec<String>) {
    let mut ok = 0usize;
    let mut errs = Vec::new();
    let mut used_targets: HashSet<PathBuf> = HashSet::new();

    for s in statuses {
        let (src, file_name) = match s {
            MatchStatus::Found { source, file_name } => (source, file_name),
            _ => continue,
        };
        let dst = output_dir.join(file_name);
        if !used_targets.insert(dst.clone()) {
            errs.push(format!(
                "{}: {}",
                t!("toolbox.excel_move.duplicate_output"),
                dst.display()
            ));
            continue;
        }
        if dst.exists() {
            errs.push(format!(
                "{}: {}",
                t!("toolbox.excel_move.output_exists"),
                dst.display()
            ));
            continue;
        }
        if !src.is_file() {
            errs.push(format!(
                "{}: {}",
                t!("toolbox.excel_move.source_missing"),
                src.display()
            ));
            continue;
        }
        if let Err(e) = move_file_fallback(src, &dst) {
            errs.push(format!("{} → {}: {e}", src.display(), dst.display()));
        } else {
            ok += 1;
        }
    }

    if suffixes.is_empty() {
        errs.push(t!("toolbox.excel_move.no_suffixes").to_string());
    }

    (ok, errs)
}
