# Credential Manager - Rust Backend Spec

## 项目
Oasis Tauri 2 应用，项目根目录: /Users/shaipe/workspace/rust/tools/oasis
Rust 代码目录: src-tauri/crates/credential/

## 概述
凭据管理后端模块，位于 `src-tauri/crates/oasis-credential/`，包括加密、数据库、Tauri commands。
命令注册由 `build.rs` 自动扫描，无需手动注册。

## 目录结构
```
src-tauri/crates/credential/
├── Cargo.toml
└── src/
    ├── lib.rs          # 模块入口，导出子模块
    ├── crypto.rs       # 加密/解密/密钥派生
    ├── db.rs           # SQLite 数据库操作
    ├── models.rs       # 数据结构定义
    └── commands.rs     # Tauri #[command] 接口
```

## 1. 加密模块 (crypto.rs)

### 密钥派生链路
```
用户主密钥 (Master Password string)
  → PBKDF2-SHA256(600,000 rounds, 32-byte random salt)
  → key_hash (用于验证，存 DB)
  → HKDF-SHA256(salt=dek_salt 32-byte, info=b"oasis-credential-key")
  → DEK (32 bytes, base64 编码传给前端)
```

### 函数
```rust
pub fn derive_master_key(password: &str, salt: &[u8]) -> [u8; 32]
pub fn derive_dek(password: &str, dek_salt: &[u8]) -> [u8; 32]
pub fn encrypt(dek: &[u8; 32], plaintext: &[u8]) -> Result<(Vec<u8>, [u8; 12])>
pub fn decrypt(dek: &[u8; 32], ciphertext: &[u8], nonce: &[u8; 12]) -> Result<Vec<u8>>
pub fn generate_salt() -> [u8; 32]
pub fn generate_nonce() -> [u8; 12]
```

使用 `ring` crate:
- `ring::pbkdf2` for PBKDF2
- `ring::hkdf` for HKDF
- `ring::aead` for AES-256-GCM

## 2. 数据库模块 (db.rs)

### 初始化
```rust
pub fn init_db(app_data_dir: &Path) -> Result<rusqlite::Connection>
```
- 创建 `credentials.db` 文件于 Tauri app data 目录
- 执行建表 SQL
- 插入预置分类数据（如果分类表为空）

### Schema
```sql
CREATE TABLE IF NOT EXISTS master_key (
    id            INTEGER PRIMARY KEY CHECK (id = 1),
    key_hash      BLOB NOT NULL,
    salt          BLOB NOT NULL,
    dek_salt      BLOB NOT NULL,
    created_at    TEXT NOT NULL,
    updated_at    TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS categories (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    name          TEXT NOT NULL UNIQUE,
    icon          TEXT,
    sort_order    INTEGER DEFAULT 0,
    created_at    TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS credentials (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    category_id   INTEGER NOT NULL REFERENCES categories(id),
    title         TEXT NOT NULL,
    username      TEXT,
    url           TEXT,
    encrypted_data BLOB NOT NULL,
    nonce         BLOB NOT NULL,
    tags          TEXT,
    notes         TEXT,
    created_at    TEXT NOT NULL,
    updated_at    TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS sites (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    name          TEXT NOT NULL,
    url           TEXT,
    category_id   INTEGER NOT NULL REFERENCES categories(id),
    tags          TEXT,
    notes         TEXT,
    created_at    TEXT NOT NULL,
    updated_at    TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS site_accounts (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    site_id       INTEGER NOT NULL REFERENCES sites(id),
    username      TEXT NOT NULL,
    password      TEXT NOT NULL,
    api_key       TEXT,
    secret_key    TEXT,
    created_at    TEXT NOT NULL
);
```

### 预置分类
社交媒体, 邮箱, 开发工具, API密钥, 云服务, 数据库, 自定义

### CRUD 函数
```rust
// 主密钥
pub fn is_master_key_set(conn: &Connection) -> Result<bool>
pub fn set_master_key(conn: &Connection, key_hash: &[u8], salt: &[u8], dek_salt: &[u8]) -> Result<()>
pub fn verify_master_key(conn: &Connection, key_hash: &[u8]) -> Result<bool>
pub fn get_master_key_salts(conn: &Connection) -> Result<(Vec<u8>, Vec<u8>)>

// 分类
pub fn list_categories(conn: &Connection) -> Result<Vec<Category>>
pub fn create_category(conn: &Connection, name: &str, icon: Option<&str>) -> Result<Category>

// 凭证
pub fn list_credentials(conn: &Connection, category_id: Option<i64>) -> Result<Vec<Credential>>
pub fn get_credential(conn: &Connection, id: i64) -> Result<Credential>
pub fn create_credential(conn: &Connection, cred: &NewCredential) -> Result<Credential>
pub fn update_credential(conn: &Connection, id: i64, cred: &UpdateCredential) -> Result<Credential>
pub fn delete_credential(conn: &Connection, id: i64) -> Result<()>

// 网站账号
pub fn list_sites(conn: &Connection, category_id: Option<i64>) -> Result<Vec<Site>>
pub fn get_site(conn: &Connection, id: i64) -> Result<SiteDetail>
pub fn create_site(conn: &Connection, site: &NewSite) -> Result<Site>
pub fn update_site(conn: &Connection, id: i64, site: &UpdateSite) -> Result<Site>
pub fn delete_site(conn: &Connection, id: i64) -> Result<()>
pub fn search_sites(conn: &Connection, query: &str) -> Result<Vec<Site>>
```

## 3. 数据结构 (models.rs)

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Category {
    pub id: i64,
    pub name: String,
    pub icon: Option<String>,
    pub sort_order: i64,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CredentialView {
    pub id: i64,
    pub category_id: i64,
    pub title: String,
    pub username: Option<String>,
    pub url: Option<String>,
    pub tags: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub category_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CredentialDetail {
    pub id: i64,
    pub category_id: i64,
    pub title: String,
    pub username: Option<String>,
    pub url: Option<String>,
    pub sensitive_data: SensitiveData,
    pub tags: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub category_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SensitiveData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_fields: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Site {
    pub id: i64,
    pub name: String,
    pub url: Option<String>,
    pub category_id: i64,
    pub tags: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SiteDetail {
    #[serde(flatten)]
    pub site: Site,
    pub accounts: Vec<SiteAccount>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SiteAccount {
    pub id: i64,
    pub username: String,
    pub password: String,
    pub api_key: Option<String>,
    pub secret_key: Option<String>,
}
```

## 4. Tauri Commands (commands.rs)

命令由 `build.rs` 自动扫描注册，无需手动在 `lib.rs` 中添加。

```rust
#[tauri::command]
pub fn is_master_key_set(app: tauri::AppHandle) -> Result<bool, String>

#[tauri::command]
pub fn setup_master_key(app: tauri::AppHandle, password: String) -> Result<String, String>
// 返回 DEK base64

#[tauri::command]
pub fn verify_master_key(app: tauri::AppHandle, password: String) -> Result<String, String>
// 验证主密钥，成功返回 DEK base64

#[tauri::command]
pub fn change_master_key(app: tauri::AppHandle, old_password: String, new_password: String) -> Result<String, String>

#[tauri::command]
pub fn list_categories(app: tauri::AppHandle) -> Result<Vec<Category>, String>

#[tauri::command]
pub fn create_category(app: tauri::AppHandle, name: String, icon: Option<String>) -> Result<Category, String>

#[tauri::command]
pub fn list_credentials(app: tauri::AppHandle, category_id: Option<i64>) -> Result<Vec<CredentialView>, String>

#[tauri::command]
pub fn get_credential(app: tauri::AppHandle, id: i64, dek_base64: String) -> Result<CredentialDetail, String>

#[tauri::command]
pub fn create_credential(app: tauri::AppHandle, credential: NewCredential) -> Result<CredentialView, String>

#[tauri::command]
pub fn update_credential(app: tauri::AppHandle, credential: UpdateCredential) -> Result<CredentialView, String>

#[tauri::command]
pub fn delete_credential(app: tauri::AppHandle, id: i64) -> Result<(), String>

#[tauri::command]
pub fn list_sites(app: tauri::AppHandle, category_id: Option<i64>) -> Result<Vec<Site>, String>

#[tauri::command]
pub fn get_site(app: tauri::AppHandle, id: i64, dek_base64: String) -> Result<SiteDetail, String>

#[tauri::command]
pub fn create_site(app: tauri::AppHandle, site: NewSite) -> Result<Site, String>

#[tauri::command]
pub fn update_site(app: tauri::AppHandle, site: UpdateSite) -> Result<Site, String>

#[tauri::command]
pub fn delete_site(app: tauri::AppHandle, id: i64) -> Result<(), String>

#[tauri::command]
pub fn search_sites(app: tauri::AppHandle, query: String) -> Result<Vec<Site>, String>
```

### DB 连接管理
每个 command 中通过 `app.path().app_data_dir()` 获取数据目录，打开 SQLite 连接。不使用全局 state 管理连接。

## 5. 关键注意事项

- **ring crate PBKDF2**: `ring::pbkdf2::derive(PBKDF2_HMAC_SHA256, 600_000.nonzero(), salt, password.as_bytes(), &mut hash)`
- **ring HKDF**: `ring::hkdf::Prk::new_salt(HKDF_SHA256, dek_salt).extract(password_hash).expand(&[b"oasis-credential-key"], DEK_LEN)`
- **ring AES-256-GCM**: 使用 `ring::aead::seal_in_place_separate_tag` / `open_in_place`
- **错误处理**: 所有 command 返回 `Result<T, String>`，用 `.map_err(|e| e.to_string())`
- **时间格式**: SQLite 中用 ISO 8601 字符串
- **命令注册**: 由 `build.rs` 自动扫描，无需手动注册
