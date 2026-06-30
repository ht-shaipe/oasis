use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::Manager;

const CONFIG_FILE: &str = "local_llm_config.json";

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct LocalLlmConfig {
    #[serde(default)]
    pub active_model_id: Option<String>,
    #[serde(default)]
    pub hidden_ids: Vec<String>,
}

fn app_data_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

fn config_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    Ok(app_data_dir(app)?.join(CONFIG_FILE))
}

pub fn load_config(app: &tauri::AppHandle) -> Result<LocalLlmConfig, String> {
    let path = config_path(app)?;
    if path.exists() {
        let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        serde_json::from_str(&content).map_err(|e| e.to_string())
    } else {
        Ok(LocalLlmConfig::default())
    }
}

pub fn save_config(app: &tauri::AppHandle, config: &LocalLlmConfig) -> Result<(), String> {
    let path = config_path(app)?;
    let content = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    fs::write(&path, content).map_err(|e| e.to_string())
}

pub fn models_cache_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let dir = app_data_dir(app)?.join("local_llm_models");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}
