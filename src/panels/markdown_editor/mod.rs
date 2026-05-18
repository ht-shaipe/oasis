#![allow(dead_code)]
mod editor_view;
mod model;
mod syntax;
mod text_utils;

pub use editor_view::{EditorView, InlineMarkdownState};
pub use model::DocumentState;

use gpui::{prelude::FluentBuilder, *};
use gpui_component::{
    ActiveTheme, Disableable, Icon, IconName, Sizable, StyledExt,
    button::{Button, ButtonVariants as _},
    h_flex, v_flex,
};
use rust_i18n::t;

use crate::panels::dock_panel::DockPanel;
use crate::app::actions::{NewFile, OpenFile, SaveFile, SaveFileAs};

/// A dock panel for editing Markdown files with inline WYSIWYG rendering.
pub struct MarkdownEditorPanel {
    editor: Entity<EditorView>,
    document: Entity<DocumentState>,
    inline_markdown: Entity<InlineMarkdownState>,
    current_file_path: Option<std::path::PathBuf>,
    has_opened_file: bool,
    focus_handle: FocusHandle,
}

impl DockPanel for MarkdownEditorPanel {
    fn title() -> &'static str {
        "Markdown"
    }

    fn title_key() -> Option<&'static str> {
        Some("markdown_editor.title")
    }

    fn description() -> &'static str {
        "A Markdown editor with inline WYSIWYG rendering"
    }

    fn tab_icon() -> Option<IconName> {
        Some(IconName::File)
    }

    fn new_view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        Self::view(window, cx)
    }

    fn paddings() -> Pixels {
        px(0.)
    }

    fn on_active(&mut self, _active: bool, _window: &mut Window, _cx: &mut App) {}
}

impl MarkdownEditorPanel {
    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }

    pub fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        let document = cx.new(|_cx| DocumentState::new_empty());
        let inline_markdown = cx.new(|_cx| InlineMarkdownState::new());
        let editor = cx.new(|_cx| EditorView::new(document.clone(), inline_markdown.clone()));
        let focus_handle = cx.focus_handle();

        Self {
            editor,
            document,
            inline_markdown,
            current_file_path: None,
            has_opened_file: false,
            focus_handle,
        }
    }

    pub fn open_file(
        &mut self,
        path: std::path::PathBuf,
        cx: &mut Context<Self>,
    ) -> anyhow::Result<()> {
        log::info!("Opening file: {:?}", path);
        let content = std::fs::read_to_string(&path)?;
        log::info!("File content loaded, length: {} bytes", content.len());

        // Update the document - this will trigger editor re-render automatically
        // since EditorView observes DocumentState changes
        self.document.update(cx, |doc, _cx| {
            doc.set_text(&content);
            doc.clear_undo_history();
            doc.save_snapshot();
        });

        self.current_file_path = Some(path);
        self.has_opened_file = true;
        log::info!("File opened successfully, has_opened_file set to true, notifying");
        cx.notify();
        Ok(())
    }

    fn save_file(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        let Some(path) = self.current_file_path.clone() else {
            return;
        };
        let text = self.document.read(cx).text();
        let dirty = self.document.read(cx).dirty;
        if !dirty {
            return;
        }
        match std::fs::write(&path, &text) {
            Ok(()) => {
                self.document.update(cx, |doc, cx| {
                    doc.save_snapshot();
                    cx.notify();
                });
            }
            Err(e) => {
                log::error!("Failed to save file {:?}: {}", path, e);
            }
        }
    }

    /// Create a new empty document
    fn new_file(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        log::info!("Creating new markdown file");
        self.document.update(cx, |doc, cx| {
            doc.set_text("");
            doc.clear_undo_history();
            doc.save_snapshot();
            cx.notify();
        });
        self.current_file_path = None;
        self.has_opened_file = true;
        cx.notify();
    }

    /// Show save dialog and save to a new file
    fn save_file_as(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        let suggested_name = self
            .current_file_path
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "untitled.md".to_string());

        let entity = cx.entity().clone();
        let text = self.document.read(cx).text();
        let _current_path = self.current_file_path.clone();
        cx.spawn(async move |_entity, cx| {
            let path = rfd::AsyncFileDialog::new()
                .set_title("Save Markdown File")
                .set_file_name(&suggested_name)
                .add_filter("Markdown", &["md", "markdown"])
                .save_file()
                .await;

            if let Some(file_handle) = path {
                let path = file_handle.path().to_path_buf();
                match std::fs::write(&path, &text) {
                    Ok(()) => {
                        let path_clone = path.clone();
                        _ = entity.update(cx, |this, cx| {
                            this.current_file_path = Some(path_clone);
                            this.document.update(cx, |doc, cx| {
                                doc.save_snapshot();
                                cx.notify();
                            });
                        });
                    }
                    Err(e) => {
                        log::error!("Failed to save file: {}", e);
                    }
                }
            }
        }).detach();
    }

    /// Show file picker to open a markdown file
    pub fn show_open_file_dialog(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        log::info!("Opening file dialog");

        // Run rfd async to avoid blocking GPUI's main loop
        let entity = cx.entity().clone();
        cx.spawn(async move |_this, cx| {
            let path = rfd::AsyncFileDialog::new()
                .set_title("Open Markdown File")
                .add_filter("Markdown", &["md", "markdown"])
                .pick_file()
                .await;

            if let Some(file_handle) = path {
                log::info!("File selected: {:?}", file_handle.path());
                let path = file_handle.path().to_path_buf();
                _ = entity.update(cx, |this, cx| {
                    if let Err(e) = this.open_file(path, cx) {
                        log::error!("Failed to open file: {}", e);
                    }
                });
            } else {
                log::info!("No file selected");
            }
        }).detach();
    }

    fn render_toolbar(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let dirty = self.document.read(cx).dirty;
        let file_name = self
            .current_file_path
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "Untitled".to_string());

        h_flex()
            .justify_between()
            .items_center()
            .text_sm()
            .h(px(30.))
            .px_4()
            .border_t_1()
            .border_color(cx.theme().border)
            .text_color(cx.theme().muted_foreground)
            .child(
                h_flex().gap_3().items_center().child(
                    Button::new("md-open")
                        .icon(Icon::new(IconName::FolderOpen))
                        .ghost()
                        .xsmall()
                        .on_click(cx.listener(|this, _, window, cx| {
                            this.show_open_file_dialog(window, cx);
                        })),
                ).child(
                    h_flex()
                        .gap_1()
                        .items_center()
                        .child(file_name)
                        .when(dirty, |el| el.child("●")),
                ),
            )
            .child(
                h_flex().gap_2().child(
                    Button::new("md-save")
                        .icon(Icon::new(IconName::File))
                        .ghost()
                        .xsmall()
                        .disabled(!dirty)
                        .on_click(cx.listener(|this, _, window, cx| {
                            this.save_file(window, cx);
                        })),
                ),
            )
    }

    fn render_empty_state(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        div()
            .id("md-empty-state")
            .size_full()
            .child(
                v_flex()
                    .size_full()
                    .items_center()
                    .justify_center()
                    .gap_4()
                    .child(
                        v_flex()
                            .items_center()
                            .gap_3()
                            .child(
                                Icon::new(IconName::File)
                                    .text_color(theme.muted_foreground)
                                    .text_size(px(48.)),
                            )
                            .child(
                                div()
                                    .text_xl()
                                    .font_semibold()
                                    .text_color(theme.foreground)
                                    .child(t!("markdown_editor.empty.title").to_string()),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(theme.muted_foreground)
                                    .child(t!("markdown_editor.empty.description").to_string()),
                            )
                            .child(
                                h_flex()
                                    .mt_2()
                                    .gap_2()
                                    .child(
                                        Button::new("md-empty-open")
                                            .icon(Icon::new(IconName::FolderOpen).small().text_color(theme.background))
                                            .label("Open")
                                            .primary()
                                            .small()
                                            .on_click(cx.listener(|this, _, window, cx| {
                                                this.show_open_file_dialog(window, cx);
                                            })),
                                    ),
                            ),
                    ),
            )
    }
}

impl Render for MarkdownEditorPanel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let main_content = if self.has_opened_file {
            v_flex()
                .size_full()
                .child(self.editor.clone())
                .into_any_element()
        } else {
            self.render_empty_state(window, cx).into_any_element()
        };

        v_flex()
            .id("markdown-editor-panel")
            .size_full()
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(|this, _: &NewFile, window, cx| {
                log::info!("NewFile action received!");
                this.new_file(window, cx);
            }))
            .on_action(cx.listener(|this, _: &OpenFile, window, cx| {
                log::info!("OpenFile action received!");
                this.show_open_file_dialog(window, cx);
            }))
            .on_action(cx.listener(|this, _: &SaveFile, window, cx| {
                log::info!("SaveFile action received!");
                this.save_file(window, cx);
            }))
            .on_action(cx.listener(|this, _: &SaveFileAs, window, cx| {
                log::info!("SaveFileAs action received!");
                this.save_file_as(window, cx);
            }))
            .child(v_flex().flex_1().min_h_0().child(main_content))
            .child(self.render_toolbar(window, cx))
    }
}
