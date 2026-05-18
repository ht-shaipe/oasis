#![allow(dead_code)]
use crate::panels::markdown_editor::model::DocumentState;
use crate::panels::markdown_editor::syntax::{SyntaxKind, SyntaxSpan, markdown_spans};
use crate::panels::markdown_editor::text_utils::ellipsize_chars;
use gpui::prelude::FluentBuilder as _;
use gpui::{
    App, Bounds, Context, Entity, FocusHandle, Focusable, FontStyle, FontWeight,
    HighlightStyle, Hsla, InteractiveElement, IntoElement, KeyDownEvent, MouseButton, MouseDownEvent,
    MouseMoveEvent, ParentElement, Render, ScrollHandle, StatefulInteractiveElement, Styled,
    StyledText, UnderlineStyle, Window, canvas, combine_highlights, div, fill, point, px,
    rgb, size,
};
use gpui_component::ActiveTheme;
use std::ops::Range;
use std::panic::AssertUnwindSafe;
use std::sync::Arc;
use std::time::Duration;

struct SearchCache {
    revision: u64,
    query: String,
    matches: Vec<Range<usize>>,
}

#[derive(Clone, Debug)]
struct ProjectionSegment {
    source: Range<usize>,
    display_start: usize,
    hidden: bool,
}

#[derive(Clone, Debug)]
struct DisplayProjection {
    display_text: String,
    source_len: usize,
    segments: Vec<ProjectionSegment>,
}

impl DisplayProjection {
    fn from_source(source: &str, spans: &[SyntaxSpan]) -> Self {
        let source_len = source.len();
        let hidden = merged_hidden_ranges(spans);
        if hidden.is_empty() {
            return Self {
                display_text: source.to_string(),
                source_len,
                segments: vec![ProjectionSegment {
                    source: 0..source_len,
                    display_start: 0,
                    hidden: false,
                }],
            };
        }

        let mut display_text = String::with_capacity(source_len);
        let mut segments = Vec::new();
        let mut source_ix = 0usize;
        let mut display_ix = 0usize;

        for hidden_range in hidden {
            if hidden_range.start > source_ix {
                let visible = source_ix..hidden_range.start;
                let slice = &source[visible.clone()];
                display_text.push_str(slice);
                segments.push(ProjectionSegment {
                    source: visible,
                    display_start: display_ix,
                    hidden: false,
                });
                display_ix += slice.len();
            }

            if hidden_range.end > hidden_range.start {
                segments.push(ProjectionSegment {
                    source: hidden_range.clone(),
                    display_start: display_ix,
                    hidden: true,
                });
            }
            source_ix = hidden_range.end;
        }

        if source_ix < source_len {
            let visible = source_ix..source_len;
            let slice = &source[visible.clone()];
            display_text.push_str(slice);
            segments.push(ProjectionSegment {
                source: visible,
                display_start: display_ix,
                hidden: false,
            });
        }

        if segments.is_empty() {
            segments.push(ProjectionSegment {
                source: 0..0,
                display_start: 0,
                hidden: false,
            });
        }

        Self {
            display_text,
            source_len,
            segments,
        }
    }

    fn source_to_display_bytes(&self, source_byte: usize) -> usize {
        let source_byte = source_byte.min(self.source_len);
        let mut mapped = 0usize;
        for segment in &self.segments {
            if source_byte < segment.source.start {
                return mapped;
            }
            if source_byte <= segment.source.end {
                if segment.hidden {
                    return segment.display_start;
                }
                return segment.display_start + (source_byte - segment.source.start);
            }

            mapped = if segment.hidden {
                segment.display_start
            } else {
                segment.display_start + segment.source.len()
            };
        }
        self.display_text.len()
    }

    fn display_to_source_byte(&self, display_byte: usize) -> usize {
        let display_byte = display_byte.min(self.display_text.len());
        for segment in &self.segments {
            if display_byte < segment.display_start {
                return segment.source.start;
            }
            if segment.hidden {
                continue;
            }

            let display_end = segment.display_start + segment.source.len();
            if display_byte <= display_end {
                return segment.source.start + (display_byte - segment.display_start);
            }
        }
        self.source_len
    }

    fn project_highlights(
        &self,
        highlights: Vec<(Range<usize>, HighlightStyle)>,
    ) -> Vec<(Range<usize>, HighlightStyle)> {
        highlights
            .into_iter()
            .filter_map(|(range, style)| {
                let start = self.source_to_display_bytes(range.start);
                let end = self.source_to_display_bytes(range.end);
                if start < end {
                    Some((start..end, style))
                } else {
                    None
                }
            })
            .collect()
    }
}

/// Inline markdown state — lightweight wrapper around parsed syntax spans.
#[derive(Clone)]
pub struct InlineMarkdownState {
    pub spans: Arc<Vec<SyntaxSpan>>,
    pub source_revision: u64,
}

impl InlineMarkdownState {
    pub fn new() -> Self {
        Self {
            spans: Arc::new(Vec::new()),
            source_revision: 0,
        }
    }

    /// Re-parse from the current document text if the revision has changed.
    pub fn update_if_stale(&mut self, text: &str, revision: u64) {
        if self.source_revision == revision {
            return;
        }
        self.spans = Arc::new(markdown_spans(text));
        self.source_revision = revision;
    }
}

pub struct EditorView {
    document: Entity<DocumentState>,
    inline_markdown: Entity<InlineMarkdownState>,
    focus_handle: Option<FocusHandle>,
    caret_visible: bool,
    blink_task: Option<gpui::Task<()>>,
    scroll_handle: ScrollHandle,
    cached_text: Option<(u64, String)>,
    search_active: bool,
    search_query: String,
    search_current_match: usize,
    cached_search: Option<SearchCache>,
    pending_scroll_to_byte: Option<usize>,
}

impl EditorView {
    pub fn new(
        document: Entity<DocumentState>,
        inline_markdown: Entity<InlineMarkdownState>,
    ) -> Self {
        Self {
            document,
            inline_markdown,
            focus_handle: None,
            caret_visible: true,
            blink_task: None,
            scroll_handle: ScrollHandle::new(),
            cached_text: None,
            search_active: false,
            search_query: String::new(),
            search_current_match: 0,
            cached_search: None,
            pending_scroll_to_byte: None,
        }
    }

    fn start_cursor_blink(&mut self, cx: &mut Context<Self>) {
        if self.blink_task.is_some() {
            return;
        }
        let entity = cx.entity();
        self.blink_task = Some(cx.spawn(async move |_editor, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(500))
                    .await;
                let _ = entity.update(cx, |view, cx| {
                    view.caret_visible = !view.caret_visible;
                    cx.notify();
                });
            }
        }));
    }

    fn selection_highlights(&self, doc: &DocumentState, cx: &App) -> Vec<(Range<usize>, HighlightStyle)> {
        doc.selection_bytes().map_or_else(Vec::new, |range| {
            vec![(
                range,
                HighlightStyle {
                    background_color: Some(hsla_with_alpha(cx.theme().selection, 0.18)),
                    ..Default::default()
                },
            )]
        })
    }

    fn inline_syntax_highlights(
        &self,
        spans: &[SyntaxSpan],
        cx: &App,
    ) -> Vec<(Range<usize>, HighlightStyle)> {
        let theme = cx.theme();
        spans
            .iter()
            .map(|span| {
                let hide_markers = is_inline_hidden_kind(span.kind);
                (span.range.clone(), syntax_style(span.kind, hide_markers, theme))
            })
            .collect()
    }

    fn current_text_and_revision(&mut self, cx: &mut Context<Self>) -> (String, u64) {
        let revision = self.document.read(cx).revision;
        if let Some((cached_revision, cached)) = &self.cached_text
            && *cached_revision == revision
        {
            return (cached.clone(), revision);
        }

        let text = self.document.read(cx).text();
        self.cached_text = Some((revision, text.clone()));
        (text, revision)
    }

    fn invalidate_search_cache(&mut self) {
        self.cached_search = None;
    }

    fn ensure_search_cache<'a>(&'a mut self, text: &str, revision: u64) -> &'a SearchCache {
        let query = self.search_query.clone();
        let should_recompute = self
            .cached_search
            .as_ref()
            .is_none_or(|cache| cache.revision != revision || cache.query != query);

        if should_recompute {
            let matches = find_all_matches_case_insensitive(text, &query);
            self.cached_search = Some(SearchCache {
                revision,
                query,
                matches,
            });
        }

        self.cached_search
            .as_ref()
            .expect("search cache should be initialized")
    }

    fn search_highlights(
        &mut self,
        text: &str,
        revision: u64,
    ) -> (Vec<(Range<usize>, HighlightStyle)>, usize) {
        if !self.search_active || self.search_query.is_empty() {
            return (Vec::new(), 0);
        }

        let matches = self.ensure_search_cache(text, revision).matches.clone();
        let count = matches.len();
        let style = HighlightStyle {
            background_color: Some(hsla_with_alpha(rgb(0xffd66b), 0.42)),
            ..Default::default()
        };

        (
            matches.into_iter().map(|range| (range, style)).collect(),
            count,
        )
    }

    fn activate_search(&mut self, cx: &mut Context<Self>) {
        self.search_active = true;

        if self.search_query.is_empty() {
            let seed_query = {
                let doc = self.document.read(cx);
                doc.selection_range().map(|range| doc.slice_chars(range))
            };

            if let Some(seed) = seed_query {
                if !seed.trim().is_empty() && !seed.contains('\n') {
                    self.search_query = seed;
                }
            }
        }

        self.search_current_match = 0;
        self.invalidate_search_cache();
        self.select_current_search_match(cx);
        cx.notify();
    }

    fn close_search(&mut self, cx: &mut Context<Self>) {
        self.search_active = false;
        cx.notify();
    }

    fn select_current_search_match(&mut self, cx: &mut Context<Self>) {
        if self.search_query.is_empty() {
            return;
        }

        let (text, revision) = self.current_text_and_revision(cx);
        let matches = self.ensure_search_cache(&text, revision).matches.clone();
        let match_range = {
            let total = matches.len();
            if total == 0 {
                None
            } else {
                if self.search_current_match >= total {
                    self.search_current_match = 0;
                }
                matches.get(self.search_current_match).cloned()
            }
        };

        if let Some(range) = match_range {
            let _ = self.document.update(cx, |doc, cx| {
                let start = doc.byte_to_char(range.start);
                let end = doc.byte_to_char(range.end);
                doc.set_selection(start, end);
                cx.notify();
            });
            self.pending_scroll_to_byte = Some(range.start);
        }
    }

    fn jump_search(&mut self, cx: &mut Context<Self>, forward: bool) {
        if self.search_query.is_empty() {
            return;
        }

        let (text, revision) = self.current_text_and_revision(cx);
        let total_matches = self.ensure_search_cache(&text, revision).matches.len();
        if total_matches == 0 {
            return;
        }

        if forward {
            self.search_current_match = (self.search_current_match + 1) % total_matches;
        } else if self.search_current_match == 0 {
            self.search_current_match = total_matches - 1;
        } else {
            self.search_current_match -= 1;
        }

        self.select_current_search_match(cx);
        cx.notify();
    }

    fn handle_search_key(&mut self, event: &KeyDownEvent, cx: &mut Context<Self>) {
        let key = event.keystroke.key.to_lowercase();
        let modifiers = event.keystroke.modifiers;
        let is_cmd = modifiers.platform || modifiers.control;
        let shift = modifiers.shift;

        if key == "escape" {
            self.close_search(cx);
            return;
        }

        if key == "enter" || key == "return" {
            self.jump_search(cx, !shift);
            return;
        }

        if key == "backspace" {
            pop_last_char(&mut self.search_query);
            self.search_current_match = 0;
            self.invalidate_search_cache();
            self.select_current_search_match(cx);
            cx.notify();
            return;
        }

        if is_cmd {
            return;
        }

        if let Some(raw) = &event.keystroke.key_char {
            if raw != "\n" && raw != "\r" && !raw.is_empty() {
                self.search_query.push_str(raw);
                self.search_current_match = 0;
                self.invalidate_search_cache();
                self.select_current_search_match(cx);
                cx.notify();
            }
        }
    }

    fn reveal_pending_byte(
        &mut self,
        text_layout: &gpui::TextLayout,
        projection: &DisplayProjection,
        window: &mut Window,
    ) {
        let Some(target_source_byte) = self.pending_scroll_to_byte else {
            return;
        };
        let target_display_byte = projection.source_to_display_bytes(target_source_byte);

        let Some(target_pos) = std::panic::catch_unwind(AssertUnwindSafe(|| {
            text_layout.position_for_index(target_display_byte)
        }))
        .ok()
        .flatten() else {
            return;
        };

        let line_height = std::panic::catch_unwind(AssertUnwindSafe(|| text_layout.line_height()))
            .ok()
            .unwrap_or(px(0.));
        if line_height <= px(0.) {
            return;
        }

        let max = self.scroll_handle.max_offset();
        let offset = self.scroll_handle.offset();
        let bounds = self.scroll_handle.bounds();
        let viewport_height = bounds.size.height;
        if viewport_height <= px(0.) {
            return;
        }

        let padding = px(28.);
        let visible_top = -offset.y;
        let visible_bottom = visible_top + viewport_height;
        let target_top = target_pos.y;
        let target_bottom = target_pos.y + line_height;

        let mut new_offset_y = offset.y;
        if target_top < visible_top + padding {
            new_offset_y = -(target_top - padding);
        } else if target_bottom > visible_bottom - padding {
            new_offset_y = -(target_bottom - viewport_height + padding);
        }

        new_offset_y = new_offset_y.clamp(-max.height, px(0.));
        self.scroll_handle.set_offset(point(offset.x, new_offset_y));
        self.pending_scroll_to_byte = None;
        window.refresh();
    }

    /// Public accessor for the document entity.
    pub fn document(&self) -> &Entity<DocumentState> {
        &self.document
    }
}

impl Focusable for EditorView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle
            .clone()
            .expect("focus handle should be initialized during render")
    }
}

impl Render for EditorView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.start_cursor_blink(cx);
        let focus_handle = self
            .focus_handle
            .get_or_insert_with(|| {
                let handle = cx.focus_handle();
                handle.focus(window);
                handle
            })
            .clone();

        let is_focused = focus_handle.is_focused(window);
        let theme = cx.theme().clone();

        let (text_owned, doc_revision) = {
            let doc = self.document.read(cx);
            let rev = doc.revision;
            if let Some((cached_rev, ref text)) = self.cached_text {
                if cached_rev == rev {
                    (text.clone(), rev)
                } else {
                    (doc.text(), rev)
                }
            } else {
                (doc.text(), rev)
            }
        };

        if self.cached_text.as_ref().map(|(r, _)| *r) != Some(doc_revision) {
            self.cached_text = Some((doc_revision, text_owned.clone()));
        }

        // Update inline markdown if stale
        self.inline_markdown.update(cx, |md, _| {
            md.update_if_stale(&text_owned, doc_revision);
        });

        let doc = self.document.read(cx);
        let cursor_source_byte = doc.char_to_byte(doc.cursor);
        let show_caret = doc.selection.is_none();
        let draw_caret = show_caret && is_focused && self.caret_visible;
        let inline_spans = {
            let inline = self.inline_markdown.read(cx);
            inline.spans.clone()
        };
        let projection = Arc::new(DisplayProjection::from_source(
            &text_owned,
            inline_spans.as_ref(),
        ));
        let cursor_display_byte = projection.source_to_display_bytes(cursor_source_byte);
        let syntax_highlights =
            projection.project_highlights(self.inline_syntax_highlights(inline_spans.as_ref(), cx));
        let (search_highlights, search_match_count) =
            self.search_highlights(&text_owned, doc_revision);
        let search_highlights = projection.project_highlights(search_highlights);
        let selection_highlights = projection.project_highlights(self.selection_highlights(&doc, cx));

        let syntax_and_search = if search_highlights.is_empty() {
            syntax_highlights
        } else {
            combine_highlights(syntax_highlights, search_highlights).collect()
        };

        let all_highlights = if selection_highlights.is_empty() {
            syntax_and_search
        } else {
            combine_highlights(syntax_and_search, selection_highlights).collect()
        };

        let mut styled = StyledText::new(projection.display_text.clone());
        let safe_highlights = sanitize_highlights(&projection.display_text, all_highlights);
        if !safe_highlights.is_empty() {
            styled = std::panic::catch_unwind(AssertUnwindSafe(|| {
                StyledText::new(projection.display_text.clone()).with_highlights(safe_highlights)
            }))
            .unwrap_or_else(|_| StyledText::new(projection.display_text.clone()));
        }

        let text_layout = styled.layout().clone();
        self.reveal_pending_byte(&text_layout, projection.as_ref(), window);

        let search_match_display = if search_match_count == 0 {
            0
        } else {
            self.search_current_match.min(search_match_count - 1) + 1
        };

        let font_size = theme.mono_font_size;

        div()
            .id("editor_scroll")
            .relative()
            .flex_1()
            .min_w(px(0.))
            .min_h(px(0.))
            .bg(theme.background)
            .p(px(18.))
            .text_size(font_size)
            .text_color(theme.foreground)
            .font_family(theme.mono_font_family.clone())
            .overflow_y_scroll()
            .overflow_x_hidden()
            .scrollbar_width(px(10.))
            .track_scroll(&self.scroll_handle)
            .track_focus(&focus_handle)
            .on_mouse_down(MouseButton::Left, {
                let focus_handle = focus_handle.clone();
                let doc_handle = self.document.clone();
                let layout_for_event = text_layout.clone();
                let projection_for_event = projection.clone();
                move |event: &MouseDownEvent, window: &mut Window, cx_app: &mut App| {
                    focus_handle.focus(window);
                    let _ = doc_handle.update(cx_app, |doc, cx| {
                        let byte_idx = std::panic::catch_unwind(AssertUnwindSafe(|| {
                            layout_for_event.index_for_position(event.position)
                        }))
                        .ok()
                        .map(|res| match res {
                            Ok(ix) => ix,
                            Err(ix) => ix,
                        });
                        let source_byte =
                            byte_idx.map(|b| projection_for_event.display_to_source_byte(b));
                        if let Some(byte_idx) = source_byte.map(|b| doc.byte_to_char(b)) {
                            if event.modifiers.shift {
                                let anchor = doc.selection_anchor.unwrap_or(doc.cursor);
                                doc.set_selection(anchor, byte_idx);
                            } else {
                                doc.set_cursor(byte_idx);
                            }
                            cx.notify();
                        }
                    });
                }
            })
            .on_mouse_move({
                let doc_handle = self.document.clone();
                let layout_for_event = text_layout.clone();
                let projection_for_event = projection.clone();
                move |event: &MouseMoveEvent, _window: &mut Window, cx_app: &mut App| {
                    if !event.dragging() {
                        return;
                    }
                    let _ = doc_handle.update(cx_app, |doc, cx| {
                        let byte_idx = std::panic::catch_unwind(AssertUnwindSafe(|| {
                            layout_for_event.index_for_position(event.position)
                        }))
                        .ok()
                        .map(|res| match res {
                            Ok(ix) => ix,
                            Err(ix) => ix,
                        });
                        let source_byte =
                            byte_idx.map(|b| projection_for_event.display_to_source_byte(b));
                        if let Some(byte_idx) = source_byte.map(|b| doc.byte_to_char(b)) {
                            let anchor = doc.selection_anchor.unwrap_or(doc.cursor);
                            doc.set_selection(anchor, byte_idx);
                            cx.notify();
                        }
                    });
                }
            })
            .on_key_down({
                let focus = focus_handle.clone();
                cx.listener(move |this, event: &KeyDownEvent, window, cx| {
                    if !focus.is_focused(window) {
                        return;
                    }

                    let key = event.keystroke.key.to_lowercase();
                    let modifiers = event.keystroke.modifiers;
                    let is_cmd = modifiers.platform || modifiers.control;
                    let shift = modifiers.shift;

                    if is_cmd && key == "f" {
                        this.activate_search(cx);
                        return;
                    }

                    if is_cmd && key == "g" {
                        if !this.search_active {
                            this.activate_search(cx);
                        } else {
                            this.jump_search(cx, !shift);
                        }
                        return;
                    }

                    if this.search_active {
                        this.handle_search_key(event, cx);
                        return;
                    }

                    if is_cmd {
                        return;
                    }

                    if key == "pageup" || key == "pagedown" {
                        let max = this.scroll_handle.max_offset();
                        let offset = this.scroll_handle.offset();
                        let bounds = this.scroll_handle.bounds();
                        let page = bounds.size.height;
                        if page > px(0.) {
                            let amount = page * 0.9;
                            let delta = if key == "pagedown" { -amount } else { amount };
                            let mut new_offset = offset;
                            new_offset.y = (new_offset.y + delta).clamp(-max.height, px(0.));
                            this.scroll_handle
                                .set_offset(point(new_offset.x, new_offset.y));
                            window.refresh();
                        }
                        return;
                    }

                    let _ = this.document.update(cx, |doc, cx_doc| {
                        let len = doc.rope.len_chars();
                        match key.as_str() {
                            "backspace" => {
                                doc.begin_edit();
                                if doc.delete_selection().is_some() {
                                    doc.commit_edit();
                                    cx_doc.notify();
                                    return;
                                }
                                if doc.cursor > 0 && len > 0 {
                                    let start = doc.cursor.saturating_sub(1);
                                    doc.delete_range(start..doc.cursor);
                                    doc.cursor = start;
                                    doc.commit_edit();
                                    cx_doc.notify();
                                }
                            }
                            "delete" => {
                                doc.begin_edit();
                                if doc.delete_selection().is_some() {
                                    doc.commit_edit();
                                    cx_doc.notify();
                                    return;
                                }
                                if doc.cursor < len {
                                    let end = (doc.cursor + 1).min(len);
                                    doc.delete_range(doc.cursor..end);
                                    doc.commit_edit();
                                    cx_doc.notify();
                                }
                            }
                            "enter" | "return" => {
                                doc.begin_edit();
                                doc.delete_selection();
                                doc.insert(doc.cursor, "\n");
                                doc.cursor += 1;
                                doc.commit_edit();
                                cx_doc.notify();
                            }
                            "left" | "arrowleft" => {
                                if shift {
                                    let anchor = doc.selection_anchor.unwrap_or(doc.cursor);
                                    if doc.cursor > 0 {
                                        doc.set_selection(anchor, doc.cursor - 1);
                                        cx_doc.notify();
                                    }
                                } else if doc.cursor > 0 {
                                    doc.cursor -= 1;
                                    doc.clear_selection();
                                    cx_doc.notify();
                                }
                            }
                            "right" | "arrowright" => {
                                if shift {
                                    let anchor = doc.selection_anchor.unwrap_or(doc.cursor);
                                    if doc.cursor < len {
                                        doc.set_selection(anchor, doc.cursor + 1);
                                        cx_doc.notify();
                                    }
                                } else if doc.cursor < len {
                                    doc.cursor += 1;
                                    doc.clear_selection();
                                    cx_doc.notify();
                                }
                            }
                            "up" | "arrowup" => {
                                let cursor = doc.cursor.min(len);
                                let line_idx = doc.rope.char_to_line(cursor);
                                if line_idx == 0 {
                                    return;
                                }
                                let line_start = doc.rope.line_to_char(line_idx);
                                let col = cursor.saturating_sub(line_start);
                                let target_line = line_idx - 1;
                                let target_start = doc.rope.line_to_char(target_line);
                                let target_len = doc.rope.line(target_line).len_chars();
                                let max_col = if target_line + 1 < doc.rope.len_lines() {
                                    target_len.saturating_sub(1)
                                } else {
                                    target_len
                                };
                                let new_cursor = target_start + col.min(max_col);

                                if shift {
                                    let anchor = doc.selection_anchor.unwrap_or(cursor);
                                    doc.set_selection(anchor, new_cursor);
                                } else {
                                    doc.cursor = new_cursor;
                                    doc.clear_selection();
                                }
                                cx_doc.notify();
                            }
                            "down" | "arrowdown" => {
                                let cursor = doc.cursor.min(len);
                                let line_idx = doc.rope.char_to_line(cursor);
                                if line_idx + 1 >= doc.rope.len_lines() {
                                    return;
                                }
                                let line_start = doc.rope.line_to_char(line_idx);
                                let col = cursor.saturating_sub(line_start);
                                let target_line = line_idx + 1;
                                let target_start = doc.rope.line_to_char(target_line);
                                let target_len = doc.rope.line(target_line).len_chars();
                                let max_col = if target_line + 1 < doc.rope.len_lines() {
                                    target_len.saturating_sub(1)
                                } else {
                                    target_len
                                };
                                let new_cursor = target_start + col.min(max_col);

                                if shift {
                                    let anchor = doc.selection_anchor.unwrap_or(cursor);
                                    doc.set_selection(anchor, new_cursor);
                                } else {
                                    doc.cursor = new_cursor;
                                    doc.clear_selection();
                                }
                                cx_doc.notify();
                            }
                            _ => {
                                if let Some(ch) = event
                                    .keystroke
                                    .key_char
                                    .as_ref()
                                    .and_then(|s| s.chars().next())
                                {
                                    let insert = ch.to_string();
                                    doc.begin_edit();
                                    doc.delete_selection();
                                    doc.insert(doc.cursor, &insert);
                                    doc.cursor = doc.cursor.saturating_add(insert.chars().count());
                                    doc.commit_edit();
                                    cx_doc.notify();
                                } else if let Some(raw) = &event.keystroke.key_char {
                                    if raw == "\n" {
                                        doc.begin_edit();
                                        doc.delete_selection();
                                        doc.insert(doc.cursor, "\n");
                                        doc.cursor += 1;
                                        doc.commit_edit();
                                        cx_doc.notify();
                                    }
                                }
                            }
                        }
                    });
                })
            })
            .child(
                div().relative().child(styled).child({
                    let canvas_theme = theme.clone();
                    canvas(
                        move |_, _, _| {},
                        move |_bounds: Bounds<_>, (), window: &mut Window, _cx: &mut App| {
                            if !draw_caret {
                                return;
                            }

                            let caret_pos = std::panic::catch_unwind(AssertUnwindSafe(|| {
                                text_layout.position_for_index(cursor_display_byte)
                            }))
                            .ok()
                            .flatten();
                            let Some(caret_pos) = caret_pos else {
                                return;
                            };

                            let line_height = std::panic::catch_unwind(AssertUnwindSafe(|| {
                                text_layout.line_height()
                            }))
                            .ok()
                            .unwrap_or(px(0.));
                            if line_height <= px(0.) {
                                return;
                            }

                            window.paint_quad(fill(
                                Bounds {
                                    origin: point(caret_pos.x, caret_pos.y),
                                    size: size(px(1.), line_height),
                                },
                                gpui::Background::from(canvas_theme.accent_foreground),
                            ));
                        },
                    )
                    .absolute()
                    .top_0()
                    .left_0()
                    .size_full()
                }),
            )
            .when(self.search_active, |this| {
                this.child(
                    div()
                        .absolute()
                        .top(px(8.))
                        .right(px(12.))
                        .flex()
                        .items_center()
                        .gap_2()
                        .px(px(10.))
                        .py(px(6.))
                        .rounded(px(6.))
                        .bg(theme.secondary)
                        .border_1()
                        .border_color(theme.border)
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::BOLD)
                                .text_color(theme.muted_foreground)
                                .child("FIND"),
                        )
                        .child(
                            div()
                                .text_sm()
                                .max_w(px(300.))
                                .overflow_hidden()
                                .text_color(theme.foreground)
                                .child(if self.search_query.is_empty() {
                                    "Type to search".to_string()
                                } else {
                                    ellipsize_chars(&self.search_query, 80)
                                }),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme.muted_foreground)
                                .child(format!("{}/{}", search_match_display, search_match_count)),
                        ),
                )
            })
    }
}

// ── Syntax styling using gpui_component theme ──────────────────────────

fn syntax_style(kind: SyntaxKind, hide_markers: bool, theme: &gpui_component::Theme) -> HighlightStyle {
    if hide_markers {
        let hidden_color = match kind {
            SyntaxKind::InlineCodeMarker => theme.muted,
            _ => theme.background,
        };
        let hidden_background = match kind {
            SyntaxKind::InlineCodeMarker => Some(theme.muted.into()),
            _ => None,
        };
        return HighlightStyle {
            color: Some(hidden_color.into()),
            background_color: hidden_background,
            ..Default::default()
        };
    }

    match kind {
        SyntaxKind::HeadingMarker => HighlightStyle {
            color: Some(theme.muted_foreground.into()),
            font_weight: Some(FontWeight::BOLD),
            ..Default::default()
        },
        SyntaxKind::HeadingText => HighlightStyle {
            color: Some(theme.accent_foreground.into()),
            font_weight: Some(FontWeight::BOLD),
            ..Default::default()
        },
        SyntaxKind::QuoteMarker => HighlightStyle {
            color: Some(theme.muted_foreground.into()),
            ..Default::default()
        },
        SyntaxKind::ListMarker | SyntaxKind::TaskMarker => HighlightStyle {
            color: Some(theme.accent_foreground.into()),
            font_weight: Some(FontWeight::BOLD),
            ..Default::default()
        },
        SyntaxKind::CodeFence => HighlightStyle {
            color: Some(theme.muted_foreground.into()),
            ..Default::default()
        },
        SyntaxKind::InlineCodeMarker | SyntaxKind::InlineCode => HighlightStyle {
            color: Some(rgb(0x1f6f8b).into()),
            background_color: Some(theme.muted.into()),
            ..Default::default()
        },
        SyntaxKind::LinkTextDelimiter | SyntaxKind::LinkUrlDelimiter => HighlightStyle {
            color: Some(theme.muted_foreground.into()),
            ..Default::default()
        },
        SyntaxKind::LinkText => HighlightStyle {
            color: Some(theme.accent_foreground.into()),
            underline: Some(UnderlineStyle {
                thickness: px(1.),
                color: Some(theme.accent_foreground.into()),
                wavy: false,
            }),
            ..Default::default()
        },
        SyntaxKind::LinkUrl => HighlightStyle {
            color: Some(theme.muted_foreground.into()),
            font_style: Some(FontStyle::Italic),
            ..Default::default()
        },
        SyntaxKind::EmphasisMarker => HighlightStyle {
            color: Some(theme.muted_foreground.into()),
            ..Default::default()
        },
        SyntaxKind::EmphasisText => HighlightStyle {
            font_style: Some(FontStyle::Italic),
            ..Default::default()
        },
        SyntaxKind::StrongText => HighlightStyle {
            font_weight: Some(FontWeight::BOLD),
            ..Default::default()
        },
    }
}

fn is_inline_hidden_kind(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::HeadingMarker
            | SyntaxKind::QuoteMarker
            | SyntaxKind::InlineCodeMarker
            | SyntaxKind::EmphasisMarker
            | SyntaxKind::CodeFence
            | SyntaxKind::LinkTextDelimiter
            | SyntaxKind::LinkUrlDelimiter
            | SyntaxKind::LinkUrl
    )
}

fn merged_hidden_ranges(spans: &[SyntaxSpan]) -> Vec<Range<usize>> {
    let mut hidden = spans
        .iter()
        .filter(|span| is_inline_hidden_kind(span.kind))
        .map(|span| span.range.clone())
        .collect::<Vec<_>>();
    if hidden.is_empty() {
        return hidden;
    }

    hidden.sort_by_key(|r| (r.start, r.end));
    let mut merged = Vec::with_capacity(hidden.len());
    let mut current = hidden[0].clone();

    for range in hidden.into_iter().skip(1) {
        if range.start <= current.end {
            current.end = current.end.max(range.end);
        } else {
            merged.push(current);
            current = range;
        }
    }
    merged.push(current);
    merged
}

fn pop_last_char(s: &mut String) {
    if let Some((idx, _)) = s.char_indices().next_back() {
        s.truncate(idx);
    }
}

fn find_all_matches_case_insensitive(haystack: &str, needle: &str) -> Vec<Range<usize>> {
    if needle.is_empty() {
        return Vec::new();
    }

    let hay_bytes = haystack.as_bytes();
    let needle_bytes = needle.as_bytes();
    if needle_bytes.len() > hay_bytes.len() {
        return Vec::new();
    }

    let needle_folded = needle_bytes
        .iter()
        .map(u8::to_ascii_lowercase)
        .collect::<Vec<_>>();

    let mut matches = Vec::new();
    let mut start = 0usize;

    while start + needle_folded.len() <= hay_bytes.len() {
        if !haystack.is_char_boundary(start) {
            start += 1;
            continue;
        }

        let end = start + needle_folded.len();
        if !haystack.is_char_boundary(end) {
            start += 1;
            continue;
        }

        let mut matched = true;
        for idx in 0..needle_folded.len() {
            if hay_bytes[start + idx].to_ascii_lowercase() != needle_folded[idx] {
                matched = false;
                break;
            }
        }

        if matched {
            matches.push(start..end);
            start = end;
        } else {
            start += 1;
        }
    }

    matches
}

fn hsla_with_alpha(color: impl Into<Hsla>, alpha: f32) -> Hsla {
    let mut hsla: Hsla = color.into();
    hsla.a = alpha;
    hsla
}

fn sanitize_highlights(
    text: &str,
    highlights: Vec<(Range<usize>, HighlightStyle)>,
) -> Vec<(Range<usize>, HighlightStyle)> {
    let len = text.len();
    highlights
        .into_iter()
        .filter_map(|(range, style)| {
            if range.start >= range.end || range.start >= len {
                return None;
            }

            let mut start = range.start.min(len);
            let mut end = range.end.min(len);

            while start > 0 && !text.is_char_boundary(start) {
                start -= 1;
            }
            while end < len && !text.is_char_boundary(end) {
                end += 1;
            }

            if start < end && text.is_char_boundary(start) && text.is_char_boundary(end) {
                Some((start..end, style))
            } else {
                None
            }
        })
        .collect()
}
