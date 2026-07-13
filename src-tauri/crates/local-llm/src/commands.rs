use crate::catalog::{find_catalog_entry, model_catalog};
use crate::config;
use crate::config::CustomLocalModelEntry;
use crate::inference;
use crate::progress::{DownloadProgressPayload, TauriProgress};
use hf_hub::api::sync::ApiBuilder;
use serde::{Deserialize, Serialize};
use std::fs;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::Emitter;

const HF_API_BASE: &str = "https://huggingface.co/api";

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
    pub is_custom: bool,
    pub gguf_file: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ModelResolveInfo {
    pub hf_repo: String,
    pub gguf_file: String,
    pub tok_model_id: String,
}

fn resolve_model_info(
    model_id: &str,
    cfg: &config::LocalLlmConfig,
) -> Option<ModelResolveInfo> {
    if let Some(entry) = find_catalog_entry(model_id) {
        Some(ModelResolveInfo {
            hf_repo: entry.hf_repo.to_string(),
            gguf_file: entry.gguf_file.to_string(),
            tok_model_id: entry.tok_model_id.to_string(),
        })
    } else if let Some(custom) = cfg.custom_models.iter().find(|m| m.model_id == model_id) {
        Some(ModelResolveInfo {
            hf_repo: custom.hf_repo.clone(),
            gguf_file: custom.gguf_file.clone(),
            tok_model_id: custom.tok_model_id.clone(),
        })
    } else {
        None
    }
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

    let mut result: Vec<LocalChatModel> = catalog
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
            is_custom: false,
            gguf_file: entry.gguf_file.to_string(),
        })
        .collect();

    for custom in &cfg.custom_models {
        if !result.iter().any(|m| m.id == custom.model_id) {
            result.push(LocalChatModel {
                id: custom.model_id.clone(),
                name: custom.name.clone(),
                params_billions: custom.params_billions,
                size_mb: custom.size_mb,
                license: custom.license.clone(),
                description: custom.description.clone(),
                downloaded: inference::is_custom_model_downloaded(&app, &custom.hf_repo, &custom.gguf_file),
                is_active: active_id == Some(custom.model_id.as_str()),
                is_loaded: loaded_id.as_deref() == Some(custom.model_id.as_str()),
                is_custom: true,
                gguf_file: custom.gguf_file.clone(),
            });
        }
    }

    Ok(result)
}

#[tauri::command]
pub async fn download_local_chat_model(
    app: tauri::AppHandle,
    model_id: String,
    auto_activate: Option<bool>,
) -> Result<bool, String> {
    DOWNLOAD_CANCELLED.store(false, Ordering::SeqCst);

    let cfg = config::load_config(&app)?;
    let info = resolve_model_info(&model_id, &cfg)
        .ok_or(format!("Unknown model: {}", model_id))?;

    let cache_dir = config::models_cache_dir(&app)?;
    let cache_dir_display = cache_dir.display().to_string();

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
    let hf_repo = info.hf_repo;
    let gguf_file = info.gguf_file;
    let tok_model_id = info.tok_model_id;

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
            let repo_id = if file_name.ends_with(".gguf") {
                &hf_repo
            } else {
                &tok_model_id
            };

            let max_retries = 3;
            let mut attempt = 0;
            loop {
                attempt += 1;
                let file_progress = TauriProgress {
                    app: app_clone.clone(),
                    model_id: model_id_clone.clone(),
                    file_name: String::new(),
                    total: 0,
                    current: 0,
                };
                match repo.download_with_progress(file_name, file_progress) {
                    Ok(_) => break,
                    Err(e) => {
                        let err_str = format!("{}", e);
                        if err_str.contains("404") || err_str.contains("Not Found") {
                            eprintln!(
                                "Warning: skipping '{}' for model '{}' (not found in repo)",
                                file_name, model_id_clone
                            );
                            break;
                        }
                        if attempt < max_retries {
                            eprintln!(
                                "Retry {}/{} for '{}': {}",
                                attempt, max_retries, file_name, err_str
                            );
                            std::thread::sleep(std::time::Duration::from_secs(2u64.pow(attempt as u32 - 1)));
                        } else {
                            let download_url = format!(
                                "https://huggingface.co/{}/resolve/main/{}",
                                repo_id, file_name
                            );
                            return Err(format!(
                                "Failed to download '{}' after {} retries: {}\n\nManual download:\n  URL: {}\n  Save to: {}/\n\nAfter downloading, place the file in the directory above and retry.",
                                file_name, max_retries, err_str, download_url, cache_dir_display
                            ));
                        }
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
    let cfg = config::load_config(&app)?;

    let info = resolve_model_info(&model_id, &cfg)
        .ok_or(format!("Unknown model: {}", model_id))?;

    for repo in &[info.hf_repo.as_str(), info.tok_model_id.as_str()] {
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
        return Err("Only built-in models can be hidden. Use remove_custom for custom models.".to_string());
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

#[tauri::command]
pub fn add_custom_local_chat_model(
    app: tauri::AppHandle,
    model: CustomLocalModelEntry,
) -> Result<(), String> {
    let mut cfg = config::load_config(&app)?;

    if cfg.custom_models.iter().any(|m| m.model_id == model.model_id) {
        return Err(format!("Custom model '{}' already exists", model.model_id));
    }
    if find_catalog_entry(&model.model_id).is_some() {
        return Err(format!("Model '{}' is already in the built-in catalog", model.model_id));
    }

    cfg.custom_models.push(model);
    config::save_config(&app, &cfg)?;
    Ok(())
}

#[tauri::command]
pub fn remove_custom_local_chat_model(
    app: tauri::AppHandle,
    model_id: String,
) -> Result<(), String> {
    let mut cfg = config::load_config(&app)?;
    let before = cfg.custom_models.len();
    cfg.custom_models.retain(|m| m.model_id != model_id);
    if cfg.custom_models.len() == before {
        return Err(format!("Custom model '{}' not found", model_id));
    }

    if cfg.active_model_id.as_deref() == Some(&model_id) {
        cfg.active_model_id = None;
    }

    config::save_config(&app, &cfg)?;
    Ok(())
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct HfModelSearchResult {
    pub model_id: String,
    pub author: String,
    pub downloads: u64,
    pub likes: u64,
    pub pipeline_tag: Option<String>,
    pub tags: Vec<String>,
    pub library_name: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct HfModelFileInfo {
    pub rfilename: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct HfModelInfo {
    pub model_id: String,
    pub siblings: Vec<HfModelFileInfo>,
    pub tags: Vec<String>,
    pub pipeline_tag: Option<String>,
    pub library_name: Option<String>,
    pub downloads: Option<u64>,
    pub likes: Option<u64>,
}

#[tauri::command]
pub fn search_hf_chat_models(
    query: Option<String>,
    limit: Option<u32>,
) -> Result<Vec<HfModelSearchResult>, String> {
    let limit = limit.unwrap_or(30).min(100);
    let url = if let Some(q) = &query {
        let trimmed = q.trim();
        if trimmed.is_empty() {
            format!(
                "{}/models?filter=gguf&sort=downloads&direction=-1&limit={}",
                HF_API_BASE, limit
            )
        } else {
            let encoded: String = trimmed
                .replace(' ', "+")
                .chars()
                .map(|c| if c.is_alphanumeric() || c == '+' || c == '-' || c == '_' {
                    c.to_string()
                } else {
                    format!("%{:02X}", c as u32)
                })
                .collect();
            format!(
                "{}/models?search={}&filter=gguf&sort=downloads&direction=-1&limit={}",
                HF_API_BASE, encoded, limit
            )
        }
    } else {
        format!(
            "{}/models?filter=gguf&sort=downloads&direction=-1&limit={}",
            HF_API_BASE, limit
        )
    };

    let mut resp = ureq::get(&url)
        .header("User-Agent", "Oasis/1.0 (local-llm-search)")
        .call()
        .map_err(|e| format!("HF API request failed: {}", e))?;

    let raw: Vec<serde_json::Value> = resp
        .body_mut()
        .read_json()
        .map_err(|e| format!("Failed to parse HF API response: {}", e))?;

    let results: Vec<HfModelSearchResult> = raw
        .iter()
        .filter_map(|v| {
            let id = v.get("id")?.as_str()?.to_string();
            let author = id.split('/').next().unwrap_or("unknown").to_string();
            let downloads = v.get("downloads").and_then(|d| d.as_u64()).unwrap_or(0);
            let likes = v.get("likes").and_then(|l| l.as_u64()).unwrap_or(0);
            let pipeline_tag = v.get("pipeline_tag").and_then(|t| t.as_str()).map(String::from);
            let tags = v
                .get("tags")
                .and_then(|t| t.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|t| t.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();
            let library_name = v
                .get("library_name")
                .and_then(|l| l.as_str())
                .map(String::from);
            let created_at = v
                .get("createdAt")
                .and_then(|c| c.as_str())
                .map(String::from);

            Some(HfModelSearchResult {
                model_id: id,
                author,
                downloads,
                likes,
                pipeline_tag,
                tags,
                library_name,
                created_at,
            })
        })
        .collect();

    Ok(results)
}

#[tauri::command]
pub fn get_hf_chat_model_info(model_id: String) -> Result<HfModelInfo, String> {
    let url = format!("{}/models/{}", HF_API_BASE, model_id);

    let mut resp = ureq::get(&url)
        .header("User-Agent", "Oasis/1.0 (local-llm-model-info)")
        .call()
        .map_err(|e| format!("HF API request failed for '{}': {}", model_id, e))?;

    let raw: serde_json::Value = resp
        .body_mut()
        .read_json()
        .map_err(|e| format!("Failed to parse HF model info: {}", e))?;

    let siblings = raw
        .get("siblings")
        .and_then(|s| s.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|f| {
                    f.get("rfilename")
                        .and_then(|r| r.as_str())
                        .map(|name| HfModelFileInfo {
                            rfilename: name.to_string(),
                        })
                })
                .collect()
        })
        .unwrap_or_default();

    let tags = raw
        .get("tags")
        .and_then(|t| t.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|t| t.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    Ok(HfModelInfo {
        model_id: raw
            .get("id")
            .and_then(|i| i.as_str())
            .unwrap_or(&model_id)
            .to_string(),
        siblings,
        tags,
        pipeline_tag: raw
            .get("pipeline_tag")
            .and_then(|t| t.as_str())
            .map(String::from),
        library_name: raw
            .get("library_name")
            .and_then(|l| l.as_str())
            .map(String::from),
        downloads: raw.get("downloads").and_then(|d| d.as_u64()),
        likes: raw.get("likes").and_then(|l| l.as_u64()),
    })
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GgufFileOption {
    pub filename: String,
    pub quant_label: String,
    pub estimated_size_mb: f64,
}

fn gguf_quant_label(filename: &str) -> String {
    let name = filename.to_uppercase();
    if name.contains("Q8_0") { return "Q8_0".to_string(); }
    if name.contains("Q6_K_L") { return "Q6_K_L".to_string(); }
    if name.contains("Q6_K_M") { return "Q6_K_M".to_string(); }
    if name.contains("Q6_K") { return "Q6_K".to_string(); }
    if name.contains("Q5_K_M") { return "Q5_K_M".to_string(); }
    if name.contains("Q5_K_S") { return "Q5_K_S".to_string(); }
    if name.contains("Q5_1") { return "Q5_1".to_string(); }
    if name.contains("Q5_0") { return "Q5_0".to_string(); }
    if name.contains("Q4_K_M") { return "Q4_K_M".to_string(); }
    if name.contains("Q4_K_S") { return "Q4_K_S".to_string(); }
    if name.contains("Q4_1") { return "Q4_1".to_string(); }
    if name.contains("Q4_0") { return "Q4_0".to_string(); }
    if name.contains("Q3_K_M") { return "Q3_K_M".to_string(); }
    if name.contains("Q3_K_S") { return "Q3_K_S".to_string(); }
    if name.contains("Q2_K") { return "Q2_K".to_string(); }
    if name.contains("IQ4_XS") { return "IQ4_XS".to_string(); }
    if name.contains("IQ3_M") { return "IQ3_M".to_string(); }
    if name.contains("IQ3_XS") { return "IQ3_XS".to_string(); }
    if name.contains("FP16") { return "FP16".to_string(); }
    if name.contains("BF16") { return "BF16".to_string(); }
    if name.contains("F16") { return "F16".to_string(); }
    "Other".to_string()
}

fn estimate_gguf_size_mb(filename: &str, model_id: &str) -> f64 {
    let combined = filename.to_lowercase() + " " + &model_id.to_lowercase();
    let is_quant_half = combined.contains("q4") || combined.contains("iq4") || combined.contains("q3") || combined.contains("q2");
    let is_quant_high = combined.contains("q8") || combined.contains("fp16") || combined.contains("bf16") || combined.contains("f16");

    let params = if combined.contains("135m") { 0.135 }
    else if combined.contains("360m") { 0.36 }
    else if combined.contains("0.5b") || combined.contains("500m") { 0.5 }
    else if combined.contains("1.7b") || combined.contains("1_7b") { 1.7 }
    else if combined.contains("1.8b") || combined.contains("1_8b") { 1.8 }
    else if combined.contains("2b") { 2.0 }
    else if combined.contains("3b") { 3.0 }
    else if combined.contains("7b") { 7.0 }
    else if combined.contains("8b") { 8.0 }
    else if combined.contains("9b") { 9.0 }
    else if combined.contains("11b") { 11.0 }
    else if combined.contains("13b") { 13.0 }
    else if combined.contains("14b") { 14.0 }
    else if combined.contains("32b") { 32.0 }
    else if combined.contains("70b") { 70.0 }
    else { 7.0 };

    let bytes_per_param = if is_quant_half { 0.5 }
    else if is_quant_high { 2.0 }
    else { 0.75 };

    params * bytes_per_param * 1000.0
}

#[tauri::command]
pub fn list_gguf_files(model_id: String) -> Result<Vec<GgufFileOption>, String> {
    let info = get_hf_chat_model_info(model_id)?;

    let gguf_files: Vec<GgufFileOption> = info
        .siblings
        .iter()
        .filter(|f| f.rfilename.ends_with(".gguf"))
        .map(|f| {
            let label = gguf_quant_label(&f.rfilename);
            let size = estimate_gguf_size_mb(&f.rfilename, &info.model_id);
            GgufFileOption {
                filename: f.rfilename.clone(),
                quant_label: label,
                estimated_size_mb: size,
            }
        })
        .filter(|g| g.quant_label != "Other")
        .collect();

    Ok(gguf_files)
}
