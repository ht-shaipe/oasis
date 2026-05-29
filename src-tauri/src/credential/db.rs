use std::path::Path;
use rusqlite::{Connection, Result, params};

use crate::credential::models::{Category, Credential, CredentialView, NewCredential, UpdateCredential};

const DEFAULT_CATEGORIES: &[(&str, &str)] = &[
    ("社交媒体", "users"),
    ("邮箱", "mail"),
    ("开发工具", "code"),
    ("API密钥", "key"),
    ("云服务", "cloud"),
    ("数据库", "database"),
    ("自定义", "folder"),
];

pub fn init_db(app_data_dir: &Path) -> Result<Connection> {
    std::fs::create_dir_all(app_data_dir).ok();
    let db_path = app_data_dir.join("credentials.db");
    let conn = Connection::open(db_path)?;

    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS master_key (
            id          INTEGER PRIMARY KEY CHECK (id = 1),
            key_hash    BLOB NOT NULL,
            salt        BLOB NOT NULL,
            dek_salt    BLOB NOT NULL,
            created_at  TEXT NOT NULL,
            updated_at  TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS categories (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            name        TEXT NOT NULL UNIQUE,
            icon        TEXT,
            sort_order  INTEGER DEFAULT 0,
            created_at  TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS credentials (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            category_id     INTEGER NOT NULL REFERENCES categories(id),
            title           TEXT NOT NULL,
            username        TEXT,
            url             TEXT,
            encrypted_data  BLOB NOT NULL,
            nonce           BLOB NOT NULL,
            tags            TEXT,
            notes           TEXT,
            created_at      TEXT NOT NULL,
            updated_at      TEXT NOT NULL
        );
        "
    )?;

    // Insert default categories if empty
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM categories",
        [],
        |row| row.get(0),
    )?;
    if count == 0 {
        let now = chrono_now();
        for (i, (name, icon)) in DEFAULT_CATEGORIES.iter().enumerate() {
            conn.execute(
                "INSERT INTO categories (name, icon, sort_order, created_at) VALUES (?1, ?2, ?3, ?4)",
                params![name, icon, i as i64, now],
            )?;
        }
    }

    Ok(conn)
}

fn now_iso() -> String {
    chrono_now()
}

fn chrono_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();
    // Simple ISO 8601-like format
    let dt = chrono::DateTime::from_timestamp(secs as i64, 0)
        .map(|d| d.format("%Y-%m-%dT%H:%M:%SZ").to_string())
        .unwrap_or_else(|| format!("{}", secs));
    dt
}

// ── Master Key ──────────────────────────────────────────────────────────────────

pub fn is_master_key_set(conn: &Connection) -> Result<bool> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM master_key",
        [],
        |row| row.get(0),
    )?;
    Ok(count > 0)
}

pub fn set_master_key(conn: &Connection, key_hash: &[u8], salt: &[u8], dek_salt: &[u8]) -> Result<()> {
    let now = now_iso();
    conn.execute(
        "INSERT OR REPLACE INTO master_key (id, key_hash, salt, dek_salt, created_at, updated_at) VALUES (1, ?1, ?2, ?3, ?4, ?4)",
        params![key_hash, salt, dek_salt, now],
    )?;
    Ok(())
}

pub fn verify_master_key(conn: &Connection, key_hash: &[u8]) -> Result<bool> {
    let stored: Vec<u8> = conn.query_row(
        "SELECT key_hash FROM master_key WHERE id = 1",
        [],
        |row| row.get(0),
    )?;
    Ok(stored == key_hash)
}

pub fn get_master_key_salts(conn: &Connection) -> Result<(Vec<u8>, Vec<u8>)> {
    conn.query_row(
        "SELECT salt, dek_salt FROM master_key WHERE id = 1",
        [],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )
}

// ── Categories ─────────────────────────────────────────────────────────────────

pub fn list_categories(conn: &Connection) -> Result<Vec<Category>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, icon, sort_order, created_at FROM categories ORDER BY sort_order"
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(Category {
            id: row.get(0)?,
            name: row.get(1)?,
            icon: row.get(2)?,
            sort_order: row.get(3)?,
            created_at: row.get(4)?,
        })
    })?;
    rows.collect()
}

pub fn create_category(conn: &Connection, name: &str, icon: Option<&str>) -> Result<Category> {
    let now = now_iso();
    let max_order: Option<i64> = conn.query_row(
        "SELECT MAX(sort_order) FROM categories",
        [],
        |row| row.get(0),
    ).ok();
    let sort_order = max_order.unwrap_or(0) + 1;
    conn.execute(
        "INSERT INTO categories (name, icon, sort_order, created_at) VALUES (?1, ?2, ?3, ?4)",
        params![name, icon, sort_order, now],
    )?;
    let id = conn.last_insert_rowid();
    Ok(Category {
        id,
        name: name.to_string(),
        icon: icon.map(String::from),
        sort_order,
        created_at: now,
    })
}

// ── Credentials ────────────────────────────────────────────────────────────────

fn to_credential_view(row: &rusqlite::Row) -> rusqlite::Result<CredentialView> {
    Ok(CredentialView {
        id: row.get(0)?,
        category_id: row.get(1)?,
        title: row.get(2)?,
        username: row.get(3)?,
        url: row.get(4)?,
        tags: row.get(5)?,
        notes: row.get(6)?,
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
        category_name: row.get(9)?,
    })
}

pub fn list_credentials(conn: &Connection, category_id: Option<i64>) -> Result<Vec<CredentialView>> {
    let sql = if category_id.is_some() {
        "SELECT c.id, c.category_id, c.title, c.username, c.url, c.tags, c.notes,
                c.created_at, c.updated_at, cat.name
         FROM credentials c
         LEFT JOIN categories cat ON c.category_id = cat.id
         WHERE c.category_id = ?1
         ORDER BY c.updated_at DESC"
    } else {
        "SELECT c.id, c.category_id, c.title, c.username, c.url, c.tags, c.notes,
                c.created_at, c.updated_at, cat.name
         FROM credentials c
         LEFT JOIN categories cat ON c.category_id = cat.id
         ORDER BY c.updated_at DESC"
    };
    let mut stmt = conn.prepare(sql)?;
    let rows = if let Some(cid) = category_id {
        stmt.query_map(params![cid], to_credential_view)?
    } else {
        stmt.query_map([], to_credential_view)?
    };
    rows.collect()
}

pub fn get_credential(conn: &Connection, id: i64) -> Result<Credential> {
    conn.query_row(
        "SELECT id, category_id, title, username, url, encrypted_data, nonce,
                tags, notes, created_at, updated_at
         FROM credentials WHERE id = ?1",
        params![id],
        |row| {
            Ok(Credential {
                id: row.get(0)?,
                category_id: row.get(1)?,
                title: row.get(2)?,
                username: row.get(3)?,
                url: row.get(4)?,
                encrypted_data: row.get(5)?,
                nonce: row.get(6)?,
                tags: row.get(7)?,
                notes: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        },
    )
}

pub fn create_credential(conn: &Connection, cred: &NewCredential) -> Result<Credential> {
    let now = now_iso();
    let enc_data = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &cred.sensitive_data_json)
        .map_err(|_| rusqlite::Error::InvalidParameterName("invalid base64".to_string()))?;

    // Generate nonce for this credential
    let nonce_bytes = crate::credential::crypto::generate_nonce();

    conn.execute(
        "INSERT INTO credentials (category_id, title, username, url, encrypted_data, nonce, tags, notes, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9)",
        params![
            cred.category_id,
            cred.title,
            cred.username,
            cred.url,
            enc_data,
            nonce_bytes.as_ref(),
            cred.tags,
            cred.notes,
            now,
        ],
    )?;
    let id = conn.last_insert_rowid();
    get_credential(conn, id)
}

pub fn update_credential(conn: &Connection, id: i64, cred: &UpdateCredential) -> Result<Credential> {
    let now = now_iso();

    // Build dynamic update
    if let Some(cid) = cred.category_id {
        conn.execute("UPDATE credentials SET category_id = ?1, updated_at = ?2 WHERE id = ?3", params![cid, now, id])?;
    }
    if let Some(ref title) = cred.title {
        conn.execute("UPDATE credentials SET title = ?1, updated_at = ?2 WHERE id = ?3", params![title, now, id])?;
    }
    if let Some(ref username) = cred.username {
        conn.execute("UPDATE credentials SET username = ?1, updated_at = ?2 WHERE id = ?3", params![username, now, id])?;
    }
    if let Some(ref url) = cred.url {
        conn.execute("UPDATE credentials SET url = ?1, updated_at = ?2 WHERE id = ?3", params![url, now, id])?;
    }
    if let Some(ref enc_json) = cred.sensitive_data_json {
        let enc_data = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, enc_json)
            .map_err(|_| rusqlite::Error::InvalidParameterName("invalid base64".to_string()))?;
        let nonce_bytes = crate::credential::crypto::generate_nonce();
        conn.execute(
            "UPDATE credentials SET encrypted_data = ?1, nonce = ?2, updated_at = ?3 WHERE id = ?4",
            params![enc_data, nonce_bytes.as_ref(), now, id],
        )?;
    }
    if let Some(ref tags) = cred.tags {
        conn.execute("UPDATE credentials SET tags = ?1, updated_at = ?2 WHERE id = ?3", params![tags, now, id])?;
    }
    if let Some(ref notes) = cred.notes {
        conn.execute("UPDATE credentials SET notes = ?1, updated_at = ?2 WHERE id = ?3", params![notes, now, id])?;
    }

    get_credential(conn, id)
}

pub fn delete_credential(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM credentials WHERE id = ?1", params![id])?;
    Ok(())
}
