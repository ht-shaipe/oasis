use crate::chunker;
use crate::db;
use crate::parser;
use crate::vector_index::{ChunkMeta as VectorChunkMeta, VectorIndex, get_chunk_meta_path, get_index_path};
use ai_llm_kit::LlmService;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;
use tube::value;

static INDEXING_IN_PROGRESS: AtomicBool = AtomicBool::new(false);

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct IndexResult {
    pub indexed_files: i32,
    pub skipped_files: i32,
    pub deleted_files: i32,
    pub total_chunks: i32,
    pub elapsed_secs: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeStatus {
    pub workspace_dir: String,
    pub total_files: i64,
    pub total_chunks: i64,
    pub embedded_chunks: i64,
    pub last_index_time: Option<String>,
    pub is_indexing: bool,
    pub embedding_model: Option<String>,
    pub embedding_dim: Option<i32>,
}

pub fn get_status(conn: &Connection, workspace_dir: &str) -> KnowledgeStatus {
    let total_files = db::count_files(conn).unwrap_or(0);
    let total_chunks = db::count_chunks(conn).unwrap_or(0);
    let embedded_chunks = db::count_embedded_chunks(conn).unwrap_or(0);
    let last_index_time = db::get_meta(conn, "last_index_time").ok().flatten();
    let embedding_model = db::get_meta(conn, "embedding_model").ok().flatten();
    let embedding_dim = db::get_meta(conn, "embedding_dim")
        .ok()
        .flatten()
        .and_then(|s| s.parse::<i32>().ok());

    KnowledgeStatus {
        workspace_dir: workspace_dir.to_string(),
        total_files,
        total_chunks,
        embedded_chunks,
        last_index_time,
        is_indexing: INDEXING_IN_PROGRESS.load(Ordering::Relaxed),
        embedding_model,
        embedding_dim,
    }
}

pub fn is_indexing() -> bool {
    INDEXING_IN_PROGRESS.load(Ordering::Relaxed)
}

pub fn stop_indexing() {
    INDEXING_IN_PROGRESS.store(false, Ordering::Relaxed);
}

pub fn scan_workspace(workspace_dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let walker = walkdir::WalkDir::new(workspace_dir).follow_links(false);

    for entry in walker {
        let Ok(entry) = entry else { continue };
        let path = entry.path().to_path_buf();

        if !path.is_file() {
            continue;
        }

        if let Some(parent) = entry.path().parent() {
            if let Some(dir_name) = parent.file_name().and_then(|n| n.to_str()) {
                if parser::should_skip_dir(dir_name) {
                    continue;
                }
            }
        }

        if parser::is_indexable(&path) {
            files.push(path);
        }
    }

    files
}

pub fn run_index(
    conn: &mut Connection,
    workspace_dir: &Path,
    _cancel: &AtomicBool,
) -> Result<IndexResult, String> {
    if INDEXING_IN_PROGRESS.swap(true, Ordering::Relaxed) {
        return Err("Indexing already in progress".to_string());
    }

    let start = Instant::now();
    let mut result = IndexResult::default();

    let tx = conn.transaction().map_err(|e| e.to_string())?;

    let workspace_str = workspace_dir.to_string_lossy().to_string();

    let existing_files = db::get_all_files(&tx).map_err(|e| e.to_string())?;
    let existing_map: std::collections::HashMap<String, db::IndexedFile> = existing_files
        .into_iter()
        .map(|f| (f.path.clone(), f))
        .collect();

    let disk_files = scan_workspace(workspace_dir);
    let disk_set: std::collections::HashSet<String> = disk_files
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    for (path_str, _existing) in &existing_map {
        if !disk_set.contains(path_str) {
            db::delete_file_by_path(&tx, path_str).map_err(|e| e.to_string())?;
            result.deleted_files += 1;
        }
    }

    for file_path in &disk_files {
        if !INDEXING_IN_PROGRESS.load(Ordering::Relaxed) {
            break;
        }

        let path_str = file_path.to_string_lossy().to_string();
        let rel_path = file_path
            .strip_prefix(workspace_dir)
            .unwrap_or(file_path)
            .to_string_lossy()
            .to_string();

        let Some(content) = parser::read_file_content(file_path) else {
            result.skipped_files += 1;
            continue;
        };

        let hash = parser::compute_hash(&content.content);

        if let Some(existing) = existing_map.get(&path_str) {
            if existing.hash.as_deref() == Some(&hash) {
                continue;
            }
            db::delete_chunks_for_file(&tx, &existing.id).map_err(|e| e.to_string())?;
        }

        let metadata = std::fs::metadata(file_path).map_err(|e| e.to_string())?;
        let modified = metadata
            .modified()
            .ok()
            .and_then(|t| {
                let dur = t.duration_since(std::time::UNIX_EPOCH).ok()?;
                chrono::DateTime::from_timestamp(dur.as_secs() as i64, 0)
                    .map(|d| d.format("%Y-%m-%dT%H:%M:%SZ").to_string())
            })
            .unwrap_or_default();

        let file_id = uuid::Uuid::new_v4().to_string();
        let ext = file_path
            .extension()
            .and_then(|e| e.to_str())
            .map(String::from);

        let file_record = db::IndexedFile {
            id: file_id.clone(),
            path: path_str,
            rel_path,
            ext,
            size: metadata.len() as i64,
            modified,
            hash: Some(hash),
            indexed_at: now_iso(),
            chunk_count: 0,
        };

        let chunks = chunker::chunk_text(&content.content);
        let chunk_count = chunks.len() as i32;

        for chunk in chunks {
            let chunk_record = db::Chunk {
                id: uuid::Uuid::new_v4().to_string(),
                file_id: file_id.clone(),
                chunk_index: chunk.index,
                content: chunk.content,
                embedding: None,
                token_count: None,
                line_start: Some(chunk.line_start),
                line_end: Some(chunk.line_end),
                created_at: now_iso(),
            };
            db::insert_chunk(&tx, &chunk_record).map_err(|e| e.to_string())?;
        }

        let mut file_record = file_record;
        file_record.chunk_count = chunk_count;
        db::upsert_file(&tx, &file_record).map_err(|e| e.to_string())?;

        result.indexed_files += 1;
        result.total_chunks += chunk_count;
    }

    let total_chunks = db::count_chunks(&tx).map_err(|e| e.to_string())?;
    result.total_chunks = total_chunks as i32;

    db::set_meta(&tx, "last_index_time", &now_iso()).map_err(|e| e.to_string())?;
    db::set_meta(&tx, "workspace_dir", &workspace_str).map_err(|e| e.to_string())?;

    tx.commit().map_err(|e| e.to_string())?;

    result.elapsed_secs = start.elapsed().as_secs_f64();
    INDEXING_IN_PROGRESS.store(false, Ordering::Relaxed);

    Ok(result)
}

pub fn generate_embeddings_blocking(
    conn: &mut Connection,
    model_id: &str,
    base_url: &str,
    api_key: &str,
    batch_size: usize,
    app_data_dir: &Path,
) -> Result<i32, String> {
    if INDEXING_IN_PROGRESS.swap(true, Ordering::Relaxed) {
        return Err("Indexing already in progress".to_string());
    }

    let mut embedded_count = 0i32;

    let chunks = db::get_chunks_without_embedding(conn).map_err(|e| e.to_string())?;
    if chunks.is_empty() {
        INDEXING_IN_PROGRESS.store(false, Ordering::Relaxed);
        return Ok(0);
    }

    db::set_meta(conn, "embedding_model", model_id).map_err(|e| e.to_string())?;

    let access_token = format!("Bearer {}", api_key);

    for batch in chunks.chunks(batch_size) {
        if !INDEXING_IN_PROGRESS.load(Ordering::Relaxed) {
            break;
        }

        let texts: Vec<String> = batch.iter().map(|c| c.content.clone()).collect();
        let model_id_owned = model_id.to_string();

        let resp = {
            let client = ai_llm_kit::LlmClient::new(base_url, "", &access_token)
                .set_model(&model_id_owned);
            let body = value!({
                "model": model_id_owned,
                "input": texts,
            });

            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|e| format!("Runtime error: {}", e))?;
            let local = tokio::task::LocalSet::new();
            local
                .block_on(&rt, client.embeddings(&body))
                .map_err(|e| format!("Embedding failed: {}", e))?
        };

        let data = resp
            .get("data")
            .and_then(|d| d.as_array())
            .ok_or("Invalid embedding response: missing data array")?;

        if data.len() != batch.len() {
            INDEXING_IN_PROGRESS.store(false, Ordering::Relaxed);
            return Err(format!(
                "Embedding count mismatch: expected {}, got {}",
                batch.len(),
                data.len()
            ));
        }

        let mut dim: Option<i32> = None;

        for (i, chunk) in batch.iter().enumerate() {
            let embedding_data = data[i]
                .get("embedding")
                .and_then(|e| e.as_array())
                .ok_or("Missing embedding vector in response")?;

            let embedding: Vec<f32> = embedding_data
                .iter()
                .filter_map(|v| v.as_f64().map(|f| f as f32))
                .collect();

            if embedding.is_empty() {
                continue;
            }

            if dim.is_none() {
                dim = Some(embedding.len() as i32);
            }

            db::update_chunk_embedding(conn, &chunk.id, &embedding)
                .map_err(|e| e.to_string())?;
            embedded_count += 1;
        }

        if let Some(d) = dim {
            db::set_meta(conn, "embedding_dim", &d.to_string()).map_err(|e| e.to_string())?;
        }

        if batch.len() >= batch_size {
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
    }

    let _embedded_total = db::count_embedded_chunks(conn).map_err(|e| e.to_string())?;
    db::set_meta(conn, "last_index_time", &now_iso()).map_err(|e| e.to_string())?;

    rebuild_vector_index(conn, app_data_dir)?;

    INDEXING_IN_PROGRESS.store(false, Ordering::Relaxed);

    Ok(embedded_count)
}

pub fn generate_embeddings_local(
    conn: &mut Connection,
    model_id: &str,
    app_data_dir: &Path,
) -> Result<i32, String> {
    if INDEXING_IN_PROGRESS.swap(true, Ordering::Relaxed) {
        return Err("Indexing already in progress".to_string());
    }

    let mut embedded_count = 0i32;

    let chunks = db::get_chunks_without_embedding(conn).map_err(|e| e.to_string())?;
    if chunks.is_empty() {
        INDEXING_IN_PROGRESS.store(false, Ordering::Relaxed);
        return Ok(0);
    }

    let cache_dir = app_data_dir.join("embed_models");
    std::fs::create_dir_all(&cache_dir).map_err(|e| e.to_string())?;

    let key = oasis_embed::commands::find_catalog_key(model_id)
        .ok_or(format!("Unknown local model: {}", model_id))?;

    let mut model = fastembed::TextEmbedding::try_new(
        fastembed::TextInitOptions::new(key).with_cache_dir(cache_dir),
    )
    .map_err(|e| format!("Failed to load local model '{}': {}", model_id, e))?;

    db::set_meta(conn, "embedding_model", model_id).map_err(|e| e.to_string())?;

    let batch_size = 20;
    for batch in chunks.chunks(batch_size) {
        if !INDEXING_IN_PROGRESS.load(Ordering::Relaxed) {
            break;
        }

        let texts: Vec<String> = batch.iter().map(|c| c.content.clone()).collect();

        let embeddings = model
            .embed(texts, None)
            .map_err(|e| format!("Local embedding inference failed: {}", e))?;

        if embeddings.len() != batch.len() {
            INDEXING_IN_PROGRESS.store(false, Ordering::Relaxed);
            return Err(format!(
                "Embedding count mismatch: expected {}, got {}",
                batch.len(),
                embeddings.len()
            ));
        }

        let mut dim: Option<i32> = None;

        for (i, chunk) in batch.iter().enumerate() {
            let embedding = &embeddings[i];

            if embedding.is_empty() {
                continue;
            }

            if dim.is_none() {
                dim = Some(embedding.len() as i32);
            }

            db::update_chunk_embedding(conn, &chunk.id, embedding)
                .map_err(|e| e.to_string())?;
            embedded_count += 1;
        }

        if let Some(d) = dim {
            db::set_meta(conn, "embedding_dim", &d.to_string()).map_err(|e| e.to_string())?;
        }
    }

    db::set_meta(conn, "last_index_time", &now_iso()).map_err(|e| e.to_string())?;

    rebuild_vector_index(conn, app_data_dir)?;

    INDEXING_IN_PROGRESS.store(false, Ordering::Relaxed);

    Ok(embedded_count)
}

fn now_iso() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();
    chrono::DateTime::from_timestamp(secs as i64, 0)
        .map(|d| d.format("%Y-%m-%dT%H:%M:%SZ").to_string())
        .unwrap_or_else(|| format!("{}", secs))
}

pub fn rebuild_vector_index(conn: &Connection, app_data_dir: &Path) -> Result<(), String> {
    let dim_str = db::get_meta(conn, "embedding_dim")
        .ok()
        .flatten()
        .ok_or("Embedding dimension not set")?;
    let dimensions = dim_str.parse::<usize>().map_err(|e| e.to_string())?;

    let index_path = get_index_path(app_data_dir);
    let meta_path = get_chunk_meta_path(app_data_dir);

    let mut vi = VectorIndex::rebuild_from_db(conn, dimensions, index_path)?;

    let meta = VectorChunkMeta {
        key_to_chunk_id: vi.key_to_chunk_id.clone(),
        chunk_id_to_key: vi.chunk_id_to_key.clone(),
        next_key: vi.next_key,
        dimensions: vi.dimensions(),
    };
    meta.save(&meta_path)?;

    Ok(())
}
