use std::ops::Range;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SyntaxKind {
    HeadingMarker, HeadingText, QuoteMarker, ListMarker, TaskMarker,
    CodeFence, InlineCodeMarker, InlineCode,
    LinkTextDelimiter, LinkText, LinkUrlDelimiter, LinkUrl,
    EmphasisMarker, EmphasisText, StrongText,
}

#[derive(Clone, Debug)]
pub struct SyntaxSpan { pub range: Range<usize>, pub kind: SyntaxKind }

pub fn markdown_spans(source: &str) -> Vec<SyntaxSpan> {
    let mut spans = Vec::new();
    let mut offset = 0usize;

    for raw_line in source.split_inclusive('\n') {
        let line = raw_line.trim_end_matches('\n').trim_end_matches('\r');
        let line_len = line.len();
        if line_len == 0 { offset += raw_line.len(); continue; }

        let leading = leading_whitespace_bytes(line);
        let content = &line[leading..];
        let content_start = offset + leading;
        let mut skip_inline = false;

        if let Some(fence_len) = fence_prefix_len(content) {
            spans.push(SyntaxSpan { range: content_start..(content_start + fence_len), kind: SyntaxKind::CodeFence });
            if content.len() > fence_len {
                spans.push(SyntaxSpan { range: (content_start + fence_len)..(content_start + content.len()), kind: SyntaxKind::CodeFence });
            }
            skip_inline = true;
        }

        if !skip_inline {
            if let Some((marker_len, _)) = heading_prefix(content) {
                spans.push(SyntaxSpan { range: content_start..(content_start + marker_len), kind: SyntaxKind::HeadingMarker });
                let text_start = content_start + marker_len;
                if text_start < content_start + content.len() {
                    spans.push(SyntaxSpan { range: text_start..(content_start + content.len()), kind: SyntaxKind::HeadingText });
                }
            }
            if content.starts_with('>') {
                spans.push(SyntaxSpan { range: content_start..(content_start + 1), kind: SyntaxKind::QuoteMarker });
            }
            if let Some(marker_len) = task_marker_len(content) {
                spans.push(SyntaxSpan { range: content_start..(content_start + marker_len), kind: SyntaxKind::TaskMarker });
            } else if let Some(marker_len) = list_marker_len(content) {
                spans.push(SyntaxSpan { range: content_start..(content_start + marker_len), kind: SyntaxKind::ListMarker });
            }
            scan_inline(line, offset, &mut spans);
        }
        offset += raw_line.len();
    }
    spans
}

fn leading_whitespace_bytes(line: &str) -> usize {
    line.char_indices().find_map(|(idx, ch)| if ch.is_whitespace() { None } else { Some(idx) }).unwrap_or(line.len())
}

fn fence_prefix_len(content: &str) -> Option<usize> {
    if content.starts_with("```") || content.starts_with("~~~") { Some(3) } else { None }
}

fn heading_prefix(content: &str) -> Option<(usize, usize)> {
    let bytes = content.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() && bytes[i] == b'#' { i += 1; }
    let hash_count = i;
    if hash_count == 0 || hash_count > 6 { return None; }
    if i < bytes.len() && bytes[i].is_ascii_whitespace() {
        while i < bytes.len() && bytes[i].is_ascii_whitespace() { i += 1; }
        return Some((i, hash_count));
    }
    None
}

fn task_marker_len(content: &str) -> Option<usize> {
    let b = content.as_bytes();
    if b.len() < 6 { return None; }
    if (b[0] == b'-' || b[0] == b'*' || b[0] == b'+') && b[1] == b' ' && b[2] == b'['
        && (b[3] == b' ' || b[3] == b'x' || b[3] == b'X') && b[4] == b']' && b[5].is_ascii_whitespace()
    { Some(6) } else { None }
}

fn list_marker_len(content: &str) -> Option<usize> {
    let b = content.as_bytes();
    if b.len() >= 2 && (b[0] == b'-' || b[0] == b'*' || b[0] == b'+') && b[1].is_ascii_whitespace() { return Some(2); }
    let mut i = 0usize;
    while i < b.len() && b[i].is_ascii_digit() { i += 1; }
    if i > 0 && i + 1 < b.len() && b[i] == b'.' && b[i + 1].is_ascii_whitespace() { Some(i + 2) } else { None }
}

fn scan_inline(line: &str, line_start: usize, spans: &mut Vec<SyntaxSpan>) {
    let bytes = line.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        match bytes[i] {
            b'`' => {
                if let Some(close) = find_next_byte(bytes, i + 1, b'`') {
                    spans.push(SyntaxSpan { range: (line_start + i)..(line_start + i + 1), kind: SyntaxKind::InlineCodeMarker });
                    if close > i + 1 { spans.push(SyntaxSpan { range: (line_start + i + 1)..(line_start + close), kind: SyntaxKind::InlineCode }); }
                    spans.push(SyntaxSpan { range: (line_start + close)..(line_start + close + 1), kind: SyntaxKind::InlineCodeMarker });
                    i = close + 1; continue;
                }
            }
            b'[' => {
                if let Some(cb) = find_next_byte(bytes, i + 1, b']') {
                    let op = cb + 1;
                    if op < bytes.len() && bytes[op] == b'(' && let Some(cp) = find_next_byte(bytes, op + 1, b')') {
                        spans.push(SyntaxSpan { range: (line_start + i)..(line_start + i + 1), kind: SyntaxKind::LinkTextDelimiter });
                        if cb > i + 1 { spans.push(SyntaxSpan { range: (line_start + i + 1)..(line_start + cb), kind: SyntaxKind::LinkText }); }
                        spans.push(SyntaxSpan { range: (line_start + cb)..(line_start + cb + 1), kind: SyntaxKind::LinkTextDelimiter });
                        spans.push(SyntaxSpan { range: (line_start + op)..(line_start + op + 1), kind: SyntaxKind::LinkUrlDelimiter });
                        if cp > op + 1 { spans.push(SyntaxSpan { range: (line_start + op + 1)..(line_start + cp), kind: SyntaxKind::LinkUrl }); }
                        spans.push(SyntaxSpan { range: (line_start + cp)..(line_start + cp + 1), kind: SyntaxKind::LinkUrlDelimiter });
                        i = cp + 1; continue;
                    }
                }
            }
            b'*' | b'_' => {
                let marker = bytes[i];
                let marker_len = if i + 1 < bytes.len() && bytes[i + 1] == marker { 2 } else { 1 };
                if let Some(close) = find_emphasis_close(bytes, i + marker_len, marker, marker_len) {
                    spans.push(SyntaxSpan { range: (line_start + i)..(line_start + i + marker_len), kind: SyntaxKind::EmphasisMarker });
                    if close > i + marker_len {
                        spans.push(SyntaxSpan { range: (line_start + i + marker_len)..(line_start + close),
                            kind: if marker_len == 2 { SyntaxKind::StrongText } else { SyntaxKind::EmphasisText } });
                    }
                    spans.push(SyntaxSpan { range: (line_start + close)..(line_start + close + marker_len), kind: SyntaxKind::EmphasisMarker });
                    i = close + marker_len; continue;
                }
            }
            _ => {}
        }
        i += 1;
    }
}

fn find_next_byte(bytes: &[u8], start: usize, needle: u8) -> Option<usize> {
    bytes[start..].iter().position(|b| *b == needle).map(|pos| start + pos)
}

fn find_emphasis_close(bytes: &[u8], start: usize, marker: u8, marker_len: usize) -> Option<usize> {
    let mut i = start;
    while i + marker_len <= bytes.len() {
        if bytes[i] == marker {
            if marker_len == 1 { return Some(i); }
            if i + 1 < bytes.len() && bytes[i + 1] == marker { return Some(i); }
        }
        i += 1;
    }
    None
}