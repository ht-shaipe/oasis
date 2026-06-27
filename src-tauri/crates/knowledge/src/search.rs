use crate::db;
use rusqlite::Connection;

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
