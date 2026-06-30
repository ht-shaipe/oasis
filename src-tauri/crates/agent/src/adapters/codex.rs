//! Codex agent adapter.
//!
//! Manages the `codex` CLI: health probe, chat spawning, stream event
//! normalization, project/session discovery.
//!
//! Ported from Jishu Hub `adapters/codex.rs` with macOS adaptations:
//! - Removed Windows `cmd /C` and `tokio_no_window` logic
//! - Uses `which` instead of `where` for binary discovery
//! - Uses `osascript` for Terminal.app integration

use crate::normalized::{NormalizedEvent, TurnEndReason};
use crate::plugin::{
    AgentHealth, AgentInfo, AgentPlugin, ChatArgs, ProjectEntry, SessionInfo, SessionMessage,
};
use chrono::Utc;
use std::collections::HashMap;
use std::io::BufRead;
use std::path::Path;

// ── Agent struct ─────────────────────────────────────────────────

pub struct CodexAdapter;

impl CodexAdapter {
    pub fn new() -> Self {
        Self
    }
}

// ── Stream event normalization ───────────────────────────────────

pub fn normalize_stream_event(event: &serde_json::Value) -> Vec<NormalizedEvent> {
    match event.get("type").and_then(|v| v.as_str()) {
        Some("message_delta") | Some("exec_command_output_delta") => {
            let delta = event
                .get("delta")
                .or_else(|| event.get("text"))
                .or_else(|| event.get("output"))
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            if delta.is_empty() {
                raw(event)
            } else {
                vec![NormalizedEvent::TextDelta {
                    delta: delta.to_string(),
                }]
            }
        }
        Some("message") => normalize_codex_message(event),
        Some("result") | Some("turn_complete") => normalize_codex_complete(event),
        _ => raw(event),
    }
}

fn normalize_codex_message(event: &serde_json::Value) -> Vec<NormalizedEvent> {
    if let Some(text) = event
        .get("message")
        .or_else(|| event.get("content"))
        .and_then(|v| v.as_str())
    {
        return vec![NormalizedEvent::TextDelta {
            delta: text.to_string(),
        }];
    }

    raw(event)
}

fn normalize_codex_complete(event: &serde_json::Value) -> Vec<NormalizedEvent> {
    let mut normalized = Vec::new();
    if let Some(session_id) = event
        .get("session_id")
        .or_else(|| event.get("sessionId"))
        .and_then(|v| v.as_str())
    {
        normalized.push(NormalizedEvent::SessionResolved {
            session_id: session_id.to_string(),
        });
    }

    if let Some(error) = event.get("error").and_then(|v| v.as_str()) {
        normalized.push(NormalizedEvent::Error {
            message: error.to_string(),
            recoverable: false,
        });
        normalized.push(NormalizedEvent::TurnComplete {
            reason: TurnEndReason::Error,
            usage: None,
        });
    } else {
        normalized.push(NormalizedEvent::TurnComplete {
            reason: TurnEndReason::Complete,
            usage: None,
        });
    }
    normalized
}

fn raw(event: &serde_json::Value) -> Vec<NormalizedEvent> {
    vec![NormalizedEvent::Raw {
        agent: "codex".to_string(),
        raw: event.clone(),
    }]
}

// ── AgentPlugin implementation ───────────────────────────────────

#[async_trait::async_trait]
impl AgentPlugin for CodexAdapter {
    fn info(&self) -> AgentInfo {
        AgentInfo {
            id: "codex".into(),
            display_name: "Codex".into(),
            version: "1.0".into(),
            icon: "bot".into(),
            enabled: true,
        }
    }

    fn probe_sync(&self) -> AgentHealth {
        probe_codex()
    }

    fn install_hint(&self) -> Option<String> {
        Some("npm install -g @openai/codex".to_string())
    }

    fn native_install_command(&self) -> Option<String> {
        Some("npm install -g @openai/codex".to_string())
    }

    fn build_chat_command(&self, args: &ChatArgs) -> tokio::process::Command {
        let mut cmd_args: Vec<String> = vec!["exec".into(), "--json".into(), args.message.clone()];

        if let Some(ref sid) = args.session_id {
            cmd_args.push("--resume".into());
            cmd_args.push(sid.clone());
        }

        let mut cmd = tokio::process::Command::new("codex");
        cmd.args(&cmd_args).current_dir(&args.project_path);
        cmd
    }

    fn build_resume_command(&self, session_id: &str) -> String {
        format!("codex --resume {session_id}")
    }

    fn normalize_stream_event(&self, event: &serde_json::Value) -> Vec<NormalizedEvent> {
        normalize_stream_event(event)
    }

    fn list_sessions(&self, project_path: &str) -> Result<Vec<SessionInfo>, String> {
        list_codex_sessions(project_path)
    }

    fn load_session_messages(
        &self,
        session_id: &str,
        project_path: &str,
    ) -> Result<Vec<SessionMessage>, String> {
        load_codex_session_messages(session_id, project_path)
    }

    fn scan_projects(&self) -> Vec<ProjectEntry> {
        scan_codex_projects()
    }

    fn add_project(&self, path: &str) -> Option<ProjectEntry> {
        add_codex_project(path)
    }

    fn open_in_terminal(
        &self,
        project_path: &str,
        resume_session_id: Option<&str>,
    ) -> Result<u32, String> {
        open_codex_in_terminal(project_path, resume_session_id)
    }

    fn init_project(&self, project_path: &str) -> Result<bool, String> {
        let codex_dir = Path::new(project_path).join(".codex");
        if codex_dir.exists() {
            return Ok(false);
        }
        std::fs::create_dir_all(&codex_dir)
            .map_err(|e| format!("failed to create .codex dir: {e}"))?;
        Ok(true)
    }
}

// ── Health probe ─────────────────────────────────────────────────

fn probe_codex() -> AgentHealth {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64;

    let which_output = std::process::Command::new("which").arg("codex").output();

    match which_output {
        Ok(output) if output.status.success() => {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let version = std::process::Command::new(&path)
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
        _ => {
            let candidates = codex_candidates();
            for c in candidates {
                if Path::new(&c).exists() {
                    let version = std::process::Command::new(&c)
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
                    return AgentHealth {
                        installed: true,
                        version,
                        error: None,
                        binary_path: Some(c),
                        last_checked_at: now,
                    };
                }
            }
            AgentHealth {
                installed: false,
                version: None,
                error: Some("codex not found in PATH".to_string()),
                binary_path: None,
                last_checked_at: now,
            }
        }
    }
}

fn codex_candidates() -> Vec<String> {
    let home = dirs_next::home_dir().unwrap_or_default();
    let home_str = home.to_string_lossy().to_string();
    vec![
        format!("{}/.bun/bin/codex", home_str),
        "/usr/local/bin/codex".to_string(),
    ]
}

// ── Project scanning (macOS) ─────────────────────────────────────

fn scan_codex_projects() -> Vec<ProjectEntry> {
    let home = match dirs_next::home_dir() {
        Some(h) => h,
        None => return vec![],
    };
    let state_path = home.join(".codex").join(".codex-global-state.json");
    if !state_path.exists() {
        return vec![];
    }

    let content = std::fs::read_to_string(&state_path).unwrap_or_default();
    let state: serde_json::Value =
        serde_json::from_str(&content).unwrap_or(serde_json::json!({}));

    let roots = state
        .get("electron-saved-workspace-roots")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let session_counts = codex_session_counts_by_cwd();

    let mut projects = Vec::new();
    for path_str in roots {
        let path = Path::new(&path_str);
        if !path.exists() {
            continue;
        }

        let name = path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| path_str.clone());

        let session_count = session_counts.get(&path_str).copied().unwrap_or(0);

        projects.push(ProjectEntry {
            name,
            path: path_str,
            encoded_name: String::new(),
            session_count,
            last_active: None,
            agent_id: "codex".into(),
            initialized: true,
        });
    }
    projects
}

fn add_codex_project(path: &str) -> Option<ProjectEntry> {
    let p = Path::new(path);
    if !p.is_dir() {
        return None;
    }

    let name = p.file_name()?.to_string_lossy().to_string();

    Some(ProjectEntry {
        name,
        path: path.to_string(),
        encoded_name: String::new(),
        session_count: 0,
        last_active: None,
        agent_id: "codex".into(),
        initialized: true,
    })
}

// ── Session management ──────────────────────────────────────────

fn list_codex_sessions(project_path: &str) -> Result<Vec<SessionInfo>, String> {
    let home = dirs_next::home_dir().ok_or("Cannot find home directory")?;
    let index_path = home.join(".codex").join("session_index.jsonl");
    if !index_path.exists() {
        return Ok(vec![]);
    }

    let content = std::fs::read_to_string(&index_path).map_err(|e| e.to_string())?;
    let mut sessions = Vec::new();

    for line in content.lines().rev() {
        if let Ok(item) = serde_json::from_str::<serde_json::Value>(line) {
            let id = item
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let thread_name = item
                .get("thread_name")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let updated_at_str = item
                .get("updated_at")
                .and_then(|v| v.as_str())
                .unwrap_or_default();

            if id.is_empty() {
                continue;
            }

            if let Some(rollout_path) = find_rollout_file(&id, updated_at_str) {
                if let Ok(cwd) = get_rollout_cwd(&rollout_path) {
                    if cwd == project_path {
                        let last_active = parse_rfc3339(updated_at_str);

                        let message_count = count_rollout_messages(&rollout_path);

                        sessions.push(SessionInfo {
                            id,
                            display_name: if thread_name.is_empty() {
                                None
                            } else {
                                Some(thread_name)
                            },
                            started_at: last_active.clone(),
                            last_active,
                            message_count,
                        });
                    }
                }
            }
        }
    }

    sessions.sort_by(|a, b| b.last_active.cmp(&a.last_active));
    Ok(sessions)
}

fn load_codex_session_messages(
    session_id: &str,
    _project_path: &str,
) -> Result<Vec<SessionMessage>, String> {
    let rollout_path = search_rollout_file(session_id)?;
    parse_rollout_messages(&rollout_path)
}

// ── Codex session index helpers ──────────────────────────────────

fn codex_session_counts_by_cwd() -> HashMap<String, usize> {
    let mut counts = HashMap::new();
    let home = match dirs_next::home_dir() {
        Some(h) => h,
        None => return counts,
    };
    let index_path = home.join(".codex").join("session_index.jsonl");
    if !index_path.exists() {
        return counts;
    }

    let content = match std::fs::read_to_string(&index_path) {
        Ok(c) => c,
        Err(_) => return counts,
    };

    for line in content.lines().rev() {
        if let Ok(item) = serde_json::from_str::<serde_json::Value>(line) {
            let id = item
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let updated_at_str = item
                .get("updated_at")
                .and_then(|v| v.as_str())
                .unwrap_or_default();

            if id.is_empty() {
                continue;
            }

            if let Some(rollout_path) = find_rollout_file(&id, updated_at_str) {
                if let Ok(cwd) = get_rollout_cwd(&rollout_path) {
                    *counts.entry(cwd).or_insert(0) += 1;
                }
            }
        }
    }

    counts
}

fn find_rollout_file(id: &str, updated_at: &str) -> Option<std::path::PathBuf> {
    let home = dirs_next::home_dir()?;
    let sessions_dir = home.join(".codex").join("sessions");

    let parts: Vec<&str> = updated_at.split('T').collect();
    if parts.is_empty() {
        return None;
    }
    let date_parts: Vec<&str> = parts[0].split('-').collect();
    if date_parts.len() < 3 {
        return None;
    }

    let year = date_parts[0];
    let month = date_parts[1];
    let day = date_parts[2];

    let target_dir = sessions_dir.join(year).join(month).join(day);
    if target_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&target_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.contains(id) && name.ends_with(".jsonl") {
                    return Some(entry.path());
                }
            }
        }
    }

    recursive_search_id(&sessions_dir, id)
}

fn recursive_search_id(dir: &Path, id: &str) -> Option<std::path::PathBuf> {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(found) = recursive_search_id(&path, id) {
                    return Some(found);
                }
            } else if path.is_file() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.contains(id) && name.ends_with(".jsonl") {
                    return Some(path);
                }
            }
        }
    }
    None
}

fn get_rollout_cwd(path: &Path) -> Result<String, String> {
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let reader = std::io::BufReader::new(file);

    if let Some(Ok(line)) = reader.lines().next() {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&line) {
            if let Some(cwd) = val
                .get("payload")
                .and_then(|p| p.get("cwd"))
                .and_then(|v| v.as_str())
            {
                return Ok(cwd.to_string());
            }
        }
    }
    Err("CWD not found in rollout".to_string())
}

fn search_rollout_file(id: &str) -> Result<std::path::PathBuf, String> {
    let home = dirs_next::home_dir().ok_or("Home dir not found")?;
    let sessions_dir = home.join(".codex").join("sessions");

    recursive_search_id(&sessions_dir, id)
        .ok_or_else(|| format!("Rollout file for session {} not found", id))
}

fn count_rollout_messages(path: &Path) -> usize {
    let file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return 0,
    };
    let reader = std::io::BufReader::new(file);
    let mut count = 0;

    for line in reader.lines().flatten() {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&line) {
            if val.get("type").and_then(|v| v.as_str()) == Some("event_msg") {
                if let Some(payload) = val.get("payload") {
                    let p_type = payload.get("type").and_then(|v| v.as_str());
                    if p_type == Some("user_message") || p_type == Some("agent_message") {
                        if payload.get("message").and_then(|v| v.as_str()).is_some() {
                            count += 1;
                        }
                    }
                }
            }
        }
    }
    count
}

fn parse_rollout_messages(path: &Path) -> Result<Vec<SessionMessage>, String> {
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let reader = std::io::BufReader::new(file);

    let mut messages = Vec::new();
    for line in reader.lines().flatten() {
        let Ok(val) = serde_json::from_str::<serde_json::Value>(&line) else {
            continue;
        };
        if val.get("type").and_then(|v| v.as_str()) != Some("event_msg") {
            continue;
        }
        let Some(payload) = val.get("payload") else {
            continue;
        };
        let p_type = payload.get("type").and_then(|v| v.as_str());
        let role = match p_type {
            Some("user_message") => "user",
            Some("agent_message") => "assistant",
            _ => continue,
        };
        let Some(msg) = payload.get("message").and_then(|v| v.as_str()) else {
            continue;
        };
        let timestamp = val
            .get("timestamp")
            .and_then(|v| v.as_str())
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.timestamp_millis());

        messages.push(SessionMessage {
            role: role.to_string(),
            content: msg.to_string(),
            timestamp,
        });
    }
    Ok(messages)
}

fn parse_rfc3339(s: &str) -> Option<String> {
    chrono::DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|dt| {
            let utc = dt.with_timezone(&Utc);
            utc.format("%Y-%m-%d %H:%M:%S").to_string()
        })
}

// ── Terminal ────────────────────────────────────────────────────

fn open_codex_in_terminal(
    project_path: &str,
    resume_session_id: Option<&str>,
) -> Result<u32, String> {
    let cmd_str = if let Some(sid) = resume_session_id {
        format!("codex --resume {sid}")
    } else {
        "codex".to_string()
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

    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_codex_message_delta() {
        let event = serde_json::json!({
            "type": "message_delta",
            "delta": "hello"
        });

        let result = normalize_stream_event(&event);
        assert!(matches!(
            &result[0],
            NormalizedEvent::TextDelta { delta } if delta == "hello"
        ));
    }

    #[test]
    fn normalizes_codex_turn_complete_with_session() {
        let event = serde_json::json!({
            "type": "turn_complete",
            "session_id": "codex-session"
        });

        let result = normalize_stream_event(&event);
        assert_eq!(result.len(), 2);
        assert!(matches!(&result[0], NormalizedEvent::SessionResolved { session_id } if session_id == "codex-session"));
        assert!(matches!(&result[1], NormalizedEvent::TurnComplete { reason: TurnEndReason::Complete, .. }));
    }
}
