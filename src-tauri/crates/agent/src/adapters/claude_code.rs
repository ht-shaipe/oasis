//! Claude Code agent adapter.
//!
//! Manages the `claude` CLI: health probe, chat spawning, stream event
//! normalization, project/session discovery.

use crate::normalized::{NormalizedEvent, TurnEndReason, UsageStats};
use crate::plugin::{
    AgentHealth, AgentInfo, AgentPlugin, ChatArgs, ProjectEntry, SessionInfo, SessionMessage,
};
use chrono::{DateTime, Utc};
use std::path::Path;
use std::process::Command;

// ── Agent struct ──────────────────────────────────────────────────

pub struct ClaudeCodeAgent;

impl ClaudeCodeAgent {
    pub fn new() -> Self {
        Self
    }
}

// ── Stream event normalization ────────────────────────────────────

pub fn normalize_stream_event(event: &serde_json::Value) -> Vec<NormalizedEvent> {
    match event.get("type").and_then(|v| v.as_str()) {
        Some("stream_event") => normalize_stream_event_inner(event),
        Some("assistant") => normalize_assistant(event),
        Some("user") => normalize_user(event),
        Some("result") => normalize_result(event),
        Some("system") => normalize_system(event),
        _ => vec![NormalizedEvent::Raw {
            agent: "claude-code".into(),
            raw: event.clone(),
        }],
    }
}

fn normalize_stream_event_inner(event: &serde_json::Value) -> Vec<NormalizedEvent> {
    let inner = event.get("event").unwrap_or(event);
    let delta = inner.get("delta");

    if let Some(text) = delta.and_then(|d| d.get("text")).and_then(|v| v.as_str()) {
        return vec![NormalizedEvent::TextDelta {
            delta: text.to_string(),
        }];
    }

    if let Some(thinking) = delta
        .and_then(|d| d.get("thinking"))
        .and_then(|v| v.as_str())
    {
        return vec![NormalizedEvent::Thinking {
            delta: thinking.to_string(),
        }];
    }

    // tool_use in stream_event deltas
    if let Some(tool_use) = delta.and_then(|d| d.get("tool_use")) {
        if tool_use.get("type").and_then(|v| v.as_str()) == Some("tool_use") {
            let call_id = tool_use
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let name = tool_use
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("tool")
                .to_string();
            let input = tool_use.get("input").cloned().unwrap_or_default();
            let tool_kind = Some(crate::classify::classify_tool_call(&name, &input));
            return vec![NormalizedEvent::ToolUseStart {
                call_id,
                tool: name,
                input,
                tool_kind,
            }];
        }
    }

    vec![NormalizedEvent::Raw {
        agent: "claude-code".into(),
        raw: event.clone(),
    }]
}

fn normalize_assistant(event: &serde_json::Value) -> Vec<NormalizedEvent> {
    let content = event
        .get("message")
        .and_then(|m| m.get("content"))
        .or_else(|| event.get("content"))
        .and_then(|v| v.as_array());

    let Some(content) = content else {
        return vec![NormalizedEvent::Raw {
            agent: "claude-code".into(),
            raw: event.clone(),
        }];
    };

    let mut normalized = Vec::new();
    for block in content {
        if block.get("type").and_then(|v| v.as_str()) == Some("tool_use") {
            let call_id = block
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let name = block
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("tool")
                .to_string();
            let input = block.get("input").cloned().unwrap_or_default();
            let tool_kind = Some(crate::classify::classify_tool_call(&name, &input));
            normalized.push(NormalizedEvent::ToolUseStart {
                call_id,
                tool: name,
                input,
                tool_kind,
            });
        }
    }
    normalized
}

fn normalize_user(event: &serde_json::Value) -> Vec<NormalizedEvent> {
    let content = event
        .get("message")
        .and_then(|m| m.get("content"))
        .or_else(|| event.get("content"));

    let Some(content) = content else {
        return vec![];
    };

    // Handle tool_result blocks
    if let Some(arr) = content.as_array() {
        let mut results = Vec::new();
        for block in arr {
            if block.get("type").and_then(|v| v.as_str()) == Some("tool_result") {
                let tool_use_id = block
                    .get("tool_use_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string();
                let is_error = block
                    .get("is_error")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let output = block.get("content").cloned().unwrap_or_default();
                results.push(NormalizedEvent::ToolUseResult {
                    call_id: tool_use_id,
                    output,
                    is_error,
                    tool_kind: None,
                });
            }
        }
        if !results.is_empty() {
            return results;
        }
    }

    // Handle string content (fallback)
    if let Some(text) = content.as_str() {
        if !text.trim().is_empty() {
            return vec![NormalizedEvent::TextDelta {
                delta: text.to_string(),
            }];
        }
    }

    vec![]
}

fn normalize_result(event: &serde_json::Value) -> Vec<NormalizedEvent> {
    let mut events = Vec::new();

    // Extract usage stats
    let usage = event.get("usage").map(|u| UsageStats {
        input_tokens: u.get("input_tokens").and_then(|v| v.as_u64()),
        output_tokens: u.get("output_tokens").and_then(|v| v.as_u64()),
        total_cost: u
            .get("total_cost_usd")
            .and_then(|v| v.as_f64())
            .or_else(|| {
                u.get("cost")
                    .and_then(|c| c.get("total"))
                    .and_then(|v| v.as_f64())
            }),
        context_remaining: u.get("context_remaining").and_then(|v| v.as_u64()),
    });

    // Session id
    if let Some(session_id) = event
        .get("session_id")
        .or_else(|| event.get("sessionId"))
        .and_then(|v| v.as_str())
    {
        events.push(NormalizedEvent::SessionResolved {
            session_id: session_id.to_string(),
        });
    }

    let reason = match event.get("subtype").and_then(|v| v.as_str()) {
        Some("max_tokens") | Some("max_turns") => TurnEndReason::MaxTokens,
        Some("aborted") | Some("interrupted") => TurnEndReason::Aborted,
        Some("error") | Some("error_during_execution") => TurnEndReason::Error,
        _ => TurnEndReason::Complete,
    };

    events.push(NormalizedEvent::TurnComplete { reason, usage });
    events
}

fn normalize_system(event: &serde_json::Value) -> Vec<NormalizedEvent> {
    // system events may carry init info; ignore noise, catch session ids
    if let Some(session_id) = event
        .get("session_id")
        .or_else(|| event.get("sessionId"))
        .and_then(|v| v.as_str())
    {
        vec![NormalizedEvent::SessionResolved {
            session_id: session_id.to_string(),
        }]
    } else {
        vec![]
    }
}

// ── AgentPlugin implementation ────────────────────────────────────

#[async_trait::async_trait]
impl AgentPlugin for ClaudeCodeAgent {
    fn info(&self) -> AgentInfo {
        AgentInfo {
            id: "claude-code".into(),
            display_name: "Claude Code".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            icon: "claude".into(),
            enabled: true,
        }
    }

    fn probe_sync(&self) -> AgentHealth {
        probe_claude_code()
    }

    fn install_hint(&self) -> Option<String> {
        Some("npm install -g @anthropic-ai/claude-code".into())
    }

    fn native_install_command(&self) -> Option<String> {
        Some("npm install -g @anthropic-ai/claude-code".into())
    }

    fn build_chat_command(&self, args: &ChatArgs) -> tokio::process::Command {
        let mut cmd = tokio::process::Command::new("claude");
        cmd.arg("-p")
            .arg(&args.message)
            .arg("--output-format")
            .arg("stream-json")
            .arg("--verbose")
            .current_dir(&args.project_path);

        if let Some(ref sid) = args.session_id {
            cmd.arg("--resume").arg(sid);
        }

        cmd
    }

    fn abort_sequence(&self) -> Option<&'static [u8]> {
        Some(b"\x03")
    }

    fn build_resume_command(&self, session_id: &str) -> String {
        format!("claude --resume {session_id}")
    }

    fn normalize_stream_event(&self, event: &serde_json::Value) -> Vec<NormalizedEvent> {
        normalize_stream_event(event)
    }

    fn list_sessions(&self, project_path: &str) -> Result<Vec<SessionInfo>, String> {
        list_claude_sessions(project_path)
    }

    fn load_session_messages(
        &self,
        session_id: &str,
        project_path: &str,
    ) -> Result<Vec<SessionMessage>, String> {
        load_claude_session_messages(session_id, project_path)
    }

    fn scan_projects(&self) -> Vec<ProjectEntry> {
        scan_claude_projects()
    }

    fn add_project(&self, path: &str) -> Option<ProjectEntry> {
        add_claude_project(path)
    }

    fn open_in_terminal(
        &self,
        project_path: &str,
        resume_session_id: Option<&str>,
    ) -> Result<u32, String> {
        open_claude_in_terminal(project_path, resume_session_id)
    }

    fn init_project(&self, project_path: &str) -> Result<bool, String> {
        init_claude_project(project_path)
    }
}

// ── Health probe ──────────────────────────────────────────────────

fn probe_claude_code() -> AgentHealth {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64;

    let which_output = Command::new("which").arg("claude").output();

    match which_output {
        Ok(output) if output.status.success() => {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            // Try to get version
            let version = Command::new(&path)
                .arg("--version")
                .output()
                .ok()
                .and_then(|o| {
                    if o.status.success() {
                        Some(
                            String::from_utf8_lossy(&o.stdout)
                                .trim()
                                .split_whitespace()
                                .last()
                                .unwrap_or("unknown")
                                .to_string(),
                        )
                    } else {
                        None
                    }
                });
            AgentHealth {
                installed: true,
                version,
                error: None,
                binary_path: Some(path),
                last_checked_at: now,
            }
        }
        _ => AgentHealth {
            installed: false,
            version: None,
            error: Some("claude not found in PATH".into()),
            binary_path: None,
            last_checked_at: now,
        },
    }
}

// ── Project scanning (macOS paths) ────────────────────────────────

fn claude_projects_dir() -> std::path::PathBuf {
    let home = dirs_next::home_dir().unwrap_or_default();
    home.join(".claude").join("projects")
}

fn scan_claude_projects() -> Vec<ProjectEntry> {
    let dir = claude_projects_dir();
    if !dir.is_dir() {
        return vec![];
    }

    let mut projects = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let file_name = entry.file_name().to_string_lossy().to_string();
            let project_dir = dir.join(&file_name);
            if !project_dir.is_dir() {
                continue;
            }

            let decoded = decode_claude_project_path(&file_name);
            let name = Path::new(&decoded)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or(file_name.clone());

            let session_count = count_jsonl_files(&project_dir);
            let last_active = last_modified(&project_dir);

            projects.push(ProjectEntry {
                name,
                path: decoded,
                encoded_name: file_name,
                session_count,
                last_active,
                agent_id: "claude-code".into(),
                initialized: true,
            });
        }
    }

    projects.sort_by(|a, b| b.last_active.cmp(&a.last_active));
    projects
}

fn add_claude_project(path: &str) -> Option<ProjectEntry> {
    let p = Path::new(path);
    if !p.is_dir() {
        return None;
    }

    let name = p.file_name()?.to_string_lossy().to_string();
    let encoded = encode_claude_project_path(path);
    let project_dir = claude_projects_dir().join(&encoded);

    let session_count = if project_dir.is_dir() {
        count_jsonl_files(&project_dir)
    } else {
        0
    };

    let last_active = if project_dir.is_dir() {
        last_modified(&project_dir)
    } else {
        None
    };

    Some(ProjectEntry {
        name,
        path: path.to_string(),
        encoded_name: encoded,
        session_count,
        last_active,
        agent_id: "claude-code".into(),
        initialized: project_dir.is_dir(),
    })
}

/// Claude Code encodes macOS paths: / → - , spaces → -
fn encode_claude_project_path(path: &str) -> String {
    path.replace('/', "-").replace(' ', "-")
}

/// Decode a Claude Code encoded project path by matching filesystem entries.
fn decode_claude_project_path(encoded: &str) -> String {
    // Common prefixes to try
    let home = dirs_next::home_dir().unwrap_or_default();
    let home_str = home.to_string_lossy().to_string();
    let _ = home_str; // used by decode_by_fs_matching below

    // Try matching from home directory
    if let Some(result) = decode_by_fs_matching(Path::new("/"), encoded) {
        return result;
    }

    // Fallback: simple replace
    format!("/{}", encoded.replace('-', "/"))
}

fn decode_by_fs_matching(root: &Path, remaining: &str) -> Option<String> {
    if remaining.is_empty() {
        return Some(root.to_string_lossy().to_string());
    }

    if !root.is_dir() {
        return None;
    }

    let mut matches: Vec<(String, usize)> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(root) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            let encoded_seg = name.replace(' ', "-");

            if remaining == encoded_seg || remaining.starts_with(&format!("{encoded_seg}-")) {
                matches.push((
                    entry.path().to_string_lossy().to_string(),
                    encoded_seg.len(),
                ));
            }
        }
    }

    matches.sort_by(|a, b| b.1.cmp(&a.1));

    for (path, consumed) in matches {
        let rest = &remaining[consumed..];
        let next = rest.strip_prefix('-').unwrap_or(rest);
        if next.is_empty() && Path::new(&path).is_dir() {
            return Some(path);
        }
        if let Some(result) = decode_by_fs_matching(Path::new(&path), next) {
            return Some(result);
        }
    }

    None
}

// ── Session management ────────────────────────────────────────────

fn list_claude_sessions(project_path: &str) -> Result<Vec<SessionInfo>, String> {
    let encoded = encode_claude_project_path(project_path);
    let dir = claude_projects_dir().join(&encoded);

    if !dir.is_dir() {
        return Ok(vec![]);
    }

    let mut sessions = Vec::new();
    let entries =
        std::fs::read_dir(&dir).map_err(|e| format!("cannot read project dir: {e}"))?;

    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.extension().map(|e| e == "jsonl").unwrap_or(false) {
            let id = path
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();
            let mtime = path
                .metadata()
                .ok()
                .and_then(|m| m.modified().ok());

            let last_active = mtime.and_then(|t| {
                let dt: DateTime<Utc> = t.into();
                Some(dt.format("%Y-%m-%d %H:%M:%S").to_string())
            });

            let content = std::fs::read_to_string(&path).unwrap_or_default();
            let (display_name, msg_count) =
                oasis_project::session::load_session_info(&content);

            sessions.push(SessionInfo {
                id,
                display_name,
                started_at: last_active.clone(),
                last_active,
                message_count: msg_count,
            });
        }
    }

    sessions.sort_by(|a, b| b.last_active.cmp(&a.last_active));
    Ok(sessions)
}

fn load_claude_session_messages(
    session_id: &str,
    project_path: &str,
) -> Result<Vec<SessionMessage>, String> {
    let encoded = encode_claude_project_path(project_path);
    let file_path = claude_projects_dir()
        .join(&encoded)
        .join(format!("{session_id}.jsonl"));

    let content =
        std::fs::read_to_string(&file_path).map_err(|e| format!("cannot read session: {e}"))?;

    let mut messages = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let parsed: serde_json::Value =
            serde_json::from_str(trimmed).map_err(|e| format!("invalid JSONL: {e}"))?;

        let role = parsed
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        if role != "user" && role != "assistant" {
            continue;
        }

        let content = parsed
            .get("message")
            .and_then(|m| m.get("content"))
            .map(|c| match c {
                serde_json::Value::String(s) => s.clone(),
                other => other.to_string(),
            })
            .unwrap_or_default();

        let timestamp = parsed.get("timestamp").and_then(|t| t.as_i64());

        messages.push(SessionMessage {
            role,
            content,
            timestamp,
        });
    }

    Ok(messages)
}

// ── Terminal ──────────────────────────────────────────────────────

fn open_claude_in_terminal(
    project_path: &str,
    resume_session_id: Option<&str>,
) -> Result<u32, String> {
    let cmd_str = if let Some(sid) = resume_session_id {
        format!("claude --resume {sid}")
    } else {
        "claude".to_string()
    };

    let script = format!(
        r#"tell application "Terminal"
    activate
    do script "cd '{}' && clear && {}"
end tell"#,
        project_path.replace('\'', "'\\''"),
        cmd_str
    );

    let output = std::process::Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()
        .map_err(|e| format!("failed to launch Terminal: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("osascript failed: {stderr}"));
    }

    // Return a placeholder PID — real terminal process tracking not practical via osascript
    Ok(0)
}

// ── Project init ──────────────────────────────────────────────────

fn init_claude_project(project_path: &str) -> Result<bool, String> {
    let claude_dir = Path::new(project_path).join(".claude");
    if claude_dir.exists() {
        return Ok(false); // already initialized
    }

    std::fs::create_dir_all(&claude_dir)
        .map_err(|e| format!("failed to create .claude dir: {e}"))?;

    Ok(true)
}

// ── Helpers ───────────────────────────────────────────────────────

fn count_jsonl_files(dir: &Path) -> usize {
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

fn last_modified(dir: &Path) -> Option<String> {
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
