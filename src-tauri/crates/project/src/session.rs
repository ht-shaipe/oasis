//! JSONL session parsing — shared across all CLI agent adapters.
//!
//! Ported from Jishu Hub `session.rs` with macOS adaptations.
//! Provides `smart_summary`, `parse_ai_title`, `merge_tool_results`,
//! `parse_message`, and `load_session`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;

// ── Content block types ─────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        #[serde(default)]
        input: serde_json::Value,
    },
    #[serde(rename = "tool_result")]
    ToolResult {
        #[serde(rename = "tool_use_id")]
        tool_use_id: String,
        content: serde_json::Value,
        #[serde(default)]
        is_error: bool,
    },
    #[serde(rename = "thinking")]
    Thinking { thinking: String },
}

// ── Message type ────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: Vec<ContentBlock>,
    pub timestamp: Option<i64>,
}

// ── Session info ───────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct Session {
    pub id: String,
    pub display_name: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub last_active: Option<DateTime<Utc>>,
    pub message_count: usize,
    pub project_path: Option<String>,
}

// ── Smart summary ───────────────────────────────────────────────

/// Generate a smart summary from text: split by punctuation, take first sentence, max 50 chars.
pub fn smart_summary(text: &str) -> String {
    let text = text.trim();
    if text.is_empty() {
        return String::new();
    }
    let first_sentence = text
        .split(&['。', '？', '！', '，', '\n', '.', '?', '!', ','][..])
        .next()
        .unwrap_or(text)
        .trim();
    if first_sentence.len() <= 50 {
        first_sentence.to_string()
    } else {
        let end = first_sentence
            .char_indices()
            .take_while(|(i, _)| *i < 50)
            .last()
            .map(|(i, c)| i + c.len_utf8())
            .unwrap_or(50);
        format!("{}…", &first_sentence[..end])
    }
}

// ── AI title parsing ────────────────────────────────────────────

/// Parse an ai-title line from JSONL, returns the aiTitle string if found.
pub fn parse_ai_title(line: &str) -> Option<String> {
    let v: serde_json::Value = serde_json::from_str(line).ok()?;
    if v.get("type")?.as_str()? == "ai-title" {
        v.get("aiTitle")?.as_str().map(|s| s.to_string())
    } else {
        None
    }
}

// ── Content block parsing ────────────────────────────────────────

const CONVERSATION_TYPES: &[&str] = &["user", "assistant"];

fn parse_content_blocks(value: &serde_json::Value) -> Vec<ContentBlock> {
    match value {
        serde_json::Value::String(s) => {
            if s.trim().is_empty() {
                vec![]
            } else {
                vec![ContentBlock::Text { text: s.clone() }]
            }
        }
        serde_json::Value::Array(arr) => arr
            .iter()
            .filter_map(|item| serde_json::from_value(item.clone()).ok())
            .collect(),
        _ => vec![],
    }
}

// ── Message parsing ─────────────────────────────────────────────

/// Parse a single JSONL line into a Message, if it is a conversation message.
pub fn parse_message(line: &str) -> Option<Message> {
    if line.trim().is_empty() {
        return None;
    }
    let v: serde_json::Value = serde_json::from_str(line).ok()?;

    let role = v.get("type")?.as_str()?.to_string();

    if !CONVERSATION_TYPES.contains(&role.as_str()) {
        return None;
    }

    let content_value = v
        .get("message")
        .and_then(|m| m.get("content"))
        .cloned()
        .unwrap_or(serde_json::Value::Null);

    let content = parse_content_blocks(&content_value);

    if content.is_empty() {
        return None;
    }

    let timestamp = v.get("timestamp").and_then(|t| t.as_i64());

    Some(Message {
        role,
        content,
        timestamp,
    })
}

// ── Merge tool results ───────────────────────────────────────────

/// Merge standalone tool_result user messages into the preceding assistant message.
/// In the Anthropic format, tool results come as separate "user" messages with
/// only tool_result content blocks. These should be appended to the previous
/// assistant message for display purposes.
pub fn merge_tool_results(messages: &mut Vec<Message>) {
    let mut i = 1;
    while i < messages.len() {
        let is_only_tool_results = messages[i].role == "user"
            && !messages[i].content.is_empty()
            && messages[i]
                .content
                .iter()
                .all(|b| matches!(b, ContentBlock::ToolResult { .. }));

        if is_only_tool_results && messages[i - 1].role == "assistant" {
            let blocks: Vec<ContentBlock> = messages[i].content.drain(..).collect();
            messages[i - 1].content.extend(blocks);
            messages.remove(i);
            continue;
        }
        i += 1;
    }
}

// ── Load session ─────────────────────────────────────────────────

/// Load a session from a JSONL file path.
/// Returns None if the file is empty or contains no conversation messages.
pub fn load_session(path: &Path) -> Option<Session> {
    let id = path.file_stem()?.to_string_lossy().to_string();
    let content = std::fs::read_to_string(path).ok()?;
    let lines: Vec<&str> = content.lines().collect();

    let mut messages = Vec::new();
    let mut last_ai_title: Option<String> = None;
    let mut first_user_text: Option<String> = None;

    for line in &lines {
        if let Some(title) = parse_ai_title(line) {
            last_ai_title = Some(title);
        }
        if let Some(msg) = parse_message(line) {
            if msg.role == "user" && first_user_text.is_none() {
                for block in &msg.content {
                    if let ContentBlock::Text { text } = block {
                        if !text.trim().is_empty() {
                            first_user_text = Some(text.clone());
                            break;
                        }
                    }
                }
            }
            messages.push(msg);
        }
    }

    merge_tool_results(&mut messages);

    if messages.is_empty() {
        return None;
    }

    let message_count = messages.len();

    let started_at = messages
        .first()
        .and_then(|m| m.timestamp)
        .and_then(|ts| DateTime::from_timestamp_millis(ts));

    let display_name = last_ai_title.or_else(|| first_user_text.map(|t| smart_summary(&t)));

    let last_active = path
        .metadata()
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|t| {
            let dt: DateTime<Utc> = t.into();
            Some(dt)
        });

    Some(Session {
        id,
        display_name,
        started_at,
        last_active,
        message_count,
        project_path: None,
    })
}

// ── List sessions in a directory ─────────────────────────────────

/// List all JSONL sessions in a project directory, sorted by last active time.
pub fn list_sessions(project_dir: &Path) -> Vec<Session> {
    let mut sessions_with_time: Vec<(Session, std::time::SystemTime)> = Vec::new();

    if let Ok(entries) = std::fs::read_dir(project_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|ext| ext == "jsonl").unwrap_or(false) {
                let mtime = path
                    .metadata()
                    .ok()
                    .and_then(|m| m.modified().ok())
                    .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                if let Some(mut session) = load_session(&path) {
                    session.last_active = mtime
                        .duration_since(std::time::SystemTime::UNIX_EPOCH)
                        .ok()
                        .and_then(|d| DateTime::from_timestamp_millis(d.as_millis() as i64));
                    sessions_with_time.push((session, mtime));
                }
            }
        }
    }

    sessions_with_time.sort_by(|a, b| b.1.cmp(&a.1));
    sessions_with_time.into_iter().map(|(s, _)| s).collect()
}

// ── Load session messages ────────────────────────────────────────

/// Load all messages from a JSONL session file.
pub fn load_session_messages(path: &Path) -> Result<Vec<Message>, String> {
    let content =
        std::fs::read_to_string(path).map_err(|e| format!("cannot read session: {e}"))?;

    let mut messages = Vec::new();
    for line in content.lines() {
        if let Some(msg) = parse_message(line) {
            messages.push(msg);
        }
    }

    merge_tool_results(&mut messages);
    Ok(messages)
}

// ── Simple session info loading (for history view) ─────────────────

/// Extract display_name and message_count from raw JSONL content.
/// Used by adapters that already have the file content in hand.
pub fn load_session_info(content: &str) -> (Option<String>, usize) {
    let mut last_ai_title: Option<String> = None;
    let mut first_user_text: Option<String> = None;
    let mut message_count: usize = 0;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some(title) = parse_ai_title(trimmed) {
            last_ai_title = Some(title);
        }
        if let Some(msg) = parse_message(trimmed) {
            if msg.role == "user" && first_user_text.is_none() {
                for block in &msg.content {
                    if let ContentBlock::Text { text } = block {
                        if !text.trim().is_empty() {
                            first_user_text = Some(text.clone());
                            break;
                        }
                    }
                }
            }
            message_count += 1;
        }
    }

    let display_name = last_ai_title.or_else(|| first_user_text.map(|t| smart_summary(&t)));
    (display_name, message_count)
}

/// Load session messages as simple (role, content_string) pairs for display.
pub fn load_session_messages_simple(path: &Path) -> Result<Vec<SimpleMessage>, String> {
    let messages = load_session_messages(path)?;
    Ok(messages
        .into_iter()
        .map(|msg| {
            let content = msg
                .content
                .iter()
                .map(|block| match block {
                    ContentBlock::Text { text } => text.clone(),
                    ContentBlock::ToolUse { name, input, .. } => {
                        format!("[Tool: {name}] {}", input.to_string())
                    }
                    ContentBlock::ToolResult { content, is_error, .. } => {
                        if *is_error {
                            format!("[Error] {}", content.to_string())
                        } else {
                            content.to_string()
                        }
                    }
                    ContentBlock::Thinking { thinking } => {
                        format!("[Thinking] {thinking}")
                    }
                })
                .collect::<Vec<_>>()
                .join("\n");
            SimpleMessage {
                role: msg.role,
                content,
                timestamp: msg.timestamp,
            }
        })
        .collect())
}

#[derive(Debug, Clone, Serialize)]
pub struct SimpleMessage {
    pub role: String,
    pub content: String,
    pub timestamp: Option<i64>,
}

// ── Helpers ────────────────────────────────────────────────────

/// Count JSONL files in a directory.
pub fn count_jsonl_files(dir: &Path) -> usize {
    std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .flatten()
                .filter(|e| {
                    e.path()
                        .extension()
                        .map(|ext| ext == "jsonl")
                        .unwrap_or(false)
                })
                .count()
        })
        .unwrap_or(0)
}

/// Get the last modified time of the most recently changed JSONL file.
pub fn last_modified(dir: &Path) -> Option<String> {
    std::fs::read_dir(dir)
        .ok()?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "jsonl")
                .unwrap_or(false)
        })
        .filter_map(|e| e.metadata().ok()?.modified().ok())
        .max()
        .map(|t| {
            let dt: DateTime<Utc> = t.into();
            dt.format("%Y-%m-%d %H:%M:%S").to_string()
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smart_summary_short() {
        assert_eq!(smart_summary("Hello world"), "Hello world");
    }

    #[test]
    fn test_smart_summary_long() {
        let long = "This is a very long sentence that exceeds fifty characters by quite a bit more text here";
        let result = smart_summary(long);
        assert!(result.ends_with('…'));
        assert!(result.len() <= 55);
    }

    #[test]
    fn test_smart_summary_splits_on_punctuation() {
        assert_eq!(smart_summary("First sentence. Second one"), "First sentence");
        assert_eq!(smart_summary("第一句。第二句"), "第一句");
    }

    #[test]
    fn test_parse_ai_title() {
        let line = r#"{"type":"ai-title","aiTitle":"Fix login bug","sessionId":"abc"}"#;
        assert_eq!(parse_ai_title(line), Some("Fix login bug".to_string()));
    }

    #[test]
    fn test_parse_ai_title_ignores_other() {
        let line = r#"{"type":"user","message":{"content":"hello"}}"#;
        assert_eq!(parse_ai_title(line), None);
    }
}
