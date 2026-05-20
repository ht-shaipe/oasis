use crate::model::document::EditDelta;
use crate::services::syntax::{SyntaxSpan, markdown_spans};
use std::time::Instant;

#[derive(Clone, Debug)]
pub struct InlineParseResult { pub spans: Vec<SyntaxSpan>, pub parse_millis: f32 }

pub fn compute_inline_spans(source: &str, _last_edit: Option<&EditDelta>) -> InlineParseResult {
    let started = Instant::now();
    let spans = markdown_spans(source);
    let parse_millis = started.elapsed().as_secs_f32() * 1000.0;
    InlineParseResult { spans, parse_millis }
}