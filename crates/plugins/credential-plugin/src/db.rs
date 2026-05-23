//! 凭证管理插件 - 数据库初始化

use rusqlite::{Connection, Result as SqliteResult};
use std::path::Path;

/// 初始化数据库表结构
pub fn init_db(conn: &Connection) -> SqliteResult<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS credentials (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            platform TEXT NOT NULL,
            category TEXT NOT NULL,
            username TEXT NOT NULL,
            password_encrypted TEXT NOT NULL,
            extra_fields TEXT,
            notes TEXT,
            is_active BOOLEAN DEFAULT 1,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            expires_at INTEGER,
            tags TEXT,
            UNIQUE(platform, username)
        );

        CREATE TABLE IF NOT EXISTS audit_logs (
            id TEXT PRIMARY KEY,
            credential_id TEXT NOT NULL,
            action TEXT NOT NULL,
            old_value_hash TEXT,
            new_value_hash TEXT,
            ip_address TEXT,
            timestamp INTEGER NOT NULL,
            result BOOLEAN,
            FOREIGN KEY(credential_id) REFERENCES credentials(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS master_key_config (
            key_version INTEGER PRIMARY KEY,
            derived_from TEXT,
            salt TEXT NOT NULL,
            iv TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            UNIQUE(key_version)
        );

        CREATE INDEX IF NOT EXISTS idx_platform ON credentials(platform);
        CREATE INDEX IF NOT EXISTS idx_category ON credentials(category);
        CREATE INDEX IF NOT EXISTS idx_timestamp ON audit_logs(timestamp DESC);
        CREATE INDEX IF NOT EXISTS idx_credential_id ON audit_logs(credential_id);
        ",
    )?;

    Ok(())
}

/// 确保数据库存在，返回连接
pub fn ensure_db_exists(db_path: &Path) -> SqliteResult<Connection> {
    let conn = Connection::open(db_path)?;
    init_db(&conn)?;
    Ok(conn)
}