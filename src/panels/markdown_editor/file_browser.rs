use gpui::prelude::FluentBuilder as _;
use gpui::{
    Context, Entity, InteractiveElement, IntoElement, MouseButton, MouseDownEvent,
    ParentElement, Render, ScrollHandle, StatefulInteractiveElement, Styled, Window, div, px,
};
use gpui_component::ActiveTheme;
use std::path::PathBuf;

use super::MarkdownEditorPanel;

pub struct FileBrowserView {
    /// Current directory being browsed.
    current_dir: Option<PathBuf>,
    /// List of .md files in current directory.
    files: Vec<PathBuf>,
    /// Scroll handle for file list.
    scroll_handle: ScrollHandle,
    /// Width of the browser panel.
    width: f32,
    /// Index of selected file (for highlighting).
    selected_index: Option<usize>,
    /// Reference to parent panel for opening files (set after creation).
    panel: Option<Entity<MarkdownEditorPanel>>,
}

impl FileBrowserView {
    pub fn new() -> Self {
        Self {
            current_dir: None,
            files: Vec::new(),
            scroll_handle: ScrollHandle::new(),
            width: 180.0,
            selected_index: None,
            panel: None,
        }
    }

    /// Set the parent panel reference (called after both entities are created).
    pub fn set_panel(&mut self, panel: Entity<MarkdownEditorPanel>, _cx: &mut Context<Self>) {
        self.panel = Some(panel);
    }

    pub fn set_width(&mut self, width: f32, cx: &mut Context<Self>) {
        self.width = width;
        cx.notify();
    }

    /// Set the directory to browse and refresh file list.
    pub fn set_directory(&mut self, dir: PathBuf, cx: &mut Context<Self>) {
        self.current_dir = Some(dir.clone());
        self.files = self.scan_md_files(&dir);
        self.selected_index = None;
        cx.notify();
    }

    /// Mark a file as selected (by path).
    pub fn set_selected_file(&mut self, path: &PathBuf, cx: &mut Context<Self>) {
        self.selected_index = self.files.iter().position(|f| f == path);
        cx.notify();
    }

    /// Scan directory for .md files (non-recursive).
    fn scan_md_files(&self, dir: &PathBuf) -> Vec<PathBuf> {
        let mut files = Vec::new();
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    let ext = path.extension().and_then(|e| e.to_str());
                    if ext.map(|e| e.eq_ignore_ascii_case("md")).unwrap_or(false) {
                        files.push(path);
                    }
                }
            }
        }
        // Sort alphabetically.
        files.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
        files
    }

    /// Get current directory name for display.
    fn dir_name(&self) -> String {
        self.current_dir
            .as_ref()
            .and_then(|d| d.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "No Folder".to_string())
    }

    pub fn current_dir(&self) -> Option<&PathBuf> {
        self.current_dir.as_ref()
    }

    pub fn width(&self) -> f32 {
        self.width
    }
}

impl Render for FileBrowserView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let has_files = !self.files.is_empty();
        let dir_name = self.dir_name();
        let files = self.files.clone();
        let selected = self.selected_index;

        let file_elements: Vec<_> = files
            .into_iter()
            .enumerate()
            .map(|(idx, path)| {
                let file_name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "???".to_string());
                let is_selected = selected == Some(idx);
                let theme = cx.theme();

                div()
                    .id(("file-entry", idx))
                    .flex()
                    .items_center()
                    .gap(px(6.))
                    .px(px(8.))
                    .py(px(4.))
                    .cursor_pointer()
                    .when(is_selected, |this| this.bg(theme.accent.opacity(0.15)))
                    .when(!is_selected, |this| this.hover(|s| s.bg(theme.secondary)))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _: &MouseDownEvent, _window, cx| {
                            this.selected_index = Some(idx);
                            let path = path.clone();
                            if let Some(panel) = this.panel.clone() {
                                let _ = panel.update(cx, |panel, cx| {
                                    if let Err(e) = panel.open_file(path, cx) {
                                        log::error!("Failed to open file: {}", e);
                                    }
                                });
                            }
                            cx.notify();
                        }),
                    )
                    .child(
                        div()
                            .text_sm()
                            .overflow_hidden()
                            .flex_1()
                            .when(is_selected, |this| {
                                this.text_color(theme.accent).font_weight(gpui::FontWeight::BOLD)
                            })
                            .when(!is_selected, |this| this.text_color(theme.foreground))
                            .child(file_name),
                    )
            })
            .collect();

        div()
            .flex()
            .flex_col()
            .h_full()
            .w(px(self.width))
            .bg(theme.secondary)
            .flex_shrink_0()
            .border_r_1()
            .border_color(theme.border)
            .child(
                div()
                    .px(px(8.))
                    .py(px(6.))
                    .text_xs()
                    .font_weight(gpui::FontWeight::BOLD)
                    .text_color(theme.muted_foreground)
                    .child(dir_name),
            )
            .child(
                div()
                    .id("file-browser-scroll")
                    .flex_1()
                    .overflow_y_scroll()
                    .track_scroll(&self.scroll_handle)
                    .when(has_files, |this| this.children(file_elements))
                    .when(!has_files, |this| {
                        this.child(
                            div()
                                .px(px(8.))
                                .py(px(8.))
                                .text_sm()
                                .text_color(theme.muted_foreground)
                                .child("No .md files"),
                        )
                    }),
            )
    }
}