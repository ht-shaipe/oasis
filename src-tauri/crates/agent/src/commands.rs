//! Tauri commands exposed by oasis-agent.

use std::collections::HashMap;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

use crate::plugin::ChatArgs;
use crate::runtime::{self, AgentProcess};

// ── Tauri commands ────────────────────────────────────────────────

#[tauri::command]
pub fn agent_list(app: AppHandle) -> Result<Vec<crate::plugin::AgentInfo>, String> {
    let registry = app.state::<Mutex<crate::plugin::AgentRegistry>>();
    let reg = registry.lock().map_err(|e| e.to_string())?;
    Ok(reg.list_agents())
}

#[tauri::command]
pub fn agent_list_statuses(app: AppHandle) -> Result<Vec<crate::plugin::AgentStatus>, String> {
    let registry = app.state::<Mutex<crate::plugin::AgentRegistry>>();
    let reg = registry.lock().map_err(|e| e.to_string())?;
    Ok(reg.list_agent_statuses())
}

#[tauri::command]
pub fn agent_set_active(app: AppHandle, id: String) -> Result<(), String> {
    let registry = app.state::<Mutex<crate::plugin::AgentRegistry>>();
    let mut reg = registry.lock().map_err(|e| e.to_string())?;
    reg.set_active(&id)
}

#[tauri::command]
pub fn agent_get_active(app: AppHandle) -> Result<String, String> {
    let registry = app.state::<Mutex<crate::plugin::AgentRegistry>>();
    let reg = registry.lock().map_err(|e| e.to_string())?;
    Ok(reg.active_id().to_string())
}

#[tauri::command]
pub async fn agent_send_message(
    app: AppHandle,
    project_path: String,
    session_id: Option<String>,
    message: String,
) -> Result<crate::plugin::ChatResult, String> {
    let args = ChatArgs {
        project_path: project_path.clone(),
        session_id: session_id.clone(),
        message,
    };

    let agent_id = {
        let registry = app.state::<Mutex<crate::plugin::AgentRegistry>>();
        let reg = registry.lock().map_err(|e| e.to_string())?;
        reg.active_id().to_string()
    };

    let (agent_id, sid, pid) =
        runtime::spawn_agent_chat(&app, &agent_id, &args, session_id).await?;

    Ok(crate::plugin::ChatResult {
        agent_id,
        session_id: sid,
        process_id: pid,
    })
}

#[tauri::command]
pub async fn agent_abort(app: AppHandle, session_id: String) -> Result<(), String> {
    runtime::abort_agent_chat(&app, &session_id).await
}

#[tauri::command]
pub fn agent_scan_projects(app: AppHandle) -> Result<Vec<crate::plugin::ProjectEntry>, String> {
    let registry = app.state::<Mutex<crate::plugin::AgentRegistry>>();
    let reg = registry.lock().map_err(|e| e.to_string())?;
    Ok(reg.active().scan_projects())
}

#[tauri::command]
pub fn agent_list_sessions(
    app: AppHandle,
    project_path: String,
) -> Result<Vec<crate::plugin::SessionInfo>, String> {
    let registry = app.state::<Mutex<crate::plugin::AgentRegistry>>();
    let reg = registry.lock().map_err(|e| e.to_string())?;
    reg.active().list_sessions(&project_path)
}

#[tauri::command]
pub fn agent_load_session(
    app: AppHandle,
    session_id: String,
    project_path: String,
) -> Result<Vec<crate::plugin::SessionMessage>, String> {
    let registry = app.state::<Mutex<crate::plugin::AgentRegistry>>();
    let reg = registry.lock().map_err(|e| e.to_string())?;
    reg.active()
        .load_session_messages(&session_id, &project_path)
}

#[tauri::command]
pub fn agent_add_project(
    app: AppHandle,
    path: String,
) -> Result<Option<crate::plugin::ProjectEntry>, String> {
    let registry = app.state::<Mutex<crate::plugin::AgentRegistry>>();
    let reg = registry.lock().map_err(|e| e.to_string())?;
    Ok(reg.active().add_project(&path))
}

#[tauri::command]
pub fn agent_init_project(app: AppHandle, project_path: String) -> Result<bool, String> {
    let registry = app.state::<Mutex<crate::plugin::AgentRegistry>>();
    let reg = registry.lock().map_err(|e| e.to_string())?;
    reg.active().init_project(&project_path)
}

#[tauri::command]
pub fn agent_open_terminal(
    app: AppHandle,
    project_path: String,
    resume_session_id: Option<String>,
) -> Result<u32, String> {
    let registry = app.state::<Mutex<crate::plugin::AgentRegistry>>();
    let reg = registry.lock().map_err(|e| e.to_string())?;
    reg.active()
        .open_in_terminal(&project_path, resume_session_id.as_deref())
}

#[tauri::command]
pub fn agent_get_streaming(app: AppHandle) -> Result<Vec<String>, String> {
    let state = app.state::<Mutex<HashMap<String, AgentProcess>>>();
    let map = state.lock().map_err(|e| e.to_string())?;
    Ok(map.keys().cloned().collect())
}

#[tauri::command]
pub fn agent_refresh_health(app: AppHandle) -> Result<Vec<crate::plugin::AgentStatus>, String> {
    let registry = app.state::<Mutex<crate::plugin::AgentRegistry>>();
    let mut reg = registry.lock().map_err(|e| e.to_string())?;
    Ok(reg.refresh_health_cache())
}

#[tauri::command]
pub fn agent_scan_all_projects(app: AppHandle) -> Result<Vec<crate::plugin::ProjectEntry>, String> {
    let registry = app.state::<Mutex<crate::plugin::AgentRegistry>>();
    let reg = registry.lock().map_err(|e| e.to_string())?;
    Ok(reg.scan_all_projects())
}
