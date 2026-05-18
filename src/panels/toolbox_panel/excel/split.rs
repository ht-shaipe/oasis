//! CSV 文件分割工具：按指定份数分割，每份带标题行。

use std::path::PathBuf;

use gpui::{
    AppContext as _, Context, Entity, IntoElement, ParentElement as _,
    SharedString, Styled, Subscription, Window, prelude::FluentBuilder as _, px,
};
use gpui_component::{
    ActiveTheme, Disableable, Icon, IconName,
    button::{Button, ButtonVariants as _},
    group_box::{GroupBox, GroupBoxVariant, GroupBoxVariants as _},
    h_flex,
    input::{NumberInput, NumberInputEvent, StepAction},
    label::Label,
    v_flex,
};
use rust_i18n::t;

use super::super::ToolboxPanel;

pub struct CsvSplitState {
    pub selected_file: Option<PathBuf>,
    /// 选中文件的总行数（含标题行），选文件后由主面板回调写入
    pub total_lines: Option<u64>,
    pub output_dir: Option<PathBuf>,
    pub loading: bool,
    pub message: Option<String>,
    pub message_ok: bool,
    pub num_parts_input: Entity<gpui_component::input::InputState>,
    pub _subscriptions: Vec<Subscription>,
}

impl CsvSplitState {
    pub fn new(window: &mut Window, cx: &mut Context<ToolboxPanel>) -> Self {
        let num_parts_input = cx.new(|cx| {
            let mut s = gpui_component::input::InputState::new(window, cx);
            s.set_value(gpui::SharedString::from("2"), window, cx);
            s
        });

        let subscriptions = vec![cx.subscribe_in(&num_parts_input, window, {
            move |_, input, event: &NumberInputEvent, window, cx| match event {
                NumberInputEvent::Step(action) => input.update(cx, |input, cx| {
                    let current = input.value().to_string().trim().parse::<i64>().unwrap_or(2);
                    let next = match action {
                        StepAction::Increment => current + 1,
                        StepAction::Decrement => current - 1,
                    }
                    .clamp(1, 1000);
                    input.set_value(SharedString::from(next.to_string()), window, cx);
                }),
            }
        })];

        Self {
            selected_file: None,
            total_lines: None,
            output_dir: None,
            loading: false,
            message: None,
            message_ok: true,
            num_parts_input,
            _subscriptions: subscriptions,
        }
    }

    pub fn render(
        state: &mut CsvSplitState,
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
            .output_dir
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| t!("toolbox.csv_split.no_output_dir").to_string());

        let n_parts: u32 = state
            .num_parts_input
            .read(cx)
            .value()
            .to_string()
            .trim()
            .parse()
            .unwrap_or(2);
        let n = n_parts.max(1).min(1000) as u64;
        let total_lines = state.total_lines.unwrap_or(0);
        let data_rows = total_lines.saturating_sub(1);
        let estimated_per_file = if n > 0 && total_lines > 0 {
            1 + (data_rows + n - 1) / n
        } else {
            0u64
        };

        let entity_file = entity.clone();
        let row_file = h_flex()
            .gap_3()
            .items_center()
            .w_full()
            .child(
                Button::new("csv-split-pick-file")
                    .label(t!("toolbox.csv_split.select_file").to_string())
                    .icon(Icon::new(IconName::File).text_color(theme.blue))
                    .outline()
                    .on_click(move |_, _, cx| {
                        entity_file.update(cx, |this, cx| this.pick_csv_split_file(cx));
                    }),
            )
            .child(
                Label::new(if file_empty {
                    t!("toolbox.csv_split.no_file").to_string()
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

        let show_stats = state.total_lines.is_some();
        let total_lines_val = total_lines;
        let estimated_per_file_val = estimated_per_file;

        let entity_out = entity.clone();
        let row_out = h_flex()
            .gap_3()
            .items_center()
            .w_full()
            .child(
                Button::new("csv-split-pick-out")
                    .label(t!("toolbox.csv_split.output_dir").to_string())
                    .icon(Icon::new(IconName::Folder).text_color(theme.blue))
                    .outline()
                    .on_click(move |_, _, cx| {
                        entity_out.update(cx, |this, cx| this.pick_csv_split_output_dir(cx));
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

        let num_parts_input = state.num_parts_input.clone();
        let row_count = h_flex()
            .gap_3()
            .items_center()
            .w_full()
            .child(
                Label::new(t!("toolbox.csv_split.num_parts").to_string())
                    .text_sm()
                    .w(px(100.)),
            )
            .child(
                gpui::div()
                    .w(px(160.))
                    .child(NumberInput::new(&num_parts_input)),
            );

        let entity_exec = entity.clone();
        let execute_btn = Button::new("csv-split-execute")
            .label(t!("toolbox.csv_split.execute").to_string())
            .primary()
            .disabled(state.loading)
            .on_click(move |_, _, cx| {
                entity_exec.update(cx, |this, cx| this.execute_csv_split(cx));
            });

        let msg = state.message.clone();
        let msg_ok = state.message_ok;

        let content = GroupBox::new()
            .with_variant(GroupBoxVariant::Outline)
            .title(t!("toolbox.tools.csv_split").to_string())
            .gap_4()
            .child(row_file)
            .when(show_stats, |this| {
                this.child(
                    h_flex()
                        .gap_6()
                        .items_center()
                        .text_sm()
                        .text_color(theme.muted_foreground)
                        .child(
                            h_flex()
                                .gap_2()
                                .items_center()
                                .child(Label::new(t!("toolbox.csv_split.total_lines").to_string()))
                                .child(
                                    Label::new(total_lines_val.to_string())
                                        .font_weight(gpui::FontWeight::SEMIBOLD)
                                        .text_color(theme.foreground),
                                ),
                        )
                        .child(
                            h_flex()
                                .gap_2()
                                .items_center()
                                .child(Label::new(
                                    t!("toolbox.csv_split.estimated_per_file").to_string(),
                                ))
                                .child(
                                    Label::new(if estimated_per_file_val > 0 {
                                        estimated_per_file_val.to_string()
                                    } else {
                                        "—".to_string()
                                    })
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(theme.blue),
                                ),
                        ),
                )
            })
            .child(row_out)
            .child(row_count)
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

pub fn do_split(
    input_path: &std::path::Path,
    output_dir: &std::path::Path,
    n_parts: u32,
) -> Result<(), String> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_path(input_path)
        .map_err(|e| e.to_string())?;
    let headers = reader.headers().map_err(|e| e.to_string())?.clone();

    let mut records = Vec::new();
    for record in reader.records() {
        records.push(record.map_err(|e| e.to_string())?);
    }
    let total = records.len();
    if total == 0 {
        return Err("除标题外无数据行".to_string());
    }

    let base_name = input_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("part");
    let n = n_parts.max(1).min(1000) as usize;
    let actual_parts = n.min(total);
    let base = total / actual_parts;
    let remainder = total % actual_parts;
    let mut cursor = 0usize;

    for i in 0..actual_parts {
        let size = base + usize::from(i < remainder);
        let name = format!("{}_{}.csv", base_name, i + 1);
        let path = output_dir.join(&name);
        let mut writer = csv::WriterBuilder::new()
            .has_headers(true)
            .from_path(&path)
            .map_err(|e| e.to_string())?;
        writer.write_record(&headers).map_err(|e| e.to_string())?;
        for record in &records[cursor..cursor + size] {
            writer.write_record(record).map_err(|e| e.to_string())?;
        }
        writer.flush().map_err(|e| e.to_string())?;
        cursor += size;
    }

    Ok(())
}
