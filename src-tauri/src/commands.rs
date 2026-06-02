use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use std::fs;
use std::path::PathBuf;

const WORKSPACE_CONFIG_FILE: &str = "workspace.json";
const DEFAULT_WORKSPACE: &str = ".oasis";

fn config_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

fn workspace_config_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(config_dir(app)?.join(WORKSPACE_CONFIG_FILE))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct WorkspaceConfig {
    pub workspace_dir: String,
}

impl Default for WorkspaceConfig {
    fn default() -> Self {
        let home = dirs_home();
        Self {
            workspace_dir: format!("{}/{}", home, DEFAULT_WORKSPACE),
        }
    }
}

fn dirs_home() -> String {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| "/".to_string())
}

fn load_workspace_config(app: &AppHandle) -> Result<WorkspaceConfig, String> {
    let path = workspace_config_path(app)?;
    if path.exists() {
        let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let config: WorkspaceConfig =
            serde_json::from_str(&content).map_err(|e| e.to_string())?;
        Ok(config)
    } else {
        Ok(WorkspaceConfig::default())
    }
}

fn save_workspace_config(app: &AppHandle, config: &WorkspaceConfig) -> Result<(), String> {
    let path = workspace_config_path(app)?;
    let content = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    fs::write(&path, content).map_err(|e| e.to_string())
}

/// Get the current workspace directory path (expanded absolute path).
#[tauri::command]
pub fn get_workspace_dir(app: AppHandle) -> Result<String, String> {
    let config = load_workspace_config(&app)?;
    let expanded = if config.workspace_dir.starts_with('~') {
        config.workspace_dir.replacen('~', &dirs_home(), 1)
    } else {
        config.workspace_dir
    };
    Ok(expanded)
}

/// Set the workspace directory path. Creates the directory if it doesn't exist.
/// Returns the expanded absolute path.
#[tauri::command]
pub fn set_workspace_dir(app: AppHandle, path: String) -> Result<String, String> {
    let expanded = if path.starts_with('~') {
        path.replacen('~', &dirs_home(), 1)
    } else {
        path
    };

    fs::create_dir_all(&expanded).map_err(|e| format!("Failed to create directory: {}", e))?;

    let config = WorkspaceConfig {
        workspace_dir: expanded.clone(),
    };
    save_workspace_config(&app, &config)?;

    Ok(expanded)
}

#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! Welcome to Oasis 🏜️", name)
}

#[derive(Serialize, Deserialize)]
pub struct TrayLocale {
    pub show: String,
    pub hide: String,
    pub about: String,
    pub quit: String,
}

#[tauri::command]
pub fn update_tray_locale(app: AppHandle, locale: TrayLocale) -> Result<(), String> {
    let ids_and_texts = [
        ("show", &locale.show),
        ("hide", &locale.hide),
        ("about", &locale.about),
        ("quit", &locale.quit),
    ];
    for (id, text) in &ids_and_texts {
        if let Some(menu) = app.menu() {
            if let Some(item) = menu.get(&tauri::menu::MenuId::new(*id)) {
                if let Some(mi) = item.as_menuitem() {
                    let _ = mi.set_text(text);
                }
            }
        }
    }
    Ok(())
}
