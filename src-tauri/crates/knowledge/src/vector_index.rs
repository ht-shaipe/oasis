use rusqlite::Connection;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use usearch::{Index, IndexOptions, MetricKind, ScalarKind};

use crate::db;

pub struct VectorIndex {
    index: Index,
    pub key_to_chunk_id: HashMap<u64, String>,
    pub chunk_id_to_key: HashMap<String, u64>,
    pub next_key: u64,
    dimensions: usize,
    index_path: PathBuf,
}

impl VectorIndex {
    pub fn new(dimensions: usize, index_path: PathBuf) -> Result<Self, String> {
        let options = IndexOptions {
            dimensions,
            metric: MetricKind::Cos,
            quantization: ScalarKind::F32,
            connectivity: 16,
            expansion_add: 256,
            expansion_search: 128,
            multi: false,
        };

        let index = Index::new(&options).map_err(|e| format!("Failed to create index: {}", e))?;

        Ok(Self {
            index,
            key_to_chunk_id: HashMap::new(),
            chunk_id_to_key: HashMap::new(),
            next_key: 1,
            dimensions,
            index_path,
        })
    }

    pub fn load_or_create(dimensions: usize, index_path: PathBuf) -> Result<Self, String> {
        if index_path.exists() {
            let index = Index::restore(index_path.to_string_lossy().as_ref())
                .map_err(|e| format!("Failed to restore index: {}", e))?;

            let loaded_dims = index.dimensions();
            if loaded_dims != dimensions {
                return Err(format!(
                    "Index dimension mismatch: expected {}, found {}",
                    dimensions, loaded_dims
                ));
            }

            return Ok(Self {
                index,
                key_to_chunk_id: HashMap::new(),
                chunk_id_to_key: HashMap::new(),
                next_key: 1,
                dimensions,
                index_path,
            });
        }

        Self::new(dimensions, index_path)
    }

    pub fn rebuild_from_db(
        conn: &Connection,
        dimensions: usize,
        index_path: PathBuf,
    ) -> Result<Self, String> {
        let all = db::get_all_chunk_embeddings(conn).map_err(|e| e.to_string())?;
        if all.is_empty() {
            return Self::new(dimensions, index_path);
        }

        let actual_dim = all
            .first()
            .map(|(_, _, _, _, emb)| emb.len())
            .unwrap_or(dimensions);

        let mut vi = Self::new(actual_dim, index_path)?;

        vi.index
            .reserve(all.len())
            .map_err(|e| format!("Failed to reserve index capacity: {}", e))?;

        for (chunk_id, _content, _rel_path, _chunk_index, embedding) in &all {
            let key = vi.next_key;
            vi.index
                .add(key, embedding.as_slice())
                .map_err(|e| format!("Failed to add vector: {}", e))?;
            vi.key_to_chunk_id.insert(key, chunk_id.clone());
            vi.chunk_id_to_key.insert(chunk_id.clone(), key);
            vi.next_key += 1;
        }

        vi.save()?;

        Ok(vi)
    }

    pub fn add(&mut self, chunk_id: &str, embedding: &[f32]) -> Result<(), String> {
        if let Some(&existing_key) = self.chunk_id_to_key.get(chunk_id) {
            self.index
                .remove(existing_key)
                .map_err(|e| format!("Failed to remove old vector: {}", e))?;
            self.key_to_chunk_id.remove(&existing_key);
        }

        let key = self.next_key;
        self.index
            .add(key, embedding)
            .map_err(|e| format!("Failed to add vector: {}", e))?;

        self.key_to_chunk_id.insert(key, chunk_id.to_string());
        self.chunk_id_to_key.insert(chunk_id.to_string(), key);
        self.next_key += 1;

        Ok(())
    }

    pub fn remove(&mut self, chunk_id: &str) -> Result<(), String> {
        if let Some(key) = self.chunk_id_to_key.remove(chunk_id) {
            self.index
                .remove(key)
                .map_err(|e| format!("Failed to remove vector: {}", e))?;
            self.key_to_chunk_id.remove(&key);
        }
        Ok(())
    }

    pub fn remove_by_prefix(&mut self, prefix: &str) -> Result<usize, String> {
        let keys_to_remove: Vec<(u64, String)> = self
            .chunk_id_to_key
            .iter()
            .filter(|(chunk_id, _)| chunk_id.starts_with(prefix))
            .map(|(chunk_id, &key)| (key, chunk_id.clone()))
            .collect();

        let count = keys_to_remove.len();
        for (key, chunk_id) in keys_to_remove {
            self.index
                .remove(key)
                .map_err(|e| format!("Failed to remove vector: {}", e))?;
            self.chunk_id_to_key.remove(&chunk_id);
            self.key_to_chunk_id.remove(&key);
        }

        Ok(count)
    }

    pub fn search(
        &self,
        query: &[f32],
        top_k: usize,
    ) -> Result<Vec<(String, f32)>, String> {
        if query.len() != self.dimensions {
            return Err(format!(
                "Query dimension mismatch: expected {}, got {}",
                self.dimensions,
                query.len()
            ));
        }

        let results = self
            .index
            .search(query, top_k)
            .map_err(|e| format!("Search failed: {}", e))?;

        let mut matches = Vec::new();
        for (i, &key) in results.keys.iter().enumerate() {
            if i >= results.distances.len() {
                break;
            }
            let distance = results.distances[i];
            let similarity = 1.0 - distance;
            if similarity <= 0.0 {
                continue;
            }
            if let Some(chunk_id) = self.key_to_chunk_id.get(&key) {
                matches.push((chunk_id.clone(), similarity));
            }
        }

        Ok(matches)
    }

    pub fn save(&self) -> Result<(), String> {
        if let Some(parent) = self.index_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("Failed to create index dir: {}", e))?;
        }
        self.index
            .save(self.index_path.to_string_lossy().as_ref())
            .map_err(|e| format!("Failed to save index: {}", e))?;
        Ok(())
    }

    pub fn size(&self) -> usize {
        self.index.size()
    }

    pub fn dimensions(&self) -> usize {
        self.dimensions
    }

    pub fn chunk_count(&self) -> usize {
        self.chunk_id_to_key.len()
    }
}

pub fn get_index_path(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join("vectors.usearch")
}

pub fn get_chunk_meta_path(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join("chunk_meta.json")
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct ChunkMeta {
    pub key_to_chunk_id: HashMap<u64, String>,
    pub chunk_id_to_key: HashMap<String, u64>,
    pub next_key: u64,
    pub dimensions: usize,
}

impl ChunkMeta {
    pub fn load(path: &Path) -> Result<Self, String> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        serde_json::from_str(&content).map_err(|e| e.to_string())
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let content = serde_json::to_string(self).map_err(|e| e.to_string())?;
        std::fs::write(path, content).map_err(|e| e.to_string())
    }
}
