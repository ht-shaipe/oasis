//! JSON 转 CSV 或 Excel：读取 JSON 文件，按指定 JSON Path 提取数组，转换输出。

use std::path::{Path, PathBuf};

use gpui::{
    prelude::FluentBuilder as _, px, AppContext as _, Context, Entity, InteractiveElement as _,
    IntoElement, ParentElement as _, Styled, Window,
};
use gpui_component::{
    button::{Button, ButtonVariants as _},
    group_box::{GroupBox, GroupBoxVariant, GroupBoxVariants as _},
    h_flex,
    input::Input,
    label::Label,
    v_flex, ActiveTheme, Disableable, Icon, IconName,
};
use rust_i18n::t;

use super::ToolboxPanel;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum JsonOutputFormat {
    Csv,
    Excel,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum JsonConvertMode {
    SingleFile,
    BatchDir,
}

pub struct JsonConvertState {
    pub mode: JsonConvertMode,
    // 单文件模式
    pub selected_file: Option<PathBuf>,
    pub output_path: Option<PathBuf>,
    // 批量模式
    pub input_dir: Option<PathBuf>,
    pub output_dir: Option<PathBuf>,
    // 共用
    pub json_path: Entity<gpui_component::input::InputState>,
    pub fields_input: Entity<gpui_component::input::InputState>,
    pub output_format: JsonOutputFormat,
    pub loading: bool,
    pub message: Option<String>,
    pub message_ok: bool,
}

impl JsonConvertState {
    pub fn new(window: &mut Window, cx: &mut Context<ToolboxPanel>) -> Self {
        let json_path = cx.new(|cx| {
            let mut s = gpui_component::input::InputState::new(window, cx);
            s.set_placeholder("data.goodsList", window, cx);
            s
        });

        let fields_input = cx.new(|cx| {
            let mut s = gpui_component::input::InputState::new(window, cx);
            s.set_placeholder("name,price,stock", window, cx);
			s.set_value("catalogName,brandName,goodsName,goodsId,goodsImage,marketPrice,goodsPrice,marketPriceUrl", window, cx);
            s
        });

        Self {
            mode: JsonConvertMode::SingleFile,
            selected_file: None,
            output_path: None,
            input_dir: None,
            output_dir: None,
            json_path,
            fields_input,
            output_format: JsonOutputFormat::Csv,
            loading: false,
            message: None,
            message_ok: true,
        }
    }

    pub fn render(
        state: &mut JsonConvertState,
        entity: Entity<ToolboxPanel>,
        _window: &mut Window,
        cx: &mut Context<ToolboxPanel>,
    ) -> impl IntoElement {
        let theme = cx.theme();

        // ── 模式切换行 ──
        let is_single = state.mode == JsonConvertMode::SingleFile;
        let entity_mode_single = entity.clone();
        let entity_mode_batch = entity.clone();
        let row_mode = h_flex()
            .gap_3()
            .items_center()
            .w_full()
            .child(
                Label::new(t!("toolbox.json_convert.mode").to_string())
                    .text_sm()
                    .w(px(100.)),
            )
            .child(
                h_flex().gap_2().child(
                    Button::new("json-mode-single")
                        .label(t!("toolbox.json_convert.mode_single").to_string())
                        .when(is_single, |b| b.primary())
                        .when(!is_single, |b| b.outline())
                        .on_click(move |_, _, cx| {
                            entity_mode_single.update(cx, |this, cx| {
                                this.json_convert.mode = JsonConvertMode::SingleFile;
                                this.json_convert.message = None;
                                cx.notify();
                            });
                        }),
                ).child(
                    Button::new("json-mode-batch")
                        .label(t!("toolbox.json_convert.mode_batch").to_string())
                        .when(!is_single, |b| b.primary())
                        .when(is_single, |b| b.outline())
                        .on_click(move |_, _, cx| {
                            entity_mode_batch.update(cx, |this, cx| {
                                this.json_convert.mode = JsonConvertMode::BatchDir;
                                this.json_convert.message = None;
                                cx.notify();
                            });
                        }),
                ),
            );

        // ── 共用行：JSON Path ──
        let json_path_entity = state.json_path.clone();
        let row_path = h_flex()
            .gap_3()
            .items_center()
            .w_full()
            .child(
                Label::new(t!("toolbox.json_convert.json_path").to_string())
                    .text_sm()
                    .w(px(100.)),
            )
            .child(
                gpui::div()
                    .flex_1()
                    .child(Input::new(&json_path_entity).cleanable(true)),
            );

        // ── 共用行：保留字段 ──
        let fields_entity = state.fields_input.clone();
        let row_fields = h_flex()
            .gap_3()
            .items_center()
            .w_full()
            .child(
                Label::new(t!("toolbox.json_convert.fields").to_string())
                    .text_sm()
                    .w(px(100.)),
            )
            .child(
                gpui::div()
                    .flex_1()
                    .child(Input::new(&fields_entity).cleanable(true)),
            );

        // ── 共用行：输出格式 ──
        let format_csv = state.output_format == JsonOutputFormat::Csv;
        let entity_fmt_csv = entity.clone();
        let entity_fmt_excel = entity.clone();
        let row_format = h_flex()
            .gap_3()
            .items_center()
            .w_full()
            .child(
                Label::new(t!("toolbox.json_convert.output_format").to_string())
                    .text_sm()
                    .w(px(100.)),
            )
            .child(
                h_flex().gap_2().child(
                    Button::new("format-json-csv")
                        .label("CSV")
                        .when(format_csv, |b| b.primary())
                        .when(!format_csv, |b| b.outline())
                        .on_click(move |_, _, cx| {
                            entity_fmt_csv.update(cx, |this, cx| {
                                this.json_convert.output_format = JsonOutputFormat::Csv;
                                cx.notify();
                            });
                        }),
                ).child(
                    Button::new("format-json-excel")
                        .label("Excel")
                        .when(!format_csv, |b| b.primary())
                        .when(format_csv, |b| b.outline())
                        .on_click(move |_, _, cx| {
                            entity_fmt_excel.update(cx, |this, cx| {
                                this.json_convert.output_format = JsonOutputFormat::Excel;
                                cx.notify();
                            });
                        }),
                ),
            );

        // ── 根据模式渲染不同内容 ──
        let mode_content = if is_single {
            // 单文件模式
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
                .unwrap_or_else(|| t!("toolbox.json_convert.no_output").to_string());

            let entity_file = entity.clone();
            let row_file = h_flex()
                .gap_3()
                .items_center()
                .w_full()
                .child(
                    Button::new("json-convert-pick-file")
                        .label(t!("toolbox.json_convert.select_file").to_string())
                        .icon(Icon::new(IconName::File).text_color(theme.blue))
                        .outline()
                        .on_click(move |_, _, cx| {
                            entity_file.update(cx, |this, cx| this.pick_json_convert_file(cx));
                        }),
                )
                .child(
                    Label::new(if file_empty {
                        t!("toolbox.json_convert.no_file").to_string()
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

            let entity_out = entity.clone();
            let row_out = h_flex()
                .gap_3()
                .items_center()
                .w_full()
                .child(
                    Button::new("json-convert-pick-out")
                        .label(t!("toolbox.json_convert.output_file").to_string())
                        .icon(Icon::new(IconName::Folder).text_color(theme.blue))
                        .outline()
                        .on_click(move |_, _, cx| {
                            entity_out.update(cx, |this, cx| this.pick_json_convert_output(cx));
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
            let execute_btn = Button::new("json-convert-execute")
                .label(t!("toolbox.json_convert.execute").to_string())
                .primary()
                .disabled(state.loading)
                .on_click(move |_, _, cx| {
                    entity_exec.update(cx, |this, cx| this.execute_json_convert(cx));
                });

            v_flex()
                .gap_4()
                .child(row_file)
                .child(row_out)
                .child(execute_btn)
                .into_any_element()
        } else {
            // 批量目录模式
            let input_label = state
                .input_dir
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| t!("toolbox.json_convert.no_input_dir").to_string());

            let output_label = state
                .output_dir
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| t!("toolbox.json_convert.no_output_dir").to_string());

            let entity_input = entity.clone();
            let row_input = h_flex()
                .gap_3()
                .items_center()
                .w_full()
                .child(
                    Button::new("json-batch-pick-input")
                        .label(t!("toolbox.json_convert.input_dir").to_string())
                        .icon(Icon::new(IconName::Folder).text_color(theme.blue))
                        .outline()
                        .on_click(move |_, _, cx| {
                            entity_input.update(cx, |this, cx| this.pick_json_batch_input_dir(cx));
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

            let entity_output = entity.clone();
            let row_output = h_flex()
                .gap_3()
                .items_center()
                .w_full()
                .child(
                    Button::new("json-batch-pick-output")
                        .label(t!("toolbox.json_convert.output_dir").to_string())
                        .icon(Icon::new(IconName::Folder).text_color(theme.blue))
                        .outline()
                        .on_click(move |_, _, cx| {
                            entity_output.update(cx, |this, cx| this.pick_json_batch_output_dir(cx));
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

            let entity_exec = entity.clone();
            let execute_btn = Button::new("json-batch-execute")
                .label(t!("toolbox.json_convert.batch_execute").to_string())
                .primary()
                .disabled(state.loading)
                .on_click(move |_, _, cx| {
                    entity_exec.update(cx, |this, cx| this.execute_json_batch_convert(cx));
                });

            v_flex()
                .gap_4()
                .child(row_input)
                .child(row_output)
                .child(execute_btn)
                .into_any_element()
        };

        let msg = state.message.clone();
        let msg_ok = state.message_ok;

        let content = GroupBox::new()
            .with_variant(GroupBoxVariant::Outline)
            .title(t!("toolbox.tools.json_convert").to_string())
            .gap_4()
            .child(row_mode)
            .child(row_path)
            .child(row_fields)
            .child(row_format)
            .child(mode_content)
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

/// 按 dot-separated path（如 "data.goodsList"）从 JSON Value 中提取子节点。
/// 如果 path 为空则返回根节点本身。
fn resolve_json_path<'a>(root: &'a serde_json::Value, path: &str) -> Option<&'a serde_json::Value> {
    if path.is_empty() {
        return Some(root);
    }
    let mut current = root;
    for key in path.split('.') {
        let key = key.trim();
        if key.is_empty() {
            continue;
        }
        match current {
            serde_json::Value::Object(map) => {
                current = map.get(key)?;
            }
            serde_json::Value::Array(arr) => {
                let idx: usize = key.parse().ok()?;
                current = arr.get(idx)?;
            }
            _ => return None,
        }
    }
    Some(current)
}

/// 从 JSON 数组中提取 (headers, rows)。
/// 如果 `fields` 非空，则只保留指定的字段（按 fields 顺序排列）。
/// 如果 `fields` 为空，headers 取所有对象的 key 并集（保持首次出现顺序）。
fn extract_table(arr: &[serde_json::Value], fields: &[String]) -> (Vec<String>, Vec<Vec<String>>) {
    let headers: Vec<String> = if fields.is_empty() {
        let mut headers_ordered: Vec<String> = Vec::new();
        let mut header_set = std::collections::HashSet::new();

        for item in arr {
            if let serde_json::Value::Object(map) = item {
                for key in map.keys() {
                    if header_set.insert(key.clone()) {
                        headers_ordered.push(key.clone());
                    }
                }
            }
        }
        headers_ordered
    } else {
        fields.to_vec()
    };

    if headers.is_empty() {
        return (Vec::new(), Vec::new());
    }

    let rows: Vec<Vec<String>> = arr
        .iter()
        .map(|item| {
            headers
                .iter()
                .map(|key| match item.get(key) {
                    Some(serde_json::Value::Null) => String::new(),
                    Some(serde_json::Value::String(s)) => s.clone(),
                    Some(serde_json::Value::Number(n)) => n.to_string(),
                    Some(serde_json::Value::Bool(b)) => b.to_string(),
                    Some(v) => serde_json::to_string(v).unwrap_or_default(),
                    None => String::new(),
                })
                .collect()
        })
        .collect();

    (headers, rows)
}

/// 将 (headers, rows) 写入 CSV 文件
fn write_csv(output_path: &Path, headers: &[String], rows: &[Vec<String>]) -> Result<(), String> {
    let mut writer = csv::WriterBuilder::new()
        .from_path(output_path)
        .map_err(|e| e.to_string())?;
    writer.write_record(headers).map_err(|e| e.to_string())?;
    for row in rows {
        writer.write_record(row).map_err(|e| e.to_string())?;
    }
    writer.flush().map_err(|e| e.to_string())?;
    Ok(())
}

/// 将 (headers, rows) 写入 Excel (.xlsx) 文件
fn write_excel(output_path: &Path, headers: &[String], rows: &[Vec<String>]) -> Result<(), String> {
    use rust_xlsxwriter::*;

    let path_str = output_path.to_string_lossy();
    let mut workbook = Workbook::new(&path_str);
    let worksheet = workbook.add_worksheet();
    let header_format = Format::new().set_bold();

    for (col, header) in headers.iter().enumerate() {
        let col_u16 = u16::try_from(col).map_err(|_| "列数超出限制")?;
        worksheet
            .write_string(0, col_u16, header, &header_format)
            .map_err(|e| e.to_string())?;
    }

    for (row_idx, row) in rows.iter().enumerate() {
        let row_u32 = u32::try_from(row_idx + 1).map_err(|_| "行数超出限制")?;
        for (col, val) in row.iter().enumerate() {
            let col_u16 = u16::try_from(col).map_err(|_| "列数超出限制")?;
            worksheet
                .write_string(row_u32, col_u16, val, &Format::new())
                .map_err(|e| e.to_string())?;
        }
    }

    workbook.close().map_err(|e| e.to_string())?;
    Ok(())
}

/// 执行 JSON → CSV/Excel 转换
pub fn do_json_convert(
    input_path: &Path,
    output_path: &Path,
    json_path: &str,
    fields: &[String],
    format: JsonOutputFormat,
) -> Result<(), String> {
    let raw = std::fs::read_to_string(input_path).map_err(|e| e.to_string())?;
    let root: serde_json::Value = serde_json::from_str(&raw).map_err(|e| {
        format!(
            "{}: {}",
            t!("toolbox.json_convert.invalid_json").to_string(),
            e
        )
    })?;

    let target = resolve_json_path(&root, json_path).ok_or_else(|| {
        format!(
            "{}: {}",
            t!("toolbox.json_convert.path_not_found").to_string(),
            if json_path.is_empty() {
                "(root)".to_string()
            } else {
                json_path.to_string()
            }
        )
    })?;

    let arr = match target {
        serde_json::Value::Array(a) => a,
        other => {
            return Err(format!(
                "{} (got {})",
                t!("toolbox.json_convert.not_array").to_string(),
                json_type_name(other)
            ));
        }
    };

    if arr.is_empty() {
        return Err(t!("toolbox.json_convert.empty_array").to_string());
    }

    let (headers, rows) = extract_table(arr, fields);
    if headers.is_empty() {
        return Err(t!("toolbox.json_convert.no_headers").to_string());
    }

    match format {
        JsonOutputFormat::Csv => write_csv(output_path, &headers, &rows),
        JsonOutputFormat::Excel => write_excel(output_path, &headers, &rows),
    }
}

fn json_type_name(v: &serde_json::Value) -> &'static str {
    match v {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "boolean",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::String(_) => "string",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::Object(_) => "object",
    }
}

/// 扫描目录下的所有 .json 文件并批量转换。
/// 返回 (成功数, 失败文件列表)
pub fn do_batch_json_convert(
    input_dir: &Path,
    output_dir: &Path,
    json_path: &str,
    fields: &[String],
    format: JsonOutputFormat,
) -> Result<(usize, Vec<String>), String> {
    if !input_dir.exists() {
        return Err(t!("toolbox.json_convert.no_input_dir").to_string());
    }
    if !output_dir.exists() {
        std::fs::create_dir_all(output_dir).map_err(|e| e.to_string())?;
    }

    let entries = std::fs::read_dir(input_dir).map_err(|e| e.to_string())?;
    let json_files: Vec<PathBuf> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.eq_ignore_ascii_case("json"))
                .unwrap_or(false)
        })
        .collect();

    if json_files.is_empty() {
        return Err(t!("toolbox.json_convert.no_json_files").to_string());
    }

    let ext = match format {
        JsonOutputFormat::Csv => "csv",
        JsonOutputFormat::Excel => "xlsx",
    };

    let mut ok = 0usize;
    let mut errors: Vec<String> = Vec::new();

    for input_path in &json_files {
        let stem = input_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");
        let output_path = output_dir.join(format!("{}.{}", stem, ext));

        match do_json_convert(input_path, &output_path, json_path, fields, format) {
            Ok(()) => ok += 1,
            Err(e) => {
                let file_name = input_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("?");
                errors.push(format!("{}: {}", file_name, e));
            }
        }
    }

    Ok((ok, errors))
}
