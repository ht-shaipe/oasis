use crate::catalog::find_catalog_entry;
use crate::config;
use mistralrs::{
    GgufModelBuilder, TextMessageRole, TextMessages,
    Response, ChatCompletionChunkResponse,
    ChunkChoice, Delta,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::Emitter;
use tokio::sync::Mutex as AsyncMutex;

static GLOBAL_MODEL: once_cell::sync::Lazy<AsyncMutex<Option<mistralrs::Model>>> =
    once_cell::sync::Lazy::new(|| AsyncMutex::new(None));
static ACTIVE_MODEL_ID: Mutex<Option<String>> = Mutex::new(None);

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ModelLoadState {
    pub model_id: String,
    pub status: String,
    pub message: Option<String>,
}

pub fn is_model_downloaded(app: &tauri::AppHandle, model_id: &str) -> bool {
    let cache_dir = match config::models_cache_dir(app) {
        Ok(d) => d,
        Err(_) => return false,
    };
    let catalog_entry = match find_catalog_entry(model_id) {
        Some(e) => e,
        None => return false,
    };
    let org_model = catalog_entry.hf_repo.replace('/', "--");
    let snapshot_dir = cache_dir
        .join(format!("models--{}", org_model))
        .join("snapshots");
    if !snapshot_dir.exists() {
        return false;
    }
    if let Ok(entries) = fs::read_dir(&snapshot_dir) {
        for dir_entry in entries.flatten() {
            if dir_entry.path().is_dir() {
                if dir_entry.path().join(&catalog_entry.gguf_file).exists() {
                    return true;
                }
            }
        }
    }
    false
}

fn find_gguf_path(app: &tauri::AppHandle, model_id: &str) -> Result<PathBuf, String> {
    let cache_dir = config::models_cache_dir(app)?;
    let catalog_entry = find_catalog_entry(model_id)
        .ok_or(format!("Unknown model: {}", model_id))?;
    let org_model = catalog_entry.hf_repo.replace('/', "--");
    let snapshot_dir = cache_dir
        .join(format!("models--{}", org_model))
        .join("snapshots");
    if let Ok(entries) = fs::read_dir(&snapshot_dir) {
        for entry_dir in entries.flatten() {
            if entry_dir.path().is_dir() {
                let gguf = entry_dir.path().join(&catalog_entry.gguf_file);
                if gguf.exists() {
                    return Ok(gguf);
                }
            }
        }
    }
    Err(format!("GGUF file not found for model '{}'. Please download it first.", model_id))
}

fn get_current_active_id() -> Option<String> {
    ACTIVE_MODEL_ID.lock().unwrap().clone()
}

fn set_active_id(id: Option<String>) {
    *ACTIVE_MODEL_ID.lock().unwrap() = id;
}

pub async fn load_model(
    app: &tauri::AppHandle,
    model_id: &str,
) -> Result<(), String> {
    let current_id = get_current_active_id();
    if current_id.as_deref() == Some(model_id) {
        let model_guard = GLOBAL_MODEL.lock().await;
        if model_guard.is_some() {
            return Ok(());
        }
    }

    let _ = app.emit("local-llm-load-progress", ModelLoadState {
        model_id: model_id.to_string(),
        status: "loading".to_string(),
        message: Some("Loading model into memory...".to_string()),
    });

    unload_model_internal().await;

    let catalog_entry = find_catalog_entry(model_id)
        .ok_or(format!("Unknown model: {}", model_id))?;

    let gguf_path = find_gguf_path(app, model_id)?;

    let parent_dir = gguf_path.parent()
        .ok_or("Invalid GGUF path")?
        .to_string_lossy()
        .to_string();
    let gguf_filename = gguf_path.file_name()
        .ok_or("Invalid GGUF filename")?
        .to_string_lossy()
        .to_string();

    let model = GgufModelBuilder::new(&parent_dir, vec![gguf_filename.as_str()])
        .with_tok_model_id(catalog_entry.tok_model_id)
        .with_logging()
        .build()
        .await
        .map_err(|e| format!("Failed to load model: {}", e))?;

    {
        let mut guard = GLOBAL_MODEL.lock().await;
        *guard = Some(model);
    }
    set_active_id(Some(model_id.to_string()));

    let _ = app.emit("local-llm-load-progress", ModelLoadState {
        model_id: model_id.to_string(),
        status: "ready".to_string(),
        message: Some("Model loaded and ready".to_string()),
    });

    Ok(())
}

async fn unload_model_internal() {
    {
        let mut guard = GLOBAL_MODEL.lock().await;
        *guard = None;
    }
    set_active_id(None);
}

pub async fn unload_model() -> Result<(), String> {
    unload_model_internal().await;
    Ok(())
}

pub fn get_loaded_model_id() -> Option<String> {
    get_current_active_id()
}

pub fn is_model_loaded() -> bool {
    get_current_active_id().is_some()
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessageForLocal {
    pub role: String,
    pub content: String,
}

pub fn chat_stream_blocking(
    messages: Vec<ChatMessageForLocal>,
    on_chunk: impl Fn(String, bool) + Send + 'static,
) -> Result<(), String> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("Runtime error: {}", e))?;

    let local = tokio::task::LocalSet::new();

    local.block_on(&rt, async {
        let model_guard = GLOBAL_MODEL.lock().await;
        let model = match model_guard.as_ref() {
            Some(m) => m,
            None => {
                on_chunk("No local model loaded.".to_string(), true);
                return;
            }
        };

        let mut text_messages = TextMessages::new();
        for msg in &messages {
            let role = match msg.role.as_str() {
                "system" => TextMessageRole::System,
                "assistant" => TextMessageRole::Assistant,
                _ => TextMessageRole::User,
            };
            text_messages = text_messages.add_message(role, &msg.content);
        }

        let mut stream = match model.stream_chat_request(text_messages).await {
            Ok(s) => s,
            Err(e) => {
                on_chunk(format!("Error: {}", e), true);
                return;
            }
        };

        use futures::StreamExt as _;

        while let Some(chunk) = stream.next().await {
            match chunk {
                Response::Chunk(ChatCompletionChunkResponse { choices, .. }) => {
                    if let Some(ChunkChoice {
                        delta: Delta { content: Some(content), .. },
                        ..
                    }) = choices.first()
                    {
                        on_chunk(content.clone(), false);
                    }
                }
                Response::ModelError(msg, _) => {
                    on_chunk(format!("Model error: {}", msg), true);
                    return;
                }
                Response::Done(_) => {
                    on_chunk(String::new(), true);
                    return;
                }
                _ => {}
            }
        }

        on_chunk(String::new(), true);
    });

    Ok(())
}
