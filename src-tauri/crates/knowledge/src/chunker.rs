const CHUNK_SIZE: usize = 512;
const CHUNK_OVERLAP: usize = 64;
const MIN_CHUNK_SIZE: usize = 100;

pub struct TextChunk {
    pub content: String,
    pub index: i32,
    pub line_start: i32,
    pub line_end: i32,
}

pub fn chunk_text(content: &str) -> Vec<TextChunk> {
    let paragraphs: Vec<&str> = split_paragraphs(content);

    let mut chunks = Vec::new();
    let mut current = String::new();
    let mut chunk_start_line = 0i32;
    let mut current_line = 0i32;
    let mut chunk_index = 0i32;

    for para in &paragraphs {
        let para_lines = para.lines().count() as i32;

        if current.len() + para.len() > CHUNK_SIZE && !current.is_empty() {
            let chunk_end_line = current_line;
            chunks.push(TextChunk {
                content: current.trim().to_string(),
                index: chunk_index,
                line_start: chunk_start_line,
                line_end: chunk_end_line,
            });
            chunk_index += 1;

            let overlap_text = take_overlap(&current, CHUNK_OVERLAP);
            let overlap_line_count = overlap_text.lines().count() as i32;
            current = overlap_text;
            chunk_start_line = chunk_end_line - overlap_line_count;
        }

        if !current.is_empty() && !current.ends_with('\n') {
            current.push('\n');
        }
        current.push_str(para);
        current_line += para_lines;
    }

    if !current.trim().is_empty() && current.trim().len() >= MIN_CHUNK_SIZE {
        chunks.push(TextChunk {
            content: current.trim().to_string(),
            index: chunk_index,
            line_start: chunk_start_line,
            line_end: current_line,
        });
    } else if !current.trim().is_empty() && !chunks.is_empty() {
        let last = chunks.last_mut().unwrap();
        last.content.push_str("\n\n");
        last.content.push_str(current.trim());
        last.line_end = current_line;
    } else if !current.trim().is_empty() {
        chunks.push(TextChunk {
            content: current.trim().to_string(),
            index: chunk_index,
            line_start: chunk_start_line,
            line_end: current_line,
        });
    }

    chunks
}

fn split_paragraphs(content: &str) -> Vec<&str> {
    let mut paragraphs = Vec::new();
    let mut start = 0;
    let mut in_blank = false;

    for (i, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            if !in_blank && i > start {
                let para = content.lines().skip(start).take(i - start).collect::<Vec<_>>().join("\n");
                if !para.trim().is_empty() {
                    paragraphs.push(leak_str(content, start, i));
                }
                start = i + 1;
            } else {
                start = i + 1;
            }
            in_blank = true;
        } else {
            in_blank = false;
        }
    }

    if start < content.lines().count() {
        let remaining: Vec<&str> = content.lines().skip(start).collect();
        let para = remaining.join("\n");
        if !para.trim().is_empty() {
            paragraphs.push(leak_str(content, start, content.lines().count()));
        }
    }

    if paragraphs.is_empty() && !content.trim().is_empty() {
        paragraphs.push(content);
    }

    paragraphs
}

fn leak_str(content: &str, start_line: usize, end_line: usize) -> &'static str {
    let lines: Vec<&str> = content.lines().collect();
    let joined = lines[start_line..end_line.min(lines.len())].join("\n");
    Box::leak(joined.into_boxed_str())
}

fn take_overlap(text: &str, overlap_chars: usize) -> String {
    if text.len() <= overlap_chars {
        return text.to_string();
    }
    let start = text.len() - overlap_chars;
    let start = (0..=start).rev().find(|&i| text.is_char_boundary(i)).unwrap_or(0);
    let slice = &text[start..];
    if let Some(pos) = slice.find('\n') {
        slice[pos + 1..].to_string()
    } else {
        slice.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_simple() {
        let content = "Hello world\n\nThis is a test paragraph that should be in its own chunk because it has some content.\n\nAnother paragraph here.";
        let chunks = chunk_text(content);
        assert!(!chunks.is_empty());
        for chunk in &chunks {
            assert!(!chunk.content.trim().is_empty());
        }
    }

    #[test]
    fn test_chunk_single_para() {
        let content = "A ".repeat(600);
        let chunks = chunk_text(&content);
        assert!(chunks.len() >= 2);
    }
}
