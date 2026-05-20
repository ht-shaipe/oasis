/// UTF-8-safe character-based ellipsizing helper for dynamic UI labels.
pub fn ellipsize_chars(text: &str, max_chars: usize) -> String {
    if max_chars == 0 { return String::new(); }
    let mut char_count = 0usize;
    let mut cutoff = text.len();
    for (idx, _) in text.char_indices() {
        if char_count == max_chars { cutoff = idx; break; }
        char_count += 1;
    }
    if cutoff == text.len() { return text.to_string(); }
    let mut output = String::with_capacity(cutoff + 3);
    output.push_str(&text[..cutoff]);
    output.push('…');
    output
}