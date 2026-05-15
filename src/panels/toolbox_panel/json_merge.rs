//! JSON 合并工具：扫描目录下所有 JSON 文件，按指定 JSON Path 提取数组，合并为一个 JSON 文件。

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

pub struct JsonMergeState {
    pub input_dir: Option<PathBuf>,
    pub json_path: Entity<gpui_component::input::InputState>,
    pub output_path: Option<PathBuf>,
    pub loading: bool,
    pub message: Option<String>,
    pub message_ok: bool,
}

impl JsonMergeState {
    pub fn new(window: &mut Window, cx: &mut Context<ToolboxPanel>) -> Self {
        let json_path = cx.new(|cx| {
            let mut s = gpui_component::input::InputState::new(window, cx);
            s.set_placeholder("data.goodsList", window, cx);
            s
        });

        Self {
            input_dir: None,
            json_path,
            output_path: None,
            loading: false,
            message: None,
            message_ok: true,
        }
    }

    pub fn render(
        state: &mut JsonMergeState,
        entity: Entity<ToolboxPanel>,
        _window: &mut Window,
        cx: &mut Context<ToolboxPanel>,
    ) -> impl IntoElement {
        let theme = cx.theme();

        let input_label = state
            .input_dir
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| t!("toolbox.json_merge.no_input_dir").to_string());

        let out_label = state
            .output_path
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .map(String::from)
            .unwrap_or_else(|| t!("toolbox.json_merge.no_output").to_string());

        let json_path_entity = state.json_path.clone();

        // 输入目录
        let entity_input = entity.clone();
        let row_input = h_flex()
            .gap_3()
            .items_center()
            .w_full()
            .child(
                Button::new("json-merge-pick-input")
                    .label(t!("toolbox.json_merge.input_dir").to_string())
                    .icon(Icon::new(IconName::Folder).text_color(theme.blue))
                    .outline()
                    .on_click(move |_, _, cx| {
                        entity_input.update(cx, |this, cx| this.pick_json_merge_input_dir(cx));
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

        // JSON Path
        let row_path = h_flex()
            .gap_3()
            .items_center()
            .w_full()
            .child(
                Label::new(t!("toolbox.json_merge.json_path").to_string())
                    .text_sm()
                    .w(px(100.)),
            )
            .child(
                gpui::div()
                    .flex_1()
                    .child(Input::new(&json_path_entity).cleanable(true)),
            );

        // 输出文件
        let entity_out = entity.clone();
        let row_out = h_flex()
            .gap_3()
            .items_center()
            .w_full()
            .child(
                Button::new("json-merge-pick-out")
                    .label(t!("toolbox.json_merge.output_file").to_string())
                    .icon(Icon::new(IconName::File).text_color(theme.blue))
                    .outline()
                    .on_click(move |_, _, cx| {
                        entity_out.update(cx, |this, cx| this.pick_json_merge_output(cx));
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

        // 执行按钮
        let entity_exec = entity.clone();
        let execute_btn = Button::new("json-merge-execute")
            .label(t!("toolbox.json_merge.execute").to_string())
            .primary()
            .disabled(state.loading)
            .on_click(move |_, _, cx| {
                entity_exec.update(cx, |this, cx| this.execute_json_merge(cx));
            });

        let msg = state.message.clone();
        let msg_ok = state.message_ok;

        let content = GroupBox::new()
            .with_variant(GroupBoxVariant::Outline)
            .title(t!("toolbox.tools.json_merge").to_string())
            .gap_4()
            .child(row_input)
            .child(row_path)
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

/// 按 dot-separated path 从 JSON Value 中提取子节点
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

/// 扫描目录下所有 .json 文件，按 JSON Path 提取数组并合并为一个 JSON 文件。
/// 返回合并的数组总长度。
pub fn do_json_merge(
    input_dir: &Path,
    output_path: &Path,
    json_path: &str,
) -> Result<usize, String> {
    if !input_dir.exists() {
        return Err(t!("toolbox.json_merge.no_input_dir").to_string());
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
        return Err(t!("toolbox.json_merge.no_json_files").to_string());
    }

    let mut merged: Vec<serde_json::Value> = Vec::new();
    let mut errors: Vec<String> = Vec::new();

    for file_path in &json_files {
        let file_name = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("?")
            .to_string();

        let raw = match std::fs::read_to_string(file_path) {
            Ok(r) => r,
            Err(e) => {
                errors.push(format!("{}: {}", file_name, e));
                continue;
            }
        };

        let root: serde_json::Value = match serde_json::from_str(&raw) {
            Ok(v) => v,
            Err(e) => {
                errors.push(format!("{}: {}", file_name, e));
                continue;
            }
        };

        let target = resolve_json_path(&root, json_path);
        match target {
            Some(serde_json::Value::Array(arr)) => {
                merged.extend(arr.clone());
            }
            Some(_) => {
                errors.push(format!(
                    "{}: {}",
                    file_name,
                    t!("toolbox.json_merge.not_array").to_string()
                ));
            }
            None => {
                errors.push(format!(
                    "{}: {}",
                    file_name,
                    t!("toolbox.json_merge.path_not_found").to_string()
                ));
            }
        }
    }

    let content = serde_json::to_string_pretty(&merged).map_err(|e| e.to_string())?;
    std::fs::write(output_path, content).map_err(|e| e.to_string())?;

    if !errors.is_empty() {
        return Err(format!(
            "{}\n{}:\n{}",
            t!("toolbox.json_merge.partial_errors").to_string(),
            t!("toolbox.json_merge.batch_partial_errors").to_string(),
            errors.join("\n")
        ));
    }

    Ok(merged.len())
}
