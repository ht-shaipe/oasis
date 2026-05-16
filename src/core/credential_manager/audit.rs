use crate::core::credential_manager::models::AuditLog;
use anyhow::Result;
use rusqlite::params;
use std::path::PathBuf;
use std::sync::Mutex;

pub struct AuditService {
    conn: Mutex<rusqlite::Connection>,
}

impl AuditService {
    pub fn new(db_path: &PathBuf) -> Result<Self> {
        let conn = rusqlite::Connection::open(db_path)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn log_action(
        &self,
        credential_id: &str,
        action: &str,
        old_hash: Option<&str>,
        new_hash: Option<&str>,
        success: bool,
    ) -> Result<String> {
        let log = AuditLog {
            id: uuid::Uuid::new_v4().to_string(),
            credential_id: credential_id.to_string(),
            action: action.to_string(),
            old_value_hash: old_hash.map(|s| s.to_string()),
            new_value_hash: new_hash.map(|s| s.to_string()),
            ip_address: "127.0.0.1".to_string(),
            timestamp: chrono::Local::now().timestamp(),
            result: success,
        };

        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO audit_logs (id, credential_id, action, old_value_hash, new_value_hash, ip_address, timestamp, result)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                &log.id,
                &log.credential_id,
                &log.action,
                &log.old_value_hash,
                &log.new_value_hash,
                &log.ip_address,
                log.timestamp,
                log.result,
            ],
        )?;

        Ok(log.id)
    }

    pub fn get_logs(&self, credential_id: &str) -> Result<Vec<AuditLog>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, credential_id, action, old_value_hash, new_value_hash, ip_address, timestamp, result
             FROM audit_logs WHERE credential_id = ? ORDER BY timestamp DESC",
        )?;

        let logs = stmt.query_map(params![credential_id], |row| {
            Ok(AuditLog {
                id: row.get(0)?,
                credential_id: row.get(1)?,
                action: row.get(2)?,
                old_value_hash: row.get(3)?,
                new_value_hash: row.get(4)?,
                ip_address: row.get(5)?,
                timestamp: row.get(6)?,
                result: row.get(7)?,
            })
        })?;

        let mut result = Vec::new();
        for log in logs {
            result.push(log?);
        }

        Ok(result)
    }

    pub fn get_all_logs(&self) -> Result<Vec<AuditLog>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, credential_id, action, old_value_hash, new_value_hash, ip_address, timestamp, result
             FROM audit_logs ORDER BY timestamp DESC",
        )?;

        let logs = stmt.query_map([], |row| {
            Ok(AuditLog {
                id: row.get(0)?,
                credential_id: row.get(1)?,
                action: row.get(2)?,
                old_value_hash: row.get(3)?,
                new_value_hash: row.get(4)?,
                ip_address: row.get(5)?,
                timestamp: row.get(6)?,
                result: row.get(7)?,
            })
        })?;

        let mut result = Vec::new();
        for log in logs {
            result.push(log?);
        }

        Ok(result)
    }

    pub fn get_logs_by_action(&self, action: &str) -> Result<Vec<AuditLog>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, credential_id, action, old_value_hash, new_value_hash, ip_address, timestamp, result
             FROM audit_logs WHERE action = ? ORDER BY timestamp DESC",
        )?;

        let logs = stmt.query_map(params![action], |row| {
            Ok(AuditLog {
                id: row.get(0)?,
                credential_id: row.get(1)?,
                action: row.get(2)?,
                old_value_hash: row.get(3)?,
                new_value_hash: row.get(4)?,
                ip_address: row.get(5)?,
                timestamp: row.get(6)?,
                result: row.get(7)?,
            })
        })?;

        let mut result = Vec::new();
        for log in logs {
            result.push(log?);
        }

        Ok(result)
    }

    pub fn delete_logs_before(&self, timestamp: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM audit_logs WHERE timestamp < ?",
            params![timestamp],
        )?;
        Ok(())
    }
}
