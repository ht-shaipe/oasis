#![allow(dead_code)]
use crate::core::credential_manager::models::{Credential, MasterKeyConfig};
use crate::core::credential_manager::db;
use anyhow::Result;
use rusqlite::{params, OptionalExtension};
use std::path::PathBuf;
use std::sync::Mutex;

pub struct CredentialService {
    db_path: PathBuf,
    conn: Mutex<rusqlite::Connection>,
}

impl CredentialService {
    pub fn new(db_path: PathBuf) -> Result<Self> {
        let conn = db::ensure_db_exists(&db_path)?;
        Ok(Self {
            db_path,
            conn: Mutex::new(conn),
        })
    }

    // CRUD Operations

    pub fn create(&self, cred: Credential) -> Result<String> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO credentials (
                id, name, platform, category, username, password_encrypted,
                extra_fields, notes, is_active, created_at, updated_at, expires_at, tags
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                &cred.id,
                &cred.name,
                &cred.platform,
                &cred.category,
                &cred.username,
                &cred.password_encrypted,
                &cred.extra_fields,
                &cred.notes,
                cred.is_active,
                cred.created_at,
                cred.updated_at,
                cred.expires_at,
                &cred.tags,
            ],
        )?;

        Ok(cred.id)
    }

    pub fn read(&self, id: &str) -> Result<Credential> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, platform, category, username, password_encrypted,
                    extra_fields, notes, is_active, created_at, updated_at, expires_at, tags
             FROM credentials WHERE id = ?",
        )?;

        let cred = stmt.query_row(params![id], |row| {
            Ok(Credential {
                id: row.get(0)?,
                name: row.get(1)?,
                platform: row.get(2)?,
                category: row.get(3)?,
                username: row.get(4)?,
                password_encrypted: row.get(5)?,
                extra_fields: row.get(6)?,
                notes: row.get(7)?,
                is_active: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
                expires_at: row.get(11)?,
                tags: row.get(12)?,
            })
        })?;

        Ok(cred)
    }

    pub fn update(&self, id: &str, mut cred: Credential) -> Result<()> {
        cred.updated_at = chrono::Local::now().timestamp();
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "UPDATE credentials SET
                name = ?, platform = ?, category = ?, username = ?,
                password_encrypted = ?, extra_fields = ?, notes = ?,
                is_active = ?, updated_at = ?, expires_at = ?, tags = ?
             WHERE id = ?",
            params![
                &cred.name,
                &cred.platform,
                &cred.category,
                &cred.username,
                &cred.password_encrypted,
                &cred.extra_fields,
                &cred.notes,
                cred.is_active,
                cred.updated_at,
                cred.expires_at,
                &cred.tags,
                id,
            ],
        )?;

        Ok(())
    }

    pub fn delete(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM credentials WHERE id = ?", params![id])?;
        Ok(())
    }

    // Query Operations

    pub fn list_all(&self) -> Result<Vec<Credential>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, platform, category, username, password_encrypted,
                    extra_fields, notes, is_active, created_at, updated_at, expires_at, tags
             FROM credentials ORDER BY updated_at DESC",
        )?;

        let creds = stmt.query_map([], |row| {
            Ok(Credential {
                id: row.get(0)?,
                name: row.get(1)?,
                platform: row.get(2)?,
                category: row.get(3)?,
                username: row.get(4)?,
                password_encrypted: row.get(5)?,
                extra_fields: row.get(6)?,
                notes: row.get(7)?,
                is_active: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
                expires_at: row.get(11)?,
                tags: row.get(12)?,
            })
        })?;

        let mut result = Vec::new();
        for cred in creds {
            result.push(cred?);
        }

        Ok(result)
    }

    pub fn search(&self, query: &str) -> Result<Vec<Credential>> {
        let conn = self.conn.lock().unwrap();
        let search_term = format!("%{}%", query);

        let mut stmt = conn.prepare(
            "SELECT id, name, platform, category, username, password_encrypted,
                    extra_fields, notes, is_active, created_at, updated_at, expires_at, tags
             FROM credentials
             WHERE name LIKE ? OR platform LIKE ? OR username LIKE ? OR notes LIKE ? OR tags LIKE ?
             ORDER BY updated_at DESC",
        )?;

        let creds = stmt.query_map(
            params![
                &search_term,
                &search_term,
                &search_term,
                &search_term,
                &search_term
            ],
            |row| {
                Ok(Credential {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    platform: row.get(2)?,
                    category: row.get(3)?,
                    username: row.get(4)?,
                    password_encrypted: row.get(5)?,
                    extra_fields: row.get(6)?,
                    notes: row.get(7)?,
                    is_active: row.get(8)?,
                    created_at: row.get(9)?,
                    updated_at: row.get(10)?,
                    expires_at: row.get(11)?,
                    tags: row.get(12)?,
                })
            },
        )?;

        let mut result = Vec::new();
        for cred in creds {
            result.push(cred?);
        }

        Ok(result)
    }

    pub fn filter_by_platform(&self, platform: &str) -> Result<Vec<Credential>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, platform, category, username, password_encrypted,
                    extra_fields, notes, is_active, created_at, updated_at, expires_at, tags
             FROM credentials WHERE platform = ? ORDER BY updated_at DESC",
        )?;

        let creds = stmt.query_map(params![platform], |row| {
            Ok(Credential {
                id: row.get(0)?,
                name: row.get(1)?,
                platform: row.get(2)?,
                category: row.get(3)?,
                username: row.get(4)?,
                password_encrypted: row.get(5)?,
                extra_fields: row.get(6)?,
                notes: row.get(7)?,
                is_active: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
                expires_at: row.get(11)?,
                tags: row.get(12)?,
            })
        })?;

        let mut result = Vec::new();
        for cred in creds {
            result.push(cred?);
        }

        Ok(result)
    }

    pub fn filter_by_category(&self, category: &str) -> Result<Vec<Credential>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, platform, category, username, password_encrypted,
                    extra_fields, notes, is_active, created_at, updated_at, expires_at, tags
             FROM credentials WHERE category = ? ORDER BY updated_at DESC",
        )?;

        let creds = stmt.query_map(params![category], |row| {
            Ok(Credential {
                id: row.get(0)?,
                name: row.get(1)?,
                platform: row.get(2)?,
                category: row.get(3)?,
                username: row.get(4)?,
                password_encrypted: row.get(5)?,
                extra_fields: row.get(6)?,
                notes: row.get(7)?,
                is_active: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
                expires_at: row.get(11)?,
                tags: row.get(12)?,
            })
        })?;

        let mut result = Vec::new();
        for cred in creds {
            result.push(cred?);
        }

        Ok(result)
    }

    pub fn filter_by_tags(&self, tags: &[String]) -> Result<Vec<Credential>> {
        let conn = self.conn.lock().unwrap();
        let mut result = Vec::new();

        for tag in tags {
            let search_term = format!("%{}%", tag);
            let mut stmt = conn.prepare(
                "SELECT id, name, platform, category, username, password_encrypted,
                        extra_fields, notes, is_active, created_at, updated_at, expires_at, tags
                 FROM credentials WHERE tags LIKE ? ORDER BY updated_at DESC",
            )?;

            let creds = stmt.query_map(params![&search_term], |row| {
                Ok(Credential {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    platform: row.get(2)?,
                    category: row.get(3)?,
                    username: row.get(4)?,
                    password_encrypted: row.get(5)?,
                    extra_fields: row.get(6)?,
                    notes: row.get(7)?,
                    is_active: row.get(8)?,
                    created_at: row.get(9)?,
                    updated_at: row.get(10)?,
                    expires_at: row.get(11)?,
                    tags: row.get(12)?,
                })
            })?;

            for cred in creds {
                result.push(cred?);
            }
        }

        // Remove duplicates
        result.sort_by(|a, b| a.id.cmp(&b.id));
        result.dedup_by(|a, b| a.id == b.id);

        Ok(result)
    }

    // Master Key Management

    pub fn set_master_key(&self, config: MasterKeyConfig) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO master_key_config (key_version, derived_from, salt, iv, created_at)
             VALUES (?, ?, ?, ?, ?)",
            params![
                config.key_version,
                &config.derived_from,
                &config.salt,
                &config.iv,
                config.created_at,
            ],
        )?;
        Ok(())
    }

    pub fn get_master_key(&self) -> Result<Option<MasterKeyConfig>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT key_version, derived_from, salt, iv, created_at FROM master_key_config
             ORDER BY key_version DESC LIMIT 1",
        )?;

        let config = stmt
            .query_row([], |row| {
                Ok(MasterKeyConfig {
                    key_version: row.get(0)?,
                    derived_from: row.get(1)?,
                    salt: row.get(2)?,
                    iv: row.get(3)?,
                    created_at: row.get(4)?,
                })
            })
            .optional()?;

        Ok(config)
    }
}
