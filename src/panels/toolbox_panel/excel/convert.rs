#![allow(dead_code)]
//! CSV/Excel 转 JSON 或 SQL：选择文件、选择输出格式、选择输出路径后转换。

use std::path::{Path, PathBuf};

use gpui::{
    AppContext as _, Context, Entity, IntoElement, ParentElement as _,
    Styled, Window, prelude::FluentBuilder as _, px,
};
use gpui_component::{
    ActiveTheme, Disableable, Icon, IconName,
    button::{Button, ButtonVariants as _},
    group_box::{GroupBox, GroupBoxVariant, GroupBoxVariants as _},
    h_flex,
    label::Label,
    v_flex,
};
use rust_i18n::t;

use super::super::ToolboxPanel;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ConvertFormat {
    Json,
    Sql,
}

pub struct CsvConvertState {
    pub selected_file: Option<PathBuf>,
    pub output_format: ConvertFormat,
    pub output_path: Option<PathBuf>,
    pub loading: bool,
    pub message: Option<String>,
    pub message_ok: bool,
}

impl CsvConvertState {
    pub fn new() -> Self {
        Self {
            selected_file: None,
            output_format: ConvertFormat::Json,
            output_path: None,
            loading: false,
            message: None,
            message_ok: true,
        }
    }

    pub fn render(
        state: &mut CsvConvertState,
        entity: Entity<ToolboxPanel>,
        _window: &mut Window,
        cx: &mut Context<ToolboxPanel>,
    ) -> impl IntoElement {
        let theme = cx.theme();

        let file_label = state
            .selected_file
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        let file_empty = file_label.is_empty();
        let out_label = state
            .output_path
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .map(String::from)
            .unwrap_or_else(|| t!("toolbox.csv_convert.no_output").to_string());

        let entity_file = entity.clone();
        let row_file = h_flex()
            .gap_3()
            .items_center()
            .w_full()
            .child(
                Button::new("csv-convert-pick-file")
                    .label(t!("toolbox.csv_convert.select_file").to_string())
                    .icon(Icon::new(IconName::File).text_color(theme.blue))
                    .outline()
                    .on_click(move |_, _, cx| {
                        entity_file.update(cx, |this, cx| this.pick_convert_file(cx));
                    }),
            )
            .child(
                Label::new(if file_empty {
                    t!("toolbox.csv_convert.no_file").to_string()
                } else {
                    file_label
                })
                .text_sm()
                .text_color(if file_empty {
                    theme.muted_foreground
                } else {
                    theme.foreground
                })
                .overflow_hidden()
                .whitespace_nowrap()
                .truncate()
                .flex_1(),
            );

        let format_json = state.output_format == ConvertFormat::Json;
        let entity_fmt_json = entity.clone();
        let entity_fmt_sql = entity.clone();
        let row_format = h_flex()
            .gap_3()
            .items_center()
            .w_full()
            .child(
                Label::new(t!("toolbox.csv_convert.output_format").to_string())
                    .text_sm()
                    .w(px(100.)),
            )
            .child(
                h_flex()
                    .gap_2()
                    .child(
                        Button::new("format-json")
                            .label("JSON")
                            .when(format_json, |b| b.primary())
                            .when(!format_json, |b| b.outline())
                            .on_click(move |_, _, cx| {
                                entity_fmt_json.update(cx, |this, cx| {
                                    this.csv_convert.output_format = ConvertFormat::Json;
                                    cx.notify();
                                });
                            }),
                    )
                    .child(
                        Button::new("format-sql")
                            .label("SQL")
                            .when(!format_json, |b| b.primary())
                            .when(format_json, |b| b.outline())
                            .on_click(move |_, _, cx| {
                                entity_fmt_sql.update(cx, |this, cx| {
                                    this.csv_convert.output_format = ConvertFormat::Sql;
                                    cx.notify();
                                });
                            }),
                    ),
            );

        let entity_out = entity.clone();
        let row_out = h_flex()
            .gap_3()
            .items_center()
            .w_full()
            .child(
                Button::new("csv-convert-pick-out")
                    .label(t!("toolbox.csv_convert.output_file").to_string())
                    .icon(Icon::new(IconName::Folder).text_color(theme.blue))
                    .outline()
                    .on_click(move |_, _, cx| {
                        entity_out.update(cx, |this, cx| this.pick_convert_output(cx));
                    }),
            )
            .child(
                Label::new(out_label)
                    .text_sm()
                    .text_color(theme.muted_foreground)
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .truncate()
                    .flex_1(),
            );

        let entity_exec = entity.clone();
        let execute_btn = Button::new("csv-convert-execute")
            .label(t!("toolbox.csv_convert.execute").to_string())
            .primary()
            .disabled(state.loading)
            .on_click(move |_, _, cx| {
                entity_exec.update(cx, |this, cx| this.execute_convert(cx));
            });

        let msg = state.message.clone();
        let msg_ok = state.message_ok;

        let content = GroupBox::new()
            .with_variant(GroupBoxVariant::Outline)
            .title(t!("toolbox.tools.csv_convert").to_string())
            .gap_4()
            .child(row_file)
            .child(row_format)
            .child(row_out)
            .child(execute_btn)
            .when(msg.is_some(), |this| {
                let m = msg.as_deref().unwrap_or("");
                this.child(Label::new(m.to_string()).text_sm().text_color(if msg_ok {
                    theme.green
                } else {
                    theme.danger
                }))
            });

        v_flex()
            .size_full()
            .overflow_hidden()
            .child(gpui::div().size_full().overflow_hidden().child(content))
    }
}

/// 从 CSV 或 Excel 读取为 (headers, rows)
fn read_sheet(path: &Path) -> Result<(Vec<String>, Vec<Vec<String>>), String> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();

    if ext == "csv" {
        read_csv(path)
    } else if ext == "xlsx" || ext == "xls" || ext == "xlsm" {
        read_excel(path)
    } else {
        Err(format!(
            "不支持的文件格式: {}，请选择 .csv / .xlsx / .xls",
            ext
        ))
    }
}

fn read_csv(path: &Path) -> Result<(Vec<String>, Vec<Vec<String>>), String> {
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

fn read_excel(path: &Path) -> Result<(Vec<String>, Vec<Vec<String>>), String> {
    use calamine::{Reader, open_workbook_auto};
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

fn to_json(headers: &[String], rows: &[Vec<String>]) -> Result<String, String> {
    let arr: Vec<serde_json::Value> = rows
        .iter()
        .map(|row| {
            let obj: std::collections::HashMap<String, serde_json::Value> = headers
                .iter()
                .zip(row.iter())
                .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
                .collect();
            serde_json::Value::Object(
                obj.into_iter()
                    .map(|(k, v)| (k, v))
                    .collect::<serde_json::Map<_, _>>(),
            )
        })
        .collect();
    serde_json::to_string_pretty(&arr).map_err(|e| e.to_string())
}

fn escape_sql(s: &str) -> String {
    s.replace('\\', "\\\\").replace('\'', "''")
}

fn to_sql(headers: &[String], rows: &[Vec<String>], table: &str) -> String {
    if headers.is_empty() || rows.is_empty() {
        return format!("-- 无数据\n");
    }
    let cols = headers.join(", ");
    let mut out = format!("INSERT INTO {} ({}) VALUES\n", table, cols);
    let values: Vec<String> = rows
        .iter()
        .map(|row| {
            let vals = row
                .iter()
                .map(|v| format!("'{}'", escape_sql(v)))
                .collect::<Vec<_>>()
                .join(", ");
            format!("({})", vals)
        })
        .collect();
    out.push_str(&values.join(",\n"));
    out.push_str(";\n");
    out
}

pub fn do_convert(
    input_path: &Path,
    output_path: &Path,
    format: ConvertFormat,
) -> Result<(), String> {
    let (headers, rows) = read_sheet(input_path)?;
    if headers.is_empty() {
        return Err("表头为空".to_string());
    }

    let content = match format {
        ConvertFormat::Json => to_json(&headers, &rows)?,
        ConvertFormat::Sql => {
            let table = input_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("data");
            to_sql(&headers, &rows, table)
        }
    };

    std::fs::write(output_path, content).map_err(|e| e.to_string())?;
    Ok(())
}
