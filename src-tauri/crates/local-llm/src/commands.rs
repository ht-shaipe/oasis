use crate::catalog::{find_catalog_entry, model_catalog};
use crate::config;
use crate::inference;
use crate::progress::{DownloadProgressPayload, TauriProgress};
use hf_hub::api::sync::ApiBuilder;
use serde::{Deserialize, Serialize};
use std::fs;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::Emitter;

static DOWNLOAD_CANCELLED: AtomicBool = AtomicBool::new(false);

const TOKENIZER_FILES: [&str; 4] = [
    "tokenizer.json",
    "config.json",
    "special_tokens_map.json",
    "tokenizer_config.json",
];

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LocalChatModel {
    pub id: String,
    pub name: String,
    pub params_billions: f64,
    pub size_mb: f64,
    pub license: String,
    pub description: String,
    pub downloaded: bool,
    pub is_active: bool,
    pub is_loaded: bool,
}

#[tauri::command]
pub fn list_local_chat_models(
    app: tauri::AppHandle,
) -> Result<Vec<LocalChatModel>, String> {
    let cfg = config::load_config(&app)?;
    let catalog = model_catalog();

    let hidden_set: std::collections::HashSet<&str> = cfg
        .hidden_ids
        .iter()
        .map(|s| s.as_str())
        .collect();

    let active_id = cfg.active_model_id.as_deref();
    let loaded_id = inference::get_loaded_model_id();

    let result: Vec<LocalChatModel> = catalog
        .iter()
        .filter(|entry| !hidden_set.contains(entry.id))
        .map(|entry| LocalChatModel {
            id: entry.id.to_string(),
            name: entry.name.to_string(),
            params_billions: entry.params_billions,
            size_mb: entry.size_mb,
            license: entry.license.to_string(),
            description: entry.description.to_string(),
            downloaded: inference::is_model_downloaded(&app, entry.id),
            is_active: active_id == Some(entry.id),
            is_loaded: loaded_id.as_deref() == Some(entry.id),
        })
        .collect();

    Ok(result)
}

#[tauri::command]
pub async fn download_local_chat_model(
    app: tauri::AppHandle,
    model_id: String,
    auto_activate: Option<bool>,
) -> Result<bool, String> {
    DOWNLOAD_CANCELLED.store(false, Ordering::SeqCst);

    let entry = find_catalog_entry(&model_id)
        .ok_or(format!("Unknown model: {}", model_id))?;

    let cache_dir = config::models_cache_dir(&app)?;

    let _ = app.emit(
        "local-llm-download-progress",
        DownloadProgressPayload {
            model_id: model_id.clone(),
            file_name: "preparing".to_string(),
            current: 0,
            total: 0,
            percentage: 0,
            status: "starting".to_string(),
        },
    );

    let app_clone = app.clone();
    let model_id_clone = model_id.clone();
    let hf_repo = entry.hf_repo.to_string();
    let gguf_file = entry.gguf_file.to_string();
    let tok_model_id = entry.tok_model_id.to_string();

    tauri::async_runtime::spawn_blocking(move || {
        let api = ApiBuilder::new()
            .with_cache_dir(cache_dir)
            .build()
            .map_err(|e| format!("Failed to create HF API: {}", e))?;

        let gguf_repo = api.model(hf_repo.clone());
        let tok_repo = api.model(tok_model_id.clone());

        let mut all_files: Vec<String> = TOKENIZER_FILES
            .iter()
            .map(|s| s.to_string())
            .collect();
        all_files.push(gguf_file.clone());

        let total_files = all_files.len();
        for (idx, file_name) in all_files.iter().enumerate() {
            if DOWNLOAD_CANCELLED.load(Ordering::SeqCst) {
                let _ = app_clone.emit(
                    "local-llm-download-progress",
                    DownloadProgressPayload {
                        model_id: model_id_clone.clone(),
                        file_name: file_name.clone(),
                        current: 0,
                        total: 0,
                        percentage: 0,
                        status: "cancelled".to_string(),
                    },
                );
                return Err("Download cancelled".to_string());
            }

            let overall_pct = ((idx * 100) / total_files) as u32;

            let _ = app_clone.emit(
                "local-llm-download-progress",
                DownloadProgressPayload {
                    model_id: model_id_clone.clone(),
                    file_name: format!("{} ({}/{})", file_name, idx + 1, total_files),
                    current: 0,
                    total: total_files,
                    percentage: overall_pct,
                    status: "downloading".to_string(),
                },
            );

            let repo = if file_name.ends_with(".gguf") {
                &gguf_repo
            } else {
                &tok_repo
            };

            let file_progress = TauriProgress {
                app: app_clone.clone(),
                model_id: model_id_clone.clone(),
                file_name: String::new(),
                total: 0,
                current: 0,
            };

            match repo.download_with_progress(file_name, file_progress) {
                Ok(_) => {}
                Err(e) => {
                    let err_str = format!("{}", e);
                    if err_str.contains("404") || err_str.contains("Not Found") {
                        eprintln!(
                            "Warning: skipping '{}' for model '{}' (not found in repo)",
                            file_name, model_id_clone
                        );
                    } else {
                        return Err(format!(
                            "Failed to download '{}': {}",
                            file_name, e
                        ));
                    }
                }
            }
        }

        let _ = app_clone.emit(
            "local-llm-download-progress",
            DownloadProgressPayload {
                model_id: model_id_clone.clone(),
                file_name: "complete".to_string(),
                current: total_files,
                total: total_files,
                percentage: 100,
                status: "complete".to_string(),
            },
        );

        let should_activate = auto_activate.unwrap_or(true);
        if should_activate {
            let mut cfg = config::load_config(&app_clone)?;
            cfg.active_model_id = Some(model_id_clone.clone());
            config::save_config(&app_clone, &cfg)?;
            return Ok(true);
        }

        Ok(false)
    })
    .await
    .map_err(|e| format!("Download task error: {}", e))?
}

#[tauri::command]
pub fn cancel_local_chat_download() -> Result<(), String> {
    DOWNLOAD_CANCELLED.store(true, Ordering::SeqCst);
    Ok(())
}

#[tauri::command]
pub fn delete_local_chat_model(
    app: tauri::AppHandle,
    model_id: String,
) -> Result<(), String> {
    let cache_dir = config::models_cache_dir(&app)?;
    let entry = find_catalog_entry(&model_id)
        .ok_or(format!("Unknown model: {}", model_id))?;

    for repo in &[entry.hf_repo, entry.tok_model_id] {
        let org_model = repo.replace('/', "--");
        let model_dir = cache_dir.join(format!("models--{}", org_model));
        if model_dir.exists() {
            fs::remove_dir_all(&model_dir).map_err(|e| e.to_string())?;
        }
    }

    let mut cfg = config::load_config(&app)?;
    if cfg.active_model_id.as_deref() == Some(&model_id) {
        cfg.active_model_id = None;
        config::save_config(&app, &cfg)?;
    }

    Ok(())
}

#[tauri::command]
pub async fn set_active_local_chat_model(
    app: tauri::AppHandle,
    model_id: String,
) -> Result<(), String> {
    if !inference::is_model_downloaded(&app, &model_id) {
        return Err(format!(
            "Model '{}' is not downloaded. Please download it first.",
            model_id
        ));
    }

    let mut cfg = config::load_config(&app)?;
    cfg.active_model_id = Some(model_id.clone());
    config::save_config(&app, &cfg)?;

    inference::load_model(&app, &model_id).await?;

    Ok(())
}

#[tauri::command]
pub async fn unload_local_chat_model() -> Result<(), String> {
    inference::unload_model().await
}

#[tauri::command]
pub fn get_local_chat_config(
    app: tauri::AppHandle,
) -> Result<config::LocalLlmConfig, String> {
    config::load_config(&app)
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ModelStatus {
    pub active_model_id: Option<String>,
    pub is_loaded: bool,
    pub loaded_model_id: Option<String>,
}

#[tauri::command]
pub fn get_local_chat_model_status() -> Result<ModelStatus, String> {
    let loaded_id = inference::get_loaded_model_id();
    Ok(ModelStatus {
        active_model_id: None,
        is_loaded: loaded_id.is_some(),
        loaded_model_id: loaded_id,
    })
}

#[tauri::command]
pub fn hide_local_chat_model(
    app: tauri::AppHandle,
    model_id: String,
) -> Result<(), String> {
    if find_catalog_entry(&model_id).is_none() {
        return Err("Only built-in models can be hidden.".to_string());
    }
    let mut cfg = config::load_config(&app)?;
    if !cfg.hidden_ids.contains(&model_id) {
        cfg.hidden_ids.push(model_id.clone());
    }
    if cfg.active_model_id.as_deref() == Some(&model_id) {
        cfg.active_model_id = None;
    }
    config::save_config(&app, &cfg)
}

#[tauri::command]
pub fn restore_local_chat_model(
    app: tauri::AppHandle,
    model_id: String,
) -> Result<(), String> {
    let mut cfg = config::load_config(&app)?;
    cfg.hidden_ids.retain(|id| id != &model_id);
    config::save_config(&app, &cfg)
}
