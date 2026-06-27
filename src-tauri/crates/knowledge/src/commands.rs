use crate::db;
use crate::indexer;
use crate::search;
use ai_llm_kit::LlmService;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::AppHandle;
use tauri::Manager;
use tube::value;

fn db_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

fn get_conn(app: &AppHandle) -> Result<Connection, String> {
    let dir = db_path(app)?;
    db::init_db(&dir).map_err(|e| e.to_string())
}

fn workspace_dir(app: &AppHandle) -> Result<String, String> {
    let config_path = db_path(app)?.join("workspace.json");
    if config_path.exists() {
        let content = fs::read_to_string(&config_path).map_err(|e| e.to_string())?;
        let config: serde_json::Value =
            serde_json::from_str(&content).map_err(|e| e.to_string())?;
        let dir = config
            .get("workspace_dir")
            .and_then(|v| v.as_str())
            .unwrap_or("~/.oasis");
        let expanded = shellexpand::tilde(dir).to_string();
        Ok(expanded)
    } else {
        let home = dirs_next::home_dir().unwrap_or_default();
        let default = home.join(".oasis");
        Ok(default.to_string_lossy().to_string())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeStatusResponse {
    pub workspace_dir: String,
    pub total_files: i64,
    pub total_chunks: i64,
    pub embedded_chunks: i64,
    pub last_index_time: Option<String>,
    pub is_indexing: bool,
    pub embedding_model: Option<String>,
    pub embedding_dim: Option<i32>,
}

#[tauri::command]
pub fn get_knowledge_status(app: AppHandle) -> Result<KnowledgeStatusResponse, String> {
    let conn = get_conn(&app)?;
    let ws = workspace_dir(&app)?;
    let status = indexer::get_status(&conn, &ws);
    Ok(KnowledgeStatusResponse {
        workspace_dir: status.workspace_dir,
        total_files: status.total_files,
        total_chunks: status.total_chunks,
        embedded_chunks: status.embedded_chunks,
        last_index_time: status.last_index_time,
        is_indexing: status.is_indexing,
        embedding_model: status.embedding_model,
        embedding_dim: status.embedding_dim,
    })
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct IndexResultResponse {
    pub indexed_files: i32,
    pub skipped_files: i32,
    pub deleted_files: i32,
    pub total_chunks: i32,
    pub elapsed_secs: f64,
}

#[tauri::command]
pub async fn start_indexing(
    app: AppHandle,
    embedding_model_id: String,
) -> Result<IndexResultResponse, String> {
    let ws = workspace_dir(&app)?;
    let ws_path = PathBuf::from(&ws);
    if !ws_path.exists() {
        return Err(format!("Workspace directory does not exist: {}", ws));
    }

    let app_for_index = app.clone();
    let index_result = tauri::async_runtime::spawn_blocking(move || {
        let mut conn = get_conn(&app_for_index)?;
        let cancel = std::sync::atomic::AtomicBool::new(true);
        indexer::run_index(&mut conn, &ws_path, &cancel)
    })
    .await
    .map_err(|e| format!("Index task error: {}", e))??;

    let embedding_config = get_embedding_model_config(&app, &embedding_model_id)?;

    let app_for_embed = app.clone();
    let _embed_result = tauri::async_runtime::spawn_blocking(move || {
        let mut conn = get_conn(&app_for_embed)?;
        indexer::generate_embeddings_blocking(
            &mut conn,
            &embedding_model_id,
            &embedding_config.base_url,
            &embedding_config.api_key,
            20,
        )
    })
    .await
    .map_err(|e| format!("Embed task error: {}", e))??;

    Ok(IndexResultResponse {
        indexed_files: index_result.indexed_files,
        skipped_files: index_result.skipped_files,
        deleted_files: index_result.deleted_files,
        total_chunks: index_result.total_chunks,
        elapsed_secs: index_result.elapsed_secs,
    })
}

#[tauri::command]
pub fn stop_indexing() -> Result<(), String> {
    indexer::stop_indexing();
    Ok(())
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SearchResultResponse {
    pub file_path: String,
    pub rel_path: String,
    pub chunk_content: String,
    pub chunk_index: i32,
    pub score: f64,
}

#[tauri::command]
pub async fn semantic_search(
    app: AppHandle,
    query: String,
    top_k: Option<u32>,
) -> Result<Vec<SearchResultResponse>, String> {
    let embedding_model_id = {
        let conn = get_conn(&app)?;
        db::get_meta(&conn, "embedding_model")
            .ok()
            .flatten()
            .ok_or("No embedding model configured. Run start_indexing first.")?
    };

    let config = get_embedding_model_config(&app, &embedding_model_id)?;

    let query_embedding = {
        let model_id = embedding_model_id.clone();
        let base_url = config.base_url.clone();
        let api_key = config.api_key.clone();
        let query = query.clone();
        tauri::async_runtime::spawn_blocking(move || {
            generate_query_embedding_blocking(&model_id, &base_url, &api_key, &query)
        })
        .await
        .map_err(|e| format!("Query embedding error: {}", e))??
    };

    let conn = get_conn(&app)?;
    let top_k = top_k.unwrap_or(5) as usize;
    let results = search::search_by_query_embedding(&conn, &query_embedding, top_k)?;

    Ok(results
        .into_iter()
        .map(|r| SearchResultResponse {
            file_path: r.file_path,
            rel_path: r.rel_path,
            chunk_content: r.chunk_content,
            chunk_index: r.chunk_index,
            score: r.score,
        })
        .collect())
}

#[tauri::command]
pub fn delete_knowledge_index(app: AppHandle) -> Result<(), String> {
    let conn = get_conn(&app)?;
    db::clear_all(&conn).map_err(|e| e.to_string())
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct IndexedFileInfo {
    pub id: String,
    pub path: String,
    pub rel_path: String,
    pub ext: Option<String>,
    pub size: i64,
    pub chunk_count: i32,
    pub indexed_at: String,
}

#[tauri::command]
pub fn get_indexed_files(app: AppHandle) -> Result<Vec<IndexedFileInfo>, String> {
    let conn = get_conn(&app)?;
    let files = db::get_all_files(&conn).map_err(|e| e.to_string())?;
    Ok(files
        .into_iter()
        .map(|f| IndexedFileInfo {
            id: f.id,
            path: f.path,
            rel_path: f.rel_path,
            ext: f.ext,
            size: f.size,
            chunk_count: f.chunk_count,
            indexed_at: f.indexed_at,
        })
        .collect())
}

#[tauri::command]
pub fn remove_indexed_file(app: AppHandle, path: String) -> Result<(), String> {
    let conn = get_conn(&app)?;
    db::delete_file_by_path(&conn, &path).map_err(|e| e.to_string())
}

struct EmbeddingModelConfig {
    base_url: String,
    api_key: String,
}

fn get_embedding_model_config(
    app: &AppHandle,
    model_id: &str,
) -> Result<EmbeddingModelConfig, String> {
    let config_path = db_path(app)?.join("llm_config.json");
    if !config_path.exists() {
        return Err("LLM config not found. Please configure a model first.".to_string());
    }

    let content = fs::read_to_string(&config_path).map_err(|e| e.to_string())?;
    let config: serde_json::Value = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    let models = config
        .get("models")
        .and_then(|m| m.as_array())
        .ok_or("Invalid LLM config format")?;

    let model = models
        .iter()
        .find(|m| m.get("model_id").and_then(|v| v.as_str()) == Some(model_id))
        .or_else(|| {
            models
                .iter()
                .find(|m| m.get("id").and_then(|v| v.as_str()) == Some(model_id))
        })
        .ok_or(format!("Model '{}' not found in LLM config", model_id))?;

    let base_url = model
        .get("base_url")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let api_key = model
        .get("api_key")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    Ok(EmbeddingModelConfig { base_url, api_key })
}

fn generate_query_embedding_blocking(
    model_id: &str,
    base_url: &str,
    api_key: &str,
    query: &str,
) -> Result<Vec<f32>, String> {
    let access_token = format!("Bearer {}", api_key);
    let client = ai_llm_kit::LlmClient::new(base_url, "", &access_token).set_model(model_id);

    let body = value!({
        "model": model_id,
        "input": vec![query],
    });

    let resp = {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| format!("Runtime error: {}", e))?;
        let local = tokio::task::LocalSet::new();
        local
            .block_on(&rt, client.embeddings(&body))
            .map_err(|e| format!("Embedding failed: {}", e))?
    };

    let data_arr = resp
        .get("data")
        .and_then(|d| d.as_array())
        .ok_or("Invalid embedding response: missing data array")?;

    let first_item = data_arr
        .first()
        .ok_or("Empty embedding response data array")?;

    let embedding_arr = first_item
        .get("embedding")
        .and_then(|e| e.as_array())
        .ok_or("Missing embedding vector in response")?;

    let embedding: Vec<f32> = embedding_arr
        .iter()
        .filter_map(|v| v.as_f64().map(|f| f as f32))
        .collect();

    if embedding.is_empty() {
        return Err("Empty embedding vector returned".to_string());
    }

    Ok(embedding)
}
