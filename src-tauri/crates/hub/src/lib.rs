//! Oasis Hub — persistent application state stored in `~/.oasis/`.
//!
//! Ported from jishu-hub `hub.rs`, adapted for macOS and `~/.oasis/` path.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

// ── Helpers ────────────────────────────────────────────────────

pub fn hub_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let home = dirs_next::home_dir().ok_or("Cannot find home directory")?;
    let dir = home.join(".oasis");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn read_json<T: for<'de> Deserialize<'de>>(
    path: &PathBuf,
) -> Result<T, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&content)?)
}

fn write_json<T: Serialize>(path: &PathBuf, data: &T) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string_pretty(data)?;
    atomic_write(path, json.as_bytes())?;
    Ok(())
}

pub fn atomic_write(path: &PathBuf, content: &[u8]) -> std::io::Result<()> {
    let tmp = unique_tmp_path(path);
    std::fs::write(&tmp, content)?;
    if let Err(e) = std::fs::rename(&tmp, path) {
        let _ = std::fs::remove_file(&tmp);
        return Err(e);
    }
    Ok(())
}

fn unique_tmp_path(path: &PathBuf) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let mut name = path
        .file_name()
        .map(|n| n.to_os_string())
        .unwrap_or_default();
    name.push(format!(".{}-{}.tmp", std::process::id(), nanos));
    path.with_file_name(name)
}

// ── AppState ───────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct AppState {
    pub last_project: Option<String>,
    pub language: Option<String>,
    pub always_on_top: Option<bool>,
    pub theme: Option<String>,
    pub font_size_base: Option<String>,
    pub font_size_prose: Option<String>,
    #[serde(default)]
    pub active_agent_id: Option<String>,
    #[serde(default)]
    pub agent_binary_paths: HashMap<String, String>,
    #[serde(default)]
    pub agent_last_health: HashMap<String, serde_json::Value>,
}

fn state_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    Ok(hub_dir()?.join("state.json"))
}

pub fn load_state() -> Result<AppState, Box<dyn std::error::Error>> {
    let path = state_path()?;
    if !path.exists() {
        return Ok(AppState::default());
    }
    read_json(&path)
}

pub fn save_state(state: &AppState) -> Result<(), Box<dyn std::error::Error>> {
    write_json(&state_path()?, state)
}

pub fn load_active_agent_id() -> Result<Option<String>, Box<dyn std::error::Error>> {
    Ok(load_state()?.active_agent_id)
}

pub fn save_active_agent_id(id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut state = load_state().unwrap_or_default();
    state.active_agent_id = Some(id.to_string());
    save_state(&state)
}

pub fn load_language() -> Result<Option<String>, Box<dyn std::error::Error>> {
    Ok(load_state()?.language)
}

pub fn save_language(lang: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut state = load_state().unwrap_or_default();
    state.language = Some(lang.to_string());
    save_state(&state)
}

pub fn load_always_on_top() -> Result<bool, Box<dyn std::error::Error>> {
    Ok(load_state().unwrap_or_default().always_on_top.unwrap_or(false))
}

pub fn save_always_on_top(value: bool) -> Result<(), Box<dyn std::error::Error>> {
    let mut state = load_state().unwrap_or_default();
    state.always_on_top = Some(value);
    save_state(&state)
}

pub fn load_last_project() -> Result<Option<String>, Box<dyn std::error::Error>> {
    Ok(load_state()?.last_project)
}

pub fn save_last_project(encoded_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut state = load_state().unwrap_or_default();
    state.last_project = Some(encoded_name.to_string());
    save_state(&state)
}

pub fn load_font_sizes() -> Result<(Option<String>, Option<String>), Box<dyn std::error::Error>> {
    let state = load_state()?;
    Ok((state.font_size_base, state.font_size_prose))
}

pub fn save_font_sizes(base: &str, prose: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut state = load_state().unwrap_or_default();
    state.font_size_base = Some(base.to_string());
    state.font_size_prose = Some(prose.to_string());
    save_state(&state)
}

// ── Session Names ──────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct SessionNames {
    pub names: HashMap<String, String>,
}

fn session_names_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    Ok(hub_dir()?.join("session_names.json"))
}

pub fn get_session_names() -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    let path = session_names_path()?;
    if !path.exists() {
        return Ok(HashMap::new());
    }
    let data: SessionNames = read_json(&path)?;
    Ok(data.names)
}

pub fn rename_session(session_id: String, name: String) -> Result<(), Box<dyn std::error::Error>> {
    let path = session_names_path()?;
    let mut data = if path.exists() {
        read_json::<SessionNames>(&path)?
    } else {
        SessionNames::default()
    };
    data.names.insert(session_id, name);
    write_json(&path, &data)
}

pub fn delete_session_name(session_id: String) -> Result<(), Box<dyn std::error::Error>> {
    let path = session_names_path()?;
    if !path.exists() {
        return Ok(());
    }
    let mut data: SessionNames = read_json(&path)?;
    data.names.remove(&session_id);
    write_json(&path, &data)
}

// ── Hidden Projects ─────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct HiddenProjects {
    pub encoded_names: Vec<String>,
}

fn hidden_projects_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    Ok(hub_dir()?.join("hidden_projects.json"))
}

pub fn hide_project(encoded_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let path = hidden_projects_path()?;
    let mut data = if path.exists() {
        read_json::<HiddenProjects>(&path)?
    } else {
        HiddenProjects::default()
    };
    if !data.encoded_names.contains(&encoded_name.to_string()) {
        data.encoded_names.push(encoded_name.to_string());
    }
    write_json(&path, &data)
}

pub fn unhide_project(encoded_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let path = hidden_projects_path()?;
    if !path.exists() {
        return Ok(());
    }
    let mut data: HiddenProjects = read_json(&path)?;
    data.encoded_names.retain(|e| e != encoded_name);
    write_json(&path, &data)
}

pub fn is_project_hidden(encoded_name: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let path = hidden_projects_path()?;
    if !path.exists() {
        return Ok(false);
    }
    let data: HiddenProjects = read_json(&path)?;
    Ok(data.encoded_names.contains(&encoded_name.to_string()))
}

// ── Manual Projects ─────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ManualProjects {
    #[serde(default)]
    pub paths: Vec<String>,
}

fn manual_projects_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    Ok(hub_dir()?.join("manual_projects.json"))
}

pub fn add_manual_project(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let path_obj = manual_projects_path()?;
    let mut data = if path_obj.exists() {
        read_json::<ManualProjects>(&path_obj)?
    } else {
        ManualProjects::default()
    };
    if !data.paths.contains(&path.to_string()) {
        data.paths.push(path.to_string());
    }
    write_json(&path_obj, &data)
}

pub fn remove_manual_project(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let path_obj = manual_projects_path()?;
    if !path_obj.exists() {
        return Ok(());
    }
    let mut data: ManualProjects = read_json(&path_obj)?;
    data.paths.retain(|p| p != path);
    write_json(&path_obj, &data)
}

pub fn load_manual_projects() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let path = manual_projects_path()?;
    if !path.exists() {
        return Ok(Vec::new());
    }
    let data: ManualProjects = read_json(&path)?;
    Ok(data.paths)
}

// ── Presets ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preset {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub config: serde_json::Value,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Presets {
    pub presets: Vec<Preset>,
}

fn presets_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    Ok(hub_dir()?.join("presets.json"))
}

pub fn list_presets() -> Result<Vec<Preset>, Box<dyn std::error::Error>> {
    let path = presets_path()?;
    if !path.exists() {
        return Ok(Vec::new());
    }
    let data: Presets = read_json(&path)?;
    Ok(data.presets)
}

pub fn save_preset(preset: Preset) -> Result<(), Box<dyn std::error::Error>> {
    let path = presets_path()?;
    let mut data = if path.exists() {
        read_json::<Presets>(&path)?
    } else {
        Presets::default()
    };
    if let Some(idx) = data.presets.iter().position(|p| p.id == preset.id) {
        data.presets[idx] = preset;
    } else {
        data.presets.push(preset);
    }
    write_json(&path, &data)
}

pub fn delete_preset(id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let path = presets_path()?;
    if !path.exists() {
        return Ok(());
    }
    let mut data: Presets = read_json(&path)?;
    data.presets.retain(|p| p.id != id);
    write_json(&path, &data)
}

// ── Project Meta ────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectMeta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectMetas {
    #[serde(default)]
    pub metas: HashMap<String, ProjectMeta>,
}

fn project_metas_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    Ok(hub_dir()?.join("project_meta.json"))
}

pub fn load_project_metas() -> Result<HashMap<String, ProjectMeta>, Box<dyn std::error::Error>> {
    let path = project_metas_path()?;
    if !path.exists() {
        return Ok(HashMap::new());
    }
    let metas: ProjectMetas = read_json(&path)?;
    Ok(metas.metas)
}

pub fn save_project_meta(
    encoded_name: &str,
    meta: ProjectMeta,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = project_metas_path()?;
    let mut metas = if path.exists() {
        read_json::<ProjectMetas>(&path)?
    } else {
        ProjectMetas::default()
    };
    if meta.custom_name.is_none() && meta.tags.is_none() && meta.notes.is_none() {
        metas.metas.remove(encoded_name);
    } else {
        metas.metas.insert(encoded_name.to_string(), meta);
    }
    write_json(&path, &metas)
}

// ── Project Merges ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectMerges {
    #[serde(default)]
    pub merges: HashMap<String, Vec<String>>,
}

fn project_merges_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    Ok(hub_dir()?.join("project_merges.json"))
}

pub fn load_project_merges() -> Result<ProjectMerges, Box<dyn std::error::Error>> {
    let path = project_merges_path()?;
    if !path.exists() {
        return Ok(ProjectMerges::default());
    }
    read_json(&path)
}

fn save_project_merges(merges: &ProjectMerges) -> Result<(), Box<dyn std::error::Error>> {
    write_json(&project_merges_path()?, merges)
}

pub fn merge_projects_logical(
    primary: &str,
    secondaries: Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut merges = load_project_merges()?;
    for s in &secondaries {
        merges.merges.remove(s);
    }
    let existing = merges.merges.entry(primary.to_string()).or_default();
    for s in secondaries {
        if !existing.contains(&s) {
            existing.push(s);
        }
    }
    save_project_merges(&merges)
}

pub fn split_project(primary: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut merges = load_project_merges()?;
    merges.merges.remove(primary);
    save_project_merges(&merges)
}

pub fn get_merged_secondaries(primary: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let merges = load_project_merges()?;
    Ok(merges.merges.get(primary).cloned().unwrap_or_default())
}

pub fn get_all_secondaries() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let merges = load_project_merges()?;
    Ok(merges.merges.values().flatten().cloned().collect())
}

pub fn resolve_primary(secondary: &str) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let merges = load_project_merges()?;
    for (primary, secondaries) in &merges.merges {
        if secondaries.contains(&secondary.to_string()) {
            return Ok(Some(primary.clone()));
        }
    }
    Ok(None)
}

// ── Custom Commands ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomCommand {
    pub id: String,
    pub name: String,
    pub command: String,
    #[serde(rename = "agentId", default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    #[serde(rename = "projectPath", default, skip_serializing_if = "Option::is_none")]
    pub project_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Commands {
    pub commands: Vec<CustomCommand>,
}

fn commands_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    Ok(hub_dir()?.join("commands.json"))
}

pub fn list_custom_commands() -> Result<Vec<CustomCommand>, Box<dyn std::error::Error>> {
    let path = commands_path()?;
    if !path.exists() {
        return Ok(Vec::new());
    }
    let data: Commands = read_json(&path)?;
    Ok(data.commands)
}

pub fn save_custom_command(cmd: CustomCommand) -> Result<(), Box<dyn std::error::Error>> {
    let path = commands_path()?;
    let mut data = if path.exists() {
        read_json::<Commands>(&path)?
    } else {
        Commands::default()
    };
    if let Some(idx) = data.commands.iter().position(|c| c.id == cmd.id) {
        data.commands[idx] = cmd;
    } else {
        data.commands.push(cmd);
    }
    write_json(&path, &data)
}

pub fn delete_custom_command(id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let path = commands_path()?;
    if !path.exists() {
        return Ok(());
    }
    let mut data: Commands = read_json(&path)?;
    data.commands.retain(|c| c.id != id);
    write_json(&path, &data)
}

// ── Terminal Command Execution (macOS) ──────────────────────────

pub fn run_in_terminal(
    command: &str,
    cwd: Option<&str>,
) -> Result<bool, Box<dyn std::error::Error>> {
    let escaped = command.replace('\'', "'\\''");
    let shell_cmd = match cwd {
        Some(dir) => format!(
            "cd '{}' && echo '> {}'; echo; {}; exec bash",
            dir.replace('\'', "'\\''"),
            escaped,
            command
        ),
        None => format!("echo '> {}'; echo; {}; exec bash", escaped, command),
    };
    std::process::Command::new("open")
        .args(["-a", "Terminal"])
        .spawn()?;
    std::thread::sleep(std::time::Duration::from_millis(500));
    std::process::Command::new("osascript")
        .args(["-e", &format!("tell application \"Terminal\" to do script \"{}\"", shell_cmd)])
        .spawn()?;
    Ok(true)
}

pub fn open_in_terminal_with_command(
    project_path: &str,
    command: &str,
) -> Result<u32, Box<dyn std::error::Error>> {
    let child = std::process::Command::new("open")
        .args(["-a", "Terminal", project_path])
        .spawn()?;
    std::thread::sleep(std::time::Duration::from_millis(500));
    let safe_path = project_path.replace('\'', "'\\''");
    std::process::Command::new("osascript")
        .args([
            "-e",
            &format!(
                "tell application \"Terminal\" to do script \"cd '{}' && {}\"",
                safe_path, command
            ),
        ])
        .spawn()?;
    Ok(child.id())
}
