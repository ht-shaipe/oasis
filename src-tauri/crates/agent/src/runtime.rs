//! CLI subprocess runtime — spawn, stream, and abort CLI agents.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::{Emitter, Manager};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::ChildStdin;
use tokio::sync::Mutex as TokioMutex;

use crate::normalized::NormalizedEvent;
use crate::plugin::ChatArgs;

// ── Public types ──────────────────────────────────────────────────

#[derive(Debug, Clone, serde::Serialize)]
pub struct AgentStreamChunk {
    pub agent_id: String,
    pub session_id: String,
    pub event_type: String,
    pub data: serde_json::Value,
}

#[derive(Clone)]
pub struct AgentProcess {
    pub agent_id: String,
    pub process_id: u32,
    pub stdin: Option<Arc<TokioMutex<Option<ChildStdin>>>>,
}

// ── Public API ────────────────────────────────────────────────────

/// Spawn an agent CLI process. Returns (agent_id, session_id, process_id).
pub async fn spawn_agent_chat(
    app: &tauri::AppHandle,
    agent_id: &str,
    args: &ChatArgs,
    session_id: Option<String>,
) -> Result<(String, String, u32), String> {
    // Build command and probe agent config synchronously first
    let (mut command, wants_stdin, _abort_seq) = {
        let registry = app.state::<Mutex<crate::plugin::AgentRegistry>>();
        let reg = registry.lock().map_err(|e| e.to_string())?;
        let agent = reg
            .get(agent_id)
            .ok_or_else(|| format!("agent not found: {agent_id}"))?;
        let cmd = agent.build_chat_command(args);
        let wants = agent.wants_stdin_pipe();
        let abort = agent.abort_sequence().map(|s| s.to_vec());
        (cmd, wants, abort)
    };

    if wants_stdin {
        command.stdin(std::process::Stdio::piped());
    }
    command
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    let mut child = command
        .spawn()
        .map_err(|e| format!("failed to spawn agent: {e}"))?;

    let pid = child.id().unwrap_or(0);
    let sid = session_id.unwrap_or_else(|| format!("pending-{pid}"));
    let aid = agent_id.to_string();

    let stdin = child.stdin.take().map(|s| Arc::new(TokioMutex::new(Some(s))));
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "no stdout from agent")?;
    let stderr = child.stderr.take();

    // Store process
    {
        let processes = app.state::<Mutex<HashMap<String, AgentProcess>>>();
        let mut map = processes.lock().map_err(|e| e.to_string())?;
        map.insert(
            sid.clone(),
            AgentProcess {
                agent_id: aid.clone(),
                process_id: pid,
                stdin: stdin.clone(),
            },
        );
    }

    // Spawn readers
    let app_stdout = app.clone();
    let app_stderr = app.clone();
    let app_resolve = app.clone();
    let aid_reader1 = aid.clone();
    let aid_reader2 = aid.clone();
    let sid_reader1 = sid.clone();
    let sid_reader2 = sid.clone();
    let sid_resolve = sid.clone();

    tokio::spawn(async move {
        drain_stderr(app_stderr, &aid_reader1, &sid_reader1, stderr).await;
    });

    tokio::spawn(async move {
        stream_stdout(app_stdout, &aid_reader2, &sid_reader2, stdout).await;

        // Cleanup
        let processes = app_resolve.state::<Mutex<HashMap<String, AgentProcess>>>();
        if let Ok(mut map) = processes.lock() {
            map.remove(&sid_resolve);
        }
    });

    // Detached child waiter
    tokio::spawn(async move {
        let _ = child.wait().await;
    });

    Ok((aid, sid, pid))
}

/// Abort a running agent session.
pub async fn abort_agent_chat(app: &tauri::AppHandle, session_id: &str) -> Result<(), String> {
    let process = {
        let processes = app.state::<Mutex<HashMap<String, AgentProcess>>>();
        let map = processes.lock().map_err(|e| e.to_string())?;
        map.get(session_id).cloned()
    };

    let Some(process) = process else {
        return Ok(());
    };

    // Try abort sequence via stdin
    if let Some(stdin_mutex) = &process.stdin {
        let mut guard = stdin_mutex.lock().await;
        if let Some(mut stdin) = guard.take() {
            use tokio::io::AsyncWriteExt;
            let _ = stdin.write_all(b"\x03").await;
            let _ = stdin.flush().await;
            tokio::time::sleep(std::time::Duration::from_millis(1200)).await;
        }
    }

    // Force kill
    let _ = std::process::Command::new("kill")
        .arg("-TERM")
        .arg(process.process_id.to_string())
        .output();

    let _ = std::process::Command::new("kill")
        .arg("-KILL")
        .arg(process.process_id.to_string())
        .output();

    // Cleanup
    {
        let processes = app.state::<Mutex<HashMap<String, AgentProcess>>>();
        if let Ok(mut map) = processes.lock() {
            map.remove(session_id);
        }
    }

    Ok(())
}

// ── stdout streaming ──────────────────────────────────────────────

async fn stream_stdout(
    app: tauri::AppHandle,
    agent_id: &str,
    session_id: &str,
    stdout: tokio::process::ChildStdout,
) {
    let reader = BufReader::new(stdout);
    let mut lines = reader.lines();

    while let Ok(Some(line)) = lines.next_line().await {
        if line.trim().is_empty() {
            continue;
        }

        let parsed: serde_json::Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let events = normalize_event(agent_id, &parsed);

        // Handle session resolution aliasing
        for event in &events {
            if let NormalizedEvent::SessionResolved {
                session_id: real_id,
            } = event
            {
                if real_id != session_id {
                    let processes =
                        app.state::<Mutex<HashMap<String, AgentProcess>>>();
                    if let Ok(mut map) = processes.lock() {
                        if let Some(proc) = map.remove(session_id) {
                            map.insert(real_id.clone(), proc);
                        }
                    }
                }
            }
        }

        for event in events {
            let chunk = AgentStreamChunk {
                agent_id: agent_id.to_string(),
                session_id: session_id.to_string(),
                event_type: event.event_type().to_string(),
                data: serde_json::to_value(&event).unwrap_or_default(),
            };
            let _ = app.emit("agent-stream-chunk", &chunk);
        }
    }
}

/// Dispatch normalization to the correct adapter.
fn normalize_event(agent_id: &str, event: &serde_json::Value) -> Vec<NormalizedEvent> {
    match agent_id {
        "claude-code" => crate::adapters::claude_code::normalize_stream_event(event),
        "codex" => crate::adapters::codex::normalize_stream_event(event),
        _ => vec![NormalizedEvent::Raw {
            agent: agent_id.to_string(),
            raw: event.clone(),
        }],
    }
}

// ── stderr draining ───────────────────────────────────────────────

async fn drain_stderr(
    app: tauri::AppHandle,
    agent_id: &str,
    session_id: &str,
    stderr: Option<tokio::process::ChildStderr>,
) {
    let Some(stderr) = stderr else {
        return;
    };

    let reader = BufReader::new(stderr);
    let mut lines = reader.lines();

    while let Ok(Some(line)) = lines.next_line().await {
        if line.trim().is_empty() {
            continue;
        }
        log::warn!("[{agent_id} stderr] {line}");

        let chunk = AgentStreamChunk {
            agent_id: agent_id.to_string(),
            session_id: session_id.to_string(),
            event_type: "error".to_string(),
            data: serde_json::json!({
                "kind": "error",
                "message": line,
                "recoverable": true,
            }),
        };
        let _ = app.emit("agent-stream-chunk", &chunk);
    }
}
