#![allow(dead_code)]
mod debouncer;
mod editor_view;
mod file_browser;
mod file_explorer;
mod model;
mod syntax;
mod text_utils;

pub use debouncer::Debouncer;
pub use editor_view::{EditorView, InlineMarkdownState};
pub use file_browser::FileBrowserView;
pub use file_explorer::FileExplorerView;
pub use model::DocumentState;

use gpui::{prelude::FluentBuilder, *};
use gpui_component::{
    ActiveTheme, Disableable, Icon, IconName, Sizable, StyledExt,
    button::{Button, ButtonVariants as _},
    h_flex, v_flex,
};
use rust_i18n::t;
use std::sync::Arc;
use std::time::Duration;

use crate::panels::dock_panel::DockPanel;
use crate::panels::markdown_editor::syntax::markdown_spans;
use crate::app::actions::{NewFile, OpenFile, SaveFile, SaveFileAs};

/// Max document size for synchronous inline parsing (64 KB).
const INLINE_SYNC_PARSE_MAX_BYTES: usize = 64 * 1024;

/// A dock panel for editing Markdown files with inline WYSIWYG rendering.
pub struct MarkdownEditorPanel {
    editor: Entity<EditorView>,
    document: Entity<DocumentState>,
    inline_markdown: Entity<InlineMarkdownState>,
    file_explorer: Entity<FileExplorerView>,
    file_browser: Entity<FileBrowserView>,
    current_file_path: Option<std::path::PathBuf>,
    has_opened_file: bool,
    focus_handle: FocusHandle,
    /// Debouncer for background inline markdown parsing.
    inline_debounce: Debouncer<MarkdownEditorPanel>,
    /// Highest document revision for which an inline parse has been scheduled.
    scheduled_inline_revision: u64,
    /// Cached document text to avoid O(n) rope conversion every frame.
    cached_doc_text: Option<(u64, String)>,
    /// Left sidebar (file browser) width in pixels.
    file_browser_width: f32,
    /// Right sidebar (outline) width in pixels.
    outline_width: f32,
    /// Which sidebar is being resized: "left" or "right".
    resizing: Option<String>,
    /// X position (window-relative) where resize drag started.
    resize_start_x: f32,
    /// Sidebar width at the start of the resize drag.
    resize_start_width: f32,
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
        let file_explorer = cx.new(|_cx| FileExplorerView::new(document.clone()));
        let file_browser = cx.new(|_cx| FileBrowserView::new());
        let focus_handle = cx.focus_handle();

        // Set panel reference after creation (avoids borrow conflict).
        let panel_entity = cx.entity().clone();
        file_browser.update(cx, |fb, cx| fb.set_panel(panel_entity, cx));

        Self {
            editor,
            document,
            inline_markdown,
            file_explorer,
            file_browser,
            current_file_path: None,
            has_opened_file: false,
            focus_handle,
            inline_debounce: Debouncer::new(Duration::from_millis(35)),
            scheduled_inline_revision: 0,
            cached_doc_text: None,
            file_browser_width: 180.0,
            outline_width: 200.0,
            resizing: None,
            resize_start_x: 0.0,
            resize_start_width: 200.0,
        }
    }

    pub fn open_file(
        &mut self,
        path: std::path::PathBuf,
        cx: &mut Context<Self>,
    ) -> anyhow::Result<()> {
        let content = std::fs::read_to_string(&path)?;

        self.document.update(cx, |doc, _cx| {
            doc.set_text(&content);
            doc.clear_undo_history();
            doc.save_snapshot();
        });

        // Update file browser: set directory and selected file.
        if let Some(parent) = path.parent() {
            let dir = parent.to_path_buf();
            self.file_browser.update(cx, |fb, cx| {
                let needs_refresh = fb.current_dir() != Some(&dir);
                if needs_refresh {
                    fb.set_directory(dir, cx);
                }
                fb.set_selected_file(&path, cx);
            });
        }

        self.current_file_path = Some(path);
        self.has_opened_file = true;
        self.cached_doc_text = None;
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

    fn new_file(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.document.update(cx, |doc, cx| {
            doc.set_text("");
            doc.clear_undo_history();
            doc.save_snapshot();
            cx.notify();
        });
        self.current_file_path = None;
        self.has_opened_file = true;
        self.cached_doc_text = None;
        cx.notify();
    }

    fn save_file_as(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        let suggested_name = self
            .current_file_path
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "untitled.md".to_string());

        let entity = cx.entity().clone();
        let text = self.document.read(cx).text();
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

    pub fn show_open_file_dialog(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        let entity = cx.entity().clone();
        cx.spawn(async move |_this, cx| {
            let path = rfd::AsyncFileDialog::new()
                .set_title("Open Markdown File")
                .add_filter("Markdown", &["md", "markdown"])
                .pick_file()
                .await;

            if let Some(file_handle) = path {
                let path = file_handle.path().to_path_buf();
                _ = entity.update(cx, |this, cx| {
                    if let Err(e) = this.open_file(path, cx) {
                        log::error!("Failed to open file: {}", e);
                    }
                });
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
        let word_count = self.document.update(cx, |doc, _| doc.get_word_count());
        let (inline_parse_millis, inline_dropped_updates) = {
            let inline = self.inline_markdown.read(cx);
            (inline.parse_millis, inline.dropped_updates)
        };
        let status_right = if inline_dropped_updates > 0 {
            format!(
                "{} words · inline {:.1} ms · dropped {}",
                word_count, inline_parse_millis, inline_dropped_updates
            )
        } else {
            format!("{} words · inline {:.1} ms", word_count, inline_parse_millis)
        };

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
                h_flex().gap_2()
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .overflow_hidden()
                            .max_w(px(400.))
                            .child(status_right),
                    )
                    .child(
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
        // ── Debounced inline markdown parsing ──
        let doc_revision = self.document.read(cx).revision;

        let doc_text = if let Some((cached_rev, ref text)) = self.cached_doc_text {
            if cached_rev == doc_revision {
                text.clone()
            } else {
                let text = self.document.read(cx).text();
                self.cached_doc_text = Some((doc_revision, text.clone()));
                text
            }
        } else {
            let text = self.document.read(cx).text();
            self.cached_doc_text = Some((doc_revision, text.clone()));
            text
        };

        let inline_rev = self.inline_markdown.read(cx).source_revision;
        if doc_revision != inline_rev && self.scheduled_inline_revision < doc_revision {
            self.scheduled_inline_revision = doc_revision;
            let target_rev = doc_revision;
            if doc_text.len() <= INLINE_SYNC_PARSE_MAX_BYTES {
                let parsed = markdown_spans(&doc_text);
                let _ = self.inline_markdown.update(cx, |state, cx| {
                    if target_rev >= state.source_revision {
                        state.spans = Arc::new(parsed);
                        state.source_revision = target_rev;
                        cx.notify();
                    }
                });
            } else {
                let text = doc_text.clone();
                let inline_markdown = self.inline_markdown.clone();
                self.inline_debounce.schedule(cx, move |_, cx| {
                    let text = text.clone();
                    let inline_markdown = inline_markdown.clone();
                    cx.spawn(async move |_, cx| {
                        let parsed = cx
                            .background_executor()
                            .spawn(async move { markdown_spans(&text) })
                            .await;
                        let _ = inline_markdown.update(cx, |state, cx| {
                            if target_rev >= state.source_revision {
                                state.spans = Arc::new(parsed);
                                state.source_revision = target_rev;
                                cx.notify();
                            }
                        });
                    })
                    .detach();
                });
            }
        }

        // ── Sync widths to child views ──
        let fb_width = self.file_browser_width;
        let outline_w = self.outline_width;
        self.file_browser.update(cx, |fb, cx| fb.set_width(fb_width, cx));
        self.file_explorer.update(cx, |fe, cx| fe.set_width(outline_w, cx));

        // ── Resize handle style ──
        let is_resizing = self.resizing.is_some();
        let resize_line_color: Hsla = if is_resizing {
            gpui::rgba(0x2d7fd299).into()
        } else {
            cx.theme().border
        };

        // ── Build three-column layout ──
        let main_content = if self.has_opened_file {
            div()
                .flex_1()
                .min_w(px(0.))
                .min_h(px(0.))
                .flex()
                .flex_row()
                // ── Left: File Browser ──
                .child(self.file_browser.clone())
                // ── Left resize handle ──
                .child(
                    div()
                        .id("md-left-resize")
                        .w(px(1.))
                        .h_full()
                        .cursor_col_resize()
                        .bg(resize_line_color)
                        .hover(|s| s.bg(gpui::rgba(0x2d7fd24d)))
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(|this, event: &MouseDownEvent, _, cx| {
                                this.resizing = Some("left".to_string());
                                this.resize_start_x = event.position.x.into();
                                this.resize_start_width = this.file_browser_width;
                                cx.notify();
                            }),
                        ),
                )
                // ── Center: Editor ──
                .child(
                    v_flex()
                        .flex_1()
                        .min_h(px(0.))
                        .child(self.editor.clone()),
                )
                // ── Right resize handle ──
                .child(
                    div()
                        .id("md-right-resize")
                        .w(px(1.))
                        .h_full()
                        .cursor_col_resize()
                        .bg(resize_line_color)
                        .hover(|s| s.bg(gpui::rgba(0x2d7fd24d)))
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(|this, event: &MouseDownEvent, _, cx| {
                                this.resizing = Some("right".to_string());
                                this.resize_start_x = event.position.x.into();
                                this.resize_start_width = this.outline_width;
                                cx.notify();
                            }),
                        ),
                )
                // ── Right: Outline ──
                .child(self.file_explorer.clone())
                .into_any_element()
        } else {
            self.render_empty_state(window, cx).into_any_element()
        };

        v_flex()
            .id("markdown-editor-panel")
            .size_full()
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(|this, _: &NewFile, window, cx| {
                this.new_file(window, cx);
            }))
            .on_action(cx.listener(|this, _: &OpenFile, window, cx| {
                this.show_open_file_dialog(window, cx);
            }))
            .on_action(cx.listener(|this, _: &SaveFile, window, cx| {
                this.save_file(window, cx);
            }))
            .on_action(cx.listener(|this, _: &SaveFileAs, window, cx| {
                this.save_file_as(window, cx);
            }))
            // Handle resize drag at root level
            .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, _, cx| {
                let Some(ref side) = this.resizing else {
                    return;
                };
                let current_x: f32 = event.position.x.into();
                let dx = current_x - this.resize_start_x;
                if side == "left" {
                    this.file_browser_width = (this.resize_start_width + dx).clamp(100.0, 400.0);
                } else {
                    // Right sidebar: dragging right increases outline width
                    this.outline_width = (this.resize_start_width - dx).clamp(100.0, 400.0);
                }
                cx.notify();
            }))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _, _, cx| {
                    if this.resizing.is_some() {
                        this.resizing = None;
                        cx.notify();
                    }
                }),
            )
            .child(v_flex().flex_1().min_h_0().child(main_content))
            .child(self.render_toolbar(window, cx))
    }
}
