//! 凭证管理插件 - 数据模型定义

use serde::{Deserialize, Serialize};

/// 凭证实体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credential {
    pub id: String,
    pub name: String,
    pub credential_type: String,
    pub platform: String,
    pub category: String,
    pub username: String,
    pub password_encrypted: String,
    pub extra_fields: String, // JSON string
    pub notes: String,
    pub is_active: bool,
    pub created_at: i64,
    pub updated_at: i64,
    pub expires_at: Option<i64>,
    pub tags: String, // comma-separated
}

impl Credential {
    pub fn new(
        name: String,
        credential_type: String,
        platform: String,
        category: String,
        username: String,
        password_encrypted: String,
    ) -> Self {
        let now = chrono::Local::now().timestamp();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            credential_type,
            platform,
            category,
            username,
            password_encrypted,
            extra_fields: String::new(),
            notes: String::new(),
            is_active: true,
            created_at: now,
            updated_at: now,
            expires_at: None,
            tags: String::new(),
        }
    }
}

/// 审计日志实体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    pub id: String,
    pub credential_id: String,
    pub action: String, // CREATE, READ, UPDATE, DELETE
    pub old_value_hash: Option<String>,
    pub new_value_hash: Option<String>,
    pub ip_address: String,
    pub timestamp: i64,
    pub result: bool,
}

impl AuditLog {
    pub fn new(credential_id: String, action: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            credential_id,
            action,
            old_value_hash: None,
            new_value_hash: None,
            ip_address: "127.0.0.1".to_string(),
            timestamp: chrono::Local::now().timestamp(),
            result: true,
        }
    }
}

/// 主密钥配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MasterKeyConfig {
    pub key_version: u32,
    pub derived_from: String, // "password"
    pub salt: String,         // base64
    pub iv: String,           // base64
    pub created_at: i64,
}

/// 加密数据
#[derive(Debug, Clone)]
pub struct EncryptedData {
    pub ciphertext: Vec<u8>,
    pub iv: Vec<u8>,
    pub tag: Vec<u8>,
}