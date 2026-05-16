//! CSV 文件统计工具：扫描目录下 CSV 并统计行数。

use std::path::{Path, PathBuf};

use gpui::{
    App, AppContext as _, ClipboardItem, Context, Entity, InteractiveElement as _, IntoElement,
    ParentElement as _, Styled, Window, prelude::FluentBuilder as _, px,
};
use gpui_component::{
    ActiveTheme, Icon, IconName,
    button::{Button, ButtonVariants as _},
    group_box::{GroupBox, GroupBoxVariant, GroupBoxVariants as _},
    h_flex,
    label::Label,
    table::{Column, ColumnSort, Table, TableDelegate, TableState},
};
use rust_i18n::t;


use super::super::ToolboxPanel;

#[derive(Clone, Debug)]
pub struct CsvEntry {
    pub name: String,
    pub lines: u64,
}

pub struct CsvTableDelegate {
    pub entries: Vec<CsvEntry>,
    columns: Vec<Column>,
}

impl CsvTableDelegate {
    pub fn new(entries: Vec<CsvEntry>) -> Self {
        let columns = vec![
            Column::new("index", "#".to_string())
                .width(px(60.)),
            Column::new("name", t!("toolbox.csv.col_name").to_string())
                .width(px(400.))
                .sortable(),
            Column::new("lines", t!("toolbox.csv.col_lines").to_string())
                .width(px(100.))
                .sortable()
                .text_right(),
        ];
        Self { entries, columns }
    }

    pub fn set_entries(&mut self, entries: Vec<CsvEntry>) {
        self.entries = entries;
    }
}

impl gpui_component::table::TableDelegate for CsvTableDelegate {
    fn columns_count(&self, _cx: &App) -> usize {
        3
    }

    fn rows_count(&self, _cx: &App) -> usize {
        self.entries.len()
    }

    fn column(&self, col_ix: usize, _cx: &App) -> &Column {
        match col_ix {
            // 第 0 列：序号列，固定较小宽度
            0 => &self.columns[0],
            // 第 1 列：文件名，列宽自动，给一个更大的最小宽度，并支持排序
            1 => &self.columns[1],
            // 第 2 列：行数
            2 => &self.columns[2],
            _ => &self.columns[0],
        }
    }

    fn render_header(
        &mut self,
        _window: &mut Window,
        _cx: &mut Context<TableState<Self>>,
    ) -> gpui::Stateful<gpui::Div> {
        gpui::div().id("header")
    }

    fn render_th(
        &mut self,
        col_ix: usize,
        _window: &mut Window,
        _cx: &mut Context<TableState<Self>>,
    ) -> impl IntoElement {
        // 这里不依赖 App，直接根据列索引构造表头标题
        let name = match col_ix {
            0 => "#".to_string(),
            1 => t!("toolbox.csv.col_name").to_string(),
            2 => t!("toolbox.csv.col_lines").to_string(),
            _ => String::new(),
        };
        gpui::div().size_full().child(
            Label::new(name)
                .text_sm()
                .font_weight(gpui::FontWeight::MEDIUM),
        )
    }

    fn perform_sort(
        &mut self,
        col_ix: usize,
        sort: ColumnSort,
        _window: &mut Window,
        _cx: &mut Context<TableState<Self>>,
    ) {
        match col_ix {
            // 名称列：按名称排序
            1 => {
                self.entries.sort_by(|a, b| match sort {
                    ColumnSort::Descending => b.name.cmp(&a.name),
                    _ => a.name.cmp(&b.name),
                });
            }
            // 行数列：按行数排序
            2 => {
                self.entries.sort_by(|a, b| match sort {
                    ColumnSort::Descending => b.lines.cmp(&a.lines),
                    _ => a.lines.cmp(&b.lines),
                });
            }
            _ => {}
        }
    }

    fn render_tr(
        &mut self,
        row_ix: usize,
        _window: &mut Window,
        _cx: &mut Context<TableState<Self>>,
    ) -> gpui::Stateful<gpui::Div> {
        gpui::div().id(("row", row_ix))
    }

    fn render_td(
        &mut self,
        row_ix: usize,
        col_ix: usize,
        _window: &mut Window,
        cx: &mut Context<TableState<Self>>,
    ) -> impl IntoElement {
        let Some(entry) = self.entries.get(row_ix) else {
            return gpui::div();
        };
        let theme = cx.theme();
        match col_ix {
            // 序号列
            0 => gpui::div()
                .size_full()
                .flex()
                .justify_end()
                .items_center()
                .child(
                    Label::new((row_ix + 1).to_string())
                        .text_sm()
                        .text_color(theme.muted_foreground),
                ),
            // 文件名列
            1 => gpui::div().size_full().child(
                Label::new(entry.name.clone())
                    .text_sm()
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .truncate(),
            ),
            // 行数列
            2 => gpui::div()
                .size_full()
                .flex()
                .justify_end()
                .items_center()
                .child(
                    Label::new(entry.lines.to_string())
                        .text_sm()
                        .text_color(theme.muted_foreground),
                ),
            _ => gpui::div(),
        }
    }
}

pub fn count_lines(path: &Path) -> u64 {
    // 优先按 CSV 记录计数，而不是按物理换行计数。
    // 这样即使字段内部包含换行（例如被双引号包裹的多行文本），统计也仍然准确。
    let Ok(file) = std::fs::File::open(path) else {
        return 0;
    };

    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(file);

    let mut count = 0u64;

    // 若存在表头，则将表头也计入总行数，保持与当前工具“总行数（含标题行）”的语义一致。
    match reader.headers() {
        Ok(headers) if !headers.is_empty() => count += 1,
        Ok(_) => return 0,
        Err(_) => return count_physical_lines(path),
    }

    for record in reader.byte_records() {
        match record {
            Ok(_) => count += 1,
            Err(_) => return count_physical_lines(path),
        }
    }

    count
}

fn count_physical_lines(path: &Path) -> u64 {
    let Ok(file) = std::fs::File::open(path) else {
        return 0;
    };
    let mut reader = std::io::BufReader::new(file);
    let mut buf = Vec::new();
    let mut count = 0u64;
    loop {
        buf.clear();
        match std::io::BufRead::read_until(&mut reader, b'\n', &mut buf) {
            Ok(0) => break,
            Ok(_) => count += 1,
            Err(_) => break,
        }
    }
    count
}

pub fn scan_csv_in_dir(dir: &Path) -> (Vec<CsvEntry>, u64) {
    let mut entries = Vec::new();
    let mut total = 0u64;

    let Ok(rd) = std::fs::read_dir(dir) else {
        return (entries, total);
    };

    for entry in rd.flatten() {
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext.eq_ignore_ascii_case("csv") {
                    let name = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("")
                        .to_string();
                    let lines = count_lines(&path);
                    total += lines;
                    entries.push(CsvEntry { name, lines });
                }
            }
        }
    }

    entries.sort_by(|a, b| a.name.cmp(&b.name));
    (entries, total)
}

pub struct CsvStatsState {
    pub selected_dir: Option<PathBuf>,
    pub csv_entries: Vec<CsvEntry>,
    pub total_lines: u64,
    pub loading: bool,
    pub table_state: Entity<TableState<CsvTableDelegate>>,
}

impl CsvStatsState {
    pub fn new(window: &mut Window, cx: &mut Context<ToolboxPanel>) -> Self {
        let delegate = CsvTableDelegate::new(Vec::new());
        let table_state = cx.new(|cx| {
            TableState::new(delegate, window, cx)
                // 启用表头排序，结合列上的 .sortable() 和 perform_sort 一起工作
                .sortable(true)
                .col_movable(false)
                .col_resizable(true)
                .row_selectable(true)
                .col_selectable(false)
        });

        Self {
            selected_dir: None,
            csv_entries: Vec::new(),
            total_lines: 0,
            loading: false,
            table_state,
        }
    }

    pub fn render(
        state: &mut CsvStatsState,
        entity: Entity<ToolboxPanel>,
        window: &mut Window,
        cx: &mut Context<ToolboxPanel>,
    ) -> impl IntoElement {
        let theme = cx.theme();

        let dir_label = state
            .selected_dir
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| t!("toolbox.csv.no_dir").to_string());

        // 为不同操作克隆实体，避免所有权冲突
        let entity_pick_dir = entity.clone();
        let entity_copy_md = entity.clone();

        let header_section = h_flex()
            .gap_3()
            .items_center()
            .w_full()
            .child(
                Button::new("toolbox-pick-dir")
                    .label(t!("toolbox.csv.select_dir").to_string())
                    .icon(Icon::new(IconName::Folder).text_color(theme.blue))
                    .outline()
                    .on_click(move |_, _, cx| {
                        entity_pick_dir.update(cx, |this, cx| this.pick_and_scan_csv_stats(cx));
                    }),
            )
            .child(
                Label::new(dir_label)
                    .text_sm()
                    .text_color(theme.muted_foreground)
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .truncate()
                    .flex_1(),
            );

        let table_state = state.table_state.clone();
        let total_lines = state.total_lines;
        let has_data = !state.loading && !state.csv_entries.is_empty();

        let content = GroupBox::new()
            .with_variant(GroupBoxVariant::Outline)
            .title(t!("toolbox.csv.section_title").to_string())
            .gap_4()
            .child(header_section)
            .when(state.loading, |this| {
                this.child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .text_sm()
                        .text_color(theme.muted_foreground)
                        .child(Icon::new(IconName::Loader).size_12())
                        .child(Label::new(t!("toolbox.csv.loading").to_string())),
                )
            })
            .when(has_data, |this| {
                this.child(
                    gpui::div()
                        .flex_1()
                        .min_h(px(320.))
                        .rounded_md()
                        .overflow_hidden()
                        .border_1()
                        .border_color(theme.border)
                        .child(Table::new(&table_state).stripe(true).bordered(true)),
                )
            })
            .when(has_data, |this| {
                this.child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .pt_3()
                        .border_t_1()
                        .border_color(theme.border)
                        .child(
                            Label::new(t!("toolbox.csv.total").to_string())
                                .text_sm()
                                .font_weight(gpui::FontWeight::MEDIUM),
                        )
                        .child(
                            Label::new(format!(
                                "{} {}",
                                total_lines,
                                t!("toolbox.csv.lines_suffix")
                            ))
                            .text_sm()
                            .text_color(theme.blue)
                            .font_weight(gpui::FontWeight::SEMIBOLD),
                        ),
                )
            })
            .when(has_data, |this| {
                // 统计下方增加“复制为 Markdown 表格”按钮
                this.child(
                    h_flex().gap_2().items_center().pt_2().child(
                        Button::new("toolbox-copy-md-table")
                            .label("复制为 Markdown 表格".to_string())
                            .ghost()
                            .on_click(move |_, _, cx| {
                                entity_copy_md.update(cx, |this, cx| {
                                    let mut rows: Vec<CsvEntry> = Vec::new();
                                    // 从当前表格 delegate 中获取已排序后的 entries
                                    this.csv_stats.table_state.update(cx, |table_state, _app| {
                                        rows = table_state.delegate_mut().entries.clone();
                                    });

                                    if rows.is_empty() {
                                        return;
                                    }

                                    // 构造 Markdown 表格
                                    let header_name = t!("toolbox.csv.col_name").to_string();
                                    let header_lines = t!("toolbox.csv.col_lines").to_string();
                                    let mut md = String::new();
                                    md.push_str(&format!(
                                        "| # | {} | {} |\n",
                                        header_name, header_lines
                                    ));
                                    md.push_str("| --- | --- | --- |\n");

                                    for (idx, entry) in rows.iter().enumerate() {
                                        let safe_name = entry.name.replace("|", "\\|");
                                        md.push_str(&format!(
                                            "| {} | {} | {} |\n",
                                            idx + 1,
                                            safe_name,
                                            entry.lines
                                        ));
                                    }

                                    cx.write_to_clipboard(ClipboardItem::new_string(md));
                                });
                            }),
                    ),
                )
            })
            .when(
                !state.loading && state.selected_dir.is_some() && state.csv_entries.is_empty(),
                |this| {
                    this.child(
                        Label::new(t!("toolbox.csv.no_csv").to_string())
                            .text_sm()
                            .text_color(theme.muted_foreground),
                    )
                },
            );

        gpui::div()
            .size_full()
            .overflow_hidden()
            .child(gpui::div().size_full().overflow_hidden().child(content))
    }
}
