#![allow(dead_code)]
//! CSV 文件统计工具：扫描目录下 CSV 并统计行数。

use std::path::{Path, PathBuf};

use gpui::{
     Context, Entity, IntoElement,
    ParentElement as _, Styled, Window, prelude::FluentBuilder as _
};
use gpui_component::{
    ActiveTheme, Icon, IconName,
    button::{Button},
    group_box::{GroupBox, GroupBoxVariant, GroupBoxVariants as _},
    h_flex,
    label::Label,
};
use rust_i18n::t;


use super::super::ToolboxPanel;

#[derive(Clone, Debug)]
pub struct CsvEntry {
    pub name: String,
    pub lines: u64,
}

// TODO: Implement CSV statistics table UI
// This requires TableDelegate trait and related types from gpui_component
// For now, we provide the basic data structures and helper functions

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

    // 若存在表头，则将表头也计入总行数，保持与当前工具"总行数（含标题行）"的语义一致。
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

// Placeholder state - TODO: Implement proper table UI
pub struct CsvStatsState {
    pub selected_dir: Option<PathBuf>,
    pub csv_entries: Vec<CsvEntry>,
    pub total_lines: u64,
    pub loading: bool,
}

impl CsvStatsState {
    pub fn new(_window: &mut Window, _cx: &mut Context<ToolboxPanel>) -> Self {
        Self {
            selected_dir: None,
            csv_entries: Vec::new(),
            total_lines: 0,
            loading: false,
        }
    }

    pub fn render(
        state: &mut CsvStatsState,
        _entity: Entity<ToolboxPanel>,
        _window: &mut Window,
        cx: &mut Context<ToolboxPanel>,
    ) -> impl IntoElement {
        let theme = cx.theme();

        let dir_label = state
            .selected_dir
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| t!("toolbox.csv.no_dir").to_string());

        let header_section = h_flex()
            .gap_3()
            .items_center()
            .w_full()
            .child(
                Button::new("toolbox-pick-dir")
                    .label(t!("toolbox.csv.select_dir").to_string())
                    .icon(Icon::new(IconName::Folder).text_color(theme.blue))
                    .outline(),
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
            .when(!state.loading && state.csv_entries.is_empty(), |this| {
                this.child(
                    Label::new(t!("toolbox.csv.no_csv").to_string())
                        .text_sm()
                        .text_color(theme.muted_foreground),
                )
            })
            .when(!state.csv_entries.is_empty(), |this| {
                // Simple list view instead of table - TODO: Implement proper table
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
                                state.total_lines,
                                t!("toolbox.csv.lines_suffix")
                            ))
                            .text_sm()
                            .text_color(theme.blue)
                            .font_weight(gpui::FontWeight::SEMIBOLD),
                        ),
                )
            });

        gpui::div()
            .size_full()
            .overflow_hidden()
            .child(gpui::div().size_full().overflow_hidden().child(content))
    }
}
