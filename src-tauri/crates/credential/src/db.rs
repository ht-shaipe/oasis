use rusqlite::{params, Connection, Result};
use std::path::Path;

use crate::models::{
    Category, Credential, CredentialView, NewCredential, UpdateCredential,
    Site, SiteDetail,
};

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

    // Enable foreign keys
    conn.execute("PRAGMA foreign_keys = ON", [])?;

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
            parent_id   INTEGER REFERENCES categories(id),
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

        CREATE TABLE IF NOT EXISTS sites (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            category_id     INTEGER NOT NULL REFERENCES categories(id),
            name            TEXT NOT NULL,
            url             TEXT,
            tags            TEXT,
            notes           TEXT,
            created_at      TEXT NOT NULL,
            updated_at      TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS site_accounts (
            id                      INTEGER PRIMARY KEY AUTOINCREMENT,
            site_id                 INTEGER NOT NULL REFERENCES sites(id) ON DELETE CASCADE,
            username                TEXT NOT NULL,
            password_encrypted      BLOB NOT NULL,
            password_nonce          BLOB NOT NULL,
            api_key_encrypted       BLOB,
            api_key_nonce           BLOB,
            secret_key_encrypted    BLOB,
            secret_key_nonce        BLOB,
            created_at              TEXT NOT NULL
        );
        ",
    )?;

    // Migration: Add parent_id to categories if it doesn't exist
    let has_parent_id: bool = conn
        .query_row(
            "SELECT count(*) FROM pragma_table_info('categories') WHERE name='parent_id'",
            [],
            |row| Ok(row.get::<_, i64>(0)? > 0),
        )
        .unwrap_or(false);

    if !has_parent_id {
        conn.execute(
            "ALTER TABLE categories ADD COLUMN parent_id INTEGER REFERENCES categories(id)",
            [],
        )?;
    }

    // Insert default categories if empty
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM categories", [], |row| row.get(0))?;
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
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM master_key", [], |row| row.get(0))?;
    Ok(count > 0)
}

pub fn set_master_key(
    conn: &Connection,
    key_hash: &[u8],
    salt: &[u8],
    dek_salt: &[u8],
) -> Result<()> {
    let now = now_iso();
    conn.execute(
        "INSERT OR REPLACE INTO master_key (id, key_hash, salt, dek_salt, created_at, updated_at) VALUES (1, ?1, ?2, ?3, ?4, ?4)",
        params![key_hash, salt, dek_salt, now],
    )?;
    Ok(())
}

pub fn verify_master_key(conn: &Connection, key_hash: &[u8]) -> Result<bool> {
    let stored: Vec<u8> =
        conn.query_row("SELECT key_hash FROM master_key WHERE id = 1", [], |row| {
            row.get(0)
        })?;
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
        "SELECT id, parent_id, name, icon, sort_order, created_at FROM categories ORDER BY sort_order"
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(Category {
            id: row.get(0)?,
            parent_id: row.get(1)?,
            name: row.get(2)?,
            icon: row.get(3)?,
            sort_order: row.get(4)?,
            created_at: row.get(5)?,
        })
    })?;
    rows.collect()
}

pub fn create_category(
    conn: &Connection,
    name: &str,
    icon: Option<&str>,
    parent_id: Option<i64>,
) -> Result<Category> {
    let now = now_iso();
    let max_order: Option<i64> = conn
        .query_row("SELECT MAX(sort_order) FROM categories", [], |row| {
            row.get(0)
        })
        .ok();
    let sort_order = max_order.unwrap_or(0) + 1;
    conn.execute(
        "INSERT INTO categories (name, icon, sort_order, created_at, parent_id) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![name, icon, sort_order, now, parent_id],
    )?;
    let id = conn.last_insert_rowid();
    Ok(Category {
        id,
        parent_id,
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

pub fn list_credentials(
    conn: &Connection,
    category_id: Option<i64>,
) -> Result<Vec<CredentialView>> {
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
    let enc_data = base64::Engine::decode(
        &base64::engine::general_purpose::STANDARD,
        &cred.sensitive_data_json,
    )
    .map_err(|_| rusqlite::Error::InvalidParameterName("invalid base64".to_string()))?;

    // Decode provided nonce (expected base64) or generate a new one
    let nonce_bytes = base64::Engine::decode(
        &base64::engine::general_purpose::STANDARD,
        &cred.nonce_base64,
    )
    .map(|v| {
        let mut arr = [0u8; 12];
        arr.copy_from_slice(&v[..12]);
        arr
    })
    .unwrap_or_else(|_| crate::crypto::generate_nonce());

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

pub fn update_credential(
    conn: &Connection,
    id: i64,
    cred: &UpdateCredential,
) -> Result<Credential> {
    let now = now_iso();

    // Build dynamic update
    if let Some(cid) = cred.category_id {
        conn.execute(
            "UPDATE credentials SET category_id = ?1, updated_at = ?2 WHERE id = ?3",
            params![cid, now, id],
        )?;
    }
    if let Some(ref title) = cred.title {
        conn.execute(
            "UPDATE credentials SET title = ?1, updated_at = ?2 WHERE id = ?3",
            params![title, now, id],
        )?;
    }
    if let Some(ref username) = cred.username {
        conn.execute(
            "UPDATE credentials SET username = ?1, updated_at = ?2 WHERE id = ?3",
            params![username, now, id],
        )?;
    }
    if let Some(ref url) = cred.url {
        conn.execute(
            "UPDATE credentials SET url = ?1, updated_at = ?2 WHERE id = ?3",
            params![url, now, id],
        )?;
    }
    if let Some(ref enc_json) = cred.sensitive_data_json {
        let enc_data = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, enc_json)
            .map_err(|_| rusqlite::Error::InvalidParameterName("invalid base64".to_string()))?;
        // Use provided nonce if present in UpdateCredential (we expect base64), otherwise generate
        let nonce_bytes = if let Some(ref nb64) = cred.nonce_base64 {
            base64::Engine::decode(&base64::engine::general_purpose::STANDARD, nb64)
                .map(|v| {
                    let mut arr = [0u8; 12];
                    arr.copy_from_slice(&v[..12]);
                    arr
                })
                .unwrap_or_else(|_| crate::crypto::generate_nonce())
        } else {
            crate::crypto::generate_nonce()
        };

        conn.execute(
            "UPDATE credentials SET encrypted_data = ?1, nonce = ?2, updated_at = ?3 WHERE id = ?4",
            params![enc_data, nonce_bytes.as_ref(), now, id],
        )?;
    }
    if let Some(ref tags) = cred.tags {
        conn.execute(
            "UPDATE credentials SET tags = ?1, updated_at = ?2 WHERE id = ?3",
            params![tags, now, id],
        )?;
    }
    if let Some(ref notes) = cred.notes {
        conn.execute(
            "UPDATE credentials SET notes = ?1, updated_at = ?2 WHERE id = ?3",
            params![notes, now, id],
        )?;
    }

    get_credential(conn, id)
}

pub fn delete_category(conn: &Connection, id: i64) -> Result<()> {
    // Check if category or its subcategories have credentials
    let has_credentials: bool = conn.query_row(
        "WITH RECURSIVE category_tree(id) AS (
            SELECT id FROM categories WHERE id = ?1
            UNION ALL
            SELECT c.id FROM categories c
            JOIN category_tree ct ON c.parent_id = ct.id
        )
        SELECT EXISTS(SELECT 1 FROM credentials WHERE category_id IN (SELECT id FROM category_tree))",
        params![id],
        |row| row.get(0),
    )?;

    if has_credentials {
        return Err(rusqlite::Error::InvalidParameterName(
            "Cannot delete category: contains credentials".into(),
        ));
    }

    // Check if it has direct children
    let has_children: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM categories WHERE parent_id = ?1)",
        params![id],
        |row| row.get(0),
    )?;

    if has_children {
        return Err(rusqlite::Error::InvalidParameterName(
            "Cannot delete category: has sub-categories".into(),
        ));
    }

    conn.execute("DELETE FROM categories WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn delete_credential(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM credentials WHERE id = ?1", params![id])?;
    Ok(())
}

// ── Sites ─────────────────────────────────────────────────────────────────────────────

fn to_site(row: &rusqlite::Row) -> rusqlite::Result<Site> {
    Ok(Site {
        id: row.get(0)?,
        name: row.get(1)?,
        url: row.get(2)?,
        category_id: row.get(3)?,
        tags: row.get(4)?,
        notes: row.get(5)?,
        created_at: row.get(6)?,
        updated_at: row.get(7)?,
        category_name: row.get(8)?,
        accounts_count: row.get(9)?,
    })
}

pub fn list_sites(conn: &Connection, category_id: Option<i64>) -> Result<Vec<Site>> {
    let sql = if category_id.is_some() {
        "SELECT s.id, s.name, s.url, s.category_id, s.tags, s.notes, s.created_at, s.updated_at,
                c.name, COUNT(sa.id) as accounts_count
         FROM sites s
         LEFT JOIN categories c ON s.category_id = c.id
         LEFT JOIN site_accounts sa ON s.id = sa.site_id
         WHERE s.category_id = ?1
         GROUP BY s.id
         ORDER BY s.updated_at DESC"
    } else {
        "SELECT s.id, s.name, s.url, s.category_id, s.tags, s.notes, s.created_at, s.updated_at,
                c.name, COUNT(sa.id) as accounts_count
         FROM sites s
         LEFT JOIN categories c ON s.category_id = c.id
         LEFT JOIN site_accounts sa ON s.id = sa.site_id
         GROUP BY s.id
         ORDER BY s.updated_at DESC"
    };
    let mut stmt = conn.prepare(sql)?;
    let rows = if let Some(cid) = category_id {
        stmt.query_map(params![cid], to_site)?
    } else {
        stmt.query_map([], to_site)?
    };
    rows.collect()
}

pub fn get_site(conn: &Connection, id: i64) -> Result<SiteDetail> {
    // Get site info
    let site: Site = conn.query_row(
        "SELECT s.id, s.name, s.url, s.category_id, s.tags, s.notes, s.created_at, s.updated_at,
                c.name, COUNT(sa.id) as accounts_count
         FROM sites s
         LEFT JOIN categories c ON s.category_id = c.id
         LEFT JOIN site_accounts sa ON s.id = sa.site_id
         WHERE s.id = ?1
         GROUP BY s.id",
        params![id],
        to_site,
    )?;

    // Get accounts (encrypted)
    let mut stmt = conn.prepare(
        "SELECT username, password_encrypted, password_nonce,
                api_key_encrypted, api_key_nonce,
                secret_key_encrypted, secret_key_nonce
         FROM site_accounts WHERE site_id = ?1"
    )?;

    let accounts_iter = stmt.query_map(params![id], |row| {
        Ok((
            row.get::<_, String>(0)?, // username
            row.get::<_, Vec<u8>>(1)?, // password_encrypted
            row.get::<_, Vec<u8>>(2)?, // password_nonce
            row.get::<_, Option<Vec<u8>>>(3)?, // api_key_encrypted
            row.get::<_, Option<Vec<u8>>>(4)?, // api_key_nonce
            row.get::<_, Option<Vec<u8>>>(5)?, // secret_key_encrypted
            row.get::<_, Option<Vec<u8>>>(6)?, // secret_key_nonce
        ))
    })?;

    // Collect encrypted accounts - will be decrypted in commands layer
    let _encrypted_accounts: Vec<(String, Vec<u8>, Vec<u8>, Option<Vec<u8>>, Option<Vec<u8>>, Option<Vec<u8>>, Option<Vec<u8>>)> =
        accounts_iter.filter_map(|r| r.ok()).collect();

    // Create SiteDetail with placeholder accounts (decryption happens in commands)
    Ok(SiteDetail {
        id: site.id,
        name: site.name,
        url: site.url,
        category_id: site.category_id,
        tags: site.tags,
        notes: site.notes,
        created_at: site.created_at,
        updated_at: site.updated_at,
        category_name: site.category_name,
        accounts: vec![], // Will be populated in commands layer after decryption
    })
}

pub fn create_site(conn: &Connection, name: &str, url: Option<&str>, category_id: i64,
                   tags: Option<&str>, notes: Option<&str>, _accounts_json: &str) -> Result<i64> {
    let now = now_iso();
    conn.execute(
        "INSERT INTO sites (name, url, category_id, tags, notes, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
        params![name, url, category_id, tags, notes, now],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn update_site(conn: &Connection, id: i64, name: Option<&str>, url: Option<&str>,
                   category_id: Option<i64>, tags: Option<&str>, notes: Option<&str>) -> Result<()> {
    let now = now_iso();

    if let Some(n) = name {
        conn.execute(
            "UPDATE sites SET name = ?1, updated_at = ?2 WHERE id = ?3",
            params![n, now, id],
        )?;
    }
    if let Some(u) = url {
        conn.execute(
            "UPDATE sites SET url = ?1, updated_at = ?2 WHERE id = ?3",
            params![u, now, id],
        )?;
    }
    if let Some(cid) = category_id {
        conn.execute(
            "UPDATE sites SET category_id = ?1, updated_at = ?2 WHERE id = ?3",
            params![cid, now, id],
        )?;
    }
    if let Some(t) = tags {
        conn.execute(
            "UPDATE sites SET tags = ?1, updated_at = ?2 WHERE id = ?3",
            params![t, now, id],
        )?;
    }
    if let Some(n) = notes {
        conn.execute(
            "UPDATE sites SET notes = ?1, updated_at = ?2 WHERE id = ?3",
            params![n, now, id],
        )?;
    }

    Ok(())
}

pub fn delete_site(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM sites WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn search_sites(conn: &Connection, query: &str) -> Result<Vec<Site>> {
    let pattern = format!("%{}%", query);
    let mut stmt = conn.prepare(
        "SELECT s.id, s.name, s.url, s.category_id, s.tags, s.notes, s.created_at, s.updated_at,
                c.name, COUNT(sa.id) as accounts_count
         FROM sites s
         LEFT JOIN categories c ON s.category_id = c.id
         LEFT JOIN site_accounts sa ON s.id = sa.site_id
         WHERE s.name LIKE ?1 OR s.url LIKE ?1 OR s.tags LIKE ?1
         GROUP BY s.id
         ORDER BY s.updated_at DESC"
    )?;
    let rows = stmt.query_map(params![pattern], to_site)?;
    rows.collect()
}

// ── Site Accounts ───────────────────────────────────────────────────────────────────

pub fn create_site_account(conn: &Connection, site_id: i64, username: &str,
                          password_encrypted: &[u8], password_nonce: &[u8],
                          api_key_encrypted: Option<&[u8]>, api_key_nonce: Option<&[u8]>,
                          secret_key_encrypted: Option<&[u8]>, secret_key_nonce: Option<&[u8]>) -> Result<i64> {
    let now = now_iso();
    conn.execute(
        "INSERT INTO site_accounts (site_id, username, password_encrypted, password_nonce,
                                   api_key_encrypted, api_key_nonce,
                                   secret_key_encrypted, secret_key_nonce, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![site_id, username, password_encrypted, password_nonce,
               api_key_encrypted, api_key_nonce,
               secret_key_encrypted, secret_key_nonce, now],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn delete_site_accounts(conn: &Connection, site_id: i64) -> Result<()> {
    conn.execute("DELETE FROM site_accounts WHERE site_id = ?1", params![site_id])?;
    Ok(())
}

pub fn get_encrypted_site_accounts(conn: &Connection, site_id: i64) -> Result<Vec<(String, Vec<u8>, Vec<u8>, Option<Vec<u8>>, Option<Vec<u8>>, Option<Vec<u8>>, Option<Vec<u8>>)>> {
    let mut stmt = conn.prepare(
        "SELECT username, password_encrypted, password_nonce,
                api_key_encrypted, api_key_nonce,
                secret_key_encrypted, secret_key_nonce
         FROM site_accounts WHERE site_id = ?1"
    )?;

    let rows = stmt.query_map(params![site_id], |row| {
        Ok((
            row.get::<_, String>(0)?, // username
            row.get::<_, Vec<u8>>(1)?, // password_encrypted
            row.get::<_, Vec<u8>>(2)?, // password_nonce
            row.get::<_, Option<Vec<u8>>>(3)?, // api_key_encrypted
            row.get::<_, Option<Vec<u8>>>(4)?, // api_key_nonce
            row.get::<_, Option<Vec<u8>>>(5)?, // secret_key_encrypted
            row.get::<_, Option<Vec<u8>>>(6)?, // secret_key_nonce
        ))
    })?;

    rows.collect()
}
