//! 批量重命名：在目录中查找文件名包含指定子串的文件，将子串替换为新文本（非正则，字面量替换）。

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use gpui::{
    AppContext as _, Context, Entity, IntoElement, ParentElement as _,
    SharedString, Styled, Window, px,
};
use gpui_component::{
    ActiveTheme, Disableable, Icon, IconName,
    button::{Button, ButtonVariants as _},
    checkbox::Checkbox,
    group_box::{GroupBox, GroupBoxVariant, GroupBoxVariants as _},
    h_flex,
    input::Input,
    label::Label,
    scroll::ScrollableElement as _,
    v_flex,
};
use rust_i18n::t;

use super::ToolboxPanel;

/// 列出目录下的文件（可选递归子目录）。
pub fn list_files(dir: &Path, recursive: bool) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut out = Vec::new();
    let read = fs::read_dir(dir)?;
    for ent in read {
        let ent = ent?;
        let p = ent.path();
        if p.is_file() {
            out.push(p);
        } else if recursive && p.is_dir() {
            out.extend(list_files(&p, true)?);
        }
    }
    Ok(out)
}

/// 根据「文件名包含 needle」生成 (旧路径, 新路径)。needle 为空则报错；同一目录下若多个文件重命名为同名则报错。
pub fn build_rename_plan(
    files: &[PathBuf],
    needle: &str,
    replacement: &str,
) -> Result<Vec<(PathBuf, PathBuf)>, String> {
    let needle = needle.trim();
    if needle.is_empty() {
        return Err(t!("toolbox.rename.empty_needle").to_string());
    }

    let mut plan = Vec::new();
    let mut new_names: HashSet<PathBuf> = HashSet::new();

    for path in files {
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if !name.contains(needle) {
            continue;
        }
        let new_name = name.replace(needle, replacement);
        if new_name == name {
            continue;
        }
        let Some(parent) = path.parent() else {
            continue;
        };
        let new_path = parent.join(&new_name);
        if !new_names.insert(new_path.clone()) {
            return Err(format!(
                "{}: {}",
                t!("toolbox.rename.duplicate_target"),
                new_path.display()
            ));
        }
        plan.push((path.clone(), new_path));
    }

    Ok(plan)
}

/// 执行重命名；若目标已存在且不是源文件则跳过并记录错误。
pub fn apply_rename_plan(plan: &[(PathBuf, PathBuf)]) -> (usize, Vec<String>) {
    let mut ok = 0usize;
    let mut errs = Vec::new();

    for (old, new) in plan {
        if old == new {
            continue;
        }
        if new.exists() && new != old {
            errs.push(format!(
                "{} → {}: {}",
                old.display(),
                new.display(),
                t!("toolbox.rename.target_exists")
            ));
            continue;
        }
        match fs::rename(old, new) {
            Err(e) => errs.push(format!("{}: {}", old.display(), e)),
            Ok(()) => ok += 1,
        }
    }

    (ok, errs)
}

pub struct BatchRenameState {
    pub dir: Option<PathBuf>,
    pub needle_input: Entity<gpui_component::input::InputState>,
    pub replace_input: Entity<gpui_component::input::InputState>,
    pub recursive: bool,
    /// 预览通过后的计划，供执行（执行时会再次校验并重建计划）
    pub plan: Vec<(PathBuf, PathBuf)>,
    pub preview_summary: Option<String>,
    pub loading: bool,
    pub message: Option<String>,
    pub message_ok: bool,
}

impl BatchRenameState {
    pub fn new(window: &mut Window, cx: &mut Context<ToolboxPanel>) -> Self {
        let needle_input = cx.new(|cx| {
            let mut s = gpui_component::input::InputState::new(window, cx);
            s.set_placeholder(
                SharedString::from(t!("toolbox.rename.placeholder_needle")),
                window,
                cx,
            );
            s
        });
        let replace_input = cx.new(|cx| {
            let mut s = gpui_component::input::InputState::new(window, cx);
            s.set_placeholder(
                SharedString::from(t!("toolbox.rename.placeholder_replace")),
                window,
                cx,
            );
            s
        });

        Self {
            dir: None,
            needle_input,
            replace_input,
            recursive: false,
            plan: Vec::new(),
            preview_summary: None,
            loading: false,
            message: None,
            message_ok: true,
        }
    }

    pub fn render(
        state: &mut BatchRenameState,
        entity: Entity<ToolboxPanel>,
        _window: &mut Window,
        cx: &mut Context<ToolboxPanel>,
    ) -> impl IntoElement {
        let theme = cx.theme();
        let mono = theme.mono_font_family.clone();

        let dir_label = state
            .dir
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| t!("toolbox.rename.no_dir").to_string());

        let entity_dir = entity.clone();
        let row_dir = h_flex()
            .gap_3()
            .items_center()
            .w_full()
            .child(
                Button::new("batch-rename-pick-dir")
                    .label(t!("toolbox.rename.select_dir").to_string())
                    .icon(Icon::new(IconName::Folder).text_color(theme.blue))
                    .outline()
                    .on_click(move |_, _, cx| {
                        entity_dir.update(cx, |this, cx| this.pick_batch_rename_dir(cx));
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

        let needle_input = state.needle_input.clone();
        let replace_input = state.replace_input.clone();

        let row_rules = h_flex()
            .gap_4()
            .w_full()
            .child(
                v_flex()
                    .gap_1()
                    .flex_1()
                    .child(
                        Label::new(t!("toolbox.rename.field_needle").to_string())
                            .text_sm()
                            .text_color(theme.muted_foreground),
                    )
                    .child(Input::new(&needle_input).w_full()),
            )
            .child(
                v_flex()
                    .gap_1()
                    .flex_1()
                    .child(
                        Label::new(t!("toolbox.rename.field_replace").to_string())
                            .text_sm()
                            .text_color(theme.muted_foreground),
                    )
                    .child(Input::new(&replace_input).w_full()),
            );

        let entity_rec = entity.clone();
        let recursive = state.recursive;
        let row_recursive = h_flex().items_center().child(
            Checkbox::new("toolbox-rename-recursive")
                .label(t!("toolbox.rename.recursive").to_string())
                .checked(recursive)
                .on_click(move |checked, _, cx| {
                    entity_rec.update(cx, |this, cx| {
                        this.batch_rename.recursive = *checked;
                        cx.notify();
                    });
                }),
        );

        let hint = Label::new(t!("toolbox.rename.hint").to_string())
            .text_xs()
            .text_color(theme.muted_foreground);

        let entity_preview = entity.clone();
        let preview_btn = Button::new("batch-rename-preview")
            .label(t!("toolbox.rename.preview").to_string())
            .outline()
            .disabled(state.loading)
            .on_click(move |_, window, cx| {
                entity_preview.update(cx, |this, cx| {
                    this.batch_rename_preview(window, cx);
                });
            });

        let entity_exec = entity.clone();
        let can_exec = !state.plan.is_empty() && !state.loading;
        let exec_btn = Button::new("batch-rename-exec")
            .label(t!("toolbox.rename.execute").to_string())
            .primary()
            .disabled(!can_exec)
            .on_click(move |_, window, cx| {
                entity_exec.update(cx, |this, cx| {
                    this.batch_rename_execute(window, cx);
                });
            });

        let preview_block = if let Some(ref s) = state.preview_summary {
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
                    Label::new(t!("toolbox.rename.preview_title").to_string())
                        .text_sm()
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_color(theme.foreground),
                )
                .child(
                    gpui::div()
                        .w_full()
                        .max_h(px(220.))
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
                        .text_color(if msg_ok { theme.primary } else { theme.danger })
                        .size_12(),
                )
                .child(msg_lines)
                .into_any_element()
        });

        let stats_row = h_flex()
            .gap_2()
            .items_center()
            .child(
                Icon::new(IconName::Info)
                    .text_color(theme.muted_foreground)
                    .size_12(),
            )
            .child(
                Label::new(format!(
                    "{} {}",
                    t!("toolbox.rename.preview_count_prefix"),
                    state.plan.len()
                ))
                .text_xs()
                .text_color(theme.muted_foreground),
            );

        let content = GroupBox::new()
            .with_variant(GroupBoxVariant::Outline)
            .title(t!("toolbox.tools.batch_rename").to_string())
            .gap_4()
            .child(row_dir)
            .child(row_rules)
            .child(row_recursive)
            .child(hint)
            .child(
                h_flex()
                    .gap_3()
                    .items_center()
                    .child(preview_btn)
                    .child(exec_btn),
            )
            .child(stats_row)
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
}
