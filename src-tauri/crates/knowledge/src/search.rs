use crate::db;
use crate::vector_index::{ChunkMeta, VectorIndex, get_chunk_meta_path, get_index_path};
use rusqlite::Connection;
use std::path::Path;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub file_path: String,
    pub rel_path: String,
    pub chunk_content: String,
    pub chunk_index: i32,
    pub score: f64,
    pub line_start: Option<i32>,
    pub line_end: Option<i32>,
}

pub fn search_by_query_embedding(
    conn: &Connection,
    query_embedding: &[f32],
    top_k: usize,
    app_data_dir: &Path,
) -> Result<Vec<SearchResult>, String> {
    let index_path = get_index_path(app_data_dir);
    let meta_path = get_chunk_meta_path(app_data_dir);

    let dim_str = db::get_meta(conn, "embedding_dim")
        .ok()
        .flatten()
        .ok_or("Embedding dimension not set. Run indexing first.")?;
    let dimensions = dim_str.parse::<usize>().map_err(|e| e.to_string())?;

    if let Some(results) = try_ann_search(conn, query_embedding, top_k, &index_path, &meta_path, dimensions) {
        return results;
    }

    brute_force_search(conn, query_embedding, top_k)
}

fn try_ann_search(
    conn: &Connection,
    query_embedding: &[f32],
    top_k: usize,
    index_path: &Path,
    meta_path: &Path,
    dimensions: usize,
) -> Option<Result<Vec<SearchResult>, String>> {
    if !index_path.exists() {
        return None;
    }

    let index = match VectorIndex::load_or_create(dimensions, index_path.to_path_buf()) {
        Ok(idx) => idx,
        Err(_) => return None,
    };

    let meta = match ChunkMeta::load(meta_path) {
        Ok(m) => m,
        Err(_) => return None,
    };

    if meta.chunk_id_to_key.is_empty() || index.size() == 0 {
        return None;
    }

    let mut index = index;
    index.key_to_chunk_id = meta.key_to_chunk_id;
    index.chunk_id_to_key = meta.chunk_id_to_key;
    index.next_key = meta.next_key;

    let matches = match index.search(query_embedding, top_k) {
        Ok(m) => m,
        Err(e) => return Some(Err(e)),
    };

    if matches.is_empty() {
        return None;
    }

    let chunk_ids: Vec<&str> = matches.iter().map(|(id, _)| id.as_str()).collect();
    let chunk_meta = match db::get_chunks_by_ids(conn, &chunk_ids) {
        Ok(m) => m,
        Err(e) => return Some(Err(e.to_string())),
    };

    let mut results = Vec::new();
    for (chunk_id, score) in &matches {
        if let Some(meta) = chunk_meta.get(chunk_id) {
            results.push(SearchResult {
                file_path: String::new(),
                rel_path: meta.rel_path.clone(),
                chunk_content: meta.content.clone(),
                chunk_index: meta.chunk_index,
                score: *score as f64,
                line_start: meta.line_start,
                line_end: meta.line_end,
            });
        }
    }

    Some(Ok(results))
}

fn brute_force_search(
    conn: &Connection,
    query_embedding: &[f32],
    top_k: usize,
) -> Result<Vec<SearchResult>, String> {
    let all = db::get_all_chunk_embeddings(conn).map_err(|e| e.to_string())?;

    let mut scored: Vec<SearchResult> = all
        .into_iter()
        .map(|(_chunk_id, content, rel_path, chunk_index, embedding)| {
            let score = cosine_similarity(query_embedding, &embedding);
            SearchResult {
                file_path: String::new(),
                rel_path,
                chunk_content: content,
                chunk_index,
                score,
                line_start: None,
                line_end: None,
            }
        })
        .filter(|r| r.score > 0.0)
        .collect();

    scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(top_k);

    Ok(scored)
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f64 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let mut dot = 0.0f64;
    let mut norm_a = 0.0f64;
    let mut norm_b = 0.0f64;
    for i in 0..a.len() {
        let av = a[i] as f64;
        let bv = b[i] as f64;
        dot += av * bv;
        norm_a += av * av;
        norm_b += bv * bv;
    }
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    dot / (norm_a.sqrt() * norm_b.sqrt())
}
