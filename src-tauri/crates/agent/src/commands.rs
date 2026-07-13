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

// ── Agent Config commands ───────────────────────────────────────

#[tauri::command]
pub fn agent_load_config() -> Result<oasis_agent_config::ClaudeConfig, String> {
    oasis_agent_config::load_config().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn agent_save_config(config: oasis_agent_config::ClaudeConfig) -> Result<(), String> {
    oasis_agent_config::save_config(&config).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn agent_load_raw_config() -> Result<oasis_agent_config::RawConfigInfo, String> {
    oasis_agent_config::load_raw_config().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn agent_save_raw_config(content: String) -> Result<(), String> {
    oasis_agent_config::save_raw_config(&content).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn agent_list_backups() -> Result<Vec<oasis_agent_config::BackupEntry>, String> {
    oasis_agent_config::list_backups().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn agent_restore_backup(backup_path: String) -> Result<(), String> {
    oasis_agent_config::restore_backup(&backup_path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn agent_export_config(export_path: String) -> Result<(), String> {
    oasis_agent_config::export_config(&export_path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn agent_import_config(import_path: String) -> Result<oasis_agent_config::ClaudeConfig, String> {
    oasis_agent_config::import_config(&import_path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn agent_load_project_settings(
    project_path: String,
) -> Result<oasis_agent_config::project::ProjectSettings, String> {
    oasis_agent_config::project::load_project_settings(&project_path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn agent_save_project_settings(
    project_path: String,
    settings: oasis_agent_config::project::ProjectSettings,
) -> Result<(), String> {
    oasis_agent_config::project::save_project_settings(&project_path, &settings).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn agent_load_project_settings_local(
    project_path: String,
) -> Result<oasis_agent_config::project::ProjectSettings, String> {
    oasis_agent_config::project::load_project_settings_local(&project_path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn agent_save_project_settings_local(
    project_path: String,
    settings: oasis_agent_config::project::ProjectSettings,
) -> Result<(), String> {
    oasis_agent_config::project::save_project_settings_local(&project_path, &settings).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn agent_load_claude_md(project_path: String) -> Result<Option<String>, String> {
    oasis_agent_config::project::load_claude_md(&project_path).map_err(|e| e.to_string())
}

// ── Hub: Presets ────────────────────────────────────────────────

#[tauri::command]
pub fn agent_list_presets() -> Result<Vec<oasis_hub::Preset>, String> {
    oasis_hub::list_presets().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn agent_save_preset(preset: oasis_hub::Preset) -> Result<(), String> {
    oasis_hub::save_preset(preset).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn agent_delete_preset(id: String) -> Result<(), String> {
    oasis_hub::delete_preset(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn agent_apply_preset(id: String) -> Result<(), String> {
    let presets = oasis_hub::list_presets().map_err(|e| e.to_string())?;
    let preset = presets
        .into_iter()
        .find(|p| p.id == id)
        .ok_or_else(|| format!("preset not found: {id}"))?;
    let config: oasis_agent_config::ClaudeConfig =
        serde_json::from_value(preset.config).map_err(|e| format!("invalid preset config: {e}"))?;
    oasis_agent_config::save_config(&config).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn agent_list_config_templates() -> Result<Vec<oasis_agent_config::templates::ConfigTemplate>, String> {
    Ok(oasis_agent_config::templates::list_config_templates())
}

// ── Hub: Session Names ──────────────────────────────────────────

#[tauri::command]
pub fn agent_get_session_names() -> Result<HashMap<String, String>, String> {
    oasis_hub::get_session_names().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn agent_rename_session(session_id: String, name: String) -> Result<(), String> {
    oasis_hub::rename_session(session_id, name).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn agent_delete_session_name(session_id: String) -> Result<(), String> {
    oasis_hub::delete_session_name(session_id).map_err(|e| e.to_string())
}

// ── Hub: Project Meta ───────────────────────────────────────────

#[tauri::command]
pub fn agent_load_project_metas() -> Result<HashMap<String, oasis_hub::ProjectMeta>, String> {
    oasis_hub::load_project_metas().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn agent_save_project_meta(
    encoded_name: String,
    meta: oasis_hub::ProjectMeta,
) -> Result<(), String> {
    oasis_hub::save_project_meta(&encoded_name, meta).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn agent_remove_project(encoded_name: String) -> Result<(), String> {
    oasis_hub::hide_project(&encoded_name).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn agent_hide_project(encoded_name: String) -> Result<(), String> {
    oasis_hub::hide_project(&encoded_name).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn agent_unhide_project(encoded_name: String) -> Result<(), String> {
    oasis_hub::unhide_project(&encoded_name).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn agent_add_manual_project(path: String) -> Result<(), String> {
    oasis_hub::add_manual_project(&path).map_err(|e| e.to_string())
}

// ── Hub: Project Merges ─────────────────────────────────────────

#[tauri::command]
pub fn agent_get_project_merges() -> Result<HashMap<String, Vec<String>>, String> {
    let merges = oasis_hub::load_project_merges().map_err(|e| e.to_string())?;
    Ok(merges.merges)
}

#[tauri::command]
pub fn agent_merge_projects(primary: String, secondaries: Vec<String>) -> Result<(), String> {
    oasis_hub::merge_projects_logical(&primary, secondaries).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn agent_split_project(primary: String) -> Result<(), String> {
    oasis_hub::split_project(&primary).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn agent_get_merged_secondaries(primary: String) -> Result<Vec<String>, String> {
    oasis_hub::get_merged_secondaries(&primary).map_err(|e| e.to_string())
}

// ── Hub: Commands ───────────────────────────────────────────────

#[tauri::command]
pub fn agent_command_presets(app: AppHandle) -> Result<Vec<crate::command_config::AgentCommandPreset>, String> {
    let registry = app.state::<Mutex<crate::plugin::AgentRegistry>>();
    let reg = registry.lock().map_err(|e| e.to_string())?;
    Ok(crate::command_config::built_in_commands(reg.active_id()))
}

#[tauri::command]
pub fn agent_list_custom_commands() -> Result<Vec<oasis_hub::CustomCommand>, String> {
    oasis_hub::list_custom_commands().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn agent_save_custom_command(cmd: oasis_hub::CustomCommand) -> Result<(), String> {
    oasis_hub::save_custom_command(cmd).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn agent_delete_custom_command(id: String) -> Result<(), String> {
    oasis_hub::delete_custom_command(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn agent_run_in_terminal(command: String, cwd: Option<String>) -> Result<bool, String> {
    oasis_hub::run_in_terminal(&command, cwd.as_deref()).map_err(|e| e.to_string())
}

// ── Environment Check ───────────────────────────────────────────

#[derive(Debug, Clone, serde::Serialize)]
pub struct EnvStatus {
    pub node_installed: bool,
    pub node_version: Option<String>,
    pub npm_installed: bool,
    pub npm_version: Option<String>,
    pub python_installed: bool,
    pub python_version: Option<String>,
}

#[tauri::command]
pub fn agent_check_environment() -> Result<EnvStatus, String> {
    let node_output = std::process::Command::new("which")
        .arg("node")
        .output();
    let node_installed = node_output.map(|o| o.status.success()).unwrap_or(false);
    let node_version = if node_installed {
        std::process::Command::new("node")
            .arg("--version")
            .output()
            .ok()
            .and_then(|o| {
                if o.status.success() {
                    Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
                } else {
                    None
                }
            })
    } else {
        None
    };

    let npm_output = std::process::Command::new("which")
        .arg("npm")
        .output();
    let npm_installed = npm_output.map(|o| o.status.success()).unwrap_or(false);
    let npm_version = if npm_installed {
        std::process::Command::new("npm")
            .arg("--version")
            .output()
            .ok()
            .and_then(|o| {
                if o.status.success() {
                    Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
                } else {
                    None
                }
            })
    } else {
        None
    };

    let python_output = std::process::Command::new("which")
        .arg("python3")
        .output();
    let python_installed = python_output.map(|o| o.status.success()).unwrap_or(false);
    let python_version = if python_installed {
        std::process::Command::new("python3")
            .arg("--version")
            .output()
            .ok()
            .and_then(|o| {
                if o.status.success() {
                    Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
                } else {
                    None
                }
            })
    } else {
        None
    };

    Ok(EnvStatus {
        node_installed,
        node_version,
        npm_installed,
        npm_version,
        python_installed,
        python_version,
    })
}

#[tauri::command]
pub fn agent_check_prerequisite(command: String) -> bool {
    std::process::Command::new("which")
        .arg(&command)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[tauri::command]
pub fn agent_install_agent(command: String) -> Result<String, String> {
    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(&command)
        .output()
        .map_err(|e| format!("failed to run install command: {e}"))?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if output.status.success() {
        Ok(stdout)
    } else {
        Err(format!("Install failed: {stderr}"))
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct KnownAgent {
    pub id: String,
    pub display_name: String,
    pub binary: String,
    pub installed: bool,
    pub version: Option<String>,
    pub install_hint: String,
    pub install_command: Option<String>,
    pub home_url: Option<String>,
    pub description: String,
    pub source: String,
}

struct AgentDef {
    id: &'static str,
    display_name: &'static str,
    binary: &'static str,
    install_hint: &'static str,
    install_command: Option<&'static str>,
    home_url: Option<&'static str>,
    description: &'static str,
}

fn agent_registry() -> Vec<AgentDef> {
    vec![
        AgentDef {
            id: "claude-code",
            display_name: "Claude Code",
            binary: "claude",
            install_hint: "npm install -g @anthropic-ai/claude-code",
            install_command: Some("npm install -g @anthropic-ai/claude-code"),
            home_url: Some("https://docs.anthropic.com/en/docs/claude-code"),
            description: "Anthropic 官方 CLI 编程助手",
        },
        AgentDef {
            id: "codex",
            display_name: "OpenAI Codex",
            binary: "codex",
            install_hint: "npm install -g @openai/codex",
            install_command: Some("npm install -g @openai/codex"),
            home_url: Some("https://github.com/openai/codex"),
            description: "OpenAI Codex CLI 编程助手",
        },
        AgentDef {
            id: "opencode",
            display_name: "OpenCode",
            binary: "opencode",
            install_hint: "go install github.com/opencode-ai/opencode@latest",
            install_command: Some("go install github.com/opencode-ai/opencode@latest"),
            home_url: Some("https://github.com/opencode-ai/opencode"),
            description: "Go 编写的终端 AI 编程助手",
        },
        AgentDef {
            id: "hermes",
            display_name: "Hermes Agent",
            binary: "hermes",
            install_hint: "pip install hermes-agent",
            install_command: Some("pip install hermes-agent"),
            home_url: Some("https://github.com/nicepkg/hermes"),
            description: "Hermes 自主编程 Agent",
        },
        AgentDef {
            id: "openclaw",
            display_name: "OpenClaw",
            binary: "openclaw",
            install_hint: "参阅 OpenClaw 文档安装",
            install_command: None,
            home_url: Some("https://github.com/nicepkg/openclaw"),
            description: "OpenClaw AI 编程助手",
        },
        AgentDef {
            id: "deveco",
            display_name: "DevEco Code",
            binary: "deveco",
            install_hint: "npm install -g deveco-code",
            install_command: Some("npm install -g deveco-code"),
            home_url: Some("https://developer.huawei.com/consumer/cn/deveco-studio/"),
            description: "华为 DevEco Code CLI (HarmonyOS 开发)",
        },
        AgentDef {
            id: "cursor-agent",
            display_name: "Cursor Agent",
            binary: "cursor-agent",
            install_hint: "参阅 Cursor 文档",
            install_command: None,
            home_url: Some("https://cursor.sh"),
            description: "Cursor 编辑器的 Agent CLI",
        },
        AgentDef {
            id: "aider",
            display_name: "Aider",
            binary: "aider",
            install_hint: "pip install aider-chat",
            install_command: Some("pip install aider-chat"),
            home_url: Some("https://aider.chat"),
            description: "AI 配对编程工具，支持多种 LLM",
        },
        AgentDef {
            id: "goose",
            display_name: "Goose",
            binary: "goose",
            install_hint: "curl -fsSL https://github.com/block/goose/releases/latest/download/install.sh | sh",
            install_command: None,
            home_url: Some("https://github.com/block/goose"),
            description: "Block 开发的自主编程 Agent",
        },
        AgentDef {
            id: "amazon-q",
            display_name: "Amazon Q",
            binary: "q",
            install_hint: "参阅 AWS 文档安装 Amazon Q CLI",
            install_command: None,
            home_url: Some("https://docs.aws.amazon.com/amazonq/latest/qdeveloper-ug/command-line.html"),
            description: "Amazon Q 开发者助手 CLI",
        },
        AgentDef {
            id: "gemini-cli",
            display_name: "Gemini CLI",
            binary: "gemini",
            install_hint: "npm install -g @anthropic-ai/claude-code",
            install_command: Some("npm install -g @anthropic-ai/claude-cli"),
            home_url: Some("https://github.com/nicepkg/gemini-cli"),
            description: "Google Gemini 终端编程助手",
        },
        AgentDef {
            id: "devika",
            display_name: "Devika",
            binary: "devika",
            install_hint: "pip install devika",
            install_command: Some("pip install devika"),
            home_url: Some("https://github.com/stitionapi/devika"),
            description: "AI 软件工程师",
        },
        AgentDef {
            id: "swe-agent",
            display_name: "SWE-agent",
            binary: "sweagent",
            install_hint: "pip install sweagent",
            install_command: Some("pip install sweagent"),
            home_url: Some("https://github.com/princeton-nlp/SWE-agent"),
            description: "Princeton SWE-bench Agent",
        },
        AgentDef {
            id: "openhands",
            display_name: "OpenHands",
            binary: "openhands",
            install_hint: "pip install openhands",
            install_command: Some("pip install openhands"),
            home_url: Some("https://github.com/All-Hands-AI/OpenHands"),
            description: "自主软件工程 Agent",
        },
        AgentDef {
            id: "copilot-cli",
            display_name: "GitHub Copilot CLI",
            binary: "github-copilot-cli",
            install_hint: "npm install -g @githubnext/github-copilot-cli",
            install_command: Some("npm install -g @githubnext/github-copilot-cli"),
            home_url: Some("https://githubnext.github.io/projects/copilot-cli"),
            description: "GitHub Copilot 命令行助手",
        },
        AgentDef {
            id: "llm",
            display_name: "llm (Datasette)",
            binary: "llm",
            install_hint: "pip install llm",
            install_command: Some("pip install llm"),
            home_url: Some("https://llm.datasette.io"),
            description: "Datasette llm — 终端 LLM 交互工具",
        },
        AgentDef {
            id: "open-interpreter",
            display_name: "Open Interpreter",
            binary: "interpreter",
            install_hint: "pip install open-interpreter",
            install_command: Some("pip install open-interpreter"),
            home_url: Some("https://github.com/OpenInterpreter/open-interpreter"),
            description: "自然语言控制计算机的 Agent",
        },
        AgentDef {
            id: "gpt-engineer",
            display_name: "GPT Engineer",
            binary: "gptengineer",
            install_hint: "pip install gpt-engineer",
            install_command: Some("pip install gpt-engineer"),
            home_url: Some("https://github.com/gpt-engineer-org/gpt-engineer"),
            description: "AI 软件工程代码生成器",
        },
    ]
}

const VERSION_ARGS: &[&str] = &["--version", "-v", "-V", "version"];

const VERSION_PROBE_TIMEOUT_SECS: u64 = 3;

fn get_version_for_binary(path: &str) -> Option<String> {
    for arg in VERSION_ARGS {
        let (tx, rx) = std::sync::mpsc::channel();
        let path = path.to_string();
        let arg = arg.to_string();
        std::thread::spawn(move || {
            let result = std::process::Command::new(&path)
                .arg(&arg)
                .env("TERM", "dumb")
                .env("NO_COLOR", "1")
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .output()
                .ok()
                .and_then(|o| {
                    let stdout = String::from_utf8_lossy(&o.stdout).trim().to_string();
                    let stderr = String::from_utf8_lossy(&o.stderr).trim().to_string();
                    let combined = if stdout.is_empty() { stderr } else { stdout };
                    let first_line = combined.lines().next().unwrap_or(&combined);
                    let v = first_line.to_string();
                    if v.is_empty() { None } else { Some(v) }
                });
            let _ = tx.send(result);
        });
        match rx.recv_timeout(std::time::Duration::from_secs(VERSION_PROBE_TIMEOUT_SECS)) {
            Ok(Some(v)) => return Some(v),
            Ok(None) => continue,
            Err(_) => continue,
        }
    }
    None
}

fn find_binary_in_path(binary: &str) -> Option<String> {
    let which_output = std::process::Command::new("which")
        .arg(binary)
        .output()
        .ok()?;
    if which_output.status.success() {
        let path = String::from_utf8_lossy(&which_output.stdout).trim().to_string();
        if !path.is_empty() {
            return Some(path);
        }
    }
    None
}

#[tauri::command]
pub async fn agent_probe_known_agents() -> Vec<KnownAgent> {
    tokio::task::spawn_blocking(|| {
        let registry = agent_registry();
        let mut agents: Vec<KnownAgent> = Vec::new();

        for def in &registry {
            let installed = find_binary_in_path(def.binary).is_some();
            let version = if installed {
                find_binary_in_path(def.binary)
                    .and_then(|path| get_version_for_binary(&path))
            } else {
                None
            };
            agents.push(KnownAgent {
                id: def.id.to_string(),
                display_name: def.display_name.to_string(),
                binary: def.binary.to_string(),
                installed,
                version,
                install_hint: def.install_hint.to_string(),
                install_command: def.install_command.map(|s| s.to_string()),
                home_url: def.home_url.map(|s| s.to_string()),
                description: def.description.to_string(),
                source: if installed { "registry".to_string() } else { "registry".to_string() },
            });
        }

        agents.sort_by(|a, b| {
            b.installed.cmp(&a.installed).then(a.display_name.cmp(&b.display_name))
        });

        agents
    })
    .await
    .unwrap_or_default()
}
