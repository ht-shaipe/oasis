use fastembed::{EmbeddingModel, TextEmbedding, TextInitOptions};
use hf_hub::api::sync::ApiBuilder;
use hf_hub::api::Progress;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{Emitter, Manager};

const HF_API_BASE: &str = "https://huggingface.co/api";

const EMBED_CONFIG_FILE: &str = "embed_config.json";
const EMBED_MODELS_DIR: &str = "embed_models";

static DOWNLOAD_CANCELLED: AtomicBool = AtomicBool::new(false);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum EmbedMode {
    Local,
    Remote,
}

impl Default for EmbedMode {
    fn default() -> Self {
        EmbedMode::Local
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct EmbedModelConfig {
    #[serde(default)]
    pub mode: EmbedMode,
    #[serde(default)]
    pub active_local_model_id: Option<String>,
    #[serde(default)]
    pub active_remote_model_id: Option<String>,
    #[serde(default)]
    pub custom_models: Vec<CustomModelEntry>,
    #[serde(default)]
    pub auto_activate: bool,
    #[serde(default)]
    pub hidden_builtin_ids: Vec<String>,
}

impl EmbedModelConfig {
    pub fn migrate_from_old(active_model_id: Option<String>) -> Self {
        EmbedModelConfig {
            mode: EmbedMode::Local,
            active_local_model_id: active_model_id.clone(),
            active_remote_model_id: None,
            custom_models: Vec::new(),
            auto_activate: true,
            hidden_builtin_ids: Vec::new(),
        }
    }

    pub fn effective_active_model_id(&self) -> Option<&String> {
        match self.mode {
            EmbedMode::Local => self.active_local_model_id.as_ref(),
            EmbedMode::Remote => self.active_remote_model_id.as_ref(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CustomModelEntry {
    pub model_id: String,
    pub name: String,
    pub dimensions: usize,
    pub quantized: bool,
    pub size_mb: f64,
    pub license: String,
    pub description: String,
    pub onnx_file: String,
    pub additional_files: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LocalEmbeddingModel {
    pub id: String,
    pub name: String,
    pub dimensions: usize,
    pub quantized: bool,
    pub size_mb: f64,
    pub license: String,
    pub description: String,
    pub downloaded: bool,
    pub is_custom: bool,
    pub onnx_file: String,
    pub additional_files: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DownloadProgressPayload {
    pub model_id: String,
    pub file_name: String,
    pub current: usize,
    pub total: usize,
    pub percentage: u32,
    pub status: String,
}

struct TauriProgress {
    app: tauri::AppHandle,
    model_id: String,
    file_name: String,
    total: usize,
    current: usize,
}

impl Progress for TauriProgress {
    fn init(&mut self, size: usize, filename: &str) {
        self.total = size;
        self.file_name = filename.to_string();
        self.current = 0;
        let _ = self.app.emit(
            "embed-download-progress",
            DownloadProgressPayload {
                model_id: self.model_id.clone(),
                file_name: filename.to_string(),
                current: 0,
                total: size,
                percentage: 0,
                status: "downloading".to_string(),
            },
        );
    }

    fn update(&mut self, size: usize) {
        self.current = size;
        let pct = if self.total > 0 {
            (size as u64 * 100 / self.total as u64) as u32
        } else {
            0
        };
        let _ = self.app.emit(
            "embed-download-progress",
            DownloadProgressPayload {
                model_id: self.model_id.clone(),
                file_name: self.file_name.clone(),
                current: size,
                total: self.total,
                percentage: pct,
                status: "downloading".to_string(),
            },
        );
    }

    fn finish(&mut self) {
        let _ = self.app.emit(
            "embed-download-progress",
            DownloadProgressPayload {
                model_id: self.model_id.clone(),
                file_name: self.file_name.clone(),
                current: self.total,
                total: self.total,
                percentage: 100,
                status: "file_done".to_string(),
            },
        );
    }
}

struct ModelCatalogEntry {
    key: EmbeddingModel,
    id: &'static str,
    name: &'static str,
    dimensions: usize,
    quantized: bool,
    size_mb: f64,
    license: &'static str,
    description: &'static str,
    onnx_file: &'static str,
    additional_files: Vec<&'static str>,
}

fn model_catalog() -> Vec<ModelCatalogEntry> {
    vec![
        ModelCatalogEntry {
            key: EmbeddingModel::BGESmallENV15Q,
            id: "BAAI/bge-small-en-v1.5Q",
            name: "BGE Small EN v1.5 (Quantized)",
            dimensions: 384,
            quantized: true,
            size_mb: 33.0,
            license: "MIT",
            description: "Best 384-dim quality, quantized for small footprint and fast inference",
            onnx_file: "model_optimized.onnx",
            additional_files: Vec::new(),
        },
        ModelCatalogEntry {
            key: EmbeddingModel::BGESmallENV15,
            id: "BAAI/bge-small-en-v1.5",
            name: "BGE Small EN v1.5",
            dimensions: 384,
            quantized: false,
            size_mb: 130.0,
            license: "MIT",
            description: "Best 384-dim quality, full precision",
            onnx_file: "onnx/model.onnx",
            additional_files: Vec::new(),
        },
        ModelCatalogEntry {
            key: EmbeddingModel::AllMiniLML6V2Q,
            id: "sentence-transformers/all-MiniLM-L6-v2Q",
            name: "MiniLM L6 v2 (Quantized)",
            dimensions: 384,
            quantized: true,
            size_mb: 23.0,
            license: "Apache-2.0",
            description: "Most popular embedding model, smallest quantized size",
            onnx_file: "onnx/model_quantized.onnx",
            additional_files: Vec::new(),
        },
        ModelCatalogEntry {
            key: EmbeddingModel::AllMiniLML6V2,
            id: "sentence-transformers/all-MiniLM-L6-v2",
            name: "MiniLM L6 v2",
            dimensions: 384,
            quantized: false,
            size_mb: 90.0,
            license: "Apache-2.0",
            description: "Most popular embedding model, full precision",
            onnx_file: "model.onnx",
            additional_files: Vec::new(),
        },
        ModelCatalogEntry {
            key: EmbeddingModel::NomicEmbedTextV15Q,
            id: "nomic-ai/nomic-embed-text-v1.5Q",
            name: "Nomic Embed Text v1.5 (Quantized)",
            dimensions: 768,
            quantized: true,
            size_mb: 70.0,
            license: "Apache-2.0",
            description: "768-dim with 8192 token context, Matryoshka dimension truncation",
            onnx_file: "onnx/model_quantized.onnx",
            additional_files: Vec::new(),
        },
        ModelCatalogEntry {
            key: EmbeddingModel::BGEBaseENV15Q,
            id: "BAAI/bge-base-en-v1.5Q",
            name: "BGE Base EN v1.5 (Quantized)",
            dimensions: 768,
            quantized: true,
            size_mb: 108.0,
            license: "MIT",
            description: "Best 768-dim quality/size ratio, quantized",
            onnx_file: "model_optimized.onnx",
            additional_files: Vec::new(),
        },
        ModelCatalogEntry {
            key: EmbeddingModel::BGEBaseENV15,
            id: "BAAI/bge-base-en-v1.5",
            name: "BGE Base EN v1.5",
            dimensions: 768,
            quantized: false,
            size_mb: 430.0,
            license: "MIT",
            description: "768-dim, best English base model quality",
            onnx_file: "onnx/model.onnx",
            additional_files: Vec::new(),
        },
        ModelCatalogEntry {
            key: EmbeddingModel::GTEBaseENV15Q,
            id: "Alibaba-NLP/gte-base-en-v1.5Q",
            name: "GTE Base EN v1.5 (Quantized)",
            dimensions: 768,
            quantized: true,
            size_mb: 110.0,
            license: "Apache-2.0",
            description: "768-dim from Alibaba, quantized for efficiency",
            onnx_file: "onnx/model_quantized.onnx",
            additional_files: Vec::new(),
        },
        ModelCatalogEntry {
            key: EmbeddingModel::BGESmallZHV15,
            id: "BAAI/bge-small-zh-v1.5",
            name: "BGE Small ZH v1.5",
            dimensions: 512,
            quantized: false,
            size_mb: 90.0,
            license: "MIT",
            description: "512-dim Chinese model, best for Chinese text",
            onnx_file: "onnx/model.onnx",
            additional_files: Vec::new(),
        },
        ModelCatalogEntry {
            key: EmbeddingModel::ParaphraseMLMiniLML12V2Q,
            id: "sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2Q",
            name: "Paraphrase ML MiniLM (Quantized)",
            dimensions: 384,
            quantized: true,
            size_mb: 45.0,
            license: "Apache-2.0",
            description: "384-dim multilingual, quantized for 50+ languages",
            onnx_file: "model_optimized.onnx",
            additional_files: Vec::new(),
        },
        ModelCatalogEntry {
            key: EmbeddingModel::SnowflakeArcticEmbedXSQ,
            id: "snowflake/snowflake-arctic-embed-xsQ",
            name: "Snowflake Arctic Embed XS (Quantized)",
            dimensions: 384,
            quantized: true,
            size_mb: 22.0,
            license: "Apache-2.0",
            description: "384-dim from Snowflake, extra-small quantized",
            onnx_file: "onnx/model_quantized.onnx",
            additional_files: Vec::new(),
        },
        ModelCatalogEntry {
            key: EmbeddingModel::MxbaiEmbedLargeV1Q,
            id: "mixedbread-ai/mxbai-embed-large-v1Q",
            name: "Mxbai Embed Large v1 (Quantized)",
            dimensions: 1024,
            quantized: true,
            size_mb: 130.0,
            license: "Apache-2.0",
            description: "1024-dim from MixedBread, quantized for high quality",
            onnx_file: "onnx/model_quantized.onnx",
            additional_files: Vec::new(),
        },
    ]
}

fn app_data_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

fn models_cache_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let dir = app_data_dir(app)?.join(EMBED_MODELS_DIR);
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

fn config_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    Ok(app_data_dir(app)?.join(EMBED_CONFIG_FILE))
}

pub fn load_config(app: &tauri::AppHandle) -> Result<EmbedModelConfig, String> {
    let path = config_path(app)?;
    if path.exists() {
        let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let value: serde_json::Value =
            serde_json::from_str(&content).map_err(|e| e.to_string())?;

        if value.get("mode").is_some() {
            serde_json::from_value(value).map_err(|e| e.to_string())
        } else {
            let old_active = value
                .get("activeModelId")
                .and_then(|v| v.as_str())
                .map(String::from);
            Ok(EmbedModelConfig::migrate_from_old(old_active))
        }
    } else {
        Ok(EmbedModelConfig::default())
    }
}

pub fn save_config(app: &tauri::AppHandle, config: &EmbedModelConfig) -> Result<(), String> {
    let path = config_path(app)?;
    let content = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    fs::write(&path, content).map_err(|e| e.to_string())
}

fn is_model_downloaded(cache_dir: &PathBuf, model_id: &str, config: &EmbedModelConfig) -> bool {
    let possible_repo_names: Vec<String> = {
        let mut names = vec![model_id.to_string()];
        if let Some((model_code, _, _)) = get_model_code_and_files(model_id, config) {
            if model_code != model_id {
                names.push(model_code);
            }
        }
        names
    };
    for repo_name in &possible_repo_names {
        let org_model = repo_name.replace('/', "--");
        let snapshot_dir = cache_dir
            .join(format!("models--{}", org_model))
            .join("snapshots");
        if !snapshot_dir.exists() {
            continue;
        }
        if let Ok(entries) = fs::read_dir(&snapshot_dir) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    let p = entry.path();
                    if p.join("onnx").join("model.onnx").exists()
                        || p.join("onnx").join("model_optimized.onnx").exists()
                        || p.join("onnx").join("model_quantized.onnx").exists()
                        || p.join("model.onnx").exists()
                        || p.join("model_optimized.onnx").exists()
                        || p.join("onnx").join("model_q4.onnx").exists()
                    {
                        return true;
                    }
                }
            }
        }
    }
    false
}

pub fn find_catalog_key(model_id: &str) -> Option<EmbeddingModel> {
    model_catalog()
        .iter()
        .find(|e| e.id == model_id)
        .map(|e| e.key.clone())
}

fn find_catalog_entry_owned(model_id: &str) -> Option<ModelCatalogEntry> {
    model_catalog()
        .into_iter()
        .find(|e| e.id == model_id)
}

fn get_model_code_and_files(model_id: &str, config: &EmbedModelConfig) -> Option<(String, String, Vec<String>)> {
    if let Some(entry) = find_catalog_entry_owned(model_id) {
        let model_code = match entry.key {
            EmbeddingModel::BGESmallENV15Q => "Qdrant/bge-small-en-v1.5-onnx-Q".to_string(),
            EmbeddingModel::BGESmallENV15 => "Xenova/bge-small-en-v1.5".to_string(),
            EmbeddingModel::AllMiniLML6V2Q => "Xenova/all-MiniLM-L6-v2".to_string(),
            EmbeddingModel::AllMiniLML6V2 => "Qdrant/all-MiniLM-L6-v2-onnx".to_string(),
            EmbeddingModel::NomicEmbedTextV15Q => "nomic-ai/nomic-embed-text-v1.5".to_string(),
            EmbeddingModel::BGEBaseENV15Q => "Qdrant/bge-base-en-v1.5-onnx-Q".to_string(),
            EmbeddingModel::BGEBaseENV15 => "Xenova/bge-base-en-v1.5".to_string(),
            EmbeddingModel::GTEBaseENV15Q => "Alibaba-NLP/gte-base-en-v1.5".to_string(),
            EmbeddingModel::BGESmallZHV15 => "Xenova/bge-small-zh-v1.5".to_string(),
            EmbeddingModel::ParaphraseMLMiniLML12V2Q => {
                "Qdrant/paraphrase-multilingual-MiniLM-L12-v2-onnx-Q".to_string()
            }
            EmbeddingModel::SnowflakeArcticEmbedXSQ => {
                "snowflake/snowflake-arctic-embed-xs".to_string()
            }
            EmbeddingModel::MxbaiEmbedLargeV1Q => {
                "mixedbread-ai/mxbai-embed-large-v1".to_string()
            }
            _ => return None,
        };
        Some((
            model_code,
            entry.onnx_file.to_string(),
            entry
                .additional_files
                .iter()
                .map(|s| s.to_string())
                .collect(),
        ))
    } else if let Some(custom) = config.custom_models.iter().find(|m| m.model_id == model_id) {
        Some((
            custom.model_id.clone(),
            custom.onnx_file.clone(),
            custom.additional_files.clone(),
        ))
    } else {
        None
    }
}

const TOKENIZER_FILES: [&str; 4] = [
    "tokenizer.json",
    "config.json",
    "special_tokens_map.json",
    "tokenizer_config.json",
];

#[tauri::command]
pub fn list_available_embedding_models(
    app: tauri::AppHandle,
) -> Result<Vec<LocalEmbeddingModel>, String> {
    let cache_dir = models_cache_dir(&app)?;
    let config = load_config(&app)?;
    let catalog = model_catalog();

    let hidden_set: std::collections::HashSet<&str> = config
        .hidden_builtin_ids
        .iter()
        .map(|s| s.as_str())
        .collect();

    let mut result: Vec<LocalEmbeddingModel> = catalog
        .iter()
        .filter(|entry| !hidden_set.contains(entry.id))
        .map(|entry| LocalEmbeddingModel {
            id: entry.id.to_string(),
            name: entry.name.to_string(),
            dimensions: entry.dimensions,
            quantized: entry.quantized,
            size_mb: entry.size_mb,
            license: entry.license.to_string(),
            description: entry.description.to_string(),
            downloaded: is_model_downloaded(&cache_dir, entry.id, &config),
            is_custom: false,
            onnx_file: entry.onnx_file.to_string(),
            additional_files: entry
                .additional_files
                .iter()
                .map(|s| s.to_string())
                .collect(),
        })
        .collect();

    for custom in &config.custom_models {
        if !result.iter().any(|m| m.id == custom.model_id) {
            result.push(LocalEmbeddingModel {
                id: custom.model_id.clone(),
                name: custom.name.clone(),
                dimensions: custom.dimensions,
                quantized: custom.quantized,
                size_mb: custom.size_mb,
                license: custom.license.clone(),
                description: custom.description.clone(),
                downloaded: is_model_downloaded(&cache_dir, &custom.model_id, &config),
                is_custom: true,
                onnx_file: custom.onnx_file.clone(),
                additional_files: custom.additional_files.clone(),
            });
        }
    }

    Ok(result)
}

#[tauri::command]
pub async fn download_embedding_model(
    app: tauri::AppHandle,
    model_id: String,
    auto_activate: Option<bool>,
) -> Result<bool, String> {
    DOWNLOAD_CANCELLED.store(false, Ordering::SeqCst);

    let config = load_config(&app)?;
    let (model_code, onnx_file, additional_files) =
        get_model_code_and_files(&model_id, &config)
            .ok_or(format!("Unknown model: {}", model_id))?;

    let cache_dir = models_cache_dir(&app)?;
    let cache_dir_display = cache_dir.display().to_string();

    let _ = app.emit(
        "embed-download-progress",
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
    let model_code_clone = model_code.clone();
    let model_code_for_err = model_code.clone();
    let onnx_file_clone = onnx_file.clone();
    let additional_files_clone = additional_files.clone();

    tauri::async_runtime::spawn_blocking(move || {
        let api = ApiBuilder::new()
            .with_cache_dir(cache_dir)
            .build()
            .map_err(|e| format!("Failed to create HF API: {}", e))?;

        let repo = api.model(model_code_clone);

        let mut all_files: Vec<String> = TOKENIZER_FILES
            .iter()
            .map(|s| s.to_string())
            .collect();
        all_files.push(onnx_file_clone);
        all_files.extend(additional_files_clone);

        let total_files = all_files.len();
        for (idx, file_name) in all_files.iter().enumerate() {
            if DOWNLOAD_CANCELLED.load(Ordering::SeqCst) {
                let _ = app_clone.emit(
                    "embed-download-progress",
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
                "embed-download-progress",
                DownloadProgressPayload {
                    model_id: model_id_clone.clone(),
                    file_name: format!("{} ({}/{})", file_name, idx + 1, total_files),
                    current: 0,
                    total: total_files,
                    percentage: overall_pct,
                    status: "downloading".to_string(),
                },
            );

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
                                model_code_for_err, file_name
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
            "embed-download-progress",
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
            let mut cfg = load_config(&app_clone)?;
            cfg.active_local_model_id = Some(model_id_clone.clone());
            save_config(&app_clone, &cfg)?;
            return Ok(true);
        }

        Ok(false)
    })
    .await
    .map_err(|e| format!("Download task error: {}", e))?
}

#[tauri::command]
pub fn cancel_embedding_download() -> Result<(), String> {
    DOWNLOAD_CANCELLED.store(true, Ordering::SeqCst);
    Ok(())
}

#[tauri::command]
pub fn delete_embedding_model(
    app: tauri::AppHandle,
    model_id: String,
) -> Result<(), String> {
    let cache_dir = models_cache_dir(&app)?;
    let config = load_config(&app)?;

    let possible_repo_names: Vec<String> = {
        let mut names = vec![model_id.to_string()];
        if let Some((model_code, _, _)) = get_model_code_and_files(&model_id, &config) {
            if model_code != model_id {
                names.push(model_code);
            }
        }
        names
    };

    for repo_name in &possible_repo_names {
        let org_model = repo_name.replace('/', "--");
        let model_dir = cache_dir.join(format!("models--{}", org_model));
        if model_dir.exists() {
            fs::remove_dir_all(&model_dir).map_err(|e| e.to_string())?;
        }
    }

    if config.active_local_model_id.as_deref() == Some(&model_id) {
        let mut config = config;
        config.active_local_model_id = None;
        save_config(&app, &config)?;
    }

    Ok(())
}

#[tauri::command]
pub fn get_embedding_config(app: tauri::AppHandle) -> Result<EmbedModelConfig, String> {
    load_config(&app)
}

#[tauri::command]
pub fn set_active_embedding_model(
    app: tauri::AppHandle,
    model_id: String,
) -> Result<(), String> {
    let cache_dir = models_cache_dir(&app)?;
    let config = load_config(&app)?;
    if !is_model_downloaded(&cache_dir, &model_id, &config) {
        return Err(format!(
            "Model '{}' is not downloaded. Please download it first.",
            model_id
        ));
    }

    let mut config = config;
    config.active_local_model_id = Some(model_id);
    save_config(&app, &config)?;

    Ok(())
}

#[tauri::command]
pub fn set_embed_mode(app: tauri::AppHandle, mode: String) -> Result<(), String> {
    let mut config = load_config(&app)?;
    config.mode = match mode.as_str() {
        "local" => EmbedMode::Local,
        "remote" => EmbedMode::Remote,
        _ => return Err(format!("Invalid mode: {}", mode)),
    };
    save_config(&app, &config)?;
    Ok(())
}

#[tauri::command]
pub fn set_auto_activate(app: tauri::AppHandle, auto_activate: bool) -> Result<(), String> {
    let mut config = load_config(&app)?;
    config.auto_activate = auto_activate;
    save_config(&app, &config)?;
    Ok(())
}

#[tauri::command]
pub fn add_custom_embedding_model(
    app: tauri::AppHandle,
    model: CustomModelEntry,
) -> Result<(), String> {
    let mut config = load_config(&app)?;

    if config
        .custom_models
        .iter()
        .any(|m| m.model_id == model.model_id)
    {
        return Err(format!(
            "Custom model '{}' already exists",
            model.model_id
        ));
    }
    if find_catalog_entry_owned(&model.model_id).is_some() {
        return Err(format!(
            "Model '{}' is already in the built-in catalog",
            model.model_id
        ));
    }

    config.custom_models.push(model);
    save_config(&app, &config)?;
    Ok(())
}

#[tauri::command]
pub fn remove_custom_embedding_model(
    app: tauri::AppHandle,
    model_id: String,
) -> Result<(), String> {
    let mut config = load_config(&app)?;
    let before = config.custom_models.len();
    config.custom_models.retain(|m| m.model_id != model_id);
    if config.custom_models.len() == before {
        return Err(format!("Custom model '{}' not found", model_id));
    }

    if config.active_local_model_id.as_deref() == Some(&model_id) {
        config.active_local_model_id = None;
    }

    save_config(&app, &config)?;
    Ok(())
}

#[tauri::command]
pub fn hide_embedding_model(
    app: tauri::AppHandle,
    model_id: String,
) -> Result<(), String> {
    if find_catalog_entry_owned(&model_id).is_none() {
        return Err(format!(
            "Only built-in models can be hidden. Use remove_custom for custom models."
        ));
    }
    let mut config = load_config(&app)?;
    if !config.hidden_builtin_ids.contains(&model_id) {
        config.hidden_builtin_ids.push(model_id.clone());
    }
    if config.active_local_model_id.as_deref() == Some(&model_id) {
        config.active_local_model_id = None;
    }
    save_config(&app, &config)?;
    Ok(())
}

#[tauri::command]
pub fn restore_embedding_model(
    app: tauri::AppHandle,
    model_id: String,
) -> Result<(), String> {
    let mut config = load_config(&app)?;
    config.hidden_builtin_ids.retain(|id| id != &model_id);
    save_config(&app, &config)?;
    Ok(())
}

#[tauri::command]
pub fn list_hidden_embedding_models(
    app: tauri::AppHandle,
) -> Result<Vec<LocalEmbeddingModel>, String> {
    let cache_dir = models_cache_dir(&app)?;
    let config = load_config(&app)?;
    let catalog = model_catalog();
    let hidden_set: std::collections::HashSet<&str> = config
        .hidden_builtin_ids
        .iter()
        .map(|s| s.as_str())
        .collect();

    let result: Vec<LocalEmbeddingModel> = catalog
        .iter()
        .filter(|entry| hidden_set.contains(entry.id))
        .map(|entry| LocalEmbeddingModel {
            id: entry.id.to_string(),
            name: entry.name.to_string(),
            dimensions: entry.dimensions,
            quantized: entry.quantized,
            size_mb: entry.size_mb,
            license: entry.license.to_string(),
            description: entry.description.to_string(),
            downloaded: is_model_downloaded(&cache_dir, entry.id, &config),
            is_custom: false,
            onnx_file: entry.onnx_file.to_string(),
            additional_files: entry
                .additional_files
                .iter()
                .map(|s| s.to_string())
                .collect(),
        })
        .collect();

    Ok(result)
}

#[tauri::command]
pub async fn generate_local_embedding(
    app: tauri::AppHandle,
    texts: Vec<String>,
) -> Result<Vec<Vec<f32>>, String> {
    let config = load_config(&app)?;
    let model_id = config
        .active_local_model_id
        .ok_or("No active local embedding model. Please set one in Embedding settings.")?;

    let key = find_catalog_key(&model_id)
        .ok_or(format!("Unknown model: {}", model_id))?;

    let cache_dir = models_cache_dir(&app)?;

    let mut model = TextEmbedding::try_new(
        TextInitOptions::new(key)
            .with_cache_dir(cache_dir)
            .with_show_download_progress(false),
    )
    .map_err(|e| format!("Failed to load model '{}': {}", model_id, e))?;

    let embeddings = model
        .embed(texts, None)
        .map_err(|e| format!("Embedding inference failed: {}", e))?;

    Ok(embeddings)
}

#[tauri::command]
pub fn get_local_embedding_dim(app: tauri::AppHandle) -> Result<Option<usize>, String> {
    let config = load_config(&app)?;
    if let Some(model_id) = &config.active_local_model_id {
        if let Some(entry) = find_catalog_entry_owned(model_id) {
            return Ok(Some(entry.dimensions));
        }
        if let Some(custom) = config
            .custom_models
            .iter()
            .find(|m| &m.model_id == model_id)
        {
            return Ok(Some(custom.dimensions));
        }
    }
    Ok(None)
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
pub fn search_hf_embedding_models(
    query: Option<String>,
    limit: Option<u32>,
) -> Result<Vec<HfModelSearchResult>, String> {
    let limit = limit.unwrap_or(30).min(100);
    let url = if let Some(q) = &query {
        let trimmed = q.trim();
        if trimmed.is_empty() {
            format!(
                "{}/models?filter=sentence-similarity&sort=downloads&direction=-1&limit={}",
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
                "{}/models?search={}&filter=sentence-similarity&sort=downloads&direction=-1&limit={}",
                HF_API_BASE, encoded, limit
            )
        }
    } else {
        format!(
            "{}/models?filter=sentence-similarity&sort=downloads&direction=-1&limit={}",
            HF_API_BASE, limit
        )
    };

    let mut resp = ureq::get(&url)
        .header("User-Agent", "Oasis/1.0 (embedding-model-search)")
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
pub fn get_hf_model_info(model_id: String) -> Result<HfModelInfo, String> {
    let url = format!("{}/models/{}", HF_API_BASE, model_id);

    let mut resp = ureq::get(&url)
        .header("User-Agent", "Oasis/1.0 (embedding-model-info)")
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
