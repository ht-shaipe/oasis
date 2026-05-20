use crate::commands::{NewFile, OpenFile, SaveFile, SaveFileAs};
use crate::editor_view::EditorView;
use crate::file_explorer::FileExplorerView;
use crate::model::document::DocumentState;
use crate::model::inline_markdown::InlineMarkdownState;
use crate::services::inline_markdown::compute_inline_spans;
use crate::services::tasks::Debouncer;
use crate::theme::Theme;

use gpui::{
    AppContext as _, Context, Entity, InteractiveElement, IntoElement, MouseButton, MouseDownEvent,
    MouseMoveEvent, ParentElement, Render, Styled, Window, div, px,
};

use std::path::PathBuf;
use std::time::Duration;

const INLINE_SYNC_PARSE_MAX_BYTES: usize = 64 * 1024;

pub struct AsterView {
    document: Entity<DocumentState>,
    inline_markdown: Entity<InlineMarkdownState>,
    editor_view: Entity<EditorView>,
    file_explorer_view: Entity<FileExplorerView>,
    inline_debounce: Debouncer<AsterView>,
    scheduled_inline_revision: u64,
    cached_doc_text: Option<(u64, String)>,
    sidebar_width: f32,
    resizing_sidebar: bool,
}

impl AsterView {
    pub fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        let document = cx.new(|_| DocumentState::new_empty());
        let inline_markdown = cx.new(|_| InlineMarkdownState::new());
        let editor_view = cx.new(|_cx| EditorView::new(document.clone(), inline_markdown.clone()));
        let file_explorer_view = cx.new(|_| FileExplorerView::new(document.clone()));

        Self {
            document,
            inline_markdown,
            editor_view,
            file_explorer_view,
            inline_debounce: Debouncer::new(Duration::from_millis(35)),
            scheduled_inline_revision: 0,
            cached_doc_text: None,
            sidebar_width: 200.0,
            resizing_sidebar: false,
        }
    }

    fn save_document(&mut self, cx: &mut Context<Self>, force_save_as: bool) {
        let current_path = self.document.read(cx).path.clone();

        if !force_save_as {
            if let Some(ref path) = current_path {
                self.do_save_to_path_sync(path.clone(), cx);
                return;
            }
        }

        let _doc = self.document.clone();
        cx.spawn(async move |this, cx| {
            let file = rfd::AsyncFileDialog::new()
                .set_file_name("untitled.md")
                .add_filter("Markdown", &["md", "markdown"])
                .save_file()
                .await;

            if let Some(handle) = file {
                let path = PathBuf::from(handle.path().to_path_buf());
                let contents = this.update(&mut *cx, |this, cx| this.document.read(cx).text()).ok();
                if let Some(contents) = contents {
                    if std::fs::write(&path, &contents).is_ok() {
                        let _ = this.update(&mut *cx, |this, cx| {
                            let _ = this.document.update(cx, |d, cx| {
                                d.path = Some(path.clone());
                                d.save_snapshot();
                                cx.notify();
                            });
                        });
                    }
                }
            }
        })
        .detach();
    }

    fn do_save_to_path_sync(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        let mut path = path;
        if path.extension().is_none() {
            path.set_extension("md");
        }
        let contents = self.document.read(cx).text();
        if std::fs::write(&path, &contents).is_ok() {
            let _ = self.document.update(cx, |d, cx| {
                d.path = Some(path);
                d.save_snapshot();
                cx.notify();
            });
        }
    }

    fn open_path_internal(&mut self, path: &PathBuf, cx: &mut Context<Self>) {
        match std::fs::read_to_string(path) {
            Ok(text) => {
                let _ = self.document.update(cx, |d, cx| {
                    d.path = Some(path.clone());
                    d.set_text(&text);
                    d.clear_undo_history();
                    d.save_snapshot();
                    cx.notify();
                });
            }
            Err(_) => {}
        }
    }

    fn action_new_file(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        let _ = self.document.update(cx, |d, cx| {
            d.path = None;
            d.set_text("");
            d.clear_undo_history();
            d.save_snapshot();
            cx.notify();
        });
    }

    fn action_open_file(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        cx.spawn(async move |this, cx| {
            let file = rfd::AsyncFileDialog::new()
                .add_filter("Markdown", &["md", "markdown"])
                .pick_file()
                .await;

            if let Some(handle) = file {
                let path = PathBuf::from(handle.path().to_path_buf());
                let _ = this.update(&mut *cx, |this, cx| {
                    this.open_path_internal(&path, cx);
                });
            }
        })
        .detach();
    }

    fn action_save(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.save_document(cx, false);
    }

    fn action_save_as(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.save_document(cx, true);
    }
}

impl Render for AsterView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (doc_path, doc_dirty, doc_revision, word_count) = {
            self.document.update(cx, |doc, _| {
                (doc.path.clone(), doc.dirty, doc.revision, doc.get_word_count())
            })
        };

        // Inline markdown parse
        let doc_text = if let Some((cached_rev, ref text)) = self.cached_doc_text {
            if cached_rev == doc_revision { text.clone() }
            else {
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
            let last_edit = self.document.read(cx).last_edit.clone();
            let target_rev = doc_revision;
            if doc_text.len() <= INLINE_SYNC_PARSE_MAX_BYTES {
                let parsed = compute_inline_spans(&doc_text, last_edit.as_ref());
                let _ = self.inline_markdown.update(cx, |state, cx| {
                    if target_rev >= state.source_revision {
                        state.spans = std::sync::Arc::new(parsed.spans);
                        state.source_revision = target_rev;
                        state.parse_millis = parsed.parse_millis;
                        cx.notify();
                    } else {
                        state.dropped_updates = state.dropped_updates.saturating_add(1);
                    }
                });
            } else {
                let text = doc_text.clone();
                let inline_markdown = self.inline_markdown.clone();
                self.inline_debounce.schedule(cx, move |_, cx| {
                    let text = text.clone();
                    let last_edit = last_edit.clone();
                    let inline_markdown = inline_markdown.clone();
                    cx.spawn(async move |_, cx| {
                        let parsed = cx
                            .background_executor()
                            .spawn(async move { compute_inline_spans(&text, last_edit.as_ref()) })
                            .await;
                        let _ = inline_markdown.update(cx, |state, cx| {
                            if target_rev >= state.source_revision {
                                state.spans = std::sync::Arc::new(parsed.spans);
                                state.source_revision = target_rev;
                                state.parse_millis = parsed.parse_millis;
                                cx.notify();
                            } else {
                                state.dropped_updates = state.dropped_updates.saturating_add(1);
                            }
                        });
                    })
                    .detach();
                });
            }
        }

        let (inline_parse_millis, inline_dropped_updates) = {
            let inline = self.inline_markdown.read(cx);
            (inline.parse_millis, inline.dropped_updates)
        };
        let status_right = if inline_dropped_updates > 0 {
            format!("{} words · inline {:.1} ms · dropped {}", word_count, inline_parse_millis, inline_dropped_updates)
        } else {
            format!("{} words · inline {:.1} ms", word_count, inline_parse_millis)
        };

        let file_name = doc_path
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("untitled.md");
        let dirty_marker = if doc_dirty { " •" } else { "" };
        let title_text = format!("{}{}", file_name, dirty_marker);

        // Toolbar
        let toolbar = div()
            .id("aster-toolbar")
            .h(px(38.))
            .w_full()
            .bg(Theme::panel())
            .border_b_1()
            .border_color(Theme::border())
            .flex_shrink_0()
            .flex()
            .items_center()
            .px(px(12.))
            .gap(px(8.))
            .child(
                div()
                    .text_sm()
                    .font_weight(gpui::FontWeight::BOLD)
                    .text_color(Theme::text())
                    .child(title_text),
            )
            .child(div().flex_1())
            .child(
                div()
                    .id("btn-new")
                    .px(px(6.))
                    .py(px(2.))
                    .text_xs()
                    .text_color(Theme::muted())
                    .cursor_pointer()
                    .hover(|s| s.bg(Theme::panel_alt()))
                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, window, cx| {
                        this.action_new_file(window, cx);
                    }))
                    .child("New"),
            )
            .child(
                div()
                    .id("btn-open")
                    .px(px(6.))
                    .py(px(2.))
                    .text_xs()
                    .text_color(Theme::muted())
                    .cursor_pointer()
                    .hover(|s| s.bg(Theme::panel_alt()))
                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, window, cx| {
                        this.action_open_file(window, cx);
                    }))
                    .child("Open"),
            )
            .child(
                div()
                    .id("btn-save")
                    .px(px(6.))
                    .py(px(2.))
                    .text_xs()
                    .text_color(Theme::muted())
                    .cursor_pointer()
                    .hover(|s| s.bg(Theme::panel_alt()))
                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, window, cx| {
                        this.action_save(window, cx);
                    }))
                    .child("Save"),
            );

        // Resize handle
        let resize_line_color = if self.resizing_sidebar {
            gpui::rgba(0x2d7fd299)
        } else {
            Theme::border()
        };

        // Bottom bar
        let bottom_bar = div()
            .flex()
            .items_center()
            .gap_3()
            .px(px(16.))
            .py(px(4.))
            .bg(Theme::panel())
            .border_t_1()
            .border_color(Theme::border())
            .flex_shrink_0()
            .child(
                div().w_full().flex().justify_end().child(
                    div()
                        .text_sm()
                        .text_color(Theme::muted())
                        .overflow_hidden()
                        .max_w(px(640.))
                        .child(status_right),
                ),
            );

        div()
            .relative()
            .flex()
            .flex_col()
            .bg(Theme::bg())
            .text_color(Theme::text())
            .size_full()
            .on_action(cx.listener(|this, _: &NewFile, window, cx| {
                this.action_new_file(window, cx);
            }))
            .on_action(cx.listener(|this, _: &OpenFile, window, cx| {
                this.action_open_file(window, cx);
            }))
            .on_action(cx.listener(|this, _: &SaveFile, window, cx| {
                this.action_save(window, cx);
            }))
            .on_action(cx.listener(|this, _: &SaveFileAs, window, cx| {
                this.action_save_as(window, cx);
            }))
            .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, _, cx| {
                if !this.resizing_sidebar { return; }
                let new_width: f32 = event.position.x.into();
                this.sidebar_width = new_width.clamp(100.0, 400.0);
                cx.notify();
            }))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _, _, cx| {
                    if this.resizing_sidebar {
                        this.resizing_sidebar = false;
                        cx.notify();
                    }
                }),
            )
            .child(toolbar)
            .child(
                div()
                    .flex_1()
                    .min_h(px(0.))
                    .min_w(px(0.))
                    .flex()
                    .flex_row()
                    .child({
                        let fe = self.file_explorer_view.clone();
                        let width = self.sidebar_width;
                        let _ = fe.update(cx, |view, cx| { view.set_width(width, cx); });
                        fe
                    })
                    .child(
                        div()
                            .id("sidebar-resize")
                            .w(px(1.))
                            .h_full()
                            .cursor_col_resize()
                            .bg(resize_line_color)
                            .hover(|s| s.bg(gpui::rgba(0x2d7fd24d)))
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|this, _: &MouseDownEvent, _, cx| {
                                    this.resizing_sidebar = true;
                                    cx.notify();
                                }),
                            ),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_h(px(0.))
                            .min_w(px(0.))
                            .flex()
                            .flex_col()
                            .child(self.editor_view.clone()),
                    ),
            )
            .child(bottom_bar)
    }
}