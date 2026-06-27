use rusqlite::{params, Connection, Result as SqlResult};
use serde::{Deserialize, Serialize};
use std::path::Path;

pub fn init_db(app_data_dir: &Path) -> SqlResult<Connection> {
    std::fs::create_dir_all(app_data_dir).ok();
    let db_path = app_data_dir.join("knowledge.db");
    let conn = Connection::open(db_path)?;

    conn.execute("PRAGMA foreign_keys = ON", [])?;
    conn.execute("PRAGMA journal_mode = WAL", [])?;

    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS indexed_files (
            id          TEXT PRIMARY KEY,
            path        TEXT NOT NULL UNIQUE,
            rel_path    TEXT NOT NULL,
            ext         TEXT,
            size        INTEGER NOT NULL,
            modified    TEXT NOT NULL,
            hash        TEXT,
            indexed_at  TEXT NOT NULL,
            chunk_count INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS chunks (
            id          TEXT PRIMARY KEY,
            file_id     TEXT NOT NULL REFERENCES indexed_files(id) ON DELETE CASCADE,
            chunk_index INTEGER NOT NULL,
            content     TEXT NOT NULL,
            embedding   BLOB,
            token_count INTEGER,
            line_start  INTEGER,
            line_end    INTEGER,
            created_at  TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_chunks_file ON chunks(file_id);
        CREATE INDEX IF NOT EXISTS idx_chunks_file_index ON chunks(file_id, chunk_index);

        CREATE TABLE IF NOT EXISTS index_meta (
            key         TEXT PRIMARY KEY,
            value       TEXT NOT NULL
        );
        ",
    )?;

    Ok(conn)
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

pub fn get_meta(conn: &Connection, key: &str) -> SqlResult<Option<String>> {
    let mut stmt = conn.prepare("SELECT value FROM index_meta WHERE key = ?1")?;
    let mut rows = stmt.query(params![key])?;
    match rows.next()? {
        Some(row) => Ok(Some(row.get(0)?)),
        None => Ok(None),
    }
}

pub fn set_meta(conn: &Connection, key: &str, value: &str) -> SqlResult<()> {
    conn.execute(
        "INSERT OR REPLACE INTO index_meta (key, value) VALUES (?1, ?2)",
        params![key, value],
    )?;
    Ok(())
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IndexedFile {
    pub id: String,
    pub path: String,
    pub rel_path: String,
    pub ext: Option<String>,
    pub size: i64,
    pub modified: String,
    pub hash: Option<String>,
    pub indexed_at: String,
    pub chunk_count: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Chunk {
    pub id: String,
    pub file_id: String,
    pub chunk_index: i32,
    pub content: String,
    pub embedding: Option<Vec<f32>>,
    pub token_count: Option<i32>,
    pub line_start: Option<i32>,
    pub line_end: Option<i32>,
    pub created_at: String,
}

pub fn upsert_file(conn: &Connection, file: &IndexedFile) -> SqlResult<()> {
    conn.execute(
        "INSERT OR REPLACE INTO indexed_files (id, path, rel_path, ext, size, modified, hash, indexed_at, chunk_count)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![file.id, file.path, file.rel_path, file.ext, file.size, file.modified, file.hash, file.indexed_at, file.chunk_count],
    )?;
    Ok(())
}

pub fn get_file_by_path(conn: &Connection, path: &str) -> SqlResult<Option<IndexedFile>> {
    let mut stmt = conn.prepare(
        "SELECT id, path, rel_path, ext, size, modified, hash, indexed_at, chunk_count FROM indexed_files WHERE path = ?1",
    )?;
    let mut rows = stmt.query(params![path])?;
    match rows.next()? {
        Some(row) => Ok(Some(IndexedFile {
            id: row.get(0)?,
            path: row.get(1)?,
            rel_path: row.get(2)?,
            ext: row.get(3)?,
            size: row.get(4)?,
            modified: row.get(5)?,
            hash: row.get(6)?,
            indexed_at: row.get(7)?,
            chunk_count: row.get(8)?,
        })),
        None => Ok(None),
    }
}

pub fn delete_file(conn: &Connection, file_id: &str) -> SqlResult<()> {
    conn.execute("DELETE FROM indexed_files WHERE id = ?1", params![file_id])?;
    Ok(())
}

pub fn delete_file_by_path(conn: &Connection, path: &str) -> SqlResult<()> {
    conn.execute("DELETE FROM indexed_files WHERE path = ?1", params![path])?;
    Ok(())
}

pub fn insert_chunk(conn: &Connection, chunk: &Chunk) -> SqlResult<()> {
    let embedding_blob = chunk.embedding.as_ref().map(|v| {
        let bytes: Vec<u8> = v.iter().flat_map(|f| f.to_le_bytes()).collect();
        bytes
    });
    conn.execute(
        "INSERT OR REPLACE INTO chunks (id, file_id, chunk_index, content, embedding, token_count, line_start, line_end, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![chunk.id, chunk.file_id, chunk.chunk_index, chunk.content, embedding_blob, chunk.token_count, chunk.line_start, chunk.line_end, chunk.created_at],
    )?;
    Ok(())
}

pub fn delete_chunks_for_file(conn: &Connection, file_id: &str) -> SqlResult<()> {
    conn.execute("DELETE FROM chunks WHERE file_id = ?1", params![file_id])?;
    Ok(())
}

pub fn get_all_files(conn: &Connection) -> SqlResult<Vec<IndexedFile>> {
    let mut stmt = conn.prepare(
        "SELECT id, path, rel_path, ext, size, modified, hash, indexed_at, chunk_count FROM indexed_files ORDER BY rel_path",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(IndexedFile {
            id: row.get(0)?,
            path: row.get(1)?,
            rel_path: row.get(2)?,
            ext: row.get(3)?,
            size: row.get(4)?,
            modified: row.get(5)?,
            hash: row.get(6)?,
            indexed_at: row.get(7)?,
            chunk_count: row.get(8)?,
        })
    })?;
    rows.collect()
}

pub fn get_all_chunk_embeddings(conn: &Connection) -> SqlResult<Vec<(String, String, String, i32, Vec<f32>)>> {
    let mut stmt = conn.prepare(
        "SELECT c.id, c.content, f.rel_path, c.chunk_index, c.embedding
         FROM chunks c
         JOIN indexed_files f ON c.file_id = f.id
         WHERE c.embedding IS NOT NULL
         ORDER BY f.rel_path, c.chunk_index",
    )?;
    let rows = stmt.query_map([], |row| {
        let chunk_id: String = row.get(0)?;
        let content: String = row.get(1)?;
        let rel_path: String = row.get(2)?;
        let chunk_index: i32 = row.get(3)?;
        let blob: Vec<u8> = row.get(4)?;
        let embedding: Vec<f32> = blob.chunks_exact(4).map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]])).collect();
        Ok((chunk_id, content, rel_path, chunk_index, embedding))
    })?;
    rows.collect()
}

pub fn clear_all(conn: &Connection) -> SqlResult<()> {
    conn.execute("DELETE FROM chunks", [])?;
    conn.execute("DELETE FROM indexed_files", [])?;
    conn.execute("DELETE FROM index_meta", [])?;
    Ok(())
}

pub fn count_files(conn: &Connection) -> SqlResult<i64> {
    conn.query_row("SELECT COUNT(*) FROM indexed_files", [], |row| row.get(0))
}

pub fn count_chunks(conn: &Connection) -> SqlResult<i64> {
    conn.query_row("SELECT COUNT(*) FROM chunks", [], |row| row.get(0))
}

pub fn count_embedded_chunks(conn: &Connection) -> SqlResult<i64> {
    conn.query_row("SELECT COUNT(*) FROM chunks WHERE embedding IS NOT NULL", [], |row| row.get(0))
}

pub fn update_file_chunk_count(conn: &Connection, file_id: &str, count: i32) -> SqlResult<()> {
    conn.execute(
        "UPDATE indexed_files SET chunk_count = ?1 WHERE id = ?2",
        params![count, file_id],
    )?;
    Ok(())
}

pub fn update_chunk_embedding(conn: &Connection, chunk_id: &str, embedding: &[f32]) -> SqlResult<()> {
    let blob: Vec<u8> = embedding.iter().flat_map(|f| f.to_le_bytes()).collect();
    conn.execute(
        "UPDATE chunks SET embedding = ?1 WHERE id = ?2",
        params![blob, chunk_id],
    )?;
    Ok(())
}

pub fn get_chunks_without_embedding(conn: &Connection) -> SqlResult<Vec<Chunk>> {
    let mut stmt = conn.prepare(
        "SELECT id, file_id, chunk_index, content, token_count, line_start, line_end, created_at
         FROM chunks WHERE embedding IS NULL",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(Chunk {
            id: row.get(0)?,
            file_id: row.get(1)?,
            chunk_index: row.get(2)?,
            content: row.get(3)?,
            embedding: None,
            token_count: row.get(4)?,
            line_start: row.get(5)?,
            line_end: row.get(6)?,
            created_at: row.get(7)?,
        })
    })?;
    rows.collect()
}
